// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Shared native viewport facts, one stream per Document in a Dioxus root. Pinned alpha.2
//! MountedData measures elements but exposes neither the viewport/visual viewport
//! box nor document/window scroll, resize or Escape subscriptions. This adapter
//! supplies only those missing facts; Rust owns subscribers and their lifetime.

use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
pub(crate) struct DocumentViewportFacts {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) visual_left: f64,
    pub(crate) visual_top: f64,
    pub(crate) visual_width: f64,
    pub(crate) visual_height: f64,
}
impl DocumentViewportFacts {
    #[cfg(any(feature = "web", feature = "desktop", test))]
    fn valid(self) -> bool {
        [
            self.width,
            self.height,
            self.visual_left,
            self.visual_top,
            self.visual_width,
            self.visual_height,
        ]
        .into_iter()
        .all(f64::is_finite)
            && self.width > 0.0
            && self.height > 0.0
            && self.visual_width > 0.0
            && self.visual_height > 0.0
    }
}

#[cfg(any(feature = "web", feature = "desktop", test))]
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum NativeEvent {
    Layout {
        #[serde(flatten)]
        facts: DocumentViewportFacts,
    },
    Escape,
}
#[cfg(any(feature = "web", feature = "desktop", test))]
#[derive(serde::Deserialize)]
struct Observation {
    epoch: u64,
    sequence: u64,
    #[serde(flatten)]
    event: NativeEvent,
}

struct Model {
    alive: bool,
    active: usize,
    epoch: u64,
    #[cfg(any(feature = "web", feature = "desktop", test))]
    sequence: u64,
    facts: Signal<Option<DocumentViewportFacts>>,
    layout: Signal<u64>,
    escape: Signal<u64>,
    renderer: Weak<dioxus::core::Runtime>,
    document: Option<Weak<dyn dioxus::document::Document>>,
    #[cfg(any(feature = "web", feature = "desktop"))]
    eval: Option<dioxus::document::Eval>,
    #[cfg(any(feature = "web", feature = "desktop"))]
    task: Option<dioxus::core::Task>,
}
impl Model {
    fn document_alive(&self) -> bool {
        self.document
            .as_ref()
            .is_none_or(|document| document.upgrade().is_some())
    }
    #[cfg(any(feature = "web", feature = "desktop", test))]
    fn observe(&mut self, observation: Observation) -> bool {
        if !self.alive
            || !self.document_alive()
            || self.active == 0
            || observation.epoch != self.epoch
            || observation.sequence <= self.sequence
        {
            return false;
        }
        self.sequence = observation.sequence;
        self.apply(observation.event)
    }

    #[cfg(any(feature = "web", feature = "desktop", test))]
    fn apply(&mut self, event: NativeEvent) -> bool {
        match event {
            NativeEvent::Layout { facts } if facts.valid() => {
                self.facts.set(Some(facts));
                let next = self.layout.peek().wrapping_add(1);
                self.layout.set(next);
            }
            NativeEvent::Escape => {
                let next = self.escape.peek().wrapping_add(1);
                self.escape.set(next);
            }
            _ => return false,
        }
        true
    }

    fn stop(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        #[cfg(any(feature = "web", feature = "desktop", test))]
        {
            self.sequence = 0;
        }
        // Preserve the last box for inactive readers; a new session supplies a
        // fresh initial measurement. Epochs retire facts still in the channel.
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            if let Some(eval) = self.eval.take() {
                let _ = eval.send("finish");
            }
            if let Some(task) = self.task.take() {
                task.cancel();
            }
        }
    }
}

