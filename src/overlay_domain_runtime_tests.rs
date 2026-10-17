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
