//! Actual independent VirtualDom owner-dispatch regressions. Reports below are
//! Rust inputs to the real runtime handler, not native bridge/browser evidence.

use super::*;
use std::cell::Cell;

#[derive(Clone, Copy)]
struct OwnerToken(usize);

struct FixtureState {
    id: &'static str,
    token: usize,
    veto: Rc<Cell<bool>>,
    runtime: Option<RuntimeHandle>,
    node: Option<NodeHandle>,
    scope: Option<ScopeId>,
    owner: Option<Weak<dioxus::core::Runtime>>,
    open: Option<Signal<bool>>,
    calls: Vec<(String, bool)>,
    captures: Vec<Option<Vec<usize>>>,
}

#[derive(Clone)]
struct Fixture(Rc<RefCell<FixtureState>>);

impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Fixture {
    fn new(id: &'static str, token: usize) -> Self {
        Self(Rc::new(RefCell::new(FixtureState {
            id,
            token,
            veto: Rc::new(Cell::new(false)),
            runtime: None,
            node: None,
            scope: None,
            owner: None,
            open: None,
            calls: Vec::new(),
            captures: Vec::new(),
        })))
    }

    fn runtime(&self) -> RuntimeHandle {
        self.0.borrow().runtime.clone().unwrap()
    }

    fn node(&self) -> NodeHandle {
        self.0.borrow().node.clone().unwrap()
    }

    fn scope(&self) -> ScopeId {
        self.0.borrow().scope.unwrap()
    }

    fn open(&self) -> bool {
        *self.0.borrow().open.unwrap().read()
    }
}

fn assert_owner(owner: &Weak<dioxus::core::Runtime>, scope: ScopeId, token: usize) {
    let expected = owner.upgrade().expect("registered owner is live");
    assert!(Rc::ptr_eq(&expected, &dioxus::core::Runtime::current()));
    assert_eq!(dioxus::core::current_scope_id(), scope);
    assert_eq!(consume_context::<OwnerToken>().0, token);
}

fn owner_root(fixture: Fixture) -> Element {
    let token = fixture.0.borrow().token;
    use_context_provider(move || OwnerToken(token));
    let open = use_signal(|| false);
    let runtime = use_hook(RuntimeHandle::new);
    let registered_runtime = runtime.clone();
    let registered_fixture = fixture.clone();
    let node = use_hook(move || {
        let owner = Rc::downgrade(&dioxus::core::Runtime::current());
        let scope = dioxus::core::current_scope_id();
        let (id, veto) = {
            let state = registered_fixture.0.borrow();
            (state.id, state.veto.clone())
        };
        let read_owner = owner.clone();
        let before_owner = owner.clone();
        let apply_owner = owner.clone();
        let before_fixture = Rc::downgrade(&registered_fixture.0);
        let apply_fixture = before_fixture.clone();
        let node = registered_runtime.register(
            id.to_owned(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || {
                    assert_owner(&read_owner, scope, token);
                    open.try_read().ok().map(|open| *open)
                }),
                before: Rc::new(move |change| {
                    assert_owner(&before_owner, scope, token);
                    let capture = WINDOW_CAPTURE.with(|slot| slot.borrow().clone());
                    let fixture = before_fixture
                        .upgrade()
                        .expect("registered fixture is live");
                    let mut state = fixture.borrow_mut();
                    state.calls.push(("before".into(), change.open));
                    state.captures.push(capture);
                    !veto.get()
                }),
                apply: Rc::new(move |change| {
                    assert_owner(&apply_owner, scope, token);
                    apply_fixture
                        .upgrade()
                        .expect("registered fixture is live")
                        .borrow_mut()
                        .calls
                        .push(("apply".into(), change.open));
                    let mut open = open;
                    open.set(change.open);
                }),
                key: None,
            },
        );
        {
            let mut state = registered_fixture.0.borrow_mut();
            state.runtime = Some(registered_runtime.clone());
            state.node = Some(node.clone());
            state.scope = Some(scope);
            state.owner = Some(owner);
            state.open = Some(open);
        }
        node
    });
    use_drop(move || node.unregister());
    rsx! { div { "{open}" } }
}

fn build(fixture: &Fixture) -> VirtualDom {
    let mut dom = VirtualDom::new_with_props(owner_root, fixture.clone());
    dom.rebuild_in_place();
    dom
}

fn attach(dom: &VirtualDom, fixture: &Fixture, document: &str) {
    dom.in_scope(fixture.scope(), || {
        fixture.runtime().attach_document(document)
    });
}

fn observe(
    dom: &VirtualDom,
    fixture: &Fixture,
    document: &str,
    sequence: u64,
    window_sequence: u64,
    prevented: bool,
    ids: &[&str],
) {
    let report: Report = serde_json::from_value(serde_json::json!({
        "owner": fixture.runtime().owner_id(),
        "document": document,
        "sequence": sequence,
        "window_sequence": window_sequence,
        "kind": if window_sequence == 0 { "observe" } else { "key" },
        "phase": if window_sequence == 0 { "" } else { "window" },
        "key": "Escape",
        "default_prevented": prevented,
        "active_categories": [73],
        "hosts": [],
        "triggers": [],
        "nodes": ids.iter().map(|id| serde_json::json!({
            "id": id, "generation": 1, "connected": true, "discovery_ready": true
        })).collect::<Vec<_>>()
    }))
    .unwrap();
    dom.in_scope(fixture.scope(), || {
        let before = dioxus::core::Runtime::current();
        fixture.runtime().observe(&report);
        assert!(Rc::ptr_eq(&before, &dioxus::core::Runtime::current()));
        assert_eq!(consume_context::<OwnerToken>().0, fixture.0.borrow().token);
    });
}

fn present(dom: &VirtualDom, fixture: &Fixture) {
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(true, Reason::Trigger, false));
    });
    fixture.0.borrow_mut().calls.clear();
    fixture.0.borrow_mut().captures.clear();
}

#[test]
fn independent_vdom_callbacks_restore_owner_runtime_scope_and_context() {
    let a = Fixture::new("owner-runtime-a", 101);
    let b = Fixture::new("owner-runtime-b", 202);
    let dom_a = build(&a);
    let dom_b = build(&b);
    let owner_a = a.0.borrow().owner.clone().unwrap().upgrade().unwrap();
    let owner_b = b.0.borrow().owner.clone().unwrap().upgrade().unwrap();
    assert!(
        !Rc::ptr_eq(&owner_a, &owner_b),
        "separate actual VirtualDoms are required"
    );
    attach(&dom_a, &a, "owner-runtime-native-document");
    attach(&dom_b, &b, "owner-runtime-native-document");
    observe(
        &dom_a,
        &a,
        "owner-runtime-native-document",
        1,
        0,
        false,
        &["owner-runtime-a"],
    );
    observe(
        &dom_b,
        &b,
        "owner-runtime-native-document",
        1,
        0,
        false,
        &["owner-runtime-b"],
    );
    present(&dom_a, &a);
    present(&dom_b, &b);
    let domain = a.runtime().domain().unwrap();
    dom_a.in_scope(a.scope(), || {
        assert!(domain.request(
            "owner-runtime-b",
            Change {
                open: false,
                reason: Reason::Outside,
                restore_focus: false,
            }
        ));
        assert!(Rc::ptr_eq(&owner_a, &dioxus::core::Runtime::current()));
        assert_eq!(consume_context::<OwnerToken>().0, 101);
    });
    assert!(a.open());
    assert!(!b.open());
    assert!(a.0.borrow().calls.is_empty());
    assert_eq!(
        b.0.borrow().calls,
        [("before".into(), false), ("apply".into(), false)]
    );
}

