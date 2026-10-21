use super::*;
use std::cell::RefCell;

#[test]
fn backdrop_coordinates_keep_the_panel_edges_inside() {
    assert!(point_in_bounds(10.0, 20.0, 10.0, 20.0, 100.0, 80.0));
    assert!(point_in_bounds(110.0, 100.0, 10.0, 20.0, 100.0, 80.0));
    assert!(!point_in_bounds(9.9, 50.0, 10.0, 20.0, 100.0, 80.0));
    assert!(!point_in_bounds(50.0, 100.1, 10.0, 20.0, 100.0, 80.0));
}

fn rendering_fixture() -> Element {
    let modal = use_modal();
    let popover = use_popover();
    rsx! {
        ModalTrigger { modal, "Open modal" }
        Modal { modal, label: "Confirm resource creation",
            ModalHeader { "Create Resource" }
            ModalContent { "Resource details" }
            ModalFooter { ModalClose { value: "created", "Create" } }
        }
        PopoverTrigger { popover, "Open popover" }
        Popover { popover, label: "Release highlights", close_on_click_outside: true,
            PopoverHeader { "Release highlights" }
            PopoverContent { "Release details" }
        }
    }
}

#[derive(Clone, Default)]
struct MetadataFixture(Rc<MetadataFixtureState>);
#[derive(Default)]
struct MetadataFixtureState {
    popover: std::cell::Cell<Option<PopoverHandle>>,
    anchored: std::cell::Cell<bool>,
    visible: std::cell::Cell<Option<Signal<bool>>>,
    id: RefCell<Option<Signal<String>>>,
    mounted: RefCell<Option<EventHandler<MountedEvent>>>,
}
impl PartialEq for MetadataFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn MetadataTrigger(popover: PopoverHandle, id: String, fixture: MetadataFixture) -> Element {
    let anchored = fixture.0.anchored.get();
    let binding = use_popover_trigger_registration(popover, id.clone(), anchored);
    let mounted = binding.onmounted;
    let style = anchored.then(|| binding.style());
    *fixture.0.mounted.borrow_mut() = Some(mounted);
    rsx! { button { id, style, onmounted:mounted, "Custom trigger" } }
}
fn metadata_fixture(fixture: MetadataFixture) -> Element {
    let popover = use_popover();
    let visible = use_signal(|| true);
    let id = use_signal(|| "custom-popover-trigger".to_string());
    fixture.0.popover.set(Some(popover));
    fixture.0.visible.set(Some(visible));
    *fixture.0.id.borrow_mut() = Some(id);
    rsx! {
        if visible() { MetadataTrigger {key:"{id}",popover, id:id(), fixture:fixture.clone()} }
        Popover {popover,label:"Metadata panel", "Panel content"}
    }
}
#[test]
fn popover_surface_metadata_follows_registered_trigger_id_and_cleanup() {
    let fixture = MetadataFixture::default();
    let mut dom = VirtualDom::new_with_props(metadata_fixture, fixture.clone());
    dom.rebuild_in_place();
    let popover = fixture.0.popover.get().unwrap();
    let node = popover.overlay.peek().clone().unwrap();
    let metadata = node.surface_metadata().unwrap();
    assert!(metadata.trigger_id.is_empty());
    assert_eq!(metadata.items_host, popover.id());
    assert_eq!(metadata.popover_host, popover.id());
    let mounted = fixture.0.mounted.borrow().unwrap();
    dom.in_scope(ScopeId::ROOT, || {
        mounted.call(Event::new(Rc::new(MountedData::new(())), false))
    });
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert_eq!(
        node.surface_metadata().unwrap().trigger_id,
        "custom-popover-trigger"
    );
    dom.in_scope(ScopeId::ROOT, || {
        fixture
            .0
            .id
            .borrow()
            .unwrap()
            .set("renamed-popover-trigger".into())
    });
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert!(
        node.surface_metadata().unwrap().trigger_id.is_empty(),
        "a replacement must mount before supplying a destination"
    );
    let replacement = fixture.0.mounted.borrow().unwrap();
    dom.in_scope(ScopeId::ROOT, || {
        replacement.call(Event::new(Rc::new(MountedData::new(())), false))
    });
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert_eq!(
        node.surface_metadata().unwrap().trigger_id,
        "renamed-popover-trigger"
    );
    assert!(
        popover
            .surface
            .invoker
            .peek()
            .as_ref()
            .unwrap()
            .live_node()
            .is_some()
    );
    dom.in_scope(ScopeId::ROOT, || {
        fixture.0.visible.get().unwrap().set(false)
    });
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert!(node.surface_metadata().unwrap().trigger_id.is_empty());
}

