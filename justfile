bench:
  #!/usr/bin/env bash
  benchmarks="$(cargo bench --target wasm32-wasi --features=bench --no-run --color=always 2>&1 | tee /dev/tty | grep -oP 'target/.*.wasm')"

  echo "$benchmarks" \
    | xargs -I{} wasmtime --dir $PWD/target::target {} --bench --color=always

build:
  cargo build

run: build
  zellij -n ./plugin-dev-workspace.kdl -s zjk8s-dev

test:
  cargo component test -- --nocapture

lint:
  cargo clippy --all-targets --all-features
  cargo audit

release version:
  cargo set-version {{version}}
  direnv exec . cargo build --release
  git commit -am "chore: bump version to v{{version}}"
  git tag -m "v{{version}}" v{{version}}
  git push origin main
  git push origin "v{{version}}"
