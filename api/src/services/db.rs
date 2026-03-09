use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};
use zeroize::Zeroizing;

use crate::entity::{admin_user, api_key, refresh_token};
use crate::error::AppError;

fn now_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// --- Secret loading from env ---

pub fn load_secret_from_env(env_var: &str) -> Zeroizing<Vec<u8>> {
    match std::env::var(env_var) {
        Ok(hex) if !hex.is_empty() => {
            let bytes =
                hex_to_bytes(&hex).unwrap_or_else(|e| panic!("{env_var} is not valid hex: {e}"));
            if bytes.len() != 32 {
                panic!(
                    "{env_var} must be 32 bytes (64 hex chars), got {}",
                    bytes.len()
                );
            }
            tracing::info!("{env_var} loaded from environment");
            Zeroizing::new(bytes)
        }
        _ => {
            panic!(
                "{env_var} is not set. This is required.\n\
                 Generate one with: openssl rand -hex 32\n\
                 Then add it to your .env file."
            );
        }
    }
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Odd-length hex string".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_to_bytes_valid() {
        assert_eq!(hex_to_bytes("abcd").unwrap(), vec![0xab, 0xcd]);
    }

    #[test]
    fn hex_to_bytes_empty() {
        assert_eq!(hex_to_bytes("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn hex_to_bytes_full_32_bytes() {
        let hex = "00".repeat(32);
        let result = hex_to_bytes(&hex).unwrap();
        assert_eq!(result.len(), 32);
        assert!(result.iter().all(|&b| b == 0));
    }

    #[test]
    fn hex_to_bytes_odd_length() {
        assert!(hex_to_bytes("abc").is_err());
    }

    #[test]
    fn hex_to_bytes_invalid_chars() {
        assert!(hex_to_bytes("gg").is_err());
    }

    #[test]
    fn hex_to_bytes_uppercase() {
        assert_eq!(hex_to_bytes("ABCD").unwrap(), vec![0xab, 0xcd]);
    }

    #[test]
    fn hex_to_bytes_mixed_case() {
        assert_eq!(hex_to_bytes("aBcD").unwrap(), vec![0xab, 0xcd]);
    }

    #[test]
    #[should_panic(expected = "is not set")]
    fn load_secret_missing_env_var() {
        load_secret_from_env("OPENPOSTERDB_TEST_NONEXISTENT_SECRET_VAR");
    }

    #[test]
    #[should_panic(expected = "must be 32 bytes")]
    fn load_secret_wrong_length() {
        let var_name = "OPENPOSTERDB_TEST_SHORT_SECRET";
        unsafe { std::env::set_var(var_name, "abcd") };
        let result = std::panic::catch_unwind(|| load_secret_from_env(var_name));
        unsafe { std::env::remove_var(var_name) };
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn load_secret_valid_32_bytes() {
        let var_name = "OPENPOSTERDB_TEST_VALID_SECRET";
        let hex = "ab".repeat(32);
        unsafe { std::env::set_var(var_name, &hex) };
        let secret = load_secret_from_env(var_name);
        unsafe { std::env::remove_var(var_name) };
        assert_eq!(secret.len(), 32);
    }
}

// --- Admin user CRUD ---

pub async fn count_admin_users(db: &impl ConnectionTrait) -> Result<u64, AppError> {
    use sea_orm::PaginatorTrait;
    admin_user::Entity::find()
        .count(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))
}

pub async fn create_admin_user(
    db: &impl ConnectionTrait,
    username: &str,
    password_hash: &str,
) -> Result<admin_user::Model, AppError> {
    let model = admin_user::ActiveModel {
        id: Default::default(),
        username: Set(username.to_owned()),
        password_hash: Set(password_hash.to_owned()),
        created_at: Set(now_utc()),
    };

    let result = admin_user::Entity::insert(model)
        .exec(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    admin_user::Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?
        .ok_or_else(|| AppError::DbError("Failed to retrieve created user".into()))
}

pub async fn create_first_admin_user(
    db: &DatabaseConnection,
    username: &str,
    password_hash: &str,
) -> Result<admin_user::Model, AppError> {
    use sea_orm::PaginatorTrait;

    let txn = db
        .begin()
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    let count = admin_user::Entity::find()
        .count(&txn)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    if count > 0 {
        txn.rollback()
            .await
            .map_err(|e| AppError::DbError(e.to_string()))?;
        return Err(AppError::Forbidden("Setup already completed".into()));
    }

    let model = admin_user::ActiveModel {
        id: Default::default(),
        username: Set(username.to_owned()),
        password_hash: Set(password_hash.to_owned()),
        created_at: Set(now_utc()),
    };

    let result = admin_user::Entity::insert(model)
        .exec(&txn)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    let user = admin_user::Entity::find_by_id(result.last_insert_id)
        .one(&txn)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?
        .ok_or_else(|| AppError::DbError("Failed to retrieve created user".into()))?;

    txn.commit()
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    Ok(user)
}

pub async fn find_admin_user_by_username(
    db: &impl ConnectionTrait,
    username: &str,
) -> Result<Option<admin_user::Model>, AppError> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    admin_user::Entity::find()
        .filter(admin_user::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))
}

