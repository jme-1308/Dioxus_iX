use super::*;
use dioxus::core::current_scope_id;
use std::cell::Cell;

#[test]
fn category_coordination_before_document_attach_allows_reentrant_removal() {
    let runtime = RuntimeHandle::new();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let source_calls = calls.clone();
    runtime.register_category(1, Rc::new(move |_, _| source_calls.borrow_mut().push(1)));
    let recipient_calls = calls.clone();
    let owner = runtime.clone();
    runtime.register_category(
        2,
        Rc::new(move |source, active| {
            assert_eq!((source, active), (1, &[1][..]));
            recipient_calls.borrow_mut().push(2);
            owner.unregister_category(2);
        }),
    );
    assert!(runtime.broadcast_category(1, &[1]));
    assert!(runtime.broadcast_category(1, &[1]));
    assert_eq!(*calls.borrow(), [2]);
}

#[derive(Clone)]
struct RequestTaskFixture(Rc<RequestTaskState>);
impl PartialEq for RequestTaskFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[derive(Default)]
struct RequestTaskState {
    node: RefCell<Option<NodeHandle>>,
    caller: Cell<Option<ScopeId>>,
    owner: Cell<Option<ScopeId>>,
    show_caller: Cell<Option<Signal<bool>>>,
    caller_dropped: Cell<bool>,
    open: Cell<bool>,
    complete: Cell<bool>,
    release: Cell<bool>,
    waker: RefCell<Option<std::task::Waker>>,
}
#[derive(Clone, Copy)]
struct RequestTaskContext(&'static str);

#[component]
fn RequestTaskOwner(fixture: RequestTaskFixture) -> Element {
    use_context_provider(|| RequestTaskContext("owner"));
    let runtime = use_hook(RuntimeHandle::new);
    let registered = fixture.clone();
    let node = use_hook(move || {
        let scope = current_scope_id();
        registered.0.owner.set(Some(scope));
        registered.0.open.set(true);
        let read = registered.clone();
        let before = registered.clone();
        let apply = registered.clone();
        let node = runtime.register(
            "caller-task-overlay".into(),
            1,
            OverlayKind::Popover,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || {
                    assert_eq!(current_scope_id(), scope);
                    assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                    Some(read.0.open.get())
                }),
                before: Rc::new(move |_| {
                    assert_eq!(current_scope_id(), before.0.owner.get().unwrap());
                    assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                    true
                }),
                apply: Rc::new(move |change| {
                    assert_eq!(current_scope_id(), scope);
                    apply.0.open.set(change.open);
                    let pending = apply.clone();
                    spawn(async move {
                        std::future::poll_fn(|cx| {
                            if pending.0.release.get() {
                                std::task::Poll::Ready(())
                            } else {
                                *pending.0.waker.borrow_mut() = Some(cx.waker().clone());
                                std::task::Poll::Pending
                            }
                        })
                        .await;
                        assert_eq!(current_scope_id(), scope);
                        assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                        pending.0.complete.set(true);
                    });
                }),
                key: None,
            },
        );
        *registered.0.node.borrow_mut() = Some(node.clone());
        node
    });
    use_drop(move || node.unregister());
    rsx! { div {} }
}

#[component]
fn RequestTaskCaller(fixture: RequestTaskFixture) -> Element {
    use_context_provider(|| RequestTaskContext("caller"));
    fixture.0.caller.set(Some(current_scope_id()));
    use_drop(move || fixture.0.caller_dropped.set(true));
    rsx! { div {} }
}

fn request_task_root(fixture: RequestTaskFixture) -> Element {
    let show = use_signal(|| true);
    fixture.0.show_caller.set(Some(show));
    rsx! {
        RequestTaskOwner { fixture: fixture.clone() }
        if show() { RequestTaskCaller { fixture: fixture.clone() } }
    }
}

#[test]
fn direct_request_native_work_survives_caller_scope_teardown() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let caller = fixture.0.caller.get().unwrap();
    assert_ne!(fixture.0.owner.get(), Some(caller));
    dom.in_scope(caller, || {
        assert!(node.request(false, Reason::Trigger, true));
        assert_eq!(current_scope_id(), caller, "dispatch restores its caller");
        assert_eq!(consume_context::<RequestTaskContext>().0, "caller");
    });
    assert!(!fixture.0.open.get());
    dom.render_immediate_to_vec();
    assert!(
        fixture.0.waker.borrow().is_some(),
        "accepted work reached await"
    );
    dom.in_scope(ScopeId::APP, || {
        let mut show = fixture.0.show_caller.get().unwrap();
        show.set(false);
    });
    dom.render_immediate_to_vec();
    assert!(fixture.0.caller_dropped.get());
    assert!(!fixture.0.complete.get());
    fixture.0.release.set(true);
    fixture.0.waker.borrow_mut().take().unwrap().wake();
    dom.render_immediate_to_vec();
    assert!(
        fixture.0.complete.get(),
        "owner task survived caller teardown"
    );
    dom.in_scope(fixture.0.owner.get().unwrap(), || node.unregister());
    assert!(
        !node.request(false, Reason::Trigger, true),
        "stale registration is rejected"
    );
    assert!(
        node.with_owner(|| panic!("stale owner cannot run work"))
            .is_none()
    );
}

#[test]
fn shared_surface_observation_runs_in_the_live_scope_and_rejects_stale_delivery() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let delivered = seen.clone();
    node.set_surface_observer(Rc::new(move |focus| {
        assert_eq!(current_scope_id(), owner);
        assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
        delivered.borrow_mut().push(focus);
    }));
    assert!(
        seen.borrow().is_empty(),
        "installation never calls during render"
    );
    dom.in_scope(owner, || node.set_mounted());
    let mut observation = report(
        &runtime,
        1,
        "observe",
        serde_json::json!([{"id":node.id(),"generation":node.generation()}]),
    );
    observation.document = "scoped surface shared observations".into();
    observation.active_id = "registered-field".into();
    observation.focus_candidates = serde_json::from_value(serde_json::json!([
        {"target":"registered-field","tab_index":0,"exact_active":true}
    ]))
    .unwrap();
    let caller = fixture.0.caller.get().unwrap();
    dom.in_scope(caller, || {
        runtime.observe(&observation);
        assert_eq!(current_scope_id(), caller);
    });
    assert_eq!(seen.borrow().len(), 1);
    let SurfaceObservation::Focus(focus) = seen.borrow()[0].clone() else {
        panic!("canonical observation supplies focus facts");
    };
    assert_eq!(focus.active_id, "registered-field");
    assert_eq!(focus.candidates.len(), 1);
    assert_eq!(node.current_focus_observation().as_ref(), Some(&focus));
    runtime.observe(&observation);
    assert_eq!(seen.borrow().len(), 1, "stale observations are rejected");
    let mut pointer: Report = serde_json::from_value(document_value(
        &runtime,
        2,
        &observation.document,
        serde_json::json!({"sequence":1,"action":"input","input_kind":"mousedown"}),
    ))
    .unwrap();
    dom.in_scope(caller, || {
        runtime.observe(&pointer);
        assert_eq!(current_scope_id(), caller);
    });
    assert_eq!(seen.borrow().as_slice()[1], SurfaceObservation::Pointer);
    runtime.observe(&pointer);
    assert_eq!(
        seen.borrow().len(),
        2,
        "duplicate local delivery is rejected"
    );
    pointer.sequence = 3;
    pointer.document_event = Some(
        serde_json::from_value(serde_json::json!({
            "sequence":2,"action":"input","input_kind":"touchstart"
        }))
        .unwrap(),
    );
    dom.in_scope(caller, || runtime.observe(&pointer));
    assert_eq!(seen.borrow().as_slice()[2], SurfaceObservation::Pointer);
    dom.in_scope(owner, || node.unregister());
    observation.sequence = 4;
    runtime.observe(&observation);
    pointer.sequence = 5;
    pointer.document_event.as_mut().unwrap().sequence = 3;
    runtime.observe(&pointer);
    assert_eq!(
        seen.borrow().len(),
        3,
        "removed owners receive neither callback"
    );
    assert!(node.current_focus_observation().is_none());
}

#[test]
fn requested_focus_facts_are_scoped_sequenced_and_never_retarget_replacements() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    dom.in_scope(owner, || node.set_mounted());
    let received = Rc::new(RefCell::new(Vec::new()));
    let delivered = received.clone();
    node.set_surface_observer(Rc::new(move |observation| {
        assert_eq!(current_scope_id(), owner);
        assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
        if let SurfaceObservation::RequestedFocus { request, focus } = observation {
            delivered.borrow_mut().push((request, focus.active_id));
        }
    }));
    assert!(
        !node.request_focus_observation(31),
        "no bridge is an explicit failure"
    );
    let mut observation = Report {
        owner: runtime.owner_id(),
        sequence: 1,
        kind: "observe".into(),
        document: "requested focus scope".into(),
        active_id: "body-field".into(),
        focus_request: Some(FocusRequest {
            id: node.id().into(),
            generation: node.generation(),
            request: 31,
        }),
        ..Default::default()
    };
    let caller = fixture.0.caller.get().unwrap();
    dom.in_scope(caller, || {
        runtime.observe(&observation);
        assert_eq!(current_scope_id(), caller);
    });
    assert_eq!(*received.borrow(), [(31, "body-field".into())]);
    assert!(
        !runtime
            .0
            .borrow()
            .observation
            .as_ref()
            .unwrap()
            .focus_changed
    );
    runtime.observe(&observation);
    assert_eq!(received.borrow().len(), 1, "duplicate delivery is rejected");
    observation.sequence = 2;
    observation.focus_request.as_mut().unwrap().generation += 1;
    runtime.observe(&observation);
    assert_eq!(
        received.borrow().len(),
        1,
        "another registration cannot receive the response"
    );
    dom.in_scope(owner, || node.unregister());
    observation.sequence = 3;
    observation.focus_request.as_mut().unwrap().generation = node.generation();
    runtime.observe(&observation);
    assert_eq!(
        received.borrow().len(),
        1,
        "removed requester is not invoked"
    );
    assert!(!node.request_focus_observation(32));
    let replacement = dom.in_scope(owner, || {
        runtime.register(
            node.id().into(),
            node.generation() + 1,
            OverlayKind::Popover,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(|| Some(true)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        )
    });
    replacement.set_mounted();
    replacement.set_surface_observer(Rc::new(|observation| {
        assert!(
            !matches!(observation, SurfaceObservation::RequestedFocus { .. }),
            "old response must not reach a replacement"
        );
    }));
    observation.sequence = 4;
    runtime.observe(&observation);
    assert_eq!(received.borrow().len(), 1);
}

#[test]
fn opening_focus_uses_registered_partition_and_authored_header_with_mount_lifetimes() {
    let runtime = RuntimeHandle::new();
    let popover = model(&runtime, "opening", OverlayKind::Popover, Rc::default()).node;
    popover.set_surface_metadata(SurfaceMetadata {
        popover_host: "opening-panel".into(),
        ..Default::default()
    });
    let nested = model(&runtime, "nested", OverlayKind::Popover, Rc::default()).node;
    let mounted = Rc::new(MountedData::new(ItemMountMock::default()));
    for id in ["header", "body", "nested-header", "outside"] {
        runtime.register_item_mount(id, id, mounted.clone());
    }
    runtime.set_item_popover_scope(
        "header",
        Some(PopoverFocusScope {
            owner: Some(popover.clone()),
            header: true,
        }),
    );
    runtime.set_item_popover_scope(
        "nested-header",
        Some(PopoverFocusScope {
            owner: Some(nested),
            header: true,
        }),
    );
    let observation = FocusObservation {
        candidates: serde_json::from_value(serde_json::json!([
            {"target":"outside","tab_index":1,"popover":"different-panel"},
            {"target":"unregistered","tab_index":1,"popover":"opening-panel"},
            {"target":"header","tab_index":1,"popover":"opening-panel"},
            {"target":"body","tab_index":5,"popover":"opening-panel"},
            {"target":"nested-header","tab_index":0,"popover":"opening-panel"}
        ]))
        .unwrap(),
        ..Default::default()
    };
    let candidates = popover.opening_focus_candidates(&observation);
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (candidate.target.as_str(), candidate.in_header))
            .collect::<Vec<_>>(),
        [("header", true), ("body", false), ("nested-header", false)]
    );
    assert_eq!(
        crate::popover_focus::opening_target(&candidates, false).as_deref(),
        Some("body")
    );
    runtime.register_item_mount("header", "header", mounted.clone());
    assert!(
        popover.opening_focus_candidates(&observation)[0].in_header,
        "same mounted handle retains its role"
    );
    runtime.register_item_mount(
        "header",
        "header",
        Rc::new(MountedData::new(ItemMountMock::default())),
    );
    assert!(
        !popover.opening_focus_candidates(&observation)[0].in_header,
        "replacement cannot inherit old role"
    );
    runtime.unregister_item_cue("body");
    assert!(
        !popover
            .opening_focus_candidates(&observation)
            .iter()
            .any(|candidate| candidate.target == "body")
    );
    // A control whose actual overlay owner was replaced is not live merely
    // because a native candidate with the same ID remains in old facts.
    runtime.register_owned_item_control_mount(
        "header",
        "header",
        "header",
        mounted,
        Some((popover.id().into(), popover.generation() + 1)),
    );
    assert!(
        !popover
            .opening_focus_candidates(&observation)
            .iter()
            .any(|candidate| candidate.target == "header")
    );
    popover.unregister();
    assert!(popover.opening_focus_candidates(&observation).is_empty());
}

#[test]
fn passive_focus_requests_use_original_scope_and_exact_non_overlay_recipient() {
    use dioxus::document::{Eval, EvalError, Evaluator};
    use std::task::{Context, Poll};
    struct Wire(Rc<RefCell<Vec<serde_json::Value>>>);
    impl Evaluator for Wire {
        fn send(&self, value: serde_json::Value) -> Result<(), EvalError> {
            self.0.borrow_mut().push(value);
            Ok(())
        }
        fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            Poll::Pending
        }
        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            Poll::Pending
        }
    }
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let runtime = fixture.0.node.borrow().as_ref().unwrap().runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let caller = fixture.0.caller.get().unwrap();
    let received = Rc::new(RefCell::new(Vec::new()));
    let delivered = received.clone();
    let generation = dom.in_scope(owner, || {
        runtime.register_focus_observer(
            "passive-pane".into(),
            Rc::new(move |event| {
                assert_eq!(current_scope_id(), owner);
                assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                delivered.borrow_mut().push(event);
            }),
        )
    });
    let peer_requests = Rc::new(Cell::new(0));
    let peer_seen = peer_requests.clone();
    let peer_generation = dom.in_scope(caller, || {
        runtime.register_focus_observer(
            "passive-peer".into(),
            Rc::new(move |event| {
                assert_eq!(current_scope_id(), caller);
                if matches!(event, ScopedFocusEvent::Requested { .. }) {
                    peer_seen.set(peer_seen.get() + 1);
                }
            }),
        )
    });
    assert!(!runtime.0.borrow().nodes.contains_key("passive-pane"));
    assert!(
        !runtime.request_scoped_focus("passive-pane", generation, 44),
        "missing bridge is explicit failure"
    );
    let sent = Rc::new(RefCell::new(Vec::new()));
    let eval_owner = dioxus::signals::Owner::default();
    runtime.0.borrow_mut().bridge = Some(Eval::new(
        eval_owner.insert(Box::new(Wire(sent.clone())) as Box<dyn Evaluator>),
    ));
    assert!(dom.in_scope(caller, || runtime.request_scoped_focus(
        "passive-pane",
        generation,
        44
    )));
    assert_eq!(
        sent.borrow()[0],
        serde_json::json!({"focus_request":{"id":"passive-pane","generation":generation,"request":44}})
    );
    assert!(!runtime.request_scoped_focus("passive-pane", generation + 100, 44));
    let mut report: Report = serde_json::from_value(serde_json::json!({
            "owner":runtime.owner_id(),"sequence":1,"kind":"observe","document":"passive shared facts",
            "active_id":"pane-input","active_hosts":["passive-pane"],"active_nested":true,"eligible_focus_ids":["programmatic-opener","pane-input"],
            "focus_candidates":[{"target":"pane-input","tab_index":0,"hosts":["passive-pane"],"autofocus":true}],
            "focus_request":{"id":"passive-pane","generation":generation,"request":44}
        })).unwrap();
    dom.in_scope(caller, || {
        runtime.observe(&report);
        assert_eq!(current_scope_id(), caller);
    });
    let events = received.borrow();
    let ScopedFocusEvent::Observe(facts) = &events[0] else {
        panic!("canonical report broadcasts ordinary facts first");
    };
    assert_eq!(facts.active_hosts, ["passive-pane"]);
    assert!(facts.nested);
    assert_eq!(
        facts.eligible_focus_ids,
        ["programmatic-opener", "pane-input"]
    );
    assert!(
        !facts
            .observation
            .candidates
            .iter()
            .any(|candidate| candidate.target == "programmatic-opener"),
        "return eligibility does not broaden Tab candidates"
    );
    let legacy: Report = serde_json::from_value(serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","focus_candidates":[{"target":"legacy-control","tab_index":0}]})).unwrap();
    assert!(
        ScopedFocusFacts::from(&legacy)
            .eligible_focus_ids
            .is_empty(),
        "missing native eligibility never invents a destination"
    );
    assert!(!legacy.active_nested);
    assert!(
        !legacy.focus_candidates[0].autofocus,
        "missing autofocus metadata defaults to false"
    );
    assert!(facts.observation.candidates[0].autofocus);
    assert_eq!(
        events[1],
        ScopedFocusEvent::Requested {
            request: 44,
            facts: facts.clone()
        }
    );
    drop(events);
    assert_eq!(
        peer_requests.get(),
        0,
        "another passive root cannot consume this request"
    );
    runtime.observe(&report);
    report.sequence = 2;
    report.owner = "foreign-runtime".into();
    runtime.observe(&report);
    assert_eq!(
        received.borrow().len(),
        2,
        "duplicate and foreign reports are not admitted"
    );
    runtime.unregister_focus_observer("passive-pane", generation);
    assert!(!runtime.request_scoped_focus("passive-pane", generation, 45));
    runtime.unregister_focus_observer("passive-peer", peer_generation);
}

