# CADCraft — reproducible test execution

## Baseline reference

Fork: `Rwanbt/cadcraft`. Base branch: `main`.
Reference commit: `699dd1aeca1dba018931d2eb95e92c0cf2033bbd`.
Existing draft: [PR #1](https://github.com/Rwanbt/cadcraft/pull/1).
Baseline Linux run: [38146658343](https://github.com/Rwanbt/cadcraft/actions/runs/38146658343), **923 passed / 0 failed / 0 ignored** across 138 Cargo test-result summaries.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p cadcraft-dxf --test parser_limits --locked
cargo test -p cadcraft-io --test dwg_output_limits --locked
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --locked --lcov --output-path coverage.lcov
cargo llvm-cov report --json --output-path coverage.json
```

Coverage uses `llvm-tools-preview`. Install the native Linux dependencies listed in the workflows before running locally. Each command's result must be associated with the exact tested SHA.

## CI

- `.github/workflows/rust-quality.yml`: Linux formatting, lint and unit/integration execution.
- `.github/workflows/coverage.yml`: instrumented workspace tests and downloadable LCOV/JSON.
- `.github/workflows/packaging-lint.yml`: static packaging checks.
- `.github/workflows/release.yml`: release/packaging pipeline, not a substitute for interactive E2E coverage.

A pending, skipped, cancelled or failed job is **not a pass**. UI E2E, Windows, macOS, WASM and fuzzing remain distinct gates. No automatic merge or upstream PR is permitted.
