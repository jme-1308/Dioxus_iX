//! Registered submenu relationships, mounted return focus and typed owner dispatch.
use super::*;
use std::cell::Cell;

struct SerializedForm(String);
impl dioxus::html::HasFileData for SerializedForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for SerializedForm {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
fn form(value: serde_json::Value) -> FormEvent {
    Event::new(
        Rc::new(dioxus::html::FormData::new(SerializedForm(
            value.to_string(),
        ))),
        true,
    )
}

#[derive(Default)]
struct FixtureState {
    runtime: RefCell<Option<RuntimeHandle>>,
    node: RefCell<Option<NodeHandle>>,
    open: Cell<bool>,
    veto: Cell<bool>,
    resets: Cell<usize>,
}
#[derive(Clone)]
struct Fixture(Rc<FixtureState>);

fn fixture_root(fixture: Fixture) -> Element {
    let runtime = use_overlay_runtime();
    // This is a real hook-owned handle, not a fabricated private context.
    let handle = crate::dropdown::use_dropdown();
    let state = fixture.clone();
    let node = use_hook(move || {
        let read = state.clone();
        let before = state.clone();
        let apply = state.clone();
        let node = runtime.register(
            "configured".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::None,
            Callbacks {
                read: Rc::new(move || Some(read.0.open.get())),
                before: Rc::new(move |_| !before.0.veto.get()),
                apply: Rc::new(move |change| {
                    apply.0.open.set(change.open);
                }),
                key: None,
            },
        );
        let reset = state.clone();
        node.set_submenu_access(
            handle,
            Rc::new(|| None),
            Rc::new(move || reset.0.resets.set(reset.0.resets.get() + 1)),
        );
        *state.0.runtime.borrow_mut() = Some(runtime.clone());
        *state.0.node.borrow_mut() = Some(node.clone());
        node
    });
    use_drop(move || node.unregister());
    rsx! { div {} }
}

fn fixture() -> (VirtualDom, Fixture, RuntimeHandle, NodeHandle) {
    let state = Fixture(Rc::new(FixtureState::default()));
    let mut dom = VirtualDom::new_with_props(fixture_root, state.clone());
    dom.rebuild_in_place();
    let runtime = state.0.runtime.borrow().clone().unwrap();
    let node = state.0.node.borrow().clone().unwrap();
    (dom, state, runtime, node)
}

fn native_node() -> serde_json::Value {
    serde_json::json!({"id":"configured","generation":1,"connected":true,
        "discovery_ready":true,"root_capture":"actual-root"})
}
fn send(runtime: &RuntimeHandle, value: serde_json::Value) -> FormEvent {
    let event = form(value);
    runtime.input(event.clone());
    event
}
fn bind(runtime: &RuntimeHandle) {
    let node = native_node();
    send(
        runtime,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":1,
        "kind":"observe","nodes":[node]}),
    );
}

#[test]
fn submenu_routing_marker_tracks_current_inventory_and_registered_owner() {
    let (_dom, _state, runtime, node) = fixture();
    bind(&runtime);
    let host = HostFacts {
        item: ItemRef("actual-member".into()),
        kind: ItemKind::Select,
        target: "consumer-public-id".into(),
        capture: "actual-member".into(),
        root_path: Vec::new(),
        registration_key: "source-select".into(),
        source_focusable: true,
        disabled_attribute: false,
        hidden_attribute: false,
        tabindex_attribute: None,
        checked_attribute: false,
        visible_focus: false,
        roving_opt_in: false,
        linked_trigger: None,
        parts: Vec::new(),
    };
    let mut observed = native_node();
    observed["inventory"] = serde_json::json!({"items_host":"actual-items",
        "hosts":[host.clone()],"navigation_order":[host.item.clone()],"query_order":[host.item.clone()]});
    send(
        &runtime,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":2,
        "kind":"observe","nodes":[observed]}),
    );
    assert_eq!(runtime.first_submenu_marker("configured", 1), None);
    node.mark_submenu_trigger(&host);
    for _ in 0..2 {
        assert_eq!(
            runtime.first_submenu_marker("configured", 1).as_deref(),
            Some("actual-member")
        );
    }
    assert_eq!(runtime.first_submenu_marker("configured", 2), None);
    assert_eq!(runtime.first_submenu_marker("copied-root-id", 1), None);
    node.unregister();
    assert_eq!(runtime.first_submenu_marker("configured", 1), None);
    assert!(!runtime.0.borrow().submenu_marks.contains_key(&host.capture));
}

