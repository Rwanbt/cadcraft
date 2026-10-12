# CADCraft — feature/test traceability (initial audit)

**Base:** `main` @ `699dd1aeca1dba018931d2eb95e92c0cf2033bbd`.
**Status convention:** `PARTIAL` = tests exist but relevant cross-platform/E2E/coverage evidence is incomplete; `UNVERIFIED` = no applicable execution confirmed; `BLOCKED` = cannot execute with current CI. No component is marked `VERIFIED` solely because the Linux workspace test job passes.

| ID | Component / behavioural contract | Observable evidence | Remaining verification | Status |
|---|---|---|---|---|
| CAD-GEO-01 | `geom`: geometry and intersections | `crates/geom/src/tests.rs` | property tests, boundary/mutation coverage | PARTIAL |
| CAD-DXF-01 | `dxf`: ASCII/binary tags, limits, hostile bytes | `crates/dxf/src/lib.rs`; `crates/dxf/tests/parser_limits.rs` (new) | fuzz corpus, real-file interoperability, coverage | PARTIAL |
| CAD-DWG-01 | `dwg`: capped conversion and version detection | `crates/dwg/src/lib.rs`; `crates/io/tests/dwg_output_limits.rs` (new) | diverse DWG versions and malformed corpus | PARTIAL |
| CAD-DOC-01 | `doc`: entities, layouts, tables and persistence model | `crates/doc/src/tests.rs` | full serialization/migration matrix | PARTIAL |
| CAD-CST-01 | `constraints`: solver and numeric residuals | `crates/constraints/src/tests.rs` | convergence failures, fuzz numeric bounds | PARTIAL |
| CAD-REN-01 | `render`: drawing rasterization, dimensions, hatch | `crates/render/src/tests.rs`; `crates/render/tests/` | independent pixel oracles and GPU parity | PARTIAL |
| CAD-IO-01 | `io`: DXF/DWG save and reload, SVG/PNG/PDF export | `crates/io/src/tests.rs`; `crates/io/tests/` | full E2E file lifecycle and reference corpora | PARTIAL |
| CAD-ENG-01 | `engine`: drawing commands, state, undo/redo | `crates/engine/src/tests.rs`; `crates/engine/tests/` | complete command contract matrix and concurrency | PARTIAL |
| CAD-MCP-01 | `mcp`: protocol and command dispatch | `crates/mcp/src/tests.rs`; `crates/mcp/tests/` | protocol error and cancellation matrix | PARTIAL |
| CAD-CLI-01 | `cadcraft-cli`: real process, MCP and drawing lifecycle | `apps/cadcraft-cli/tests/mcp_connect.rs`; `apps/cadcraft-cli/tests/lifecycle.rs` (new) | native GUI workflow and packaging E2E | PARTIAL |
| CAD-UI-01 | `ui-egui`: dialogs, controls, layout and focus | `crates/ui-egui/tests/` | native real-window E2E on each OS | PARTIAL |
| CAD-WEB-01 | `cadcraft-web`: upload, download and WASM interface | `apps/cadcraft-web/src/` | real browser E2E, responsive and JS errors | UNVERIFIED |
| CAD-DESK-01 | `cadcraft`: install, launch, control server and menus | `apps/cadcraft/src/` | packaged native smoke/E2E and uninstall | UNVERIFIED |
| CAD-PKG-01 | Packaging and release metadata | [Packaging lint run 38146658224](https://github.com/Rwanbt/cadcraft/actions/runs/38146658224) | actual installers on target OSs | PARTIAL |

This is a starting inventory, not an exhaustive proof of every public command/API. Expand rows with measured branch/file coverage, CI SHA and reproducible user journeys before promoting their state.
