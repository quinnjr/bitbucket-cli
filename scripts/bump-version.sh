#!/usr/bin/env bash
# Synchronize the version from Cargo.toml into every file that hardcodes it.
#
# Usage:
#   ./scripts/bump-version.sh           # propagate the current Cargo.toml version
#   ./scripts/bump-version.sh 0.5.0     # set Cargo.toml to 0.5.0 first, then propagate
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_toml="$repo_root/Cargo.toml"

if [[ $# -gt 1 ]]; then
    echo "usage: $0 [X.Y.Z]" >&2
    exit 1
fi

if [[ $# -eq 1 ]]; then
    new_version="$1"
    if [[ ! "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "error: '$new_version' is not a valid X.Y.Z version" >&2
        exit 1
    fi
    if ! grep -q '^version = "[0-9][0-9a-zA-Z.+-]*"$' "$cargo_toml"; then
        echo "error: no 'version = \"...\"' line found in $cargo_toml" >&2
        exit 1
    fi
    sed -i "0,/^version = \"[0-9][0-9a-zA-Z.+-]*\"$/s//version = \"$new_version\"/" "$cargo_toml"
    echo "updated: $cargo_toml"
fi

version="$(grep -m1 '^version = ' "$cargo_toml" | sed 's/^version = "\(.*\)"$/\1/')"
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "error: could not read a valid version from $cargo_toml (got '$version')" >&2
    exit 1
fi

echo "canonical version (Cargo.toml): $version"

# bump <file> <grep pattern for the current value> <sed substitution>
bump() {
    local file="$1" pattern="$2" substitution="$3"
    if [[ ! -f "$file" ]]; then
        echo "error: expected file not found: $file" >&2
        exit 1
    fi
    if ! grep -qE "$pattern" "$file"; then
        echo "error: expected version pattern not found in $file (looked for: $pattern)" >&2
        exit 1
    fi
    local before after
    before="$(<"$file")"
    after="$(sed -E "$substitution" "$file")"
    if [[ "$after" != "$before" ]]; then
        printf '%s\n' "$after" > "$file"
        echo "updated: $file"
    else
        echo "already current: $file"
    fi
}

ver_re='[0-9]+\.[0-9]+\.[0-9]+'

bump "$repo_root/packaging/windows/main.wxs" \
    "Version=\"$ver_re\"" \
    "s/Version=\"$ver_re\"/Version=\"$version\"/"

bump "$repo_root/packaging/arch/PKGBUILD" \
    "^pkgver=$ver_re$" \
    "s/^pkgver=$ver_re$/pkgver=$version/"

bump "$repo_root/packaging/alpine/APKBUILD" \
    "^pkgver=$ver_re$" \
    "s/^pkgver=$ver_re$/pkgver=$version/"

bump "$repo_root/docs/src/app/components/header/header.component.ts" \
    "v$ver_re" \
    "s/v$ver_re/v$version/"

bump "$repo_root/docs/src/app/pages/installation/installation.component.ts" \
    "bitbucket $ver_re" \
    "s/bitbucket $ver_re/bitbucket $version/"

echo "done: all files at $version"
