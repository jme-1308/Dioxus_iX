//! Actual independent VirtualDom owner-dispatch regressions. Reports below are
//! Rust inputs to the real runtime handler, not native bridge/browser evidence.

use super::*;
use dioxus::html::Modifiers;
use std::cell::Cell;

thread_local! {
    static FIXTURE_RUNTIMES: RefCell<Vec<Weak<RefCell<Runtime>>>> = const { RefCell::new(Vec::new()) };
}
fn remember_fixture_runtime(runtime: &RuntimeHandle) {
    FIXTURE_RUNTIMES.with(|runtimes| {
        let mut runtimes = runtimes.borrow_mut();
        runtimes.retain(|runtime| runtime.strong_count() > 0);
        if !runtimes
            .iter()
            .any(|old| old.ptr_eq(&Rc::downgrade(&runtime.0)))
        {
            runtimes.push(Rc::downgrade(&runtime.0));
        }
    });
}
fn fixture_node(id: &str) -> NodeHandle {
    FIXTURE_RUNTIMES
        .with(|runtimes| {
            runtimes.borrow().iter().rev().find_map(|weak| {
                let runtime = RuntimeHandle(weak.upgrade()?);
                let entry = runtime.entry(id)?;
                Some(NodeHandle {
                    runtime: Rc::downgrade(&runtime.0),
                    id: id.into(),
                    generation: entry.generation,
                })
            })
        })
        .unwrap_or_else(|| panic!("missing explicit Rust fixture registration {id}"))
}
fn set_fixture_ownership(id: &str, ancestors: &[&str]) {
    let node = fixture_node(id);
    node.set_ownership(ancestors.iter().map(|id| fixture_node(id)).collect(), true);
    node.set_mounted();
}

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
    applied: Vec<Change>,
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
            applied: Vec::new(),
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
                    let fixture = before_fixture
                        .upgrade()
                        .expect("registered fixture is live");
                    let mut state = fixture.borrow_mut();
                    state.calls.push(("before".into(), change.open));
                    !veto.get()
                }),
                apply: Rc::new(move |change| {
                    assert_owner(&apply_owner, scope, token);
                    {
                        let fixture = apply_fixture.upgrade().expect("registered fixture is live");
                        let mut state = fixture.borrow_mut();
                        state.calls.push(("apply".into(), change.open));
                        state.applied.push(change);
                    }
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
    remember_fixture_runtime(&fixture.runtime());
    set_fixture_ownership(fixture.node().id(), &[]);
    dom
}

fn attach(dom: &VirtualDom, fixture: &Fixture, document: &str) {
    dom.in_scope(fixture.scope(), || {
        fixture.runtime().attach_document(document)
    });
}

fn observe(dom: &VirtualDom, fixture: &Fixture, document: &str, sequence: u64, ids: &[&str]) {
    let report: Report = serde_json::from_value(serde_json::json!({
        "owner": fixture.runtime().owner_id(),"document": document,"sequence": sequence,"kind":"observe",
        "nodes": ids.iter().map(|id| serde_json::json!({
            "id": id,"generation":1
        })).collect::<Vec<_>>()
    })).unwrap();
    dom.in_scope(fixture.scope(), || {
        let before = dioxus::core::Runtime::current();
        fixture.runtime().observe(&report);
        assert!(Rc::ptr_eq(&before, &dioxus::core::Runtime::current()));
        assert_eq!(consume_context::<OwnerToken>().0, fixture.0.borrow().token);
    });
}

fn document_report(
    fixture: &Fixture,
    document: &str,
    sequence: u64,
    delivery: u64,
    action: &str,
) -> serde_json::Value {
    serde_json::json!({"owner":fixture.runtime().owner_id(),"document":document,"sequence":sequence,"kind":"document",
        "document_event":{"sequence":delivery,"action":action}})
}

fn document_input(dom: &VirtualDom, fixture: &Fixture, report: serde_json::Value, prevented: bool) {
    let event = trap_form(report);
    if prevented {
        event.prevent_default();
    }
    dom.in_scope(fixture.scope(), || {
        let before = dioxus::core::Runtime::current();
        fixture.runtime().input(event.clone());
        assert_eq!(
            event.default_action_enabled(),
            !prevented,
            "document input preserves original defaults"
        );
        assert!(Rc::ptr_eq(&before, &dioxus::core::Runtime::current()));
        assert_eq!(consume_context::<OwnerToken>().0, fixture.0.borrow().token);
    });
}

fn present(dom: &VirtualDom, fixture: &Fixture) {
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(true, Reason::Trigger, false));
    });
    fixture.0.borrow_mut().calls.clear();
}