#[test]
fn popover_anchor_registration_restores_viewport_fallbacks_and_detach_centers_again() {
    struct AnchorMount;
    impl dioxus::html::RenderedElementBacking for AnchorMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn get_client_rect(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                    >,
            >,
        > {
            Box::pin(async {
                Ok(dioxus::html::geometry::PixelsRect::new(
                    dioxus::html::geometry::euclid::Point2D::new(40., 40.),
                    dioxus::html::geometry::euclid::Size2D::new(80., 32.),
                ))
            })
        }
    }
    fn panel(dom: &VirtualDom) -> String {
        dioxus_ssr::render(dom)
            .split('<')
            .find(|part| part.starts_with("div ") && part.contains("aria-label=\"Metadata panel\""))
            .unwrap()
            .split('>')
            .next()
            .unwrap()
            .to_string()
    }
    fn settle(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.render_immediate_to_vec();
            dom.process_events();
        }
    }
    let fixture = MetadataFixture::default();
    fixture.0.anchored.set(true);
    let mut dom = VirtualDom::new_with_props(metadata_fixture, fixture.clone());
    dom.rebuild_in_place();
    let popover = fixture.0.popover.get().unwrap();
    let node = popover.overlay.peek().clone().unwrap();
    let initial = panel(&dom);
    assert!(node.surface_metadata().unwrap().trigger_id.is_empty());
    assert!(initial.contains("data-unanchored=\"true\""));
    assert!(
        initial.contains("position-anchor:auto;position-area:none;position-try-fallbacks:none")
    );

    let mounted = fixture.0.mounted.borrow().unwrap();
    dom.in_scope(ScopeId::ROOT, || {
        mounted.call(Event::new(Rc::new(MountedData::new(AnchorMount)), false));
    });
    settle(&mut dom);
    let anchored = panel(&dom);
    assert_eq!(
        node.surface_metadata().unwrap().trigger_id,
        "custom-popover-trigger"
    );
    assert!(!anchored.contains("data-unanchored"));
    assert!(anchored.contains("position-anchor:--ix-popover-trigger-"));
    assert!(anchored.contains("position-area:bottom"));
    assert!(anchored.contains("position-try-fallbacks:flip-block,flip-inline,flip-block flip-inline,--ix-popover-viewport"),
        "the update must replace the old inline none, not rely on a stylesheet declaration");

    dom.in_scope(ScopeId::ROOT, || {
        fixture.0.visible.get().unwrap().set(false)
    });
    settle(&mut dom);
    let detached = panel(&dom);
    assert!(node.surface_metadata().unwrap().trigger_id.is_empty());
    assert!(detached.contains("data-unanchored=\"true\""));
    assert!(
        detached.contains("position-anchor:auto;position-area:none;position-try-fallbacks:none")
    );
}

#[test]
fn native_invokers_reference_closed_accessibly_named_surfaces() {
    let mut dom = VirtualDom::new(rendering_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("command=\"show-modal\""));
    let target = html
        .split("commandfor=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert!(html.contains(&format!("id=\"{target}\"")));
    let dialog_tag = html
        .split("<dialog")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(dialog_tag.contains("aria-modal=\"true\""));
    assert!(dialog_tag.contains("aria-label=\"Confirm resource creation\""));
    assert!(dialog_tag.contains("data-surface-generation=\"0\""));
    assert!(
        !dialog_tag
            .split_whitespace()
            .any(|attribute| attribute == "open" || attribute.starts_with("open="))
    );
    let popover_target = html
        .split("popovertarget=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert!(html.contains(&format!("id=\"{popover_target}\"")));
    assert!(html.contains("popover=\"manual\""));
    assert!(html.contains("aria-label=\"Release highlights\""));
    assert_eq!(html.matches("data-surface-generation=\"0\"").count(), 2);
    assert!(!html.contains("<script"));
}