#[test]
fn window_escape_deduplicates_across_real_roots_ignores_prevention_and_retains_veto() {
    let a = Fixture::new("window-root-a", 301);
    let b = Fixture::new("window-root-b", 302);
    let c = Fixture::new("window-other-document", 303);
    let dom_a = build(&a);
    let dom_b = build(&b);
    let dom_c = build(&c);
    let document = "window-escape-shared-native-document";
    let separate = "window-escape-separate-native-document";
    attach(&dom_a, &a, document);
    attach(&dom_b, &b, document);
    attach(&dom_c, &c, separate);
    observe(&dom_a, &a, document, 1, 0, false, &["window-root-a"]);
    observe(&dom_b, &b, document, 1, 0, false, &["window-root-b"]);
    observe(
        &dom_c,
        &c,
        separate,
        1,
        0,
        false,
        &["window-other-document"],
    );
    present(&dom_a, &a);
    present(&dom_b, &b);
    present(&dom_c, &c);
    a.0.borrow().veto.set(true);
    // Both original Window listeners receive this event, but one document
    // claim closes only the selected B, irrespective of the outside target.
    observe(
        &dom_a,
        &a,
        document,
        2,
        1,
        false,
        &["window-root-a", "window-root-b"],
    );
    assert!(a.open() && !b.open() && c.open());
    assert!(a.0.borrow().calls.is_empty());
    assert_eq!(
        b.0.borrow().calls,
        [("before".into(), false), ("apply".into(), false)]
    );
    assert_eq!(b.0.borrow().captures, [Some(vec![73])]);
    observe(
        &dom_b,
        &b,
        document,
        2,
        1,
        false,
        &["window-root-a", "window-root-b"],
    );
    assert!(
        a.0.borrow().calls.is_empty(),
        "second owner cannot dismiss the next overlay for one Window event"
    );
    // A real default-prevented Window delivery still attempts selected A.
    observe(
        &dom_b,
        &b,
        document,
        3,
        2,
        true,
        &["window-root-a", "window-root-b"],
    );
    assert!(a.open() && !b.open() && c.open());
    assert_eq!(a.0.borrow().calls, [("before".into(), false)]);
    assert_eq!(a.0.borrow().captures, [Some(vec![73])]);
    assert!(c.0.borrow().calls.is_empty());
    assert!(
        WINDOW_CAPTURE.with(|slot| slot.borrow().is_none()),
        "synchronous capture is released after routing"
    );

    let domain = a.runtime().domain().unwrap();
    let weak_b = Rc::downgrade(&b.runtime().0);
    let owner_b = b.0.borrow().owner.clone().unwrap();
    b.0.borrow_mut().runtime.take();
    drop(dom_b);
    assert!(
        weak_b.upgrade().is_none(),
        "foreign domain callbacks must not retain the dropped root runtime"
    );
    assert!(
        owner_b.upgrade().is_none(),
        "callback dispatch must not retain a dropped Dioxus runtime"
    );
    assert_eq!(domain.topmost().as_deref(), Some("window-root-a"));
    assert!(b.node().runtime().is_none());
    assert!(a.open() && c.open());
}

