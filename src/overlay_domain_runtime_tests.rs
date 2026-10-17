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