fn controller_fixture() -> Element {
    let controller = use_modal();
    let capture = consume_context::<Rc<RefCell<Option<ModalHandle>>>>();
    *capture.borrow_mut() = Some(controller);
    VNode::empty()
}

fn popover_controller_fixture() -> Element {
    let mut controller = use_popover();
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let node = use_hook(move || {
        runtime.register(
            controller.id(),
            1,
            crate::overlay_registry::OverlayKind::Popover,
            crate::overlay_registry::CloseBehavior::Outside,
            popover_overlay_callbacks(controller, Rc::new(std::cell::Cell::new(true))),
        )
    });
    node.set_ownership(Vec::new(), true);
    node.set_mounted();
    let registered = node.clone();
    use_hook(move || controller.overlay.set(Some(registered)));
    use_drop(move || node.unregister());
    let capture = consume_context::<Rc<RefCell<Option<PopoverHandle>>>>();
    *capture.borrow_mut() = Some(controller);
    VNode::empty()
}

#[test]
fn popover_hide_is_cancelable_and_reentrant_requests_are_coalesced() {
    use dioxus::document::{Document, Eval, EvalError, Evaluator};
    use std::task::{Context, Poll, Waker};

    #[derive(Default)]
    struct NativeHide {
        expected: Option<serde_json::Value>,
        sent: Vec<serde_json::Value>,
        reply: Option<bool>,
        waiter: Option<Waker>,
        evaluations: usize,
        acknowledgements: usize,
    }
    struct HideReply(Rc<RefCell<NativeHide>>);
    impl Evaluator for HideReply {
        fn send(&self, message: serde_json::Value) -> Result<(), EvalError> {
            let mut wire = self.0.borrow_mut();
            if message["kind"] == "finish" {
                wire.acknowledgements += 1;
            } else {
                assert_eq!(message["kind"], "data");
                assert_eq!(
                    Some(&message["value"]),
                    wire.expected.as_ref(),
                    "only the owned current-generation hide is accepted"
                );
            }
            wire.sent.push(message);
            Ok(())
        }
        fn poll_recv(
            &mut self,
            context: &mut Context<'_>,
        ) -> Poll<Result<serde_json::Value, EvalError>> {
            let mut wire = self.0.borrow_mut();
            if let Some(reply) = wire.reply.take() {
                Poll::Ready(Ok(serde_json::json!({"value":reply,"error":null})))
            } else {
                wire.waiter = Some(context.waker().clone());
                Poll::Pending
            }
        }
        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            panic!("native hide uses the acknowledged one-shot receive channel")
        }
    }
    struct HideDocument {
        owner: dioxus::signals::Owner,
        wire: Rc<RefCell<NativeHide>>,
    }
    impl Document for HideDocument {
        fn eval(&self, _: String) -> Eval {
            self.wire.borrow_mut().evaluations += 1;
            Eval::new(
                self.owner
                    .insert(Box::new(HideReply(self.wire.clone())) as Box<dyn Evaluator>),
            )
        }
    }
    fn drive_until(dom: &mut VirtualDom, ready: impl Fn() -> bool) {
        // A bounded work driver, not a frame/timing assertion. The predicate
        // observes the native request or real production completion queue.
        for _ in 0..32 {
            dom.render_immediate_to_vec();
            dom.process_events();
            if ready() {
                return;
            }
        }
        assert!(
            ready(),
            "Dioxus work did not reach the expected fixture state"
        );
    }

    let wire = Rc::new(RefCell::new(NativeHide::default()));
    let document: Rc<dyn Document> = Rc::new(HideDocument {
        owner: Default::default(),
        wire: wire.clone(),
    });
    let native = cfg!(any(
        all(feature = "web", target_arch = "wasm32"),
        all(feature = "desktop", not(target_arch = "wasm32"))
    ));
    let capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(popover_controller_fixture)
        .with_root_context(capture.clone())
        .with_root_context(document);
    dom.rebuild_in_place();
    let mut popover: PopoverHandle = capture.borrow().unwrap();
    let completions = Rc::new(RefCell::new(Vec::new()));
    dom.in_scope(ScopeId::ROOT, || {
        // Native presentation requires the same explicit mounted ownership as
        // the actual component. This controller-only fixture renders no node.
        popover
            .surface
            .node
            .set(Some(Rc::new(MountedData::new(()))));
        popover.surface.open.set(true);
        popover.reported_open.set(true);
        let generation = *popover.surface.generation.peek();
        wire.borrow_mut().expected = Some(serde_json::json!([
            popover.id(),
            false,
            ["data-surface-generation", (generation + 1).to_string()],
            null,
            "popover"
        ]));
        popover
            .onshowchange
            .set(Some(EventHandler::new(|event: ToggleEvent| {
                event.prevent_default()
            })));
        let results = completions.clone();
        popover
            .onshowchanged
            .set(Some(EventHandler::new(move |visible| {
                results.borrow_mut().push(visible)
            })));
        popover.hide();
        assert!(
            popover.is_open(),
            "a rejected hide must retain the live surface"
        );
        assert_eq!(*popover.surface.generation.peek(), generation);
        assert!(completions.borrow().is_empty());
        assert_eq!(
            wire.borrow().evaluations,
            0,
            "a veto cannot issue a native hide"
        );
        popover
            .onshowchange
            .set(Some(EventHandler::new(move |_| popover.hide())));
        popover.hide();
        assert!(!popover.is_open());
        assert_eq!(*popover.surface.generation.peek(), generation + 1);
        assert!(
            completions.borrow().is_empty(),
            "close completion waits for its Dioxus effect"
        );
    });
    if native {
        drive_until(&mut dom, || !wire.borrow().sent.is_empty());
        assert!(
            completions.borrow().is_empty(),
            "native completion waits for the confirmed hide reply"
        );
        assert_eq!(wire.borrow().acknowledgements, 0);
        let waiter = {
            let mut wire = wire.borrow_mut();
            wire.reply = Some(true);
            wire.waiter
                .take()
                .expect("hide is awaiting its native result")
        };
        waiter.wake();
    }
    drive_until(&mut dom, || !completions.borrow().is_empty());
    assert_eq!(
        &*completions.borrow(),
        &[false],
        "one completion for a reentrant hide"
    );
    if native {
        let wire = wire.borrow();
        assert_eq!(wire.evaluations, 1);
        assert_eq!(wire.acknowledgements, 1);
        assert_eq!(
            wire.sent,
            vec![
                serde_json::json!({"kind":"data","value":wire.expected.as_ref().unwrap()}),
                serde_json::json!({"kind":"finish"}),
            ]
        );
    } else {
        assert!(
            wire.borrow().sent.is_empty(),
            "SSR requires no native presentation query"
        );
    }
    dom.in_scope(ScopeId::ROOT, || popover.hide());
    for _ in 0..4 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
    assert_eq!(completions.borrow().len(), 1);
    assert_eq!(
        wire.borrow().evaluations,
        usize::from(native),
        "already-hidden requests do not duplicate native operations"
    );
}

