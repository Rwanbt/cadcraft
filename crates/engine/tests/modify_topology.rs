//! Synthetic, deterministic topological modification contracts.
//! Validate the drawing after each command, including undo and rejected edits.

use cadcraft_engine::Session;
use cadcraft_engine::doc::{Circle, Common, Drawing, EntityKind, Line, Space};
use cadcraft_engine::geom::Vec3;
use serde_json::json;

fn one_line() -> (Session, String) {
    let mut d = Drawing::new_metric();
    let handle = d.add(
        &Space::Model,
        Common::default(),
        EntityKind::Line(Line { a: Vec3::ZERO, b: Vec3::new(10.0, 0.0, 0.0) }),
    ).unwrap();
    let mut session = Session::empty();
    session.open_drawing(d, "Synthetic", None);
    (session, handle.hex())
}

fn segments(s: &Session) -> Vec<(f64, f64)> {
    let mut result: Vec<_> = s.doc().unwrap().model.iter().filter_map(|entity| {
        if let EntityKind::Line(l) = &entity.kind {
            Some((l.a.x.min(l.b.x), l.a.x.max(l.b.x)))
        } else {
            None
        }
    }).collect();
    result.sort_by(|a, b| a.0.total_cmp(&b.0));
    result
}

#[test]
fn break_at_point_splits_line_without_losing_either_side() {
    let (mut s, handle) = one_line();
    let original = s.doc().unwrap().clone();
    let result = s.execute("breakatpoint", &json!({"handle": handle, "at": [4.0, 0.0]})).unwrap();
    assert_eq!(result["handles"].as_array().unwrap().len(), 2);
    assert_eq!(segments(&s), vec![(0.0, 4.0), (4.0, 10.0)]);
    assert!(s.undo().unwrap().is_some());
    assert_eq!(s.doc().unwrap(), &original);
}

#[test]
fn break_between_points_removes_only_the_selected_interval() {
    let (mut s, handle) = one_line();
    s.execute("break", &json!({"handle": handle, "p1": [3.0, 0.0], "p2": [7.0, 0.0]})).unwrap();
    assert_eq!(segments(&s), vec![(0.0, 3.0), (7.0, 10.0)]);
}

#[test]
fn join_contiguous_lines_preserves_vertex_chain_and_undo_redo() {
    let mut d = Drawing::new_metric();
    let first = d.add(&Space::Model, Common::default(), EntityKind::Line(Line { a: Vec3::ZERO, b: Vec3::new(5.0, 0.0, 0.0) })).unwrap();
    let second = d.add(&Space::Model, Common::default(), EntityKind::Line(Line { a: Vec3::new(5.0, 0.0, 0.0), b: Vec3::new(10.0, 0.0, 0.0) })).unwrap();
    let mut s = Session::empty();
    s.open_drawing(d, "Two segments", None);
    let original = s.doc().unwrap().clone();
    let result = s.execute("join", &json!({"handles": [first.hex(), second.hex()]})).unwrap();
    assert!(result["handle"].is_string());
    assert_eq!(s.doc().unwrap().model.len(), 1);
    let joined = s.doc().unwrap().model.iter().next().unwrap();
    let EntityKind::LwPolyline(poly) = &joined.kind else { panic!("JOIN must produce a polyline"); };
    assert!(!poly.closed);
    assert_eq!(poly.vertices.len(), 3);
    assert_eq!(poly.vertices.iter().map(|v| (v.p.x, v.p.y)).collect::<Vec<_>>(), vec![(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)]);
    let edited = s.doc().unwrap().clone();
    assert!(s.undo().unwrap().is_some());
    assert_eq!(s.doc().unwrap(), &original);
    assert!(s.redo().unwrap().is_some());
    assert_eq!(s.doc().unwrap(), &edited);
}

#[test]
fn join_unconnected_lines_returns_error_without_mutating_drawing() {
    let mut d = Drawing::new_metric();
    let a = d.add(&Space::Model, Common::default(), EntityKind::Line(Line { a: Vec3::ZERO, b: Vec3::new(1.0, 0.0, 0.0) })).unwrap();
    let b = d.add(&Space::Model, Common::default(), EntityKind::Line(Line { a: Vec3::new(10.0, 0.0, 0.0), b: Vec3::new(11.0, 0.0, 0.0) })).unwrap();
    let mut s = Session::empty();
    s.open_drawing(d, "Unconnected", None);
    let original = s.doc().unwrap().clone();
    assert!(s.execute("join", &json!({"handles": [a.hex(), b.hex()]})).is_err());
    assert_eq!(s.doc().unwrap(), &original);
}

#[test]
fn stretch_crossing_window_moves_only_the_included_line_endpoint() {
    let (mut s, _) = one_line();
    let original = s.doc().unwrap().clone();
    let result = s.execute("stretch", &json!({"window": [[9.0, -1.0], [11.0, 1.0]], "delta": [0.0, 5.0]})).unwrap();
    assert_eq!(result["stretched"], 1);
    let e = s.doc().unwrap().model.iter().next().unwrap();
    let EntityKind::Line(line) = &e.kind else { panic!("STRETCH should preserve line type"); };
    assert_eq!(line.a, Vec3::ZERO);
    assert_eq!(line.b, Vec3::new(10.0, 5.0, 0.0));
    s.undo().unwrap();
    assert_eq!(s.doc().unwrap(), &original);
}

#[test]
fn breaking_a_circle_at_one_point_is_rejected_without_data_loss() {
    let mut d = Drawing::new_metric();
    let handle = d.add(
        &Space::Model,
        Common::default(),
        EntityKind::Circle(Circle { center: Vec3::ZERO, radius: 5.0 }),
    ).unwrap();
    let mut s = Session::empty();
    s.open_drawing(d, "Circle", None);
    let original = s.doc().unwrap().clone();
    assert!(s.execute("breakatpoint", &json!({"handle": handle.hex(), "at": [5.0, 0.0]})).is_err());
    assert_eq!(s.doc().unwrap(), &original);
}
