#!/bin/sh
set -e

# Ensure data directories exist and are writable by the opdb user.
# Bind mounts (e.g. Unraid) may be owned by a different uid/gid,
# so we chown them before dropping privileges.
for dir in "${CACHE_DIR:-/data/cache}" "${DB_DIR:-/data/db}"; do
    mkdir -p "$dir"
    chown opdb:opdb "$dir"
done

exec su -s /bin/sh opdb -c '"$0" "$@"' -- "$@"