#[test]
fn passive_focus_registration_does_not_keep_disposed_scope_or_renderer_alive() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let runtime = fixture.0.node.borrow().as_ref().unwrap().runtime().unwrap();
    let caller = fixture.0.caller.get().unwrap();
    let seen = Rc::new(Cell::new(0));
    let delivered = seen.clone();
    let generation = dom.in_scope(caller, || {
        runtime.register_focus_observer(
            "disposed-pane".into(),
            Rc::new(move |_| delivered.set(delivered.get() + 1)),
        )
    });
    let mut report = Report {
        owner: runtime.owner_id(),
        sequence: 1,
        kind: "observe".into(),
        ..Default::default()
    };
    runtime.observe(&report);
    assert_eq!(seen.get(), 1);
    dom.in_scope(ScopeId::APP, || {
        fixture.0.show_caller.get().unwrap().set(false)
    });
    settle_item_effects(&mut dom);
    assert!(fixture.0.caller_dropped.get());
    report.sequence = 2;
    report.focus_request = Some(FocusRequest {
        id: "disposed-pane".into(),
        generation,
        request: 1,
    });
    runtime.observe(&report);
    assert_eq!(
        seen.get(),
        1,
        "forgotten cleanup still cannot invoke a disposed scope"
    );
    let weak = runtime.0.borrow().focus_observers["disposed-pane"]
        .renderer
        .clone();
    drop(dom);
    assert!(
        weak.upgrade().is_none(),
        "registration holds no renderer ownership"
    );
    report.sequence = 3;
    runtime.observe(&report);
    assert_eq!(seen.get(), 1);
    runtime.unregister_focus_observer("disposed-pane", generation);
}

#[test]
fn passive_focus_delivery_rechecks_replacement_and_newer_admission_after_callbacks() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let runtime = fixture.0.node.borrow().as_ref().unwrap().runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let caller = fixture.0.caller.get().unwrap();
    let old_calls = Rc::new(Cell::new(0));
    let old_seen = old_calls.clone();
    let old_generation = dom.in_scope(caller, || {
        runtime.register_focus_observer(
            "z-peer".into(),
            Rc::new(move |_| old_seen.set(old_seen.get() + 1)),
        )
    });
    let replaced = Rc::new(Cell::new(false));
    let current_generation = Rc::new(Cell::new(0));
    let new_generation = current_generation.clone();
    let received = Rc::new(RefCell::new(Vec::new()));
    let delivered = received.clone();
    let reentrant = runtime.clone();
    let first_generation = dom.in_scope(owner, || {
        runtime.register_focus_observer(
            "a-first".into(),
            Rc::new(move |event| {
                if !matches!(event, ScopedFocusEvent::Observe(_)) || replaced.replace(true) {
                    return;
                }
                reentrant.unregister_focus_observer("z-peer", old_generation);
                let output = delivered.clone();
                let generation = reentrant.register_focus_observer(
                    "z-peer".into(),
                    Rc::new(move |event| {
                        let (request, facts) = match event {
                            ScopedFocusEvent::Observe(facts) => (0, facts),
                            ScopedFocusEvent::Requested { request, facts } => (request, facts),
                        };
                        output
                            .borrow_mut()
                            .push((request, facts.observation.active_id));
                    }),
                );
                new_generation.set(generation);
                reentrant.observe(&Report {
                    owner: reentrant.owner_id(),
                    sequence: 2,
                    kind: "observe".into(),
                    active_id: "fresh".into(),
                    focus_request: Some(FocusRequest {
                        id: "z-peer".into(),
                        generation,
                        request: 9,
                    }),
                    ..Default::default()
                });
            }),
        )
    });
    runtime.observe(&Report {
        owner: runtime.owner_id(),
        sequence: 1,
        kind: "observe".into(),
        active_id: "retired".into(),
        focus_request: Some(FocusRequest {
            id: "z-peer".into(),
            generation: old_generation,
            request: 8,
        }),
        ..Default::default()
    });
    assert_eq!(
        old_calls.get(),
        0,
        "earlier callback removal retires the old recipient"
    );
    assert_eq!(
        *received.borrow(),
        [(0, "fresh".into()), (9, "fresh".into())],
        "new recipient sees only current admitted facts and its exact request"
    );
    runtime.unregister_focus_observer("z-peer", old_generation);
    assert_eq!(
        runtime.0.borrow().focus_observers["z-peer"].generation,
        current_generation.get(),
        "stale cleanup cannot remove a replacement"
    );
    runtime.unregister_focus_observer("z-peer", current_generation.get());
    runtime.unregister_focus_observer("a-first", first_generation);
}

#[derive(Default)]
struct TypedStreamBackend {
    queued: std::collections::VecDeque<serde_json::Value>,
    sent: Vec<serde_json::Value>,
    waker: Option<std::task::Waker>,
}
struct TypedStreamEval(Rc<RefCell<TypedStreamBackend>>);
impl dioxus::document::Evaluator for TypedStreamEval {
    fn send(&self, value: serde_json::Value) -> Result<(), dioxus::document::EvalError> {
        self.0.borrow_mut().sent.push(value);
        Ok(())
    }
    fn poll_recv(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<serde_json::Value, dioxus::document::EvalError>> {
        let mut state = self.0.borrow_mut();
        if let Some(value) = state.queued.pop_front() {
            std::task::Poll::Ready(Ok(value))
        } else {
            state.waker = Some(cx.waker().clone());
            std::task::Poll::Pending
        }
    }
    fn poll_join(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<serde_json::Value, dioxus::document::EvalError>> {
        std::task::Poll::Pending
    }
}
#[derive(Default)]
struct TypedStreamState {
    scripts: Vec<String>,
    streams: Vec<Rc<RefCell<TypedStreamBackend>>>,
}
struct TypedStreamDocument {
    owner: dioxus::signals::Owner,
    state: Rc<RefCell<TypedStreamState>>,
}
impl dioxus::document::Document for TypedStreamDocument {
    fn eval(&self, script: String) -> dioxus::document::Eval {
        assert_eq!(current_scope_id(), ScopeId::ROOT);
        assert!(
            try_consume_context::<Rc<dyn dioxus::document::Document>>().is_none(),
            "ROOT intentionally has no Document provider"
        );
        assert!(!script.contains("__START_SEQUENCE__"));
        let owner: String = serde_json::from_str(
            script
                .split("const owner=")
                .nth(1)
                .unwrap()
                .split(",listeners=")
                .next()
                .unwrap(),
        )
        .unwrap();
        let sequence: u64 = script
            .split("let alive=true,sequence=")
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let backend = Rc::new(RefCell::new(TypedStreamBackend::default()));
        backend.borrow_mut().queued.push_back(serde_json::json!({"owner":owner,"document":"typed stream fixture","sequence":sequence+1,"kind":"observe","active_default":true}));
        self.state.borrow_mut().scripts.push(script);
        self.state.borrow_mut().streams.push(backend.clone());
        dioxus::document::Eval::new(
            self.owner
                .insert(Box::new(TypedStreamEval(backend)) as Box<dyn dioxus::document::Evaluator>),
        )
    }
}
#[derive(Clone)]
struct TypedStreamFixture {
    document: Rc<dyn dioxus::document::Document>,
    runtime: Rc<RefCell<Option<RuntimeHandle>>>,
    first: Rc<Cell<Option<Signal<bool>>>>,
    second: Rc<Cell<Option<Signal<bool>>>>,
    attempts: Rc<Cell<usize>>,
}
impl PartialEq for TypedStreamFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.runtime, &other.runtime)
    }
}
#[component]
fn TypedStreamProvider(fixture: TypedStreamFixture) -> Element {
    use_context_provider(|| fixture.document.clone());
    let runtime = fixture.runtime.borrow().clone().unwrap();
    rsx! { OverlayTransport { runtime } }
}
fn typed_stream_root(fixture: TypedStreamFixture) -> Element {
    let runtime = use_overlay_runtime();
    *fixture.runtime.borrow_mut() = Some(runtime.clone());
    let first = use_signal(|| true);
    let second = use_signal(|| true);
    fixture.first.set(Some(first));
    fixture.second.set(Some(second));
    let passive = runtime.clone();
    let generation =
        use_hook(|| passive.register_focus_observer("typed-startup".into(), Rc::new(|_| {})));
    let request = runtime.clone();
    let attempts = fixture.attempts.clone();
    use_effect(move || {
        attempts.set(attempts.get() + 1);
        let _ = request.request_scoped_focus("typed-startup", generation, 42);
    });
    use_drop(move || runtime.unregister_focus_observer("typed-startup", generation));
    rsx! {
        div { "Renderless typed stream" }
        if first() { TypedStreamProvider { fixture:fixture.clone() } }
        if second() { TypedStreamProvider { fixture } }
    }
}
fn typed_stream_fixture() -> (TypedStreamFixture, Rc<RefCell<TypedStreamState>>) {
    let state = Rc::new(RefCell::new(TypedStreamState::default()));
    (
        TypedStreamFixture {
            document: Rc::new(TypedStreamDocument {
                owner: Default::default(),
                state: state.clone(),
            }),
            runtime: Rc::default(),
            first: Rc::default(),
            second: Rc::default(),
            attempts: Rc::default(),
        },
        state,
    )
}
fn typed_packet(runtime: &RuntimeHandle, sequence: u64, active: &str) -> Report {
    Report {
        owner: runtime.owner_id(),
        document: "typed stream fixture".into(),
        sequence,
        kind: "observe".into(),
        active_id: active.into(),
        ..Default::default()
    }
}
fn typed_push(stream: &Rc<RefCell<TypedStreamBackend>>, value: serde_json::Value) {
    let wake = {
        let mut state = stream.borrow_mut();
        state.queued.push_back(value);
        state.waker.take()
    };
    if let Some(wake) = wake {
        wake.wake();
    }
}

#[test]
fn typed_overlay_stream_uses_scoped_provider_shares_leases_and_restarts_from_current_sequence() {
    let (fixture, native) = typed_stream_fixture();
    let mut dom = VirtualDom::new_with_props(typed_stream_root, fixture.clone());
    dom.rebuild_in_place();
    settle_item_effects(&mut dom);
    let runtime = fixture.runtime.borrow().clone().unwrap();
    assert_eq!(
        native.borrow().scripts.len(),
        1,
        "two renderless leases share one native stream"
    );
    assert_eq!(runtime.0.borrow().bridge_leases, 2);
    assert_eq!(
        runtime.0.borrow().sequence,
        1,
        "typed receiver admits initial native report"
    );
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("data-ix-overlay-transport"));
    let stream = native.borrow().streams[0].clone();
    assert!(
        stream
            .borrow()
            .sent
            .iter()
            .any(|value| value["focus_request"]["request"] == 42),
        "startup request effect observes source readiness without a request queue"
    );
    assert!(fixture.attempts.get() >= 1);
    let epoch = runtime.0.borrow().bridge_epoch;
    typed_push(
        &stream,
        serde_json::json!({"owner":runtime.owner_id(),"document":"typed stream fixture","sequence":7,"kind":"observe","active_id":"first-input"}),
    );
    settle_item_effects(&mut dom);
    assert_eq!(runtime.0.borrow().sequence, 7);
    dom.in_scope(ScopeId::ROOT, || fixture.first.get().unwrap().set(false));
    settle_item_effects(&mut dom);
    assert_eq!(runtime.0.borrow().bridge_leases, 1);
    assert_eq!(native.borrow().scripts.len(), 1);
    assert!(
        !stream
            .borrow()
            .sent
            .iter()
            .any(|value| value["finish"] == true)
    );
    typed_push(
        &stream,
        serde_json::json!({"owner":runtime.owner_id(),"document":"typed stream fixture","sequence":8,"kind":"observe","active_id":"peer-input"}),
    );
    settle_item_effects(&mut dom);
    assert_eq!(
        runtime.0.borrow().observation.as_ref().unwrap().active_id,
        "peer-input",
        "root task outlives the first provider component"
    );
    dom.in_scope(ScopeId::ROOT, || fixture.second.get().unwrap().set(false));
    settle_item_effects(&mut dom);
    assert!(runtime.0.borrow().bridge.is_none());
    assert!(runtime.0.borrow().bridge_task.is_none());
    assert_eq!(
        stream
            .borrow()
            .sent
            .iter()
            .filter(|value| value["finish"] == true)
            .count(),
        1
    );
    assert!(!runtime.receive_report(epoch, typed_packet(&runtime, 999, "retired")));
    assert_eq!(runtime.0.borrow().sequence, 8);
    dom.in_scope(ScopeId::ROOT, || fixture.second.get().unwrap().set(true));
    settle_item_effects(&mut dom);
    assert_eq!(native.borrow().scripts.len(), 2);
    assert!(native.borrow().scripts[1].contains("sequence=8,"));
    assert_eq!(
        runtime.0.borrow().sequence,
        9,
        "restarted initial report exceeds the retained sequence"
    );
    assert!(!runtime.receive_report(epoch, typed_packet(&runtime, 1000, "retired-again")));
    let active_epoch = runtime.0.borrow().bridge_epoch;
    let mut foreign = typed_packet(&runtime, 999, "foreign");
    foreign.owner = "other-root".into();
    assert!(runtime.receive_report(active_epoch, foreign));
    assert_eq!(
        runtime.0.borrow().sequence,
        9,
        "current source still uses normal owner admission"
    );
    let renderer = runtime.0.borrow().bridge_owner.as_ref().unwrap().0.clone();
    let restarted = native.borrow().streams[1].clone();
    drop(dom);
    assert!(
        renderer.upgrade().is_none(),
        "receiver retains neither renderer nor original consumer"
    );
    assert_eq!(runtime.0.borrow().bridge_leases, 0);
    assert_eq!(
        restarted
            .borrow()
            .sent
            .iter()
            .filter(|value| value["finish"] == true)
            .count(),
        1
    );
    assert!(!runtime.receive_report(active_epoch, typed_packet(&runtime, 1001, "removed-root")));
}

#[test]
fn typed_overlay_streams_keep_independent_roots_and_reports_isolated() {
    let (first, first_native) = typed_stream_fixture();
    let (second, second_native) = typed_stream_fixture();
    let mut a = VirtualDom::new_with_props(typed_stream_root, first.clone());
    let mut b = VirtualDom::new_with_props(typed_stream_root, second.clone());
    a.rebuild_in_place();
    b.rebuild_in_place();
    settle_item_effects(&mut a);
    settle_item_effects(&mut b);
    let ra = first.runtime.borrow().clone().unwrap();
    let rb = second.runtime.borrow().clone().unwrap();
    assert_ne!(ra.owner_id(), rb.owner_id());
    assert_eq!(
        (
            first_native.borrow().scripts.len(),
            second_native.borrow().scripts.len()
        ),
        (1, 1)
    );
    let epoch = ra.0.borrow().bridge_epoch;
    a.in_scope(ScopeId::ROOT, || {
        assert!(ra.receive_report(epoch, typed_packet(&rb, 100, "wrong-root")));
    });
    assert_eq!(ra.0.borrow().sequence, 1);
    drop(a);
    assert!(ra.0.borrow().bridge.is_none());
    assert!(rb.0.borrow().bridge.is_some());
    let active_epoch = rb.0.borrow().bridge_epoch;
    b.in_scope(ScopeId::ROOT, || {
        assert!(rb.receive_report(active_epoch, typed_packet(&rb, 2, "live-root")));
    });
    assert_eq!(rb.0.borrow().sequence, 2);
}

