use super::*;

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

// Independent Bernstein summation, rather than production de Casteljau.
fn polynomial(controls: &[[f64; 2]], t: f64) -> Vector {
    let n = controls.len() - 1;
    let mut result = Vector { x: 0.0, y: 0.0 };
    for (i, [x, y]) in controls.iter().enumerate() {
        let weight = binomial(n, i) * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32);
        result.x += x * weight;
        result.y += y * weight;
    }
    result
}

fn reference(t: f64, source: Source) -> Vector {
    let controls: Vec<_> = CONTROL_POINTS
        .windows(2)
        .map(|pair| {
            [
                7.0 * (pair[1][0] - pair[0][0]),
                7.0 * (pair[1][1] - pair[0][1]),
            ]
        })
        .collect();
    let q = polynomial(&CONTROL_POINTS, t);
    let v = polynomial(&controls, t);
    let speed = v.x.hypot(v.y);
    Vector {
        x: q.x * source.extent - source.offset * v.y / speed,
        y: q.y * source.extent + source.offset * (v.x / speed - 1.0),
    }
}

fn local(q: Point, corner: &Curve, geometry: &Geometry, index: usize) -> Vector {
    match index {
        0 => Vector {
            x: q.x - (geometry.width - corner.extent),
            y: q.y,
        },
        1 => Vector {
            x: q.y - (geometry.height - corner.extent),
            y: geometry.width - q.x,
        },
        2 => Vector {
            x: corner.extent - q.x,
            y: geometry.height - q.y,
        },
        _ => Vector {
            x: corner.extent - q.y,
            y: q.x,
        },
    }
}

fn assert_convex(geometry: &Geometry) {
    let mut distinct: Vec<Point> = Vec::new();
    for point in &geometry.points {
        if distinct
            .last()
            .is_none_or(|last| (point.x - last.x).hypot(point.y - last.y) > 1e-8)
        {
            distinct.push(*point);
        }
    }
    if (distinct[0].x - distinct[distinct.len() - 1].x)
        .hypot(distinct[0].y - distinct[distinct.len() - 1].y)
        < 1e-8
    {
        distinct.pop();
    }
    for index in 0..distinct.len() {
        let a = distinct[index];
        let b = distinct[(index + 1) % distinct.len()];
        let c = distinct[(index + 2) % distinct.len()];
        assert!((b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x) >= -1e-7);
    }
    for q in &geometry.points {
        assert!(q.x >= -1e-8 && q.x <= geometry.width + 1e-8);
        assert!(q.y >= -1e-8 && q.y <= geometry.height + 1e-8);
    }
}

fn check_error(geometry: &Geometry) {
    let mut observed: f64 = 0.0;
    for (index, corner) in geometry.corners.iter().enumerate() {
        if corner.extent == 0.0 {
            continue;
        }
        for pair in corner.points.windows(2) {
            let [a, b] = [pair[0], pair[1]];
            let rounded = |p: Point| Point {
                x: fmt(p.x).parse().unwrap(),
                y: fmt(p.y).parse().unwrap(),
                ..p
            };
            let al = local(rounded(a), corner, geometry, index);
            let bl = local(rounded(b), corner, geometry, index);
            for j in 1..16 {
                let fraction = f64::from(j) / 16.0;
                let t =
                    a.parameter.unwrap() + (b.parameter.unwrap() - a.parameter.unwrap()) * fraction;
                let q = reference(t, corner.source);
                observed = observed.max(
                    (q.x - al.x * (1.0 - fraction) - bl.x * fraction)
                        .hypot(q.y - al.y * (1.0 - fraction) - bl.y * fraction),
                );
            }
        }
    }
    assert!(
        observed <= geometry.error_bound,
        "observed {observed} > {}",
        geometry.error_bound
    );
}

#[test]
fn source_controls_proportions_and_independent_polynomial_agree() {
    assert_eq!(PROFILE, "mac-mini-bezier7");
    assert_eq!(PHOTO_EXTENT, 0.31056899327095955);
    assert_eq!(CONTROL_POINTS[3], [0.7281206143651912, 0.0]);
    let diagonal = profile(0.5).unwrap();
    close(diagonal.x, 0.8141975551596445, 1e-10);
    close(diagonal.y, 0.1858024448403555, 1e-10);
    close(diagonal.heading, PI / 4.0, 1e-10);
    close(diagonal.curvature, 1.7600990604903817, 1e-10);
    for i in 0..=100 {
        let t = f64::from(i) / 100.0;
        let q = profile(t).unwrap();
        let independent = polynomial(&CONTROL_POINTS, t);
        close(q.x, independent.x, 1e-10);
        close(q.y, independent.y, 1e-10);
    }
    let geometry = create(127.0, 127.0, 127.0 * PHOTO_EXTENT, 0.05).unwrap();
    close(geometry.extent, 39.44226214541186, 1e-10);
    close(
        geometry.width - 2.0 * geometry.extent,
        48.11547570917629,
        1e-10,
    );
}