/// The root context owns shutdown independently of consumer/test handle clones.
/// The receive future holds Weak<Model>, so it cannot keep this owner alive.
#[derive(Clone)]
struct RootViewport(Rc<RefCell<Vec<Rc<RootOwner>>>>);
impl RootViewport {
    fn for_document(
        &self,
        document: Option<Rc<dyn dioxus::document::Document>>,
    ) -> DocumentViewport {
        let mut owners = self.0.borrow_mut();
        // A removed Window must not retain a stream or poison a later Window
        // that happens to reuse renderer scope IDs. There is no native cache.
        owners.retain(|owner| {
            owner
                .0
                .borrow()
                .document
                .as_ref()
                .is_none_or(|document| document.upgrade().is_some())
        });
        if let Some(owner) = owners.iter().find(|owner| {
            let model = owner.0.borrow();
            match (&model.document, &document) {
                (None, None) => true,
                (Some(old), Some(current)) => {
                    old.upgrade().is_some_and(|old| Rc::ptr_eq(&old, current))
                }
                _ => false,
            }
        }) {
            return DocumentViewport(owner.0.clone());
        }
        let model = Rc::new(RefCell::new(Model {
            alive: true,
            active: 0,
            epoch: 1,
            #[cfg(any(feature = "web", feature = "desktop", test))]
            sequence: 0,
            facts: Signal::new_in_scope(None, ScopeId::ROOT),
            layout: Signal::new_in_scope(0, ScopeId::ROOT),
            escape: Signal::new_in_scope(0, ScopeId::ROOT),
            renderer: Rc::downgrade(&dioxus::core::Runtime::current()),
            document: document.as_ref().map(Rc::downgrade),
            #[cfg(any(feature = "web", feature = "desktop"))]
            eval: None,
            #[cfg(any(feature = "web", feature = "desktop"))]
            task: None,
        }));
        owners.push(Rc::new(RootOwner(model.clone())));
        DocumentViewport(model)
    }
}
struct RootOwner(Rc<RefCell<Model>>);
impl Drop for RootOwner {
    fn drop(&mut self) {
        let mut model = self.0.borrow_mut();
        model.alive = false;
        model.active = 0;
        model.stop();
    }
}

#[derive(Clone)]
pub(crate) struct DocumentViewport(Rc<RefCell<Model>>);
impl DocumentViewport {
    pub(crate) fn facts(&self) -> Option<DocumentViewportFacts> {
        let model = self.0.borrow();
        if model.alive && model.document_alive() {
            (model.facts)()
        } else {
            None
        }
    }
    pub(crate) fn layout_revision(&self) -> u64 {
        let model = self.0.borrow();
        if model.alive && model.document_alive() {
            (model.layout)()
        } else {
            0
        }
    }
    pub(crate) fn escape_revision(&self) -> u64 {
        let model = self.0.borrow();
        if model.alive && model.document_alive() {
            (model.escape)()
        } else {
            0
        }
    }

    fn set_enabled(&self, lease: &Cell<bool>, enabled: bool) {
        if lease.replace(enabled) == enabled {
            return;
        }
        let start = {
            let mut model = self.0.borrow_mut();
            if !model.alive || (enabled && !model.document_alive()) {
                lease.set(false);
                return;
            }
            if enabled {
                model.active += 1;
                model.active == 1
            } else {
                model.active -= 1;
                if model.active == 0 {
                    model.stop();
                }
                false
            }
        };
        if start {
            self.start();
        }
    }

    fn start(&self) {
        let (renderer, document, epoch) = {
            let model = self.0.borrow();
            (
                model.renderer.upgrade(),
                model.document.as_ref().and_then(Weak::upgrade),
                model.epoch,
            )
        };
        let Some(renderer) = renderer else {
            return;
        };
        #[cfg(any(feature = "web", feature = "desktop"))]
        let Some(document) = document else {
            return;
        };
        #[cfg(any(feature = "web", feature = "desktop"))]
        renderer.in_scope(ScopeId::ROOT, || {
            // Desktop Window provides its Document below ROOT. Capture that
            // actual provider before entering ROOT; resolving it here would
            // produce NoOpDocument and also mix sibling Window viewports.
            let eval = document.eval(observer_script(epoch));
            let weak = Rc::downgrade(&self.0);
            let mut receiver = eval;
            let task = spawn(async move {
                while let Ok(observation) = receiver.recv::<Observation>().await {
                    let Some(model) = weak.upgrade() else {
                        break;
                    };
                    if !model.borrow_mut().observe(observation) {
                        let state = model.borrow();
                        if !state.alive
                            || !state.document_alive()
                            || state.epoch != epoch
                            || state.active == 0
                        {
                            break;
                        }
                    }
                }
            });
            let mut model = self.0.borrow_mut();
            model.eval = Some(eval);
            model.task = Some(task);
        });
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        let _ = (renderer, document, epoch);
    }