#[test]
fn dioxus_inside_click_enters_each_real_owner_and_preserves_foreign_inside_policy() {
    let origin = Fixture::new("host-inside-origin", 811);
    let peer = Fixture::new("host-inside-peer", 812);
    let protected = Fixture::new("host-inside-protected", 813);
    let a = build(&origin);
    let b = build(&peer);
    let c = build(&protected);
    let owners = [(&a, &origin), (&b, &peer), (&c, &protected)];
    let document = "host-inside-cross-owner-document";
    let facts = anchor_focus_nodes(&[(&origin, &[]), (&peer, &[]), (&protected, &[])]);
    prepare_others(&owners, document, &facts);
    origin.node().set_close_behavior(CloseBehavior::Inside);
    protected.node().set_close_behavior(CloseBehavior::Inside);
    for (dom, fixture) in &owners {
        present(dom, fixture);
    }
    let click = super::tests::typed_mouse();
    a.in_scope(origin.scope(), || {
        origin.node().dioxus_inside_click(&click);
        assert_eq!(dioxus::core::current_scope_id(), origin.scope());
        assert_eq!(consume_context::<OwnerToken>().0, origin.0.borrow().token);
    });
    assert!(click.default_action_enabled() && !click.propagates());
    assert!(!origin.open() && !peer.open() && protected.open());
    assert_eq!(
        origin.0.borrow().calls,
        [("before".into(), false), ("apply".into(), false)]
    );
    assert_eq!(
        peer.0.borrow().calls,
        [("before".into(), false), ("apply".into(), false)]
    );
    assert!(protected.0.borrow().calls.is_empty());
    present(&a, &origin);
    present(&b, &peer);
    let stale = NodeHandle {
        generation: 2,
        ..origin.node()
    };
    let click = super::tests::typed_mouse();
    a.in_scope(origin.scope(), || stale.dioxus_inside_click(&click));
    assert!(click.default_action_enabled() && click.propagates());
    assert!(
        origin.open() && peer.open(),
        "a stale owner cannot dismiss another root's current state"
    );
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
        &["owner-runtime-a"],
    );
    observe(
        &dom_b,
        &b,
        "owner-runtime-native-document",
        1,
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
fn minimal_document_escape_deduplicates_across_real_roots_and_retains_veto() {
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
    observe(&dom_a, &a, document, 1, &["window-root-a"]);
    observe(&dom_b, &b, document, 1, &["window-root-b"]);
    observe(&dom_c, &c, separate, 1, &["window-other-document"]);
    present(&dom_a, &a);
    present(&dom_b, &b);
    present(&dom_c, &c);
    a.0.borrow().veto.set(true);
    // One minimal document event closes the selected owner. A duplicated
    // transport delivery cannot dismiss the next owner.
    document_input(
        &dom_a,
        &a,
        document_report(&a, document, 2, 1, "escape"),
        false,
    );
    assert!(a.open() && !b.open() && c.open());
    assert!(a.0.borrow().calls.is_empty());
    assert_eq!(
        b.0.borrow().calls,
        [("before".into(), false), ("apply".into(), false)]
    );
    document_input(
        &dom_b,
        &b,
        document_report(&b, document, 2, 1, "escape"),
        false,
    );
    assert!(
        a.0.borrow().calls.is_empty(),
        "second owner cannot dismiss the next overlay for one document event"
    );
    // Global Escape is independent of an original event default cancellation.
    document_input(
        &dom_b,
        &b,
        document_report(&b, document, 3, 2, "escape"),
        true,
    );
    assert!(a.open() && !b.open() && c.open());
    assert_eq!(a.0.borrow().calls, [("before".into(), false)]);
    assert!(c.0.borrow().calls.is_empty());

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

#[test]
fn minimal_document_outside_preserves_current_hierarchy_and_enters_foreign_owners() {
    let root = Fixture::new("document-outside-root", 304);
    let child = Fixture::new("document-outside-child", 305);
    let dom_root = build(&root);
    let dom_child = build(&child);
    let document = "minimal outside shared document";
    let nodes = anchor_focus_nodes(&[(&root, &[]), (&child, &["document-outside-root"])]);
    prepare_anchor_focus(
        &[(&dom_root, &root), (&dom_child, &child)],
        document,
        &nodes,
    );
    let domain = root.runtime().domain().unwrap();
    let mut protected = document_report(&root, document, 3, 2, "outside");
    protected["document_event"]["hosts"] = serde_json::json!([child.node().id()]);
    document_input(&dom_root, &root, protected, false);
    assert!(
        root.open() && child.open(),
        "an inside child protects its current ancestors"
    );
    assert_eq!(
        root.runtime()
            .0
            .borrow()
            .observation
            .as_ref()
            .unwrap()
            .sequence,
        1
    );
    assert_eq!(
        domain.cached_parent(child.node().id()).as_deref(),
        Some(root.node().id())
    );
    child.0.borrow().veto.set(true);
    document_input(
        &dom_root,
        &root,
        document_report(&root, document, 4, 3, "outside"),
        true,
    );
    assert!(
        !root.open() && child.open(),
        "outside dismissal enters the child's owner and honors veto"
    );
    assert_eq!(
        domain.cached_parent(child.node().id()).as_deref(),
        Some(root.node().id())
    );
    assert_eq!(
        child
            .runtime()
            .0
            .borrow()
            .observation
            .as_ref()
            .unwrap()
            .sequence,
        1
    );
    child.0.borrow().veto.set(false);
    document_input(
        &dom_child,
        &child,
        document_report(&child, document, 2, 4, "outside"),
        false,
    );
    assert!(!child.open());
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
    set_fixture_ownership(id, ancestors);
    serde_json::json!({
        "id":id,"generation":1,"trap_enabled":true,
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
    serde_json::json!({
        "owner":fixture.runtime().owner_id(),"document":document,"sequence":sequence,
        "kind":"key","phase":"trap","key":"Tab","shift":true,
        "active_hosts":[fixture.0.borrow().id],"active_popover":fixture.0.borrow().id,
        "nodes":nodes
    })
}
struct DomainKey(String, Modifiers);
impl dioxus::html::ModifiersInteraction for DomainKey {
    fn modifiers(&self) -> Modifiers {
        self.1
    }
}
impl dioxus::html::HasKeyboardData for DomainKey {
    fn key(&self) -> dioxus::html::Key {
        use dioxus::html::Key;
        match self.0.as_str() {
            "Tab" => Key::Tab,
            "Escape" => Key::Escape,
            "Home" => Key::Home,
            "End" => Key::End,
            "ArrowDown" => Key::ArrowDown,
            "ArrowUp" => Key::ArrowUp,
            "ArrowLeft" => Key::ArrowLeft,
            "ArrowRight" => Key::ArrowRight,
            "Enter" => Key::Enter,
            value => Key::Character(value.into()),
        }
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
fn deliver_trap(dom: &VirtualDom, fixture: &Fixture, report: serde_json::Value) -> FormEvent {
    let event = trap_form(report.clone());
    let runtime = fixture.runtime();
    let node = fixture.node();
    if report["kind"] == "key"
        && runtime
            .entry(node.id())
            .is_some_and(|entry| entry.kind == OverlayKind::Dropdown)
    {
        // Fixture JSON supplies only current order/tab facts. The key itself
        // enters the same Rust API used by Dioxus KeyboardEvent handlers.
        let mut observation = report.clone();
        observation["kind"] = "observe".into();
        dom.in_scope(fixture.scope(), || runtime.input(trap_form(observation)));
        let source = report["nodes"]
            .as_array()
            .and_then(|nodes| nodes.iter().find(|fact| fact["id"] == node.id()));
        if let Some(mode) = source.and_then(|fact| fact["mode"].as_str()) {
            dom.in_scope(fixture.scope(), || node.set_navigation_mode(mode));
        }
        let contains = |field: &str| {
            report[field]
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value == node.id()))
        };
        let on_trigger =
            contains("triggers") || source.is_some_and(|source| source["on_trigger"] == true);
        let inside = contains("hosts") || source.is_some_and(|source| source["inside"] == true);
        let mut modifiers = Modifiers::empty();
        if report["shift"] == true {
            modifiers |= Modifiers::SHIFT;
        }
        if report["alt"] == true {
            modifiers |= Modifiers::ALT;
        }
        if report["ctrl"] == true {
            modifiers |= Modifiers::CONTROL;
        }
        if report["meta"] == true {
            modifiers |= Modifiers::META;
        }
        let key = Event::new(
            Rc::new(dioxus::html::KeyboardData::new(DomainKey(
                report["key"].as_str().unwrap_or_default().into(),
                modifiers,
            ))),
            true,
        );
        if report["default_prevented"] == true {
            key.prevent_default();
        }
        let focused = report["active_id"].as_str().map(str::to_owned);
        let decision = dom.in_scope(fixture.scope(), || {
            node.dioxus_key_facts(&key, on_trigger, focused, inside)
                .map(|facts| node.dioxus_key(facts))
                .unwrap_or_default()
        });
        if decision.prevent {
            event.prevent_default();
        }
        if decision.stop || decision.immediate_stop {
            event.stop_propagation();
        }
    } else {
        dom.in_scope(fixture.scope(), || runtime.input(event.clone()));
    }
    event
}

#[test]
fn document_trap_handler_preserves_interior_empty_prevented_and_focuses_registered_boundary_once() {
    let fixture = Fixture::new("document-trap-policy", 501);
    let mut dom = build_popover_fixture(&fixture);
    let document = "document trap handler policy";
    attach(&dom, &fixture, document);
    observe(&dom, &fixture, document, 1, &["document-trap-policy"]);
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
    let mounted = register_exit_mount(&dom, &fixture, "last");
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
    assert_eq!(fixture.runtime().pending_item_focus_count(), 1);
    assert!(mounted.calls.borrow().is_empty());
    settle_exit(&mut dom);
    assert_eq!(*mounted.calls.borrow(), [("last".into(), fixture.scope())]);
    settle_exit(&mut dom);
    assert_eq!(
        mounted.calls.borrow().len(),
        1,
        "one admitted Tab focuses once"
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
    let mut disabled = trap_report(&fixture, document, 6, vec![node()]);
    disabled["nodes"][0]["trap_enabled"] = false.into();
    assert!(deliver_trap(&dom, &fixture, disabled).default_action_enabled());
    let mut other_phase = trap_report(&fixture, document, 7, vec![node()]);
    other_phase["phase"] = "fallback".into();
    assert!(
        deliver_trap(&dom, &fixture, other_phase).default_action_enabled(),
        "boundary policy belongs to document trap phase"
    );
    dom.in_scope(fixture.scope(), || {
        fixture.runtime().unregister_item_cue("last")
    });
    assert!(
        deliver_trap(
            &dom,
            &fixture,
            trap_report(&fixture, document, 8, vec![node()])
        )
        .default_action_enabled(),
        "an unregistered boundary preserves native Tab"
    );
    assert!(
        fixture.0.borrow().calls.is_empty(),
        "Tab trap never runs visibility callbacks"
    );
}

#[test]
fn document_trap_handler_rejects_closed_reopened_and_replaced_focus_authorizations() {
    let fixture = Fixture::new("document-trap-stale", 502);
    let mut dom = build_popover_fixture(&fixture);
    let document = "document trap stale authorization";
    attach(&dom, &fixture, document);
    observe(&dom, &fixture, document, 1, &["document-trap-stale"]);
    present(&dom, &fixture);
    let mounted = register_exit_mount(&dom, &fixture, "last");
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
    settle_exit(&mut dom);
    assert!(
        mounted.calls.borrow().is_empty(),
        "closed host cannot execute open-trap authorization"
    );
    present(&dom, &fixture);
    assert!(!deliver_trap(&dom, &fixture, report(3)).default_action_enabled());
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(false, Reason::Trigger, false));
        assert!(fixture.node().request(true, Reason::Trigger, false));
    });
    settle_exit(&mut dom);
    assert!(
        mounted.calls.borrow().is_empty(),
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
        replacement.set_ownership(Vec::new(), true);
        replacement.set_mounted();
        fixture.0.borrow_mut().node = Some(replacement);
    });
    settle_exit(&mut dom);
    assert!(
        mounted.calls.borrow().is_empty(),
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
    let mut dom_parent = build_popover_fixture(&parent);
    let mut dom_child = build_popover_fixture(&child);
    let document = "cross-root document traps";
    attach(&dom_parent, &parent, document);
    attach(&dom_child, &child, document);
    observe(&dom_parent, &parent, document, 1, &["trap-parent"]);
    observe(&dom_child, &child, document, 1, &["trap-child"]);
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
    let child_last = register_exit_mount(&dom_child, &child, "child-last");
    let parent_first = register_exit_mount(&dom_parent, &parent, "parent-first");
    let parent_last = register_exit_mount(&dom_parent, &parent, "parent-last");
    let mut focused_child = trap_report(&parent, document, 2, vec![parent_node(), child_node()]);
    focused_child["active_hosts"] = serde_json::json!(["trap-child", "trap-parent"]);
    focused_child["active_popover"] = "trap-child".into();
    assert!(
        deliver_trap(&dom_parent, &parent, focused_child.clone()).default_action_enabled(),
        "foreign focused Popover owns its trap"
    );
    focused_child["owner"] = child.runtime().owner_id().into();
    assert!(!deliver_trap(&dom_child, &child, focused_child).default_action_enabled());
    settle_exit(&mut dom_child);
    assert_eq!(
        *child_last.calls.borrow(),
        [("child-last".into(), child.scope())]
    );

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
    settle_exit(&mut dom_parent);
    assert_eq!(
        *parent_first.calls.borrow(),
        [("parent-first".into(), parent.scope())]
    );

    drop(dom_child);
    let after_drop = trap_report(&parent, document, 5, vec![parent_node()]);
    assert!(
        !deliver_trap(&dom_parent, &parent, after_drop).default_action_enabled(),
        "dropped foreign owner no longer defers live parent"
    );
    settle_exit(&mut dom_parent);
    assert_eq!(
        *parent_last.calls.borrow(),
        [("parent-last".into(), parent.scope())]
    );
    assert!(parent.open());
}

fn build_exit_fixture(fixture: &Fixture) -> VirtualDom {
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
            close.runtime().unwrap().close_and_release_focus(close.id());
            KeyDecision {
                release_focus: true,
                ..Default::default()
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

fn exit_report(fixture: &Fixture, document: &str, sequence: u64) -> serde_json::Value {
    let id = fixture.0.borrow().id;
    serde_json::json!({
        "owner":fixture.runtime().owner_id(),"document":document,"sequence":sequence,
        "kind":"key","phase":"capture","key":"Tab","hosts":[id],"triggers":[id],
        "tab_order":["before","trigger","after"],
        "nodes":[{"id":id,"generation":fixture.node().generation(),
            "mode":"roving-tabindex","trigger_id":"trigger",
            "inside":true,"on_trigger":true}]
    })
}

fn exit_observe(dom: &VirtualDom, fixture: &Fixture, document: &str, sequence: u64) {
    let value = serde_json::json!({
        "owner":fixture.runtime().owner_id(),"document":document,"sequence":sequence,
        "kind":"observe","nodes":[{"id":fixture.0.borrow().id,
            "generation":fixture.node().generation()}]
    });
    assert!(deliver_trap(dom, fixture, value).default_action_enabled());
}

#[derive(Clone)]
struct ExitMount {
    target: String,
    calls: Rc<RefCell<Vec<(String, ScopeId)>>>,
}
impl dioxus::html::RenderedElementBacking for ExitMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let mount = self.clone();
        Box::pin(async move {
            mount
                .calls
                .borrow_mut()
                .push((mount.target, dioxus::core::current_scope_id()));
            Ok(())
        })
    }
}
fn register_exit_mount(dom: &VirtualDom, fixture: &Fixture, target: &str) -> ExitMount {
    let mount = ExitMount {
        target: target.into(),
        calls: Rc::new(RefCell::new(Vec::new())),
    };
    dom.in_scope(fixture.scope(), || {
        fixture.runtime().register_item_mount(
            target,
            target,
            Rc::new(MountedData::new(mount.clone())),
        )
    });
    mount
}
fn settle_exit(dom: &mut VirtualDom) {
    super::tests::settle_item_effects(dom);
}

#[test]
fn mounted_tab_exit_focuses_forward_or_reverse_and_preserves_close_veto() {
    for (shift, veto, token) in [(false, false, 601), (true, false, 602), (false, true, 603)] {
        let fixture = Fixture::new("mounted-tab-exit", token);
        let mut dom = build_exit_fixture(&fixture);
        let document = format!("mounted Tab exit {token}");
        attach(&dom, &fixture, &document);
        exit_observe(&dom, &fixture, &document, 1);
        let destination =
            register_exit_mount(&dom, &fixture, if shift { "trigger" } else { "after" });
        present(&dom, &fixture);
        fixture.0.borrow().veto.set(veto);
        let mut report = exit_report(&fixture, &document, 2);
        report["shift"] = shift.into();
        assert!(!deliver_trap(&dom, &fixture, report).default_action_enabled());
        assert_eq!(fixture.open(), veto);
        assert!(
            destination.calls.borrow().is_empty(),
            "focus belongs to the Dioxus effect"
        );
        settle_exit(&mut dom);
        assert_eq!(
            *destination.calls.borrow(),
            [(destination.target.clone(), fixture.scope())]
        );
        assert_eq!(fixture.runtime().pending_item_focus_count(), 0);
    }
}

#[test]
fn mounted_tab_exit_rejects_unregistered_unmounted_replaced_and_removed_owners() {
    for (change, token) in [
        ("unregistered", 604),
        ("unmount", 605),
        ("replacement", 606),
        ("owner", 607),
    ] {
        let fixture = Fixture::new("mounted-tab-lifetime", token);
        let mut dom = build_exit_fixture(&fixture);
        let document = format!("mounted Tab lifetime {token}");
        attach(&dom, &fixture, &document);
        exit_observe(&dom, &fixture, &document, 1);
        let destination =
            (change != "unregistered").then(|| register_exit_mount(&dom, &fixture, "after"));
        present(&dom, &fixture);
        let event = deliver_trap(&dom, &fixture, exit_report(&fixture, &document, 2));
        assert_eq!(!event.default_action_enabled(), destination.is_some());
        let replacement = dom.in_scope(fixture.scope(), || match change {
            "unmount" => {
                fixture.runtime().unregister_item_cue("after");
                None
            }
            "replacement" => Some(register_exit_mount(&dom, &fixture, "after")),
            "owner" => {
                fixture.node().unregister();
                None
            }
            _ => None,
        });
        settle_exit(&mut dom);
        if let Some(destination) = destination {
            assert!(
                destination.calls.borrow().is_empty(),
                "obsolete {change} request must not focus"
            );
        }
        if let Some(replacement) = replacement {
            assert!(
                replacement.calls.borrow().is_empty(),
                "replacement needs a fresh intent"
            );
        }
        assert_eq!(fixture.runtime().pending_item_focus_count(), 0);
    }
}

#[test]
fn mounted_tab_exit_keeps_only_latest_current_intent() {
    let fixture = Fixture::new("mounted-tab-latest", 618);
    let mut dom = build_exit_fixture(&fixture);
    let document = "mounted latest Tab intent document";
    attach(&dom, &fixture, document);
    exit_observe(&dom, &fixture, document, 1);
    let forward = register_exit_mount(&dom, &fixture, "after");
    let reverse = register_exit_mount(&dom, &fixture, "trigger");
    present(&dom, &fixture);
    fixture.0.borrow().veto.set(true);
    assert!(
        !deliver_trap(&dom, &fixture, exit_report(&fixture, document, 2)).default_action_enabled()
    );
    let mut report = exit_report(&fixture, document, 3);
    report["shift"] = true.into();
    assert!(!deliver_trap(&dom, &fixture, report).default_action_enabled());
    assert_eq!(fixture.runtime().pending_item_focus_count(), 1);
    settle_exit(&mut dom);
    assert!(forward.calls.borrow().is_empty());
    assert_eq!(
        *reverse.calls.borrow(),
        [("trigger".into(), fixture.scope())]
    );
}

#[test]
fn mounted_tab_exit_resolves_same_document_foreign_owner_and_rejects_other_documents() {
    for (same_document, token) in [(true, 608), (false, 610)] {
        let origin = Fixture::new("mounted-tab-origin", token);
        let destination = Fixture::new("mounted-tab-destination", token + 1);
        let mut dom = build_exit_fixture(&origin);
        let destination_dom = build(&destination);
        let document = format!("mounted foreign Tab {token}");
        attach(&dom, &origin, &document);
        attach(
            &destination_dom,
            &destination,
            if same_document {
                &document
            } else {
                "isolated destination document"
            },
        );
        exit_observe(&dom, &origin, &document, 1);
        let mounted = register_exit_mount(&destination_dom, &destination, "after");
        present(&dom, &origin);
        let event = deliver_trap(&dom, &origin, exit_report(&origin, &document, 2));
        assert_eq!(!event.default_action_enabled(), same_document);
        settle_exit(&mut dom);
        assert_eq!(mounted.calls.borrow().len(), usize::from(same_document));
        if same_document {
            assert_eq!(
                mounted.calls.borrow()[0].1,
                origin.scope(),
                "mounted backing executes in the originating owner"
            );
        }
        assert!(!origin.open());
    }
}

#[test]
fn mounted_tab_exit_rejects_destination_document_change_and_dropped_foreign_owner() {
    for (drop_destination, token) in [(false, 612), (true, 614)] {
        let origin = Fixture::new("mounted-tab-document-origin", token);
        let destination = Fixture::new("mounted-tab-document-destination", token + 1);
        let mut dom = build_exit_fixture(&origin);
        let destination_dom = build(&destination);
        let document = format!("mounted changing document {token}");
        attach(&dom, &origin, &document);
        attach(&destination_dom, &destination, &document);
        exit_observe(&dom, &origin, &document, 1);
        let mounted = register_exit_mount(&destination_dom, &destination, "after");
        present(&dom, &origin);
        assert!(
            !deliver_trap(&dom, &origin, exit_report(&origin, &document, 2))
                .default_action_enabled()
        );
        if drop_destination {
            drop(destination_dom);
        } else {
            attach(&destination_dom, &destination, "moved destination document");
        }
        settle_exit(&mut dom);
        assert!(mounted.calls.borrow().is_empty());
    }
}

#[test]
fn mounted_dropdown_tab_reaches_registered_adjacent_control_inside_popover() {
    let parent = Fixture::new("mounted-tab-popover", 616);
    let child = Fixture::new("mounted-tab-popover-child", 617);
    let parent_dom = build_popover_fixture(&parent);
    let mut child_dom = build_exit_fixture(&child);
    let document = "mounted Popover child Tab document";
    attach(&parent_dom, &parent, document);
    attach(&child_dom, &child, document);
    let parent_node = trap_node(
        parent.node().id(),
        &[],
        &[
            ("before", &[parent.node().id()], false),
            ("trigger", &[parent.node().id()], true),
            ("after", &[parent.node().id()], false),
        ],
    );
    let mut report = exit_report(&child, document, 1);
    report["kind"] = "observe".into();
    child.node().set_ownership(vec![parent.node()], true);
    report["nodes"].as_array_mut().unwrap().push(parent_node);
    deliver_trap(&child_dom, &child, report.clone());
    let mounted = register_exit_mount(&parent_dom, &parent, "after");
    present(&parent_dom, &parent);
    present(&child_dom, &child);
    report["kind"] = "key".into();
    report["sequence"] = 2.into();
    assert!(!deliver_trap(&child_dom, &child, report).default_action_enabled());
    settle_exit(&mut child_dom);
    assert!(!child.open() && parent.open());
    assert_eq!(*mounted.calls.borrow(), [("after".into(), child.scope())]);
}

struct HeaderExitFixture {
    child: Fixture,
    header: NodeHandle,
    header_open: Rc<Cell<bool>>,
    header_veto: Rc<Cell<bool>>,
    order: Rc<RefCell<Vec<String>>>,
}

fn build_header_exit_fixture(child: &Fixture) -> (VirtualDom, HeaderExitFixture) {
    let dom = build_exit_fixture(child);
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
    fixture.header.set_ownership(vec![parent.node()], true);
    fixture.header.set_mounted();
    fixture.child.node().set_ownership(
        if nested {
            vec![fixture.header.clone(), parent.node()]
        } else {
            vec![parent.node()]
        },
        true,
    );
    fixture.child.node().set_mounted();
    vec![
        serde_json::json!({
            "id":fixture.child.0.borrow().id,"generation":1,
            "mode":"active-descendant","trigger_id":"header-child-trigger",
            "inside":true,"on_trigger":true,"tab_candidates":[]
        }),
        serde_json::json!({
            "id":fixture.header.id(),"generation":1,
            "trigger_id":"header-more",
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
    relocate_after_open: bool,
) -> u64 {
    attach(dom_parent, parent, document);
    attach(dom_child, &fixture.child, document);
    observe(dom_parent, parent, document, 1, &[parent.0.borrow().id]);
    observe_header_exit(
        dom_child,
        fixture,
        parent,
        document,
        1,
        !relocate_after_open,
    );
    present(dom_parent, parent);
    dom_child.in_scope(fixture.child.scope(), || {
        assert!(fixture.header.request(true, Reason::Trigger, false));
    });
    present(dom_child, &fixture.child);
    if relocate_after_open {
        // Same registered instances: Large initially projects the default-slot
        // trigger outside Header overflow, then Small moves its live ancestry.
        observe_header_exit(dom_child, fixture, parent, document, 2, true);
    }
    fixture.order.borrow_mut().clear();
    if relocate_after_open { 3 } else { 2 }
}

#[test]
fn header_logical_root_veto_is_owner_scoped_and_child_veto_is_requested_once() {
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
    assert_eq!(
        domain.cached_children(fixture.header.id()),
        [child.node().id()],
        "closing visibility preserves the current logical child"
    );
    assert!(parent.open());
    // Source's later second child veto is caused by native trigger visibility;
    // direct Rust dismissal must not manufacture it via a cross-type pass.
}

#[test]
fn header_current_relocation_updates_logical_root_and_close_cascade() {
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
        Some(fixture.header.id())
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
        [
            "header:before:false",
            "child:before:false",
            "child:apply:false",
            "header:apply:false"
        ]
    );
    assert!(
        !child.open() && !fixture.header_open.get() && parent.open(),
        "current relocation participates in the Header close cascade"
    );
    fixture.order.borrow_mut().clear();
    dom_child.in_scope(child.scope(), || {
        assert!(fixture.header.request(true, Reason::Trigger, false));
    });
    present(&dom_child, &child);
    fixture.order.borrow_mut().clear();
    dom_parent.in_scope(parent.scope(), || {
        assert!(domain.dismiss_root(child.node().id(), Reason::Keyboard, false));
    });
    assert_eq!(
        *fixture.order.borrow(),
        [
            "header:before:false",
            "child:before:false",
            "child:apply:false",
            "header:apply:false"
        ]
    );
    assert!(!fixture.header_open.get() && !child.open() && parent.open());
}

#[test]
fn header_focus_owner_defers_parent_trap_and_excludes_overflow_across_vdom_owners() {
    let parent = Fixture::new("header-trap-parent", 705);
    let child = Fixture::new("header-trap-child", 706);
    let mut dom_parent = build_popover_fixture(&parent);
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
    let mounted = register_exit_mount(&dom_parent, &parent, "parent-before");
    let mut report = trap_report(&parent, "header focus owner document", 2, nodes);
    report["shift"] = false.into();
    report["active_hosts"] = serde_json::json!([fixture.header.id(), parent_id]);
    assert!(
        deliver_trap(&dom_parent, &parent, report.clone()).default_action_enabled(),
        "focused topmost Header owns its original Tab"
    );
    assert_eq!(parent.runtime().pending_item_focus_count(), 0);
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
    settle_exit(&mut dom_parent);
    assert_eq!(
        *mounted.calls.borrow(),
        [("parent-before".into(), parent.scope())]
    );
    assert!(fixture.header_open.get() && parent.open());
}

#[test]
fn header_child_visibility_events_honor_suppression_veto_and_unregistration() {
    let parent = Fixture::new("header-visibility-parent", 717);
    let child = Fixture::new("header-visibility-child", 718);
    let dom_parent = build_popover_fixture(&parent);
    let (dom_child, fixture) = build_header_exit_fixture(&child);
    let document = "header child visibility document";
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
        "topology updates do not invent visibility dismissal"
    );
    let node = child.node();
    dom_child.in_scope(child.scope(), || node.observe_anchor_visibility(true, None));
    node.set_trigger_visibility_check(false);
    dom_child.in_scope(child.scope(), || {
        node.observe_anchor_visibility(false, None)
    });
    assert_eq!(*fixture.order.borrow(), synchronous);
    node.set_trigger_visibility_check(true);
    dom_child.in_scope(child.scope(), || {
        node.observe_anchor_visibility(false, None)
    });
    let mut after = synchronous;
    after.push("child:before:false".into());
    assert_eq!(*fixture.order.borrow(), after);
    assert!(
        child.open(),
        "normal visibility dismissal honors the child's veto"
    );
    dom_child.in_scope(child.scope(), || node.unregister());
    dom_child.in_scope(child.scope(), || {
        node.observe_anchor_visibility(false, None)
    });
    assert_eq!(
        *fixture.order.borrow(),
        after,
        "unregistered owner rejects late visibility"
    );
}

fn anchor_geometry() -> AnchorIntersection {
    AnchorIntersection {
        bounding: AnchorRect {
            top: 0.0,
            bottom: 40.0,
            left: 0.0,
            right: 100.0,
        },
        intersection: AnchorRect {
            top: 10.0,
            bottom: 40.0,
            left: 0.0,
            right: 100.0,
        },
        anchor_in_dropdown_item: true,
    }
}

#[test]
fn positive_anchor_geometry_and_full_reset_enter_registered_owner_from_another_vdom() {
    let fixture = Fixture::new("anchor-geometry-owner", 719);
    let dom = build(&fixture);
    let received = Rc::new(RefCell::new(Vec::<AnchorIntersection>::new()));
    let capture = received.clone();
    let owner = fixture.0.borrow().owner.clone().unwrap();
    let scope = fixture.scope();
    fixture
        .node()
        .set_anchor_intersection_handler(Rc::new(move |geometry| {
            assert_owner(&owner, scope, 719);
            capture.borrow_mut().push(geometry);
        }));
    dom.in_scope(scope, || {
        assert!(fixture.node().request(true, Reason::Trigger, false));
    });
    fixture.0.borrow_mut().calls.clear();
    let caller = Fixture::new("anchor-geometry-caller", 720);
    let dom_caller = build(&caller);
    dom_caller.in_scope(caller.scope(), || {
        fixture
            .node()
            .observe_anchor_visibility(true, Some(anchor_geometry()))
    });
    assert!(fixture.open() && fixture.0.borrow().calls.is_empty());
    assert_eq!(received.borrow().len(), 1);
    assert_eq!(received.borrow()[0].intersection.top, 10.0);
    let mut full = anchor_geometry();
    full.intersection = full.bounding;
    full.anchor_in_dropdown_item = false;
    dom_caller.in_scope(caller.scope(), || {
        fixture.node().observe_anchor_visibility(true, Some(full))
    });
    assert_eq!(received.borrow().as_slice(), &[anchor_geometry(), full]);
    assert!(fixture.open() && fixture.0.borrow().calls.is_empty());
}

#[test]
fn anchor_visibility_rejects_invalid_closed_replaced_and_dropped_owners_and_honors_veto() {
    let fixture = Fixture::new("anchor-geometry-guards", 721);
    let dom = build(&fixture);
    let received = Rc::new(Cell::new(0));
    let capture = received.clone();
    fixture
        .node()
        .set_anchor_intersection_handler(Rc::new(move |_| capture.set(capture.get() + 1)));
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(true, Reason::Trigger, false));
    });
    fixture.0.borrow_mut().calls.clear();
    let mut inverted = anchor_geometry();
    inverted.bounding.top = 50.0;
    let mut nonfinite = anchor_geometry();
    nonfinite.intersection.left = f64::NAN;
    for geometry in [None, Some(inverted), Some(nonfinite)] {
        dom.in_scope(fixture.scope(), || {
            fixture.node().observe_anchor_visibility(true, geometry)
        });
    }
    assert_eq!(received.get(), 0);
    assert!(fixture.0.borrow().calls.is_empty());
    fixture.node().set_trigger_visibility_check(false);
    dom.in_scope(fixture.scope(), || {
        fixture
            .node()
            .observe_anchor_visibility(true, Some(anchor_geometry()))
    });
    assert_eq!(received.get(), 0);
    fixture.node().set_trigger_visibility_check(true);
    fixture.0.borrow().veto.set(true);
    dom.in_scope(fixture.scope(), || {
        fixture.node().observe_anchor_visibility(false, None)
    });
    assert!(fixture.open());
    assert_eq!(fixture.0.borrow().calls, [("before".into(), false)]);
    fixture.0.borrow().veto.set(false);
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(false, Reason::Trigger, false));
    });
    fixture.0.borrow_mut().calls.clear();
    dom.in_scope(fixture.scope(), || {
        fixture
            .node()
            .observe_anchor_visibility(true, Some(anchor_geometry()))
    });
    assert_eq!(received.get(), 0, "closed owners do not receive geometry");
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(true, Reason::Trigger, false));
    });
    let old = fixture.node();
    let runtime = fixture.runtime();
    let replacement = dom.in_scope(fixture.scope(), || {
        let callbacks = runtime.entry("anchor-geometry-guards").unwrap().callbacks;
        runtime.register(
            "anchor-geometry-guards".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        )
    });
    let capture = received.clone();
    replacement.set_anchor_intersection_handler(Rc::new(move |_| capture.set(capture.get() + 1)));
    dom.in_scope(fixture.scope(), || {
        old.observe_anchor_visibility(false, None);
        old.observe_anchor_visibility(true, Some(anchor_geometry()));
    });
    assert!(fixture.open());
    assert_eq!(
        received.get(),
        0,
        "old generation cannot mutate its replacement"
    );
    dom.in_scope(fixture.scope(), || {
        replacement.observe_anchor_visibility(true, Some(anchor_geometry()))
    });
    assert_eq!(received.get(), 1);
    fixture.0.borrow_mut().calls.clear();
    let owner = fixture.0.borrow().owner.clone().unwrap();
    drop(dom);
    assert!(owner.upgrade().is_none());
    let caller = Fixture::new("anchor-geometry-after-drop", 722);
    let dom_caller = build(&caller);
    dom_caller.in_scope(caller.scope(), || {
        replacement.observe_anchor_visibility(true, Some(anchor_geometry()))
    });
    assert_eq!(
        received.get(),
        1,
        "a dropped VDOM cannot receive geometry through retained handles"
    );
    assert!(fixture.0.borrow().calls.is_empty());
}