type PopoverOverlayCapture =
    Rc<RefCell<Option<(PopoverHandle, crate::overlay_runtime::NodeHandle)>>>;
fn popover_overlay_fixture() -> Element {
    let popover = use_popover();
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let capture = consume_context::<PopoverOverlayCapture>();
    let node = use_hook(move || {
        runtime.register(
            popover.id(),
            1,
            crate::overlay_registry::OverlayKind::Popover,
            crate::overlay_registry::CloseBehavior::Outside,
            popover_overlay_callbacks(popover, Rc::new(std::cell::Cell::new(true))),
        )
    });
    node.set_ownership(Vec::new(), true);
    node.set_mounted();
    *capture.borrow_mut() = Some((popover, node));
    VNode::empty()
}

#[test]
fn popover_overlay_request_observes_one_veto_and_preserves_close_focus_policy() {
    let capture: PopoverOverlayCapture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(popover_overlay_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let (mut popover, node) = capture.borrow().clone().unwrap();
    let requests = Rc::new(std::cell::Cell::new(0));
    let canceled = Rc::new(std::cell::Cell::new(true));
    dom.in_scope(ScopeId::ROOT, || {
        popover.hover_mode.set(true);
        popover.surface.open.set(true);
        let seen = requests.clone();
        let veto = canceled.clone();
        popover
            .onshowchange
            .set(Some(EventHandler::new(move |event: ToggleEvent| {
                seen.set(seen.get() + 1);
                if veto.get() {
                    event.prevent_default();
                }
            })));
        assert!(!node.request(false, crate::overlay_runtime::Reason::Escape, true));
        assert!(popover.is_open());
        assert_eq!(requests.get(), 1);
        assert_eq!(*popover.surface.generation.peek(), 0);
        assert!(
            !*popover.suppress_focus_open.peek(),
            "a veto never changes hover focus admission"
        );
        canceled.set(false);
        assert!(node.request(false, crate::overlay_runtime::Reason::Outside, false));
        assert_eq!(
            requests.get(),
            2,
            "accepted request emits before exactly once"
        );
        assert_eq!(
            *popover.surface.restore_after_mount.peek(),
            None,
            "outside close releases invoker focus"
        );
        assert!(!popover.is_open());
        assert!(
            !*popover.suppress_focus_open.peek(),
            "outside focus closes without queuing restoration"
        );
        assert!(
            popover.hover_focus_allowed(),
            "the next genuine trigger focus may reopen"
        );
        popover.surface.open.set(true);
        assert!(node.request(false, crate::overlay_runtime::Reason::Escape, true));
        assert_eq!(requests.get(), 3);
        assert_eq!(
            *popover.surface.restore_after_mount.peek(),
            None,
            "persistent Popover no longer relies on remount-driven focus"
        );
        assert!(
            *popover.suppress_focus_open.peek(),
            "explicit close reserves one restoration focus event"
        );
        assert!(
            !popover.hover_focus_allowed(),
            "restoration must not immediately reopen hover content"
        );
        assert!(
            popover.hover_focus_allowed(),
            "suppression is consumed once; later genuine focus may open"
        );
    });
}

#[test]
fn popover_typed_requests_distinguish_closed_dismissal_and_share_cancellation() {
    let capture: PopoverOverlayCapture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(popover_overlay_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let (mut popover, node) = capture.borrow().clone().unwrap();
    let requests = Rc::new(RefCell::new(Vec::new()));
    let legacy = Rc::new(std::cell::Cell::new(0));
    dom.in_scope(ScopeId::ROOT, || {
        let seen = requests.clone();
        popover
            .onbeforechange
            .set(Some(EventHandler::new(move |event: PopoverChangeEvent| {
                seen.borrow_mut().push((event.open, event.previous));
                if event.open {
                    event.prevent_default();
                }
            })));
        let old = legacy.clone();
        popover
            .onshowchange
            .set(Some(EventHandler::new(move |event: ToggleEvent| {
                old.set(old.get() + 1);
                if old.get() == 2 {
                    assert!(
                        !event.default_action_enabled(),
                        "typed veto is visible to legacy listener"
                    );
                }
            })));
        assert!(!node.request(false, crate::overlay_runtime::Reason::Parent, true));
        assert_eq!(
            *popover.surface.generation.peek(),
            0,
            "closed request has no native command"
        );
        assert!(!popover.request_change(true));
        assert_eq!(&*requests.borrow(), &[(false, false), (true, false)]);
        assert_eq!(legacy.get(), 2);
        popover.surface.open.set(true);
        assert!(node.request(false, crate::overlay_runtime::Reason::Parent, false));
        assert_eq!(requests.borrow().last(), Some(&(false, true)));
    });
}

#[test]
fn imperative_opening_before_mount_returns_error_without_changing_controller_state() {
    use std::future::Future;
    use std::task::{Context, Poll, Waker};
    let capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let modal: ModalHandle = capture.borrow().unwrap();
    dom.in_scope(ScopeId::ROOT, || {
        let mut operation = std::pin::pin!(modal.open());
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            operation.as_mut().poll(&mut context),
            Poll::Ready(Err(NativeSurfaceError::NotMounted))
        ));
        assert!(!modal.is_open());
        assert!(modal.result().is_none());
    });
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[test]
fn a_delayed_native_open_reply_cannot_restore_a_closed_generation_or_its_focus_target() {
    use dioxus::document::{Document, Eval, EvalError, Evaluator};
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    struct DelayedNativeReply {
        reply: Rc<RefCell<Option<serde_json::Value>>>,
        acknowledgements: Rc<std::cell::Cell<usize>>,
        active_focus: bool,
    }
    impl Evaluator for DelayedNativeReply {
        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            // The shared one-shot transport receives before acknowledging;
            // a completed desktop evaluation's direct return may be gone.
            Poll::Ready(Err(EvalError::Finished))
        }
        fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            match self.reply.borrow_mut().take() {
                Some(reply) => Poll::Ready(Ok(serde_json::json!({
                    "value": reply, "error": null
                }))),
                None => Poll::Pending,
            }
        }
        fn send(&self, message: serde_json::Value) -> Result<(), EvalError> {
            if message["kind"] == "finish" {
                self.acknowledgements.set(self.acknowledgements.get() + 1);
            } else {
                assert_eq!(message["kind"], "data");
                if self.active_focus {
                    assert_eq!(message["value"], serde_json::json!([null, null]));
                } else {
                    assert_eq!(
                        message["value"][2],
                        serde_json::json!(["data-surface-generation", "0"])
                    );
                }
            }
            Ok(())
        }
    }
    struct DelayedDocument {
        owner: dioxus::signals::Owner,
        reply: Rc<RefCell<Option<serde_json::Value>>>,
        acknowledgements: Rc<std::cell::Cell<usize>>,
    }
    impl Document for DelayedDocument {
        fn eval(&self, script: String) -> Eval {
            let active_focus = script.contains("document.activeElement");
            let reply = if active_focus {
                Rc::new(RefCell::new(Some(serde_json::json!({
                    "connected": false, "inside": false, "active_id": "retained-focus-target"
                }))))
            } else {
                self.reply.clone()
            };
            Eval::new(self.owner.insert(Box::new(DelayedNativeReply {
                reply,
                acknowledgements: self.acknowledgements.clone(),
                active_focus,
            }) as Box<dyn Evaluator>))
        }
    }

    let capture = Rc::new(RefCell::new(None));
    let reply = Rc::new(RefCell::new(None));
    let acknowledgements = Rc::new(std::cell::Cell::new(0));
    let document: Rc<dyn Document> = Rc::new(DelayedDocument {
        owner: Default::default(),
        reply: reply.clone(),
        acknowledgements: acknowledgements.clone(),
    });
    let mut dom = VirtualDom::new(controller_fixture)
        .with_root_context(capture.clone())
        .with_root_context(document);
    dom.rebuild_in_place();
    let mut modal: ModalHandle = capture.borrow().unwrap();
    dom.in_scope(ScopeId::ROOT, || {
        modal.surface.node.set(Some(Rc::new(MountedData::new(()))));
        let invoker = Rc::new(MountedData::new(()));
        modal.focus_runtime.peek().register_item_mount(
            "modal-opener",
            "retained-focus-target",
            invoker.clone(),
        );
        let mut operation = std::pin::pin!(modal.open());
        let mut context = Context::from_waker(Waker::noop());
        assert!(operation.as_mut().poll(&mut context).is_pending());
        assert_eq!(
            acknowledgements.get(),
            1,
            "identity query completed before native open"
        );
        assert!(
            modal
                .surface
                .invoker
                .peek()
                .as_ref()
                .unwrap()
                .matches_node(&invoker)
        );
        // A native opening notification and application close can arrive
        // before the queued bridge result reaches this Rust task.
        modal.surface.open.set(true);
        modal.close("saved before native reply");
        *reply.borrow_mut() = Some(serde_json::json!(true));
        assert!(matches!(
            operation.as_mut().poll(&mut context),
            Poll::Ready(Err(NativeSurfaceError::Superseded))
        ));
        assert_eq!(acknowledgements.get(), 2);
        assert!(!modal.is_open());
        assert_eq!(
            modal.result(),
            Some(ModalResult::Closed("saved before native reply".into()))
        );
        assert!(
            modal
                .surface
                .invoker
                .peek()
                .as_ref()
                .unwrap()
                .matches_node(&invoker)
        );
    });
}

