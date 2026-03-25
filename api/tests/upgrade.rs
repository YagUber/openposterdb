mod common;

use sea_orm::{ConnectionTrait, DatabaseConnection, SqlxSqliteConnector};

async fn setup_db() -> DatabaseConnection {
    let sqlite_opts = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true)
        .pragma("foreign_keys", "ON");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlite_opts)
        .await
        .expect("failed to connect to test database");
    let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);

    for sql in openposterdb_api::SCHEMA_SQL {
        db.execute_unprepared(sql)
            .await
            .expect("failed to create table");
    }
    for (sql, expected_err) in openposterdb_api::MIGRATIONS {
        match db.execute_unprepared(sql).await {
            Ok(_) => {}
            Err(e) if e.to_string().to_lowercase().contains(expected_err) => {}
            Err(e) => panic!("migration failed: {e}\n  SQL: {sql}"),
        }
    }

    db
}

async fn count_upgrades(db: &DatabaseConnection, name: &str) -> u64 {
    let row = db
        .query_one(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT count(*) as cnt FROM upgrades WHERE name = ?",
            [name.into()],
        ))
        .await
        .unwrap();
    match row {
        Some(r) => {
            use sea_orm::QueryResult;
            r.try_get::<i32>("", "cnt").unwrap() as u64
        }
        None => 0,
    }
}

// --- upgrade framework tests ---

#[tokio::test]
async fn upgrade_run_creates_upgrades_table_and_records_completion() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();

    openposterdb_api::upgrade::run(&db, dir.path().to_str().unwrap(), false)
        .await
        .expect("upgrade::run should succeed");

    // The upgrades table should exist and have an entry
    let count = count_upgrades(&db, "v001_backdrop_cache_keys").await;
    assert_eq!(count, 1, "v001 should be recorded as completed");
}

#[tokio::test]
async fn upgrade_run_is_idempotent() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path().to_str().unwrap();

    openposterdb_api::upgrade::run(&db, cache_dir, false)
        .await
        .expect("first run should succeed");
    openposterdb_api::upgrade::run(&db, cache_dir, false)
        .await
        .expect("second run should succeed (idempotent)");

    let count = count_upgrades(&db, "v001_backdrop_cache_keys").await;
    assert_eq!(count, 1, "should still have exactly one record");
}

// --- v001 backdrop cache key migration: DB ---

#[tokio::test]
async fn v001_renames_backdrop_cache_keys_in_db() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();

    // Insert rows with old-style _b@ keys
    db.execute_unprepared(
        "INSERT INTO image_meta (cache_key, image_type, created_at, updated_at) VALUES
         ('imdb/tt1234567_b@abc', 'b', 1000, 1000),
         ('imdb/tt7654321_b@def', 'b', 1000, 1000),
         ('imdb/tt0000001_p@ghi', 'poster', 1000, 1000)",
    )
    .await
    .unwrap();

    openposterdb_api::upgrade::run(&db, dir.path().to_str().unwrap(), false)
        .await
        .expect("upgrade should succeed");

    // Backdrop keys should be renamed
    let row = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT cache_key FROM image_meta WHERE cache_key = 'imdb/tt1234567_b_f@abc'".to_string(),
        ))
        .await
        .unwrap();
    assert!(row.is_some(), "first backdrop key should be renamed to _b_f@");

    let row = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT cache_key FROM image_meta WHERE cache_key = 'imdb/tt7654321_b_f@def'".to_string(),
        ))
        .await
        .unwrap();
    assert!(row.is_some(), "second backdrop key should be renamed to _b_f@");

    // Poster key should be untouched
    let row = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT cache_key FROM image_meta WHERE cache_key = 'imdb/tt0000001_p@ghi'".to_string(),
        ))
        .await
        .unwrap();
    assert!(row.is_some(), "poster key should be untouched");
}

// --- v001 backdrop cache key migration: filesystem ---