fn anchor_shape_report(fixture: &Fixture, sequence: u64) -> serde_json::Value {
    serde_json::json!({"owner":fixture.runtime().owner_id(),"document":"current anchor shape document",
        "sequence":sequence,"kind":"observe","nodes":[{"id":fixture.0.borrow().id,"generation":1,
        "trigger_id":"geometry-trigger","anchor_id":"geometry-anchor"}]})
}

#[test]
fn current_anchor_shape_is_independent_of_visibility_suppression_and_rejects_stale_owners() {
    let fixture = Fixture::new("anchor-shape-owner", 723);
    let dom = build(&fixture);
    let shapes = Rc::new(RefCell::new(Vec::new()));
    let capture = shapes.clone();
    let owner = fixture.0.borrow().owner.clone().unwrap();
    let scope = fixture.scope();
    fixture
        .node()
        .set_anchor_shape_handler(Rc::new(move |in_item| {
            assert_owner(&owner, scope, 723);
            capture.borrow_mut().push(in_item);
        }));
    let geometries = Rc::new(Cell::new(0));
    let capture = geometries.clone();
    fixture
        .node()
        .set_anchor_intersection_handler(Rc::new(move |_| capture.set(capture.get() + 1)));
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(true, Reason::Trigger, false))
    });
    fixture.0.borrow_mut().calls.clear();
    fixture.node().set_trigger_visibility_check(false);
    let mut suppressed = anchor_shape_report(&fixture, 1);
    suppressed["nodes"][0]["anchor_in_dropdown_item"] = true.into();
    assert!(deliver_trap(&dom, &fixture, suppressed).default_action_enabled());
    assert_eq!(
        *shapes.borrow(),
        [true],
        "initially suppressed visibility still reports current submenu anchor shape"
    );
    assert_eq!(
        geometries.get(),
        0,
        "shape admission does not fabricate positive geometry"
    );

    let mut current = anchor_shape_report(&fixture, 2);
    current["kind"] = "observe".into();
    current["nodes"][0]["anchor_id"] = "reparented-current-anchor".into();
    current["nodes"][0]["anchor_in_dropdown_item"] = false.into();
    assert!(deliver_trap(&dom, &fixture, current.clone()).default_action_enabled());
    assert_eq!(
        *shapes.borrow(),
        [true, false],
        "ordinary current-anchor observation resets a stale item relationship without an IO threshold crossing"
    );
    assert_eq!(geometries.get(), 0);
    assert!(fixture.open() && fixture.0.borrow().calls.is_empty());
    for (sequence, invalid) in [(3, "missing"), (4, "stale"), (5, "foreign-owner")] {
        let mut report = current.clone();
        report["sequence"] = sequence.into();
        match invalid {
            "missing" => {
                report["nodes"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("anchor_in_dropdown_item");
            }
            "stale" => report["nodes"][0]["generation"] = 2.into(),
            "foreign-owner" => report["owner"] = "wrong-owner".into(),
            _ => unreachable!(),
        }
        assert!(deliver_trap(&dom, &fixture, report).default_action_enabled());
        assert_eq!(
            *shapes.borrow(),
            [true, false],
            "{invalid} shape cannot replace current state"
        );
    }
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(false, Reason::Trigger, false))
    });
    fixture.0.borrow_mut().calls.clear();
    current["sequence"] = 6.into();
    assert!(deliver_trap(&dom, &fixture, current.clone()).default_action_enabled());
    assert_eq!(
        *shapes.borrow(),
        [true, false],
        "closed registration cannot receive shape commands"
    );
    dom.in_scope(fixture.scope(), || {
        assert!(fixture.node().request(true, Reason::Trigger, false))
    });
    fixture.0.borrow_mut().calls.clear();
    current["sequence"] = 7.into();
    let late = trap_form(current);
    let runtime = fixture.runtime();
    let owner = fixture.0.borrow().owner.clone().unwrap();
    drop(dom);
    assert!(owner.upgrade().is_none());
    let caller = Fixture::new("anchor-shape-after-drop", 724);
    let dom_caller = build(&caller);
    dom_caller.in_scope(caller.scope(), || runtime.input(late.clone()));
    assert!(late.default_action_enabled());
    assert_eq!(
        *shapes.borrow(),
        [true, false],
        "late shape cannot reach the dropped actual owner"
    );
    assert!(fixture.0.borrow().calls.is_empty());
}

