//! Exact held-listener routing through the real runtime form handler.
use super::*;
use std::cell::Cell;

struct RawForm(String);
impl dioxus::html::HasFileData for RawForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for RawForm {
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
fn input(runtime: &RuntimeHandle, value: serde_json::Value) -> bool {
    let event = Event::new(
        Rc::new(dioxus::html::FormData::new(RawForm(value.to_string()))),
        true,
    );
    runtime.input(event.clone());
    !event.default_action_enabled()
}
struct Fixture {
    node: NodeHandle,
    open: Rc<Cell<bool>>,
    calls: Rc<RefCell<Vec<KeyFacts>>>,
}
fn registered(runtime: &RuntimeHandle, generation: u64) -> Fixture {
    let open = Rc::new(Cell::new(true));
    let calls = Rc::new(RefCell::new(Vec::new()));
    let read = open.clone();
    let apply = open.clone();
    let keys = calls.clone();
    let node = runtime.register(
        "nav-root".into(),
        generation,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        Callbacks {
            read: Rc::new(move || Some(read.get())),
            before: Rc::new(|_| true),
            apply: Rc::new(move |change| apply.set(change.open)),
            key: Some(Rc::new(move |facts| {
                keys.borrow_mut().push(facts.clone());
                KeyDecision {
                    prevent: true,
                    ..KeyDecision::default()
                }
            })),
        },
    );
    Fixture { node, open, calls }
}
fn node(generation: u64, current: &str) -> serde_json::Value {
    serde_json::json!({"id":"nav-root","generation":generation,"connected":true,
        "discovery_ready":true,"root_capture":format!("actual-root-{generation}"),
        "trigger_capture":current,"mode":"active-descendant","on_trigger":false,"inside":false})
}
fn report(
    runtime: &RuntimeHandle,
    sequence: u64,
    phase: &str,
    key: &str,
    node: serde_json::Value,
    prevented: bool,
) -> serde_json::Value {
    serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,
        "kind":if phase.is_empty(){"observe"}else{"key"},"phase":phase,"key":key,
        "default_prevented":prevented,"nodes":[node]})
}
fn listener(mut node: serde_json::Value, version: u64, capture: &str) -> serde_json::Value {
    node["navigation_listener"] = serde_json::json!({"version":version,"capture":capture});
    node
}
fn binding(mut node: serde_json::Value, version: u64, capture: &str) -> serde_json::Value {
    node["navigation_binding"] = serde_json::json!({"version":version,"capture":capture});
    node
}

#[test]
fn navigation_input_retains_old_actual_listener_and_cannot_retarget_from_new_facts() {
    let runtime = RuntimeHandle::new();
    let fixture = registered(&runtime, 1);
    assert!(fixture.node.configure_navigation("active-descendant", true));
    input(
        &runtime,
        report(
            &runtime,
            1,
            "",
            "",
            binding(node(1, "actual-a"), 1, "actual-a"),
            false,
        ),
    );
    // Source configureKeyboardInteraction holds A even after the experimental
    // trigger binding becomes B. Original prevention does not remove that site.
    let old = listener(node(1, "actual-b"), 1, "actual-a");
    let admitted = report(&runtime, 2, "navigation", "ArrowDown", old.clone(), true);
    assert!(input(&runtime, admitted.clone()));
    assert_eq!(fixture.calls.borrow().len(), 1);
    let call = fixture.calls.borrow()[0].clone();
    assert_eq!(call.route, KeyRoute::Navigation);
    assert_eq!(call.navigation_mode.as_deref(), Some("active-descendant"));
    assert!(call.default_prevented);
    assert!(!call.inside && !call.on_trigger);
    assert!(
        !input(&runtime, admitted),
        "transport replay does not invoke twice"
    );

    let wrong = listener(binding(node(1, "actual-b"), 1, "actual-b"), 1, "actual-b");
    assert!(!input(
        &runtime,
        report(&runtime, 3, "navigation", "ArrowDown", wrong, false)
    ));
    assert_eq!(
        fixture.calls.borrow().len(),
        1,
        "bound version cannot change objects"
    );
    assert!(input(
        &runtime,
        report(&runtime, 4, "navigation", "ArrowDown", old, false)
    ));
    assert_eq!(fixture.calls.borrow().len(), 2);
    let refreshed = registered(&runtime, 1);
    assert!(input(
        &runtime,
        report(
            &runtime,
            5,
            "navigation",
            "Home",
            listener(node(1, "actual-b"), 1, "actual-a"),
            false
        )
    ));
    assert_eq!(
        refreshed.calls.borrow().len(),
        1,
        "same-generation callbacks retain the installed binding"
    );
    assert_eq!(
        fixture.calls.borrow().len(),
        2,
        "delivery reads the current callback"
    );
}

