//! Opening admission must survive cancellation of its owning Rust future.
//! These tests stop at the deliberate pre-native yield, so no renderer, DOM
//! bridge or native opening is required and no completion is fabricated.
use super::*;
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::task::{Context, Waker};

type Capture = Rc<RefCell<Option<PopoverHandle>>>;

fn fixture() -> Element {
    let popover = use_popover();
    let capture = consume_context::<Capture>();
    *capture.borrow_mut() = Some(popover);
    VNode::empty()
}

#[test]
fn canceled_opening_future_releases_admission_for_another_show() {
    let capture: Capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let mut popover = capture.borrow().unwrap();
    let requests = Rc::new(Cell::new(0));

    dom.in_scope(ScopeId::ROOT, || {
        popover
            .surface
            .node
            .set(Some(Rc::new(MountedData::new(()))));
        let seen = requests.clone();
        popover
            .onbeforechange
            .set(Some(EventHandler::new(move |event: PopoverChangeEvent| {
                assert!(event.open && !event.previous);
                seen.set(seen.get() + 1);
            })));
        let mut context = Context::from_waker(Waker::noop());
        let mut first = Box::pin(popover.show_native());
        assert!(first.as_mut().poll(&mut context).is_pending());
        let first_generation = *popover.surface.generation.peek();
        assert!(*popover.opening.peek());
        assert!(
            !popover.is_open(),
            "the first yield precedes native opening"
        );
        assert_eq!(requests.get(), 1);

        // Keep both the controller and its signal owner alive. Only the
        // admitted asynchronous operation is canceled here.
        drop(first);
        assert!(
            !*popover.opening.peek(),
            "cancellation must release admission"
        );
        assert!(!popover.is_open());
        assert_eq!(*popover.surface.generation.peek(), first_generation);
        assert!(popover.surface.node.peek().is_some());

        let mut second = Box::pin(popover.show_native());
        assert!(second.as_mut().poll(&mut context).is_pending());
        assert!(*popover.opening.peek());
        assert_eq!(*popover.surface.generation.peek(), first_generation + 1);
        assert_eq!(requests.get(), 2, "a new show must reach its own admission");
        drop(second);
        assert!(!*popover.opening.peek());
    });
}

#[test]
fn dropping_superseded_opening_future_preserves_newer_admission() {
    let capture: Capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let mut popover = capture.borrow().unwrap();

    dom.in_scope(ScopeId::ROOT, || {
        popover
            .surface
            .node
            .set(Some(Rc::new(MountedData::new(()))));
        let mut context = Context::from_waker(Waker::noop());
        let mut first = Box::pin(popover.show_native());
        assert!(first.as_mut().poll(&mut context).is_pending());
        let first_generation = *popover.surface.generation.peek();
        assert!(*popover.opening.peek());

        // Model lifecycle supersession while retaining the old future. The
        // replacement operation below is real and owns a distinct generation.
        popover.surface.generation.set(first_generation + 1);
        popover.opening.set(false);
        let mut second = Box::pin(popover.show_native());
        assert!(second.as_mut().poll(&mut context).is_pending());
        let second_generation = *popover.surface.generation.peek();
        assert!(second_generation > first_generation);
        assert!(*popover.opening.peek());

        drop(first);
        assert!(
            *popover.opening.peek(),
            "an old future must not release the newer operation's admission"
        );
        assert_eq!(*popover.surface.generation.peek(), second_generation);
        assert!(!popover.is_open());
        drop(second);
        assert!(
            !*popover.opening.peek(),
            "the current owner still releases itself"
        );
    });
}

