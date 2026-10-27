#!/bin/sh
# Creates only new disposable evidence fixtures; never contacts a remote.
set -eu
fixture_dir=$(mktemp -d /private/tmp/coqui-development-indicators.XXXXXX)
git_fixture() {
  env GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_COMMITTER_DATE=2000-01-01T12:00:00Z GIT_AUTHOR_DATE=2000-01-01T12:00:00Z /usr/bin/git \
    -c user.name=CoquiFixture -c user.email=fixture@invalid \
    -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@" >/dev/null 2>&1
}
for project in cached-match local-commit dirty-project; do
  mkdir -p "$fixture_dir/$project/target/debug/incremental"
  printf '[package]\nname="disposable_fixture"\nversion="0.1.0"\n' > "$fixture_dir/$project/Cargo.toml"
  printf '/target/\n' > "$fixture_dir/$project/.gitignore"
  printf 'Disposable generated fixture only.\n' > "$fixture_dir/$project/target/debug/incremental/output.cache"
  git_fixture -C "$fixture_dir/$project" init -b main
  git_fixture -C "$fixture_dir/$project" add Cargo.toml .gitignore
  git_fixture -C "$fixture_dir/$project" commit -m 'Disposable fixture'
  git_fixture -C "$fixture_dir/$project" remote add origin https://example.invalid/coqui-fixture.git
  git_fixture -C "$fixture_dir/$project" config branch.main.remote origin
  git_fixture -C "$fixture_dir/$project" config branch.main.merge refs/heads/main
  git_fixture -C "$fixture_dir/$project" update-ref refs/remotes/origin/main HEAD
done
printf 'Disposable local-only commit.\n' > "$fixture_dir/local-commit/source.rs"
git_fixture -C "$fixture_dir/local-commit" add source.rs
git_fixture -C "$fixture_dir/local-commit" commit -m 'Local-only fixture'
printf 'Disposable untracked work.\n' > "$fixture_dir/dirty-project/notes.txt"
mkdir -p "$fixture_dir/.codex/worktrees"
git_fixture -C "$fixture_dir/cached-match" worktree add --detach "$fixture_dir/.codex/worktrees/review-fixture"
mkdir -p "$fixture_dir/.codex/worktrees/review-fixture/target/debug"
printf 'Disposable agent-worktree output.\n' > "$fixture_dir/.codex/worktrees/review-fixture/target/debug/agent.cache"
printf '%s\n' "$fixture_dir"
