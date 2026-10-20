use super::*;

#[test]
fn open_tooltip_markup_does_not_make_interactive_content_inert() {
    fn opened() -> Element {
        render_tooltip(
            TooltipForProps {
                id: "open-tooltip".into(),
                controller: Some(use_tooltip("open-tooltip")),
                anchor_id: "anchor".into(),
                anchor_selectors: Vec::new(),
                placement: TooltipPlacement::Top,
                show_delay: 0,
                hide_delay: 50,
                interactive: true,
                animation_frame: false,
                title: None,
                title_icon: None,
                aria_label: None,
                class: String::new(),
                onnativeerror: EventHandler::default(),
                children: rsx! { button { "Tooltip action" } },
            },
            Lifecycle {
                visible: Some("anchor-token".into()),
                ..Default::default()
            },
        )
    }
    let mut dom = VirtualDom::new(opened);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("role=\"tooltip\""), "{html}");
    assert!(html.contains("popover=\"manual\""), "{html}");
    assert!(html.contains("data-open=\"true\""), "{html}");
    assert!(html.contains("Tooltip action"), "{html}");
    assert!(!html.contains(" inert"), "Open tooltip is inert: {html}");

    fn closed() -> Element {
        rsx! { TooltipFor { id: "closed-tooltip", anchor_id: "anchor", "Description" } }
    }
    let mut closed_dom = VirtualDom::new(closed);
    closed_dom.rebuild_in_place();
    let closed = dioxus_ssr::render(&closed_dom);
    assert!(closed.contains("data-open=\"false\""), "{closed}");
    assert!(closed.contains(" inert"), "{closed}");
}

type OwnerCapture = Rc<
    RefCell<
        Option<(
            TooltipController,
            TooltipAnchorBinding,
            TooltipAnchorBinding,
            Signal<bool>,
        )>,
    >,