#[tokio::test]
async fn v001_renames_backdrop_files() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path();

    let backdrop_dir = cache_dir.join("backdrops");
    std::fs::create_dir_all(&backdrop_dir).unwrap();

    // Create files with old naming
    std::fs::write(backdrop_dir.join("tt1234567_b@abc.jpg"), b"data1").unwrap();
    std::fs::write(backdrop_dir.join("tt7654321_b@def.jpg"), b"data2").unwrap();
    // Already migrated file — should not be touched
    std::fs::write(backdrop_dir.join("tt0000001_b_f@ghi.jpg"), b"data3").unwrap();
    // TMDB-sourced file — should not be touched
    std::fs::write(backdrop_dir.join("tt9999999_b_t@jkl.jpg"), b"data4").unwrap();

    openposterdb_api::upgrade::run(&db, cache_dir.to_str().unwrap(), false)
        .await
        .expect("upgrade should succeed");

    // Old files should be renamed
    assert!(
        backdrop_dir.join("tt1234567_b_f@abc.jpg").exists(),
        "file should be renamed from _b@ to _b_f@"
    );
    assert!(
        backdrop_dir.join("tt7654321_b_f@def.jpg").exists(),
        "file should be renamed from _b@ to _b_f@"
    );
    assert!(
        !backdrop_dir.join("tt1234567_b@abc.jpg").exists(),
        "old file should no longer exist"
    );

    // Already-migrated files untouched
    assert!(backdrop_dir.join("tt0000001_b_f@ghi.jpg").exists());
    assert!(backdrop_dir.join("tt9999999_b_t@jkl.jpg").exists());
}

#[tokio::test]
async fn v001_handles_missing_backdrop_directory() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();
    // Don't create the backdrops subdirectory — should not error
    openposterdb_api::upgrade::run(&db, dir.path().to_str().unwrap(), false)
        .await
        .expect("upgrade should succeed even without backdrops dir");
}

#[tokio::test]
async fn v001_renames_files_in_subdirectories() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path();

    let sub_dir = cache_dir.join("backdrops").join("subdir");
    std::fs::create_dir_all(&sub_dir).unwrap();
    std::fs::write(sub_dir.join("tt5555555_b@nested.jpg"), b"nested").unwrap();

    openposterdb_api::upgrade::run(&db, cache_dir.to_str().unwrap(), false)
        .await
        .expect("upgrade should succeed");

    assert!(
        sub_dir.join("tt5555555_b_f@nested.jpg").exists(),
        "nested file should be renamed"
    );
    assert!(
        !sub_dir.join("tt5555555_b@nested.jpg").exists(),
        "old nested file should be gone"
    );
}

#[tokio::test]
async fn v001_skips_filesystem_when_external_cache_only() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path();

    let backdrop_dir = cache_dir.join("backdrops");
    std::fs::create_dir_all(&backdrop_dir).unwrap();
    std::fs::write(backdrop_dir.join("tt1234567_b@abc.jpg"), b"data").unwrap();

    openposterdb_api::upgrade::run(&db, cache_dir.to_str().unwrap(), true)
        .await
        .expect("upgrade should succeed");

    // File should NOT be renamed when external_cache_only is true
    assert!(
        backdrop_dir.join("tt1234567_b@abc.jpg").exists(),
        "file should be untouched in external_cache_only mode"
    );
    assert!(
        !backdrop_dir.join("tt1234567_b_f@abc.jpg").exists(),
        "renamed file should not exist in external_cache_only mode"
    );
}

// --- v002 backdrop position/direction cache key migration: DB ---

#[tokio::test]
async fn v002_inserts_position_and_direction_in_db() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();

    // Insert rows with old-style backdrop cache keys (no position/direction suffix)
    db.execute_unprepared(
        "INSERT INTO image_meta (cache_key, image_type, created_at, updated_at) VALUES
         ('imdb/tt1234567_b_f@mil.sv.lt.bm.zm', 'b', 1000, 1000),
         ('imdb/tt7654321_b_t@ir.sh.li.bl.zl', 'b', 1000, 1000),
         ('imdb/tt0000001_p@mil.sv.lt.bm.zm', 'poster', 1000, 1000)"
    )
    .await
    .unwrap();

    openposterdb_api::upgrade::run(&db, dir.path().to_str().unwrap(), false)
        .await
        .expect("upgrade should succeed");

    // Backdrop keys should have position and direction inserted
    let row = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT cache_key FROM image_meta WHERE cache_key = 'imdb/tt1234567_b_f@mil.ptr.sv.lt.dv.bm.zm'".to_string(),
        ))
        .await
        .unwrap();
    assert!(row.is_some(), "first backdrop key should have .ptr and .dv inserted");

    let row = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT cache_key FROM image_meta WHERE cache_key = 'imdb/tt7654321_b_t@ir.ptr.sh.li.dv.bl.zl'".to_string(),
        ))
        .await
        .unwrap();
    assert!(row.is_some(), "second backdrop key should have .ptr and .dv inserted");

    // Poster key should be untouched
    let row = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT cache_key FROM image_meta WHERE cache_key = 'imdb/tt0000001_p@mil.sv.lt.bm.zm'".to_string(),
        ))
        .await
        .unwrap();
    assert!(row.is_some(), "poster key should be untouched");
}