// Document-trap reports exercise the actual form-input/query handlers below.
// They are Rust transport evidence, not an original native event/focus replay.
struct TrapForm(String);
impl dioxus::html::HasFileData for TrapForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for TrapForm {
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
fn trap_form(value: serde_json::Value) -> FormEvent {
    Event::new(
        Rc::new(dioxus::html::FormData::new(TrapForm(value.to_string()))),
        true,
    )
}

fn build_popover_fixture(fixture: &Fixture) -> VirtualDom {
    let dom = build(fixture);
    dom.in_scope(fixture.scope(), || {
        let runtime = fixture.runtime();
        let id = fixture.0.borrow().id;
        let callbacks = runtime.entry(id).unwrap().callbacks;
        let node = runtime.register(
            id.into(),
            1,
            OverlayKind::Popover,
            CloseBehavior::Both,
            callbacks,
        );
        fixture.0.borrow_mut().node = Some(node);
    });
    dom
}

fn trap_node(
    id: &str,
    ancestors: &[&str],
    candidates: &[(&str, &[&str], bool)],
) -> serde_json::Value {
    serde_json::json!({
        "id":id,"generation":1,"connected":true,"discovery_ready":true,
        "trigger_ancestors":ancestors,"trap_enabled":true,
        "tab_candidates":candidates.iter().map(|(target,hosts,active)| serde_json::json!({
            "target":target,"hosts":hosts,"active":active,"tab_index":0
        })).collect::<Vec<_>>()
    })
}

fn trap_report(
    fixture: &Fixture,
    document: &str,
    sequence: u64,
    nodes: Vec<serde_json::Value>,
) -> serde_json::Value {
    let targets = nodes
        .iter()
        .flat_map(|node| {
            node["tab_candidates"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item["target"].clone())
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "owner":fixture.runtime().owner_id(),"document":document,"sequence":sequence,
        "kind":"key","phase":"trap","key":"Tab","shift":true,
        "active_hosts":[fixture.0.borrow().id],"active_popover":fixture.0.borrow().id,
        "nodes":nodes,"focus_targets":targets
    })
}
fn deliver_trap(dom: &VirtualDom, fixture: &Fixture, report: serde_json::Value) -> FormEvent {
    let event = trap_form(report);
    dom.in_scope(fixture.scope(), || fixture.runtime().input(event.clone()));
    event
}
fn trap_query(dom: &VirtualDom, fixture: &Fixture, sequence: u64, target: &str) -> FormEvent {
    let event = trap_form(
        serde_json::json!({"owner":fixture.runtime().owner_id(),"sequence":sequence,"focus_target":target}),
    );
    dom.in_scope(fixture.scope(), || fixture.runtime().query(event.clone()));
    event
}

#[test]
fn document_trap_handler_preserves_interior_empty_prevented_and_consumes_open_focus_once() {
    let fixture = Fixture::new("document-trap-policy", 501);
    let dom = build_popover_fixture(&fixture);
    let document = "document trap handler policy";
    attach(&dom, &fixture, document);
    observe(
        &dom,
        &fixture,
        document,
        1,
        0,
        false,
        &["document-trap-policy"],
    );
    present(&dom, &fixture);
    let node = || {
        trap_node(
            "document-trap-policy",
            &[],
            &[
                ("header", &["document-trap-policy"], true),
                ("middle", &["document-trap-policy"], false),
                ("last", &["document-trap-policy"], false),
            ],
        )
    };
    let report = trap_report(&fixture, document, 2, vec![node()]);
    let event = deliver_trap(&dom, &fixture, report);
    assert!(
        !event.default_action_enabled(),
        "reverse boundary prevents native default"
    );
    assert!(fixture.open(), "trapping does not dismiss the Popover");
    let immediate = trap_form(
        serde_json::json!({"owner":fixture.runtime().owner_id(),"sequence":2,"immediate_stop":true}),
    );
    dom.in_scope(fixture.scope(), || {
        fixture.runtime().query(immediate.clone())
    });
    assert!(
        !immediate.default_action_enabled(),
        "native immediate-stop has its own ACK"
    );
    assert!(
        trap_query(&dom, &fixture, 1, "last").default_action_enabled(),
        "stale sequence cannot consume authorization"
    );
    assert!(
        trap_query(&dom, &fixture, 2, "middle").default_action_enabled(),
        "wrong target cannot consume authorization"
    );
    assert!(!trap_query(&dom, &fixture, 2, "last").default_action_enabled());
    assert!(
        trap_query(&dom, &fixture, 2, "last").default_action_enabled(),
        "focus authorization is consumed once"
    );

    let mut interior = trap_report(&fixture, document, 3, vec![node()]);
    interior["nodes"][0]["tab_candidates"][0]["active"] = false.into();
    interior["nodes"][0]["tab_candidates"][1]["active"] = true.into();
    assert!(deliver_trap(&dom, &fixture, interior).default_action_enabled());
    let empty = trap_report(
        &fixture,
        document,
        4,
        vec![trap_node("document-trap-policy", &[], &[])],
    );
    assert!(deliver_trap(&dom, &fixture, empty).default_action_enabled());
    let mut prevented = trap_report(&fixture, document, 5, vec![node()]);
    prevented["default_prevented"] = true.into();
    assert!(deliver_trap(&dom, &fixture, prevented).default_action_enabled());
    assert!(trap_query(&dom, &fixture, 5, "last").default_action_enabled());
    let mut disabled = trap_report(&fixture, document, 6, vec![node()]);
    disabled["nodes"][0]["trap_enabled"] = false.into();
    assert!(deliver_trap(&dom, &fixture, disabled).default_action_enabled());
    let mut other_phase = trap_report(&fixture, document, 7, vec![node()]);
    other_phase["phase"] = "fallback".into();
    assert!(
        deliver_trap(&dom, &fixture, other_phase).default_action_enabled(),
        "boundary policy belongs to document trap phase"
    );
    assert!(
        fixture.0.borrow().calls.is_empty(),
        "Tab trap never runs visibility callbacks"
    );
}

#[test]
fn document_trap_handler_rejects_closed_reopened_and_replaced_focus_authorizations() {
    let fixture = Fixture::new("document-trap-stale", 502);
    let dom = build_popover_fixture(&fixture);
    let document = "document trap stale authorization";
    attach(&dom, &fixture, document);
    observe(
        &dom,
        &fixture,
        document,
        1,
        0,
        false,
        &["document-trap-stale"],
    );
    present(&dom, &fixture);
    let report = |sequence| {
        trap_report(
            &fixture,
            document,
            sequence,
            vec![trap_node(
                "document-trap-stale",
                &[],
                &[
                    ("first", &["document-trap-stale"], true),
                    ("last", &["document-trap-stale"], false),
                ],
            )],
        )
    };
    assert!(!deliver_trap(&dom, &fixture, report(2)).default_action_enabled());
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(false, Reason::Trigger, false));
    });
    assert!(
        trap_query(&dom, &fixture, 2, "last").default_action_enabled(),
        "closed host cannot execute open-trap authorization"
    );
    present(&dom, &fixture);
    assert!(!deliver_trap(&dom, &fixture, report(3)).default_action_enabled());
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(false, Reason::Trigger, false));
        assert!(fixture.node().request(true, Reason::Trigger, false));
    });
    assert!(
        trap_query(&dom, &fixture, 3, "last").default_action_enabled(),
        "same-generation reopen changes revision"
    );
    assert!(!deliver_trap(&dom, &fixture, report(4)).default_action_enabled());
    dom.in_scope(fixture.scope(), || {
        let runtime = fixture.runtime();
        let callbacks = runtime.entry("document-trap-stale").unwrap().callbacks;
        let replacement = runtime.register(
            "document-trap-stale".into(),
            2,
            OverlayKind::Popover,
            CloseBehavior::Both,
            callbacks,
        );
        fixture.0.borrow_mut().node = Some(replacement);
    });
    assert!(
        trap_query(&dom, &fixture, 4, "last").default_action_enabled(),
        "new registration cannot inherit old focus admission"
    );
    assert!(
        deliver_trap(&dom, &fixture, report(5)).default_action_enabled(),
        "old native generation cannot authorize replacement"
    );
}