fn anchor_focus_nodes(nodes: &[(&Fixture, &[&str])]) -> Vec<serde_json::Value> {
    nodes
        .iter()
        .map(|(fixture, ancestors)| {
            let id = fixture.0.borrow().id;
            set_fixture_ownership(id, ancestors);
            serde_json::json!({"id":id,"generation":fixture.node().generation(),
            "trigger_id":format!("{id}-trigger"),
            "mode":"roving-tabindex","on_trigger":true})
        })
        .collect()
}

fn anchor_focus_report(
    fixture: &Fixture,
    document: &str,
    sequence: u64,
    kind: &str,
    nodes: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({"owner":fixture.runtime().owner_id(),"document":document,
        "sequence":sequence,"kind":kind,"nodes":nodes})
}

fn prepare_anchor_focus(
    owners: &[(&VirtualDom, &Fixture)],
    document: &str,
    nodes: &[serde_json::Value],
) {
    for (dom, fixture) in owners {
        deliver_trap(
            dom,
            fixture,
            anchor_focus_report(fixture, document, 1, "observe", nodes),
        );
    }
    for (dom, fixture) in owners {
        dom.in_scope(fixture.scope(), || {
            assert!(fixture.node().request(true, Reason::Trigger, false))
        });
        fixture.0.borrow_mut().calls.clear();
        fixture.0.borrow_mut().applied.clear();
    }
    let (dom, fixture) = owners[0];
    let mut mode = document_report(fixture, document, 2, 1, "input");
    mode["document_event"]["input_kind"] = "keydown".into();
    mode["document_event"]["key"] = "Tab".into();
    deliver_trap(dom, fixture, mode);
}

