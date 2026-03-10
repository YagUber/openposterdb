#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
    repair)
        TAG=$(gh release list --limit 1 --json tagName --jq '.[0].tagName')
        echo "Re-triggering release for $TAG..."
        gh release delete "$TAG" --yes
        git tag -f "$TAG"
        git push origin "$TAG" --force
        gh release create "$TAG" --generate-notes
        echo "Re-created release $TAG"
        exit 0
        ;;
    ""|--help|-h)
        echo "Usage: $0 <version>    Create a new release" >&2
        echo "       $0 repair       Re-trigger the last release workflow" >&2
        echo "Example: $0 1.0.0" >&2
        exit 1
        ;;
esac

VERSION="${1#v}"
TAG="v$VERSION"

# Ensure working tree is clean
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Error: working tree is not clean. Commit or stash changes first." >&2
    exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"

# Update Cargo.toml version
sed -i "0,/^version = \".*\"/s//version = \"$VERSION\"/" "$REPO_ROOT/api/Cargo.toml"

# Update Cargo.lock
(cd "$REPO_ROOT/api" && cargo update --workspace)

# Update package.json version
cd "$REPO_ROOT/web" && npm version "$VERSION" --no-git-tag-version
cd "$REPO_ROOT"

git add "$REPO_ROOT/api/Cargo.toml" "$REPO_ROOT/api/Cargo.lock" "$REPO_ROOT/web/package.json" "$REPO_ROOT/web/package-lock.json"
git commit -m "release: $TAG"
git push origin main
gh release create "$TAG" --generate-notes

echo "Released $TAG"
