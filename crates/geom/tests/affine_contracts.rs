//! Independent affine and OCS numerical contracts with deterministic inputs.
//! These check explicit analytical expectations, not just code snapshots.

use cadcraft_geom::{Mat3, Mat4, Vec2, Vec3};

#[test]
fn affine_composition_order_matches_hand_computed_coordinates() {
    let p = Vec2::new(4.0, -3.0);
    let m = Mat3::translate(Vec2::new(3.0, -2.0)).then(Mat3::scale(2.0, -4.0));
    assert!(m.apply(p).near(Vec2::new(14.0, 20.0), 1e-12));
    assert!(m.apply_vec(Vec2::new(1.0, 1.0)).near(Vec2::new(2.0, -4.0), 1e-12));
    assert!((m.det() + 8.0).abs() < 1e-12);
    assert!(m.is_mirroring());
}

#[test]
fn fixed_seed_affine_inverse_reverses_a_diverse_set_of_transforms() {
    let samples = [
        (Vec2::new(0.0, 0.0), 0.0, 1.0, 1.0),
        (Vec2::new(-17.0, 8.5), 0.3, 2.0, 0.25),
        (Vec2::new(1_000.0, -15.0), -2.7, -0.5, 7.0),
        (Vec2::new(3.25, 99.0), std::f64::consts::PI, 1.25, -3.0),
    ];
    let points = [Vec2::ZERO, Vec2::new(1.0, -2.0), Vec2::new(125.0, 88.0)];

    for (translation, angle, sx, sy) in samples {
        let transform = Mat3::translate(translation).then(Mat3::rotate(angle)).then(Mat3::scale(sx, sy));
        let inverse = transform.inverse().expect("well-conditioned affine transform is invertible");
        for p in points {
            assert!(inverse.apply(transform.apply(p)).near(p, 1e-9), "failed for transform {transform:?}, point {p:?}");
            assert!(transform.apply(inverse.apply(p)).near(p, 1e-9), "right inverse failed for transform {transform:?}, point {p:?}");
        }
    }
}

#[test]
fn singular_and_non_finite_affine_matrices_have_no_inverse() {
    assert!(Mat3::scale(0.0, 1.0).inverse().is_none());
    assert!(Mat3::scale(1e-310, 1.0).inverse().is_none());
    assert!(Mat3::scale(f64::INFINITY, 1.0).inverse().is_none());
    assert!(Mat3::scale(f64::NAN, 1.0).inverse().is_none());
}

#[test]
fn line_reflections_preserve_points_on_axis_and_are_involutions() {
    let mirror = Mat3::mirror(Vec2::ZERO, Vec2::new(1.0, 1.0));
    let source = Vec2::new(2.0, 5.0);
    assert!(mirror.apply(source).near(Vec2::new(5.0, 2.0), 1e-12));
    assert!(mirror.apply(Vec2::new(4.0, 4.0)).near(Vec2::new(4.0, 4.0), 1e-12));
    assert!(mirror.apply(mirror.apply(source)).near(source, 1e-12));
    assert_eq!(Mat3::mirror(Vec2::ZERO, Vec2::ZERO), Mat3::IDENTITY);
}

#[test]
fn ocs_maps_local_z_to_the_declared_extrusion_axis() {
    let normals = [Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(-2.0, 7.0, 1.0)];
    for normal in normals {
        let frame = Mat4::ocs(normal);
        let actual = frame.apply(Vec3::Z);
        let expected = normal.normalized();
        assert!((actual - expected).len() < 1e-12, "normal {normal:?} became {actual:?}");
        let x = frame.apply(Vec3::new(1.0, 0.0, 0.0));
        let y = frame.apply(Vec3::new(0.0, 1.0, 0.0));
        assert!(x.dot(y).abs() < 1e-12);
        assert!(x.dot(actual).abs() < 1e-12);
        assert!(y.dot(actual).abs() < 1e-12);
        assert!((x.len() - 1.0).abs() < 1e-12);
        assert!((y.len() - 1.0).abs() < 1e-12);
    }
}
