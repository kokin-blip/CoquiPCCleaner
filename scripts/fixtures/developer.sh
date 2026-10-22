#!/bin/sh
# Creates only new disposable projects. Prints the selected scan root.
set -eu
fixture_dir=$(mktemp -d /private/tmp/coqui-developer.XXXXXX)
git_fixture() {
  env GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null /usr/bin/git \
    -c user.name=CoquiFixture -c user.email=fixture@invalid \
    -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@" >/dev/null 2>&1
}
mkdir "$fixture_dir/node-project" "$fixture_dir/rust-project"
printf '{"name":"coqui-disposable-fixture","private":true}\n' > "$fixture_dir/node-project/package.json"
mkdir "$fixture_dir/node-project/dist"
printf 'Intentionally versioned output; preserve this fixture.\n' > "$fixture_dir/node-project/dist/keep.txt"
git_fixture -C "$fixture_dir/node-project" init -b main
git_fixture -C "$fixture_dir/node-project" add .
git_fixture -C "$fixture_dir/node-project" commit -m 'Disposable fixture'
printf 'Untracked source fixture; do not remove.\n' > "$fixture_dir/node-project/notes.txt"
printf '[package]\nname="coqui_fixture"\nversion="0.1.0"\n' > "$fixture_dir/rust-project/Cargo.toml"
printf '/target/\n' > "$fixture_dir/rust-project/.gitignore"
git_fixture -C "$fixture_dir/rust-project" init -b main
git_fixture -C "$fixture_dir/rust-project" add .
git_fixture -C "$fixture_dir/rust-project" commit -m 'Disposable fixture'
git_fixture -C "$fixture_dir/rust-project" worktree add --detach "$fixture_dir/linked-worktree"
mkdir "$fixture_dir/rust-project/target" "$fixture_dir/linked-worktree/target"
printf 'Generated fixture.\n' > "$fixture_dir/rust-project/target/output.cache"
printf 'Linked checkout fixture.\n' > "$fixture_dir/linked-worktree/target/output.cache"
printf '%s\n' "$fixture_dir"
