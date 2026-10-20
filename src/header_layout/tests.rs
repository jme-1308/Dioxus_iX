use super::*;

#[derive(Clone, Default)]
struct FocusFixture(
    Rc<RefCell<Option<Controller>>>,
    Rc<std::cell::Cell<bool>>,
    Rc<RefCell<Vec<bool>>>,
);
impl PartialEq for FocusFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
fn registered_header(fixture: FocusFixture) -> Element {
    let runtime = overlay_runtime::use_overlay_runtime();
    let model = use_hook(|| {
        let mut model = Model::new(true, false);
        model.mounted = true;
        model.presence = [true; 4];
        Rc::new(RefCell::new(model))
    });
    let read = model.clone();
    let apply = model.clone();
    let veto = fixture.1.clone();
    let requests = fixture.2.clone();
    let node = runtime.register(
        model.borrow().owner.clone(),
        1,
        OverlayKind::Header,
        CloseBehavior::Both,
        Callbacks {
            read: Rc::new(move || {
                let model = read.borrow();
                model.alive.then_some(model.open)
            }),
            before: Rc::new(move |change| {
                requests.borrow_mut().push(change.open);
                !veto.get()
            }),
            apply: Rc::new(move |change| {
                let mut model = apply.borrow_mut();
                model.open = change.open;
                model.apply_focus(change.restore_focus);
            }),
            key: None,
        },
    );
    node.set_ownership(Vec::new(), true);
    node.set_mounted();
    let revision = use_signal(|| 0);
    *fixture.0.borrow_mut() = Some(Controller {
        model,
        revision,
        node,
    });
    rsx! { div {} }
}
#[derive(Clone, Default)]
struct OwnershipFixture(Rc<OwnershipState>);
#[derive(Default)]
struct OwnershipState {
    small: std::cell::Cell<Option<Signal<bool>>>,
    groups: RefCell<std::collections::BTreeMap<String, Vec<NodeHandle>>>,
}
impl PartialEq for OwnershipFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn OwnershipProbe(group: String, fixture: OwnershipFixture) -> Element {
    let ancestors = overlay_runtime::use_overlay_ancestors();
    fixture.0.groups.borrow_mut().insert(group, ancestors);
    VNode::empty()
}
fn ownership_fixture(fixture: OwnershipFixture) -> Element {
    let small = use_signal(|| false);
    fixture.0.small.set(Some(small));
    rsx! {
        HeaderLayout {
            small: small(), discovery_ready: true, more_label: "More",
            secondary: rsx! { OwnershipProbe {group:"secondary",fixture:fixture.clone()} },
            default_content: rsx! { OwnershipProbe {group:"default",fixture:fixture.clone()} },
            overflow: rsx! { OwnershipProbe {group:"overflow",fixture:fixture.clone()} },
            avatar: rsx! { OwnershipProbe {group:"avatar",fixture} }
        }
    }
}
#[test]
fn responsive_header_ownership_changes_without_relocating_slot_components() {
    let fixture = OwnershipFixture::default();
    let mut dom = VirtualDom::new_with_props(ownership_fixture, fixture.clone());
    dom.rebuild_in_place();
    let header = fixture.0.groups.borrow()["overflow"][0].clone();
    let metadata = header.surface_metadata().unwrap();
    assert_eq!(metadata.trigger_id, format!("{}-more", header.id()));
    assert_eq!(metadata.items_host, format!("{}-large", header.id()));
    assert!(metadata.popover_host.is_empty());
    for group in ["secondary", "default", "avatar"] {
        assert!(fixture.0.groups.borrow()[group].is_empty());
    }
    dom.in_scope(ScopeId::ROOT, || fixture.0.small.get().unwrap().set(true));
    settle_focus(&mut dom);
    for group in ["secondary", "default", "overflow"] {
        assert!(fixture.0.groups.borrow()[group] == [header.clone()]);
    }
    assert!(fixture.0.groups.borrow()["avatar"].is_empty());
    assert_eq!(
        header.surface_metadata().unwrap().items_host,
        format!("{}-small", header.id())
    );
    dom.in_scope(ScopeId::ROOT, || fixture.0.small.get().unwrap().set(false));
    settle_focus(&mut dom);
    for group in ["secondary", "default", "avatar"] {
        assert!(fixture.0.groups.borrow()[group].is_empty());
    }
    assert_eq!(
        header.surface_metadata().unwrap().items_host,
        format!("{}-large", header.id())
    );
    assert!(fixture.0.groups.borrow()["overflow"] == [header]);
}