#[test]
fn analytic_endpoints_join_lines_at_g3_not_g4() {
    let start = profile(0.0).unwrap();
    let end = profile(1.0).unwrap();
    close(start.x, 0.0, 1e-10);
    close(start.y, 0.0, 1e-10);
    close(end.x, 1.0, 1e-10);
    close(end.y, 1.0, 1e-10);
    close(start.heading, 0.0, 1e-10);
    close(end.heading, PI / 2.0, 1e-10);
    for q in [&start, &end] {
        close(q.curvature, 0.0, 1e-10);
        close(q.curvature_derivative, 0.0, 1e-10);
        assert!(q.curvature_second_derivative > 5.4 && q.curvature_second_derivative < 5.5);
        assert!(q.speed > 0.0);
    }
    assert!(start.fourth.y > 0.0 && end.fourth.x < 0.0);
}

#[test]
fn polynomial_hulls_certify_speed_curvature_and_offset_radius() {
    let bound = bounds();
    assert_eq!(bound.intervals, 512);
    assert!(bound.convex);
    assert!(bound.minimum_radius > 0.563 && bound.minimum_radius < 0.565);
    let mut maximum: f64 = 0.0;
    for i in 0..=5000 {
        let q = profile(f64::from(i) / 5000.0).unwrap();
        assert!(q.speed >= bound.minimum_speed);
        assert!(q.curvature >= -1e-12 && q.curvature <= bound.maximum_curvature);
        maximum = maximum.max(q.curvature);
    }
    assert!(bound.maximum_curvature / maximum < 1.01);
}

#[test]
fn corner_boxes_are_convex_across_rectangles_and_small_extents() {
    for [width, height] in [
        [12.0, 48.0],
        [48.0, 12.0],
        [320.0, 216.0],
        [216.0, 144.0],
        [1024.0, 1024.0],
    ] {
        for extent in [0.0, 1e-12, 0.01, 27.0, 42.0, 1000.0] {
            let g = create(width, height, extent, 0.05).unwrap();
            assert_eq!(g.extent, extent.min(width / 2.0).min(height / 2.0));
            assert!(g.error_bound <= 0.05);
            assert_convex(&g);
            for corner in &g.corners {
                close(corner.extent, g.extent, 1e-10);
            }
        }
    }
    let g = create(10.123456789, 20.987654321, 0.0, 0.05).unwrap();
    let rounding = (g.width - fmt(g.width).parse::<f64>().unwrap())
        .hypot(g.height - fmt(g.height).parse::<f64>().unwrap());
    assert!(rounding > 0.0 && rounding <= g.error_bound);
}

#[test]
fn adaptive_error_budgets_bound_independently_evaluated_curves() {
    for [width, height, extent] in [
        [600.0, 400.0, 150.0],
        [536.0, 336.0, 118.0],
        [100.0, 80.0, 27.0],
        [72.0, 56.0, 56.0 * PHOTO_EXTENT],
    ] {
        let g = create(width, height, extent, 0.02).unwrap();
        assert!(g.error_bound <= 0.02);
        assert!(g.corners.iter().all(|c| c.points.len() < 500));
        check_error(&g);
        assert_convex(&g);
    }
}

#[test]
fn all_four_corners_preserve_identical_physical_coordinates() {
    for [width, height, extent] in [
        [240.0, 120.0, 27.0],
        [120.0, 240.0, 27.0],
        [127.0, 127.0, 127.0 * PHOTO_EXTENT],
        [274.0, 144.0, 144.0 * PHOTO_EXTENT],
        [48.0, 44.0, 27.0],
    ] {
        let g = create(width, height, extent, 0.02).unwrap();
        assert!(g.canonical);
        assert_eq!(g.extent, extent.min(width / 2.0).min(height / 2.0));
        for corner in &g.corners {
            assert_eq!(corner.extent, g.extent);
        }
        for i in 0..g.corners[0].points.len() {
            let first = local(g.corners[0].points[i], &g.corners[0], &g, 0);
            for index in 1..4 {
                let q = local(g.corners[index].points[i], &g.corners[index], &g, index);
                close(q.x, first.x, 1e-10);
                close(q.y, first.y, 1e-10);
            }
        }
        check_error(&g);
        assert_convex(&g);
    }
}