#[test]
fn programmatic_dismissal_preserves_reason_and_completes_callbacks_once() {
    let capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let mut modal: ModalHandle = capture.borrow().unwrap();
    let order = Rc::new(RefCell::new(Vec::new()));
    dom.in_scope(ScopeId::ROOT, || {
        modal.surface.node.set(Some(Rc::new(MountedData::new(()))));
        let visibility = order.clone();
        let dismissal = order.clone();
        let completed = order.clone();
        let reason = ModalDismissReason::Programmatic("app-switch".into());
        let expected = reason.clone();
        modal.callbacks.set(ModalCallbacks {
            onopenchange: Some(EventHandler::new(move |open| {
                visibility
                    .borrow_mut()
                    .push(if open { "opened" } else { "closed" });
            })),
            ondismiss: Some(EventHandler::new(move |received| {
                assert_eq!(received, expected);
                dismissal.borrow_mut().push("dismissed");
            })),
            onresult: Some(EventHandler::new(move |result| {
                assert_eq!(
                    result,
                    ModalResult::Dismissed(ModalDismissReason::Programmatic("app-switch".into()))
                );
                completed.borrow_mut().push("result");
            })),
            ..Default::default()
        });
        modal.dismiss(reason.clone());
        assert!(
            order.borrow().is_empty(),
            "a closed dialog cannot emit a dismissal"
        );
        modal.opened();
        let generation = modal.surface_generation();
        modal.dismiss(reason.clone());
        assert!(!modal.is_open());
        assert_eq!(modal.result(), Some(ModalResult::Dismissed(reason.clone())));
        assert_eq!(modal.surface_generation(), generation + 1);
        modal.dismiss(reason);
        assert_eq!(
            &*order.borrow(),
            &["opened", "closed", "dismissed", "result"]
        );
    });
}