#[derive(Clone)]
struct FocusMount {
    id: String,
    calls: Rc<RefCell<Vec<String>>>,
}
impl dioxus::html::RenderedElementBacking for FocusMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        self.calls.borrow_mut().push(self.id.clone());
        Box::pin(async { Ok(()) })
    }
}
fn install_focus(
    dom: &VirtualDom,
    controller: &Controller,
    id: &str,
    calls: &Rc<RefCell<Vec<String>>>,
) {
    let mount = Rc::new(MountedData::new(FocusMount {
        id: id.into(),
        calls: calls.clone(),
    }));
    dom.in_runtime(|| {
        controller
            .node
            .runtime()
            .unwrap()
            .register_item_mount(id, id, mount)
    });
}
fn settle_focus(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
}

#[test]
fn shared_pointer_cancels_pending_header_focus_without_repeated_renders() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
    dom.rebuild_in_place();
    let controller = fixture.0.borrow().clone().unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    install_focus(&dom, &controller, "registered-field", &calls);
    {
        let mut model = controller.model.borrow_mut();
        model.open = true;
        model.controls = controls(&["registered-field"], None);
        model.focus("first".into());
    }
    dom.in_runtime(|| controller.dispatch_focus());
    let runtime = controller.node.runtime().unwrap();
    assert_eq!(runtime.pending_item_focus_count(), 1);
    {
        let target = controller
            .node
            .capture_registered_focus("registered-field")
            .unwrap();
        let mut model = controller.model.borrow_mut();
        model.preserve_focus(FocusedControl {
            id: "registered-field".into(),
            group: 0,
        });
        model.preserve_target = Some(target);
    }
    let revision = *controller.revision.peek();
    dom.in_runtime(|| controller.pointer_input());
    assert_eq!(runtime.pending_item_focus_count(), 0);
    {
        let model = controller.model.borrow();
        assert_eq!(model.focus, "none");
        assert!(model.preserve.is_none() && model.preserve_target.is_none());
        assert!(model.open, "pointer input alone never closes the surface");
    }
    assert_eq!(*controller.revision.peek(), revision + 1);
    dom.in_runtime(|| controller.pointer_input());
    assert_eq!(
        *controller.revision.peek(),
        revision + 1,
        "ordinary pointer input does not request a render without pending Header intent"
    );
    settle_focus(&mut dom);
    assert!(
        calls.borrow().is_empty(),
        "the canceled job cannot steal native pointer focus"
    );
    assert!(
        fixture.2.borrow().is_empty(),
        "no synthetic close request or veto bypass"
    );
}

#[test]
fn registered_header_opening_edges_and_escape_return_use_current_mounted_controls() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
    dom.rebuild_in_place();
    let controller = fixture.0.borrow().clone().unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let more = format!("{}-more", controller.model.borrow().owner);
    for id in ["registered-field", "registered-action", more.as_str()] {
        install_focus(&dom, &controller, id, &calls);
    }
    {
        let mut model = controller.model.borrow_mut();
        model.open = true;
        model.focus("first".into());
    }
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    assert!(
        calls.borrow().is_empty(),
        "opening waits for visible candidate facts"
    );
    controller.model.borrow_mut().controls = controls(
        &["", "unregistered", "registered-field", "registered-action"],
        None,
    );
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    controller.model.borrow_mut().focus("last".into());
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    {
        let mut model = controller.model.borrow_mut();
        model.open = false;
        model.apply_focus(true);
    }
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    assert_eq!(
        calls.borrow().as_slice(),
        ["registered-field", "registered-action", more.as_str()]
    );
}