#[derive(Clone, Default)]
struct FocusFixture(Rc<FocusState>);
#[derive(Default)]
struct FocusState {
    popover: Cell<Option<PopoverHandle>>,
    node: RefCell<Option<crate::overlay_runtime::NodeHandle>>,
    first: Cell<Option<Signal<bool>>>,
    second: Cell<Option<Signal<bool>>>,
    mounts: RefCell<std::collections::BTreeMap<String, EventHandler<MountedEvent>>>,
}
impl PartialEq for FocusFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn RegisteredTrigger(popover: PopoverHandle, id: String, fixture: FocusFixture) -> Element {
    let mounted = use_popover_trigger_mount(popover, id.clone());
    fixture.0.mounts.borrow_mut().insert(id, mounted);
    VNode::empty()
}
fn focus_fixture(fixture: FocusFixture) -> Element {
    let mut popover = use_popover();
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let registered = runtime.clone();
    let node = use_hook(move || {
        registered.register(
            popover.id(),
            1,
            crate::overlay_registry::OverlayKind::Popover,
            crate::overlay_registry::CloseBehavior::Outside,
            popover_overlay_callbacks(popover, Rc::new(Cell::new(true))),
        )
    });
    node.set_ownership(Vec::new(), true);
    node.set_mounted();
    let mounted = node.clone();
    use_hook(move || popover.overlay.set(Some(mounted)));
    let cleanup = node.clone();
    use_drop(move || cleanup.unregister());
    fixture.0.popover.set(Some(popover));
    *fixture.0.node.borrow_mut() = Some(node);
    let first = use_signal(|| true);
    let second = use_signal(|| true);
    fixture.0.first.set(Some(first));
    fixture.0.second.set(Some(second));
    rsx! {
        if first() {RegisteredTrigger {popover,id:"first-trigger",fixture:fixture.clone()}}
        if second() {RegisteredTrigger {popover,id:"second-trigger",fixture:fixture.clone()}}
    }
}
#[derive(Clone)]
struct FocusMount {
    name: String,
    calls: Rc<RefCell<Vec<String>>>,
}
impl dioxus::html::RenderedElementBacking for FocusMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let mounted = self.clone();
        Box::pin(async move {
            mounted.calls.borrow_mut().push(mounted.name);
            Ok(())
        })
    }
}
fn mount_focus(fixture: &FocusFixture, name: &str, calls: &Rc<RefCell<Vec<String>>>) {
    fixture
        .0
        .node
        .borrow()
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap()
        .register_item_mount(
            name,
            name,
            Rc::new(MountedData::new(FocusMount {
                name: name.into(),
                calls: calls.clone(),
            })),
        );
}
fn mounted_trigger(
    dom: &VirtualDom,
    fixture: &FocusFixture,
    id: &str,
    calls: &Rc<RefCell<Vec<String>>>,
) {
    dom.in_scope(ScopeId::ROOT, || {
        fixture.0.mounts.borrow()[id].call(Event::new(
            Rc::new(MountedData::new(FocusMount {
                name: id.into(),
                calls: calls.clone(),
            })),
            false,
        ))
    });
}
fn settle_focus(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
}
fn present_focus(dom: &VirtualDom, fixture: &FocusFixture) -> (PopoverHandle, u64) {
    let mut popover = fixture.0.popover.get().unwrap();
    dom.in_scope(ScopeId::ROOT, || {
        popover.surface.open.set(true);
        let generation = popover.surface.generation.peek().wrapping_add(1);
        popover.surface.generation.set(generation);
        fixture.0.node.borrow().as_ref().unwrap().sync_open(true);
    });
    (popover, *popover.surface.generation.peek())
}
fn candidate(name: &str, tab_index: i32, in_header: bool) -> crate::popover_focus::FocusCandidate {
    crate::popover_focus::FocusCandidate {
        target: name.into(),
        tab_index,
        in_header,
    }
}

#[test]
fn popover_opening_chooses_registered_content_then_header_and_hover_preserves_focus() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(focus_fixture, fixture.clone());
    dom.rebuild_in_place();
    let calls = Rc::new(RefCell::new(Vec::new()));
    for id in ["header", "body-zero", "body-positive"] {
        mount_focus(&fixture, id, &calls);
    }
    mounted_trigger(&dom, &fixture, "second-trigger", &calls);
    let (mut popover, generation) = present_focus(&dom, &fixture);
    let candidates = [
        candidate("unregistered", 1, false),
        candidate("header", 1, true),
        candidate("body-zero", 0, false),
        candidate("body-positive", 2, false),
    ];
    dom.in_scope(ScopeId::ROOT, || {
        popover.apply_opening_focus(generation, &candidates)
    });
    settle_focus(&mut dom);
    assert_eq!(*calls.borrow(), ["body-positive"]);
    assert!(*popover.focusable_at_open.peek());
    dom.in_scope(ScopeId::ROOT, || {
        popover.apply_opening_focus(generation, &[candidate("header", 0, true)])
    });
    settle_focus(&mut dom);
    assert_eq!(*calls.borrow(), ["body-positive", "header"]);
    dom.in_scope(ScopeId::ROOT, || {
        popover.hover_mode.set(true);
        popover.apply_opening_focus(generation, &candidates);
    });
    settle_focus(&mut dom);
    assert_eq!(
        *calls.borrow(),
        ["body-positive", "header"],
        "hover opening never moves focus"
    );
}

