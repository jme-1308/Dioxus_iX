//! Assignment discovery through real runtime form input, not native DOM proof.
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
fn input(runtime: &RuntimeHandle, value: serde_json::Value) {
    runtime.input(Event::new(
        Rc::new(dioxus::html::FormData::new(RawForm(value.to_string()))),
        true,
    ));
}
fn report(
    runtime: &RuntimeHandle,
    sequence: u64,
    document: &str,
    nodes: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,
        "document":document,"kind":"observe","nodes":nodes})
}
fn observed(
    id: &str,
    generation: u64,
    live: &[&str],
    emissions: &[(u64, &[&str])],
) -> serde_json::Value {
    let emissions = emissions
        .iter()
        .map(|(sequence, ancestors)| serde_json::json!({"sequence":sequence,"ancestors":ancestors}))
        .collect::<Vec<_>>();
    serde_json::json!({"id":id,"generation":generation,"connected":true,
        "discovery_ready":true,"trigger_ancestors":live,"assignment_ancestors":live,
        "assignment_emissions":emissions})
}

struct Registered {
    node: NodeHandle,
    open: Rc<Cell<bool>>,
    before: Rc<Cell<usize>>,
    applied: Rc<RefCell<Vec<bool>>>,
}
fn register(runtime: &RuntimeHandle, id: &str, generation: u64) -> Registered {
    let open = Rc::new(Cell::new(false));
    let before = Rc::new(Cell::new(0));
    let applied = Rc::new(RefCell::new(Vec::new()));
    let read_open = open.clone();
    let apply_open = open.clone();
    let before_count = before.clone();
    let apply_log = applied.clone();
    let node = runtime.register(
        id.into(),
        generation,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        Callbacks {
            read: Rc::new(move || Some(read_open.get())),
            before: Rc::new(move |_| {
                before_count.set(before_count.get() + 1);
                true
            }),
            apply: Rc::new(move |change| {
                apply_open.set(change.open);
                apply_log.borrow_mut().push(change.open);
            }),
            key: None,
        },
    );
    Registered {
        node,
        open,
        before,
        applied,
    }
}

#[test]
fn assignment_input_rebind_preserves_old_cache_until_new_parent_reopens() {
    let runtime = RuntimeHandle::new();
    let a = register(&runtime, "a", 1);
    let b = register(&runtime, "b", 1);
    let _c = register(&runtime, "c", 1);
    let child = register(&runtime, "child", 1);
    let mut nodes = vec![
        observed("a", 1, &[], &[]),
        observed("b", 1, &[], &[]),
        observed("c", 1, &[], &[]),
        observed("child", 1, &["a"], &[(1, &["a"])]),
    ];
    input(&runtime, report(&runtime, 1, "", &nodes));
    assert_eq!(
        child.before.get(),
        0,
        "observation is not a visibility request"
    );
    assert!(a.node.request(true, Reason::Trigger, false));
    assert!(b.node.request(true, Reason::Trigger, false));
    assert!(child.node.request(true, Reason::Trigger, false));
    assert_eq!(runtime.0.borrow().registry.cached_children("a"), ["child"]);
    assert!(runtime.0.borrow().registry.cached_children("b").is_empty());

    nodes[3] = observed("child", 1, &["b"], &[(2, &["b"])]);
    let mut foreign = report(&runtime, 2, "", &nodes);
    foreign["owner"] = serde_json::json!("unrelated-transport-owner");
    input(&runtime, foreign);
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .assigned_children("b")
            .is_empty()
    );
    input(&runtime, report(&runtime, 2, "", &nodes));
    assert_eq!(
        runtime.0.borrow().registry.assigned_children("a"),
        ["child"]
    );
    assert_eq!(
        runtime.0.borrow().registry.assigned_children("b"),
        ["child"]
    );
    assert!(runtime.0.borrow().registry.cached_children("b").is_empty());
    assert_eq!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("child")
            .as_deref(),
        Some("a")
    );

    nodes[3] = observed("child", 1, &["c"], &[(2, &["c"])]);
    input(&runtime, report(&runtime, 3, "", &nodes));
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .assigned_children("c")
            .is_empty()
    );
    assert_eq!(
        child.before.get(),
        1,
        "replayed discovery never calls before"
    );
    assert!(b.node.request(false, Reason::Trigger, false));
    assert!(child.open.get(), "already-open B had no captured child");
    assert!(b.node.request(true, Reason::Trigger, false));
    assert_eq!(runtime.0.borrow().registry.cached_children("b"), ["child"]);
    assert!(b.node.request(false, Reason::Trigger, false));
    assert!(
        !child.open.get(),
        "B's next accepted presentation captures it"
    );
    assert!(child.node.request(true, Reason::Trigger, false));
    assert!(a.node.request(false, Reason::Trigger, false));
    assert!(
        !child.open.get(),
        "old A's captured assignment was retained"
    );
    assert_eq!(*child.applied.borrow(), [true, false, true, false]);
}

