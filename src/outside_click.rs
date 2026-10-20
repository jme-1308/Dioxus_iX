// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Component-owned outside-click facts. Pinned alpha.2 MountedData provides
//! normal element events, but Document has no document-click subscription.
//! The native adapter only checks the composed path against the explicitly
//! mounted Dioxus root; Rust owns eligibility, callbacks and cancellation.

use dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};

struct OutsideClick {
    alive: bool,
    mounted: bool,
    enabled: bool,
    root_id: String,
    generation: u64,
    #[cfg(any(feature = "web", feature = "desktop", test))]
    onoutside: EventHandler<()>,
    #[cfg(any(feature = "web", feature = "desktop", test))]
    document: Option<Rc<dyn dioxus::document::Document>>,
    #[cfg(any(feature = "web", feature = "desktop", test))]
    session: Option<(dioxus::document::Eval, dioxus::core::Task)>,
}

impl OutsideClick {
    fn stop(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        #[cfg(any(feature = "web", feature = "desktop", test))]
        if let Some((eval, task)) = self.session.take() {
            let _ = eval.send("finish");
            task.cancel();
        }
    }

    fn start(state: &Rc<RefCell<Self>>) {
        {
            let state = state.borrow();
            if !state.alive || !state.mounted || !state.enabled {
                return;
            }
        }
        #[cfg(any(feature = "web", feature = "desktop", test))]
        {
            let (document, root_id, generation) = {
                let state = state.borrow();
                if state.session.is_some() {
                    return;
                }
                let Some(document) = state.document.clone() else {
                    return;
                };
                (document, state.root_id.clone(), state.generation)
            };
            let eval = document.eval(OUTSIDE_CLICK.to_string());
            if eval.send(root_id).is_err() {
                return;
            }
            let weak = Rc::downgrade(state);
            let mut receiver = eval;
            let task = spawn(async move {
                while receiver.recv::<()>().await.is_ok() {
                    let Some(state) = weak.upgrade() else {
                        break;
                    };
                    let callback = {
                        let state = state.borrow();
                        if !state.alive || !state.enabled || state.generation != generation {
                            break;
                        }
                        state.onoutside
                    };
                    // A closing callback may disable or unmount this component.
                    // Release the state borrow before invoking application code.
                    callback.call(());
                }
            });
            state.borrow_mut().session = Some((eval, task));
        }
        #[cfg(not(any(feature = "web", feature = "desktop", test)))]
        let _ = state;
    }
}

/// Bind this handler to the explicitly owned root's onmounted event. No native
/// listener exists before mount or while disabled. A changed root retires the
/// previous stream, including any facts already queued in its channel.
pub(crate) fn use_outside_click(
    root_id: String,
    enabled: bool,
    onoutside: EventHandler<()>,
) -> EventHandler<MountedEvent> {
    #[cfg(not(any(feature = "web", feature = "desktop", test)))]
    let _ = onoutside;
    let state = use_hook(|| {
        Rc::new(RefCell::new(OutsideClick {
            alive: true,
            mounted: false,
            enabled,
            root_id: root_id.clone(),
            generation: 0,
            #[cfg(any(feature = "web", feature = "desktop", test))]
            onoutside,
            #[cfg(any(feature = "web", feature = "desktop", test))]
            document: try_consume_context::<Rc<dyn dioxus::document::Document>>(),
            #[cfg(any(feature = "web", feature = "desktop", test))]
            session: None,
        }))
    });
    {
        let mut state = state.borrow_mut();
        if state.root_id != root_id || (state.enabled && !enabled) {
            state.stop();
        }
        state.root_id.clone_from(&root_id);
        state.enabled = enabled;
        #[cfg(any(feature = "web", feature = "desktop", test))]
        {
            state.onoutside = onoutside;
        }
    }
    let effect_state = state.clone();
    use_effect(use_reactive((&root_id, &enabled), move |_| {
        OutsideClick::start(&effect_state);
    }));
    let cleanup = state.clone();
    use_drop(move || {
        let mut state = cleanup.borrow_mut();
        state.alive = false;
        state.stop();
    });
    use_callback(move |_: MountedEvent| {
        state.borrow_mut().mounted = true;
        OutsideClick::start(&state);
    })
}