    #[cfg(test)]
    pub(crate) fn test_layout(&self, width: f64, height: f64) {
        self.test_event(NativeEvent::Layout {
            facts: DocumentViewportFacts {
                width,
                height,
                visual_left: 0.0,
                visual_top: 0.0,
                visual_width: width,
                visual_height: height,
            },
        });
    }
    #[cfg(test)]
    pub(crate) fn test_escape(&self) {
        self.test_event(NativeEvent::Escape);
    }
    #[cfg(test)]
    fn test_event(&self, event: NativeEvent) {
        let mut model = self.0.borrow_mut();
        // Composed SSR tests may seed the last-known box before opening the
        // first native consumer. This invokes the same typed state update, not
        // a native session or production fallback; real reports still pass
        // active/epoch/sequence admission above.
        if model.alive && model.document_alive() {
            model.apply(event);
        }
    }
}

pub(crate) fn use_document_viewport(enabled: bool) -> DocumentViewport {
    let root = use_root_context(|| RootViewport(Rc::new(RefCell::new(Vec::new()))));
    let viewport =
        use_hook(|| root.for_document(try_consume_context::<Rc<dyn dioxus::document::Document>>()));
    let lease = use_hook(|| Rc::new(Cell::new(false)));
    let effect_viewport = viewport.clone();
    let effect_lease = lease.clone();
    use_effect(use_reactive((&enabled,), move |(enabled,)| {
        effect_viewport.set_enabled(&effect_lease, enabled);
    }));
    let cleanup = viewport.clone();
    use_drop(move || {
        cleanup.set_enabled(&lease, false);
    });
    viewport
}

