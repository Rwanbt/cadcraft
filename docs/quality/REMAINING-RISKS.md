# CADCraft — remaining production-readiness risks

As of the 2026-10-12 audit, the quality of the complete application is **PARTIALLY VERIFIED**, not production-ready. The Linux Rust tests alone cannot verify the user experience or integrity of every drawing.

| Priority | Residual risk | Necessary evidence |
|---|---|---|
| Critical | Untested save/reopen/export paths may corrupt or silently drop drawing data | Synthetic reference lifecycle E2E plus licensed independent DXF/DWG corpus |
| High | DXF/DWG malformed and oversized inputs | Deterministic truncation and limit tests (started), corpus replay, structured fuzzing and sanitizers |
| High | Real desktop window, GPU interactions, installers and OS differences | Windows/macOS/Linux native build, smoke and real-window E2E |
| High | Browser/WASM upload, editing and export | WASM build plus browser-driven E2E |
| High | Coverage unknown until LLVM completes | Read actual `coverage.json`, map uncovered paths, add verified anti-regression gate |
| Medium | Concurrency, cancellation, timeouts and recovery | Independent asynchronous and resource-safety contracts |
| Medium | Dependency, license and build provenance drift | audit tooling, pinned dependencies, release artefact validation |
| Medium | Fork lags upstream | Initial comparison found 145 commits behind; review upstream separately, never auto-merge into this quality branch |

No evidence of live fuzzing, real GUI E2E, or cross-platform test execution was established during this audit. Requirements not yet verified remain open; do not reclassify them from unit-test counts.
