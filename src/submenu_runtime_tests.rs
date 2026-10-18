//! Configured native trigger admission through actual runtime form handlers.
use super::*;
use dioxus::core::current_scope_id;
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
    calls: Cell<usize>,
    applied: Cell<usize>,
    veto: Cell<bool>,
}
#[derive(Clone)]
struct Fixture(Rc<FixtureState>);

fn fixture_root(fixture: Fixture) -> Element {
    let runtime = use_overlay_runtime();
    // This is a real hook-owned handle, not a fabricated private context.
    let handle = crate::dropdown::use_dropdown();
    let state = fixture.clone();
    let node = use_hook(move || {
        let scope = current_scope_id();
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
                    apply.0.applied.set(apply.0.applied.get() + 1);
                }),
                key: None,
            },
        );
        let clicked = state.clone();
        let click_node = node.clone();
        node.set_submenu_access(
            handle,
            Rc::new(|| None),
            Rc::new(|| {}),
            Rc::new(move || {
                assert_eq!(
                    current_scope_id(),
                    scope,
                    "configured callback belongs to its registered owner"
                );
                clicked.0.calls.set(clicked.0.calls.get() + 1);
                click_node.request(!clicked.0.open.get(), Reason::Trigger, false);
            }),
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
        "discovery_ready":true,"root_capture":"actual-root","trigger_capture":"actual-trigger",
        "trigger_on_path":true,"on_trigger":true,"configured_trigger_listener":"actual-trigger"})
}
fn click(runtime: &RuntimeHandle, sequence: u64) -> serde_json::Value {
    serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"click",
        "phase":"trigger","triggers":["configured"],"nodes":[native_node()]})
}
fn send(runtime: &RuntimeHandle, value: serde_json::Value) -> FormEvent {
    let event = form(value);
    runtime.input(event.clone());
    event
}
fn bind(runtime: &RuntimeHandle) {
    let mut node = native_node();
    node["configured_trigger_listener"] = serde_json::Value::Null;
    send(
        runtime,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":1,
        "kind":"observe","nodes":[node]}),
    );
}

#[test]
fn configured_receiver_click_toggles_once_while_path_only_and_invalid_reports_do_not() {
    let (_dom, state, runtime, _) = fixture();
    bind(&runtime);
    let mut sequence = 1;
    let mut deliver = |mut value: serde_json::Value| {
        sequence += 1;
        value["sequence"] = sequence.into();
        send(&runtime, value)
    };
    let mut path_only = click(&runtime, 0);
    path_only["nodes"][0]["configured_trigger_listener"] = serde_json::Value::Null;
    deliver(path_only);
    assert_eq!(
        state.0.calls.get(),
        0,
        "ordinary/path-only trigger reports retain their existing component onclick route"
    );
    for (field, value) in [
        ("connected", serde_json::json!(false)),
        ("generation", serde_json::json!(2)),
        ("trigger_capture", serde_json::json!("copied-trigger")),
        (
            "configured_trigger_listener",
            serde_json::json!("another-listener-object"),
        ),
    ] {
        let mut invalid = click(&runtime, 0);
        invalid["nodes"][0][field] = value;
        deliver(invalid);
        assert_eq!(
            state.0.calls.get(),
            0,
            "{field} cannot authorize the receiver"
        );
    }
    let mut prevented = click(&runtime, 0);
    prevented["default_prevented"] = true.into();
    deliver(prevented);
    assert_eq!(state.0.calls.get(), 0);
    // Every new native event starts with document capture. It clears the prior
    // prevented-event guard while preserving the report-specific protections.
    let mut capture = click(&runtime, 0);
    capture["phase"] = "capture".into();
    deliver(capture.clone());
    let mut accepted = click(&runtime, 0);
    sequence += 1;
    accepted["sequence"] = sequence.into();
    send(&runtime, accepted.clone());
    assert_eq!(state.0.calls.get(), 1);
    assert_eq!(state.0.applied.get(), 1);
    assert!(state.0.open.get());
    send(&runtime, accepted);
    assert_eq!(
        state.0.calls.get(),
        1,
        "same native report replay cannot toggle back"
    );
    assert!(state.0.open.get());

    // A foreign nearer trigger blocks this composed ancestor's callback.
    sequence += 1;
    capture["sequence"] = sequence.into();
    capture["triggers"] = serde_json::json!(["foreign", "configured"]);
    send(&runtime, capture.clone());
    sequence += 1;
    let mut blocked = click(&runtime, sequence);
    blocked["triggers"] = serde_json::json!(["foreign", "configured"]);
    send(&runtime, blocked);
    assert_eq!(state.0.calls.get(), 1);
    assert!(state.0.open.get());

    sequence += 1;
    capture["sequence"] = sequence.into();
    capture["triggers"] = serde_json::json!(["configured"]);
    send(&runtime, capture);
    state.0.veto.set(true);
    sequence += 1;
    send(&runtime, click(&runtime, sequence));
    assert_eq!(
        state.0.calls.get(),
        2,
        "an own request veto still admits the trigger callback"
    );
    assert_eq!(state.0.applied.get(), 1);
    assert!(state.0.open.get());
    state.0.veto.set(false);
    sequence += 1;
    send(&runtime, click(&runtime, sequence));
    assert_eq!(state.0.calls.get(), 3);
    assert_eq!(state.0.applied.get(), 2);
    assert!(!state.0.open.get());
}

#[test]
fn submenu_marker_query_uses_exact_root_and_object_capture_without_consuming_truth() {
    let (_dom, _state, runtime, node) = fixture();
    bind(&runtime);
    let marker = |capture: &str, active| {
        serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
        "submenu_marker":{"root":"configured","generation":1,"root_capture":"actual-root",
            "capture":capture,"active":active}})
    };
    let permitted = |value| {
        let event = form(value);
        runtime.query(event.clone());
        !event.default_action_enabled()
    };
    assert!(permitted(marker("actual-member", false)));
    assert!(!permitted(marker("actual-member", true)));
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
    node.mark_submenu_trigger(&host);
    for _ in 0..2 {
        assert!(permitted(marker("actual-member", true)));
        assert!(!permitted(marker("actual-member", false)));
    }
    assert!(!permitted(marker("same-public-id-new-object", true)));
    assert!(permitted(marker("same-public-id-new-object", false)));
    for (field, value) in [
        ("root", serde_json::json!("copied-root-id")),
        ("generation", serde_json::json!(2)),
        ("root_capture", serde_json::json!("copied-root")),
        ("capture", serde_json::json!("")),
    ] {
        let mut invalid = marker("actual-member", true);
        invalid["submenu_marker"][field] = value;
        assert!(
            !permitted(invalid),
            "{field} cannot authorize a marker write"
        );
    }
    node.unregister();
    assert!(!permitted(marker("actual-member", true)));
    assert!(!permitted(marker("actual-member", false)));
}