#[test]
fn header_tab_wrap_requires_a_registered_target_and_keeps_interior_tab_native() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
    dom.rebuild_in_place();
    let controller = fixture.0.borrow().clone().unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    install_focus(&dom, &controller, "registered-action", &calls);
    {
        let mut model = controller.model.borrow_mut();
        model.open = true;
        model.controls = controls(&["unregistered", "registered-action"], None);
    }
    let mut fact = key_input(&controller.model.borrow());
    fact.key = "Tab".into();
    fact.focus_count = 2;
    fact.focused_index = Some(1);
    let mut decision = decide(&mut controller.model.borrow_mut(), &fact, false);
    dom.in_runtime(|| controller.filter_tab_focus(&fact, &mut decision));
    assert!(!decision.prevent && !decision.stop && decision.focus.is_none());
    fact.shift = true;
    fact.focused_index = Some(0);
    let mut decision = decide(&mut controller.model.borrow_mut(), &fact, false);
    dom.in_runtime(|| controller.filter_tab_focus(&fact, &mut decision));
    assert!(decision.prevent && decision.stop);
    controller.model.borrow_mut().focus(decision.focus.unwrap());
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    assert_eq!(calls.borrow().as_slice(), ["registered-action"]);
    fact.shift = false;
    fact.focused_index = Some(0);
    assert!(!decide(&mut controller.model.borrow_mut(), &fact, false).prevent);
}

#[test]
fn responsive_focus_keeps_registered_visible_control_and_rejects_replacement_or_outside_focus() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
    dom.rebuild_in_place();
    let controller = fixture.0.borrow().clone().unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    install_focus(&dom, &controller, "registered-field", &calls);
    let captured = controller
        .node
        .capture_registered_focus("registered-field")
        .unwrap();
    {
        let mut model = controller.model.borrow_mut();
        model.open = true;
        model.small = false;
        model.controls = vec![control("registered-field", 1, 0, true, false)];
        assert!(!model.sync_mode(true, false));
        model.preserve_target = Some(captured.clone());
        model.focus_generation = model.generation;
        model.controls = vec![control("registered-field", 1, 0, true, true)];
    }
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    assert_eq!(calls.borrow().as_slice(), ["registered-field"]);
    controller.model.borrow_mut().preserve_focus(capture(1));
    controller.model.borrow_mut().preserve_target = Some(captured.clone());
    install_focus(&dom, &controller, "registered-field", &calls);
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    assert_eq!(
        calls.borrow().len(),
        1,
        "replacement cannot inherit an old snapshot"
    );
    let current = controller
        .node
        .capture_registered_focus("registered-field")
        .unwrap();
    {
        let mut model = controller.model.borrow_mut();
        model.preserve_focus(capture(1));
        model.preserve_target = Some(current);
        for control in &mut model.controls {
            control.active = false;
        }
        model.active_default = false;
    }
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    assert_eq!(
        calls.borrow().len(),
        1,
        "outside focus is not stolen by mode preservation"
    );
    let current = controller
        .node
        .capture_registered_focus("registered-field")
        .unwrap();
    {
        let mut model = controller.model.borrow_mut();
        model.preserve_focus(capture(1));
        model.preserve_target = Some(current);
        model.active_default = true;
        model.controls.clear();
    }
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    assert_eq!(
        calls.borrow().len(),
        1,
        "a hidden registered control is not revived"
    );
    controller.model.borrow_mut().invalidate();
    dom.in_runtime(|| controller.dispatch_focus());
    settle_focus(&mut dom);
    assert_eq!(calls.borrow().len(), 1);
}

