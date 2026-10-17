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
        let mut first = Box::pin(popover.show_native(false));
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

        let mut second = Box::pin(popover.show_native(false));
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
        let mut first = Box::pin(popover.show_native(false));
        assert!(first.as_mut().poll(&mut context).is_pending());
        let first_generation = *popover.surface.generation.peek();
        assert!(*popover.opening.peek());

        // Model lifecycle supersession while retaining the old future. The
        // replacement operation below is real and owns a distinct generation.
        popover.surface.generation.set(first_generation + 1);
        popover.opening.set(false);
        let mut second = Box::pin(popover.show_native(false));
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

#[test]
fn canceling_unpolled_close_future_releases_hover_focus_suppression() {
    let capture: Capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let mut popover = capture.borrow().unwrap();

    dom.in_scope(ScopeId::ROOT, || {
        popover.surface.generation.set(11);
        popover.suppress_focus_open.set(true);
        // Close commits suppression before spawn. The guard must already be a
        // captured future field, since a canceled task may never be polled.
        let suppression = PopoverFocusSuppression {
            popover,
            generation: 11,
        };
        let close = Box::pin(async move {
            let _suppression = suppression;
            std::future::pending::<()>().await;
        });
        assert!(*popover.suppress_focus_open.peek());
        drop(close);
        assert!(
            !*popover.suppress_focus_open.peek(),
            "an unpolled close cannot leave a retained trigger suppressed"
        );
    });
}

#[test]
fn old_focus_suppression_guard_preserves_newer_close_operation() {
    let capture: Capture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let mut popover = capture.borrow().unwrap();

    dom.in_scope(ScopeId::ROOT, || {
        popover.surface.generation.set(21);
        popover.suppress_focus_open.set(true);
        let old = PopoverFocusSuppression {
            popover,
            generation: 21,
        };
        popover.surface.generation.set(22);
        let current = PopoverFocusSuppression {
            popover,
            generation: 22,
        };
        drop(old);
        assert!(
            *popover.suppress_focus_open.peek(),
            "an old close cannot release the newer restore-focus interval"
        );
        assert_eq!(*popover.surface.generation.peek(), 22);
        drop(current);
        assert!(
            !*popover.suppress_focus_open.peek(),
            "the current close still releases its own suppression"
        );
    });
}