#[test]
fn navigation_reconfiguration_revokes_only_listener_and_keeps_independent_jobs() {
    let runtime = RuntimeHandle::new();
    let fixture = registered(&runtime, 1);
    assert!(fixture.node.configure_navigation("active-descendant", true));
    input(
        &runtime,
        report(&runtime, 1, "", "", binding(node(1, "a"), 1, "a"), false),
    );
    let completed = Rc::new(Cell::new(0));
    let delivery = completed.clone();
    assert!(
        fixture
            .node
            .queue_completion(Rc::new(move || delivery.set(delivery.get() + 1)))
    );
    let frames = runtime.pending_completion_frames();
    assert!(fixture.node.clear_navigation());
    assert!(!input(
        &runtime,
        report(
            &runtime,
            2,
            "navigation",
            "ArrowDown",
            listener(node(1, "b"), 1, "a"),
            false
        )
    ));
    assert!(fixture.node.configure_navigation("roving-tabindex", true));
    // An old completion cannot bind the newly configured strategy.
    let stale = listener(binding(node(1, "b"), 1, "a"), 1, "a");
    assert!(!input(
        &runtime,
        report(&runtime, 3, "navigation", "ArrowDown", stale, false)
    ));
    let roving = listener(
        binding(node(1, "b"), 3, "actual-items-host"),
        3,
        "actual-items-host",
    );
    assert!(input(
        &runtime,
        report(&runtime, 4, "navigation", "Tab", roving, false)
    ));
    let calls = fixture.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].navigation_mode.as_deref(), Some("roving-tabindex"));
    drop(calls);
    assert_eq!(runtime.pending_completion_frames(), frames);
    assert_eq!(completed.get(), 0);
    assert!(fixture.open.get());
}

#[test]
fn navigation_input_has_no_ad_escape_tab_and_rejects_wrong_root_owner_generation() {
    let runtime = RuntimeHandle::new();
    let fixture = registered(&runtime, 1);
    assert!(fixture.node.configure_navigation("active-descendant", true));
    input(
        &runtime,
        report(&runtime, 1, "", "", binding(node(1, "a"), 1, "a"), false),
    );
    for (sequence, key) in [(2, "Escape"), (3, "Tab")] {
        assert!(!input(
            &runtime,
            report(
                &runtime,
                sequence,
                "navigation",
                key,
                listener(node(1, "b"), 1, "a"),
                false
            )
        ));
    }
    assert!(fixture.open.get());
    assert!(fixture.calls.borrow().is_empty());
    let mut wrong_root = listener(node(1, "b"), 1, "a");
    wrong_root["root_capture"] = serde_json::json!("copied-root");
    assert!(!input(
        &runtime,
        report(&runtime, 4, "navigation", "Home", wrong_root, false)
    ));
    let mut foreign = report(
        &runtime,
        5,
        "navigation",
        "Home",
        listener(node(1, "b"), 1, "a"),
        false,
    );
    foreign["owner"] = serde_json::json!("another-transport-owner");
    assert!(!input(&runtime, foreign));
    assert!(input(
        &runtime,
        report(
            &runtime,
            5,
            "navigation",
            "Home",
            listener(node(1, "b"), 1, "a"),
            false
        )
    ));
    let current = registered(&runtime, 2);
    assert!(current.node.configure_navigation("active-descendant", true));
    assert!(!input(
        &runtime,
        report(
            &runtime,
            6,
            "navigation",
            "Home",
            listener(binding(node(1, "a"), 1, "a"), 1, "a"),
            false
        )
    ));
    assert!(input(
        &runtime,
        report(
            &runtime,
            7,
            "navigation",
            "Home",
            listener(binding(node(2, "b"), 1, "b"), 1, "b"),
            false
        )
    ));
    assert_eq!(current.calls.borrow().len(), 1);
    assert_eq!(fixture.calls.borrow().len(), 1);
    assert!(!fixture.node.clear_navigation());
    current.node.unregister();
    assert!(!input(
        &runtime,
        report(
            &runtime,
            8,
            "navigation",
            "Home",
            listener(node(2, "b"), 1, "b"),
            false
        )
    ));
}

#[test]
fn trigger_key_uses_actual_installed_listener_independently_of_current_trigger() {
    let runtime = RuntimeHandle::new();
    let fixture = registered(&runtime, 1);
    let mut actual = node(1, "current-b");
    actual["trigger_listener_capture"] = serde_json::json!("installed-a");
    actual["configured_trigger_listener"] = serde_json::json!("installed-a");
    assert!(input(
        &runtime,
        report(
            &runtime,
            1,
            "trigger-key",
            "ArrowDown",
            actual.clone(),
            false
        )
    ));
    let call = fixture.calls.borrow()[0].clone();
    assert_eq!(call.route, KeyRoute::Trigger);
    assert!(call.navigation_mode.is_none());
    assert!(!call.on_trigger);
    actual["configured_trigger_listener"] = serde_json::json!("current-b");
    assert!(!input(
        &runtime,
        report(&runtime, 2, "trigger-key", "ArrowDown", actual, false)
    ));
    assert_eq!(fixture.calls.borrow().len(), 1);

    let mut legacy = node(1, "current-b");
    legacy["configured_trigger_listener"] = serde_json::json!("current-b");
    assert!(input(
        &runtime,
        report(&runtime, 3, "trigger-key", "ArrowDown", legacy, false)
    ));
    assert_eq!(
        fixture.calls.borrow().len(),
        2,
        "omitted installed field retains old producer compatibility"
    );
}

#[test]
fn legacy_key_reports_keep_current_trigger_routing_without_navigation_configuration() {
    let runtime = RuntimeHandle::new();
    let fixture = registered(&runtime, 1);
    let mut actual = node(1, "current");
    actual["on_trigger"] = serde_json::json!(true);
    let mut value = report(&runtime, 1, "bubble", "ArrowDown", actual, false);
    value["triggers"] = serde_json::json!(["nav-root"]);
    assert!(input(&runtime, value));
    assert_eq!(fixture.calls.borrow().len(), 1);
    assert_eq!(fixture.calls.borrow()[0].route, KeyRoute::Legacy);
    assert!(fixture.calls.borrow()[0].navigation_mode.is_none());
}