#[test]
fn deferred_dismissal_is_coalesced_and_stale_decisions_cannot_close_a_reopened_dialog() {
    let capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let mut modal: ModalHandle = capture.borrow().unwrap();
    let requests = Rc::new(RefCell::new(Vec::<ModalDismissRequest>::new()));
    dom.in_scope(ScopeId::ROOT, || {
        let received = requests.clone();
        modal.callbacks.set(ModalCallbacks {
            ondismissrequest: Some(EventHandler::new(move |request| {
                received.borrow_mut().push(request)
            })),
            ..Default::default()
        });
        modal.surface.open.set(true);
        modal.dismiss(ModalDismissReason::Cancel);
        modal.dismiss(ModalDismissReason::Escape);
        assert_eq!(
            requests.borrow().len(),
            1,
            "one outstanding decision per dialog"
        );
        let request = requests.borrow_mut().pop().unwrap();
        assert!(modal.is_dismiss_pending());
        assert!(request.clone().deny());
        assert!(!modal.is_dismiss_pending());
        assert!(
            !request.allow(),
            "a denied request cannot later be accepted"
        );

        modal.dismiss(ModalDismissReason::Header);
        let stale_request = requests.borrow_mut().pop().unwrap();
        let id = modal.id();
        let old_generation = *modal.surface.generation.peek();
        modal.close("saved");
        assert_eq!(modal.result(), Some(ModalResult::Closed("saved".into())));
        let replacement_generation = *modal.surface.generation.peek();
        assert_eq!(
            *modal.surface.restore_after_mount.peek(),
            Some(replacement_generation)
        );
        modal
            .surface
            .mounted(old_generation, Rc::new(MountedData::new(())));
        assert!(
            modal.surface.node.peek().is_none(),
            "stale mount events must not replace the current surface reference"
        );
        assert_eq!(
            *modal.surface.restore_after_mount.peek(),
            Some(replacement_generation)
        );
        modal
            .surface
            .mounted(replacement_generation, Rc::new(MountedData::new(())));
        assert_eq!(
            *modal.surface.restore_after_mount.peek(),
            None,
            "focus restoration is consumed only by the closed replacement mount"
        );
        modal.surface.open.set(true);
        assert!(
            !stale_request.allow(),
            "a prior generation cannot close a newly opened surface"
        );
        assert!(modal.is_open());
        assert_eq!(
            modal.id(),
            id,
            "the declarative invoker's ID survives close/remount"
        );

        modal.dismiss(ModalDismissReason::Backdrop);
        assert!(requests.borrow_mut().pop().unwrap().allow());
        assert!(!modal.is_open());
        assert_eq!(
            modal.result(),
            Some(ModalResult::Dismissed(ModalDismissReason::Backdrop))
        );
        modal.callbacks.set(ModalCallbacks {
            ondismissrequest: Some(EventHandler::new(|request: ModalDismissRequest| {
                assert!(request.allow());
            })),
            ..Default::default()
        });
        modal.surface.open.set(true);
        modal.dismiss(ModalDismissReason::Escape);
        assert!(!modal.is_open(), "guards may also approve synchronously");
    });
}

