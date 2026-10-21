use super::*;
use dioxus::document::{Document, Eval, EvalError, Evaluator};
use std::{
    future::Future,
    task::{Context, Poll, Waker},
};

#[derive(Default)]
struct Wire {
    scripts: Vec<String>,
    sent: Vec<serde_json::Value>,
    reply: Option<serde_json::Value>,
    error: Option<String>,
    fail_send: bool,
    fail_recv: bool,
}
struct Backend(Rc<RefCell<Wire>>);
impl Evaluator for Backend {
    fn send(&self, value: serde_json::Value) -> Result<(), EvalError> {
        let mut wire = self.0.borrow_mut();
        wire.sent.push(value);
        if wire.fail_send {
            Err(EvalError::Finished)
        } else {
            Ok(())
        }
    }
    fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
        let wire = self.0.borrow();
        if wire.fail_recv {
            return Poll::Ready(Err(EvalError::Finished));
        }
        if let Some(error) = &wire.error {
            return Poll::Ready(Ok(serde_json::json!({"value":null,"error":error})));
        }
        match &wire.reply {
            Some(reply) => Poll::Ready(Ok(serde_json::json!({"value":reply,"error":null}))),
            None => Poll::Pending,
        }
    }
    fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
        panic!("native facts use the acknowledged receive channel")
    }
}
struct NativeDocument {
    owner: dioxus::signals::Owner,
    wire: Rc<RefCell<Wire>>,
}
impl Document for NativeDocument {
    fn eval(&self, script: String) -> Eval {
        self.wire.borrow_mut().scripts.push(script);
        Eval::new(
            self.owner
                .insert(Box::new(Backend(self.wire.clone())) as Box<dyn Evaluator>),
        )
    }
}
fn environment() -> (VirtualDom, Rc<RefCell<Wire>>) {
    let wire = Rc::new(RefCell::new(Wire::default()));
    let document: Rc<dyn Document> = Rc::new(NativeDocument {
        owner: Default::default(),
        wire: wire.clone(),
    });
    let mut dom = VirtualDom::new(|| rsx! {}).with_root_context(document);
    dom.rebuild_in_place();
    (dom, wire)
}
fn ready<T>(future: impl Future<Output = T>) -> T {
    match std::pin::pin!(future)
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("mock facts should be available"),
    }
}
fn facts(form: bool) -> serde_json::Value {
    serde_json::json!({
        "validity":{"bad_input":true,"custom_error":false,"pattern_mismatch":false,
            "range_overflow":false,"range_underflow":true,"step_mismatch":true,"too_long":false,
            "too_short":false,"type_mismatch":false,"value_missing":false,"valid":false},
        "no_validate":form,"validation_message":"Enter a valid reading.",
        "form":if form {serde_json::json!({"id":"external-form","name":"equipment","method":"post","action":"https://example.test/save","no_validate":true})} else {serde_json::Value::Null}
    })
}
#[test]
fn text_validity_and_both_form_queries_share_typed_facts_and_data_only_identifiers() {
    let (dom, wire) = environment();
    let id = "input\"};throw new Error('injected');//";
    wire.borrow_mut().reply = Some(facts(true));
    dom.in_scope(ScopeId::ROOT, || {
        let snapshot = ready(read_snapshot(id)).unwrap();
        assert_eq!(
            snapshot.validity,
            NativeValidityState {
                bad_input: true,
                range_underflow: true,
                step_mismatch: true,
                valid: false,
                ..Default::default()
            }
        );
        assert!(snapshot.no_validate);
        assert_eq!(snapshot.validation_message, "Enter a valid reading.");
        assert_eq!(
            ready(read_associated_form(id, NativeFormControl::Text)).unwrap(),
            Some(NativeFormElement {
                id: "external-form".into(),
                name: "equipment".into(),
                method: "post".into(),
                action: "https://example.test/save".into(),
                no_validate: true
            })
        );
        wire.borrow_mut().reply = Some(facts(false));
        assert_eq!(
            ready(read_associated_form(id, NativeFormControl::Select)).unwrap(),
            None
        );
    });
    let wire = wire.borrow();
    assert_eq!(wire.scripts.len(), 3);
    assert!(
        wire.scripts
            .iter()
            .all(|script| script == &wire.scripts[0] && !script.contains(id))
    );
    assert_eq!(
        wire.sent,
        vec![
            serde_json::json!({"kind":"data","value":[id,true]}),
            serde_json::json!({"kind":"finish"}),
            serde_json::json!({"kind":"data","value":[id,true]}),
            serde_json::json!({"kind":"finish"}),
            serde_json::json!({"kind":"data","value":[id,false]}),
            serde_json::json!({"kind":"finish"}),
        ]
    );
}
#[test]
fn native_schema_and_transport_errors_keep_existing_caller_errors_and_close_channels() {
    let (dom, wire) = environment();
    dom.in_scope(ScopeId::ROOT, || {
        wire.borrow_mut().error = Some("Text control is not mounted".into());
        assert_eq!(
            ready(read_snapshot("text")).err(),
            Some(TextControlError::Dom(
                EvalError::InvalidJs("Text control is not mounted".into()).to_string()
            ))
        );
        wire.borrow_mut().error = Some("Select input is not mounted".into());
        assert_eq!(
            ready(read_associated_form("textarea", NativeFormControl::Select)).unwrap_err(),
            EvalError::InvalidJs("Select input is not mounted".into()).to_string()
        );
        wire.borrow_mut().error = None;
        wire.borrow_mut().reply = Some(serde_json::json!({"form":null}));
        assert!(
            ready(read_snapshot("text")).is_err(),
            "partial native facts are not invented as valid"
        );
        wire.borrow_mut().fail_recv = true;
        assert!(ready(read_associated_form("select", NativeFormControl::Select)).is_err());
        wire.borrow_mut().fail_recv = false;
        wire.borrow_mut().fail_send = true;
        assert!(matches!(
            ready(read_snapshot("text")),
            Err(TextControlError::Dom(_))
        ));
    });
    assert_eq!(
        wire.borrow()
            .sent
            .iter()
            .filter(|message| message["kind"] == "finish")
            .count(),
        5
    );
}
#[test]
fn cancelling_pending_native_facts_releases_the_live_channel() {
    let (dom, wire) = environment();
    dom.in_scope(ScopeId::ROOT, || {
        let mut future = Box::pin(read_associated_form("input", NativeFormControl::Text));
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        drop(future);
    });
    assert_eq!(
        wire.borrow().sent,
        vec![
            serde_json::json!({"kind":"data","value":["input",true]}),
            serde_json::json!({"kind":"finish"})
        ]
    );
}
