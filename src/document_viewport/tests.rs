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
fn media_only_changes_invalidate_layout_without_replacing_geometry_or_inventing_matches() {
    let capture = capture();
    let mut dom = VirtualDom::new_with_props(fixture, capture.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let viewport = handle(&capture);
    dom.in_scope(ScopeId::ROOT, || {
        let mut facts: DocumentViewportFacts = serde_json::from_value(serde_json::json!({
            "width":1000.0,"height":700.0,"visual_left":0.0,"visual_top":0.0,
            "visual_width":1000.0,"visual_height":700.0
        }))
        .unwrap();
        assert_eq!(
            facts.media, None,
            "missing media does not invent query matches"
        );
        viewport.test_facts(facts);
        assert_eq!(
            viewport.facts(),
            Some(facts),
            "unavailable media must not block geometry"
        );
        facts.media = Some(DocumentMediaFacts {
            sm: true,
            md: false,
            lg: false,
        });
        viewport.test_facts(facts);
        let before = viewport.layout_revision();
        facts.media = Some(DocumentMediaFacts {
            sm: false,
            md: false,
            lg: true,
        });
        viewport.test_facts(facts);
        assert_eq!(
            viewport.layout_revision(),
            before + 1,
            "media-only changes must notify existing consumers"
        );
        assert_eq!(viewport.facts().unwrap().width, 1000.0);
        assert_eq!(viewport.facts().unwrap().media, facts.media);
        viewport.test_layout(360.0, 700.0);
        assert_eq!(
            viewport.facts().unwrap().media,
            facts.media,
            "layout helper preserves raw media; it does not infer CSS em matches from width"
        );
        facts.media = None;
        viewport.test_facts(facts);
        assert_eq!(
            viewport.facts(),
            Some(facts),
            "media becoming unavailable keeps valid layout facts"
        );
    });
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
        fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            Poll::Pending
        }
        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
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
    let mut dom = VirtualDom::new_with_props(fixture, capture.clone()).with_root_context(document);
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
        fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            Poll::Pending
        }
        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
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