fn control(id: &str, group: u8, tab_index: i32, active: bool, in_popup: bool) -> PresentedControl {
    PresentedControl {
        id: id.into(),
        group: Some(group),
        tab_index,
        active,
        in_popup,
    }
}
fn controls(ids: &[&str], active: Option<usize>) -> Vec<PresentedControl> {
    ids.iter()
        .enumerate()
        .map(|(index, id)| control(id, 2, 0, active == Some(index), true))
        .collect()
}
#[test]
fn canonical_controls_derive_dom_order_boundaries_and_current_focus() {
    let mut model = Model::new(true, false);
    model.controls = vec![
        control("more", 4, 0, false, false),
        control("", 2, 0, false, true),
        control("field", 2, 0, false, true),
        control("positive", 2, 3, true, true),
        control("negative", 1, -1, false, true),
    ];
    assert_eq!(
        model.popup_ids(),
        ["", "field", "positive"],
        "Header keeps its existing DOM-order policy"
    );
    assert_eq!(
        model.popup_controls().count(),
        3,
        "unidentified controls retain their boundary position"
    );
    assert_eq!(
        model.popup_controls().position(|control| control.active),
        Some(2)
    );
    assert_eq!(
        model.focused_control(),
        Some(FocusedControl {
            id: "positive".into(),
            group: 2
        })
    );
    assert!(
        model.presented("negative"),
        "negative tabindex can still preserve a current mounted control"
    );
    assert!(
        !model.presented(""),
        "an unidentified control is never a registered destination"
    );
    for (index, control) in model.controls.iter_mut().enumerate() {
        control.active = index == 4;
    }
    assert_eq!(
        model.popup_controls().position(|control| control.active),
        None
    );
    assert_eq!(
        model.focused_control(),
        Some(FocusedControl {
            id: "negative".into(),
            group: 1
        })
    );
    for (index, control) in model.controls.iter_mut().enumerate() {
        control.active = index == 1;
    }
    assert!(model.focused_control().is_none());
}
#[test]
fn canonical_focus_projection_requires_current_group_registration_and_exact_focus() {
    let candidate =
        |id: &str, hosts: &[&str], active: bool| crate::popover_focus::NativeFocusCandidate {
            target: id.into(),
            tab_index: 0,
            autofocus: false,
            hosts: hosts.iter().map(|host| (*host).into()).collect(),
            nearest_host: String::new(),
            popover: String::new(),
            exact_active: active,
            contains_active: !active,
        };
    let observation = FocusObservation {
        candidates: vec![
            candidate("outside", &["header"], false),
            candidate("field", &["header", "nested"], true),
            candidate("inline", &[], false),
            candidate("header-more", &[], false),
        ],
        active_id: "field".into(),
        active_default: false,
    };
    let groups = BTreeMap::from([("inline".into(), 1), ("field".into(), 2)]);
    let projected = presented_controls(&observation, &groups, "header");
    assert_eq!(
        projected
            .iter()
            .map(|control| control.id.as_str())
            .collect::<Vec<_>>(),
        ["field", "inline", "header-more"]
    );
    assert!(projected[0].active && projected[0].in_popup);
    assert_eq!(projected[0].group, Some(2));
    assert!(!projected[1].active && !projected[1].in_popup);
    assert_eq!(projected[2].group, Some(4));
    let mut retired = groups;
    retired.remove("field");
    assert!(
        !presented_controls(&observation, &retired, "header")
            .iter()
            .any(|control| control.id == "field")
    );
}

fn key_input(_: &Model) -> KeyInput {
    KeyInput {
        focus_count: 6,
        focused_index: Some(0),
        inside: true,
        ..KeyInput::default()
    }
}
#[test]
fn mounted_popup_placement_clamps_and_rejects_obsolete_layout_work() {
    assert_eq!(
        popup_position(1000.0, 64.0, 48.0, 240.0),
        Some(PopupPosition {
            left: 64.0,
            top: 48.0
        })
    );
    assert_eq!(
        popup_position(360.0, 330.0, 48.0, 200.0),
        Some(PopupPosition {
            left: 152.0,
            top: 48.0
        })
    );
    assert_eq!(
        popup_position(360.0, -20.0, 48.0, 344.0),
        Some(PopupPosition {
            left: 8.0,
            top: 48.0
        })
    );
    assert!(
        popup_position(360.0, 16.0, 48.0, 0.0).is_none(),
        "hidden native surface waits for its visible resize"
    );
    assert!(popup_position(f64::NAN, 16.0, 48.0, 200.0).is_none());
    let mut model = Model::new(true, true);
    model.mounted = true;
    model.open = true;
    model.viewport_width = 360.0;
    let epoch = model.layout_epoch;
    let generation = model.generation;
    assert!(model.accepts_layout(epoch, generation, true));
    model.position = popup_position(360.0, 330.0, 48.0, 200.0);
    assert!(
        model
            .popup_style(true)
            .unwrap()
            .contains("max-width:344px;left:152px;top:48px;")
    );
    assert!(
        model.popup_style(false).is_none(),
        "inactive host retains no geometry writes"
    );
    model.invalidate_layout();
    assert!(!model.accepts_layout(epoch, generation, true));
    let epoch = model.layout_epoch;
    model.sync_mode(false, true);
    assert!(
        !model.accepts_layout(epoch, generation, true),
        "responsive replacement invalidates pending measurement"
    );
    let epoch = model.layout_epoch;
    let generation = model.generation;
    model.open = false;
    assert!(!model.accepts_layout(epoch, generation, false));
    assert!(model.popup_style(false).is_none());
    model.open = true;
    model.invalidate();
    assert!(
        !model.accepts_layout(epoch, generation, false),
        "unmounted owner cannot publish geometry"
    );
}

