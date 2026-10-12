# CADCraft — instrumented test-coverage baseline

**Scope:** `Rwanbt/cadcraft`; base `main` at `699dd1aeca1dba018931d2eb95e92c0cf2033bbd`.
**Captured:** 2026-10-12, based on the completed 2026-10-11 Rust CI run.

## Verified starting point

- [Rust quality run 38146658343](https://github.com/Rwanbt/cadcraft/actions/runs/38146658343), head `7fbac8eb3231d4662145ac2e294dc7fae4220e45`: Linux rustfmt, Clippy and workspace tests succeeded.
- The real cargo test output reports **923 passed, 0 failed, 0 ignored**, summed from **138** `test result:` summaries. These are tests, not test files. The source branch also adds a CI workflow to `main`; consequently, this is a CI baseline, **not** a measurement of coverage on `main`.
- Code coverage at this starting point: **not measured**. No percentage may be inferred from the test count.

## Instrumented run

`.github/workflows/coverage.yml` executes `cargo llvm-cov --workspace --locked --lcov --output-path coverage.lcov`, then exports JSON with `cargo llvm-cov report --json --output-path coverage.json`. The job uploads both reports as `rust-coverage-linux`, with 30-day retention.

**Verified baseline:** [GitHub Actions coverage run 38269733408](https://github.com/Rwanbt/cadcraft/actions/runs/38269733408), SHA `1064b65775d4a44c8c08d0c309c8c192088fbdcf`, completed successfully on Linux (2026-10-12). The logged LLVM JSON totals are:
- **Lines:** 44,634 / 55,377 = **80.60%**.
- **Functions:** 6,181 / 7,773 = **79.52%**.
- **Regions:** 105,728 / 134,362 = **78.69%**.
- **Branches:** 0 / 0 (not instrumented; **not** proof of zero coverage).
- [Rust quality run 38269733463](https://github.com/Rwanbt/cadcraft/actions/runs/38269733463), same SHA: **931 tests passed, 0 failed, 0 ignored**.
- LCOV and JSON were uploaded in the `rust-coverage-linux` artifact. This is an actual instrumented Linux baseline; it is **not** a statement of complete product coverage.

The Linux report excludes platform-only execution paths (macOS, Windows, FreeBSD, wasm and native UI automation). Do not treat it as cross-platform or whole-product proof.

## Next required measurements

1. Extract whole-workspace line/function/branch totals and per-source-file results from JSON.
2. Classify missing lines as testable paths, unsupported platforms, generated code or vendored code; justify all exclusions.
3. Add targeted, reproducible tests on the highest-risk uncovered format and persistence paths.
4. Establish a no-regression coverage gate **after** recording the real baseline; do not set a speculative 100% threshold.
