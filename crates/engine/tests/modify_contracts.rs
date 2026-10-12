//! Deterministic behavioural contracts for JSON-driven drawing modifications.
//! Assert geometry, handles, source immutability and undo/redo, not only result JSON.

use cadcraft_engine::Session;
use cadcraft_engine::doc::{Common, Drawing, EntityKind, Layer, Line, Space};
use cadcraft_engine::geom::Vec3;
use serde_json::json;

fn line(a: [f64; 2], b: [f64; 2]) -> EntityKind {
    EntityKind::Line(Line { a: Vec3::new(a[0], a[1], 0.0), b: Vec3::new(b[0], b[1], 0.0) })
}

fn drawing_with_line() -> (Session, String) {
    let mut drawing = Drawing::new_metric();
    let handle = drawing.add(&Space::Model, Common::default(), line([0.0, 0.0], [10.0, 0.0])).unwrap();
    let mut session = Session::empty();
    session.open_drawing(drawing, "Synthetic", None);
    (session, handle.hex())
}

fn line_ends(s: &Session) -> Vec<([f64; 2], [f64; 2])> {
    let mut out: Vec<_> = s
        .doc()
        .unwrap()
        .model
        .iter()
        .filter_map(|e| match &e.kind {
            EntityKind::Line(l) => Some(([l.a.x, l.a.y], [l.b.x, l.b.y])),
            _ => None,
        })
        .collect();
    out.sort_by(|a, b| a.0[0].total_cmp(&b.0[0]).then(a.0[1].total_cmp(&b.0[1])));
    out
}

#[test]
fn move_from_to_preserves_handle_and_undo_redo_geometry() {
    let (mut s, h) = drawing_with_line();
    let original = s.doc().unwrap().clone();
    assert_eq!(s.execute("move", &json!({"handles": [h], "from": [1.0, 1.0], "to": [4.0, -1.0]})).unwrap()["moved"], 1);
    assert_eq!(line_ends(&s), vec![([3.0, -2.0], [13.0, -2.0])]);
    let edited = s.doc().unwrap().clone();
    assert_eq!(s.doc().unwrap().model.handles()[0].hex(), h);
    assert!(s.undo().unwrap().is_some());
    assert_eq!(s.doc().unwrap(), &original);
    assert!(s.redo().unwrap().is_some());
    assert_eq!(s.doc().unwrap(), &edited);
}

#[test]
fn multi_copy_uses_original_each_time_and_is_one_undo_step() {
    let (mut s, h) = drawing_with_line();
    let original = s.doc().unwrap().clone();
    let response = s.execute("copy", &json!({"handles": [h], "delta": [0.0, 2.0], "count": 3})).unwrap();
    let handles = response["handles"].as_array().unwrap();
    assert_eq!(handles.len(), 3);
    assert!(handles.iter().all(|v| v.as_str() != Some(h.as_str())));
    assert_eq!(line_ends(&s), vec![
        ([0.0, 0.0], [10.0, 0.0]),
        ([0.0, 2.0], [10.0, 2.0]),
        ([0.0, 4.0], [10.0, 4.0]),
        ([0.0, 6.0], [10.0, 6.0]),
    ]);
    let edited = s.doc().unwrap().clone();
    assert!(s.undo().unwrap().is_some(), "a batch COPY must be atomic in undo history");
    assert_eq!(s.doc().unwrap(), &original);
    assert!(s.redo().unwrap().is_some());
    assert_eq!(s.doc().unwrap(), &edited);
}

#[test]
fn mirror_copy_and_mirror_erase_have_distinct_source_semantics() {
    let (mut copy, h) = drawing_with_line();
    copy.execute("mirror", &json!({"handles": [h], "p1": [0,0], "p2": [0,1]})).unwrap();
    assert_eq!(line_ends(&copy), vec![([-10.0, 0.0], [0.0, 0.0]), ([0.0, 0.0], [10.0, 0.0])]);

    let (mut replace, h) = drawing_with_line();
    replace.execute("mirror", &json!({"handles": [h], "p1": [0,0], "p2": [0,1], "erase": true})).unwrap();
    assert_eq!(line_ends(&replace), vec![([0.0, 0.0], [-10.0, 0.0])]);
    assert_eq!(replace.doc().unwrap().model.len(), 1);
}

#[test]
fn invalid_transform_parameters_do_not_modify_the_drawing() {
    let (mut s, h) = drawing_with_line();
    let original = s.doc().unwrap().clone();
    for (cmd, params) in [
        ("scale", json!({"handles": [h], "base": [0,0], "factor": 0})),
        ("scale", json!({"handles": [h], "base": [0,0], "factor": -2})),
        ("mirror", json!({"handles": [h], "p1": [1,1], "p2": [1,1]})),
        ("move", json!({"handles": [h], "from": [0,0]})),
    ] {
        assert!(s.execute(cmd, &params).is_err(), "{cmd} incorrectly accepted {params}");
        assert_eq!(s.doc().unwrap(), &original, "{cmd} corrupted the drawing on error");
    }
}

#[test]
fn move_and_erase_skip_locked_layers_but_change_unlocked_entities() {
    let mut d = Drawing::new_metric();
    d.layers.push(Layer { name: "Locked".into(), locked: true, ..Layer::default() });
    let locked = d.add(&Space::Model, Common { layer: "Locked".into(), ..Common::default() }, line([0.0, 0.0], [3.0, 0.0])).unwrap();
    let open = d.add(&Space::Model, Common::default(), line([0.0, 2.0], [3.0, 2.0])).unwrap();
    let mut s = Session::empty();
    s.open_drawing(d, "Locked layers", None);
    let handles = [locked.hex(), open.hex()];

    let moved = s.execute("move", &json!({"handles": handles, "delta": [5,0]})).unwrap();
    assert_eq!(moved["moved"], 1);
    assert_eq!(line_ends(&s), vec![([0.0, 0.0], [3.0, 0.0]), ([5.0, 2.0], [8.0, 2.0])]);
    let erased = s.execute("erase", &json!({"handles": handles})).unwrap();
    assert_eq!(erased["erased"], 1);
    assert_eq!(line_ends(&s), vec![([0.0, 0.0], [3.0, 0.0])]);
    assert_eq!(s.doc().unwrap().model.handles(), vec![locked]);
}

#[test]
fn overkill_removes_exact_duplicates_only_and_is_undoable() {
    let mut d = Drawing::new_metric();
    d.add(&Space::Model, Common::default(), line([0.0, 0.0], [10.0, 0.0])).unwrap();
    d.add(&Space::Model, Common::default(), line([0.0, 0.0], [10.0, 0.0])).unwrap();
    d.add(&Space::Model, Common::default(), line([0.0, 1.0], [10.0, 1.0])).unwrap();
    let mut s = Session::empty();
    s.open_drawing(d, "Duplicates", None);
    let original = s.doc().unwrap().clone();
    let result = s.execute("overkill", &json!({})).unwrap();
    assert_eq!(result["deleted"], 1);
    assert_eq!(line_ends(&s), vec![([0.0, 0.0], [10.0, 0.0]), ([0.0, 1.0], [10.0, 1.0])]);
    assert!(s.undo().unwrap().is_some());
    assert_eq!(s.doc().unwrap(), &original);
}