#[test]
fn document_trap_cross_root_deferral_exclusions_and_dropped_topmost() {
    let parent = Fixture::new("trap-parent", 503);
    let child = Fixture::new("trap-child", 504);
    let dom_parent = build_popover_fixture(&parent);
    let dom_child = build_popover_fixture(&child);
    let document = "cross-root document traps";
    attach(&dom_parent, &parent, document);
    attach(&dom_child, &child, document);
    observe(
        &dom_parent,
        &parent,
        document,
        1,
        0,
        false,
        &["trap-parent"],
    );
    observe(&dom_child, &child, document, 1, 0, false, &["trap-child"]);
    present(&dom_parent, &parent);
    present(&dom_child, &child);
    let parent_node = || {
        trap_node(
            "trap-parent",
            &[],
            &[
                ("parent-first", &["trap-parent"], true),
                ("nested-dropdown", &["trap-parent", "trap-child"], false),
                ("parent-last", &["trap-parent"], false),
            ],
        )
    };
    let child_node = || {
        trap_node(
            "trap-child",
            &["trap-parent"],
            &[
                ("child-first", &["trap-parent", "trap-child"], true),
                ("child-last", &["trap-parent", "trap-child"], false),
            ],
        )
    };
    let mut focused_child = trap_report(&parent, document, 2, vec![parent_node(), child_node()]);
    focused_child["active_hosts"] = serde_json::json!(["trap-child", "trap-parent"]);
    focused_child["active_popover"] = "trap-child".into();
    assert!(
        deliver_trap(&dom_parent, &parent, focused_child.clone()).default_action_enabled(),
        "foreign focused Popover owns its trap"
    );
    focused_child["owner"] = child.runtime().owner_id().into();
    assert!(!deliver_trap(&dom_child, &child, focused_child).default_action_enabled());
    assert!(!trap_query(&dom_child, &child, 2, "child-last").default_action_enabled());

    // Replace the foreign participant with a Dropdown using actual registration.
    dom_child.in_scope(child.scope(), || {
        let runtime = child.runtime();
        let callbacks = runtime.entry("trap-child").unwrap().callbacks;
        let node = runtime.register(
            "trap-child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        child.0.borrow_mut().node = Some(node);
    });
    let mut focused_dropdown = trap_report(&parent, document, 3, vec![parent_node(), child_node()]);
    focused_dropdown["active_hosts"] = serde_json::json!(["trap-child", "trap-parent"]);
    assert!(
        deliver_trap(&dom_parent, &parent, focused_dropdown).default_action_enabled(),
        "focused topmost Dropdown defers parent trap"
    );
    let mut trigger_focus = trap_report(&parent, document, 4, vec![parent_node(), child_node()]);
    trigger_focus["nodes"][0]["tab_candidates"] = serde_json::json!([
        {"target":"parent-first","tab_index":0,"hosts":["trap-parent"],"active":false},
        {"target":"parent-last","tab_index":0,"hosts":["trap-parent"],"active":true},
        {"target":"nested-dropdown","tab_index":0,"hosts":["trap-parent","trap-child"],"active":false}
    ]);
    trigger_focus["shift"] = false.into();
    assert!(
        !deliver_trap(&dom_parent, &parent, trigger_focus).default_action_enabled(),
        "nonfocused Dropdown branch is excluded before boundary calculation"
    );
    assert!(!trap_query(&dom_parent, &parent, 4, "parent-first").default_action_enabled());

    drop(dom_child);
    let after_drop = trap_report(&parent, document, 5, vec![parent_node()]);
    assert!(
        !deliver_trap(&dom_parent, &parent, after_drop).default_action_enabled(),
        "dropped foreign owner no longer defers live parent"
    );
    assert!(!trap_query(&dom_parent, &parent, 5, "parent-last").default_action_enabled());
    assert!(parent.open());
}

fn build_deferred_exit_fixture(fixture: &Fixture, nested_observe: Rc<Cell<bool>>) -> VirtualDom {
    let dom = build(fixture);
    dom.in_scope(fixture.scope(), || {
        let runtime = fixture.runtime();
        let id = fixture.0.borrow().id;
        let mut callbacks = runtime.entry(id).unwrap().callbacks;
        let close = fixture.node();
        callbacks.key = Some(Rc::new(move |facts| {
            if facts.key != "Tab" || facts.default_prevented {
                return KeyDecision::default();
            }
            let runtime = close.runtime().unwrap();
            runtime.dismiss_root(close.id(), Reason::Keyboard, false);
            if nested_observe.replace(false) {
                // Real nested input replaces global ACK state while the outer
                // key callback has not yet returned/admitted its focus job.
                let sequence = runtime.0.borrow().sequence + 1;
                let document = runtime.0.borrow().document.clone();
                let generation = runtime.entry(close.id()).unwrap().generation;
                let report = serde_json::json!({
                    "owner":runtime.owner_id(),"document":document,"sequence":sequence,
                    "kind":"observe","nodes":[{"id":close.id(),"generation":generation,
                        "connected":true,"discovery_ready":true}]
                });
                runtime.input(trap_form(report));
            }
            KeyDecision {
                release_focus: true,
                ..KeyDecision::default()
            }
        }));
        let node = runtime.register(
            id.into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        fixture.0.borrow_mut().node = Some(node);
    });
    dom
}

fn deferred_exit_report(fixture: &Fixture, document: &str, sequence: u64) -> serde_json::Value {
    let id = fixture.0.borrow().id;
    serde_json::json!({
        "owner":fixture.runtime().owner_id(),"document":document,"sequence":sequence,
        "kind":"key","phase":"capture","key":"Tab","hosts":[id],"triggers":[id],
        "tab_order":["before","trigger","after"],
        "nodes":[{"id":id,"generation":fixture.node().generation(),"connected":true,
            "discovery_ready":true,"mode":"roving-tabindex","trigger_id":"trigger",
            "inside":true,"on_trigger":true}]
    })
}

fn deferred_query(
    dom: &VirtualDom,
    fixture: &Fixture,
    sequence: u64,
    field: &str,
    target: &str,
    owner: Option<&str>,
) -> FormEvent {
    let mut value = serde_json::json!({
        "owner":owner.map(str::to_string).unwrap_or_else(||fixture.runtime().owner_id()),
        "sequence":sequence
    });
    value[field] = target.into();
    let event = trap_form(value);
    dom.in_scope(fixture.scope(), || fixture.runtime().query(event.clone()));
    event
}

fn deferred_observe(dom: &VirtualDom, fixture: &Fixture, document: &str, sequence: u64) {
    let value = serde_json::json!({
        "owner":fixture.runtime().owner_id(),"document":document,"sequence":sequence,
        "kind":"observe","nodes":[{"id":fixture.0.borrow().id,
            "generation":fixture.node().generation(),"connected":true,"discovery_ready":true}]
    });
    assert!(deliver_trap(dom, fixture, value).default_action_enabled());
}

#[test]
fn deferred_exit_schedule_and_execution_survive_observe_ack_and_consume_once() {
    let fixture = Fixture::new("deferred-exit-ack", 601);
    let dom = build_deferred_exit_fixture(&fixture, Rc::new(Cell::new(false)));
    let document = "deferred exit ACK document";
    attach(&dom, &fixture, document);
    deferred_observe(&dom, &fixture, document, 1);
    present(&dom, &fixture);
    assert!(
        !deliver_trap(&dom, &fixture, deferred_exit_report(&fixture, document, 2))
            .default_action_enabled()
    );
    assert!(!fixture.open());
    assert!(
        deferred_query(&dom, &fixture, 2, "pending_focus_target", "after", None)
            .default_action_enabled(),
        "unarmed job cannot execute"
    );
    assert!(
        deferred_query(
            &dom,
            &fixture,
            2,
            "schedule_focus_target",
            "after",
            Some("foreign owner")
        )
        .default_action_enabled()
    );
    assert!(
        deferred_query(&dom, &fixture, 99, "schedule_focus_target", "after", None)
            .default_action_enabled()
    );
    assert!(
        deferred_query(&dom, &fixture, 2, "schedule_focus_target", "wrong", None)
            .default_action_enabled()
    );
    deferred_observe(&dom, &fixture, document, 3);
    assert!(
        trap_query(&dom, &fixture, 2, "after").default_action_enabled(),
        "deferred job is separate from immediate focus ACK"
    );
    assert!(
        !deferred_query(&dom, &fixture, 2, "schedule_focus_target", "after", None)
            .default_action_enabled(),
        "later observe ACK does not erase scheduling admission"
    );
    assert!(
        deferred_query(&dom, &fixture, 2, "schedule_focus_target", "after", None)
            .default_action_enabled(),
        "schedule only once"
    );
    assert!(
        deferred_query(&dom, &fixture, 2, "pending_focus_target", "wrong", None)
            .default_action_enabled()
    );
    assert!(
        deferred_query(
            &dom,
            &fixture,
            2,
            "pending_focus_target",
            "after",
            Some("foreign owner")
        )
        .default_action_enabled()
    );
    assert!(
        !deferred_query(&dom, &fixture, 2, "pending_focus_target", "after", None)
            .default_action_enabled()
    );
    assert!(
        deferred_query(&dom, &fixture, 2, "pending_focus_target", "after", None)
            .default_action_enabled(),
        "execution consumes once"
    );
}

#[test]
fn deferred_veto_exit_survives_same_owner_close_reopen_and_later_observation() {
    let fixture = Fixture::new("deferred-exit-veto", 602);
    let dom = build_deferred_exit_fixture(&fixture, Rc::new(Cell::new(false)));
    let document = "deferred veto owner document";
    attach(&dom, &fixture, document);
    deferred_observe(&dom, &fixture, document, 1);
    present(&dom, &fixture);
    fixture.0.borrow().veto.set(true);
    assert!(
        !deliver_trap(&dom, &fixture, deferred_exit_report(&fixture, document, 2))
            .default_action_enabled()
    );
    assert!(
        fixture.open(),
        "veto retains root but still admits exit focus"
    );
    assert!(
        !deferred_query(&dom, &fixture, 2, "schedule_focus_target", "after", None)
            .default_action_enabled()
    );
    fixture.0.borrow().veto.set(false);
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(false, Reason::Trigger, false));
        assert!(fixture.node().request(true, Reason::Trigger, false));
    });
    deferred_observe(&dom, &fixture, document, 3);
    assert!(
        !deferred_query(&dom, &fixture, 2, "pending_focus_target", "after", None)
            .default_action_enabled(),
        "later same-registration visibility/revision must not revoke source job"
    );
    assert!(fixture.open());
}