#[test]
fn popover_close_returns_registered_trigger_and_hover_focus_does_not_reopen() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(focus_fixture, fixture.clone());
    dom.rebuild_in_place();
    let calls = Rc::new(RefCell::new(Vec::new()));
    mount_focus(&fixture, "body", &calls);
    mounted_trigger(&dom, &fixture, "second-trigger", &calls);
    let (mut popover, generation) = present_focus(&dom, &fixture);
    dom.in_scope(ScopeId::ROOT, || {
        popover.apply_opening_focus(generation, &[candidate("body", 0, false)])
    });
    settle_focus(&mut dom);
    let veto = Rc::new(Cell::new(true));
    let events = Rc::new(RefCell::new(Vec::new()));
    dom.in_scope(ScopeId::ROOT, || {
        let blocked = veto.clone();
        popover
            .onbeforechange
            .set(Some(EventHandler::new(move |event: PopoverChangeEvent| {
                if blocked.get() {
                    event.prevent_default();
                }
            })));
        let seen = events.clone();
        popover
            .onshowchanged
            .set(Some(EventHandler::new(move |open| {
                seen.borrow_mut().push(open)
            })));
        assert!(!fixture.0.node.borrow().as_ref().unwrap().request(
            false,
            crate::overlay_runtime::Reason::Escape,
            true
        ));
    });
    settle_focus(&mut dom);
    assert!(popover.is_open());
    assert_eq!(*calls.borrow(), ["body"]);
    veto.set(false);
    dom.in_scope(ScopeId::ROOT, || {
        popover.hover_mode.set(true);
        assert!(fixture.0.node.borrow().as_ref().unwrap().request(
            false,
            crate::overlay_runtime::Reason::Escape,
            true
        ));
    });
    assert!(
        events.borrow().is_empty(),
        "completion is owned by a Dioxus effect"
    );
    settle_focus(&mut dom);
    assert_eq!(*calls.borrow(), ["body", "second-trigger"]);
    assert_eq!(*events.borrow(), [false]);
    dom.in_scope(ScopeId::ROOT, || {
        assert!(!popover.hover_focus_allowed());
        assert!(!popover.is_open());
        assert!(popover.hover_focus_allowed());
    });
    let (popover, generation) = present_focus(&dom, &fixture);
    dom.in_scope(ScopeId::ROOT, || {
        popover.apply_opening_focus(generation, &[candidate("body", 0, false)]);
        assert!(fixture.0.node.borrow().as_ref().unwrap().request(
            false,
            crate::overlay_runtime::Reason::Outside,
            false
        ));
    });
    settle_focus(&mut dom);
    assert_eq!(
        *calls.borrow(),
        ["body", "second-trigger"],
        "outside dismissal cancels pending opening and preserves the outside destination"
    );
}

#[test]
fn popover_focus_rejects_replaced_unmounted_or_newly_closed_registered_content() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(focus_fixture, fixture.clone());
    dom.rebuild_in_place();
    let old = Rc::new(RefCell::new(Vec::new()));
    let replacement = Rc::new(RefCell::new(Vec::new()));
    mount_focus(&fixture, "body", &old);
    let (popover, generation) = present_focus(&dom, &fixture);
    dom.in_scope(ScopeId::ROOT, || {
        popover.apply_opening_focus(generation, &[candidate("body", 0, false)])
    });
    mount_focus(&fixture, "body", &replacement);
    settle_focus(&mut dom);
    assert!(old.borrow().is_empty() && replacement.borrow().is_empty());
    dom.in_scope(ScopeId::ROOT, || {
        popover.apply_opening_focus(generation, &[candidate("body", 0, false)])
    });
    fixture
        .0
        .node
        .borrow()
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap()
        .unregister_item_cue("body");
    settle_focus(&mut dom);
    assert!(replacement.borrow().is_empty());
    mount_focus(&fixture, "body", &replacement);
    dom.in_scope(ScopeId::ROOT, || {
        popover.apply_opening_focus(generation, &[candidate("body", 0, false)]);
        popover.commit_hide_with_focus(false);
    });
    settle_focus(&mut dom);
    assert!(replacement.borrow().is_empty());
}

