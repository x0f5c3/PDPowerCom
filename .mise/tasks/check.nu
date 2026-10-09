#!/usr/bin/env -S nu

def main [] {
  npm ci
  npm run build
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
}