#[cfg(any(feature = "web", feature = "desktop", test))]
const OUTSIDE_CLICK: &str = r#"
const root = document.getElementById(await dioxus.recv());
if (!root?.isConnected) return;
const outside = event => { if (!event.composedPath().includes(root)) dioxus.send(null); };
document.addEventListener('click', outside, true);
try { await dioxus.recv(); } finally { document.removeEventListener('click', outside, true); }
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::document::{Document, Eval, EvalError, Evaluator};
    use std::{
        cell::Cell,
        collections::VecDeque,
        task::{Context, Poll, Waker},
    };

    #[derive(Default)]
    struct Channel {
        inputs: RefCell<Vec<serde_json::Value>>,
        events: RefCell<VecDeque<serde_json::Value>>,
        waker: RefCell<Option<Waker>>,
    }
    impl Channel {
        fn click(&self) {
            self.events.borrow_mut().push_back(serde_json::Value::Null);
            if let Some(waker) = self.waker.borrow_mut().take() {
                waker.wake();
            }
        }
    }
    struct Backend(Rc<Channel>);
    impl Evaluator for Backend {
        fn send(&self, value: serde_json::Value) -> Result<(), EvalError> {
            self.0.inputs.borrow_mut().push(value);
            Ok(())
        }
        fn poll_recv(
            &mut self,
            cx: &mut Context<'_>,
        ) -> Poll<Result<serde_json::Value, EvalError>> {
            if let Some(value) = self.0.events.borrow_mut().pop_front() {
                Poll::Ready(Ok(value))
            } else {
                *self.0.waker.borrow_mut() = Some(cx.waker().clone());
                Poll::Pending
            }
        }
        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            Poll::Pending
        }
    }
    #[derive(Default)]
    struct NativeDocument {
        owner: dioxus::signals::Owner,
        channels: RefCell<Vec<Rc<Channel>>>,
    }
    impl Document for NativeDocument {
        fn eval(&self, _: String) -> Eval {
            let channel = Rc::new(Channel::default());
            self.channels.borrow_mut().push(channel.clone());
            Eval::new(
                self.owner
                    .insert(Box::new(Backend(channel)) as Box<dyn Evaluator>),
            )
        }
    }
    #[derive(Clone, Default)]
    struct Fixture {
        show: Rc<Cell<Option<Signal<bool>>>>,
        enabled: Rc<Cell<Option<Signal<bool>>>>,
        root: Rc<Cell<Option<Signal<&'static str>>>>,
        mounted: Rc<Cell<Option<EventHandler<MountedEvent>>>>,
        clicks: Rc<RefCell<Vec<String>>>,
    }
    impl PartialEq for Fixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.clicks, &other.clicks)
        }
    }
    #[component]
    fn Consumer(fixture: Fixture, root: String, enabled: bool) -> Element {
        let current = root.clone();
        let clicks = fixture.clicks.clone();
        let mounted = use_outside_click(
            root.clone(),
            enabled,
            EventHandler::new(move |_| {
                clicks.borrow_mut().push(current.clone());
            }),
        );
        fixture.mounted.set(Some(mounted));
        rsx! { div { id: root, onmounted: mounted } }
    }
    fn fixture(state: Fixture) -> Element {
        let show = use_signal(|| true);
        let enabled = use_signal(|| true);
        let root = use_signal(|| "first");
        state.show.set(Some(show));
        state.enabled.set(Some(enabled));
        state.root.set(Some(root));
        rsx! { if show() { Consumer { fixture: state, root: root(), enabled: enabled() } } }
    }
    fn settle(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
    }
    fn mount(dom: &VirtualDom, fixture: &Fixture) {
        dom.in_runtime(|| {
            fixture
                .mounted
                .get()
                .unwrap()
                .call(Event::new(Rc::new(MountedData::new(())), false))
        });
    }

    #[test]
    fn actual_hook_starts_after_mount_and_retires_disabled_retargeted_and_removed_streams() {
        let document = Rc::new(NativeDocument::default());
        let provider: Rc<dyn Document> = document.clone();
        let state = Fixture::default();
        let mut dom =
            VirtualDom::new_with_props(fixture, state.clone()).with_root_context(provider);
        dom.rebuild_in_place();
        settle(&mut dom);
        assert!(
            document.channels.borrow().is_empty(),
            "render alone never binds document events"
        );
        mount(&dom, &state);
        settle(&mut dom);
        let first = document.channels.borrow()[0].clone();
        assert_eq!(*first.inputs.borrow(), [serde_json::json!("first")]);
        first.click();
        settle(&mut dom);
        assert_eq!(*state.clicks.borrow(), ["first"]);

        first.click();
        dom.in_scope(ScopeId::ROOT, || state.enabled.get().unwrap().set(false));
        settle(&mut dom);
        assert_eq!(
            first.inputs.borrow().last(),
            Some(&serde_json::json!("finish"))
        );
        let delivered = state.clicks.borrow().len();
        first.click();
        settle(&mut dom);
        assert_eq!(
            state.clicks.borrow().len(),
            delivered,
            "disabled subscription cannot deliver queued facts"
        );
        dom.in_scope(ScopeId::ROOT, || state.enabled.get().unwrap().set(true));
        settle(&mut dom);
        assert_eq!(document.channels.borrow().len(), 2);
        let second = document.channels.borrow()[1].clone();
        second.click();
        settle(&mut dom);
        assert_eq!(state.clicks.borrow().len(), delivered + 1);

        dom.in_scope(ScopeId::ROOT, || {
            state.root.get().unwrap().set("replacement")
        });
        settle(&mut dom);
        assert_eq!(
            second.inputs.borrow().last(),
            Some(&serde_json::json!("finish"))
        );
        let replacement = document.channels.borrow()[2].clone();
        assert_eq!(
            *replacement.inputs.borrow(),
            [serde_json::json!("replacement")]
        );
        first.click();
        second.click();
        replacement.click();
        settle(&mut dom);
        assert_eq!(
            state.clicks.borrow().last().map(String::as_str),
            Some("replacement")
        );
        assert_eq!(
            state.clicks.borrow().len(),
            delivered + 2,
            "old roots never retarget the new callback"
        );

        dom.in_scope(ScopeId::ROOT, || state.show.get().unwrap().set(false));
        settle(&mut dom);
        assert_eq!(
            replacement.inputs.borrow().last(),
            Some(&serde_json::json!("finish"))
        );
        replacement.click();
        settle(&mut dom);
        assert_eq!(state.clicks.borrow().len(), delivered + 2);
        dom.in_scope(ScopeId::ROOT, || state.show.get().unwrap().set(true));
        settle(&mut dom);
        assert_eq!(
            document.channels.borrow().len(),
            3,
            "remount also waits for onmounted"
        );
        mount(&dom, &state);
        settle(&mut dom);
        let remounted = document.channels.borrow()[3].clone();
        remounted.click();
        settle(&mut dom);
        assert_eq!(state.clicks.borrow().len(), delivered + 3);
        drop(dom);
        assert_eq!(
            remounted.inputs.borrow().last(),
            Some(&serde_json::json!("finish"))
        );
    }
}
