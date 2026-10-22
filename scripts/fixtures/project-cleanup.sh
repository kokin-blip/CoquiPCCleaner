#!/bin/sh
# Creates only new disposable fixtures. It does not clean any project or cache.
set -eu
base=$(mktemp -d /private/tmp/coqui-project-cleanup.XXXXXX)
fixture_git() {
  env -i PATH=/usr/bin:/bin GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null \
    /usr/bin/git -c user.name=CoquiFixture -c user.email=fixture@invalid \
    -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@" >/dev/null
}
mkdir "$base/rust-project" "$base/node-project"
for project in rust-project node-project; do
  fixture_git -C "$base/$project" init -b main
  if [ "$project" = rust-project ]; then
    printf '[package]\nname="coqui-disposable-fixture"\nversion="0.1.0"\n' > "$base/$project/Cargo.toml"
    printf '/target/\n' > "$base/$project/.gitignore"
    folder=target
  else
    printf '{"name":"coqui-disposable-fixture","scripts":{"build":"example-tool build"}}\n' > "$base/$project/package.json"
    printf '{"name":"coqui-disposable-fixture","lockfileVersion":3,"packages":{}}\n' > "$base/$project/package-lock.json"
    printf '/dist/\n' > "$base/$project/.gitignore"
    folder=dist
  fi
  fixture_git -C "$base/$project" add .
  fixture_git -C "$base/$project" commit -m 'Disposable cleanup validation inputs'
  mkdir "$base/$project/$folder"
  printf 'Disposable generated fixture; contains no user data.\n' > "$base/$project/$folder/coqui-disposable.cache"
done
printf 'Disposable fixture scope: %s\nSelect only this new folder for native review validation.\n' "$base"
