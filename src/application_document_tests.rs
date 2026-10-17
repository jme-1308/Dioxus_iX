use super::*;
use dioxus::core::internal::generational_box::{Owner, UnsyncStorage};
use dioxus::document::{Document, Eval, EvalError, Evaluator};
use std::collections::VecDeque;
use std::task::{Context, Poll, Waker};

#[derive(Default)]
struct DocumentState {
    evaluation_scopes: Vec<ScopeId>,
    replies: VecDeque<serde_json::Value>,
    sent: Vec<serde_json::Value>,
    waiter: Option<Waker>,
}

struct FakeDocument {
    owner: Owner<UnsyncStorage>,
    state: Rc<RefCell<DocumentState>>,
}

impl Document for FakeDocument {
    fn eval(&self, _: String) -> Eval {
        self.state
            .borrow_mut()
            .evaluation_scopes
            .push(dioxus::core::current_scope_id());
        Eval::new(
            self.owner
                .insert(Box::new(FakeEvaluator(self.state.clone())) as Box<dyn Evaluator>),
        )
    }
}

struct FakeEvaluator(Rc<RefCell<DocumentState>>);

impl Evaluator for FakeEvaluator {
    fn send(&self, value: serde_json::Value) -> Result<(), EvalError> {
        self.0.borrow_mut().sent.push(value);
        Ok(())
    }

    fn poll_recv(
        &mut self,
        context: &mut Context<'_>,
    ) -> Poll<Result<serde_json::Value, EvalError>> {
        let mut state = self.0.borrow_mut();
        if let Some(value) = state.replies.pop_front() {
            Poll::Ready(Ok(value))
        } else {
            state.waiter = Some(context.waker().clone());
            Poll::Pending
        }
    }

    fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
        panic!("the document observer must consume its stream, not join the pending script")
    }
}

struct FixtureState {
    visible: bool,
    provider: Rc<dyn Document>,
    coordinator: Option<CategoryHoverCoordinator>,
    update: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    provider_removed: bool,
}

#[derive(Clone)]
struct Fixture(Rc<RefCell<FixtureState>>);

impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn document_fixture(fixture: Fixture) -> Element {
    fixture.0.borrow_mut().update = Some(dioxus::core::schedule_update());
    let visible = fixture.0.borrow().visible;
    rsx! {
        if visible {
            DescendantDocumentProvider { fixture }
        }
    }
}

#[component]
fn DescendantDocumentProvider(fixture: Fixture) -> Element {
    let provider = fixture.0.borrow().provider.clone();
    use_context_provider(move || provider);
    let coordinator = use_root_context(CategoryHoverCoordinator::default);
    fixture.0.borrow_mut().coordinator = Some(coordinator.clone());
    use_effect(move || coordinator.start_document_input());
    use_drop(move || fixture.0.borrow_mut().provider_removed = true);
    rsx! { div {} }
}

fn pump(dom: &mut VirtualDom) {
    for _ in 0..3 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
}

#[test]
fn document_observer_captures_descendant_provider_and_survives_until_root_drop() {
    let state = Rc::new(RefCell::new(DocumentState::default()));
    // Keep the evaluator owner alive independently of the provider component
    // and VDOM, so a finish send after root cancellation remains observable.
    let document = Rc::new(FakeDocument {
        owner: Owner::default(),
        state: state.clone(),
    });
    let fixture = Fixture(Rc::new(RefCell::new(FixtureState {
        visible: true,
        provider: document.clone(),
        coordinator: None,
        update: None,
        provider_removed: false,
    })));
    let mut dom = VirtualDom::new_with_props(document_fixture, fixture.clone());
    dom.rebuild(&mut dioxus::core::NoOpMutations);
    pump(&mut dom);

    assert!(
        dom.in_scope(ScopeId::ROOT, || {
            try_consume_context::<Rc<dyn Document>>().is_none()
        }),
        "the live renderer provider must exist below ROOT for this regression"
    );
    assert_eq!(state.borrow().evaluation_scopes, [ScopeId::ROOT]);
    let coordinator = fixture.0.borrow().coordinator.clone().unwrap();
    assert!(coordinator.1.borrow().owners.is_empty());
    assert!(!coordinator.keyboard());
    assert!(state.borrow().waiter.is_some());

    let update = {
        let mut fixture = fixture.0.borrow_mut();
        fixture.visible = false;
        fixture.update.clone().unwrap()
    };
    update();
    pump(&mut dom);
    assert!(fixture.0.borrow().provider_removed);
    assert!(
        state.borrow().sent.is_empty(),
        "removing the provider component must not finish the root observer"
    );

    let owner = coordinator.1.borrow().document_owner.clone();
    let waiter = {
        let mut state = state.borrow_mut();
        state.replies.push_back(serde_json::json!({
            "owner": owner,
            "entries": [
                {"sequence": 1, "kind": "keydown", "key": "Tab"},
                {"sequence": 2, "kind": "keydown", "key": "x"}
            ]
        }));
        state.waiter.take().unwrap()
    };
    waiter.wake();
    pump(&mut dom);
    assert!(coordinator.keyboard());
    assert_eq!(state.borrow().sent, [serde_json::json!({"ack": 2})]);
    assert_eq!(state.borrow().evaluation_scopes, [ScopeId::ROOT]);

    drop(dom);
    assert_eq!(
        state.borrow().sent,
        [serde_json::json!({"ack": 2}), serde_json::json!("finish")],
        "root teardown must finish the pending observer exactly once"
    );
    // This external owner remains alive through the VDOM drop above.
    drop(document);
}