#[test]
fn confirmation_bypasses_pending_guard_and_emits_each_payload_once() {
    let capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let mut modal: ModalHandle = capture.borrow().unwrap();
    let requests = Rc::new(RefCell::new(Vec::<ModalDismissRequest>::new()));
    let results = Rc::new(RefCell::new(Vec::<ModalResult>::new()));
    let confirmations = Rc::new(RefCell::new(Vec::<String>::new()));
    let dismissals = Rc::new(RefCell::new(Vec::<ModalDismissReason>::new()));
    dom.in_scope(ScopeId::ROOT, || {
        let received = requests.clone();
        let completed = results.clone();
        let confirmed = confirmations.clone();
        let dismissed = dismissals.clone();
        modal.callbacks.set(ModalCallbacks {
            ondismissrequest: Some(EventHandler::new(move |request| {
                received.borrow_mut().push(request)
            })),
            onresult: Some(EventHandler::new(move |result| {
                completed.borrow_mut().push(result)
            })),
            onclose: Some(EventHandler::new(move |payload| {
                confirmed.borrow_mut().push(payload)
            })),
            ondismiss: Some(EventHandler::new(move |reason| {
                dismissed.borrow_mut().push(reason)
            })),
            ..Default::default()
        });
        modal.surface.open.set(true);
        modal.dismiss(ModalDismissReason::Header);
        assert!(modal.is_dismiss_pending());
        modal.close("close payload!");
        assert!(!modal.is_dismiss_pending());
        assert!(!modal.is_open());
        assert!(!requests.borrow_mut().pop().unwrap().allow());
        modal.close("duplicate completion");
        modal.dismiss(ModalDismissReason::Cancel);
        assert_eq!(&*confirmations.borrow(), &["close payload!"]);
        assert_eq!(results.borrow().len(), 1);
        assert!(dismissals.borrow().is_empty());

        modal.surface.open.set(true);
        let reason = ModalDismissReason::Programmatic("dismiss payload".into());
        modal.dismiss(reason.clone());
        assert!(requests.borrow_mut().pop().unwrap().allow());
        assert!(!modal.is_open());
        assert_eq!(&*dismissals.borrow(), std::slice::from_ref(&reason));
        assert_eq!(
            &*results.borrow(),
            &[
                ModalResult::Closed("close payload!".into()),
                ModalResult::Dismissed(reason),
            ]
        );
    });
}
mod message_dialog_tests {
    use super::*;
    include!("../message_dialog_tests.rs");
}