struct Model {
    open: Rc<Cell<bool>>,
    veto: Rc<Cell<bool>>,
    node: NodeHandle,
}
fn model(
    runtime: &RuntimeHandle,
    id: &str,
    kind: OverlayKind,
    log: Rc<RefCell<Vec<String>>>,
) -> Model {
    let open = Rc::new(Cell::new(false));
    let veto = Rc::new(Cell::new(false));
    let read = open.clone();
    let apply = open.clone();
    let before_veto = veto.clone();
    let before_log = log.clone();
    let apply_log = log.clone();
    let before_id = id.to_string();
    let apply_id = id.to_string();
    let node = runtime.register(
        id.to_string(),
        1,
        kind,
        CloseBehavior::Both,
        Callbacks {
            read: Rc::new(move || Some(read.get())),
            before: Rc::new(move |change| {
                before_log
                    .borrow_mut()
                    .push(format!("before:{before_id}:{}", change.open));
                !before_veto.get()
            }),
            apply: Rc::new(move |change| {
                apply.set(change.open);
                apply_log
                    .borrow_mut()
                    .push(format!("apply:{apply_id}:{}", change.open));
            }),
            key: None,
        },
    );
    node.set_ownership(Vec::new(), true);
    node.set_mounted();
    Model { open, veto, node }
}
fn assign(runtime: &RuntimeHandle, id: &str, ancestors: &[&str], ready: bool) {
    let handle = |id: &str| NodeHandle {
        runtime: Rc::downgrade(&runtime.0),
        id: id.into(),
        generation: runtime.entry(id).unwrap().generation,
    };
    let node = handle(id);
    node.set_ownership(ancestors.iter().map(|id| handle(id)).collect(), ready);
    node.set_mounted();
}
#[test]
fn authored_ownership_ignores_native_graph_claims_and_guards_actual_lifetimes() {
    let runtime = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(&runtime, "owned-parent", OverlayKind::Dropdown, log.clone());
    let child = model(&runtime, "owned-child", OverlayKind::Dropdown, log.clone());
    child.node.set_ownership(
        vec![parent.node.clone(), child.node.clone(), parent.node.clone()],
        true,
    );
    assert_eq!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("owned-child")
            .as_deref(),
        Some("owned-parent")
    );
    runtime.observe(&report(
        &runtime,
        1,
        "observe",
        serde_json::json!([
            {"id":"owned-child","generation":1,"connected":false,"discovery_ready":false,
                "trigger_ancestors":["fabricated"],"assignment_ancestors":[]}
        ]),
    ));
    runtime.observe(&report(&runtime, 2, "observe", serde_json::json!([])));
    assert_eq!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("owned-child")
            .as_deref(),
        Some("owned-parent")
    );
    let same = runtime.register(
        "owned-child".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        runtime.entry("owned-child").unwrap().callbacks,
    );
    assert!(same == child.node);
    assert_eq!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("owned-child")
            .as_deref(),
        Some("owned-parent")
    );
    let replacement = runtime.register(
        "owned-parent".into(),
        2,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        runtime.entry("owned-parent").unwrap().callbacks,
    );
    replacement.set_ownership(Vec::new(), true);
    replacement.set_mounted();
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("owned-child")
            .is_none(),
        "a same-ID replacement cannot inherit a stale parent handle"
    );
    parent.node.set_ownership(Vec::new(), true);
    child.node.set_ownership(vec![replacement.clone()], true);
    assert_eq!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("owned-child")
            .as_deref(),
        Some("owned-parent")
    );
    let foreign = RuntimeHandle::new();
    let copied = model(&foreign, "owned-parent", OverlayKind::Dropdown, log);
    assert!(
        copied.node != parent.node,
        "runtime identity belongs to handle equality"
    );
    child.node.unregister();
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .cached_children("owned-parent")
            .is_empty()
    );
}

#[derive(Clone, Default)]
struct ScopeFixture(Rc<ScopeFixtureState>);
#[derive(Default)]
struct ScopeFixtureState {
    switch: Cell<Option<Signal<bool>>>,
    seen: RefCell<Vec<String>>,
}
impl PartialEq for ScopeFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn ScopeFixtureChild(state: ScopeFixture) -> Element {
    let ancestors = use_overlay_scope(None);
    *state.0.seen.borrow_mut() = ancestors.iter().map(|node| node.id.clone()).collect();
    rsx! { div {} }
}
fn scope_fixture_root(state: ScopeFixture) -> Element {
    let runtime = use_hook(RuntimeHandle::new);
    let log = Rc::new(RefCell::new(Vec::new()));
    let nodes = use_hook(move || {
        (
            model(&runtime, "scope-first", OverlayKind::Header, log.clone()).node,
            model(&runtime, "scope-second", OverlayKind::Header, log).node,
        )
    });
    let switch = use_signal(|| false);
    state.0.switch.set(Some(switch));
    use_overlay_scope(Some(if switch() { nodes.1 } else { nodes.0 }));
    rsx! { ScopeFixtureChild { state } }
}
fn scope_override_fixture_root(state: ScopeFixture) -> Element {
    let runtime = use_hook(RuntimeHandle::new);
    let log = Rc::new(RefCell::new(Vec::new()));
    let nodes = use_hook(move || {
        (
            model(&runtime, "scope-inner", OverlayKind::Popover, log.clone()).node,
            model(&runtime, "scope-effective-parent", OverlayKind::Header, log).node,
        )
    });
    let lexical = use_overlay_ancestors();
    assert!(lexical.is_empty());
    use_overlay_scope_with_ancestors(Some(nodes.0), vec![nodes.1.clone(), nodes.1]);
    rsx! { ScopeFixtureChild { state } }
}
#[test]
fn explicit_surface_lineage_reaches_body_without_lexical_parent_discovery() {
    let state = ScopeFixture::default();
    let mut dom = VirtualDom::new_with_props(scope_override_fixture_root, state.clone());
    dom.rebuild_in_place();
    assert_eq!(
        *state.0.seen.borrow(),
        ["scope-inner", "scope-effective-parent"]
    );
}
#[test]
fn reactive_overlay_scope_tracks_parent_updates_without_consuming_own_provider() {
    let state = ScopeFixture::default();
    let mut dom = VirtualDom::new_with_props(scope_fixture_root, state.clone());
    dom.rebuild_in_place();
    assert_eq!(*state.0.seen.borrow(), ["scope-first"]);
    dom.in_runtime(|| state.0.switch.get().unwrap().set(true));
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert_eq!(*state.0.seen.borrow(), ["scope-second"]);
    dom.in_runtime(|| state.0.switch.get().unwrap().set(false));
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert_eq!(*state.0.seen.borrow(), ["scope-first"]);
}

fn report(runtime: &RuntimeHandle, sequence: u64, kind: &str, nodes: serde_json::Value) -> Report {
    serde_json::from_value(serde_json::json!({
        "owner":runtime.owner_id(),"sequence":sequence,"kind":kind,"nodes":nodes,
    }))
    .unwrap()
}
fn nodes(runtime: &RuntimeHandle, ancestors: &[&str]) -> serde_json::Value {
    assign(runtime, "child", ancestors, true);
    serde_json::json!([
        {"id":"header","generation":1},
        {"id":"child","generation":1}
    ])
}

struct TypedMouse;
impl dioxus::html::InteractionLocation for TypedMouse {
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
impl dioxus::html::InteractionElementOffset for TypedMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for TypedMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for TypedMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for TypedMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
pub(super) fn typed_mouse() -> MouseEvent {
    Event::new(Rc::new(dioxus::html::MouseData::new(TypedMouse)), true)
}

#[test]
fn dioxus_inside_click_keeps_browser_defaults_and_current_peer_policies() {
    for behavior in [
        CloseBehavior::Both,
        CloseBehavior::Inside,
        CloseBehavior::Outside,
        CloseBehavior::None,
    ] {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let origin = model(&runtime, "origin", OverlayKind::Dropdown, log.clone());
        origin.node.set_close_behavior(behavior);
        origin.open.set(true);
        let peers = [
            model(&runtime, "both", OverlayKind::Dropdown, log.clone()),
            model(&runtime, "outside", OverlayKind::Dropdown, log.clone()),
            model(&runtime, "inside", OverlayKind::Dropdown, log.clone()),
            model(&runtime, "none", OverlayKind::Dropdown, log.clone()),
        ];
        peers[1].node.set_close_behavior(CloseBehavior::Outside);
        peers[2].node.set_close_behavior(CloseBehavior::Inside);
        peers[3].node.set_close_behavior(CloseBehavior::None);
        for peer in &peers {
            peer.open.set(true);
        }
        let click = typed_mouse();
        origin.node.dioxus_inside_click(&click);
        assert!(
            click.default_action_enabled(),
            "native input/form defaults remain enabled"
        );
        assert!(
            !click.propagates(),
            "the content owns virtual propagation for every policy"
        );
        assert_eq!(origin.open.get(), !behavior.allows_inside());
        assert_eq!(
            peers.map(|peer| peer.open.get()),
            if behavior.allows_inside() {
                [false, false, true, true]
            } else {
                [true; 4]
            }
        );
        assert_eq!(
            log.borrow()
                .iter()
                .filter(|entry| entry.as_str() == "before:origin:false")
                .count(),
            if behavior.allows_inside() { 1 } else { 0 }
        );
    }
}

#[test]
fn dioxus_inside_click_honors_consumer_prevention_veto_and_current_owner() {
    let runtime = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let origin = model(&runtime, "origin", OverlayKind::Header, log.clone());
    let peer = model(&runtime, "peer", OverlayKind::Dropdown, log.clone());
    origin.open.set(true);
    peer.open.set(true);
    let prevented = typed_mouse();
    prevented.prevent_default();
    origin.node.dioxus_inside_click(&prevented);
    assert!(!prevented.default_action_enabled() && !prevented.propagates());
    assert!(origin.open.get() && peer.open.get() && log.borrow().is_empty());

    origin.veto.set(true);
    let click = typed_mouse();
    origin.node.dioxus_inside_click(&click);
    assert!(click.default_action_enabled() && !click.propagates());
    assert!(
        origin.open.get() && !peer.open.get(),
        "origin veto does not veto an eligible peer"
    );
    assert_eq!(
        log.borrow()
            .iter()
            .filter(|entry| entry.as_str() == "before:origin:false")
            .count(),
        1
    );
    origin.open.set(false);
    let closed = typed_mouse();
    origin.node.dioxus_inside_click(&closed);
    assert!(closed.default_action_enabled() && closed.propagates());
    origin.open.set(true);
    peer.open.set(true);
    let callbacks = runtime.entry("origin").unwrap().callbacks;
    let replacement = runtime.register(
        "origin".into(),
        2,
        OverlayKind::Header,
        CloseBehavior::Both,
        callbacks,
    );
    let stale = typed_mouse();
    origin.node.dioxus_inside_click(&stale);
    assert!(stale.default_action_enabled() && stale.propagates() && peer.open.get());
    replacement.unregister();
    let removed = typed_mouse();
    replacement.dioxus_inside_click(&removed);
    assert!(removed.default_action_enabled() && removed.propagates() && peer.open.get());
}

#[test]
fn dioxus_surface_stop_does_not_replace_window_outside_coordination() {
    let runtime = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let origin = model(&runtime, "origin", OverlayKind::Dropdown, log.clone());
    let peer = model(&runtime, "peer", OverlayKind::Dropdown, log);
    origin.node.set_close_behavior(CloseBehavior::Outside);
    origin.open.set(true);
    peer.open.set(true);
    let click = typed_mouse();
    origin.node.dioxus_inside_click(&click);
    assert!(click.default_action_enabled() && !click.propagates());
    assert!(origin.open.get() && peer.open.get());
    runtime.input(event(
        serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),
            "sequence":1,"kind":"observe",
            "nodes":[{"id":"origin","generation":1},
                {"id":"peer","generation":1}]}),
    ));
    let outside = event(
        serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),
            "sequence":2,"kind":"document","document_event":{"sequence":1,"action":"outside","hosts":["origin"]}}),
    );
    runtime.input(outside);
    assert!(origin.open.get() && !peer.open.get());
}

#[test]
fn parent_request_child_veto_and_more_outside_are_not_atomic() {
    let runtime = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let header = model(&runtime, "header", OverlayKind::Header, log.clone());
    let child = model(&runtime, "child", OverlayKind::Dropdown, log.clone());
    runtime.observe(&report(
        &runtime,
        1,
        "observe",
        nodes(&runtime, &["header"]),
    ));
    assert!(header.node.request(true, Reason::Trigger, false));
    assert!(child.node.request(true, Reason::Trigger, false));
    log.borrow_mut().clear();
    child.veto.set(true);
    assert!(header.node.request(false, Reason::Trigger, false));
    assert_eq!(
        &*log.borrow(),
        &[
            "before:header:false",
            "before:child:false",
            "apply:header:false"
        ]
    );
    assert!(!header.open.get() && child.open.get());
    child.veto.set(false);
    assert!(header.node.request(true, Reason::Trigger, false));
    header.veto.set(true);
    assert!(!header.node.request(false, Reason::Trigger, false));
    assert!(runtime.dismiss_dropdown_others(header.node.id(), header.node.generation()));
    assert!(
        header.open.get() && !child.open.get(),
        "the explicit More peer pass remains independent of its toggle veto"
    );
}

#[test]
fn current_parent_controls_escape_and_header_focus_adapter() {
    let runtime = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let header = model(&runtime, "header", OverlayKind::Header, log.clone());
    let child = model(&runtime, "child", OverlayKind::Dropdown, log);
    runtime.observe(&report(&runtime, 1, "observe", nodes(&runtime, &[])));
    runtime.observe(&report(
        &runtime,
        2,
        "observe",
        nodes(&runtime, &["header"]),
    ));
    assert!(header.node.request(true, Reason::Trigger, false));
    assert_eq!(
        runtime
            .0
            .borrow()
            .registry
            .cached_parent("child")
            .as_deref(),
        Some("header")
    );
    assert!(header.node.owns_keyboard_topmost());
    assert!(!child.node.owns_keyboard_topmost());
    assert!(child.node.request(true, Reason::Trigger, false));
    assert!(child.node.owns_keyboard_topmost());
    child.veto.set(true);
    assert!(header.node.request(false, Reason::Trigger, false));
    child.veto.set(false);
    assert!(header.node.request(true, Reason::Trigger, false));
    assert!(
        !child.node.owns_keyboard_topmost(),
        "newer topmost Header must retain flat Escape authority"
    );
    assert!(header.node.owns_keyboard_topmost());
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("child")
        .unwrap()
        .callbacks
        .key = Some(Rc::new(|_| panic!("newer topmost Header owns this Escape")));
    let decision = child.node.dioxus_key(KeyFacts {
        key: "Escape".into(),
        on_trigger: true,
        ..Default::default()
    });
    assert!(!decision.stop && !decision.prevent);
    assert!(header.open.get() && child.open.get());
    assert!(child.node.request(false, Reason::Trigger, false));
    assert!(child.node.request(true, Reason::Trigger, false));
    let decision = child.node.dioxus_key(KeyFacts {
        key: "Escape".into(),
        on_trigger: true,
        ..Default::default()
    });
    assert!(
        decision.prevent && decision.stop,
        "handled Escape cancels shared native delivery"
    );
    // Install a callback without changing registration generation or graph.
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("child")
        .unwrap()
        .callbacks
        .key = Some(Rc::new(|_| {
        panic!("closed Escape/Tab cannot reach child policy")
    }));
    let decision = child.node.dioxus_key(KeyFacts {
        key: "Escape".into(),
        on_trigger: true,
        ..Default::default()
    });
    assert!(!decision.stop && !decision.prevent);
    let decision = child.node.dioxus_key(KeyFacts {
        key: "Tab".into(),
        on_trigger: true,
        ..Default::default()
    });
    assert!(!decision.stop && !decision.prevent);
    assert!(
        !header.open.get() && !child.open.get(),
        "current assignment routes child Escape through its enclosing Header"
    );
    header.node.unregister();
    child.node.unregister();
    let log = Rc::new(RefCell::new(Vec::new()));
    let header = model(&runtime, "header", OverlayKind::Header, log.clone());
    let child = model(&runtime, "child", OverlayKind::Dropdown, log);
    runtime.observe(&report(
        &runtime,
        3,
        "observe",
        nodes(&runtime, &["header"]),
    ));
    assert!(header.node.request(true, Reason::Trigger, false));
    assert!(child.node.request(true, Reason::Trigger, false));
    assert!(runtime.dismiss_root("child", Reason::Escape, true));
    assert!(!header.open.get() && !child.open.get());
}

#[test]
fn submenu_bound_after_parent_open_still_closes_through_its_current_parent() {
    let runtime = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(&runtime, "late-parent", OverlayKind::Dropdown, log.clone());
    let child = model(&runtime, "late-child", OverlayKind::Dropdown, log);
    assert!(parent.node.request(true, Reason::Trigger, false));
    child.node.set_ownership(vec![parent.node.clone()], false);
    let mut observed = report(
        &runtime,
        1,
        "observe",
        serde_json::json!([
            {"id":"late-parent","generation":1},
            {"id":"late-child","generation":1}
        ]),
    );
    runtime.observe(&observed);
    assert!(
        runtime
            .0
            .borrow()
            .registry
            .cached_children("late-parent")
            .is_empty()
    );
    observed.sequence = 2;
    child.node.set_ownership(vec![parent.node.clone()], true);
    runtime.observe(&observed);
    assert_eq!(
        runtime.0.borrow().registry.cached_children("late-parent"),
        ["late-child"]
    );
    assert!(child.node.request(true, Reason::Keyboard, false));
    runtime.close_submenu(child.node.id());
    assert!(parent.open.get(), "return keeps the enclosing menu open");
    assert!(
        !child.open.get(),
        "a newly bound child belongs to its open parent"
    );
}