fn negative_anchor_focus(
    dom: &VirtualDom,
    fixture: &Fixture,
    document: &str,
    sequence: u64,
    nodes: &[serde_json::Value],
) {
    let _ = (document, sequence, nodes);
    dom.in_scope(fixture.scope(), || {
        fixture.node().observe_anchor_visibility(false, None)
    });
}

fn release_anchor_tab(
    dom: &VirtualDom,
    fixture: &Fixture,
    document: &str,
    sequence: u64,
    nodes: &[serde_json::Value],
) -> FormEvent {
    let weak = Rc::downgrade(&fixture.runtime().0);
    let id = fixture.0.borrow().id;
    fixture
        .runtime()
        .0
        .borrow_mut()
        .nodes
        .get_mut(id)
        .unwrap()
        .callbacks
        .key = Some(Rc::new(move |_| {
        let runtime = RuntimeHandle(weak.upgrade().unwrap());
        runtime.close_and_release_focus(id);
        KeyDecision {
            release_focus: true,
            ..KeyDecision::default()
        }
    }));
    let mut report = anchor_focus_report(fixture, document, sequence, "key", nodes);
    report["key"] = "Tab".into();
    report["phase"] = "capture".into();
    report["triggers"] = serde_json::json!([id]);
    // No document/parent exit candidates: suppression must not depend on a target.
    deliver_trap(dom, fixture, report)
}

fn trigger_suppressed(fixture: &Fixture) -> bool {
    fixture
        .runtime()
        .entry(fixture.0.borrow().id)
        .unwrap()
        .suppress_trigger_restore
}

#[test]
fn vetoed_tab_marks_cached_owners_before_reentrant_before_and_later_hide_consumes_once() {
    let root = Fixture::new("anchor-focus-root", 725);
    let child = Fixture::new("anchor-focus-child", 726);
    let dom_root = build(&root);
    let dom_child = build(&child);
    let document = "anchor focus root-veto document";
    let nodes = anchor_focus_nodes(&[(&root, &[]), (&child, &["anchor-focus-root"])]);
    prepare_anchor_focus(
        &[(&dom_root, &root), (&dom_child, &child)],
        document,
        &nodes,
    );
    root.0.borrow().veto.set(true);
    let runtime = root.runtime();
    let old_before = runtime.entry("anchor-focus-root").unwrap().callbacks.before;
    let callback_before = old_before.clone();
    let root_weak = Rc::downgrade(&runtime.0);
    let child_weak = Rc::downgrade(&child.runtime().0);
    let reentrant_nodes = nodes.clone();
    let checked = Rc::new(Cell::new(0));
    let observed = checked.clone();
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("anchor-focus-root")
        .unwrap()
        .callbacks
        .before = Rc::new(move |change| {
        if !change.open {
            let runtime = RuntimeHandle(root_weak.upgrade().unwrap());
            assert!(
                runtime
                    .entry("anchor-focus-root")
                    .unwrap()
                    .suppress_trigger_restore
            );
            assert!(
                RuntimeHandle(child_weak.upgrade().unwrap())
                    .entry("anchor-focus-child")
                    .unwrap()
                    .suppress_trigger_restore
            );
            if observed.replace(observed.get() + 1) == 0 {
                runtime.input(trap_form(
                    serde_json::json!({"owner":runtime.owner_id(),"document":document,
                    "sequence":3,"kind":"observe","nodes":reentrant_nodes}),
                ));
            }
        }
        callback_before(change)
    });
    let event = release_anchor_tab(&dom_child, &child, document, 2, &nodes);
    assert!(
        event.default_action_enabled(),
        "undefined exit target does not prevent Tab"
    );
    assert_eq!(checked.get(), 1);
    assert!(root.open() && child.open() && trigger_suppressed(&root) && trigger_suppressed(&child));
    assert_eq!(
        child.runtime().pending_item_focus_count(),
        0,
        "undefined exit did not admit mounted focus"
    );
    root.0.borrow().veto.set(false);
    negative_anchor_focus(&dom_root, &root, document, 4, &nodes);
    assert!(!root.open() && !child.open());
    assert!(!trigger_suppressed(&root) && !trigger_suppressed(&child));
    assert!(!root.0.borrow().applied.last().unwrap().restore_focus);
    assert!(!child.0.borrow().applied.last().unwrap().restore_focus);
    // Remove the observation-only wrapper before the next ordinary lifecycle.
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("anchor-focus-root")
        .unwrap()
        .callbacks
        .before = old_before;
    for (dom, fixture) in [(&dom_root, &root), (&dom_child, &child)] {
        dom.in_scope(fixture.scope(), || {
            assert!(fixture.node().request(true, Reason::Trigger, false))
        });
    }
    negative_anchor_focus(&dom_root, &root, document, 5, &nodes);
    assert!(root.0.borrow().applied.last().unwrap().restore_focus);
    assert!(child.0.borrow().applied.last().unwrap().restore_focus);
    assert_eq!(
        child.0.borrow().applied.last().unwrap().reason,
        Reason::AnchorVisibilityParent
    );
}

#[test]
fn child_veto_keeps_current_parent_relation_and_consumes_latch_on_own_hide() {
    let root = Fixture::new("anchor-veto-root", 727);
    let child = Fixture::new("anchor-veto-child", 728);
    let dom_root = build(&root);
    let dom_child = build(&child);
    let document = "anchor focus child-veto document";
    let nodes = anchor_focus_nodes(&[(&root, &[]), (&child, &["anchor-veto-root"])]);
    prepare_anchor_focus(
        &[(&dom_root, &root), (&dom_child, &child)],
        document,
        &nodes,
    );
    child.0.borrow().veto.set(true);
    release_anchor_tab(&dom_child, &child, document, 2, &nodes);
    assert!(!root.open() && child.open());
    assert!(!trigger_suppressed(&root) && trigger_suppressed(&child));
    assert_eq!(
        root.runtime()
            .domain()
            .unwrap()
            .cached_children("anchor-veto-root"),
        [child.node().id()],
        "visibility veto leaves the current child relationship intact"
    );
    child.0.borrow().veto.set(false);
    negative_anchor_focus(&dom_child, &child, document, 3, &nodes);
    assert!(!child.open() && !trigger_suppressed(&child));
    assert!(!child.0.borrow().applied.last().unwrap().restore_focus);
    dom_child.in_scope(child.scope(), || {
        assert!(child.node().request(true, Reason::Trigger, false))
    });
    negative_anchor_focus(&dom_child, &child, document, 4, &nodes);
    assert!(child.0.borrow().applied.last().unwrap().restore_focus);
    assert!(
        !root.open(),
        "negative child fact never reopens or dismisses its cached parent"
    );
}

