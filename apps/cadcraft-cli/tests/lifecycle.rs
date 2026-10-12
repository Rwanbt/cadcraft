//! Real CLI processes must preserve the drawing through save, reopen, edit and export.
//! Synthetic data only; no external corpus, GPU or network service is required.

use std::path::PathBuf;
use std::process::{Command, Output};

use cadcraft_engine::doc::EntityKind;

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_success(output: &Output, step: &str) {
    assert!(
        output.status.success(),
        "{step} failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_creates_saves_reopens_edits_and_exports_a_real_drawing() {
    let dir = TempDir(std::env::temp_dir().join(format!("cadcraft-cli-lifecycle-{}", std::process::id())));
    std::fs::create_dir_all(&dir.0).expect("create isolated temp directory");
    let first = dir.0.join("initial.dxf");
    let second = dir.0.join("edited.dxf");
    let svg = dir.0.join("edited.svg");
    let exe = env!("CARGO_BIN_EXE_cadcraft-cli");

    let created = Command::new(exe)
        .arg("run")
        .arg("--metric")
        .arg("--cmd")
        .arg(r#"line {"points":[[0,0],[10,0]]}"#)
        .arg("--save")
        .arg(&first)
        .output()
        .expect("spawn CLI create");
    assert_success(&created, "create and save");

    let first_bytes = std::fs::read(&first).expect("saved DXF exists");
    let first_doc = cadcraft_io::read(&first_bytes, "initial.dxf").expect("saved DXF parses");
    assert_eq!(first_doc.model.len(), 1, "initial document contains one line");
    assert!(first_doc.model.iter().any(|e| matches!(&e.kind, EntityKind::Line(_))));

    let edited = Command::new(exe)
        .arg("run")
        .arg(&first)
        .arg("--cmd")
        .arg(r#"circle {"center":[5,5],"radius":2}"#)
        .arg("--save")
        .arg(&second)
        .output()
        .expect("spawn CLI reopen/edit");
    assert_success(&edited, "reopen, edit and save");

    let second_bytes = std::fs::read(&second).expect("edited DXF exists");
    let second_doc = cadcraft_io::read(&second_bytes, "edited.dxf").expect("edited DXF parses");
    assert_eq!(second_doc.model.len(), 2, "both original line and subsequent circle survive");
    assert!(second_doc.model.iter().any(|e| matches!(&e.kind, EntityKind::Line(_))));
    assert!(second_doc.model.iter().any(|e| matches!(&e.kind, EntityKind::Circle(_))));

    let inspected = Command::new(exe).arg("info").arg(&second).output().expect("spawn CLI info");
    assert_success(&inspected, "inspect reopened DXF");
    let info: serde_json::Value = serde_json::from_slice(&inspected.stdout).expect("info returns JSON");
    assert!(info.is_object(), "drawing inspection must be a JSON object");

    let exported = Command::new(exe).arg("convert").arg(&second).arg(&svg).output().expect("spawn CLI export");
    assert_success(&exported, "export SVG");
    let svg_data = std::fs::read_to_string(&svg).expect("SVG export exists");
    assert!(svg_data.contains("<svg"), "output must be an SVG document");
}