#[test]
fn before_recursion_aba_and_forced_configuration_reject_superseded_work() {
    let runtime = RuntimeHandle::new();
    let open = Rc::new(Cell::new(true));
    let pending = Rc::new(RefCell::new(None::<NodeHandle>));
    let edit = Rc::new(Cell::new(true));
    let force_inside = Rc::new(Cell::new(false));
    let before_force = force_inside.clone();
    let read = open.clone();
    let before_node = pending.clone();
    let before_open = open.clone();
    let before_edit = edit.clone();
    let apply = open.clone();
    let applied = Rc::new(Cell::new(0));
    let count = applied.clone();
    let node = runtime.register(
        "node".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        Callbacks {
            read: Rc::new(move || Some(read.get())),
            before: Rc::new(move |_| {
                let node = before_node.borrow().clone().unwrap();
                assert!(
                    !node.request(false, Reason::Trigger, false),
                    "recursive before must be rejected"
                );
                if before_force.replace(false) {
                    assert!(node.request(false, Reason::Configuration, false));
                    return true;
                }
                if before_edit.replace(false) {
                    before_open.set(false);
                    node.sync_open(false);
                    before_open.set(true);
                    node.sync_open(true);
                }
                true
            }),
            apply: Rc::new(move |change| {
                apply.set(change.open);
                count.set(count.get() + 1);
            }),
            key: None,
        },
    );
    *pending.borrow_mut() = Some(node.clone());
    assert!(
        !node.request(false, Reason::Trigger, false),
        "ABA invalidates older command"
    );
    assert!(open.get() && applied.get() == 0);
    assert!(
        node.request(false, Reason::Trigger, false),
        "superseded before does not leave recursion guard stuck"
    );
    force_inside.set(true);
    assert!(
        !node.request(true, Reason::Trigger, false),
        "forced unchanged close supersedes pending opener"
    );
    assert!(!open.get());
    open.set(true);
    node.sync_open(true);
    assert!(
        node.request(false, Reason::Configuration, false),
        "configuration bypasses veto callback"
    );
    assert!(!open.get());

    let runtime = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(&runtime, "header", OverlayKind::Header, log.clone());
    let child_open = Rc::new(Cell::new(false));
    let read = child_open.clone();
    let apply = child_open.clone();
    let reopen = parent.node.clone();
    let child = runtime.register(
        "child".into(),
        1,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        Callbacks {
            read: Rc::new(move || Some(read.get())),
            before: Rc::new(move |change| {
                if !change.open {
                    assert!(reopen.request(true, Reason::Trigger, false));
                }
                true
            }),
            apply: Rc::new(move |change| apply.set(change.open)),
            key: None,
        },
    );
    runtime.observe(&report(
        &runtime,
        1,
        "observe",
        nodes(&runtime, &["header"]),
    ));
    assert!(parent.node.request(true, Reason::Trigger, false));
    assert!(child.request(true, Reason::Trigger, false));
    log.borrow_mut().clear();
    assert!(
        !parent.node.request(false, Reason::Trigger, false),
        "unchanged reopen from child supersedes pending parent close"
    );
    assert!(parent.open.get() && !child_open.get());
    assert!(!log.borrow().contains(&"apply:header:false".into()));
}

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
fn event(value: serde_json::Value) -> FormEvent {
    Event::new(
        Rc::new(dioxus::html::FormData::new(RawForm(value.to_string()))),
        true,
    )
}
struct TypedKeyboard(Key, Modifiers);
impl dioxus::html::ModifiersInteraction for TypedKeyboard {
    fn modifiers(&self) -> Modifiers {
        self.1
    }
}
impl dioxus::html::HasKeyboardData for TypedKeyboard {
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
pub(super) fn typed_keyboard(key: &str, modifiers: Modifiers) -> KeyboardEvent {
    let key = key.parse().unwrap_or_else(|_| Key::Character(key.into()));
    Event::new(
        Rc::new(dioxus::html::KeyboardData::new(TypedKeyboard(
            key, modifiers,
        ))),
        true,
    )
}
#[test]
fn dioxus_key_dispatch_uses_current_registration_without_native_listener_readiness() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "typed",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    root.open.set(true);
    let calls = Rc::new(Cell::new(0));
    let called = calls.clone();
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("typed")
        .unwrap()
        .callbacks
        .key = Some(Rc::new(move |_| {
        called.set(called.get() + 1);
        KeyDecision {
            prevent: true,
            activate: Some("registered-item".into()),
            ..Default::default()
        }
    }));
    let key = typed_keyboard("Enter", Modifiers::empty());
    let facts = root.node.dioxus_key_facts(&key, true, None, false).unwrap();
    assert!(facts.inventory.as_ref().unwrap().hosts.is_empty());
    let decision = root.node.dioxus_key(facts);
    assert!(decision.prevent);
    assert_eq!(decision.activate.as_deref(), Some("registered-item"));
    assert_eq!(calls.get(), 1);

    let prevented = typed_keyboard("Enter", Modifiers::empty());
    prevented.prevent_default();
    let facts = root
        .node
        .dioxus_key_facts(&prevented, true, None, false)
        .unwrap();
    assert!(!root.node.dioxus_key(facts).prevent);
    assert_eq!(calls.get(), 1);

    // Legacy native key reports cannot dispatch policy a second time.
    let native = report(
        &runtime,
        1,
        "key",
        serde_json::json!([
            {"id":"typed","generation":1,"on_trigger":true}
        ]),
    );
    runtime.observe(&native);
    assert_eq!(calls.get(), 1);
    let replacement = runtime.register(
        "typed".into(),
        2,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        runtime.entry("typed").unwrap().callbacks,
    );
    assert!(
        root.node
            .dioxus_key_facts(&key, true, None, false)
            .is_none()
    );
    assert!(
        !root
            .node
            .dioxus_key(KeyFacts {
                key: "Enter".into(),
                on_trigger: true,
                ..Default::default()
            })
            .prevent
    );
    assert_eq!(calls.get(), 1);
    replacement.unregister();
}

#[test]
fn dioxus_key_cannot_activate_after_its_callback_removes_the_owner() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "activation-owner",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    root.open.set(true);
    let called = runtime.clone();
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut(root.node.id())
        .unwrap()
        .callbacks
        .key = Some(Rc::new(move |_| {
        called.unregister("activation-owner", 1);
        KeyDecision {
            prevent: true,
            activate: Some("old-control".into()),
            ..Default::default()
        }
    }));
    let decision = root.node.dioxus_key(KeyFacts {
        key: "Enter".into(),
        inside: true,
        ..Default::default()
    });
    assert!(decision.prevent && decision.activate.is_none());
}

fn mixed_inventory_facts() -> InventoryFacts {
    serde_json::from_value(serde_json::json!({"items_host":"actual-items-host",
            "hosts":[
                {"item":"key-a","kind":"dropdown","target":"a","capture":"key-a","registration_key":"key-a"},
                {"item":"key-b","kind":"menu","target":"b","capture":"key-b","registration_key":"key-b","tabindex_attribute":"4"}
            ],"navigation_order":["key-a","key-b"],"query_order":["key-a","key-b"]})).unwrap()
}
fn describe_inventory(
    runtime: &RuntimeHandle,
    root: &str,
    generation: u64,
    facts: &InventoryFacts,
) {
    for host in &facts.hosts {
        runtime.register_owned_item_description(
            &host.registration_key,
            Some((root.into(), generation)),
            host.clone(),
            ItemOrder::default(),
        );
    }
}
fn describe_ordered_item(
    runtime: &RuntimeHandle,
    root: &str,
    key: &str,
    position: Option<usize>,
    parent: Option<&str>,
) {
    let mut host = mixed_inventory_facts().hosts.remove(0);
    host.target = format!("control-{key}");
    runtime.register_owned_item_description(
        key,
        Some((root.into(), 1)),
        host,
        ItemOrder {
            position,
            parent: parent.map(str::to_string),
        },
    );
}
fn inventory_keys(node: &NodeHandle) -> Vec<String> {
    node.inventory()
        .unwrap()
        .query_order
        .into_iter()
        .map(|key| key.0)
        .collect()
}

#[test]
fn owned_item_order_groups_children_and_tracks_current_insertions_and_removals() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "ordered",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    // A child may render/register before its containing item. Parentage,
    // rather than component mounting order, keeps their final group intact.
    describe_ordered_item(&runtime, "ordered", "child", Some(1), Some("parent"));
    describe_ordered_item(&runtime, "ordered", "tail", Some(1), None);
    describe_ordered_item(&runtime, "ordered", "parent", Some(2), None);
    describe_ordered_item(&runtime, "ordered", "static-child", None, Some("parent"));
    assert_eq!(
        inventory_keys(&root.node),
        ["tail", "parent", "child", "static-child"]
    );
    describe_ordered_item(&runtime, "ordered", "first", Some(0), None);
    assert_eq!(
        inventory_keys(&root.node),
        ["first", "tail", "parent", "child", "static-child"]
    );
    // Updates preserve registration sequence for tied/omitted siblings.
    describe_ordered_item(&runtime, "ordered", "tail", Some(0), None);
    assert_eq!(
        inventory_keys(&root.node),
        ["tail", "first", "parent", "child", "static-child"]
    );
    runtime.unregister_item_cue("parent");
    assert_eq!(
        inventory_keys(&root.node),
        ["tail", "first", "child", "static-child"],
        "removed parents leave their current children at this owner's root"
    );
    describe_ordered_item(&runtime, "ordered", "parent", Some(1), None);
    assert_eq!(
        inventory_keys(&root.node),
        ["tail", "first", "parent", "child", "static-child"]
    );
}

#[test]
fn owned_item_order_is_local_and_malformed_parent_cycles_keep_each_member_once() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "local",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    let foreign = model(
        &runtime,
        "foreign",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    describe_ordered_item(&runtime, "foreign", "foreign-parent", Some(0), None);
    describe_ordered_item(
        &runtime,
        "local",
        "local-child",
        Some(0),
        Some("foreign-parent"),
    );
    describe_ordered_item(&runtime, "local", "missing-child", Some(1), Some("missing"));
    assert_eq!(inventory_keys(&root.node), ["local-child", "missing-child"]);
    assert_eq!(inventory_keys(&foreign.node), ["foreign-parent"]);
    describe_ordered_item(&runtime, "local", "cycle-a", Some(0), Some("cycle-b"));
    describe_ordered_item(&runtime, "local", "cycle-b", Some(0), Some("cycle-a"));
    describe_ordered_item(&runtime, "local", "self-parent", None, Some("self-parent"));
    let keys = inventory_keys(&root.node);
    assert_eq!(
        keys,
        inventory_keys(&root.node),
        "malformed relationships still have deterministic order"
    );
    assert_eq!(keys.len(), 5);
    assert_eq!(
        keys.iter().collect::<BTreeSet<_>>().len(),
        5,
        "no cycle duplicates members"
    );
    assert!(
        keys.iter().any(|key| key == "cycle-a")
            && keys.iter().any(|key| key == "cycle-b")
            && keys.iter().any(|key| key == "self-parent")
    );
    assert_eq!(&keys[..2], ["local-child", "missing-child"]);
}

#[test]
fn owned_inventory_uses_registered_members_props_and_rust_order_only() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "owned-items",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    let raw = mixed_inventory_facts();
    describe_inventory(&runtime, "owned-items", 1, &raw);
    let mut other = raw.hosts[0].clone();
    other.registration_key = "other-owner".into();
    runtime.register_owned_item_description(
        "other-owner",
        Some(("foreign".into(), 1)),
        other,
        ItemOrder::default(),
    );
    runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":"owned-items","generation":1,
                "item_order":["unknown-native-control","key-b","key-b","other-owner"],
                "inventory":{"items_host":"untrusted","hosts":[{"item":"native","kind":"dropdown","target":"native","capture":"native"}],
                "navigation_order":["native"],"query_order":["native"]}}]})));
    let inventory = root.node.inventory().unwrap();
    assert_eq!(
        inventory.query_order,
        [ItemRef("key-a".into()), ItemRef("key-b".into())]
    );
    assert_eq!(inventory.hosts.len(), 2);
    assert!(inventory.slots.is_none());
    runtime.register_owned_item_description(
        "key-b",
        Some(("owned-items".into(), 1)),
        raw.hosts[1].clone(),
        ItemOrder {
            position: Some(0),
            parent: None,
        },
    );
    assert_eq!(
        root.node.inventory().unwrap().query_order,
        [ItemRef("key-b".into()), ItemRef("key-a".into())],
        "only an authored position changes current order"
    );
    let mut changed = raw.hosts[0].clone();
    changed.disabled_attribute = true;
    changed.checked_attribute = true;
    changed.target = "updated-rust-target".into();
    runtime.register_owned_item_description(
        "key-a",
        Some(("owned-items".into(), 1)),
        changed,
        ItemOrder::default(),
    );
    let inventory = root.node.inventory().unwrap();
    let updated = inventory
        .hosts
        .iter()
        .find(|host| host.registration_key == "key-a")
        .unwrap();
    assert!(updated.disabled_attribute && updated.checked_attribute);
    assert_eq!(updated.target, "updated-rust-target");
    assert_eq!(updated.item.0, "key-a");
    runtime.unregister_item_cue("key-b");
    assert_eq!(root.node.inventory().unwrap().hosts.len(), 1);
    root.node.unregister();
    assert!(root.node.inventory().is_none());
    let replacement = runtime.register(
        "owned-items".into(),
        2,
        OverlayKind::Dropdown,
        CloseBehavior::None,
        Callbacks {
            read: Rc::new(|| Some(false)),
            before: Rc::new(|_| true),
            apply: Rc::new(|_| {}),
            key: None,
        },
    );
    // An old-generation publisher cannot enroll an item in a replacement root.
    describe_inventory(&runtime, "owned-items", 1, &raw);
    assert!(replacement.inventory().unwrap().hosts.is_empty());
}

