//! Final output size, including synthetic DWG header repair, must respect the configured
//! conversion cap. The drawing is generated inside the test: no external DWG corpus is needed.
#![cfg(not(target_arch = "wasm32"))]

use cadcraft_doc::Drawing;
use cadcraft_dwg::{Limits, dwg_to_dxf, dwg_to_dxf_with};

#[test]
fn dwg_conversion_never_returns_more_bytes_than_the_final_limit() {
    let dwg = cadcraft_io::write(&Drawing::new_metric(), "synthetic.dwg").expect("synthetic DWG");
    let full = dwg_to_dxf(&dwg).expect("unrestricted DWG conversion");
    assert!(!full.is_empty());

    // Before the fix, DXF header variables were appended after the capped writer stopped
    // counting. The last-byte boundary catches an unchecked post-processing expansion.
    let limit = full.len() - 1;
    let capped = dwg_to_dxf_with(&dwg, Limits { max_dwg_bytes: dwg.len(), max_dxf_bytes: limit });
    assert!(capped.is_err(), "conversion must reject a final DXF larger than {limit} bytes");

    let exact =
        dwg_to_dxf_with(&dwg, Limits { max_dwg_bytes: dwg.len(), max_dxf_bytes: full.len() }).expect("exact-size cap must allow the full output");
    assert_eq!(exact.len(), full.len());
}
