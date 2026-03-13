# Scripts

Utility scripts for developing, testing, releasing, and seeding OpenPosterDB.

## seed.sh

Warms the OpenPosterDB cache by requesting posters for titles from the IMDB dataset. Entries are processed newest-first, using `endYear` for series (if available) and `startYear` otherwise.

Requires `title.basics.tsv` in the scripts directory. Download it from <https://datasets.imdbws.com/title.basics.tsv.gz> and extract.

```bash
./scripts/seed.sh <BASE_URL> [OPTIONS]
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-n, --limit NUM` | Total entries to seed (0 = unlimited) | `100` |
| `-N, --limit-per-type NUM` | Cap each type independently (e.g. 100k movies + 100k series) | none |
| `-t, --type TYPE` | `movie`, `tv`, or `both` | `both` |
| `-g, --genres GENRES` | Comma-separated genres to include (e.g. `"Action,Horror"`) | all |
| `-f, --year-from YEAR` | Minimum year (inclusive) | none |
| `-y, --year-to YEAR` | Maximum year (inclusive) | none |
| `-k, --key KEY` | API key to use | `t0-free-rpdb` |
| `-a, --assets ASSETS` | `poster`, `logo`, `backdrop`, or `all` | `poster` |
| `-d, --dry-run` | Print matching titles without making requests | off |

### Examples

```bash
# Seed the 100 newest titles (default)
./scripts/seed.sh http://localhost:3000

# Seed 500 movies from 2000 onwards
./scripts/seed.sh http://localhost:3000 -n 500 -t movie -f 2000

# Seed all horror and thriller titles, including logos and backdrops
./scripts/seed.sh http://localhost:3000 -n 0 -g "Horror,Thriller" -a all

# Preview what TV series from 2015-2023 would be seeded
./scripts/seed.sh http://localhost:3000 -t tv -f 2015 -y 2023 -d

# Seed 100k movies + 100k series in one run
./scripts/seed.sh http://localhost:3000 -N 100000


```

### Data files

| File | Description |
|------|-------------|
| `title.basics.tsv` | Extracted IMDB dataset used by the seed script. Not committed to git. |
| `imdb_ids.txt` | Plain list of all IMDB IDs (one per line, sorted). |

---

## test.sh

Runs the full test suite: backend (Rust), frontend unit tests (Vitest), and end-to-end tests (Playwright).

```bash
./scripts/test.sh
```

### What it does

1. Runs `cargo test` in `api/`
2. Runs `npx vitest run` in `web/`
3. Builds a container image with the `test-support` feature flag
4. Starts the container on port `3333`, loading API keys from `api/.env`
5. Waits for the backend to become healthy (up to 60s)
6. Runs Playwright E2E tests (`setup`, `settings`, `chromium`, `live` projects)
7. Tears down the container on exit

Requires either `podman` or `docker`.

---

## release.sh

Creates a new tagged release and pushes it to GitHub.

```bash
./scripts/release.sh <VERSION>
./scripts/release.sh repair
```

### Create a release

```bash
./scripts/release.sh 1.0.0
```

1. Checks that the working tree is clean
2. Updates the version in `api/Cargo.toml` and `web/package.json`
3. Updates `Cargo.lock` and `package-lock.json`
4. Commits, pushes to `main`, and creates a GitHub release with auto-generated notes

The `v` prefix is added automatically — pass `1.0.0`, not `v1.0.0`.

### Repair a release

```bash
./scripts/release.sh repair
```

Re-triggers the release workflow for the most recent tag by deleting and recreating the GitHub release. Useful when CI failed on the initial release.

---

## regenerate-examples.sh

Fetches example poster, logo, and backdrop images for a set of classic films and saves them to `web/public/examples/`. These are used in the web UI.

```bash
./scripts/regenerate-examples.sh [BASE_URL]
```

Defaults to `http://localhost:3000`. Uses the free API key. Fetches assets for:

- Nosferatu (`tt0013442`)
- Metropolis (`tt0017136`)
- The Cabinet of Dr. Caligari (`tt0010323`)
- The Phantom of the Opera (`tt0016220`)
- A Trip to the Moon (`tt0000417`)
- Safety Last! (`tt0014429`)
- The General (`tt0017925`)

Logos and backdrops that aren't available are silently skipped.