#[test]
fn parallel_offsets_resample_analytic_curves_and_reject_singularities() {
    let g = create(200.0, 120.0, 42.0, 0.05).unwrap();
    assert!(offset_path(&g, 3.0).unwrap().starts_with("M42 -3L158 -3"));
    assert_eq!(offset_path(&g, 0.0).unwrap(), g.path);
    assert!(offset_path(&g, -1.0).unwrap().starts_with("M42 1L158 1"));
    let square = create(200.0, 120.0, 0.0, 0.05).unwrap();
    assert!(
        offset_path(&square, -1.0)
            .unwrap()
            .starts_with("M1 1L199 1")
    );
    assert!(offset_path(&square, -60.0).is_err());
    assert!(offset_path(&g, -g.radius).is_err());
    // Check rounded emitted segments against independently evaluated offsets.
    let svg = offset_path(&g, 12.0).unwrap();
    let vertices: Vec<Vector> = svg
        .trim_end_matches('Z')
        .split(['M', 'L'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (x, y) = part.split_once(' ').unwrap();
            Vector {
                x: x.parse().unwrap(),
                y: y.parse().unwrap(),
            }
        })
        .collect();
    let distance = |q: Vector, a: Vector, b: Vector| {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let square = dx * dx + dy * dy;
        let t = if square == 0.0 {
            0.0
        } else {
            ((q.x - a.x) * dx + (q.y - a.y) * dy) / square
        }
        .clamp(0.0, 1.0);
        (q.x - a.x - t * dx).hypot(q.y - a.y - t * dy)
    };
    let mut observed: f64 = 0.0;
    for i in 0..=1000 {
        let mut q = reference(
            f64::from(i) / 1000.0,
            Source {
                offset: -12.0,
                ..g.corners[0].source
            },
        );
        q.x += 200.0 - 42.0;
        q.y -= 12.0;
        let closest = vertices
            .windows(2)
            .map(|p| distance(q, p[0], p[1]))
            .fold(f64::INFINITY, f64::min);
        observed = observed.max(closest);
    }
    assert!(observed <= 0.05, "offset error {observed}");
}

#[test]
fn bounded_caches_reuse_geometry_and_discard_old_entries() {
    GEOMETRIES.with(|cache| cache.borrow_mut().0.clear());
    let g = create(200.0, 120.0, 42.0, 0.05).unwrap();
    assert!(Rc::ptr_eq(&g, &create(200.0, 120.0, 42.0, 0.05).unwrap()));
    for i in 0..110 {
        create(300.0 + f64::from(i), 150.0, 27.0, 0.05).unwrap();
    }
    assert!(!Rc::ptr_eq(&g, &create(200.0, 120.0, 42.0, 0.05).unwrap()));
    GEOMETRIES.with(|cache| assert_eq!(cache.borrow().0.len(), 96));
    for offset in 1..=20 {
        offset_path(&g, f64::from(offset)).unwrap();
    }
    assert_eq!(g.offset_paths.borrow().0.len(), 16);
}

#[test]
fn invalid_coordinates_budgets_parameters_and_offsets_are_rejected() {
    for input in [
        [0.0, 120.0, 42.0, 0.05],
        [200.0, 120.0, -1.0, 0.05],
        [200.0, 120.0, 42.0, 0.0],
        [200.0, f64::INFINITY, 42.0, 0.05],
        [200.0, 120.0, 42.0, 1e-9],
        [1e16, 120.0, 0.0, 0.05],
    ] {
        assert!(create(input[0], input[1], input[2], input[3]).is_err());
    }
    for t in [-0.01, 1.01, f64::NAN, f64::INFINITY] {
        assert!(profile(t).is_err());
    }
    let g = create(200.0, 120.0, 42.0, 0.05).unwrap();
    for offset in [f64::NAN, f64::INFINITY, -f64::INFINITY] {
        assert!(offset_path(&g, offset).is_err());
    }
}

#[test]
fn svg_decimal_format_preserves_javascript_rounding_and_negative_zero() {
    for (input, expected) in [
        (0.0, "0"),
        (-0.0, "0"),
        (0.0078125, "0.007813"),
        (-0.0078125, "-0.007813"),
        (1.0000015, "1.000001"),
        (0.00000049, "0"),
        (-0.00000049, "0"),
        (12.5, "12.5"),
        (1_000_000_000_000_000_128.0, "1000000000000000100"),
        (1e21, "1e+21"),
        (1e-300, "0"),
        (f64::INFINITY, "Infinity"),
        (-f64::INFINITY, "-Infinity"),
        (f64::NAN, "NaN"),
    ] {
        assert_eq!(fmt(input), expected, "{input}");
    }
}

#[test]
fn serialized_public_shapes_match_browser_contracts() {
    let geometry = create(200.0, 120.0, 42.0, 0.05).unwrap();
    let json = serde_json::to_value(geometry.as_ref()).unwrap();
    assert!(json.get("errorBound").is_some());
    assert!(json.get("tolerance").is_none());
    assert!(json.get("offset_paths").is_none());
    assert!(json["points"][0].get("parameter").is_none());
    assert!(json["corners"][0]["points"][0].get("parameter").is_some());
    let profile = serde_json::to_value(profile(0.5).unwrap()).unwrap();
    assert!(profile.get("curvatureDerivative").is_some());
    assert!(profile.get("curvatureSecondDerivative").is_some());
    let bounds = serde_json::to_value(bounds()).unwrap();
    assert_eq!(bounds["intervals"], 512);
    assert!(bounds.get("minimumRadius").is_some());
}
