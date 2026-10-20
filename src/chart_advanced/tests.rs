use super::*;
fn bounds() -> SvgBounds {
    SvgBounds {
        left: 100.0,
        top: 200.0,
        width: 400.0,
        height: 300.0,
        view_width: 800.0,
        view_height: 600.0,
    }
}
#[test]
fn pointer_owner_and_svg_bounds_ignore_child_target_and_accept_outside_motion() {
    let rect = |x, y, width, height| {
        dioxus::html::geometry::PixelsRect::new(
            dioxus::html::geometry::euclid::Point2D::new(x, y),
            dioxus::html::geometry::PixelsSize::new(width, height),
        )
    };
    assert_eq!(
        SvgBounds::from_rect(rect(100.0, 200.0, 400.0, 300.0), (800.0, 600.0)),
        Some(bounds())
    );
    assert_eq!(
        SvgBounds::from_rect(rect(0.0, 0.0, 0.0, 300.0), (800.0, 600.0)),
        None
    );
    assert_eq!(
        SvgBounds::from_rect(rect(f64::NAN, 0.0, 400.0, 300.0), (800.0, 600.0)),
        None
    );
    let mut drag = ChartDrag {
        pointer: 7,
        start: (150.0, 250.0),
        current: (150.0, 250.0),
        bounds: Some(bounds()),
        kind: (),
    };
    assert!(!drag.move_pointer(8, (900.0, 800.0)));
    assert_eq!(drag.points(), Some(((100.0, 100.0), (100.0, 100.0))));
    assert!(drag.move_pointer(7, (600.0, 150.0)));
    assert_eq!(drag.points(), Some(((100.0, 100.0), (1000.0, -100.0))));
    drag.bounds = None;
    assert_eq!(drag.points(), None);
}
#[test]
fn surface_touch_ownership_pinch_and_handoff_preserve_camera_orientation() {
    let mut orbit = SurfaceOrbit {
        bounds: Some(bounds()),
        ..Default::default()
    };
    let contact = |id, x, touch| SurfaceContact {
        id,
        point: (x, 250.0),
        touch,
        pan: false,
    };
    assert!(orbit.begin(contact(7, 150.0, true)));
    assert!(!orbit.begin(contact(8, 200.0, false)));
    assert!(orbit.begin(contact(8, 200.0, true)));
    assert!(!orbit.begin(contact(9, 300.0, true)));
    assert!(!orbit.move_contact(9, (400.0, 100.0), SurfaceCamera::default()));
    let initial = SurfaceCamera::default();
    assert!(orbit.move_contact(8, (230.0, 250.0), initial));
    let pinched = orbit.step(initial, 16.0);
    assert!(pinched.zoom > initial.zoom);
    assert_eq!(
        (pinched.yaw, pinched.pitch, pinched.pan_x, pinched.pan_y),
        (initial.yaw, initial.pitch, 0.0, 0.0)
    );
    assert!(orbit.end(7));
    assert!(!orbit.moving());
    assert!(orbit.move_contact(8, (230.0, 250.0), pinched));
    assert_eq!(orbit.step(pinched, 16.0), pinched);
    assert!(orbit.move_contact(8, (240.0, 250.0), pinched));
    assert!(orbit.step(pinched, 16.0).yaw < pinched.yaw);
}
#[test]
fn surface_momentum_continues_after_release_converges_and_new_gesture_stops_it() {
    let mut orbit = SurfaceOrbit {
        bounds: Some(bounds()),
        ..Default::default()
    };
    let contact = SurfaceContact {
        id: 1,
        point: (150.0, 250.0),
        touch: false,
        pan: false,
    };
    orbit.begin(contact);
    orbit.move_contact(1, (170.0, 260.0), SurfaceCamera::default());
    let initial = SurfaceCamera::default();
    let during = orbit.step(initial, 16.0);
    orbit.end(1);
    let after = orbit.step(during, 16.0);
    assert!(after.yaw < during.yaw && after.pitch > during.pitch);
    let mut camera = after;
    for _ in 0..200 {
        camera = orbit.step(camera, 16.0);
    }
    assert!(!orbit.moving());
    assert_eq!(orbit.step(camera, 16.0), camera);
    orbit.zoom_impulse(camera, 1.0);
    assert!(orbit.moving());
    orbit.begin(contact);
    assert!(!orbit.moving());
    assert_eq!(orbit.step(camera, 16.0), camera);
}
#[test]
fn surface_zoom_limits_pan_consumption_and_cancel_state_are_finite() {
    let mut orbit = SurfaceOrbit {
        bounds: Some(bounds()),
        ..Default::default()
    };
    orbit.begin(SurfaceContact {
        id: 1,
        point: (150.0, 250.0),
        touch: false,
        pan: true,
    });
    orbit.move_contact(1, (170.0, 260.0), SurfaceCamera::default());
    let panned = orbit.step(SurfaceCamera::default(), 16.0);
    assert!(panned.pan_x > 0.0 && panned.pan_y < 0.0);
    assert!(!orbit.moving());
    assert_eq!(orbit.step(panned, 16.0), panned);
    for direction in [-1.0, 1.0] {
        let mut camera = SurfaceCamera::default();
        for _ in 0..3000 {
            orbit.zoom_impulse(camera, direction);
            camera = orbit.step(camera, 50.0);
        }
        let limit = if direction > 0.0 {
            151.0 / 21.0
        } else {
            151.0 / 401.0
        };
        assert!((camera.zoom - limit).abs() < 1e-9);
    }
    // The same complete reset is used by Escape/Home/blur/data replacement;
    // dropped components additionally invalidate the async generation.
    orbit = SurfaceOrbit::default();
    assert!(!orbit.engaged() && !orbit.moving() && orbit.bounds.is_none());
    assert_eq!(orbit.step(panned, 16.0), panned);
}
#[test]
fn zoom_drag_uses_start_region_and_initial_window_for_pan_and_bounds() {
    let initial = ChartZoom::new(0.3, 0.6);
    let full_bounds = SvgBounds {
        left: 0.0,
        top: 0.0,
        width: 800.0,
        height: 600.0,
        view_width: 800.0,
        view_height: 600.0,
    };
    let mut drag = ChartDrag {
        pointer: 1,
        start: (200.0, 100.0),
        current: (300.0, 900.0),
        bounds: Some(full_bounds),
        kind: ZoomDrag::Plot {
            initial,
            brush: false,
        },
    };
    let inside = zoom_drag_window(drag, 50.0, 750.0).unwrap();
    assert!((inside.start - (0.3 - 0.3 / 7.0)).abs() < 1e-10);
    drag.start.1 = 560.0;
    let overview = zoom_drag_window(drag, 50.0, 750.0).unwrap();
    assert!((overview.start - (0.3 + 1.0 / 7.0)).abs() < 1e-10);
    drag.kind = ZoomDrag::Start(initial);
    drag.current.0 = 900.0;
    let start = zoom_drag_window(drag, 50.0, 750.0).unwrap();
    assert_eq!(start.end, 0.6);
    assert!((start.start - 0.5999).abs() < 1e-10);
    drag.kind = ZoomDrag::End(initial);
    drag.current.0 = -500.0;
    let end = zoom_drag_window(drag, 50.0, 750.0).unwrap();
    assert_eq!(end.start, 0.3);
    assert!((end.end - 0.3001).abs() < 1e-10);
}
#[test]
fn brush_clamps_to_plot_and_does_not_steal_overview_or_margin_gestures() {
    let mut drag = ChartDrag {
        pointer: 1,
        start: (200.0, 100.0),
        current: (900.0, 900.0),
        bounds: Some(SvgBounds {
            left: 0.0,
            top: 0.0,
            width: 800.0,
            height: 600.0,
            view_width: 800.0,
            view_height: 600.0,
        }),
        kind: ZoomDrag::Plot {
            initial: ChartZoom::default(),
            brush: true,
        },
    };
    assert_eq!(
        zoom_drag_brush(drag, 50.0, 750.0),
        Some((200.0, 750.0, ChartZoom::default()))
    );
    assert!(zoom_drag_window(drag, 50.0, 750.0).is_none());
    drag.start.1 = 560.0;
    assert!(zoom_drag_brush(drag, 50.0, 750.0).is_none());
    assert!(zoom_drag_window(drag, 50.0, 750.0).is_some());
    drag.start.0 = 20.0;
    assert!(zoom_drag_brush(drag, 50.0, 750.0).is_none());
    assert!(zoom_drag_window(drag, 50.0, 750.0).is_none());
}
#[test]
fn zoom_keeps_bounds_anchor_and_index_endpoints() {
    let zoom = ChartZoom::default();
    assert_eq!(zoom.indices(20_000), 0..2001);
    assert_eq!(zoom.pan(-1.0), zoom);
    let end = zoom.pan(1.0);
    assert_eq!(end.end, 1.0);
    let closer = zoom.zoom(0.5, 0.5);
    assert!((closer.start - 0.025).abs() < 1e-10);
    assert!((closer.end - 0.075).abs() < 1e-10);
    assert_eq!(zoom.indices(0), 0..0);
    assert_eq!(zoom.indices(1), 0..1);
    let invalid = ChartZoom {
        start: 2.0,
        end: -3.0,
    };
    assert_eq!(invalid.indices(20_000), 0..20_000);
    assert_eq!(invalid.pan(0.5), ChartZoom::new(0.0, 1.0));
    assert_eq!(invalid.zoom(f64::NAN, 0.5), ChartZoom::new(0.0, 1.0));
}
#[test]
fn lttb_preserves_extreme_peak_and_finite_endpoints_with_budget() {
    let mut samples = vec![0.0; 20_000];
    samples[10_000] = 1000.0;
    let selected = chart_lttb(&samples, 100);
    assert_eq!(selected.len(), 100);
    assert_eq!(selected[0], 0);
    assert_eq!(*selected.last().unwrap(), 19_999);
    assert!(selected.contains(&10_000));
    assert!(selected.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(
        chart_lttb(&[f64::NAN, 1.0, f64::INFINITY, 3.0], 20),
        vec![1, 3]
    );
}
#[test]
fn surface_holes_remove_faces_and_normalized_camera_stays_finite() {
    let full = ChartSurface::from_equation(41, |x, y| {
        Some((x * std::f64::consts::PI).sin() * (y * std::f64::consts::PI).sin())
    });
    let hole = ChartSurface::from_equation(41, |x, y| {
        if x.abs() < 0.1 && y.abs() < 0.1 {
            None
        } else {
            Some(x * y)
        }
    });
    assert_eq!(
        surface_faces(&full, SurfaceCamera::default(), 800., 600.).len(),
        1600
    );
    assert_eq!(
        surface_faces(&hole, SurfaceCamera::default(), 800., 600.).len(),
        1584
    );
    let camera = SurfaceCamera {
        yaw: f64::NAN,
        pitch: 900.,
        zoom: f64::INFINITY,
        pan_x: 99.,
        pan_y: -99.,
    }
    .normalized();
    assert_eq!(camera.pitch, 90.);
    assert_eq!(camera.zoom, 1.);
    assert_eq!((camera.pan_x, camera.pan_y), (99.0, -99.0));
    assert!(
        camera
            .project(SurfacePoint::default(), 800., 600.)
            .0
            .is_finite()
    );
}

#[test]
fn surface_pick_uses_source_vertex_in_hit_triangle_not_quad_average() {
    let surface = ChartSurface::from_equation(2, |x, y| Some(x * y));
    for camera in [
        SurfaceCamera::default(),
        SurfaceCamera {
            yaw: 210.0,
            pitch: -35.0,
            zoom: 1.8,
            pan_x: 0.1,
            pan_y: -0.2,
        },
    ] {
        let faces = surface_faces(&surface, camera, 800.0, 624.0);
        let vertices = faces[0].vertices;
        // Within the 0–1–2 triangle, distinctly nearest its 1 corner.
        let point =
            surface_interpolate([vertices[0], vertices[1], vertices[2]], [0.08, 0.84, 0.08]);
        let cursor = camera.project(point, 800.0, 624.0);
        let pick = surface_pick(&faces, (cursor.0, cursor.1)).unwrap();
        assert_eq!(pick.index, Some(1));
        assert_eq!(pick.point, vertices[1]);
        assert_eq!(pick.point, surface.points[1].unwrap());
    }
    assert!(
        surface_triangle_hit(
            [(0.0, 0.0, 0.0), (1.0, 1.0, 1.0), (2.0, 2.0, 2.0)],
            (1.0, 1.0)
        )
        .is_none()
    );
}

#[test]
fn surface_pick_tests_depth_at_cursor_before_nearest_vertex() {
    let camera = SurfaceCamera {
        yaw: 0.0,
        pitch: 90.0,
        ..SurfaceCamera::default()
    };
    let far = ChartSurface::from_equation(2, |_, _| Some(-0.8));
    let near = ChartSurface::from_equation(2, |_, _| Some(0.8));
    let mut faces = surface_faces(&near, camera, 800.0, 600.0);
    // Deliberately append the rear face last: picking must not use SVG order.
    faces.extend(surface_faces(&far, camera, 800.0, 600.0));
    let cursor = camera.project(
        SurfacePoint {
            x: 0.8,
            y: -0.8,
            z: 0.8,
        },
        800.0,
        600.0,
    );
    let pick = surface_pick(&faces, (cursor.0, cursor.1)).unwrap();
    assert_eq!(
        pick.point,
        SurfacePoint {
            x: 1.0,
            y: -1.0,
            z: 0.8
        }
    );
    let step = surface_inspect_step(&faces, 800.0, 600.0, None, true).unwrap();
    assert_eq!(step.point.z, 0.8);
    // Depth interpolates along a sloping triangle instead of its mean.
    let (weights, depth) = surface_triangle_hit(
        [(0.0, 0.0, -2.0), (100.0, 0.0, 2.0), (0.0, 100.0, -2.0)],
        (90.0, 5.0),
    )
    .unwrap();
    assert!((depth - 1.6).abs() < 1e-10);
    assert!((weights[1] - 0.9).abs() < 1e-10);
}

#[test]
fn surface_hole_never_reports_missing_or_disconnected_samples() {
    let camera = SurfaceCamera {
        yaw: 0.0,
        pitch: 90.0,
        ..SurfaceCamera::default()
    };
    let surface = ChartSurface::from_equation(41, |x, y| {
        if x.abs() < 0.1 && y.abs() < 0.1 {
            None
        } else {
            Some(0.0)
        }
    });
    let faces = surface_faces(&surface, camera, 800.0, 600.0);
    assert!(surface_pick(&faces, (400.0, 300.0)).is_none());
    let wall = surface_wall_pick(camera, 800.0, 600.0, (400.0, 300.0)).unwrap();
    assert_eq!(wall.index, None);
    assert!((wall.point.z + 1.0).abs() < 1e-10);
    assert!(surface_wall_pick(camera, 800.0, 600.0, (0.0, 0.0)).is_none());
    let isolated = ChartSurface {
        columns: 2,
        points: vec![Some(SurfacePoint::default()), None, None, None],
    };
    let isolated_faces = surface_faces(&isolated, camera, 800.0, 600.0);
    assert!(surface_pick(&isolated_faces, (400.0, 300.0)).is_none());
    assert!(surface_inspect_step(&isolated_faces, 800.0, 600.0, None, true).is_none());
    let first = surface_inspect_step(&faces, 800.0, 600.0, None, true).unwrap();
    let next = surface_inspect_step(&faces, 800.0, 600.0, first.index, true).unwrap();
    assert_ne!(first.index, next.index);
    assert_eq!(
        surface_inspect_step(&faces, 800.0, 600.0, next.index, false),
        Some(first)
    );
    assert!(surface.points[next.index.unwrap()].is_some());
}

#[test]
fn surface_pointer_projects_nine_lines_to_camera_visible_walls() {
    let point = SurfacePoint {
        x: 0.35,
        y: -0.2,
        z: 0.65,
    };
    for camera in [
        SurfaceCamera::default(),
        SurfaceCamera {
            yaw: 220.0,
            pitch: -20.0,
            ..SurfaceCamera::default()
        },
    ] {
        let back = surface_back_corner(camera);
        let lines = surface_pointer_lines(point, camera);
        assert_eq!(lines.len(), 9);
        assert!(
            lines[..2]
                .iter()
                .all(|(a, b)| a.z == back.z && b.z == back.z)
        );
        assert!(
            lines[2..4]
                .iter()
                .all(|(a, b)| a.x == back.x && b.x == back.x)
        );
        assert!(
            lines[4..6]
                .iter()
                .all(|(a, b)| a.y == back.y && b.y == back.y)
        );
        assert!(lines[6..].iter().all(|(a, _)| *a == point));
        assert_eq!(lines[6].1, SurfacePoint { z: back.z, ..point });
        assert_eq!(lines[7].1, SurfacePoint { x: back.x, ..point });
        assert_eq!(lines[8].1, SurfacePoint { y: back.y, ..point });
    }
}

#[test]
fn surface_source_peak_inspection_is_stable_in_native_probe_geometry() {
    let surface = ChartSurface::from_equation(41, |x, y| {
        if x.abs() < 0.1 && y.abs() < 0.1 {
            None
        } else {
            Some((x * std::f64::consts::PI).sin() * (y * std::f64::consts::PI).sin())
        }
    });
    let camera = SurfaceCamera::default();
    let faces = surface_faces(&surface, camera, 640.0, 400.0);
    let point = SurfacePoint {
        x: 0.5,
        y: 0.5,
        z: 1.0,
    };
    let projected = camera.project(point, 640.0, 400.0);
    let picked = surface_pick(&faces, (projected.0, projected.1)).unwrap();
    assert_eq!(
        picked,
        SurfaceInspection {
            index: Some(30 * 41 + 30),
            point
        }
    );
    assert!((projected.0 - 413.299).abs() < 0.01);
    assert!((projected.1 - 72.752).abs() < 0.01);
}
#[test]
fn surface_projection_preserves_source_orientation_volume_and_narrow_viewport_clipping() {
    let camera = SurfaceCamera {
        yaw: 0.0,
        pitch: 0.0,
        ..SurfaceCamera::default()
    };
    let origin = camera.project(SurfacePoint::default(), 800.0, 600.0);
    let x = camera.project(
        SurfacePoint {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        800.0,
        600.0,
    );
    let y = camera.project(
        SurfacePoint {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        800.0,
        600.0,
    );
    let z = camera.project(
        SurfacePoint {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        800.0,
        600.0,
    );
    assert!((x.0 - origin.0 - 600.0 * 50.0 / 151.0).abs() < 1e-10);
    assert_eq!(y.0, origin.0);
    assert_eq!(y.1, origin.1);
    assert_eq!(y.2, -1.0); // positive source y extends away from the camera
    assert!(z.1 < origin.1);
    let narrow = camera.project(
        SurfacePoint {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        320.0,
        600.0,
    );
    assert!((narrow.0 - 160.0 - (x.0 - origin.0)).abs() < 1e-10);
    assert!(narrow.0 > 320.0); // source orthographic scale does not fit-to-width
    let reset = SurfaceCamera::default();
    assert_eq!((reset.yaw, reset.pitch), (40.0, 20.0));
    assert_eq!(
        SurfaceCamera { zoom: 0.0, ..reset }.normalized().zoom,
        151.0 / 401.0
    );
    assert_eq!(
        SurfaceCamera {
            zoom: 100.0,
            ..reset
        }
        .normalized()
        .zoom,
        151.0 / 21.0
    );
}
#[test]
fn surface_source_height_maps_client_coordinates_without_changing_zoom_viewport() {
    let source = SvgBounds {
        left: 10.0,
        top: 20.0,
        width: 400.0,
        height: 312.0,
        view_width: 800.0,
        view_height: 624.0,
    };
    assert_eq!(source.point((110.0, 98.0)), (200.0, 156.0));
    let zoom = SvgBounds {
        height: 300.0,
        view_height: 600.0,
        ..source
    };
    assert_eq!(zoom.point((110.0, 95.0)), (200.0, 150.0));
}
#[test]
fn surface_brand_visual_map_reverses_high_to_low_theme_colors_before_lighting() {
    for (dark, colors) in [
        (
            true,
            ["rgb(255 215 50)", "rgb(255 144 0)", "rgb(255 38 64)"],
        ),
        (
            false,
            ["rgb(233 195 42)", "rgb(233 100 1)", "rgb(215 35 57)"],
        ),
    ] {
        for (z, expected) in [-1.0, 0.0, 1.0].into_iter().zip(colors) {
            assert_eq!(surface_color(z, 1.0, dark), expected);
        }
    }
    assert_eq!(surface_color(-0.5, 1.0, true), "rgb(255 180 25)");
    assert!((surface_light([0.0, 0.0, 1.0], SurfaceCamera::default()) - 0.9).abs() < 1e-10);
    assert_eq!(
        surface_light(
            [0.0, 0.0, 1.0],
            SurfaceCamera {
                pitch: -20.0,
                ..SurfaceCamera::default()
            }
        ),
        0.4
    );
}
#[test]
fn surface_grid_retains_three_back_walls_when_camera_crosses_cube() {
    let camera = SurfaceCamera::default();
    assert_eq!(
        surface_back_corner(camera),
        SurfacePoint {
            x: -1.0,
            y: 1.0,
            z: -1.0
        }
    );
    let grid = surface_grid(camera, &[-1.0, -0.5, 0.0, 0.5, 1.0]);
    assert_eq!(grid.len(), 30);
    for (a, b) in grid {
        assert!(
            (a.x == -1.0 && b.x == -1.0)
                || (a.y == 1.0 && b.y == 1.0)
                || (a.z == -1.0 && b.z == -1.0)
        );
    }
    assert_eq!(
        surface_back_corner(SurfaceCamera {
            yaw: 220.0,
            pitch: -20.0,
            ..camera
        }),
        SurfacePoint {
            x: 1.0,
            y: -1.0,
            z: 1.0
        }
    );
}