#[test]
fn suppression_stops_at_closed_logical_intermediate_and_pointer_hide_consumes_registration_state() {
    let root = Fixture::new("anchor-closed-root", 729);
    let middle = Fixture::new("anchor-closed-middle", 730);
    let leaf = Fixture::new("anchor-closed-leaf", 731);
    let dom_root = build(&root);
    let dom_middle = build(&middle);
    let dom_leaf = build(&leaf);
    let document = "anchor focus closed-intermediate document";
    let nodes = anchor_focus_nodes(&[
        (&root, &[]),
        (&middle, &["anchor-closed-root"]),
        (&leaf, &["anchor-closed-middle", "anchor-closed-root"]),
    ]);
    prepare_anchor_focus(
        &[
            (&dom_root, &root),
            (&dom_middle, &middle),
            (&dom_leaf, &leaf),
        ],
        document,
        &nodes,
    );
    dom_middle.in_scope(middle.scope(), || {
        let mut open = middle.0.borrow().open.unwrap();
        open.set(false);
        middle.node().sync_open(false);
    });
    root.0.borrow().veto.set(true);
    dom_root.in_scope(root.scope(), || {
        assert!(!root.runtime().close_and_release_focus("anchor-closed-root"))
    });
    assert!(trigger_suppressed(&root));
    assert!(!trigger_suppressed(&middle) && !trigger_suppressed(&leaf));
    dom_root.in_scope(root.scope(), || {
        root.runtime().join_node("anchor-closed-root")
    });
    assert!(
        trigger_suppressed(&root),
        "same-registration domain metadata rejoin retains latch"
    );
    let mut pointer = document_report(&root, document, 3, 2, "input");
    pointer["document_event"]["input_kind"] = "mousedown".into();
    deliver_trap(&dom_root, &root, pointer);
    root.0.borrow().veto.set(false);
    negative_anchor_focus(&dom_root, &root, document, 4, &nodes);
    assert!(!trigger_suppressed(&root));
    assert!(!root.0.borrow().applied.last().unwrap().restore_focus);
    dom_root.in_scope(root.scope(), || {
        assert!(root.node().request(true, Reason::Trigger, false))
    });
    let mut keyboard = document_report(&root, document, 5, 3, "input");
    keyboard["document_event"]["input_kind"] = "keydown".into();
    keyboard["document_event"]["key"] = "Tab".into();
    deliver_trap(&dom_root, &root, keyboard);
    negative_anchor_focus(&dom_root, &root, document, 6, &nodes);
    assert!(root.0.borrow().applied.last().unwrap().restore_focus);
}

#[test]
fn ordinary_negative_cleanup_reads_modality_after_child_before_callbacks() {
    let root = Fixture::new("anchor-mode-root", 734);
    let child = Fixture::new("anchor-mode-child", 735);
    let dom_root = build(&root);
    let dom_child = build(&child);
    let document = "anchor focus commit modality document";
    let nodes = anchor_focus_nodes(&[(&root, &[]), (&child, &["anchor-mode-root"])]);
    prepare_anchor_focus(
        &[(&dom_root, &root), (&dom_child, &child)],
        document,
        &nodes,
    );
    let runtime = child.runtime();
    let before = runtime.entry("anchor-mode-child").unwrap().callbacks.before;
    let weak = Rc::downgrade(&runtime.0);
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("anchor-mode-child")
        .unwrap()
        .callbacks
        .before = Rc::new(move |change| {
        let allowed = before(change);
        if !change.open {
            let runtime = RuntimeHandle(weak.upgrade().unwrap());
            runtime.input(trap_form(
                serde_json::json!({"owner":runtime.owner_id(),"document":document,
                "sequence":2,"kind":"document","document_event":{"sequence":2,"action":"input","input_kind":"mousedown"}}),
            ));
        }
        allowed
    });
    negative_anchor_focus(&dom_root, &root, document, 3, &nodes);
    assert!(!root.open() && !child.open());
    assert!(
        !child.0.borrow().applied.last().unwrap().restore_focus,
        "child cleanup sees its before callback's pointer modality"
    );
    assert!(
        !root.0.borrow().applied.last().unwrap().restore_focus,
        "later parent cleanup reads current mode rather than a captured pre-cascade value"
    );
}

#[test]
fn dropdown_suppression_does_not_mark_or_change_cross_type_popover_restoration() {
    let root = Fixture::new("anchor-cross-root", 736);
    let popover = Fixture::new("anchor-cross-popover", 737);
    let dom_root = build(&root);
    let dom_popover = build_popover_fixture(&popover);
    let document = "anchor focus cross-type document";
    let nodes = anchor_focus_nodes(&[(&root, &[]), (&popover, &["anchor-cross-root"])]);
    prepare_anchor_focus(
        &[(&dom_root, &root), (&dom_popover, &popover)],
        document,
        &nodes,
    );
    root.0.borrow().veto.set(true);
    dom_root.in_scope(root.scope(), || {
        assert!(!root.runtime().close_and_release_focus("anchor-cross-root"))
    });
    assert!(trigger_suppressed(&root));
    assert!(!trigger_suppressed(&popover));
    assert!(popover.open());
    root.0.borrow().veto.set(false);
    negative_anchor_focus(&dom_root, &root, document, 3, &nodes);
    assert!(!root.open() && !popover.open());
    assert!(!root.0.borrow().applied.last().unwrap().restore_focus);
    assert!(
        popover.0.borrow().applied.last().unwrap().restore_focus,
        "Popover keeps its existing own-trigger cross-type parent-close policy"
    );
}

#[test]
fn linked_roving_trigger_owns_tab_before_enclosing_active_descendant_host() {
    let child = Fixture::new("linked-roving-trigger", 739);
    let (mut dom, fixture) = build_header_exit_fixture(&child);
    let runtime = child.runtime();
    let root = dom.in_scope(child.scope(), || {
        let callbacks = runtime.entry(fixture.header.id()).unwrap().callbacks;
        let root = runtime.register(
            fixture.header.id().into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        let mut callbacks = runtime.entry(child.node().id()).unwrap().callbacks;
        let weak = Rc::downgrade(&runtime.0);
        let id = child.node().id().to_string();
        callbacks.key = Some(Rc::new(move |facts| {
            if facts.key != "Tab" || facts.default_prevented {
                return KeyDecision::default();
            }
            RuntimeHandle(weak.upgrade().unwrap()).close_and_release_focus(&id);
            KeyDecision {
                release_focus: true,
                ..KeyDecision::default()
            }
        }));
        let node = runtime.register(
            child.node().id().into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        child.0.borrow_mut().node = Some(node);
        root
    });
    let document = "linked roving trigger root host document";
    attach(&dom, &child, document);
    root.set_ownership(Vec::new(), true);
    root.set_mounted();
    child.node().set_ownership(vec![root.clone()], true);
    child.node().set_mounted();
    let nodes = serde_json::json!([
        {"id":root.id(),"generation":1,
         "mode":"active-descendant","trigger_id":"linked-root-trigger","inside":true},
        {"id":child.node().id(),"generation":1,
         "mode":"roving-tabindex",
         "trigger_id":"linked-child-trigger","on_trigger":true}
    ]);
    deliver_trap(
        &dom,
        &child,
        serde_json::json!({
            "owner":runtime.owner_id(),"document":document,"sequence":1,"kind":"observe","nodes":nodes
        }),
    );
    dom.in_scope(child.scope(), || {
        assert!(root.request(true, Reason::Trigger, false));
        assert!(child.node().request(true, Reason::Trigger, false));
    });
    assert_eq!(
        runtime.domain().unwrap().cached_root(child.node().id()),
        Some(root.id().into())
    );
    fixture.order.borrow_mut().clear();
    let mounted = register_exit_mount(&dom, &child, "linked-root-trigger");
    for (sequence, root_veto, child_veto) in [(2, true, false), (3, false, true)] {
        fixture.header_veto.set(root_veto);
        child.0.borrow().veto.set(child_veto);
        let event = deliver_trap(
            &dom,
            &child,
            serde_json::json!({
                "owner":runtime.owner_id(),"document":document,"sequence":sequence,
                "kind":"key","phase":"capture","key":"Tab","shift":true,
                "hosts":[root.id()],"triggers":[child.node().id()],"nodes":nodes
            }),
        );
        assert!(
            !event.default_action_enabled(),
            "the actual child trigger mode admits the captured cached-root target synchronously"
        );
        assert_eq!(fixture.header_open.get(), root_veto);
        assert!(child.open());
        settle_exit(&mut dom);
        assert_eq!(
            mounted.calls.borrow().len(),
            (sequence - 1) as usize,
            "each current Tab focuses the registered root trigger"
        );
    }
    assert_eq!(
        *fixture.order.borrow(),
        [
            "header:before:false",
            "header:before:false",
            "child:before:false",
            "header:apply:false"
        ]
    );
}

type OthersJournal = Rc<RefCell<Vec<(String, &'static str, Change)>>>;

#[test]
fn direct_inside_click_rejects_replaced_and_dropped_real_owners() {
    let fixture = Fixture::new("inside-click-lifetime", 763);
    let dom = build(&fixture);
    present(&dom, &fixture);
    let runtime = fixture.runtime();
    let old = fixture.node();
    let current = dom.in_scope(fixture.scope(), || {
        let callbacks = runtime.entry(old.id()).unwrap().callbacks;
        runtime.register(
            old.id().into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        )
    });
    let stale = super::tests::typed_mouse();
    old.dioxus_inside_click(&stale);
    assert!(stale.default_action_enabled() && stale.propagates());
    assert!(fixture.open() && fixture.0.borrow().calls.is_empty());
    let prevented = super::tests::typed_mouse();
    prevented.prevent_default();
    current.dioxus_inside_click(&prevented);
    assert!(!prevented.default_action_enabled() && !prevented.propagates());
    assert!(fixture.open() && fixture.0.borrow().calls.is_empty());
    fixture.0.borrow().veto.set(true);
    let vetoed = super::tests::typed_mouse();
    current.dioxus_inside_click(&vetoed);
    assert!(vetoed.default_action_enabled() && !vetoed.propagates());
    assert_eq!(fixture.0.borrow().calls, [("before".into(), false)]);
    assert!(fixture.open());
    fixture.0.borrow_mut().calls.clear();
    drop(dom);
    let dropped = super::tests::typed_mouse();
    current.dioxus_inside_click(&dropped);
    assert!(dropped.default_action_enabled() && dropped.propagates());
    assert!(
        fixture.0.borrow().calls.is_empty(),
        "retaining a runtime handle cannot retain its renderer owner"
    );
}

#[test]
fn standalone_trigger_others_cascade_consumes_veto_retained_latches_then_restores_keyboard() {
    let child = Fixture::new("others-standalone-child", 762);
    let (dom, fixture) = build_header_exit_fixture(&child);
    let runtime = child.runtime();
    let grand_open = Rc::new(Cell::new(false));
    let grand_changes = Rc::new(RefCell::new(Vec::new()));
    let (root, grand) = dom.in_scope(child.scope(), || {
        let mut child_callbacks = runtime.entry(child.node().id()).unwrap().callbacks;
        child_callbacks.key = Some(Rc::new(|_| KeyDecision::default()));
        let child_node = runtime.register(
            child.node().id().into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            child_callbacks,
        );
        child.0.borrow_mut().node = Some(child_node);
        let callbacks = runtime.entry(fixture.header.id()).unwrap().callbacks;
        let root = runtime.register(
            fixture.header.id().into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        let owner = child.0.borrow().owner.clone().unwrap();
        let scope = child.scope();
        let read_owner = owner.clone();
        let before_owner = owner.clone();
        let read = grand_open.clone();
        let apply = grand_open.clone();
        let applied = grand_changes.clone();
        let grand = runtime.register(
            "others-standalone-grandchild".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Inside,
            Callbacks {
                read: Rc::new(move || {
                    assert_owner(&read_owner, scope, 762);
                    Some(read.get())
                }),
                before: Rc::new(move |_| {
                    assert_owner(&before_owner, scope, 762);
                    true
                }),
                apply: Rc::new(move |change| {
                    assert_owner(&owner, scope, 762);
                    applied.borrow_mut().push(change);
                    apply.set(change.open);
                }),
                key: None,
            },
        );
        (root, grand)
    });
    root.set_ownership(Vec::new(), true);
    root.set_mounted();
    child.node().set_ownership(vec![root.clone()], true);
    child.node().set_mounted();
    grand.set_ownership(vec![child.node(), root.clone()], true);
    grand.set_mounted();
    let nodes = serde_json::json!([
        {"id":root.id(),"generation":1},
        {"id":child.node().id(),"generation":1,"mode":"active-descendant","on_trigger":true},
        {"id":grand.id(),"generation":1}
    ]);
    deliver_trap(
        &dom,
        &child,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":1,
        "kind":"observe","nodes":nodes}),
    );
    assert!(runtime.domain().is_none());
    dom.in_scope(child.scope(), || {
        assert!(root.request(true, Reason::Trigger, false));
        assert!(child.node().request(true, Reason::Trigger, false));
        assert!(grand.request(true, Reason::Trigger, false));
    });
    deliver_trap(
        &dom,
        &child,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":2,
        "kind":"key","phase":"bubble","input_kind":"keydown","key":"ArrowDown",
        "hosts":[root.id()],"triggers":[child.node().id()],"nodes":nodes}),
    );
    assert!(
        runtime.keyboard_mode(),
        "registered child key admission primes the standalone mode"
    );
    fixture.header_veto.set(true);
    dom.in_scope(child.scope(), || {
        assert!(!runtime.close_and_release_focus(child.node().id()))
    });
    assert!(runtime.entry(root.id()).unwrap().suppress_trigger_restore);
    assert!(trigger_suppressed(&child));
    assert!(runtime.entry(grand.id()).unwrap().suppress_trigger_restore);
    child.0.borrow().veto.set(true);
    dom.in_scope(child.scope(), || {
        assert!(runtime.dismiss_dropdown_others(root.id(), 1))
    });
    assert!(child.open() && grand_open.get());
    assert!(trigger_suppressed(&child));
    child.0.borrow().veto.set(false);
    grand_changes.borrow_mut().clear();
    child.0.borrow_mut().applied.clear();
    dom.in_scope(child.scope(), || {
        assert!(runtime.dismiss_dropdown_others(root.id(), 1))
    });
    assert!(!child.open() && !grand_open.get());
    assert!(!trigger_suppressed(&child));
    assert!(!runtime.entry(grand.id()).unwrap().suppress_trigger_restore);
    assert!(runtime.entry(root.id()).unwrap().suppress_trigger_restore);
    assert_eq!(grand_changes.borrow().len(), 1);
    assert!(
        grand_changes
            .borrow()
            .iter()
            .all(|change| change.reason == Reason::TriggerOther && !change.restore_focus)
    );
    assert!(!child.0.borrow().applied.last().unwrap().restore_focus);
    dom.in_scope(child.scope(), || {
        assert!(child.node().request(true, Reason::Trigger, false));
        assert!(grand.request(true, Reason::Trigger, false));
    });
    grand_changes.borrow_mut().clear();
    dom.in_scope(child.scope(), || {
        assert!(runtime.dismiss_dropdown_others(root.id(), 1))
    });
    assert!(!child.open() && !grand_open.get() && fixture.header_open.get());
    assert_eq!(grand_changes.borrow().len(), 1);
    assert!(
        grand_changes
            .borrow()
            .iter()
            .all(|change| change.reason == Reason::TriggerOther && change.restore_focus)
    );
    let applied = child.0.borrow().applied.last().copied().unwrap();
    assert_eq!(applied.reason, Reason::TriggerOther);
    assert!(applied.restore_focus);
}