#[test]
fn deferred_exit_rejects_replaced_and_unregistered_owner() {
    let fixture = Fixture::new("deferred-exit-replaced", 603);
    let dom = build_deferred_exit_fixture(&fixture, Rc::new(Cell::new(false)));
    let document = "deferred replaced owner document";
    attach(&dom, &fixture, document);
    deferred_observe(&dom, &fixture, document, 1);
    present(&dom, &fixture);
    assert!(
        !deliver_trap(&dom, &fixture, deferred_exit_report(&fixture, document, 2))
            .default_action_enabled()
    );
    assert!(
        !deferred_query(&dom, &fixture, 2, "schedule_focus_target", "after", None)
            .default_action_enabled()
    );
    dom.in_scope(fixture.scope(), || {
        let runtime = fixture.runtime();
        let callbacks = runtime.entry("deferred-exit-replaced").unwrap().callbacks;
        let replacement = runtime.register(
            "deferred-exit-replaced".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        fixture.0.borrow_mut().node = Some(replacement);
    });
    assert!(
        deferred_query(&dom, &fixture, 2, "pending_focus_target", "after", None)
            .default_action_enabled(),
        "new registration cannot execute old queued target"
    );
    present(&dom, &fixture);
    assert!(
        !deliver_trap(&dom, &fixture, deferred_exit_report(&fixture, document, 3))
            .default_action_enabled()
    );
    assert!(
        !deferred_query(&dom, &fixture, 3, "schedule_focus_target", "after", None)
            .default_action_enabled()
    );
    dom.in_scope(fixture.scope(), || fixture.node().unregister());
    assert!(
        deferred_query(&dom, &fixture, 3, "pending_focus_target", "after", None)
            .default_action_enabled()
    );
    assert!(
        deferred_query(&dom, &fixture, 3, "schedule_focus_target", "after", None)
            .default_action_enabled()
    );
}

#[test]
fn deferred_reentrant_observe_and_second_tab_preserve_both_source_jobs() {
    let fixture = Fixture::new("deferred-exit-reentrant", 604);
    let dom = build_deferred_exit_fixture(&fixture, Rc::new(Cell::new(true)));
    let document = "deferred reentrant report document";
    attach(&dom, &fixture, document);
    deferred_observe(&dom, &fixture, document, 1);
    present(&dom, &fixture);
    assert!(
        !deliver_trap(&dom, &fixture, deferred_exit_report(&fixture, document, 2))
            .default_action_enabled()
    );
    assert_eq!(
        fixture.runtime().0.borrow().sequence,
        3,
        "nested actual input advanced the latest ACK before outer callback returned"
    );
    present(&dom, &fixture);
    assert!(
        !deliver_trap(&dom, &fixture, deferred_exit_report(&fixture, document, 4))
            .default_action_enabled()
    );
    deferred_observe(&dom, &fixture, document, 5);
    assert!(
        !deferred_query(&dom, &fixture, 2, "schedule_focus_target", "after", None)
            .default_action_enabled()
    );
    assert!(
        !deferred_query(&dom, &fixture, 4, "schedule_focus_target", "after", None)
            .default_action_enabled()
    );
    assert!(
        !deferred_query(&dom, &fixture, 4, "pending_focus_target", "after", None)
            .default_action_enabled()
    );
    assert!(
        !deferred_query(&dom, &fixture, 2, "pending_focus_target", "after", None)
            .default_action_enabled(),
        "later job does not erase resumed outer job"
    );
    assert!(
        deferred_query(&dom, &fixture, 2, "pending_focus_target", "after", None)
            .default_action_enabled()
    );
}

#[test]
fn deferred_exit_projects_current_destination_popover_visibility_without_canceling_root_jobs() {
    let parent = Fixture::new("deferred-destination-popover", 605);
    let child = Fixture::new("deferred-destination-dropdown", 606);
    let dom_parent = build_popover_fixture(&parent);
    let dom_child = build_deferred_exit_fixture(&child, Rc::new(Cell::new(false)));
    let document = "deferred destination projection document";
    attach(&dom_parent, &parent, document);
    attach(&dom_child, &child, document);
    observe(
        &dom_parent,
        &parent,
        document,
        1,
        0,
        false,
        &[parent.0.borrow().id],
    );
    deferred_observe(&dom_child, &child, document, 1);
    present(&dom_parent, &parent);
    present(&dom_child, &child);

    let report = |sequence| {
        let mut value = deferred_exit_report(&child, document, sequence);
        value["nodes"]
            .as_array_mut()
            .unwrap()
            .push(trap_node(parent.0.borrow().id, &[], &[]));
        value
    };
    let execute = |sequence| {
        let event = trap_form(serde_json::json!({
            "owner":child.runtime().owner_id(),"sequence":sequence,
            "pending_focus_target":"after","focus_hosts":[parent.0.borrow().id]
        }));
        dom_child.in_scope(child.scope(), || child.runtime().query(event.clone()));
        event
    };

    assert!(!deliver_trap(&dom_child, &child, report(2)).default_action_enabled());
    assert!(!child.open());
    assert!(
        !deferred_query(
            &dom_child,
            &child,
            2,
            "schedule_focus_target",
            "after",
            None
        )
        .default_action_enabled()
    );
    dom_parent.in_scope(parent.scope(), || {
        assert!(parent.node().request(false, Reason::Trigger, false));
    });
    assert!(
        execute(2).default_action_enabled(),
        "Rust projects the closing destination's hidden-focus no-op before native hide catches up"
    );

    dom_parent.in_scope(parent.scope(), || {
        assert!(parent.node().request(true, Reason::Trigger, false));
    });
    assert!(
        execute(2).default_action_enabled(),
        "a rejected destination projection still consumes that source job once"
    );
    present(&dom_child, &child);
    assert!(!deliver_trap(&dom_child, &child, report(3)).default_action_enabled());
    assert!(
        !deferred_query(
            &dom_child,
            &child,
            3,
            "schedule_focus_target",
            "after",
            None
        )
        .default_action_enabled()
    );
    dom_parent.in_scope(parent.scope(), || {
        assert!(parent.node().request(false, Reason::Trigger, false));
        assert!(parent.node().request(true, Reason::Trigger, false));
    });
    assert!(
        !execute(3).default_action_enabled(),
        "current reopened destination permits a different job from the same closed Dropdown root"
    );
    assert!(
        !child.open(),
        "originating Dropdown visibility does not gate execution"
    );
    assert!(
        execute(3).default_action_enabled(),
        "successful execution consumes once"
    );
}

struct HeaderExitFixture {
    child: Fixture,
    header: NodeHandle,
    header_open: Rc<Cell<bool>>,
    header_veto: Rc<Cell<bool>>,
    order: Rc<RefCell<Vec<String>>>,
}

fn build_header_exit_fixture(child: &Fixture) -> (VirtualDom, HeaderExitFixture) {
    let dom = build_deferred_exit_fixture(child, Rc::new(Cell::new(false)));
    let header_open = Rc::new(Cell::new(false));
    let header_veto = Rc::new(Cell::new(false));
    let order = Rc::new(RefCell::new(Vec::new()));
    let header = dom.in_scope(child.scope(), || {
        let runtime = child.runtime();
        let entry = runtime.entry(child.node().id()).unwrap();
        let mut callbacks = entry.callbacks;
        let before = callbacks.before.clone();
        let before_order = order.clone();
        callbacks.before = Rc::new(move |change| {
            before_order
                .borrow_mut()
                .push(format!("child:before:{}", change.open));
            before(change)
        });
        let apply = callbacks.apply.clone();
        let apply_order = order.clone();
        callbacks.apply = Rc::new(move |change| {
            apply_order
                .borrow_mut()
                .push(format!("child:apply:{}", change.open));
            apply(change);
        });
        let node = runtime.register(
            child.node().id().into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        child.0.borrow_mut().node = Some(node);
        let owner = child.0.borrow().owner.clone().unwrap();
        let scope = child.scope();
        let token = child.0.borrow().token;
        let read_owner = owner.clone();
        let before_owner = owner.clone();
        let read = header_open.clone();
        let apply = header_open.clone();
        let veto = header_veto.clone();
        let before_order = order.clone();
        let apply_order = order.clone();
        runtime.register(
            "source-header-overflow".into(),
            1,
            OverlayKind::Header,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || {
                    assert_owner(&read_owner, scope, token);
                    Some(read.get())
                }),
                before: Rc::new(move |change| {
                    assert_owner(&before_owner, scope, token);
                    before_order
                        .borrow_mut()
                        .push(format!("header:before:{}", change.open));
                    !veto.get()
                }),
                apply: Rc::new(move |change| {
                    assert_owner(&owner, scope, token);
                    apply_order
                        .borrow_mut()
                        .push(format!("header:apply:{}", change.open));
                    apply.set(change.open);
                }),
                key: None,
            },
        )
    });
    (
        dom,
        HeaderExitFixture {
            child: child.clone(),
            header,
            header_open,
            header_veto,
            order,
        },
    )
}