#[test]
fn decision_preserves_opening_defaults_veto_intent_and_boundary_keys() {
    let mut model = Model::new(true, false);
    model.mounted = true;
    model.presence = [true; 4];
    for key in ["Home", "End", "ArrowUp", "ArrowDown", "Enter", " "] {
        let mut fact = key_input(&model);
        fact.more = true;
        fact.key = key.into();
        let decision = decide(&mut model, &fact, false);
        assert!(decision.prevent);
        assert_eq!(decision.request, Some(true));
        assert!(!decision.stop);
    }
    model.open = true;
    let mut trigger = key_input(&model);
    trigger.more = true;
    trigger.key = "Enter".into();
    let decision = decide(&mut model, &trigger, false);
    assert!(!decision.prevent);
    assert_eq!(
        decision.request, None,
        "open trigger retains native click activation"
    );
    model.open = false;
    trigger.alt = true;
    let decision = decide(&mut model, &trigger, false);
    assert!(!decision.prevent);
    assert_eq!(decision.request, Some(true));
    assert_eq!(decision.focus, None);
    model.open = true;
    let mut fact = key_input(&model);
    fact.key = "Escape".into();
    let decision = decide(&mut model, &fact, false);
    assert!(decision.prevent && decision.stop);
    assert_eq!(decision.request, Some(false));
    assert_eq!(decision.focus.as_deref(), Some("more"));
    fact.key = "Tab".into();
    fact.focused_index = Some(1);
    assert!(!decide(&mut model, &fact, false).prevent);
    fact.focused_index = Some(5);
    assert_eq!(
        decide(&mut model, &fact, false).focus.as_deref(),
        Some("index:0"),
    );
    fact.shift = true;
    fact.focused_index = Some(0);
    assert_eq!(
        decide(&mut model, &fact, false).focus.as_deref(),
        Some("index:5"),
    );
}
#[test]
fn popover_content_tab_does_not_use_standalone_header_wrap_or_more_focus() {
    let mut model = Model::new(true, true);
    model.open = true;
    model.presence = [true; 4];
    for more in [false, true] {
        for shift in [false, true] {
            for index in [None, Some(0), Some(5)] {
                let mut fact = key_input(&model);
                fact.key = "Tab".into();
                fact.more = more;
                fact.shift = shift;
                fact.focused_index = index;
                let result = decide(&mut model, &fact, true);
                assert!(!result.prevent && !result.stop);
                assert_eq!(result.request, None);
                assert_eq!(result.focus, None);
                assert!(model.open, "capture cannot close or redirect More Tab");
            }
        }
    }
    let mut ordinary = key_input(&model);
    ordinary.key = "Tab".into();
    ordinary.focused_index = Some(5);
    assert_eq!(
        decide(&mut model, &ordinary, false).focus.as_deref(),
        Some("index:0"),
        "standalone Header boundary behavior is retained"
    );
}