#[test]
fn trigger_others_captures_exclusions_but_cascades_current_reentrant_parent_and_new_registration() {
    let active = Fixture::new("others-mut-active", 752);
    let passed = Fixture::new("others-mut-passed", 753);
    let first = Fixture::new("others-mut-first", 754);
    let later = Fixture::new("others-mut-later", 755);
    let new_parent = Fixture::new("others-mut-new-parent", 756);
    let dom_active = build(&active);
    let dom_passed = build(&passed);
    let dom_first = build(&first);
    let dom_later = build(&later);
    let dom_parent = build(&new_parent);
    let owners = [
        (&dom_active, &active),
        (&dom_passed, &passed),
        (&dom_first, &first),
        (&dom_later, &later),
        (&dom_parent, &new_parent),
    ];
    let nodes = anchor_focus_nodes(&[
        (&active, &[]),
        (&passed, &[]),
        (&first, &[]),
        (&later, &[]),
        (&new_parent, &[]),
    ]);
    let document = "trigger others dynamic callbacks document";
    prepare_others(&owners, document, &nodes);
    for (dom, fixture) in &owners {
        present(dom, fixture);
    }
    passed.node().set_close_behavior(CloseBehavior::Inside);
    first.0.borrow().veto.set(true);
    let journal = Rc::new(RefCell::new(Vec::new()));
    for (_, fixture) in &owners {
        track_others(fixture, &journal);
    }
    let runtime = first.runtime();
    let before = runtime.entry(first.node().id()).unwrap().callbacks.before;
    let weak = Rc::downgrade(&runtime.0);
    let passed_node = passed.node();
    let later_node = later.node();
    let active_node = active.node();
    let active_runtime = Rc::downgrade(&active.runtime().0);
    let owner = first.0.borrow().owner.clone().unwrap();
    let scope = first.scope();
    let dynamic_open = Rc::new(Cell::new(true));
    let dynamic_result = dynamic_open.clone();
    let dynamic_node = Rc::new(RefCell::new(None));
    let retained_dynamic_node = dynamic_node.clone();
    let dynamic_journal = journal.clone();
    let once = Cell::new(false);
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut(first.node().id())
        .unwrap()
        .callbacks
        .before = Rc::new(move |change| {
        let allowed = before(change);
        if !change.open && !once.replace(true) {
            passed_node.set_close_behavior(CloseBehavior::Both);
            later_node.set_close_behavior(CloseBehavior::Inside);
            active_node.set_ownership(vec![fixture_node("others-mut-new-parent")], true);
            let relocated = nodes.clone();
            active_node
                .with_owner(|| {
                    let active_runtime = RuntimeHandle(active_runtime.upgrade().unwrap());
                    active_runtime.input(trap_form(serde_json::json!({
                        "owner":active_runtime.owner_id(),"document":document,"sequence":2,
                        "kind":"observe","nodes":relocated
                    })));
                })
                .unwrap();
            let runtime = RuntimeHandle(weak.upgrade().unwrap());
            let read = dynamic_open.clone();
            let apply = dynamic_open.clone();
            let read_owner = owner.clone();
            let before_owner = owner.clone();
            let apply_owner = owner.clone();
            let before_log = dynamic_journal.clone();
            let apply_log = dynamic_journal.clone();
            let node = runtime.register(
                "others-mut-appended".into(),
                1,
                OverlayKind::Dropdown,
                CloseBehavior::Both,
                Callbacks {
                    read: Rc::new(move || {
                        assert_owner(&read_owner, scope, 754);
                        Some(read.get())
                    }),
                    before: Rc::new(move |change| {
                        assert_owner(&before_owner, scope, 754);
                        before_log.borrow_mut().push((
                            "others-mut-appended".into(),
                            "before",
                            change,
                        ));
                        true
                    }),
                    apply: Rc::new(move |change| {
                        assert_owner(&apply_owner, scope, 754);
                        apply_log.borrow_mut().push((
                            "others-mut-appended".into(),
                            "apply",
                            change,
                        ));
                        apply.set(change.open);
                    }),
                    key: None,
                },
            );
            node.set_ownership(Vec::new(), true);
            node.set_mounted();
            *dynamic_node.borrow_mut() = Some(node);
            assert_owner(&owner, scope, 754);
        }
        allowed
    });
    dom_active.in_scope(active.scope(), || {
        assert!(
            active
                .runtime()
                .dismiss_dropdown_others(active.node().id(), 1)
        );
    });
    assert_eq!(
        others_before_ids(&journal),
        [
            "others-mut-first",
            "others-mut-new-parent",
            "others-mut-active",
            "others-mut-appended"
        ]
    );
    assert!(
        !active.open() && passed.open() && first.open() && later.open(),
        "the active control moved beneath a dismissed parent during a callback, so the current child cascade includes it"
    );
    assert!(!new_parent.open() && !dynamic_result.get());
    assert!(
        retained_dynamic_node
            .borrow()
            .as_ref()
            .unwrap()
            .is_registered()
    );
}