>;
type ViewportCapture = Rc<RefCell<Option<crate::document_viewport::DocumentViewport>>>;
fn registered_owner_fixture() -> Element {
    let viewport = crate::document_viewport::use_document_viewport(false);
    if let Some(capture) = try_consume_context::<ViewportCapture>() {
        *capture.borrow_mut() = Some(viewport);
    }
    let capture = use_context::<OwnerCapture>();
    let show = use_signal(|| true);
    let controller = use_tooltip("registered-tooltip");
    let first = use_tooltip_anchor(controller.clone(), "first");
    let second = use_tooltip_anchor(controller.clone(), "second");
    *capture.borrow_mut() = Some((controller.clone(), first.clone(), second.clone(), show));
    rsx! {
        button {"aria-describedby":first.describedby("external description"),"First"}
        button {"aria-describedby":second.describedby("other"),"Second"}
        if show() { TooltipFor {id:"registered-tooltip",controller:controller.clone(),interactive:true,"Description"} }
    }
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
#[derive(Clone)]
struct RectMount(Rect);
impl dioxus::html::RenderedElementBacking for RectMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_client_rect(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                >,
        >,
    > {
        let rect = self.0;
        Box::pin(async move {
            Ok(dioxus::html::geometry::PixelsRect::new(
                dioxus::html::geometry::euclid::Point2D::new(rect.x, rect.y),
                dioxus::html::geometry::euclid::Size2D::new(rect.width, rect.height),
            ))
        })
    }
}
fn mounted(rect: Rect) -> Rc<MountedData> {
    Rc::new(MountedData::new(RectMount(rect)))
}

#[test]
fn registered_shared_anchors_preserve_descriptions_and_panel_drop_removes_only_own_token() {
    let capture: OwnerCapture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(registered_owner_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let (controller, first, second, mut show) = capture.borrow().clone().unwrap();
    assert_eq!(
        first.describedby("external description"),
        Some("external description registered-tooltip".into())
    );
    assert_eq!(
        second.describedby("other"),
        Some("other registered-tooltip".into())
    );
    dom.in_scope(ScopeId::ROOT, || show.set(false));
    settle(&mut dom);
    assert!(controller.model.borrow().panel.is_none());
    assert_eq!(
        first.describedby("external registered-tooltip description"),
        Some("external description".into())
    );
    assert_eq!(second.describedby("other"), Some("other".into()));
    dom.in_scope(ScopeId::ROOT, || show.set(true));
    settle(&mut dom);
    assert_eq!(
        second.describedby("other"),
        Some("other registered-tooltip".into())
    );
    drop(dom);
    let model = controller.model.borrow();
    assert!(!model.alive);
    assert!(model.anchors.is_empty());
    assert!(model.panel.is_none());
}

#[test]
fn mounted_measurements_switch_shared_anchors_and_removed_anchor_cannot_reopen() {
    let capture: OwnerCapture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(registered_owner_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let (controller, first, second, _) = capture.borrow().clone().unwrap();
    {
        let mut model = controller.model.borrow_mut();
        model.viewport = (400.0, 300.0);
        model.anchors.get_mut(&first.token).unwrap().mount = Some(mounted(scene().anchor));
        model.anchors.get_mut(&second.token).unwrap().mount = Some(mounted(Rect {
            x: 200.0,
            ..scene().anchor
        }));
        model.panel.as_mut().unwrap().mount = Some(mounted(Rect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 50.0,
        }));
    }
    dom.in_scope(ScopeId::ROOT, || controller.enter(first.token));
    settle(&mut dom);
    assert_eq!(
        controller.model.borrow().current_anchor(),
        Some(first.token)
    );
    assert_eq!(
        controller.model.borrow().coordinates,
        position(&scene(), TooltipPlacement::Top)
    );
    controller.model.borrow_mut().coordinates = None;
    dom.in_scope(ScopeId::ROOT, || controller.measure());
    controller
        .model
        .borrow_mut()
        .anchors
        .get_mut(&first.token)
        .unwrap()
        .mount = Some(mounted(Rect {
        x: 250.0,
        ..scene().anchor
    }));
    settle(&mut dom);
    assert!(
        controller.model.borrow().coordinates.is_none(),
        "old mounted result must not position a replacement anchor"
    );
    dom.in_scope(ScopeId::ROOT, || controller.measure());
    settle(&mut dom);
    assert_eq!(controller.model.borrow().coordinates.unwrap().x, 230.0);
    dom.in_scope(ScopeId::ROOT, || controller.enter(second.token));
    settle(&mut dom);
    assert_eq!(
        controller.model.borrow().current_anchor(),
        Some(second.token)
    );
    assert_eq!(controller.model.borrow().coordinates.unwrap().x, 180.0);
    controller.model.borrow_mut().remove_anchor(second.token);
    assert!(controller.model.borrow().lifecycle.visible.is_none());
    dom.in_scope(ScopeId::ROOT, || controller.enter(second.token));
    settle(&mut dom);
    assert!(controller.model.borrow().lifecycle.visible.is_none());
    dom.in_scope(ScopeId::ROOT, || controller.enter(first.token));
    settle(&mut dom);
    controller.dismiss();
    settle(&mut dom);
    assert!(controller.model.borrow().lifecycle.visible.is_none());
}

#[test]
fn shared_viewport_remeasures_shown_tooltip_and_escape_does_not_replay_on_reopening() {
    let capture: OwnerCapture = Rc::new(RefCell::new(None));
    let viewport_capture: ViewportCapture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(registered_owner_fixture)
        .with_root_context(capture.clone())
        .with_root_context(viewport_capture.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let (controller, first, _, _) = capture.borrow().clone().unwrap();
    let viewport = viewport_capture.borrow().clone().unwrap();
    {
        let mut model = controller.model.borrow_mut();
        model.anchors.get_mut(&first.token).unwrap().mount = Some(mounted(scene().anchor));
        model.panel.as_mut().unwrap().mount = Some(mounted(Rect {
            width: 120.0,
            height: 50.0,
            ..Rect::default()
        }));
    }
    dom.in_scope(ScopeId::ROOT, || {
        viewport.test_layout(400.0, 300.0);
        viewport.test_escape();
    });
    settle(&mut dom);
    assert_eq!(
        controller.model.borrow().viewport,
        (0.0, 0.0),
        "a hidden owner does not consume layout facts"
    );
    dom.in_scope(ScopeId::ROOT, || controller.enter(first.token));
    settle(&mut dom);
    assert_eq!(
        controller.model.borrow().current_anchor(),
        Some(first.token)
    );
    assert_eq!(
        controller.model.borrow().coordinates,
        position(&scene(), TooltipPlacement::Top),
        "the Escape before opening is only a baseline"
    );

    controller
        .model
        .borrow_mut()
        .anchors
        .get_mut(&first.token)
        .unwrap()
        .mount = Some(mounted(Rect {
        x: 200.0,
        ..scene().anchor
    }));
    dom.in_scope(ScopeId::ROOT, || viewport.test_layout(400.0, 300.0));
    settle(&mut dom);
    assert_eq!(
        controller.model.borrow().coordinates.unwrap().x,
        180.0,
        "scroll invalidation remeasures even with unchanged viewport dimensions"
    );
    dom.in_scope(ScopeId::ROOT, || viewport.test_escape());
    settle(&mut dom);
    assert!(controller.model.borrow().lifecycle.visible.is_none());

    dom.in_scope(ScopeId::ROOT, || {
        viewport.test_layout(260.0, 200.0);
        viewport.test_escape();
    });
    settle(&mut dom);
    assert_eq!(controller.model.borrow().viewport, (400.0, 300.0));
    dom.in_scope(ScopeId::ROOT, || controller.enter(first.token));
    settle(&mut dom);
    assert_eq!(
        controller.model.borrow().current_anchor(),
        Some(first.token),
        "a later opening never replays earlier Escape revisions"
    );
    assert_eq!(controller.model.borrow().viewport, (260.0, 200.0));
    assert_eq!(controller.model.borrow().coordinates.unwrap().x, 130.0);
    dom.in_scope(ScopeId::ROOT, || viewport.test_escape());
    settle(&mut dom);
    assert!(
        controller.model.borrow().lifecycle.visible.is_none(),
        "a new Escape still dismisses the reopened tooltip"
    );
}

#[test]
fn interactive_retention_invalidates_pending_close_and_panel_replacement_cancels_old_show() {
    let capture: OwnerCapture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(registered_owner_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let (controller, first, _, mut show) = capture.borrow().clone().unwrap();
    let generation = {
        let mut model = controller.model.borrow_mut();
        model.lifecycle.enter(first.token.to_string()).unwrap()
    };
    {
        let mut model = controller.model.borrow_mut();
        assert!(model.lifecycle.show(generation, first.token.to_string()));
    }
    let close = controller.model.borrow_mut().lifecycle.leave();
    controller.retain();
    controller.model.borrow_mut().lifecycle.hide(close);
    assert_eq!(
        controller.model.borrow().current_anchor(),
        Some(first.token)
    );
    let old_panel = controller.model.borrow().panel.as_ref().unwrap().token;
    dom.in_scope(ScopeId::ROOT, || show.set(false));
    settle(&mut dom);
    dom.in_scope(ScopeId::ROOT, || show.set(true));
    settle(&mut dom);
    assert_ne!(
        controller.model.borrow().panel.as_ref().unwrap().token,
        old_panel
    );
    assert!(
        !controller
            .model
            .borrow_mut()
            .lifecycle
            .show(generation, first.token.to_string())
    );
    assert!(controller.model.borrow().lifecycle.visible.is_none());
}

fn scene() -> Metrics {
    Metrics {
        anchor: Rect {
            x: 100.0,
            y: 100.0,
            width: 80.0,
            height: 30.0,
        },
        width: 120.0,
        height: 50.0,
        viewport_width: 400.0,
        viewport_height: 300.0,
        clips: vec![],
    }
}
#[test]
fn placement_flips_and_shifts_while_arrow_tracks_the_anchor() {
    let mut m = scene();
    let p = position(&m, TooltipPlacement::Top).unwrap();
    assert_eq!(
        (p.x, p.y, p.side, p.arrow),
        (80.0, 38.0, TooltipPlacement::Top, 54.0)
    );
    m.anchor.y = 5.0;
    assert_eq!(
        position(&m, TooltipPlacement::Top).unwrap().side,
        TooltipPlacement::Bottom
    );
    m.anchor.x = 370.0;
    let p = position(&m, TooltipPlacement::Bottom).unwrap();
    assert_eq!(p.x, 270.0);
    assert_eq!(p.arrow, 102.0);
}
#[test]
fn clipped_or_removed_references_cannot_leave_visible_tooltips() {
    let mut m = scene();
    m.clips.push(Rect {
        x: 0.0,
        y: 0.0,
        width: 90.0,
        height: 300.0,
    });
    assert!(position(&m, TooltipPlacement::Top).is_none());
    m.clips.clear();
    m.anchor.width = 0.0;
    assert!(position(&m, TooltipPlacement::Top).is_none());
    m.anchor.width = f64::NAN;
    assert!(position(&m, TooltipPlacement::Top).is_none());
}
#[test]
fn leaving_cancels_a_delayed_open_and_reentry_cancels_a_pending_close() {
    let mut state = Lifecycle::default();
    let open = state.enter("first".into()).unwrap();
    assert!(state.enter("first".into()).is_none());
    assert_eq!(state.generation, open);
    state.leave();
    assert!(!state.show(open, "first".into()));
    let open = state.enter("first".into()).unwrap();
    assert!(state.show(open, "first".into()));
    let close = state.leave();
    assert!(state.enter("first".into()).is_none());
    state.hide(close);
    assert_eq!(state.visible.as_deref(), Some("first"));
    let second = state.enter("second".into()).unwrap();
    assert!(state.show(second, "second".into()));
    assert_eq!(state.visible.as_deref(), Some("second"));
}
#[test]
fn interactive_default_extends_only_the_source_default_hide_delay() {
    let config = Config {
        placement: TooltipPlacement::Top,
        show_delay: 500,
        hide_delay: 50,
        interactive: true,
    };
    assert_eq!(config.hide_delay(), 150);
    assert_eq!(
        Config {
            hide_delay: 75,
            ..config
        }
        .hide_delay(),
        75
    );
    assert_eq!(
        Config {
            interactive: false,
            ..config
        }
        .hide_delay(),
        50
    );
}