fn header_exit_nodes(
    fixture: &HeaderExitFixture,
    parent: &Fixture,
    nested: bool,
) -> Vec<serde_json::Value> {
    let parent_id = parent.0.borrow().id;
    vec![
        serde_json::json!({
            "id":fixture.child.0.borrow().id,"generation":1,"connected":true,
            "discovery_ready":true,"trigger_ancestors":if nested {vec![fixture.header.id(),parent_id]} else {vec![parent_id]},
            "mode":"active-descendant","trigger_id":"header-child-trigger",
            "inside":true,"on_trigger":true,"tab_candidates":[]
        }),
        serde_json::json!({
            "id":fixture.header.id(),"generation":1,"connected":true,"discovery_ready":true,
            "trigger_ancestors":[parent_id],"trigger_id":"header-more",
            "header_controls":["header-more","header-menu"],"tab_candidates":[]
        }),
        trap_node(
            parent_id,
            &[],
            &[
                ("parent-before", &[parent_id], false),
                ("header-menu", &[parent_id], false),
                ("header-default", &[fixture.header.id(), parent_id], false),
                (
                    "header-child-trigger",
                    &[fixture.header.id(), parent_id],
                    false,
                ),
                ("header-overflow", &[fixture.header.id(), parent_id], false),
                ("header-more", &[parent_id], false),
                ("parent-after", &[parent_id], false),
            ],
        ),
    ]
}

fn observe_header_exit(
    dom: &VirtualDom,
    fixture: &HeaderExitFixture,
    parent: &Fixture,
    document: &str,
    sequence: u64,
    nested: bool,
) {
    let value = serde_json::json!({
        "owner":fixture.child.runtime().owner_id(),"document":document,"sequence":sequence,
        "kind":"observe","nodes":header_exit_nodes(fixture,parent,nested)
    });
    assert!(deliver_trap(dom, &fixture.child, value).default_action_enabled());
}

fn prepare_header_exit(
    dom_parent: &VirtualDom,
    parent: &Fixture,
    dom_child: &VirtualDom,
    fixture: &HeaderExitFixture,
    document: &str,
    retained: bool,
) -> u64 {
    attach(dom_parent, parent, document);
    attach(dom_child, &fixture.child, document);
    observe(
        dom_parent,
        parent,
        document,
        1,
        0,
        false,
        &[parent.0.borrow().id],
    );
    observe_header_exit(dom_child, fixture, parent, document, 1, !retained);
    present(dom_parent, parent);
    dom_child.in_scope(fixture.child.scope(), || {
        assert!(fixture.header.request(true, Reason::Trigger, false));
    });
    present(dom_child, &fixture.child);
    if retained {
        // Same registered instances: Large initially projects the default-slot
        // trigger outside Header overflow, then Small moves its live ancestry.
        observe_header_exit(dom_child, fixture, parent, document, 2, true);
    }
    fixture.order.borrow_mut().clear();
    if retained { 3 } else { 2 }
}

fn header_exit_report(
    fixture: &HeaderExitFixture,
    parent: &Fixture,
    document: &str,
    sequence: u64,
    shift: bool,
) -> serde_json::Value {
    let parent_id = parent.0.borrow().id;
    let candidates = [
        ("parent-before", vec![parent_id]),
        ("header-menu", vec![parent_id]),
        ("header-default", vec![fixture.header.id(), parent_id]),
        ("header-child-trigger", vec![fixture.header.id(), parent_id]),
        ("header-overflow", vec![fixture.header.id(), parent_id]),
        ("header-more", vec![parent_id]),
        ("parent-after", vec![parent_id]),
    ];
    serde_json::json!({
        "owner":fixture.child.runtime().owner_id(),"document":document,"sequence":sequence,
        "kind":"key","phase":"bubble","key":"Tab","shift":shift,
        "hosts":[fixture.header.id(),parent_id],"triggers":[fixture.child.0.borrow().id],
        "active_hosts":[fixture.header.id(),parent_id],"active_popover":parent_id,
        "nodes":header_exit_nodes(fixture,parent,true),
        "tab_order":candidates.iter().map(|(target,_)|*target).collect::<Vec<_>>(),
        "document_tab_candidates":candidates.iter().map(|(target,hosts)|serde_json::json!({
            "target":target,"hosts":hosts,"tab_index":0,"active":*target=="header-child-trigger"
        })).collect::<Vec<_>>()
    })
}