#[test]
fn popover_custom_trigger_cleanup_preserves_newer_mount_and_cancels_removed_return() {
    let fixture = FocusFixture::default();
    let mut dom = VirtualDom::new_with_props(focus_fixture, fixture.clone());
    dom.rebuild_in_place();
    let calls = Rc::new(RefCell::new(Vec::new()));
    mount_focus(&fixture, "body", &calls);
    mounted_trigger(&dom, &fixture, "first-trigger", &calls);
    let first_ownership = fixture
        .0
        .popover
        .get()
        .unwrap()
        .trigger_ownership
        .peek()
        .clone();
    mounted_trigger(&dom, &fixture, "second-trigger", &calls);
    let second_ownership = fixture
        .0
        .popover
        .get()
        .unwrap()
        .trigger_ownership
        .peek()
        .clone();
    assert!(first_ownership.is_some() && second_ownership.is_some());
    assert!(
        first_ownership != second_ownership,
        "replacement trigger owns its lineage"
    );
    dom.in_scope(ScopeId::ROOT, || fixture.0.first.get().unwrap().set(false));
    settle_focus(&mut dom);
    let (popover, generation) = present_focus(&dom, &fixture);
    assert!(
        *popover.trigger_ownership.peek() == second_ownership,
        "removing an older trigger preserves its replacement's ownership"
    );
    assert_eq!(popover.trigger_id.peek().as_deref(), Some("second-trigger"));
    dom.in_scope(ScopeId::ROOT, || {
        popover.apply_opening_focus(generation, &[candidate("body", 0, false)]);
        popover.commit_hide_with_focus(true);
        fixture.0.second.get().unwrap().set(false);
    });
    settle_focus(&mut dom);
    assert!(calls.borrow().is_empty());
    assert!(popover.trigger_id.peek().is_none());
    assert!(popover.surface.invoker.peek().is_none());
    assert!(popover.trigger_ownership.peek().is_none());
}

#[derive(Clone, Default)]
struct ExternalOwnershipFixture(Rc<ExternalOwnershipState>);
#[derive(Default)]
struct ExternalOwnershipState {
    parent: RefCell<Option<crate::overlay_runtime::NodeHandle>>,
    nested: Cell<Option<Signal<bool>>>,
    body: RefCell<Vec<crate::overlay_runtime::NodeHandle>>,
}
impl PartialEq for ExternalOwnershipFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn ExternalOwnershipTrigger(popover: PopoverHandle) -> Element {
    let _ = use_popover_trigger_mount(popover, "external-popover-trigger");
    VNode::empty()
}
#[component]
fn ExternalOwnershipScope(
    node: Option<crate::overlay_runtime::NodeHandle>,
    popover: PopoverHandle,
) -> Element {
    crate::overlay_runtime::use_overlay_scope(node);
    rsx! { ExternalOwnershipTrigger {popover} }
}
#[component]
fn ExternalOwnershipBody(fixture: ExternalOwnershipFixture) -> Element {
    *fixture.0.body.borrow_mut() = crate::overlay_runtime::use_overlay_ancestors();
    VNode::empty()
}
fn external_ownership_fixture(fixture: ExternalOwnershipFixture) -> Element {
    let popover = use_popover();
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let parent = use_hook(move || {
        runtime.register(
            "external-header".into(),
            1,
            crate::overlay_registry::OverlayKind::Header,
            crate::overlay_registry::CloseBehavior::Both,
            crate::overlay_runtime::Callbacks {
                read: Rc::new(|| Some(false)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        )
    });
    parent.set_ownership(Vec::new(), true);
    parent.set_mounted();
    *fixture.0.parent.borrow_mut() = Some(parent.clone());
    let cleanup = parent.clone();
    use_drop(move || cleanup.unregister());
    let nested = use_signal(|| true);
    fixture.0.nested.set(Some(nested));
    rsx! {
        ExternalOwnershipScope {node:nested().then_some(parent),popover}
        Popover {popover,label:"External trigger",ExternalOwnershipBody {fixture}}
    }
}
#[test]
fn externally_rendered_popover_body_inherits_current_trigger_ancestry() {
    let fixture = ExternalOwnershipFixture::default();
    let mut dom = VirtualDom::new_with_props(external_ownership_fixture, fixture.clone());
    dom.rebuild_in_place();
    settle_focus(&mut dom);
    let parent = fixture.0.parent.borrow().clone().unwrap();
    let body = fixture.0.body.borrow().clone();
    assert!(
        body.len() == 2 && body[1] == parent,
        "body retains its Popover and trigger-owned outer Header"
    );
    let popover = body[0].clone();
    dom.in_scope(ScopeId::ROOT, || fixture.0.nested.get().unwrap().set(false));
    settle_focus(&mut dom);
    assert!(
        *fixture.0.body.borrow() == [popover.clone()],
        "moving the trigger into an inline group removes the outer Header"
    );
    dom.in_scope(ScopeId::ROOT, || fixture.0.nested.get().unwrap().set(true));
    settle_focus(&mut dom);
    assert!(
        *fixture.0.body.borrow() == [popover, parent],
        "the same retained body follows restored trigger ownership"
    );
}