#[test]
fn description_prop_updates_preserve_completed_cue_and_identity_replacement_clears_it() {
    let runtime = RuntimeHandle::new();
    let _root = model(
        &runtime,
        "description-cue",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    let mut host = mixed_inventory_facts().hosts.remove(0);
    runtime.register_owned_item_description(
        "key-a",
        Some(("description-cue".into(), 1)),
        host.clone(),
        ItemOrder::default(),
    );
    install_item_mock(&runtime, &host, ItemMountMock::default());
    runtime.0.borrow_mut().keyboard = true;
    runtime.focus_registered_item("key-a", true, None);
    assert_eq!(runtime.item_visual_key().as_deref(), Some("key-a"));
    host.checked_attribute = true;
    host.tabindex_attribute = Some("0".into());
    runtime.register_owned_item_description(
        "key-a",
        Some(("description-cue".into(), 1)),
        host.clone(),
        ItemOrder::default(),
    );
    assert!(runtime.item_visual_matches(&host));
    assert!(
        runtime
            .0
            .borrow()
            .item_visual_host
            .as_ref()
            .unwrap()
            .checked_attribute
    );
    host.hidden_attribute = true;
    runtime.register_owned_item_description(
        "key-a",
        Some(("description-cue".into(), 1)),
        host.clone(),
        ItemOrder::default(),
    );
    assert!(
        runtime.item_visual_matches(&host),
        "inline panel closure preserves a completed cue"
    );
    host.target = "replacement-control".into();
    runtime.register_owned_item_description(
        "key-a",
        Some(("description-cue".into(), 1)),
        host,
        ItemOrder::default(),
    );
    assert_eq!(runtime.item_visual_key(), None);
}

fn document_value(
    runtime: &RuntimeHandle,
    sequence: u64,
    document: &str,
    payload: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"document":document,
            "kind":"document","document_event":payload})
}
#[test]
fn document_input_keeps_snapshots_and_clears_each_broadcast_owners_cue() {
    let first = RuntimeHandle::new();
    let second = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let root = model(&first, "document-root", OverlayKind::Dropdown, log.clone());
    let _child = model(&first, "document-child", OverlayKind::Dropdown, log.clone());
    let peer = model(&second, "document-peer", OverlayKind::Dropdown, log.clone());
    let removed = model(&first, "document-removed-surface", OverlayKind::Header, log);
    let pointers = Rc::new(RefCell::new(Vec::new()));
    let first_pointers = pointers.clone();
    let removed_node = removed.node.clone();
    root.node.set_surface_observer(Rc::new(move |observation| {
        if observation == SurfaceObservation::Pointer {
            first_pointers.borrow_mut().push("first");
            removed_node.unregister();
        }
    }));
    removed.node.set_surface_observer(Rc::new(|observation| {
        assert_ne!(
            observation,
            SurfaceObservation::Pointer,
            "an earlier observer removed this surface before its turn"
        );
    }));
    let second_pointers = pointers.clone();
    peer.node.set_surface_observer(Rc::new(move |observation| {
        if observation == SurfaceObservation::Pointer {
            second_pointers.borrow_mut().push("second");
        }
    }));
    assign(&first, "document-child", &["document-root"], true);
    let document = "document snapshot and local cue broadcast";
    let raw = mixed_inventory_facts();
    describe_inventory(&first, "document-root", 1, &raw);
    describe_inventory(&second, "document-peer", 1, &raw);
    first.input(event(serde_json::json!({"owner":first.owner_id(),"sequence":1,"document":document,"kind":"observe",
            "nodes":[{"id":"document-root","generation":1,
                },
                {"id":"document-child","generation":1}]})));
    second.input(event(serde_json::json!({"owner":second.owner_id(),"sequence":1,"document":document,"kind":"observe",
            "nodes":[{"id":"document-peer","generation":1}]})));
    let snapshots = {
        let state = first.0.borrow();
        state.item_descriptions.clone()
    };
    for runtime in [&first, &second] {
        install_item_mock(runtime, &raw.hosts[0], ItemMountMock::default());
        runtime.input(event(document_value(
            runtime,
            2,
            document,
            serde_json::json!({
                "sequence":1,"action":"input","input_kind":"keydown","key":"Tab"}),
        )));
        runtime.focus_registered_item("key-a", true, None);
        assert_eq!(runtime.item_visual_key().as_deref(), Some("key-a"));
    }
    for runtime in [&first, &second] {
        runtime.input(event(document_value(
            runtime,
            3,
            document,
            serde_json::json!({
                "sequence":2,"action":"input","input_kind":"mousedown"}),
        )));
        assert_eq!(runtime.item_visual_key(), None);
        assert!(!runtime.keyboard_mode() && !runtime.0.borrow().keyboard);
    }
    assert_eq!(
        *pointers.borrow(),
        ["first", "second"],
        "each current owner receives the same admitted document pointer input"
    );
    first.input(event(document_value(
        &first,
        4,
        document,
        serde_json::json!({
            "sequence":3,"action":"input","input_kind":"keydown","key":"Tab"}),
    )));
    first.focus_registered_item("key-a", true, None);
    first.input(event(document_value(
        &first,
        99,
        document,
        serde_json::json!({
            "sequence":2,"action":"input","input_kind":"touchstart"}),
    )));
    assert_eq!(
        first.item_visual_key().as_deref(),
        Some("key-a"),
        "older input cannot clear a newer cue"
    );
    assert_eq!(
        *pointers.borrow(),
        ["first", "second"],
        "stale shared input does not notify surface observers"
    );
    let state = first.0.borrow();
    assert_eq!(
        state.sequence, 4,
        "stale shared IDs do not consume the owner watermark"
    );
    assert_eq!(state.observation.as_ref().unwrap().sequence, 1);
    assert_eq!(snapshots, state.item_descriptions.clone());
    drop(state);
    assert_eq!(
        first
            .domain()
            .unwrap()
            .cached_parent("document-child")
            .as_deref(),
        Some("document-root")
    );
}

#[test]
fn document_auth_and_schema_rejection_cannot_consume_or_handoff_observations() {
    let runtime = RuntimeHandle::new();
    let _root = model(
        &runtime,
        "document-auth",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    let document = "document authentication";
    runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"document":document,"kind":"observe",
            "nodes":[{"id":"document-auth","generation":1}]})));
    let valid = document_value(
        &runtime,
        99,
        document,
        serde_json::json!({
            "sequence":1,"action":"input","input_kind":"keydown","key":"Tab"}),
    );
    let mut wrong_owner = valid.clone();
    wrong_owner["owner"] = "foreign-owner".into();
    let mut wrong_document = valid.clone();
    wrong_document["document"] = "another-document".into();
    let mut zero = valid.clone();
    zero["document_event"]["sequence"] = 0.into();
    let mut invalid_action = valid.clone();
    invalid_action["document_event"]["action"] = "unsupported".into();
    let mut invalid_input = valid.clone();
    invalid_input["document_event"]["input_kind"] = "scroll".into();
    let mut stale_owner = valid.clone();
    stale_owner["sequence"] = 1.into();
    for rejected in [
        wrong_owner,
        wrong_document,
        zero,
        invalid_action,
        invalid_input,
        stale_owner,
    ] {
        runtime.input(event(rejected));
        let state = runtime.0.borrow();
        assert_eq!(state.sequence, 1);
        assert_eq!(state.document, document);
        assert_eq!(state.observation.as_ref().unwrap().sequence, 1);
        assert!(state.nodes.contains_key("document-auth"));
    }
    let mut accepted = valid;
    accepted["sequence"] = 2.into();
    runtime.input(event(accepted));
    assert_eq!(runtime.document_keyboard_mode(), Some(true));
    runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":3,"document":"authorized next document","kind":"observe",
            "nodes":[{"id":"document-auth","generation":1}]})));
    runtime.input(event(document_value(
        &runtime,
        100,
        document,
        serde_json::json!({"sequence":100,"action":"escape"}),
    )));
    assert_eq!(runtime.0.borrow().sequence, 3);
    assert_eq!(runtime.0.borrow().document, "authorized next document");
}
#[derive(Clone, Default)]
struct PublicationFixture {
    runtime: Rc<RefCell<Option<RuntimeHandle>>>,
    visible: Rc<Cell<Option<Signal<bool>>>>,
}
impl PartialEq for PublicationFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.runtime, &other.runtime)
    }
}
fn publication_root(fixture: PublicationFixture) -> Element {
    let runtime = use_overlay_runtime();
    *fixture.runtime.borrow_mut() = Some(runtime.clone());
    let visible = use_signal(|| true);
    fixture.visible.set(Some(visible));
    rsx! {
        if visible() {
            crate::Dropdown { label: "Publication", suppress_trigger_visibility_check: true,
                crate::DropdownItem { id: "publication-dropdown", label: "Dropdown" }
                crate::SelectItem { id: "publication-select", value: "select", label: "Select", class: "consumer-publication-select" }
                crate::MenuItem { id: "publication-menu", label: "Menu" }
                crate::MenuCategory { id: "publication-category", label: "Category", span { "Child" } }
            }
        }
        OverlayTransport { runtime }
    }
}
fn publication_attr(tag: &str, name: &str) -> Option<String> {
    let rest = tag.split_once(&format!(" {name}="))?.1;
    if let Some(quoted) = rest.strip_prefix('"') {
        Some(quoted.split('"').next()?.to_string())
    } else {
        Some(rest.split([' ', '>']).next()?.to_string())
    }
}
fn publication_tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('>')
        .find(|tag| publication_attr(tag, "id").as_deref() == Some(id))
        .unwrap_or_else(|| panic!("Missing rendered source host {id}"))
}
fn publication_category_parent(html: &str) -> &str {
    html.split('>')
        .find(|tag| {
            publication_attr(tag, "data-ix-item-part").as_deref() == Some("category-parent")
        })
        .expect("rendered Category parent control")
}
fn publication_cues(html: &str, expected: Option<&str>, category_active: bool) {
    assert!(
        publication_attr(publication_tag(html, "publication-select"), "class").is_some_and(
            |class| class
                .split_whitespace()
                .any(|name| name == "consumer-publication-select")
        ),
        "consumer class survives Dioxus cue updates"
    );
    for id in [
        "publication-dropdown",
        "publication-select",
        "publication-menu",
    ] {
        assert_eq!(
            publication_attr(publication_tag(html, id), "data-active").as_deref(),
            Some(if expected == Some(id) {
                "true"
            } else {
                "false"
            }),
            "rendered cue on {id}, expected {expected:?}"
        );
        assert_eq!(
            publication_attr(publication_tag(html, id), "class")
                .is_some_and(|class| class.split_whitespace().any(|name| name == "ix-focused")),
            expected == Some(id),
            "Dioxus-rendered focus class on {id}"
        );
    }
    assert_eq!(
        publication_attr(publication_tag(html, "publication-category"), "data-active"),
        None,
        "Category's outer host never gains the private parent's marker"
    );
    let select_inner = html
        .split('>')
        .find(|tag| {
            publication_attr(tag, "class")
                .is_some_and(|class| class.contains("ix-select-item-option"))
        })
        .unwrap();
    assert_eq!(
        publication_attr(select_inner, "data-active").as_deref(),
        Some(if expected == Some("publication-select") {
            "true"
        } else {
            "false"
        }),
        "Select's generated inner cue follows its source host"
    );
    assert_eq!(
        publication_attr(select_inner, "class")
            .is_some_and(|class| class.split_whitespace().any(|name| name == "ix-focused")),
        expected == Some("publication-select")
    );
    let category_parent = publication_category_parent(html);
    assert_eq!(
        publication_attr(category_parent, "data-active").as_deref(),
        Some(if category_active { "true" } else { "false" })
    );
    assert_eq!(
        publication_attr(category_parent, "class")
            .is_some_and(|class| class.split_whitespace().any(|name| name == "ix-focused")),
        category_active,
        "Category cue belongs to its physical parent control"
    );
    let root = html
        .split('>')
        .find(|tag| {
            publication_attr(tag, "aria-label").as_deref() == Some("Publication")
                && publication_attr(tag, "data-ix-overlay-root").is_some()
        })
        .unwrap();
    assert_eq!(
        publication_attr(root, "aria-activedescendant").as_deref(),
        expected
    );
}
#[test]
fn actual_vdom_item_effect_publishes_mixed_host_cues_without_another_input() {
    let fixture = PublicationFixture::default();
    let mut dom = VirtualDom::new_with_props(publication_root, fixture.clone());
    dom.rebuild_in_place();
    let runtime = fixture.runtime.borrow().clone().unwrap();
    assert!(
        runtime.0.borrow().item_visual_signal.is_some(),
        "real VDOM signal path"
    );
    assert!(
        !dioxus_ssr::render(&dom).contains("data-ix-overlay-transport"),
        "transport is renderless"
    );
    let initial = dioxus_ssr::render(&dom);
    let root_tag = initial
        .split('>')
        .find(|tag| {
            publication_attr(tag, "aria-label").as_deref() == Some("Publication")
                && publication_attr(tag, "data-ix-overlay-root").is_some()
        })
        .unwrap();
    let root = publication_attr(root_tag, "data-ix-overlay-root").unwrap();
    let generation: u64 = publication_attr(root_tag, "data-ix-overlay-generation")
        .unwrap()
        .parse()
        .unwrap();
    let node = NodeHandle {
        runtime: Rc::downgrade(&runtime.0),
        id: root.clone(),
        generation,
    };
    dom.in_runtime(|| assert!(node.request(true, Reason::Trigger, false)));
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    let html = dioxus_ssr::render(&dom);
    let hosts = [
            ("publication-dropdown", "dropdown"),
            ("publication-select", "select"),
            ("publication-menu", "menu"),
            ("publication-category", "category"),
        ].into_iter().map(|(id,kind)| {
            let tag = publication_tag(&html,id);
            serde_json::json!({"item":publication_attr(tag,"data-ix-item-key").unwrap(),"capture":publication_attr(tag,"data-ix-item-key").unwrap(),
                "kind":kind,"target":id,"registration_key":publication_attr(tag,"data-ix-item-key").unwrap(),
                "source_focusable":kind != "category",
                "parts":if kind == "category" { vec![serde_json::json!({
                    "kind":"category-parent", "capture":publication_attr(publication_category_parent(&html),"id").unwrap(),
                    "source_focusable":true
                })] } else { Vec::new() }})
        }).collect::<Vec<_>>();
    publication_cues(&html, None, false);
    for host in &hosts {
        let facts: HostFacts = serde_json::from_value(host.clone()).unwrap();
        install_item_mock(&runtime, &facts, ItemMountMock::default());
        runtime.register_owned_item_description(
            &facts.registration_key,
            Some((root.clone(), generation)),
            facts.clone(),
            ItemOrder::default(),
        );
    }
    let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":1,
            "kind":"observe",
            "nodes":[{"id":root,"generation":generation,
                "mode":"active-descendant","inside":true,"on_trigger":true}]});
    dom.in_scope(ScopeId::ROOT, || runtime.test_report(report));
    let node = NodeHandle {
        runtime: Rc::downgrade(&runtime.0),
        id: root.clone(),
        generation,
    };
    let key = typed_keyboard("Home", Modifiers::empty());
    let facts = node.dioxus_key_facts(&key, true, None, false).unwrap();
    assert!(node.dioxus_key(facts).prevent);
    assert_eq!(runtime.pending_item_focus_count(), 1);
    settle_item_effects(&mut dom);
    assert_eq!(runtime.pending_item_focus_count(), 0);
    publication_cues(
        &dioxus_ssr::render(&dom),
        Some("publication-dropdown"),
        false,
    );
    // Publish through real owner effects and mounted methods, without a
    // bridge receipt or a later keyboard/focusin event.
    for id in [
        "publication-select",
        "publication-menu",
        "publication-category",
    ] {
        let facts: HostFacts = serde_json::from_value(
            hosts
                .iter()
                .find(|host| host["target"] == id)
                .unwrap()
                .clone(),
        )
        .unwrap();
        dom.in_runtime(|| assert!(node.queue_item_host_focus(&facts, Rc::new(|| {}))));
        assert_eq!(runtime.pending_item_focus_count(), 1);
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        publication_cues(
            &dioxus_ssr::render(&dom),
            (id != "publication-category").then_some(id),
            id == "publication-category",
        );
    }
    let facts: HostFacts = serde_json::from_value(hosts[1].clone()).unwrap();
    let called = Rc::new(Cell::new(false));
    let callback = called.clone();
    dom.in_runtime(|| {
        assert!(node.queue_item_host_focus(&facts, Rc::new(move || callback.set(true))))
    });
    dom.in_runtime(|| fixture.visible.get().unwrap().set(false));
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert!(!called.get());
    assert!(runtime.0.borrow().owned_item_focus.is_empty());
    assert!(!dioxus_ssr::render(&dom).contains("publication-dropdown"));
}
#[test]
fn registered_mixed_inventory_uses_latest_roving_state_before_a_render_commit() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "mixed-root",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    assert!(root.node.request(true, Reason::Trigger, false));
    assert!(root.node.set_navigation_mode("roving-tabindex"));
    let selected = root.node.clone();
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("mixed-root")
        .unwrap()
        .callbacks
        .key = Some(Rc::new(move |facts| {
        let raw = facts.inventory.clone().unwrap();
        let inventory = crate::dropdown_inventory::Inventory::new(
            crate::dropdown_inventory::InventoryMode::RovingTabindex,
            raw.clone(),
        );
        let target = inventory
            .navigate(&facts.key, inventory.active().as_ref())
            .unwrap();
        assert!(selected.select_roving(&raw, &target));
        KeyDecision {
            prevent: true,
            ..KeyDecision::default()
        }
    }));
    let raw = mixed_inventory_facts();
    for host in &raw.hosts {
        install_item_mock(&runtime, host, ItemMountMock::default());
    }
    describe_inventory(&runtime, "mixed-root", 1, &raw);
    for (sequence, key) in [(1, "End"), (2, "ArrowUp")] {
        let input = event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"observe",
                "hosts":["mixed-root"],"nodes":[{"id":"mixed-root","generation":1,
                    "mode":"roving-tabindex","inside":true}]}),
        );
        runtime.input(input);
        let key = typed_keyboard(key, Modifiers::empty());
        let facts = root.node.dioxus_key_facts(&key, false, None, true).unwrap();
        assert!(root.node.dioxus_key(facts).prevent);
    }
    assert_eq!(runtime.item_tabindex("key-a"), Some(0));
    assert_eq!(runtime.item_tabindex("key-b"), Some(-1));
    let logical = root.node.inventory().unwrap();
    assert_eq!(logical.hosts[0].tabindex_attribute.as_deref(), Some("0"));
    assert_eq!(logical.hosts[1].tabindex_attribute.as_deref(), Some("-1"));
    assert_eq!(
        raw.hosts[0].tabindex_attribute, None,
        "the authored Rust description remains unchanged"
    );
    assert_eq!(raw.hosts[1].tabindex_attribute.as_deref(), Some("4"));
    assert!(root.node.request(false, Reason::Trigger, false));
    assert_eq!(runtime.item_tabindex("key-a"), None);
    assert_eq!(runtime.item_tabindex("key-b"), None);
}