#[derive(Clone, Default)]
struct ReturnMount(Rc<RefCell<Vec<&'static str>>>);
impl dioxus::html::RenderedElementBacking for ReturnMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let calls = self.0.clone();
        Box::pin(async move {
            calls.borrow_mut().push("focus");
            Ok(())
        })
    }
    fn scroll_to(
        &self,
        options: dioxus::html::ScrollToOptions,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(matches!(
            options.vertical,
            dioxus::html::ScrollLogicalPosition::Nearest
        ));
        assert!(matches!(
            options.horizontal,
            dioxus::html::ScrollLogicalPosition::Nearest
        ));
        let calls = self.0.clone();
        Box::pin(async move {
            calls.borrow_mut().push("scroll");
            Ok(())
        })
    }
}

struct ReturnFixture {
    dom: VirtualDom,
    state: Fixture,
    runtime: RuntimeHandle,
    parent: NodeHandle,
    child_open: Rc<Cell<bool>>,
    child_veto: Rc<Cell<bool>>,
    host: HostFacts,
    mount: ReturnMount,
}
fn return_fixture(disabled: bool, registered: bool) -> ReturnFixture {
    return_fixture_kind(disabled, registered, ItemKind::Dropdown)
}
fn return_fixture_kind(disabled: bool, registered: bool, kind: ItemKind) -> ReturnFixture {
    let (dom, state, runtime, parent) = fixture();
    let child_open = Rc::new(Cell::new(false));
    let child_veto = Rc::new(Cell::new(false));
    let read = child_open.clone();
    let apply = child_open.clone();
    let before = child_veto.clone();
    let child = dom.in_scope(ScopeId::ROOT, || {
        runtime.register(
            "nested-return".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::None,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(move |_| !before.get()),
                apply: Rc::new(move |change| apply.set(change.open)),
                key: None,
            },
        )
    });
    let mut host: HostFacts = serde_json::from_value(serde_json::json!({
        "item":"return-trigger-object", "kind":"dropdown", "target":"return-trigger",
        "capture":"return-trigger-object", "registration_key":"return-trigger-key",
        "disabled_attribute":disabled,
    }))
    .unwrap();
    host.kind = kind;
    host.source_focusable = kind != ItemKind::Category;
    let facts = InventoryFacts {
        items_host: ItemRef("configured-host".into()),
        hosts: vec![host.clone()],
        navigation_order: vec![host.item.clone()],
        query_order: vec![host.item.clone()],
        slots: None,
    };
    let mount = ReturnMount::default();
    if registered {
        runtime.register_item_cue(host.registration_key.clone(), host.kind);
        runtime.register_item_mount(
            &host.registration_key,
            &host.target,
            Rc::new(MountedData::new(mount.clone())),
        );
    }
    send(
        &runtime,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":[
            {"id":"configured","generation":1,"connected":true,"discovery_ready":true,"root_capture":"actual-root","inventory":facts},
            {"id":"nested-return","generation":1,"connected":true,"discovery_ready":true,"trigger_ancestors":["configured"]}
        ]}),
    );
    assert!(parent.request(true, Reason::Trigger, false));
    assert!(child.request(true, Reason::Trigger, false));
    assert_eq!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("nested-return")
            .as_deref(),
        Some("configured")
    );
    parent.mark_submenu_trigger(&host);
    runtime.0.borrow_mut().keyboard = true;
    ReturnFixture {
        dom,
        state,
        runtime,
        parent,
        child_open,
        child_veto,
        host,
        mount,
    }
}

#[test]
fn submenu_close_returns_registered_parent_through_mounted_focus_scroll_and_roving_state() {
    let mut fixture = return_fixture(false, true);
    assert!(fixture.parent.set_navigation_mode("roving-tabindex"));
    assert!(fixture.parent.is_item_registered(&fixture.host));
    let stale = NodeHandle {
        generation: fixture.parent.generation + 1,
        ..fixture.parent.clone()
    };
    assert!(!stale.is_item_registered(&fixture.host));
    fixture.runtime.close_submenu("nested-return");
    assert!(!fixture.child_open.get());
    assert_eq!(fixture.state.0.resets.get(), 1);
    assert!(
        !fixture
            .runtime
            .0
            .borrow()
            .submenu_marks
            .contains_key(&fixture.host.capture)
    );
    assert!(
        fixture.mount.0.borrow().is_empty(),
        "focus executes after the Dioxus render"
    );
    tests::settle_item_effects(&mut fixture.dom);
    assert_eq!(*fixture.mount.0.borrow(), ["focus", "scroll"]);
    assert_eq!(
        fixture.runtime.item_visual_key().as_deref(),
        Some("return-trigger-key")
    );
    assert_eq!(fixture.runtime.item_tabindex("return-trigger-key"), Some(0));

    let mut category = return_fixture_kind(false, true, ItemKind::Category);
    assert!(
        !category.host.source_focusable,
        "the old outer cue marker is not physical focus support"
    );
    assert!(category.parent.is_item_registered(&category.host));
    assert!(category.parent.set_navigation_mode("roving-tabindex"));
    category.runtime.close_submenu("nested-return");
    tests::settle_item_effects(&mut category.dom);
    assert_eq!(*category.mount.0.borrow(), ["focus", "scroll"]);
    assert_eq!(
        category.runtime.item_tabindex("return-trigger-key"),
        Some(0)
    );
}