#[test]
fn assignment_input_retries_ordered_captured_paths_and_rejects_retired_generation() {
    let runtime = RuntimeHandle::new();
    let _b = register(&runtime, "b", 1);
    let _c = register(&runtime, "c", 1);
    let old = register(&runtime, "child", 1);
    let mut nodes = vec![
        observed("b", 1, &[], &[]),
        observed("c", 1, &[], &[]),
        observed("child", 1, &["b"], &[(1, &["late"]), (2, &["b"])]),
    ];
    input(&runtime, report(&runtime, 1, "", &nodes));
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .assigned_children("b")
            .is_empty()
    );
    nodes[2] = observed("child", 1, &["c"], &[(1, &["c"]), (2, &["c"])]);
    input(&runtime, report(&runtime, 2, "", &nodes));
    let late = register(&runtime, "late", 1);
    nodes.push(observed("late", 1, &[], &[]));
    nodes[2] = observed("child", 1, &["c"], &[]);
    input(&runtime, report(&runtime, 3, "", &nodes));
    assert_eq!(
        runtime.0.borrow().registry.assigned_children("late"),
        ["child"]
    );
    assert_eq!(
        runtime.0.borrow().registry.assigned_children("b"),
        ["child"]
    );
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .assigned_children("c")
            .is_empty()
    );
    assert!(late.node.request(true, Reason::Trigger, false));
    assert_eq!(
        runtime.0.borrow().registry.cached_children("late"),
        ["child"]
    );

    let current = register(&runtime, "child", 2);
    nodes[2] = observed("child", 1, &["b"], &[(99, &["b"])]);
    input(&runtime, report(&runtime, 4, "", &nodes));
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .assigned_children("b")
            .is_empty()
    );
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .cached_children("late")
            .is_empty()
    );
    assert!(!old.node.request(true, Reason::Trigger, false));
    nodes[2] = observed("child", 2, &["c"], &[(1, &["c"])]);
    input(&runtime, report(&runtime, 5, "", &nodes));
    assert_eq!(
        runtime.0.borrow().registry.assigned_children("c"),
        ["child"]
    );
    assert!(current.node.request(true, Reason::Trigger, false));
    assert_eq!(current.before.get(), 1);
    assert_eq!(old.before.get(), 0);
}

#[test]
fn assignment_input_foreign_owner_parent_readiness_releases_domain_pending_events() {
    let parents = RuntimeHandle::new();
    let children = RuntimeHandle::new();
    let document = format!("assignment-domain-{}", parents.owner_id());
    let b = register(&parents, "b", 1);
    let child = register(&children, "child", 1);
    input(
        &parents,
        report(&parents, 1, &document, &[observed("b", 1, &[], &[])]),
    );
    input(
        &children,
        report(
            &children,
            1,
            &document,
            &[observed("child", 1, &["b"], &[(1, &["late"]), (2, &["b"])])],
        ),
    );
    let domain = parents.domain().unwrap();
    assert!(b.node.request(true, Reason::Trigger, false));
    assert!(domain.cached_children("b").is_empty());

    let late = register(&parents, "late", 1);
    // A foreign owner's new readiness report releases the child's stored
    // emissions without fabricating another child observation or binding.
    input(
        &parents,
        report(
            &parents,
            2,
            &document,
            &[observed("b", 1, &[], &[]), observed("late", 1, &[], &[])],
        ),
    );
    assert!(late.node.request(true, Reason::Trigger, false));
    assert_eq!(domain.cached_children("late"), ["child"]);
    assert!(domain.cached_children("b").is_empty());
    assert!(b.node.request(false, Reason::Trigger, false));
    assert!(b.node.request(true, Reason::Trigger, false));
    assert_eq!(domain.cached_children("b"), ["child"]);
    assert!(child.node.request(true, Reason::Trigger, false));
    assert!(late.node.request(false, Reason::Trigger, false));
    assert!(
        !child.open.get(),
        "domain cascade dispatches to the child's owner"
    );
    assert_eq!(*child.applied.borrow(), [true, false]);
    assert!(
        children
            .0
            .borrow()
            .registry
            .assigned_children("b")
            .is_empty()
    );
    assert!(
        parents
            .0
            .borrow()
            .registry
            .assigned_children("late")
            .is_empty()
    );
}