#[test]
fn registered_focus_uses_typed_live_control_and_keyboard_modality() {
    let runtime = RuntimeHandle::new();
    let raw = mixed_inventory_facts();
    let host = &raw.hosts[0];
    runtime.register_item_cue(host.registration_key.clone(), host.kind);
    runtime.attach_document("mixed-visual-runtime-test");
    runtime.input(event(
        serde_json::json!({"owner":runtime.owner_id(),"sequence":1,
            "document":"mixed-visual-runtime-test","kind":"document",
            "document_event":{"sequence":1,"action":"input","input_kind":"keydown","key":"Tab"}}),
    ));
    runtime.focus_registered_item(&host.registration_key, true, None);
    assert_eq!(
        runtime.item_visual_key(),
        None,
        "kind registration without a mounted control cannot focus"
    );
    install_item_mock(&runtime, host, ItemMountMock::default());
    runtime.focus_registered_item(&host.registration_key, true, None);
    assert_eq!(runtime.item_visual_key().as_deref(), Some("key-a"));
    let mut copy = host.clone();
    copy.capture = "another-native-capture".into();
    assert!(runtime.item_visual_matches(&copy));
    // Reusing a typed registration cannot silently change its kind.
    runtime.register_item_cue(host.registration_key.clone(), ItemKind::Select);
    copy.kind = ItemKind::Select;
    assert!(!runtime.item_visual_matches(&copy));
    runtime.focus_registered_item(&host.registration_key, false, None);
    assert_eq!(runtime.item_visual_key(), None);
    runtime.input(event(
        serde_json::json!({"owner":runtime.owner_id(),"sequence":2,
            "document":"mixed-visual-runtime-test","kind":"document",
            "document_event":{"sequence":2,"action":"input","input_kind":"mousedown"}}),
    ));
    runtime.focus_registered_item(&host.registration_key, true, None);
    assert_eq!(runtime.item_visual_key(), None);
}

#[derive(Clone, Default)]
pub(super) struct ItemMountMock {
    events: Rc<RefCell<Vec<(&'static str, ScopeId)>>>,
    pub(super) pending: Rc<Cell<bool>>,
    pub(super) waker: Rc<RefCell<Option<std::task::Waker>>>,
    fail_focus: bool,
}
impl dioxus::html::RenderedElementBacking for ItemMountMock {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let mock = self.clone();
        Box::pin(async move {
            mock.events.borrow_mut().push(("focus", current_scope_id()));
            std::future::poll_fn(|cx| {
                if mock.pending.get() {
                    *mock.waker.borrow_mut() = Some(cx.waker().clone());
                    std::task::Poll::Pending
                } else {
                    std::task::Poll::Ready(())
                }
            })
            .await;
            if mock.fail_focus {
                Err(dioxus::html::MountedError::NotSupported)
            } else {
                Ok(())
            }
        })
    }
    fn scroll_to(
        &self,
        options: dioxus::html::ScrollToOptions,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert_eq!(options.behavior, dioxus::html::ScrollBehavior::Instant);
        assert!(matches!(
            options.vertical,
            dioxus::html::ScrollLogicalPosition::Nearest
        ));
        assert!(matches!(
            options.horizontal,
            dioxus::html::ScrollLogicalPosition::Nearest
        ));
        let events = self.events.clone();
        Box::pin(async move {
            events.borrow_mut().push(("scroll", current_scope_id()));
            Ok(())
        })
    }
}
pub(super) fn settle_item_effects(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
}
pub(super) fn install_item_mock(
    runtime: &RuntimeHandle,
    host: &HostFacts,
    mock: ItemMountMock,
) -> Weak<MountedData> {
    runtime.register_item_cue(host.registration_key.clone(), host.kind);
    let node = Rc::new(MountedData::new(mock));
    let weak = Rc::downgrade(&node);
    runtime.register_item_mount(&host.registration_key, &host.target, node);
    weak
}
#[test]
fn tab_boundary_uses_registered_physical_control_order_and_skips_ineligible_hosts() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let mut raw = mixed_inventory_facts();
    raw.hosts[0].kind = ItemKind::Menu;
    raw.hosts[1].kind = ItemKind::Category;
    raw.hosts[1].source_focusable = false;
    for (item, disabled, hidden) in [
        ("custom", false, false),
        ("disabled", true, false),
        ("hidden", false, true),
        ("unregistered", false, false),
    ] {
        let mut host = raw.hosts[0].clone();
        host.item = ItemRef(item.into());
        host.target = item.into();
        host.capture = item.into();
        host.registration_key = format!("key-{item}");
        host.kind = ItemKind::NativeOptIn;
        host.roving_opt_in = true;
        host.disabled_attribute = disabled;
        host.hidden_attribute = hidden;
        raw.hosts.push(host);
    }
    let menu = ItemMountMock::default();
    let category = ItemMountMock::default();
    let custom = ItemMountMock::default();
    for (host, focus_id, mock) in [
        (&raw.hosts[0], "a-control", menu.clone()),
        (&raw.hosts[1], "b-parent", category.clone()),
        (&raw.hosts[2], "custom", custom.clone()),
        (&raw.hosts[3], "disabled", ItemMountMock::default()),
        (&raw.hosts[4], "hidden", ItemMountMock::default()),
    ] {
        runtime.register_item_cue(host.registration_key.clone(), host.kind);
        runtime.register_item_control_mount(
            &host.registration_key,
            &host.target,
            focus_id,
            Rc::new(MountedData::new(mock)),
        );
    }
    // A registered element outside the current inventory is not a target.
    runtime.register_item_mount(
        "orphan",
        "orphan",
        Rc::new(MountedData::new(ItemMountMock::default())),
    );
    describe_inventory(&runtime, &node.id, node.generation, &raw);
    dom.in_scope(fixture.0.owner.get().unwrap(), || {
        runtime.input(event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":node.id,"generation":1}]}),
        ))
    });
    let ids = [
        "unregistered",
        "disabled",
        "hidden",
        "orphan",
        "b-parent",
        "custom",
        "a-control",
    ]
    .map(str::to_string);
    assert!(node.queue_tab_boundary_focus(&ids, false, Rc::new(|| {})));
    settle_item_effects(&mut dom);
    assert_eq!(
        *category.events.borrow(),
        [("focus", owner), ("scroll", owner)]
    );
    assert!(menu.events.borrow().is_empty());
    assert!(node.queue_tab_boundary_focus(&ids, true, Rc::new(|| {})));
    settle_item_effects(&mut dom);
    assert_eq!(*menu.events.borrow(), [("focus", owner), ("scroll", owner)]);
    assert!(node.queue_tab_boundary_focus(&["custom".into()], false, Rc::new(|| {})));
    settle_item_effects(&mut dom);
    assert_eq!(
        *custom.events.borrow(),
        [("focus", owner), ("scroll", owner)]
    );
    assert!(!node.queue_tab_boundary_focus(
        &["a".into(), "b".into()],
        false,
        Rc::new(|| panic!("logical IDs are not physical candidates"))
    ));
}

#[test]
fn tab_boundary_metadata_replacement_dismissal_and_owner_drop_cancel_pending_work() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let raw = mixed_inventory_facts();
    let host = raw.hosts[0].clone();
    let mock = ItemMountMock::default();
    let mounted = Rc::new(MountedData::new(mock.clone()));
    runtime.register_item_cue(host.registration_key.clone(), host.kind);
    runtime.register_item_control_mount(
        &host.registration_key,
        &host.target,
        "old-control",
        mounted.clone(),
    );
    describe_inventory(&runtime, &node.id, node.generation, &raw);
    dom.in_scope(fixture.0.owner.get().unwrap(), || {
        runtime.input(event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":node.id,"generation":1}]}),
        ))
    });
    let captured = Rc::new(());
    let released = Rc::downgrade(&captured);
    assert!(node.queue_tab_boundary_focus(
        &["old-control".into()],
        false,
        Rc::new(move || {
            let _ = &captured;
            panic!("replaced physical metadata");
        })
    ));
    runtime.register_item_control_mount(
        &host.registration_key,
        &host.target,
        "new-control",
        mounted,
    );
    assert!(released.upgrade().is_none());
    assert!(!node.queue_tab_boundary_focus(&["old-control".into()], false, Rc::new(|| {})));
    settle_item_effects(&mut dom);
    assert!(mock.events.borrow().is_empty());
    assert!(node.queue_tab_boundary_focus(
        &["new-control".into()],
        false,
        Rc::new(|| panic!("dismissed owner"))
    ));
    assert!(node.request(false, Reason::Trigger, false));
    settle_item_effects(&mut dom);
    assert!(mock.events.borrow().is_empty());
    assert!(!node.queue_tab_boundary_focus(&["new-control".into()], false, Rc::new(|| {})));
    fixture.0.open.set(true);
    let stale = NodeHandle {
        generation: node.generation + 1,
        ..node.clone()
    };
    assert!(!stale.queue_tab_boundary_focus(&["new-control".into()], false, Rc::new(|| {})));
    assert!(node.queue_tab_boundary_focus(
        &["new-control".into()],
        false,
        Rc::new(|| panic!("removed owner"))
    ));
    node.unregister();
    settle_item_effects(&mut dom);
    assert!(mock.events.borrow().is_empty());
    assert_eq!(runtime.pending_item_focus_count(), 0);
}

#[test]
fn focus_projection_uses_physical_partitions_without_grouping_missing_host_ids() {
    let mut report: Report = serde_json::from_value(serde_json::json!({
            "owner":"fixture","sequence":1,"kind":"observe",
            "nodes":[
                {"id":"outer","generation":1,"items_host":"outer-host","popover_host":"outer-host"},
                {"id":"inner","generation":1,"items_host":"inner-host","popover_host":"inner-host"},
                {"id":"missing","generation":1}
            ],
            "focus_candidates":[
                {"target":"outer-host","tab_index":0,"nearest_host":"outer-host","popover":"outer-host"},
                {"target":"outer-zero","tab_index":0,"nearest_host":"outer-host","popover":"outer-host","contains_active":true},
                {"target":"inner-control","tab_index":0,"nearest_host":"inner-host","popover":"inner-host","exact_active":true},
                {"target":"outer-positive","tab_index":2,"nearest_host":"outer-host","popover":"outer-host"},
                {"target":"unrelated","tab_index":0}
            ]
        })).unwrap();
    // Isolate the pure projection with authored descriptors. Production
    // deserialization intentionally ignores the legacy native nodes field.
    report.nodes = vec![
        NodeFacts {
            id: "outer".into(),
            generation: 1,
            items_host: "outer-host".into(),
            popover_host: "outer-host".into(),
            ..Default::default()
        },
        NodeFacts {
            id: "inner".into(),
            generation: 1,
            items_host: "inner-host".into(),
            popover_host: "inner-host".into(),
            ..Default::default()
        },
        NodeFacts {
            id: "missing".into(),
            generation: 1,
            ..Default::default()
        },
    ];
    report.project_focus();
    assert_eq!(
        report.nodes[0].focusable_ids,
        ["outer-positive", "outer-zero"]
    );
    assert_eq!(report.nodes[1].focusable_ids, ["inner-control"]);
    assert!(report.nodes[2].focusable_ids.is_empty());
    assert!(report.nodes[2].tab_candidates.is_empty());
    assert!(report.nodes[0].tab_candidates[0].active);
    assert!(report.nodes[1].tab_candidates[0].active);
    assert_eq!(
        report
            .document_candidates
            .iter()
            .filter(|candidate| candidate.active)
            .count(),
        1
    );
    assert!(report.document_candidates[2].active);
}

#[test]
fn direct_key_facts_reproject_current_metadata_without_native_nodes_or_new_observation() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "current-surface",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    root.open.set(true);
    root.node.set_surface_metadata(SurfaceMetadata {
        items_host: "old-host".into(),
        ..Default::default()
    });
    let observed: Report = serde_json::from_value(serde_json::json!({
            "owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":"current-surface","generation":1,"items_host":"forged-host","trigger_id":"forged-trigger"}],
            "focus_candidates":[
                {"target":"old-control","tab_index":0,"nearest_host":"old-host"},
                {"target":"current-control","tab_index":0,"nearest_host":"current-host"},
                {"target":"forged-control","tab_index":0,"nearest_host":"forged-host"}
            ]
        })).unwrap();
    assert!(
        observed.nodes.is_empty(),
        "native metadata is ignored during deserialization"
    );
    runtime.observe(&observed);
    root.node.set_surface_metadata(SurfaceMetadata {
        items_host: "current-host".into(),
        ..Default::default()
    });
    let key = typed_keyboard("Tab", Modifiers::empty());
    let facts = root
        .node
        .dioxus_key_facts(&key, false, Some("current-control".into()), true)
        .unwrap();
    assert_eq!(facts.focusable_ids, ["current-control"]);
    assert_eq!(facts.focused_index, Some(0));
    assert_eq!(runtime.0.borrow().observation.as_ref().unwrap().sequence, 1);

    // A later liveness callback may update an earlier descriptor. Final
    // assembly uses the current value without rerunning those callbacks.
    let peer = model(
        &runtime,
        "later-surface",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    let update = root.node.clone();
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("later-surface")
        .unwrap()
        .callbacks
        .read = Rc::new(move || {
        update.set_surface_metadata(SurfaceMetadata {
            items_host: "latest-host".into(),
            ..Default::default()
        });
        Some(false)
    });
    let projected = runtime.project_report(observed);
    assert_eq!(
        projected
            .nodes
            .iter()
            .find(|node| node.id == root.node.id())
            .unwrap()
            .items_host,
        "latest-host"
    );
    peer.node.unregister();
}

#[test]
fn metadata_projection_rejects_owner_dropped_by_a_later_reader() {
    let first = RuntimeHandle::new();
    let first_node = model(
        &first,
        "dropped-surface",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    )
    .node;
    first_node.set_surface_metadata(SurfaceMetadata {
        trigger_id: "dropped-trigger".into(),
        ..Default::default()
    });
    first.attach_document("surface lease drop");
    let retained = Rc::new(RefCell::new(Some(first)));
    let second = RuntimeHandle::new();
    model(
        &second,
        "survivor",
        OverlayKind::Popover,
        Rc::new(RefCell::new(Vec::new())),
    );
    second.attach_document("surface lease drop");
    let release = retained.clone();
    second
        .0
        .borrow_mut()
        .nodes
        .get_mut("survivor")
        .unwrap()
        .callbacks
        .read = Rc::new(move || {
        release.borrow_mut().take();
        Some(false)
    });
    let projected = second.project_report(Report::default());
    assert!(retained.borrow().is_none());
    assert!(first_node.runtime().is_none());
    assert_eq!(
        projected
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        ["survivor"]
    );
    assert!(
        second
            .domain()
            .unwrap()
            .node_kind("dropped-surface", 1)
            .is_none(),
        "dropping an owner also removes its shared descriptor"
    );
}

#[test]
fn key_facts_derive_positive_tabindex_order_from_native_candidates() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "tab-report",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    root.open.set(true);
    root.node.set_surface_metadata(SurfaceMetadata {
        items_host: "tab-report-host".into(),
        ..Default::default()
    });
    let delivered = Rc::new(RefCell::new(Vec::new()));
    let captured = delivered.clone();
    runtime
        .0
        .borrow_mut()
        .nodes
        .get_mut("tab-report")
        .unwrap()
        .callbacks
        .key = Some(Rc::new(move |facts| {
        *captured.borrow_mut() = facts.focusable_ids.clone();
        KeyDecision::default()
    }));
    let mut report = report(
        &runtime,
        1,
        "key",
        serde_json::json!([
            {"id":"tab-report","generation":1,"items_host":"tab-report-host"}
        ]),
    );
    report.kind = "observe".into();
    report.focus_candidates = serde_json::from_value(serde_json::json!([
        {"target":"a-control","tab_index":0,"nearest_host":"tab-report-host"},
        {"target":"b-parent","tab_index":1,"nearest_host":"tab-report-host"},
        {"target":"custom","tab_index":2,"nearest_host":"tab-report-host"}
    ]))
    .unwrap();
    runtime.observe(&report);
    let keyboard = typed_keyboard("Tab", Modifiers::SHIFT);
    let facts = root
        .node
        .dioxus_key_facts(&keyboard, false, Some("custom".into()), true)
        .unwrap();
    assert_eq!(facts.focused_index, Some(1));
    assert!(facts.shift && !facts.on_trigger && facts.inside);
    root.node.dioxus_key(facts);
    assert_eq!(*delivered.borrow(), ["b-parent", "custom", "a-control"]);
}