#[test]
fn trigger_others_skips_dropped_owner_and_visits_replacement_at_new_connection_position() {
    let active = Fixture::new("others-life-active", 757);
    let first = Fixture::new("others-life-first", 758);
    let dropped = Fixture::new("others-life-drop", 759);
    let replaced = Fixture::new("others-life-replaced", 760);
    let later = Fixture::new("others-life-later", 761);
    let dom_active = build(&active);
    let dom_first = build(&first);
    let dom_dropped = Rc::new(RefCell::new(Some(build(&dropped))));
    let dom_replaced = build(&replaced);
    let dom_later = build(&later);
    let nodes = anchor_focus_nodes(&[
        (&active, &[]),
        (&first, &[]),
        (&dropped, &[]),
        (&replaced, &[]),
        (&later, &[]),
    ]);
    let document = "trigger others owner replacement document";
    {
        let dropped_dom = dom_dropped.borrow();
        let owners = [
            (&dom_active, &active),
            (&dom_first, &first),
            (dropped_dom.as_ref().unwrap(), &dropped),
            (&dom_replaced, &replaced),
            (&dom_later, &later),
        ];
        prepare_others(&owners, document, &nodes);
        for (dom, fixture) in &owners {
            present(dom, fixture);
        }
    }
    let journal = Rc::new(RefCell::new(Vec::new()));
    for fixture in [&active, &first, &dropped, &replaced, &later] {
        track_others(fixture, &journal);
    }
    first.0.borrow().veto.set(true);
    let runtime = first.runtime();
    let before = runtime.entry(first.node().id()).unwrap().callbacks.before;
    let old_replaced = replaced.node();
    let replacement_slot = Rc::downgrade(&replaced.0);
    let dropped_slot = dom_dropped.clone();
    let old_for_callback = old_replaced.clone();
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut(first.node().id())
        .unwrap()
        .callbacks
        .before = Rc::new(move |change| {
        let allowed = before(change);
        if !change.open {
            drop(dropped_slot.borrow_mut().take());
            old_for_callback
                .with_owner(|| {
                    let runtime = old_for_callback.runtime().unwrap();
                    let callbacks = runtime.entry(old_for_callback.id()).unwrap().callbacks;
                    let node = runtime.register(
                        old_for_callback.id().into(),
                        2,
                        OverlayKind::Dropdown,
                        CloseBehavior::Both,
                        callbacks,
                    );
                    node.set_ownership(Vec::new(), true);
                    node.set_mounted();
                    replacement_slot.upgrade().unwrap().borrow_mut().node = Some(node);
                })
                .unwrap();
        }
        allowed
    });
    dom_active.in_scope(active.scope(), || {
        assert!(
            active
                .runtime()
                .dismiss_dropdown_others(active.node().id(), 1)
        );
        assert_eq!(consume_context::<OwnerToken>().0, 757);
    });
    assert_eq!(
        others_before_ids(&journal),
        [
            "others-life-first",
            "others-life-later",
            "others-life-replaced"
        ]
    );
    assert!(first.open());
    assert!(!later.open() && !replaced.open());
    assert_eq!(replaced.node().generation(), 2);
    assert!(
        dropped
            .0
            .borrow()
            .owner
            .as_ref()
            .unwrap()
            .upgrade()
            .is_none()
    );
    assert!(!old_replaced.request(false, Reason::Trigger, false));
}

#[test]
fn trigger_others_preserves_current_ancestors_and_closes_former_parent_sibling_and_descendant() {
    let old = Fixture::new("others-old-parent", 746);
    let active = Fixture::new("others-relocated", 747);
    let sibling = Fixture::new("others-sibling", 748);
    let descendant = Fixture::new("others-descendant", 749);
    let live = Fixture::new("others-live-parent", 750);
    let middle = Fixture::new("others-middle-popover", 751);
    let dom_old = build(&old);
    let dom_active = build(&active);
    let dom_sibling = build(&sibling);
    let dom_descendant = build(&descendant);
    let dom_live = build(&live);
    let dom_middle = build_popover_fixture(&middle);
    let owners = [
        (&dom_old, &old),
        (&dom_active, &active),
        (&dom_sibling, &sibling),
        (&dom_descendant, &descendant),
        (&dom_live, &live),
        (&dom_middle, &middle),
    ];
    let nodes = anchor_focus_nodes(&[
        (&old, &[]),
        (&active, &["others-old-parent"]),
        (&sibling, &["others-old-parent"]),
        (&descendant, &["others-relocated", "others-old-parent"]),
        (&live, &[]),
        (&middle, &["others-live-parent"]),
    ]);
    let document = "trigger others cached and live ancestry document";
    prepare_others(&owners, document, &nodes);
    for (dom, fixture) in &owners {
        present(dom, fixture);
    }
    let domain = active.runtime().domain().unwrap();
    assert_eq!(
        domain.cached_parent(active.node().id()).as_deref(),
        Some(old.node().id())
    );
    assert_eq!(
        domain.cached_children(active.node().id()),
        [descendant.node().id()]
    );
    active
        .node()
        .set_ownership(vec![middle.node(), live.node()], true);
    deliver_trap(
        &dom_active,
        &active,
        anchor_focus_report(&active, document, 2, "observe", &nodes),
    );
    assert_eq!(
        domain.cached_parent(active.node().id()).as_deref(),
        Some(live.node().id())
    );
    assert_eq!(
        domain.live_ancestors(active.node().id()),
        [middle.node().id(), live.node().id()]
    );
    let journal = Rc::new(RefCell::new(Vec::new()));
    for (_, fixture) in &owners {
        track_others(fixture, &journal);
    }
    active.0.borrow().veto.set(true);
    dom_active.in_scope(active.scope(), || {
        assert!(!active.node().request(false, Reason::Trigger, false))
    });
    journal.borrow_mut().clear();
    dom_active.in_scope(active.scope(), || {
        assert!(
            active
                .runtime()
                .dismiss_dropdown_others(active.node().id(), active.node().generation())
        );
    });
    assert_eq!(
        others_before_ids(&journal),
        [old.node().id(), sibling.node().id(), descendant.node().id()]
    );
    assert!(!old.open() && active.open() && live.open() && middle.open());
    assert!(!sibling.open() && !descendant.open());
}

fn track_others(fixture: &Fixture, journal: &OthersJournal) {
    let runtime = fixture.runtime();
    let id = fixture.0.borrow().id;
    let mut state = runtime.0.borrow_mut();
    let callbacks = &mut state.nodes.get_mut(id).unwrap().callbacks;
    let before = callbacks.before.clone();
    let before_journal = journal.clone();
    callbacks.before = Rc::new(move |change| {
        before_journal
            .borrow_mut()
            .push((id.into(), "before", change));
        before(change)
    });
    let apply = callbacks.apply.clone();
    let apply_journal = journal.clone();
    callbacks.apply = Rc::new(move |change| {
        apply_journal
            .borrow_mut()
            .push((id.into(), "apply", change));
        apply(change);
    });
}

fn prepare_others(owners: &[(&VirtualDom, &Fixture)], document: &str, nodes: &[serde_json::Value]) {
    for (dom, fixture) in owners {
        attach(dom, fixture, document);
        deliver_trap(
            dom,
            fixture,
            anchor_focus_report(fixture, document, 1, "observe", nodes),
        );
    }
}

fn others_before_ids(journal: &OthersJournal) -> Vec<String> {
    journal
        .borrow()
        .iter()
        .filter(|(_, phase, change)| *phase == "before" && !change.open)
        .map(|(id, _, _)| id.clone())
        .collect()
}

#[test]
fn trigger_others_orders_owner_callbacks_and_policies_after_veto_or_closed_origin() {
    let fixtures = [
        Fixture::new("others-active", 740),
        Fixture::new("others-both", 741),
        Fixture::new("others-inside", 742),
        Fixture::new("others-outside", 743),
        Fixture::new("others-none", 744),
        Fixture::new("others-closed", 745),
    ];
    let doms = fixtures.iter().map(build).collect::<Vec<_>>();
    let owners = doms.iter().zip(&fixtures).collect::<Vec<_>>();
    let nodes = anchor_focus_nodes(
        &fixtures
            .iter()
            .map(|fixture| (fixture, &[][..]))
            .collect::<Vec<_>>(),
    );
    let document = "trigger others ordered independent owner document";
    prepare_others(&owners, document, &nodes);
    doms[3].in_scope(fixtures[3].scope(), || {
        let runtime = fixtures[3].runtime();
        let id = fixtures[3].node().id().to_owned();
        let callbacks = runtime.entry(&id).unwrap().callbacks;
        let node = runtime.register(
            id,
            1,
            OverlayKind::Header,
            CloseBehavior::Outside,
            callbacks,
        );
        fixtures[3].0.borrow_mut().node = Some(node);
    });
    for (fixture, behavior) in [
        (&fixtures[2], CloseBehavior::Inside),
        (&fixtures[3], CloseBehavior::Outside),
        (&fixtures[4], CloseBehavior::None),
    ] {
        fixture.node().set_close_behavior(behavior);
    }
    let journal = Rc::new(RefCell::new(Vec::new()));
    for (dom, fixture) in owners.iter().take(5) {
        present(dom, fixture);
    }
    for fixture in &fixtures {
        track_others(fixture, &journal);
    }
    let active = &fixtures[0];
    let active_dom = &doms[0];
    let runtime = active.runtime();
    let mut pointer = document_report(active, document, 2, 1, "input");
    pointer["document_event"]["input_kind"] = "mousedown".into();
    deliver_trap(active_dom, active, pointer);
    active.0.borrow().veto.set(true);
    fixtures[1].0.borrow().veto.set(true);
    active_dom.in_scope(active.scope(), || {
        assert!(!active.node().request(false, Reason::Trigger, false))
    });
    journal.borrow_mut().clear();
    let closed_revision = fixtures[5]
        .runtime()
        .entry(fixtures[5].node().id())
        .unwrap()
        .revision;
    active_dom.in_scope(active.scope(), || {
        assert!(runtime.dismiss_dropdown_others(active.node().id(), active.node().generation()));
        assert_eq!(consume_context::<OwnerToken>().0, 740);
    });
    assert_eq!(
        others_before_ids(&journal),
        ["others-both", "others-outside"]
    );
    assert!(active.open() && fixtures[1].open() && fixtures[2].open() && fixtures[4].open());
    assert!(!fixtures[3].open() && !fixtures[5].open());
    assert_eq!(
        fixtures[5]
            .runtime()
            .entry(fixtures[5].node().id())
            .unwrap()
            .revision,
        closed_revision
    );
    assert!(
        journal
            .borrow()
            .iter()
            .filter(|(_, phase, _)| *phase == "apply")
            .all(|(_, _, change)| change.reason == Reason::TriggerOther && !change.restore_focus)
    );
    active.0.borrow().veto.set(false);
    fixtures[1].0.borrow().veto.set(false);
    active_dom.in_scope(active.scope(), || {
        assert!(active.node().request(false, Reason::Trigger, false))
    });
    journal.borrow_mut().clear();
    active_dom.in_scope(active.scope(), || {
        assert!(runtime.dismiss_dropdown_others(active.node().id(), active.node().generation()));
    });
    assert_eq!(others_before_ids(&journal), ["others-both"]);
    assert!(!fixtures[1].open());
    journal.borrow_mut().clear();
    active_dom.in_scope(active.scope(), || {
        assert!(runtime.dismiss_dropdown_others(active.node().id(), active.node().generation()));
        assert!(
            !runtime.dismiss_dropdown_others(active.node().id(), active.node().generation() + 1)
        );
    });
    assert!(journal.borrow().is_empty());
}