#[test]
fn only_live_open_more_tab_in_popover_enters_shared_release_policy() {
    let mut model = Model::new(true, true);
    model.open = true;
    model.mounted = true;
    let mut facts = KeyFacts {
        key: "Tab".into(),
        on_trigger: true,
        popover_ancestor: true,
        ..KeyFacts::default()
    };
    assert!(releases_more_tab(&model, &facts));
    facts.on_trigger = false;
    facts.inside = true;
    assert!(
        !releases_more_tab(&model, &facts),
        "ordinary content is not More"
    );
    facts.on_trigger = true;
    facts.popover_ancestor = false;
    assert!(!releases_more_tab(&model, &facts));
    facts.popover_ancestor = true;
    facts.default_prevented = true;
    assert!(!releases_more_tab(&model, &facts));
    facts.default_prevented = false;
    facts.key = "Escape".into();
    assert!(!releases_more_tab(&model, &facts));
    facts.key = "Tab".into();
    model.open = false;
    assert!(!releases_more_tab(&model, &facts));
    model.open = true;
    model.invalidate();
    assert!(!releases_more_tab(&model, &facts));
}

#[test]
fn small_projection_preserves_opening_without_reusing_private_focus_work() {
    let mut model = Model::new(false, true);
    model.mounted = true;
    model.open = true;
    model.presence = [true; 4];
    model.focus("last".into());
    let previous = (model.layout_epoch, model.generation, model.small);
    assert!(
        !model.sync_mode(true, true),
        "slot movement must not request close"
    );
    assert!(model.open && model.small && model.more_visible());
    assert!(
        !model.accepts_layout(previous.0, previous.1, previous.2),
        "old physical host generation is stale"
    );
    assert_eq!(model.focus, "none");
    assert!(!model.sync_mode(true, true), "same props are inert");
    assert!(model.open);
    assert!(
        model.sync_mode(true, false),
        "top-layer configuration remains separate"
    );
    assert!(!model.open);
}

fn capture(group: u8) -> FocusedControl {
    FocusedControl {
        id: "registered-field".into(),
        group,
    }
}

fn pending_capture(group: u8) -> Model {
    let mut model = Model::new(false, true);
    model.mounted = true;
    model.open = true;
    model.presence = [true; 4];
    model.controls = vec![control("registered-field", group, 0, true, false)];
    assert!(!model.sync_mode(true, true));
    model
}

#[test]
fn mode_focus_preservation_uses_current_group_presentation() {
    for (small, open, expected) in [
        (false, false, [true, true, false, true, true]),
        (false, true, [true, true, true, true, true]),
        (true, false, [false, false, false, true, true]),
        (true, true, [true, true, true, true, true]),
    ] {
        let mut model = Model::new(small, true);
        model.open = open;
        model.presence = [true; 4];
        for (group, expected) in expected.into_iter().enumerate() {
            assert_eq!(model.group_presented(group as u8), expected);
        }
        assert!(!model.group_presented(5));
    }
    let mut no_overflow = Model::new(false, true);
    no_overflow.presence = [false, true, false, false];
    assert!(!no_overflow.group_presented(4));
    no_overflow.small = true;
    assert!(no_overflow.group_presented(4));

    let mut closed = Model::new(false, true);
    closed.mounted = true;
    closed.controls = vec![control("registered-field", 1, 0, true, false)];
    assert!(!closed.sync_mode(true, true));
    assert_eq!(closed.focus, "none", "hidden Small content is not revived");
    assert!(closed.preserve.is_none());
}

#[test]
fn accepted_close_retains_only_visible_pending_mode_capture() {
    let mut large_default = pending_capture(1);
    large_default.small = false;
    large_default.open = false;
    large_default.apply_focus(false);
    assert_eq!(large_default.preserve, Some(capture(1)));

    for group in [1, 2] {
        let mut hidden = pending_capture(group);
        hidden.open = false;
        hidden.apply_focus(false);
        assert!(hidden.preserve.is_none(), "closed Small group {group}");
        assert_eq!(hidden.focus, "none");
    }
    let mut restoring = pending_capture(1);
    restoring.open = false;
    restoring.apply_focus(true);
    assert_eq!(restoring.focus, "more");
    assert!(restoring.preserve.is_none());

    let mut hidden_more = pending_capture(1);
    hidden_more.small = false;
    hidden_more.presence[2] = false;
    hidden_more.open = false;
    assert!(!hidden_more.more_visible());
    hidden_more.apply_focus(true);
    assert_eq!(hidden_more.preserve, Some(capture(1)));
}