#[cfg(any(feature = "web", feature = "desktop"))]
fn observer_script(epoch: u64) -> String {
    format!(
        r#"
const epoch={epoch},listeners=new AbortController(),signal=listeners.signal;
let sequence=0;
const layout=()=>{{const v=window.visualViewport;dioxus.send({{epoch,sequence:++sequence,kind:'layout',
  width:document.documentElement.clientWidth,height:document.documentElement.clientHeight,
  visual_left:v?.offsetLeft??0,visual_top:v?.offsetTop??0,
  visual_width:v?.width??document.documentElement.clientWidth,visual_height:v?.height??document.documentElement.clientHeight}});}};
window.addEventListener('resize',layout,{{passive:true,signal}});
document.addEventListener('scroll',layout,{{capture:true,passive:true,signal}});
window.visualViewport?.addEventListener('resize',layout,{{passive:true,signal}});
window.visualViewport?.addEventListener('scroll',layout,{{passive:true,signal}});
document.addEventListener('keydown',event=>{{if(event.key==='Escape'&&!event.defaultPrevented)dioxus.send({{epoch,sequence:++sequence,kind:'escape'}});}},{{signal}});
try{{layout();await dioxus.recv();}}finally{{listeners.abort();}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct Capture {
        handles: Rc<RefCell<Vec<Option<DocumentViewport>>>>,
        show: Rc<Cell<Option<Signal<bool>>>>,
        enabled: Rc<Cell<Option<Signal<bool>>>>,
    }
    impl PartialEq for Capture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.handles, &other.handles)
        }
    }
    #[component]
    fn Consumer(capture: Capture, index: usize, enabled: bool) -> Element {
        let viewport = use_document_viewport(enabled);
        capture.handles.borrow_mut()[index] = Some(viewport.clone());
        let revision = viewport.layout_revision();
        rsx! { div { "{revision}" } }
    }
    fn fixture(capture: Capture) -> Element {
        let show = use_signal(|| true);
        let enabled = use_signal(|| true);
        capture.show.set(Some(show));
        capture.enabled.set(Some(enabled));
        let root = use_document_viewport(false);
        capture.handles.borrow_mut()[0] = Some(root);
        rsx! {
            if show() { Consumer { capture: capture.clone(), index: 1, enabled: true } }
            Consumer { capture, index: 2, enabled: enabled() }
        }
    }
    fn capture() -> Capture {
        Capture {
            handles: Rc::new(RefCell::new(vec![None, None, None])),
            ..Default::default()
        }
    }
    fn settle(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
    }
    fn handle(capture: &Capture) -> DocumentViewport {
        capture.handles.borrow()[0].as_ref().unwrap().clone()
    }

    #[test]
    fn consumers_share_root_facts_and_releasing_one_keeps_the_remaining_subscription() {
        let capture = capture();
        let mut dom = VirtualDom::new_with_props(fixture, capture.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let viewport = handle(&capture);
        for other in capture.handles.borrow().iter().flatten() {
            assert!(Rc::ptr_eq(&viewport.0, &other.0));
        }
        assert_eq!(viewport.0.borrow().active, 2);
        dom.in_scope(ScopeId::ROOT, || viewport.test_layout(1000.0, 700.0));
        assert_eq!(
            dom.in_scope(ScopeId::ROOT, || viewport.layout_revision()),
            1
        );
        let epoch = viewport.0.borrow().epoch;
        dom.in_scope(ScopeId::ROOT, || capture.show.get().unwrap().set(false));
        settle(&mut dom);
        assert_eq!(viewport.0.borrow().active, 1);
        assert_eq!(viewport.0.borrow().epoch, epoch);
        dom.in_scope(ScopeId::ROOT, || viewport.test_layout(1000.0, 700.0));
        assert_eq!(
            dom.in_scope(ScopeId::ROOT, || viewport.layout_revision()),
            2,
            "scroll invalidates even an unchanged box"
        );
        dom.in_scope(ScopeId::ROOT, || capture.enabled.get().unwrap().set(false));
        settle(&mut dom);
        assert_eq!(viewport.0.borrow().active, 0);
        assert_ne!(viewport.0.borrow().epoch, epoch);
        dom.in_scope(ScopeId::ROOT, || {
            let epoch = viewport.0.borrow().epoch;
            assert!(!viewport.0.borrow_mut().observe(Observation {
                epoch,
                sequence: 1,
                event: NativeEvent::Escape
            }));
            assert_eq!(
                viewport.escape_revision(),
                0,
                "inactive stream admits no native event"
            );
        });
        dom.in_scope(ScopeId::ROOT, || capture.enabled.get().unwrap().set(true));
        settle(&mut dom);
        assert_eq!(viewport.0.borrow().active, 1);
        let active_epoch = viewport.0.borrow().epoch;
        dom.in_scope(ScopeId::ROOT, || {
            assert!(!viewport.0.borrow_mut().observe(Observation {
                epoch,
                sequence: 999,
                event: NativeEvent::Escape
            }));
            assert!(viewport.0.borrow_mut().observe(Observation {
                epoch: active_epoch,
                sequence: 1,
                event: NativeEvent::Escape
            }));
            assert_eq!(viewport.escape_revision(), 1);
            assert!(!viewport.0.borrow_mut().observe(Observation {
                epoch: active_epoch,
                sequence: 1,
                event: NativeEvent::Escape
            }));
            assert_eq!(
                viewport.escape_revision(),
                1,
                "duplicate sequence cannot replay Escape"
            );
        });
        drop(dom);
        assert!(
            !viewport.0.borrow().alive,
            "retained handle cannot keep a removed root live"
        );
        assert_eq!(viewport.0.borrow().active, 0);
        assert_eq!(viewport.facts(), None);
        assert_eq!(viewport.layout_revision(), 0);
    }

    #[test]
    fn independent_roots_isolate_boxes_events_and_teardown() {
        let first = capture();
        let second = capture();
        let mut a = VirtualDom::new_with_props(fixture, first.clone());
        let mut b = VirtualDom::new_with_props(fixture, second.clone());
        a.rebuild_in_place();
        b.rebuild_in_place();
        settle(&mut a);
        settle(&mut b);
        let va = handle(&first);
        let vb = handle(&second);
        assert!(!Rc::ptr_eq(&va.0, &vb.0));
        a.in_scope(ScopeId::ROOT, || {
            va.test_layout(900.0, 600.0);
            va.test_escape();
        });
        b.in_scope(ScopeId::ROOT, || {
            assert_eq!(vb.facts(), None);
            assert_eq!(vb.escape_revision(), 0);
            vb.test_layout(500.0, 400.0);
        });
        drop(a);
        assert!(!va.0.borrow().alive);
        assert!(vb.0.borrow().alive);
        b.in_scope(ScopeId::ROOT, || {
            vb.test_escape();
            assert_eq!(vb.escape_revision(), 1);
            assert_eq!(vb.facts().unwrap().width, 500.0);
        });
    }

    #[cfg(any(feature = "web", feature = "desktop"))]
    #[test]
    fn one_native_stream_survives_first_consumer_and_stops_then_restarts_once() {
        use dioxus::document::{Document, Eval, EvalError, Evaluator};
        use std::task::{Context, Poll};
        #[derive(Default)]
        struct Counts {
            starts: Cell<usize>,
            stops: Cell<usize>,
        }
        struct Backend(Rc<Counts>);
        impl Evaluator for Backend {
            fn send(&self, value: serde_json::Value) -> Result<(), EvalError> {
                assert_eq!(value, "finish");
                self.0.stops.set(self.0.stops.get() + 1);
                Ok(())
            }
            fn poll_recv(
                &mut self,
                _: &mut Context<'_>,
            ) -> Poll<Result<serde_json::Value, EvalError>> {
                Poll::Pending
            }
            fn poll_join(
                &mut self,
                _: &mut Context<'_>,
            ) -> Poll<Result<serde_json::Value, EvalError>> {
                Poll::Pending
            }
        }
        struct NativeDocument {
            owner: dioxus::signals::Owner,
            counts: Rc<Counts>,
        }
        impl Document for NativeDocument {
            fn eval(&self, script: String) -> Eval {
                assert_eq!(
                    dioxus::core::current_scope_id(),
                    ScopeId::ROOT,
                    "the first consumer must not own eval"
                );
                assert!(script.contains("AbortController"));
                self.counts.starts.set(self.counts.starts.get() + 1);
                Eval::new(
                    self.owner
                        .insert(Box::new(Backend(self.counts.clone())) as Box<dyn Evaluator>),
                )
            }
        }
        let counts = Rc::new(Counts::default());
        let document: Rc<dyn Document> = Rc::new(NativeDocument {
            owner: Default::default(),
            counts: counts.clone(),
        });
        let capture = capture();
        let mut dom =
            VirtualDom::new_with_props(fixture, capture.clone()).with_root_context(document);
        dom.rebuild_in_place();
        settle(&mut dom);
        assert_eq!((counts.starts.get(), counts.stops.get()), (1, 0));
        dom.in_scope(ScopeId::ROOT, || capture.show.get().unwrap().set(false));
        settle(&mut dom);
        assert_eq!((counts.starts.get(), counts.stops.get()), (1, 0));
        dom.in_scope(ScopeId::ROOT, || capture.enabled.get().unwrap().set(false));
        settle(&mut dom);
        assert_eq!((counts.starts.get(), counts.stops.get()), (1, 1));
        dom.in_scope(ScopeId::ROOT, || capture.enabled.get().unwrap().set(true));
        settle(&mut dom);
        assert_eq!((counts.starts.get(), counts.stops.get()), (2, 1));
        drop(dom);
        assert_eq!((counts.starts.get(), counts.stops.get()), (2, 2));
    }
    #[cfg(any(feature = "web", feature = "desktop"))]
    #[derive(Clone)]
    struct ScopedDocuments {
        documents: [Rc<dyn dioxus::document::Document>; 2],
        captures: [Capture; 3],
        show_first: Rc<Cell<Option<Signal<bool>>>>,
    }
    #[cfg(any(feature = "web", feature = "desktop"))]
    impl PartialEq for ScopedDocuments {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.show_first, &other.show_first)
        }
    }
    #[cfg(any(feature = "web", feature = "desktop"))]
    #[component]
    fn ScopedWindow(state: ScopedDocuments, index: usize, document_index: usize) -> Element {
        use_context_provider(|| state.documents[document_index].clone());
        fixture(state.captures[index].clone())
    }
    #[cfg(any(feature = "web", feature = "desktop"))]
    fn sibling_windows(state: ScopedDocuments) -> Element {
        let show = use_signal(|| true);
        state.show_first.set(Some(show));
        rsx! {
            if show() { ScopedWindow { state: state.clone(), index: 0, document_index: 0 } }
            ScopedWindow { state: state.clone(), index: 1, document_index: 1 }
            ScopedWindow { state, index: 2, document_index: 0 }
        }
    }

    #[cfg(any(feature = "web", feature = "desktop"))]
    #[test]
    fn sibling_window_document_providers_are_captured_below_root_and_remain_isolated() {
        use dioxus::document::{Document, Eval, EvalError, Evaluator};
        use std::task::{Context, Poll};
        #[derive(Default)]
        struct Counts {
            starts: Cell<usize>,
            stops: Cell<usize>,
        }
        struct Backend(Rc<Counts>);
        impl Evaluator for Backend {
            fn send(&self, value: serde_json::Value) -> Result<(), EvalError> {
                assert_eq!(value, "finish");
                self.0.stops.set(self.0.stops.get() + 1);
                Ok(())
            }
            fn poll_recv(
                &mut self,
                _: &mut Context<'_>,
            ) -> Poll<Result<serde_json::Value, EvalError>> {
                Poll::Pending
            }
            fn poll_join(
                &mut self,
                _: &mut Context<'_>,
            ) -> Poll<Result<serde_json::Value, EvalError>> {
                Poll::Pending
            }
        }
        struct NativeDocument {
            owner: dioxus::signals::Owner,
            counts: Rc<Counts>,
        }
        impl Document for NativeDocument {
            fn eval(&self, _: String) -> Eval {
                assert_eq!(dioxus::core::current_scope_id(), ScopeId::ROOT);
                assert!(
                    try_consume_context::<Rc<dyn Document>>().is_none(),
                    "regression setup: ROOT has no Window Document"
                );
                self.counts.starts.set(self.counts.starts.get() + 1);
                Eval::new(
                    self.owner
                        .insert(Box::new(Backend(self.counts.clone())) as Box<dyn Evaluator>),
                )
            }
        }
        let counts = [Rc::new(Counts::default()), Rc::new(Counts::default())];
        let documents: [Rc<dyn Document>; 2] = std::array::from_fn(|index| {
            Rc::new(NativeDocument {
                owner: Default::default(),
                counts: counts[index].clone(),
            }) as Rc<dyn Document>
        });
        let state = ScopedDocuments {
            documents,
            captures: [capture(), capture(), capture()],
            show_first: Rc::default(),
        };
        let mut dom = VirtualDom::new_with_props(sibling_windows, state.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let first = handle(&state.captures[0]);
        let second = handle(&state.captures[1]);
        assert!(!Rc::ptr_eq(&first.0, &second.0));
        let same_document = handle(&state.captures[2]);
        assert!(
            Rc::ptr_eq(&first.0, &same_document.0),
            "separate provider scopes sharing one Document reuse one stream"
        );
        assert_eq!(
            (counts[0].starts.get(), counts[1].starts.get()),
            (1, 1),
            "one stream per actual Window Document"
        );
        dom.in_scope(ScopeId::ROOT, || {
            let cache = consume_context::<RootViewport>();
            let transient: Rc<dyn Document> = Rc::new(NativeDocument {
                owner: Default::default(),
                counts: Rc::new(Counts::default()),
            });
            let expired = cache.for_document(Some(transient.clone()));
            expired.test_layout(300.0, 200.0);
            expired.test_escape();
            assert_eq!(expired.facts().unwrap().width, 300.0);
            drop(transient);
            assert_eq!(
                expired.facts(),
                None,
                "expired Window cannot expose retained facts before pruning"
            );
            assert_eq!(expired.layout_revision(), 0);
            assert_eq!(expired.escape_revision(), 0);
            cache.for_document(None);
            assert!(
                !expired.0.borrow().alive,
                "next subscription prunes dead Document owners"
            );
        });
        dom.in_scope(ScopeId::ROOT, || {
            first.test_layout(1000.0, 850.0);
            first.test_escape();
            second.test_layout(500.0, 400.0);
            assert_eq!(first.escape_revision(), 1);
            assert_eq!(second.escape_revision(), 0);
            assert_eq!(first.facts().unwrap().width, 1000.0);
            assert_eq!(second.facts().unwrap().width, 500.0);
            state.show_first.get().unwrap().set(false);
        });
        settle(&mut dom);
        assert_eq!(
            (counts[0].stops.get(), counts[1].stops.get()),
            (0, 0),
            "removing first provider scope must not cancel peers for that Document"
        );
        assert_eq!(same_document.0.borrow().active, 2);
        assert_eq!(
            second.0.borrow().active,
            2,
            "closing one scope preserves the other Document subscribers"
        );
        dom.in_scope(ScopeId::ROOT, || {
            state.captures[2].show.get().unwrap().set(false);
            state.captures[2].enabled.get().unwrap().set(false);
        });
        settle(&mut dom);
        assert_eq!((counts[0].stops.get(), counts[1].stops.get()), (1, 0));
        drop(dom);
        assert_eq!((counts[0].stops.get(), counts[1].stops.get()), (1, 1));
        assert!(!first.0.borrow().alive);
        assert!(!second.0.borrow().alive);
    }
}