fn moved_item_fixture() -> (
    VirtualDom,
    RuntimeHandle,
    NodeHandle,
    HostFacts,
    ItemMountMock,
) {
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let runtime = dom.in_scope(ScopeId::ROOT, RuntimeHandle::new);
    let open = Rc::new(Cell::new(true));
    let read = open.clone();
    let apply = open.clone();
    let node = dom.in_scope(ScopeId::ROOT, || {
        runtime.register(
            "moved-items".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(|_| true),
                apply: Rc::new(move |change| apply.set(change.open)),
                key: None,
            },
        )
    });
    node.set_ownership(Vec::new(), true);
    node.set_mounted();
    node.set_navigation_mode("roving-tabindex");
    let host = mixed_inventory_facts().hosts.remove(0);
    runtime.register_item_cue(host.registration_key.clone(), host.kind);
    runtime.register_owned_item_description(
        &host.registration_key,
        Some((node.id.clone(), 1)),
        host.clone(),
        ItemOrder::default(),
    );
    let mock = ItemMountMock::default();
    dom.in_scope(ScopeId::ROOT, || {
        runtime.register_owned_item_control_mount(
            &host.registration_key,
            &host.target,
            "moved-control",
            Rc::new(MountedData::new(mock.clone())),
            Some((node.id.clone(), 1)),
        )
    });
    runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),"sequence":1,"kind":"observe","active_id":"moved-control",
            "nodes":[{"id":node.id,"generation":1}]})));
    runtime.0.borrow_mut().keyboard = true;
    (dom, runtime, node, host, mock)
}
fn report_moved_item(runtime: &RuntimeHandle, node: &NodeHandle) {
    runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),"sequence":2,"kind":"observe","active_default":true,
            "moved_focus":{"key":"key-a","focus_id":"moved-control"},"nodes":[{"id":node.id,"generation":1}]})));
}
#[test]
fn registered_move_repair_uses_current_mounted_focus_and_cue_without_extra_scroll() {
    let (mut dom, runtime, node, host, mock) = moved_item_fixture();
    report_moved_item(&runtime, &node);
    assert_eq!(runtime.pending_item_focus_count(), 1);
    // A forced opening refresh is not evidence of focus movement and must
    // not cancel a pending registered-move repair merely to force delivery.
    runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),"sequence":3,"kind":"observe",
            "active_id":"another-control","focus_request":{"id":node.id(),"generation":1,"request":7}})));
    assert_eq!(runtime.pending_item_focus_count(), 1);
    // The repair's own native focus report must not retire its cue job.
    runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),"sequence":4,"kind":"observe",
            "active_id":"moved-control","focus_changed":true,"nodes":[{"id":node.id,"generation":1}]})));
    assert_eq!(runtime.pending_item_focus_count(), 1);
    settle_item_effects(&mut dom);
    assert_eq!(*mock.events.borrow(), [("focus", ScopeId::ROOT)]);
    assert!(runtime.item_visual_matches(&host));
    assert_eq!(runtime.pending_item_focus_count(), 0);
}
#[test]
fn raw_move_facts_cannot_enroll_foreign_closed_non_roving_or_retired_controls() {
    for scenario in [
        "current",
        "foreign",
        "closed",
        "active-descendant",
        "removed",
        "replaced",
    ] {
        let (mut dom, runtime, node, host, original) = moved_item_fixture();
        let replacement = ItemMountMock::default();
        match scenario {
            "closed" => {
                node.request(false, Reason::Trigger, false);
            }
            "active-descendant" => {
                node.set_navigation_mode("active-descendant");
            }
            "removed" => runtime.unregister_item_cue(&host.registration_key),
            "replaced" => dom.in_scope(ScopeId::ROOT, || {
                runtime.register_owned_item_control_mount(
                    &host.registration_key,
                    &host.target,
                    "replacement-control",
                    Rc::new(MountedData::new(replacement.clone())),
                    Some((node.id.clone(), 1)),
                );
            }),
            _ => {}
        }
        runtime.test_report(serde_json::json!({
                "owner":runtime.owner_id(),"document":runtime.owner_id(),"sequence":2,
                "kind":"observe","active_default":true,
                "moved_focus":{"key":if scenario=="foreign"{"foreign-item"}else{&host.registration_key},"focus_id":"moved-control"}
            }));
        assert_eq!(
            runtime.pending_item_focus_count(),
            usize::from(scenario == "current"),
            "only the current registered roving owner can authorize repair: {scenario}"
        );
        settle_item_effects(&mut dom);
        if scenario == "current" {
            assert_eq!(*original.events.borrow(), [("focus", ScopeId::ROOT)]);
            assert!(runtime.item_visual_matches(&host));
            continue;
        }
        assert!(
            original.events.borrow().is_empty(),
            "{scenario} cannot focus the old control"
        );
        assert!(
            replacement.events.borrow().is_empty(),
            "{scenario} cannot retarget a replacement"
        );
        assert!(
            runtime.item_visual_key().is_none(),
            "{scenario} cannot publish a focus cue"
        );
    }
}
#[test]
fn move_repair_rejects_ineligible_and_stale_destinations_and_preserves_normal_focus_jobs() {
    for scenario in [
        "hidden",
        "disabled",
        "outside",
        "keydown",
        "blur",
        "replacement",
        "closed",
    ] {
        let (mut dom, runtime, node, mut host, mock) = moved_item_fixture();
        report_moved_item(&runtime, &node);
        assert_eq!(runtime.pending_item_focus_count(), 1);
        match scenario {
                "hidden" | "disabled" => {
                    host.hidden_attribute = scenario == "hidden";
                    host.disabled_attribute = scenario == "disabled";
                    runtime.register_owned_item_description(&host.registration_key, Some((node.id.clone(),1)), host.clone(), ItemOrder::default());
                },
                "outside" => runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),"sequence":3,"kind":"observe","active_id":"outside","focus_changed":true}))),
                "keydown" => runtime.input(event(document_value(&runtime,3,&runtime.owner_id(),serde_json::json!({"sequence":1,"action":"input","input_kind":"keydown","key":"Tab"})))),
                "blur" => runtime.input(event(document_value(&runtime,3,&runtime.owner_id(),serde_json::json!({"sequence":1,"action":"blur"})))),
                "replacement" => runtime.register_owned_item_control_mount(&host.registration_key, &host.target, "moved-control", Rc::new(MountedData::new(ItemMountMock::default())), Some((node.id.clone(),1))),
                "closed" => { node.request(false, Reason::Trigger, false); },
                _ => unreachable!(),
            }
        settle_item_effects(&mut dom);
        assert!(
            mock.events.borrow().is_empty(),
            "{scenario} cannot repair old focus"
        );
        assert_eq!(runtime.pending_item_focus_count(), 0);
    }
    let (mut dom, runtime, node, host, mock) = moved_item_fixture();
    assert!(node.queue_item_host_focus(&host, Rc::new(|| {})));
    report_moved_item(&runtime, &node);
    runtime.input(event(document_value(
        &runtime,
        3,
        &runtime.owner_id(),
        serde_json::json!({"sequence":1,"action":"blur"}),
    )));
    assert_eq!(
        runtime.pending_item_focus_count(),
        1,
        "repair cancellation must preserve an ordinary key-driven focus intent"
    );
    settle_item_effects(&mut dom);
    assert_eq!(
        *mock.events.borrow(),
        [("focus", ScopeId::ROOT), ("scroll", ScopeId::ROOT)]
    );
}

#[test]
fn mounted_item_effect_uses_latest_intent_owner_focus_and_nearest_scroll_even_on_focus_error() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let caller = fixture.0.caller.get().unwrap();
    let hosts = mixed_inventory_facts().hosts;
    let first = ItemMountMock::default();
    let second = ItemMountMock {
        fail_focus: true,
        ..Default::default()
    };
    install_item_mock(&runtime, &hosts[0], first.clone());
    install_item_mock(&runtime, &hosts[1], second.clone());
    runtime.0.borrow_mut().keyboard = true;
    let released = Rc::new(());
    let weak_callback = Rc::downgrade(&released);
    dom.in_scope(caller, || {
        assert!(node.queue_item_host_focus(
            &hosts[0],
            Rc::new(move || {
                let _ = &released;
                panic!("superseded");
            })
        ));
        assert!(node.queue_item_host_focus(
            &hosts[1],
            Rc::new(move || {
                assert_eq!(current_scope_id(), owner);
                assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
            })
        ));
    });
    assert!(
        weak_callback.upgrade().is_none(),
        "latest intent releases superseded callbacks"
    );
    assert_eq!(
        runtime.pending_item_focus_count(),
        1,
        "latest mounted intent is the only pending operation"
    );
    assert!(
        second.events.borrow().is_empty(),
        "admission is asynchronous"
    );
    dom.in_scope(ScopeId::APP, || {
        fixture.0.show_caller.get().unwrap().set(false)
    });
    settle_item_effects(&mut dom);
    assert!(fixture.0.caller_dropped.get());
    assert!(first.events.borrow().is_empty());
    assert_eq!(
        *second.events.borrow(),
        [("focus", owner), ("scroll", owner)]
    );
    assert_eq!(runtime.item_visual_key().as_deref(), Some("key-b"));
    assert!(runtime.0.borrow().owned_item_focus.is_empty());
    assert!(node.queue_item_focus(hosts[0].target.clone(), false, Rc::new(|| {})));
    settle_item_effects(&mut dom);
    assert_eq!(
        *first.events.borrow(),
        [("scroll", owner)],
        "active descendant navigation can scroll without native focus"
    );
}
#[test]
fn mounted_item_unmount_replacement_close_and_generation_cancel_pending_work() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let host = mixed_inventory_facts().hosts[0].clone();
    let pending = ItemMountMock::default();
    pending.pending.set(true);
    let weak_mount = install_item_mock(&runtime, &host, pending.clone());
    assert!(node.queue_item_host_focus(&host, Rc::new(|| {})));
    settle_item_effects(&mut dom);
    assert_eq!(pending.events.borrow().len(), 1);
    runtime.unregister_item_cue(&host.registration_key);
    assert!(
        weak_mount.upgrade().is_none(),
        "unmount releases the handle and suspended future"
    );
    pending.pending.set(false);
    pending.waker.borrow_mut().take().unwrap().wake();
    settle_item_effects(&mut dom);
    assert_eq!(
        pending.events.borrow().len(),
        1,
        "unmounted item never scrolls"
    );
    let old = ItemMountMock::default();
    let weak_old = install_item_mock(&runtime, &host, old.clone());
    let retained = Rc::new(());
    let released = Rc::downgrade(&retained);
    assert!(node.queue_item_host_focus(
        &host,
        Rc::new(move || {
            let _ = &retained;
            panic!("replaced mount");
        })
    ));
    let replacement = ItemMountMock::default();
    install_item_mock(&runtime, &host, replacement.clone());
    assert!(weak_old.upgrade().is_none());
    assert!(released.upgrade().is_none());
    settle_item_effects(&mut dom);
    assert!(old.events.borrow().is_empty());
    assert!(replacement.events.borrow().is_empty());
    assert!(node.queue_item_host_focus(&host, Rc::new(|| panic!("dismissed"))));
    assert!(node.request(false, Reason::Trigger, false));
    settle_item_effects(&mut dom);
    assert!(replacement.events.borrow().is_empty());
    assert!(!node.queue_item_host_focus(&host, Rc::new(|| panic!("closed"))));
    fixture.0.open.set(true);
    assert!(node.queue_item_host_focus(&host, Rc::new(|| panic!("old generation"))));
    dom.in_scope(fixture.0.owner.get().unwrap(), || {
        runtime.register(
            node.id.clone(),
            2,
            OverlayKind::Popover,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(|| Some(true)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        );
    });
    settle_item_effects(&mut dom);
    assert!(replacement.events.borrow().is_empty());
    assert!(runtime.0.borrow().owned_item_focus.is_empty());
}
#[test]
fn mounted_item_reentrant_new_intent_cannot_focus_or_publish_the_old_item() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let hosts = mixed_inventory_facts().hosts;
    let first = ItemMountMock::default();
    let second = ItemMountMock::default();
    install_item_mock(&runtime, &hosts[0], first.clone());
    install_item_mock(&runtime, &hosts[1], second.clone());
    runtime.0.borrow_mut().keyboard = true;
    let nested = node.clone();
    let target = hosts[1].clone();
    assert!(node.queue_item_host_focus(
        &hosts[0],
        Rc::new(move || {
            assert!(nested.queue_item_host_focus(&target, Rc::new(|| {})));
        })
    ));
    settle_item_effects(&mut dom);
    assert!(first.events.borrow().is_empty());
    assert_eq!(second.events.borrow().len(), 2);
    assert_eq!(runtime.item_visual_key().as_deref(), Some("key-b"));
}
#[test]
fn item_focus_requires_explicit_mount_registration_and_rejection_keeps_the_current_intent() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let released = Rc::new(());
    let weak = Rc::downgrade(&released);
    assert!(!node.queue_item_focus(
        "unregistered".into(),
        true,
        Rc::new(move || {
            let _ = &released;
            panic!("unregistered");
        })
    ));
    assert!(weak.upgrade().is_none());
    assert_eq!(runtime.pending_item_focus_count(), 0);
    let mock = ItemMountMock::default();
    let mut host = mixed_inventory_facts().hosts[0].clone();
    install_item_mock(&runtime, &host, mock.clone());
    // An explicitly registered consumer control resolves by target even
    // when the reported native host has no generated registration key.
    host.registration_key.clear();
    assert!(node.queue_item_host_focus(&host, Rc::new(|| {})));
    assert!(!node.queue_item_focus(
        "unregistered".into(),
        true,
        Rc::new(|| panic!("unregistered"))
    ));
    assert_eq!(runtime.pending_item_focus_count(), 1);
    settle_item_effects(&mut dom);
    assert_eq!(mock.events.borrow().len(), 2);
    assert_eq!(runtime.pending_item_focus_count(), 0);
    runtime.unregister_item_cue("key-a");
    assert!(!node.queue_item_host_focus(&host, Rc::new(|| panic!("unmounted"))));
}

#[test]
fn completion_effect_wakes_an_idle_owner_without_another_state_change() {
    use std::{
        future::Future,
        pin::pin,
        task::{Context, Poll, Waker},
    };
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    settle_item_effects(&mut dom);
    let node = fixture.0.node.borrow().clone().unwrap();
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(
        pin!(dom.wait_for_work()).poll(&mut context),
        Poll::Pending
    ));
    let delivered = Rc::new(Cell::new(false));
    let callback = delivered.clone();
    let owner = fixture.0.owner.get().unwrap();
    assert!(node.queue_completion(Rc::new(move || {
        assert_eq!(current_scope_id(), owner);
        callback.set(true);
    })));
    assert!(!delivered.get(), "completion stays asynchronous");
    assert!(
        matches!(
            pin!(dom.wait_for_work()).poll(&mut context),
            Poll::Ready(())
        ),
        "admission itself requests renderer work"
    );
    assert!(
        !delivered.get(),
        "the renderer has not flushed the queued effect yet"
    );
    dom.render_immediate_to_vec();
    // Pinned Dioxus drains post-render effects on its following idle
    // scheduler pass, rather than at the end of render_immediate itself.
    assert!(matches!(
        pin!(dom.wait_for_work()).poll(&mut context),
        Poll::Pending
    ));
    assert!(delivered.get());
}
#[test]
fn completion_effects_restore_registered_owner_and_allow_reentrant_writes() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let caller = fixture.0.caller.get().unwrap();
    let delivered = Rc::new(RefCell::new(Vec::new()));
    let first_log = delivered.clone();
    let nested_node = node.clone();
    dom.in_scope(caller, || {
        assert!(node.queue_completion(Rc::new(move || {
            assert_eq!(current_scope_id(), owner);
            assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
            first_log.borrow_mut().push(1);
            let nested_log = first_log.clone();
            assert!(nested_node.queue_completion(Rc::new(move || {
                assert_eq!(current_scope_id(), owner);
                nested_log.borrow_mut().push(3);
            })));
        })));
        let second_log = delivered.clone();
        assert!(node.queue_completion(Rc::new(move || {
            assert_eq!(current_scope_id(), owner);
            second_log.borrow_mut().push(2);
        })));
    });
    assert!(
        delivered.borrow().is_empty(),
        "request admission never delivers synchronously"
    );
    assert_eq!(runtime.pending_completions(), 2);
    dom.in_scope(ScopeId::APP, || {
        let mut show = fixture.0.show_caller.get().unwrap();
        show.set(false);
    });
    for _ in 0..3 {
        dom.render_immediate_to_vec();
    }
    assert!(fixture.0.caller_dropped.get());
    assert_eq!(*delivered.borrow(), [1, 2, 3]);
    assert_eq!(runtime.pending_completions(), 0);
    let dropped_log = delivered.clone();
    assert!(node.queue_completion(Rc::new(move || dropped_log.borrow_mut().push(4))));
    assert_eq!(runtime.pending_completions(), 1);
    drop(dom);
    assert_eq!(runtime.pending_completions(), 0);
    assert_eq!(*delivered.borrow(), [1, 2, 3]);
    assert!(!node.queue_completion(Rc::new(|| panic!("dropped owner"))));
}