#[test]
fn rust_presence_and_layout_reconciliation_reject_stale_mounted_work() {
    let mut model = Model::new(false, false);
    model.mounted = true;
    model.presence = [true, true, false, true];
    assert!(!model.more_visible());
    model.small = true;
    assert!(model.more_visible());
    model.open = true;
    let captured = (model.layout_epoch, model.generation, model.small);
    assert!(model.accepts_layout(captured.0, captured.1, captured.2));
    assert!(model.sync_mode(false, true));
    assert!(!model.open);
    assert!(!model.accepts_layout(captured.0, captured.1, captured.2));
    model.open = true;
    let current = (model.layout_epoch, model.generation, model.small);
    assert!(model.accepts_layout(current.0, current.1, current.2));
    model.invalidate();
    assert!(!model.accepts_layout(current.0, current.1, current.2));
}

struct TypedKey(Key, Modifiers);
impl dioxus::html::ModifiersInteraction for TypedKey {
    fn modifiers(&self) -> Modifiers {
        self.1
    }
}
impl dioxus::html::HasKeyboardData for TypedKey {
    fn key(&self) -> Key {
        self.0.clone()
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Unidentified
    }
    fn location(&self) -> dioxus::html::Location {
        dioxus::html::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
fn key_event(key: Key, shift: bool) -> KeyboardEvent {
    Event::new(
        Rc::new(dioxus::html::KeyboardData::new(TypedKey(
            key,
            if shift {
                Modifiers::SHIFT
            } else {
                Modifiers::empty()
            },
        ))),
        true,
    )
}
#[test]
fn direct_more_keys_and_native_field_boundaries_use_registered_focus() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
    dom.rebuild_in_place();
    let controller = fixture.0.borrow().clone().unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let more = format!("{}-more", controller.model.borrow().owner);
    for id in ["first-field", "last-action", more.as_str()] {
        install_focus(&dom, &controller, id, &calls);
    }
    {
        let mut model = controller.model.borrow_mut();
        model.controls = controls(&["first-field", "last-action"], Some(0));
    }
    let down = key_event(Key::ArrowDown, false);
    dom.in_runtime(|| controller.key(down.clone(), true, None));
    settle_focus(&mut dom);
    assert!(controller.model.borrow().open && !down.default_action_enabled() && !down.propagates());
    assert_eq!(calls.borrow().as_slice(), ["first-field"]);
    let home = key_event(Key::Home, false);
    dom.in_runtime(|| controller.key(home.clone(), false, Some(true)));
    assert!(
        home.default_action_enabled() && home.propagates(),
        "native editing remains native"
    );
    let interior = key_event(Key::Tab, false);
    dom.in_runtime(|| controller.key(interior.clone(), false, Some(true)));
    assert!(interior.default_action_enabled() && interior.propagates());
    for (index, control) in controller
        .model
        .borrow_mut()
        .controls
        .iter_mut()
        .enumerate()
    {
        control.active = index == 1;
    }
    let boundary = key_event(Key::Tab, false);
    dom.in_runtime(|| controller.key(boundary.clone(), false, Some(true)));
    settle_focus(&mut dom);
    assert!(!boundary.default_action_enabled() && !boundary.propagates());
    assert_eq!(calls.borrow().as_slice(), ["first-field", "first-field"]);
    let escape = key_event(Key::Escape, false);
    dom.in_runtime(|| controller.key(escape.clone(), false, Some(true)));
    settle_focus(&mut dom);
    assert!(
        !controller.model.borrow().open && !escape.default_action_enabled() && !escape.propagates()
    );
    assert_eq!(
        calls.borrow().as_slice(),
        ["first-field", "first-field", more.as_str()]
    );
    fixture.1.set(true);
    let end = key_event(Key::End, false);
    dom.in_runtime(|| controller.key(end.clone(), true, None));
    settle_focus(&mut dom);
    assert!(!controller.model.borrow().open && !end.default_action_enabled());
    assert_eq!(calls.borrow().len(), 3, "veto never moves focus");
    assert_eq!(*fixture.2.borrow(), [true, false, true]);
}

struct Click;
impl dioxus::html::InteractionLocation for Click {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for Click {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for Click {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for Click {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for Click {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
fn click() -> MouseEvent {
    Event::new(Rc::new(dioxus::html::MouseData::new(Click)), true)
}

#[test]
fn more_click_toggles_current_owner_and_dismisses_peers_independently_of_veto() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
    dom.rebuild_in_place();
    let controller = fixture.0.borrow().clone().unwrap();
    let peer = Rc::new(std::cell::Cell::new(true));
    dom.in_scope(ScopeId::ROOT, || {
        let read = peer.clone();
        let apply = peer.clone();
        let peer_node = controller.node.runtime().unwrap().register(
            "click-peer".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(|_| true),
                apply: Rc::new(move |change| apply.set(change.open)),
                key: None,
            },
        );
        peer_node.set_ownership(Vec::new(), true);
        peer_node.set_mounted();
    });
    let event = click();
    dom.in_runtime(|| controller.more_click(event.clone()));
    assert!(controller.model.borrow().open);
    assert!(!peer.get());
    assert!(
        event.default_action_enabled() && !event.propagates(),
        "only virtual ancestors are stopped"
    );
    assert_eq!(*fixture.2.borrow(), [true]);
    let event = click();
    dom.in_runtime(|| controller.more_click(event));
    assert!(!controller.model.borrow().open);
    fixture.1.set(true);
    peer.set(true);
    let vetoed = click();
    dom.in_runtime(|| controller.more_click(vetoed.clone()));
    assert!(!controller.model.borrow().open);
    assert!(
        !peer.get(),
        "peer dismissal remains independent of the local veto"
    );
    assert!(vetoed.default_action_enabled());
    assert_eq!(*fixture.2.borrow(), [true, false, true]);
    fixture.1.set(false);
    let prevented = click();
    prevented.prevent_default();
    dom.in_runtime(|| controller.more_click(prevented));
    controller.model.borrow_mut().presence = [false; 4];
    dom.in_runtime(|| controller.more_click(click()));
    assert_eq!(
        fixture.2.borrow().len(),
        3,
        "prevented and invisible controls cannot toggle"
    );
    controller.model.borrow_mut().presence = [true; 4];
    dom.in_runtime(|| controller.node.unregister());
    dom.in_runtime(|| controller.more_click(click()));
    assert_eq!(fixture.2.borrow().len(), 3, "a stale owner cannot act");
}

#[test]
fn active_group_clicks_dismiss_without_canceling_form_defaults_and_inline_groups_stay_idle() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(registered_header, fixture.clone());
    dom.rebuild_in_place();
    let controller = fixture.0.borrow().clone().unwrap();
    {
        let mut model = controller.model.borrow_mut();
        model.open = true;
        model.small = false;
    }
    let inline = click();
    dom.in_runtime(|| controller.group_click(1, inline.clone()));
    assert!(
        controller.model.borrow().open && inline.propagates() && inline.default_action_enabled()
    );
    let inactive = click();
    dom.in_runtime(|| controller.background_click(true, inactive.clone()));
    assert!(inactive.propagates());
    let background = click();
    dom.in_runtime(|| controller.background_click(false, background.clone()));
    assert!(
        controller.model.borrow().open
            && background.default_action_enabled()
            && !background.propagates()
    );
    let content = click();
    dom.in_runtime(|| controller.group_click(2, content.clone()));
    assert!(!controller.model.borrow().open);
    assert!(
        content.default_action_enabled() && !content.propagates(),
        "native input/form defaults remain enabled"
    );
    {
        let mut model = controller.model.borrow_mut();
        model.open = true;
        model.small = true;
    }
    let prevented = click();
    prevented.prevent_default();
    dom.in_runtime(|| controller.group_click(0, prevented.clone()));
    assert!(controller.model.borrow().open && !prevented.propagates());
    fixture.1.set(true);
    let vetoed = click();
    dom.in_runtime(|| controller.group_click(1, vetoed.clone()));
    assert!(controller.model.borrow().open && vetoed.default_action_enabled());
}