pub async fn find_admin_user_by_id(
    db: &impl ConnectionTrait,
    id: i32,
) -> Result<Option<admin_user::Model>, AppError> {
    admin_user::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))
}

// --- Refresh token CRUD ---

pub async fn create_refresh_token(
    db: &impl ConnectionTrait,
    user_id: i32,
    token_hash: &str,
    expires_at: &str,
) -> Result<refresh_token::Model, AppError> {
    let model = refresh_token::ActiveModel {
        id: Default::default(),
        user_id: Set(user_id),
        token_hash: Set(token_hash.to_owned()),
        expires_at: Set(expires_at.to_owned()),
        created_at: Set(now_utc()),
    };

    let result = refresh_token::Entity::insert(model)
        .exec(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    refresh_token::Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?
        .ok_or_else(|| AppError::DbError("Failed to retrieve created refresh token".into()))
}

pub async fn find_refresh_token_by_hash(
    db: &impl ConnectionTrait,
    token_hash: &str,
) -> Result<Option<refresh_token::Model>, AppError> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    refresh_token::Entity::find()
        .filter(refresh_token::Column::TokenHash.eq(token_hash))
        .one(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))
}

pub async fn delete_refresh_token(db: &impl ConnectionTrait, id: i32) -> Result<(), AppError> {
    refresh_token::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn delete_refresh_tokens_for_user(
    db: &impl ConnectionTrait,
    user_id: i32,
) -> Result<(), AppError> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    refresh_token::Entity::delete_many()
        .filter(refresh_token::Column::UserId.eq(user_id))
        .exec(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    Ok(())
}

pub async fn delete_expired_refresh_tokens(db: &impl ConnectionTrait) -> Result<u64, AppError> {
    let now = now_utc();
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    let result = refresh_token::Entity::delete_many()
        .filter(refresh_token::Column::ExpiresAt.lt(now))
        .exec(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    Ok(result.rows_affected)
}

// --- API key CRUD ---

pub async fn create_api_key(
    db: &impl ConnectionTrait,
    name: &str,
    key_hash: &str,
    key_prefix: &str,
    created_by: i32,
) -> Result<api_key::Model, AppError> {
    let model = api_key::ActiveModel {
        id: Default::default(),
        name: Set(name.to_owned()),
        key_hash: Set(key_hash.to_owned()),
        key_prefix: Set(key_prefix.to_owned()),
        created_by: Set(created_by),
        created_at: Set(now_utc()),
        last_used_at: Set(None),
    };

    let result = api_key::Entity::insert(model)
        .exec(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;

    api_key::Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?
        .ok_or_else(|| AppError::DbError("Failed to retrieve created API key".into()))
}

pub async fn find_api_key_by_hash(
    db: &impl ConnectionTrait,
    key_hash: &str,
) -> Result<Option<api_key::Model>, AppError> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    api_key::Entity::find()
        .filter(api_key::Column::KeyHash.eq(key_hash))
        .one(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))
}

pub async fn list_api_keys(db: &impl ConnectionTrait) -> Result<Vec<api_key::Model>, AppError> {
    api_key::Entity::find()
        .all(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))
}

pub async fn delete_api_key(db: &impl ConnectionTrait, id: i32) -> Result<(), AppError> {
    api_key::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;
    Ok(())
}

// --- Poster meta queries ---

pub async fn count_poster_meta(db: &impl ConnectionTrait) -> Result<u64, AppError> {
    use crate::entity::poster_meta;
    use sea_orm::PaginatorTrait;
    poster_meta::Entity::find()
        .count(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))
}

pub async fn count_api_keys(db: &impl ConnectionTrait) -> Result<u64, AppError> {
    use sea_orm::PaginatorTrait;
    api_key::Entity::find()
        .count(db)
        .await
        .map_err(|e| AppError::DbError(e.to_string()))
}

pub async fn list_poster_meta(
    db: &impl ConnectionTrait,
    page: u64,
    page_size: u64,
) -> Result<(Vec<crate::entity::poster_meta::Model>, u64), AppError> {
    use crate::entity::poster_meta;
    use sea_orm::PaginatorTrait;
    let paginator = poster_meta::Entity::find().paginate(db, page_size);
    let total = paginator.num_items().await.map_err(|e| AppError::DbError(e.to_string()))?;
    let items = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .map_err(|e| AppError::DbError(e.to_string()))?;
    Ok((items, total))
}

pub async fn batch_update_last_used(
    db: &impl ConnectionTrait,
    ids: &[i32],
) -> Result<(), AppError> {
    if ids.is_empty() {
        return Ok(());
    }
    let now = now_utc();
    use sea_orm::{ColumnTrait, QueryFilter, sea_query::Expr};
    for chunk in ids.chunks(100) {
        api_key::Entity::update_many()
            .col_expr(api_key::Column::LastUsedAt, Expr::value(now.clone()))
            .filter(api_key::Column::Id.is_in(chunk.iter().copied()))
            .exec(db)
            .await
            .map_err(|e| AppError::DbError(e.to_string()))?;
    }
    Ok(())
}

