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
fn observed(id: &str, generation: u64, ancestors: &[&str]) -> serde_json::Value {
    serde_json::json!({"id":id,"generation":generation,"connected":true,
        "discovery_ready":true,"trigger_ancestors":ancestors,"assignment_ancestors":ancestors})
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
fn current_ancestry_binds_nested_lifetimes_without_requesting_visibility() {
    let runtime = RuntimeHandle::new();
    let root = register(&runtime, "root", 1);
    let child = register(&runtime, "child", 1);
    let facts = [observed("root", 1, &[]), observed("child", 1, &["root"])];
    input(&runtime, report(&runtime, 1, "", &facts));
    assert_eq!(
        runtime.0.borrow().registry.cached_children("root"),
        ["child"]
    );
    assert_eq!(
        child.before.get(),
        0,
        "read-only ancestry does not open a child"
    );
    assert!(root.node.request(true, Reason::Trigger, false));
    assert!(child.node.request(true, Reason::Trigger, false));
    input(&runtime, report(&runtime, 2, "", &facts));
    assert_eq!(
        child.before.get(),
        1,
        "an unchanged observation is not another user action"
    );
    let mut foreign = report(
        &runtime,
        3,
        "",
        &[observed("root", 1, &[]), observed("child", 1, &[])],
    );
    foreign["owner"] = "foreign-owner".into();
    input(&runtime, foreign);
    assert_eq!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("child")
            .as_deref(),
        Some("root")
    );
    assert!(root.node.request(false, Reason::Trigger, false));
    assert!(!child.open.get());
    assert_eq!(*child.applied.borrow(), [true, false]);
}

#[test]
fn current_ancestry_rejects_retired_generation_and_rebinds_the_live_registration() {
    let runtime = RuntimeHandle::new();
    let root = register(&runtime, "root", 1);
    let old = register(&runtime, "child", 1);
    input(
        &runtime,
        report(
            &runtime,
            1,
            "",
            &[observed("root", 1, &[]), observed("child", 1, &["root"])],
        ),
    );
    assert_eq!(
        runtime.0.borrow().registry.cached_children("root"),
        ["child"]
    );
    let current = register(&runtime, "child", 2);
    input(
        &runtime,
        report(
            &runtime,
            2,
            "",
            &[observed("root", 1, &[]), observed("child", 1, &["root"])],
        ),
    );
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .cached_children("root")
            .is_empty()
    );
    assert!(!old.node.request(true, Reason::Trigger, false));
    input(
        &runtime,
        report(
            &runtime,
            3,
            "",
            &[observed("root", 1, &[]), observed("child", 2, &["root"])],
        ),
    );
    assert_eq!(
        runtime.0.borrow().registry.cached_children("root"),
        ["child"]
    );
    assert!(root.node.request(true, Reason::Trigger, false));
    assert!(current.node.request(true, Reason::Trigger, false));
    assert!(root.node.request(false, Reason::Trigger, false));
    assert!(!current.open.get());
    assert_eq!(old.before.get(), 0);
    assert_eq!(*current.applied.borrow(), [true, false]);
}

#[test]
fn current_ancestry_coordinates_foreign_owner_children_in_the_same_document() {
    let parents = RuntimeHandle::new();
    let children = RuntimeHandle::new();
    let document = format!("current-ancestry-domain-{}", parents.owner_id());
    let root = register(&parents, "root", 1);
    let child = register(&children, "child", 1);
    input(
        &parents,
        report(&parents, 1, &document, &[observed("root", 1, &[])]),
    );
    input(
        &children,
        report(&children, 1, &document, &[observed("child", 1, &["root"])]),
    );
    let domain = parents.domain().unwrap();
    assert!(root.node.request(true, Reason::Trigger, false));
    assert_eq!(domain.cached_children("root"), ["child"]);
    assert!(child.node.request(true, Reason::Trigger, false));
    assert!(root.node.request(false, Reason::Trigger, false));
    assert!(
        !child.open.get(),
        "parent close dispatches to the registered child's owner"
    );
    assert_eq!(*child.applied.borrow(), [true, false]);
    assert!(
        children
            .0
            .borrow()
            .registry
            .cached_children("root")
            .is_empty(),
        "foreign ownership stays in the shared Domain"
    );
    child.node.unregister();
    assert!(root.node.request(true, Reason::Trigger, false));
    assert!(
        domain.cached_children("root").is_empty(),
        "unmounted children leave the next current snapshot"
    );
}