#[test]
fn header_cached_root_veto_is_owner_scoped_and_child_veto_is_requested_once_synchronously() {
    let parent = Fixture::new("header-veto-parent", 701);
    let child = Fixture::new("header-veto-child", 702);
    let dom_parent = build_popover_fixture(&parent);
    let (dom_child, fixture) = build_header_exit_fixture(&child);
    prepare_header_exit(
        &dom_parent,
        &parent,
        &dom_child,
        &fixture,
        "header cached veto document",
        false,
    );
    let domain = child.runtime().domain().unwrap();
    assert_eq!(
        domain.cached_root(child.node().id()).as_deref(),
        Some(fixture.header.id())
    );
    assert!(
        domain.owns_topmost(fixture.header.id()),
        "Header and child have the same source Dropdown kind"
    );
    fixture.header_veto.set(true);
    dom_parent.in_scope(parent.scope(), || {
        assert!(!domain.dismiss_root(child.node().id(), Reason::Keyboard, false));
        assert_eq!(consume_context::<OwnerToken>().0, 701);
    });
    assert_eq!(*fixture.order.borrow(), ["header:before:false"]);
    assert!(fixture.header_open.get() && child.open());
    assert_eq!(
        domain.cached_root(child.node().id()).as_deref(),
        Some(fixture.header.id())
    );

    fixture.order.borrow_mut().clear();
    fixture.header_veto.set(false);
    child.0.borrow().veto.set(true);
    dom_parent.in_scope(parent.scope(), || {
        assert!(domain.dismiss_root(child.node().id(), Reason::Keyboard, false));
        assert_eq!(consume_context::<OwnerToken>().0, 701);
    });
    assert_eq!(
        *fixture.order.borrow(),
        [
            "header:before:false",
            "child:before:false",
            "header:apply:false"
        ]
    );
    assert!(!fixture.header_open.get() && child.open());
    assert!(domain.cached_children(fixture.header.id()).is_empty());
    assert!(parent.open());
    // Source's later second child veto is caused by native trigger visibility;
    // direct Rust dismissal must not manufacture it via a cross-type pass.
}

#[test]
fn header_retained_relocation_preserves_child_cached_root_and_same_type_cascade_boundary() {
    let parent = Fixture::new("header-retained-parent", 703);
    let child = Fixture::new("header-retained-child", 704);
    let dom_parent = build_popover_fixture(&parent);
    let (dom_child, fixture) = build_header_exit_fixture(&child);
    prepare_header_exit(
        &dom_parent,
        &parent,
        &dom_child,
        &fixture,
        "header retained root document",
        true,
    );
    let domain = child.runtime().domain().unwrap();
    assert_eq!(
        domain.cached_root(child.node().id()).as_deref(),
        Some(child.node().id())
    );
    assert_eq!(
        domain.live_ancestors(child.node().id()),
        [fixture.header.id(), parent.0.borrow().id]
    );
    assert!(domain.popover_focus_parent(child.node().id()).is_none());
    dom_parent.in_scope(parent.scope(), || {
        assert!(domain.request(
            fixture.header.id(),
            Change {
                open: false,
                reason: Reason::Trigger,
                restore_focus: false
            }
        ));
    });
    assert_eq!(
        *fixture.order.borrow(),
        ["header:before:false", "header:apply:false"]
    );
    assert!(
        child.open(),
        "uncached same-type live child is not a cross-type child"
    );
    fixture.order.borrow_mut().clear();
    dom_child.in_scope(child.scope(), || {
        assert!(fixture.header.request(true, Reason::Trigger, false));
    });
    fixture.order.borrow_mut().clear();
    dom_parent.in_scope(parent.scope(), || {
        assert!(domain.dismiss_root(child.node().id(), Reason::Keyboard, false));
    });
    assert_eq!(
        *fixture.order.borrow(),
        ["child:before:false", "child:apply:false"]
    );
    assert!(fixture.header_open.get() && !child.open() && parent.open());
}

#[test]
fn header_focus_owner_defers_parent_trap_and_excludes_overflow_across_vdom_owners() {
    let parent = Fixture::new("header-trap-parent", 705);
    let child = Fixture::new("header-trap-child", 706);
    let dom_parent = build_popover_fixture(&parent);
    let (dom_child, fixture) = build_header_exit_fixture(&child);
    prepare_header_exit(
        &dom_parent,
        &parent,
        &dom_child,
        &fixture,
        "header focus owner document",
        false,
    );
    let domain = child.runtime().domain().unwrap();
    assert_eq!(
        domain.node_kind(fixture.header.id(), 1),
        Some(OverlayKind::Header)
    );
    assert_eq!(domain.node_kind(fixture.header.id(), 2), None);
    assert_eq!(
        domain.node_kind(parent.0.borrow().id, 1),
        Some(OverlayKind::Popover)
    );
    let mut parent_open = parent.0.borrow().open.unwrap();
    dom_parent.in_scope(parent.scope(), || parent_open.set(false));
    assert!(
        !domain.has_popover_ancestor(child.node().id()),
        "live ancestry refreshes another owner's authoritative presence"
    );
    dom_parent.in_scope(parent.scope(), || parent_open.set(true));
    assert!(domain.has_popover_ancestor(child.node().id()));
    dom_child.in_scope(child.scope(), || {
        // Reopen Header after the parent presence check so Header is again
        // the last presented source overlay, with its child closed.
        assert!(fixture.header.request(false, Reason::Trigger, false));
        assert!(fixture.header.request(true, Reason::Trigger, false));
    });
    let parent_id = parent.0.borrow().id;
    let mut nodes = header_exit_nodes(&fixture, &parent, true);
    nodes.last_mut().unwrap()["tab_candidates"] = serde_json::json!([
        {"target":"header-menu","hosts":[parent_id],"tab_index":0,"active":false},
        {"target":"parent-before","hosts":[parent_id],"tab_index":0,"active":false},
        {"target":"parent-after","hosts":[parent_id],"tab_index":0,"active":false},
        {"target":"header-overflow","hosts":[fixture.header.id(),parent_id],"tab_index":0,"active":true},
        {"target":"header-more","hosts":[parent_id],"tab_index":0,"active":false}
    ]);
    let mut report = trap_report(&parent, "header focus owner document", 2, nodes);
    report["shift"] = false.into();
    report["active_hosts"] = serde_json::json!([fixture.header.id(), parent_id]);
    assert!(
        deliver_trap(&dom_parent, &parent, report.clone()).default_action_enabled(),
        "focused topmost Header owns its original Tab"
    );
    assert!(trap_query(&dom_parent, &parent, 2, "parent-before").default_action_enabled());
    report["sequence"] = 3.into();
    report["active_hosts"] = serde_json::json!([parent_id]);
    report["nodes"].as_array_mut().unwrap().last_mut().unwrap()["tab_candidates"][2]["active"] =
        true.into();
    report["nodes"].as_array_mut().unwrap().last_mut().unwrap()["tab_candidates"][3]["active"] =
        false.into();
    assert!(
        !deliver_trap(&dom_parent, &parent, report).default_action_enabled(),
        "parent last wraps with open Header host excluded"
    );
    assert!(!trap_query(&dom_parent, &parent, 3, "parent-before").default_action_enabled());
    assert!(fixture.header_open.get() && parent.open());
}