#[tokio::test]
async fn v002_db_is_idempotent() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();

    db.execute_unprepared(
        "INSERT INTO image_meta (cache_key, image_type, created_at, updated_at) VALUES
         ('imdb/tt1234567_b_f@mil.sv.lt.bm.zm', 'b', 1000, 1000)"
    )
    .await
    .unwrap();

    let cache_dir = dir.path().to_str().unwrap();
    openposterdb_api::upgrade::run(&db, cache_dir, false)
        .await
        .expect("first run should succeed");
    openposterdb_api::upgrade::run(&db, cache_dir, false)
        .await
        .expect("second run should succeed (idempotent)");

    let row = db
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT cache_key FROM image_meta WHERE cache_key = 'imdb/tt1234567_b_f@mil.ptr.sv.lt.dv.bm.zm'".to_string(),
        ))
        .await
        .unwrap();
    assert!(row.is_some(), "key should be migrated exactly once");
}

// --- v002 backdrop position/direction cache key migration: filesystem ---

#[tokio::test]
async fn v002_renames_backdrop_files() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path();

    let backdrop_dir = cache_dir.join("backdrops").join("imdb");
    std::fs::create_dir_all(&backdrop_dir).unwrap();

    // Old-style filenames
    std::fs::write(backdrop_dir.join("tt1234567_b_f@mil.sv.lt.bm.zm.jpg"), b"data1").unwrap();
    std::fs::write(backdrop_dir.join("tt7654321_b_t@ir.sh.li.bl.zl.jpg"), b"data2").unwrap();
    // Already migrated — should not be touched
    std::fs::write(backdrop_dir.join("tt0000001_b_f@mil.ptr.sv.lt.dv.bm.zm.jpg"), b"data3").unwrap();

    openposterdb_api::upgrade::run(&db, cache_dir.to_str().unwrap(), false)
        .await
        .expect("upgrade should succeed");

    assert!(
        backdrop_dir.join("tt1234567_b_f@mil.ptr.sv.lt.dv.bm.zm.jpg").exists(),
        "first file should be renamed with .ptr and .dv"
    );
    assert!(
        backdrop_dir.join("tt7654321_b_t@ir.ptr.sh.li.dv.bl.zl.jpg").exists(),
        "second file should be renamed with .ptr and .dv"
    );
    assert!(
        !backdrop_dir.join("tt1234567_b_f@mil.sv.lt.bm.zm.jpg").exists(),
        "old first file should no longer exist"
    );
    assert!(
        backdrop_dir.join("tt0000001_b_f@mil.ptr.sv.lt.dv.bm.zm.jpg").exists(),
        "already-migrated file should be untouched"
    );
}

#[tokio::test]
async fn v002_skips_filesystem_when_external_cache_only() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path();

    let backdrop_dir = cache_dir.join("backdrops").join("imdb");
    std::fs::create_dir_all(&backdrop_dir).unwrap();
    std::fs::write(backdrop_dir.join("tt1234567_b_f@mil.sv.lt.bm.zm.jpg"), b"data").unwrap();

    openposterdb_api::upgrade::run(&db, cache_dir.to_str().unwrap(), true)
        .await
        .expect("upgrade should succeed");

    assert!(
        backdrop_dir.join("tt1234567_b_f@mil.sv.lt.bm.zm.jpg").exists(),
        "file should be untouched in external_cache_only mode"
    );
}

#[tokio::test]
async fn v002_handles_missing_backdrop_directory() {
    let db = setup_db().await;
    let dir = tempfile::tempdir().unwrap();
    // Don't create the backdrops subdirectory — should not error
    openposterdb_api::upgrade::run(&db, dir.path().to_str().unwrap(), false)
        .await
        .expect("upgrade should succeed even without backdrops dir");
}