#[test]
fn completion_effect_cannot_retarget_replaced_registration() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let old = fixture.0.node.borrow().clone().unwrap();
    let runtime = old.runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let delivered = Rc::new(Cell::new(0));
    let first = delivered.clone();
    assert!(old.queue_completion(Rc::new(move || first.set(first.get() + 1))));
    let replacement = dom.in_scope(owner, || {
        runtime.register(
            old.id.clone(),
            2,
            OverlayKind::Popover,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(|| Some(false)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        )
    });
    assert_eq!(runtime.pending_completions(), 0);
    assert!(!old.queue_completion(Rc::new(|| panic!("old generation"))));
    let current = delivered.clone();
    assert!(replacement.queue_completion(Rc::new(move || current.set(current.get() + 10))));
    assert_eq!(delivered.get(), 0);
    dom.render_immediate_to_vec();
    dom.process_events();
    assert_eq!(delivered.get(), 10);
    assert_eq!(runtime.pending_completions(), 0);
    replacement.unregister();
}
#[test]
fn outside_tab_mounted_exit_runs_after_close_without_item_scroll_or_cue() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let destination = ItemMountMock::default();
    runtime.register_item_mount(
        "outside-after",
        "after",
        Rc::new(MountedData::new(destination.clone())),
    );
    let snapshot = runtime.mounted_focus_target("after").unwrap();
    assert!(node.request(false, Reason::Keyboard, false));
    assert!(runtime.queue_exit_focus(node.id(), node.generation, snapshot, Vec::new()));
    assert!(destination.events.borrow().is_empty());
    settle_item_effects(&mut dom);
    assert_eq!(*destination.events.borrow(), [("focus", owner)]);
    assert_eq!(runtime.item_visual_key(), None);
    assert_eq!(runtime.pending_item_focus_count(), 0);
    assert!(runtime.mounted_focus_target("unregistered").is_none());
    node.unregister();
    assert!(runtime.0.borrow().nodes.is_empty());
}
#[test]
fn mounted_focus_capture_survives_initial_document_binding_but_not_document_change() {
    let runtime = RuntimeHandle::new();
    let mounted = Rc::new(MountedData::new(ItemMountMock::default()));
    runtime.register_item_control_mount("invoker", "invoker", "invoker", mounted.clone());
    let initial = runtime.capture_registered_focus("invoker").unwrap();
    runtime.attach_document("initial typed document");
    assert!(Rc::ptr_eq(&initial.live_node().unwrap(), &mounted));
    let bound = runtime.capture_registered_focus("invoker").unwrap();
    runtime.attach_document("different document");
    assert!(initial.live_node().is_none());
    assert!(bound.live_node().is_none());
    assert!(runtime.capture_registered_focus("invoker").is_some());
}
#[test]
fn runtime_capture_uses_registered_physical_ids_and_keeps_cleanup_identity_after_removal() {
    let runtime = RuntimeHandle::new();
    let mounted = Rc::new(MountedData::new(ItemMountMock::default()));
    runtime.register_item_control_mount(
        "capture-key",
        "logical-host",
        "physical-control",
        mounted.clone(),
    );
    assert!(runtime.capture_registered_focus("unregistered").is_none());
    assert!(runtime.capture_registered_focus("logical-host").is_none());
    let captured = runtime
        .capture_registered_focus("physical-control")
        .unwrap();
    assert!(captured.matches_node(&mounted));
    assert!(Rc::ptr_eq(&captured.live_node().unwrap(), &mounted));
    runtime.unregister_item_cue("capture-key");
    assert!(
        runtime
            .capture_registered_focus("physical-control")
            .is_none()
    );
    assert!(captured.live_node().is_none());
    assert!(
        captured.matches_node(&mounted),
        "cleanup still identifies the removed invoker"
    );
    let replacement = Rc::new(MountedData::new(ItemMountMock::default()));
    runtime.register_item_control_mount(
        "capture-key",
        "logical-host",
        "physical-control",
        replacement.clone(),
    );
    assert!(!captured.matches_node(&replacement));
    assert!(captured.live_node().is_none());
    assert!(
        runtime
            .capture_registered_focus("physical-control")
            .unwrap()
            .matches_node(&replacement)
    );
}
#[test]
fn accepted_close_cancels_prior_focus_but_preserves_return_admitted_by_apply() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let owner = fixture.0.owner.get().unwrap();
    let old = ItemMountMock::default();
    let returned = ItemMountMock::default();
    runtime.register_item_mount(
        "close-old",
        "close-old",
        Rc::new(MountedData::new(old.clone())),
    );
    runtime.register_item_mount(
        "close-return",
        "close-return",
        Rc::new(MountedData::new(returned.clone())),
    );
    let return_target = node.capture_registered_focus("close-return").unwrap();
    dom.in_scope(owner, || {
        let mut callbacks = runtime.entry(node.id()).unwrap().callbacks;
        let apply = callbacks.apply.clone();
        let closing = node.clone();
        callbacks.apply = Rc::new(move |change| {
            apply(change);
            if !change.open {
                assert!(closing.queue_registered_focus(return_target.clone(), Some(false)));
            }
        });
        runtime.register(
            node.id().into(),
            node.generation,
            OverlayKind::Popover,
            CloseBehavior::Both,
            callbacks,
        );
    });
    assert!(node.queue_registered_focus(
        node.capture_registered_focus("close-old").unwrap(),
        Some(true)
    ));
    assert!(node.request(false, Reason::Trigger, true));
    assert_eq!(
        runtime.pending_item_focus_count(),
        1,
        "the new close return remains admitted"
    );
    settle_item_effects(&mut dom);
    assert!(old.events.borrow().is_empty());
    assert_eq!(*returned.events.borrow(), [("focus", owner)]);
    assert_eq!(runtime.pending_item_focus_count(), 0);
}
#[test]
fn registered_focus_respects_expected_visibility_and_explicit_cancellation() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let mock = ItemMountMock::default();
    runtime.register_item_mount(
        "registered-control",
        "control",
        Rc::new(MountedData::new(mock.clone())),
    );
    let captured = node.capture_registered_focus("control").unwrap();
    assert!(!node.queue_registered_focus(captured.clone(), Some(false)));
    assert!(node.queue_registered_focus(captured.clone(), Some(true)));
    node.cancel_registered_focus();
    settle_item_effects(&mut dom);
    assert!(mock.events.borrow().is_empty());
    assert!(node.request(false, Reason::Keyboard, false));
    assert!(!node.queue_registered_focus(captured.clone(), Some(true)));
    assert!(node.queue_registered_focus(captured.clone(), Some(false)));
    settle_item_effects(&mut dom);
    assert_eq!(
        *mock.events.borrow(),
        [("focus", fixture.0.owner.get().unwrap())]
    );
    mock.events.borrow_mut().clear();
    assert!(node.queue_registered_focus(captured, Some(false)));
    fixture.0.open.set(true);
    settle_item_effects(&mut dom);
    assert!(
        mock.events.borrow().is_empty(),
        "changed owner visibility rejects stale focus"
    );
    assert!(node.capture_registered_focus("unregistered").is_none());
}
#[test]
fn outside_tab_capture_cannot_follow_a_replacement_or_retain_unmounted_destination() {
    let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
    let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
    dom.rebuild_in_place();
    let node = fixture.0.node.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    let old = Rc::new(MountedData::new(ItemMountMock::default()));
    let weak_old = Rc::downgrade(&old);
    runtime.register_item_mount("outside-after", "after", old);
    let snapshot = runtime.mounted_focus_target("after").unwrap();
    assert!(node.request(false, Reason::Keyboard, false));
    let replacement = ItemMountMock::default();
    runtime.register_item_mount(
        "outside-after",
        "after",
        Rc::new(MountedData::new(replacement.clone())),
    );
    assert!(
        weak_old.upgrade().is_none(),
        "the captured destination is weak"
    );
    assert!(!runtime.queue_exit_focus(node.id(), node.generation, snapshot, Vec::new()));
    let current = runtime.mounted_focus_target("after").unwrap();
    assert!(runtime.queue_exit_focus(node.id(), node.generation, current, Vec::new()));
    runtime.unregister_item_cue("outside-after");
    settle_item_effects(&mut dom);
    assert!(replacement.events.borrow().is_empty());
    assert_eq!(runtime.pending_item_focus_count(), 0);
}
#[derive(Clone, Default)]
struct RovingView(Rc<RovingViewState>);
#[derive(Default)]
struct RovingViewState {
    runtime: RefCell<Option<RuntimeHandle>>,
    node: RefCell<Option<NodeHandle>>,
    authored: Cell<Option<Signal<i32>>>,
    open: Cell<bool>,
}
fn roving_view(fixture: RovingView) -> Element {
    let runtime = use_hook(RuntimeHandle::new);
    let authored = use_signal(|| 7);
    fixture.0.authored.set(Some(authored));
    *fixture.0.runtime.borrow_mut() = Some(runtime.clone());
    let registered = fixture.clone();
    let registry = runtime.clone();
    let node = use_hook(move || {
        registered.0.open.set(true);
        let read = registered.clone();
        let apply = registered.clone();
        let node = registry.register(
            "reactive-roving".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.0.open.get())),
                before: Rc::new(|_| true),
                apply: Rc::new(move |change| apply.0.open.set(change.open)),
                key: None,
            },
        );
        for host in &mixed_inventory_facts().hosts {
            install_item_mock(&registry, host, ItemMountMock::default());
        }
        *registered.0.node.borrow_mut() = Some(node.clone());
        node
    });
    let cleanup = runtime.clone();
    use_drop(move || {
        node.unregister();
        cleanup.unregister_item_cue("key-a");
        cleanup.unregister_item_cue("key-b");
    });
    let first = runtime.item_tabindex("key-a").unwrap_or(*authored.read());
    let second = runtime.item_tabindex("key-b").unwrap_or(4);
    rsx! { button { id: "a", tabindex: first, "A" } button { id: "b", tabindex: second, "B" } }
}
fn roving_rendered_tabindex(dom: &VirtualDom, id: &str) -> String {
    let html = dioxus_ssr::render(dom);
    publication_attr(publication_tag(&html, id), "tabindex").unwrap()
}
#[test]
fn roving_revision_renders_overrides_then_restores_current_authored_props() {
    let fixture = RovingView::default();
    let mut dom = VirtualDom::new_with_props(roving_view, fixture.clone());
    dom.rebuild_in_place();
    let runtime = fixture.0.runtime.borrow().clone().unwrap();
    let node = fixture.0.node.borrow().clone().unwrap();
    let raw = mixed_inventory_facts();
    assert_eq!(roving_rendered_tabindex(&dom, "a"), "7");
    assert_eq!(roving_rendered_tabindex(&dom, "b"), "4");
    assert_eq!(node.init_roving(&raw, true, false).unwrap().target, "b");
    assert_eq!(runtime.item_tabindex("key-b"), Some(0));
    assert_eq!(
        roving_rendered_tabindex(&dom, "b"),
        "4",
        "logical state precedes the Dioxus commit"
    );
    settle_item_effects(&mut dom);
    assert_eq!(roving_rendered_tabindex(&dom, "a"), "-1");
    assert_eq!(roving_rendered_tabindex(&dom, "b"), "0");
    dom.in_scope(ScopeId::APP, || fixture.0.authored.get().unwrap().set(9));
    settle_item_effects(&mut dom);
    assert_eq!(roving_rendered_tabindex(&dom, "a"), "-1");
    node.clear_roving();
    settle_item_effects(&mut dom);
    assert_eq!(
        roving_rendered_tabindex(&dom, "a"),
        "9",
        "clear uses the current authored prop rather than a raw baseline"
    );
    assert_eq!(roving_rendered_tabindex(&dom, "b"), "4");
    assert!(node.select_roving(&raw, &ItemRef("key-a".into())));
    assert!(node.request(false, Reason::Trigger, false));
    settle_item_effects(&mut dom);
    assert_eq!(roving_rendered_tabindex(&dom, "a"), "9");
    assert_eq!(runtime.item_tabindex("key-b"), None);
}
#[test]
fn roving_initialization_skips_disabled_and_unregistered_items_and_supports_custom_mounts() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "roving-root",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    root.open.set(true);
    let mut raw = mixed_inventory_facts();
    for host in &raw.hosts {
        install_item_mock(&runtime, host, ItemMountMock::default());
    }
    let mut disabled = raw.hosts[0].clone();
    disabled.item = ItemRef("disabled".into());
    disabled.capture = "disabled".into();
    disabled.target = "disabled".into();
    disabled.registration_key = "disabled-key".into();
    disabled.disabled_attribute = true;
    install_item_mock(&runtime, &disabled, ItemMountMock::default());
    let mut custom = disabled.clone();
    custom.item = ItemRef("custom".into());
    custom.capture = "custom".into();
    custom.target = "custom".into();
    custom.registration_key.clear();
    custom.kind = ItemKind::NativeOptIn;
    custom.disabled_attribute = false;
    custom.roving_opt_in = true;
    custom.checked_attribute = true;
    runtime.register_item_mount(
        "custom-key",
        "custom",
        Rc::new(MountedData::new(ItemMountMock::default())),
    );
    let mut unregistered = custom.clone();
    unregistered.item = ItemRef("unregistered".into());
    unregistered.target = "unregistered".into();
    unregistered.capture = "unregistered".into();
    raw.hosts.extend([disabled, custom, unregistered]);
    raw.query_order = raw.hosts.iter().map(|host| host.item.clone()).collect();
    raw.navigation_order = raw.query_order.clone();
    assert_eq!(
        root.node.init_roving(&raw, false, true).unwrap().target,
        "custom"
    );
    assert_eq!(runtime.item_tabindex("custom-key"), Some(0));
    assert_eq!(runtime.item_tabindex("key-a"), Some(-1));
    assert_eq!(runtime.item_tabindex("disabled-key"), None);
    assert!(!root.node.select_roving(&raw, &ItemRef("disabled".into())));
    assert!(
        !root
            .node
            .select_roving(&raw, &ItemRef("unregistered".into()))
    );
    assert_eq!(runtime.item_tabindex("custom-key"), Some(0));
}
#[test]
fn roving_owner_layers_survive_other_owner_clear_and_vetoed_nested_close() {
    let runtime = RuntimeHandle::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let parent = model(
        &runtime,
        "parent-roving",
        OverlayKind::Dropdown,
        log.clone(),
    );
    let child = model(&runtime, "child-roving", OverlayKind::Dropdown, log.clone());
    child.node.set_ownership(vec![parent.node.clone()], true);
    let raw = mixed_inventory_facts();
    for host in &raw.hosts {
        install_item_mock(&runtime, host, ItemMountMock::default());
    }
    runtime.input(event(
        serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":[
            {"id":"parent-roving","generation":1},
            {"id":"child-roving","generation":1}]}),
    ));
    assert!(parent.node.request(true, Reason::Trigger, false));
    assert!(child.node.request(true, Reason::Trigger, false));
    assert_eq!(
        runtime.0.borrow().registry.cached_children("parent-roving"),
        ["child-roving"],
        "the fixture must establish trigger discovery before exercising the vetoed nested close"
    );
    assert!(child.node.select_roving(&raw, &ItemRef("key-a".into())));
    assert!(parent.node.select_roving(&raw, &ItemRef("key-b".into())));
    assert_eq!(runtime.item_tabindex("key-a"), Some(-1));
    child.veto.set(true);
    log.borrow_mut().clear();
    assert!(parent.node.request(false, Reason::Trigger, false));
    assert!(!parent.open.get() && child.open.get());
    assert_eq!(
        *log.borrow(),
        [
            "before:parent-roving:false",
            "before:child-roving:false",
            "apply:parent-roving:false"
        ]
    );
    assert_eq!(
        runtime.item_tabindex("key-a"),
        Some(0),
        "clearing the parent reveals the still-open child's own state"
    );
    assert_eq!(runtime.item_tabindex("key-b"), Some(-1));
    child.node.clear_roving();
    assert_eq!(runtime.item_tabindex("key-a"), None);
    assert_eq!(runtime.item_tabindex("key-b"), None);
}
#[test]
fn roving_mode_generation_mount_replacement_and_drop_release_owned_overrides() {
    let runtime = RuntimeHandle::new();
    let root = model(
        &runtime,
        "owned-roving",
        OverlayKind::Dropdown,
        Rc::new(RefCell::new(Vec::new())),
    );
    root.open.set(true);
    let raw = mixed_inventory_facts();
    for host in &raw.hosts {
        install_item_mock(&runtime, host, ItemMountMock::default());
    }
    assert!(root.node.set_navigation_mode("roving-tabindex"));
    assert!(root.node.init_roving(&raw, false, false).is_some());
    assert!(root.node.set_navigation_mode("active-descendant"));
    assert_eq!(runtime.item_tabindex("key-a"), None);
    assert!(root.node.set_navigation_mode("roving-tabindex"));
    assert!(root.node.select_roving(&raw, &ItemRef("key-a".into())));
    install_item_mock(&runtime, &raw.hosts[0], ItemMountMock::default());
    assert_eq!(
        runtime.item_tabindex("key-a"),
        None,
        "a replacement mount cannot inherit the previous handle's override"
    );
    assert_eq!(runtime.item_tabindex("key-b"), Some(-1));
    runtime.unregister_item_cue("key-b");
    assert_eq!(runtime.item_tabindex("key-b"), None);
    install_item_mock(&runtime, &raw.hosts[1], ItemMountMock::default());
    assert!(root.node.init_roving(&raw, false, false).is_some());
    let callbacks = runtime.entry("owned-roving").unwrap().callbacks;
    let replacement = runtime.register(
        "owned-roving".into(),
        2,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        callbacks,
    );
    assert_eq!(runtime.item_tabindex("key-a"), None);
    assert!(replacement.init_roving(&raw, true, false).is_some());
    root.node.clear_roving();
    assert_eq!(
        runtime.item_tabindex("key-b"),
        Some(0),
        "the stale generation cannot clear its replacement"
    );
    replacement.unregister();
    assert_eq!(runtime.item_tabindex("key-a"), None);
    assert_eq!(runtime.item_tabindex("key-b"), None);
    assert!(runtime.0.borrow().item_tabindices.is_empty());
}
