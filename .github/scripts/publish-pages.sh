#!/usr/bin/env bash
# Usage: publish-pages.sh <dest> <src-dir|""> <commit message>
#
# Replaces <dest> on the gh-pages branch with the contents of <src-dir> (or removes
# it when <src-dir> is empty). <dest> "." means the site root; the root deploy keeps
# the pr/ previews. Retries on push races with other deploys.
set -euo pipefail

dest="$1"
src="$2"
msg="$3"
src_abs=""
if [ -n "$src" ]; then src_abs="$(cd "$src" && pwd)"; fi

repo_dir="$(pwd)"
remote="$(git config --get remote.origin.url)"
work="$(mktemp -d)"
cd "$work"
git init -q
git remote add origin "$remote"
# Reuse the checkout's credentials (actions/checkout stores an auth header).
auth="$(git -C "$repo_dir" config --get http.https://github.com/.extraheader || true)"
if [ -n "$auth" ]; then git config http.https://github.com/.extraheader "$auth"; fi
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

for attempt in 1 2 3 4 5; do
  if git fetch -q --depth 1 origin gh-pages; then
    git checkout -q -B gh-pages FETCH_HEAD
  elif [ -z "$src_abs" ]; then
    echo "No gh-pages branch; nothing to remove."
    exit 0
  else
    git checkout -q --orphan gh-pages
    git rm -rqf . 2>/dev/null || true
  fi

  if [ "$dest" = "." ]; then
    find . -mindepth 1 -maxdepth 1 ! -name .git ! -name pr -exec rm -rf {} +
  else
    rm -rf "$dest"
  fi
  if [ -n "$src_abs" ]; then
    mkdir -p "$dest"
    cp -R "$src_abs"/. "$dest"/
  fi
  touch .nojekyll

  git add -A
  if git diff --cached --quiet; then
    echo "Nothing to publish."
    exit 0
  fi
  git commit -qm "$msg"
  if git push -q origin gh-pages; then
    echo "Pushed: $msg"
    exit 0
  fi
  echo "Push failed (attempt $attempt); retrying."
  sleep $((attempt * 3))
done
echo "Could not push to gh-pages." >&2
exit 1