#[test]
fn header_fresh_and_retained_tab_use_source_shadow_projection_and_captured_reverse_trigger() {
    for (retained, shift, token) in [
        (false, false, 707),
        (false, true, 709),
        (true, false, 711),
        (true, true, 713),
    ] {
        let parent = Fixture::new("header-exit-parent", token);
        let child = Fixture::new("header-exit-child", token + 1);
        let dom_parent = build_popover_fixture(&parent);
        let (dom_child, fixture) = build_header_exit_fixture(&child);
        let document = format!("header source exit document {retained} {shift}");
        let sequence = prepare_header_exit(
            &dom_parent,
            &parent,
            &dom_child,
            &fixture,
            &document,
            retained,
        );
        let input = deliver_trap(
            &dom_child,
            &child,
            header_exit_report(&fixture, &parent, &document, sequence, shift),
        );
        assert_eq!(
            !input.default_action_enabled(),
            shift,
            "only the precomputed reverse trigger supplies an exit target"
        );
        assert_eq!(fixture.header_open.get(), retained);
        assert!(!child.open() && parent.open());
        if shift {
            let target = if retained {
                "header-child-trigger"
            } else {
                "header-more"
            };
            assert!(
                !deferred_query(
                    &dom_child,
                    &child,
                    sequence,
                    "schedule_focus_target",
                    target,
                    None
                )
                .default_action_enabled()
            );
            assert!(
                !deferred_query(
                    &dom_child,
                    &child,
                    sequence,
                    "pending_focus_target",
                    target,
                    None
                )
                .default_action_enabled()
            );
            assert!(
                deferred_query(
                    &dom_child,
                    &child,
                    sequence,
                    "pending_focus_target",
                    target,
                    None
                )
                .default_action_enabled()
            );
        } else {
            for target in [
                "header-more",
                "header-overflow",
                "parent-after",
                "parent-before",
            ] {
                assert!(
                    deferred_query(
                        &dom_child,
                        &child,
                        sequence,
                        "schedule_focus_target",
                        target,
                        None
                    )
                    .default_action_enabled(),
                    "missing source-shadow current target admits no guessed adjacent focus"
                );
            }
        }
    }
}

#[test]
fn header_reverse_jobs_survive_root_veto_and_newer_reports_but_not_dropped_owner() {
    let parent = Fixture::new("header-queued-parent", 715);
    let child = Fixture::new("header-queued-child", 716);
    let dom_parent = build_popover_fixture(&parent);
    let (dom_child, fixture) = build_header_exit_fixture(&child);
    let document = "header queued owner document";
    prepare_header_exit(&dom_parent, &parent, &dom_child, &fixture, document, false);
    fixture.header_veto.set(true);
    for sequence in [2, 3] {
        assert!(
            !deliver_trap(
                &dom_child,
                &child,
                header_exit_report(&fixture, &parent, document, sequence, true)
            )
            .default_action_enabled()
        );
        assert!(
            !deferred_query(
                &dom_child,
                &child,
                sequence,
                "schedule_focus_target",
                "header-more",
                None
            )
            .default_action_enabled()
        );
    }
    assert_eq!(
        *fixture.order.borrow(),
        ["header:before:false", "header:before:false"]
    );
    assert!(fixture.header_open.get() && child.open());
    observe_header_exit(&dom_child, &fixture, &parent, document, 4, true);
    assert!(
        !deferred_query(
            &dom_child,
            &child,
            2,
            "pending_focus_target",
            "header-more",
            None
        )
        .default_action_enabled(),
        "newer independent job and ACK do not revoke the first captured reverse trigger"
    );
    // Retain the overlay RuntimeHandle but drop its actual Dioxus owner. A
    // remaining value-only job cannot recover the dead registered scope.
    drop(dom_child);
    assert!(child.0.borrow().owner.as_ref().unwrap().upgrade().is_none());
    let event = trap_form(serde_json::json!({
        "owner":child.runtime().owner_id(),"sequence":3,"pending_focus_target":"header-more"
    }));
    dom_parent.in_scope(parent.scope(), || child.runtime().query(event.clone()));
    assert!(
        event.default_action_enabled(),
        "dropped owner cannot authorize the remaining independent job"
    );
}

#[test]
fn header_child_veto_retries_only_on_matching_admitted_native_visibility_delivery() {
    let parent = Fixture::new("header-visibility-parent", 717);
    let child = Fixture::new("header-visibility-child", 718);
    let dom_parent = build_popover_fixture(&parent);
    let (dom_child, fixture) = build_header_exit_fixture(&child);
    let document = "header child native visibility document";
    prepare_header_exit(&dom_parent, &parent, &dom_child, &fixture, document, false);
    child.0.borrow().veto.set(true);
    dom_child.in_scope(child.scope(), || {
        assert!(fixture.header.request(false, Reason::Keyboard, false));
    });
    let synchronous = vec![
        "header:before:false".to_string(),
        "child:before:false".to_string(),
        "header:apply:false".to_string(),
    ];
    assert_eq!(*fixture.order.borrow(), synchronous);
    assert!(!fixture.header_open.get() && child.open());
    observe_header_exit(&dom_child, &fixture, &parent, document, 2, true);
    assert_eq!(
        *fixture.order.borrow(),
        synchronous,
        "ordinary native topology observations do not invent another dismissal"
    );
    let visibility = |sequence, generation, target, intersecting| {
        let mut nodes = header_exit_nodes(&fixture, &parent, true);
        nodes[0]["anchor_id"] = "header-child-anchor".into();
        serde_json::json!({
            "owner":child.runtime().owner_id(),"document":document,"sequence":sequence,
            "kind":"anchor-visibility","nodes":nodes,
            "anchor_visibility":{"root":child.0.borrow().id,"generation":generation,
                "target":target,"intersecting":intersecting}
        })
    };
    for report in [
        visibility(3, 1, "header-child-anchor", true),
        visibility(4, 1, "header-child-trigger", false),
        visibility(5, 2, "header-child-anchor", false),
    ] {
        assert!(deliver_trap(&dom_child, &child, report).default_action_enabled());
        assert_eq!(
            *fixture.order.borrow(),
            synchronous,
            "positive, wrong-anchor and stale-generation facts do not request close"
        );
    }
    fixture.child.node().set_trigger_visibility_check(false);
    assert!(
        deliver_trap(
            &dom_child,
            &child,
            visibility(6, 1, "header-child-anchor", false)
        )
        .default_action_enabled()
    );
    assert_eq!(
        *fixture.order.borrow(),
        synchronous,
        "explicit source visibility suppression vetoes admission"
    );
    fixture.child.node().set_trigger_visibility_check(true);
    assert!(
        deliver_trap(
            &dom_child,
            &child,
            visibility(7, 1, "header-child-anchor", false)
        )
        .default_action_enabled()
    );
    let mut after_delivery = synchronous;
    after_delivery.push("child:before:false".into());
    assert_eq!(
        *fixture.order.borrow(),
        after_delivery,
        "one real false observer delivery supplies the later vetoed request"
    );
    assert!(
        child.open(),
        "the later native visibility request still honors the child veto"
    );
    // A separate legitimate observer event may ask again while veto retains
    // the child; it is not a replay of the original synchronous cascade.
    assert!(
        deliver_trap(
            &dom_child,
            &child,
            visibility(8, 1, "header-child-anchor", false)
        )
        .default_action_enabled()
    );
    after_delivery.push("child:before:false".into());
    assert_eq!(*fixture.order.borrow(), after_delivery);
    dom_child.in_scope(child.scope(), || child.node().unregister());
    assert!(
        deliver_trap(
            &dom_child,
            &child,
            visibility(9, 1, "header-child-anchor", false)
        )
        .default_action_enabled()
    );
    assert_eq!(
        *fixture.order.borrow(),
        after_delivery,
        "dropped registration cannot receive a late native visibility request"
    );
}