#[test]
fn submenu_close_skips_disabled_and_unregistered_return_targets_but_resets_forwarding() {
    for (disabled, registered) in [(true, true), (false, false)] {
        let mut fixture = return_fixture(disabled, registered);
        fixture.runtime.close_submenu("nested-return");
        tests::settle_item_effects(&mut fixture.dom);
        assert!(!fixture.child_open.get());
        assert_eq!(fixture.state.0.resets.get(), 1);
        assert!(fixture.mount.0.borrow().is_empty());
        assert_eq!(fixture.runtime.pending_item_focus_count(), 0);
    }
}

#[test]
fn submenu_close_preserves_child_veto_and_returns_to_live_parent() {
    let mut fixture = return_fixture(false, true);
    fixture.child_veto.set(true);
    fixture.runtime.close_submenu("nested-return");
    tests::settle_item_effects(&mut fixture.dom);
    assert!(fixture.child_open.get(), "veto keeps the child visible");
    assert_eq!(fixture.state.0.resets.get(), 1);
    assert_eq!(*fixture.mount.0.borrow(), ["focus", "scroll"]);
}

#[test]
fn submenu_return_rejects_closed_parent_and_cancels_when_parent_is_removed() {
    let mut closed = return_fixture(false, true);
    // A current registered relation does not authorize return focus when the
    // controlled parent is already closed.
    closed.state.0.open.set(false);
    closed.runtime.close_submenu("nested-return");
    tests::settle_item_effects(&mut closed.dom);
    assert!(closed.mount.0.borrow().is_empty());
    assert_eq!(closed.runtime.pending_item_focus_count(), 0);

    let mut removed = return_fixture(false, true);
    removed.runtime.close_submenu("nested-return");
    assert_eq!(removed.runtime.pending_item_focus_count(), 1);
    removed.parent.unregister();
    assert!(!removed.parent.is_item_registered(&removed.host));
    tests::settle_item_effects(&mut removed.dom);
    assert!(removed.mount.0.borrow().is_empty());
    assert_eq!(removed.runtime.pending_item_focus_count(), 0);
    assert!(!removed.child_open.get());
}

#[test]
fn closed_linked_trigger_arrow_right_reaches_parent_but_other_keys_and_open_child_keep_ownership() {
    let fixture = return_fixture(false, true);
    fixture.child_open.set(false);
    let delivered = Rc::new(RefCell::new(Vec::new()));
    for (id, owner) in [("configured", "parent"), ("nested-return", "child")] {
        let delivered = delivered.clone();
        fixture
            .runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut(id)
            .unwrap()
            .callbacks
            .key = Some(Rc::new(move |facts| {
            assert_eq!(facts.focused_target.as_deref(), Some("return-trigger"));
            delivered.borrow_mut().push(owner);
            KeyDecision {
                prevent: true,
                ..KeyDecision::default()
            }
        }));
    }
    let child = NodeHandle {
        runtime: Rc::downgrade(&fixture.runtime.0),
        id: "nested-return".into(),
        generation: 1,
    };
    let dispatch = |node: &NodeHandle, key: &str, on_trigger, inside| {
        fixture.dom.in_scope(ScopeId::ROOT, || {
            node.dioxus_key(KeyFacts {
                key: key.into(),
                focused_target: Some("return-trigger".into()),
                on_trigger,
                inside,
                ..KeyFacts::default()
            })
        })
    };
    // Dioxus chooses the actual registered source before dispatch: a closed
    // linked item's Right belongs to its lexical parent; the linked trigger's
    // own activation and an open child route through the child binding.
    assert!(dispatch(&fixture.parent, "ArrowRight", false, true).prevent);
    assert_eq!(*delivered.borrow(), ["parent"]);
    assert!(dispatch(&child, "Enter", true, false).prevent);
    assert_eq!(*delivered.borrow(), ["parent", "child"]);
    fixture.child_open.set(true);
    assert!(dispatch(&child, "ArrowRight", true, false).prevent);
    assert_eq!(*delivered.borrow(), ["parent", "child", "child"]);
}
