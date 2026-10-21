use super::*;
use dioxus::{
    document::{Document, Eval, Evaluator},
    prelude::*,
};
use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{Context, Poll, Waker},
};

#[derive(Default)]
struct Wire {
    scripts: Vec<String>,
    sent: Vec<serde_json::Value>,
    reply: Option<serde_json::Value>,
    native_error: Option<String>,
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
        if let Some(error) = &wire.native_error {
            return Poll::Ready(Ok(serde_json::json!({"value":null,"error":error})));
        }
        match &wire.reply {
            Some(reply) => Poll::Ready(Ok(serde_json::json!({"value":reply,"error":null}))),
            None => Poll::Pending,
        }
    }
    fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
        panic!("containment reads use the acknowledged one-shot receive channel")
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
        Poll::Pending => panic!("mock containment reply should be available"),
    }
}
#[test]
fn containment_reads_share_transport_and_send_root_and_optional_marker_as_data() {
    let (dom, wire) = environment();
    let id = "root\"};throw new Error('injected');//";
    let attribute = "data-owner\"value";
    let cases = [
        (true, true, Some(attribute)),
        (true, false, None),
        (false, false, Some(attribute)),
    ];
    dom.in_scope(ScopeId::ROOT, || {
        for (connected, inside, required_attribute) in cases {
            wire.borrow_mut().reply =
                Some(serde_json::json!({"connected":connected,"inside":inside,"active_id":"owned-control"}));
            assert_eq!(
                ready(read(id, required_attribute)).unwrap(),
                FocusContainment { connected, inside }
            );
        }
    });
    let wire = wire.borrow();
    assert_eq!(wire.scripts.len(), 3);
    assert!(wire.scripts.iter().all(|script| script == &wire.scripts[0]
        && !script.contains(id)
        && !script.contains(attribute)));
    assert_eq!(
        wire.sent,
        vec![
            serde_json::json!({"kind":"data","value":[id,attribute]}),
            serde_json::json!({"kind":"finish"}),
            serde_json::json!({"kind":"data","value":[id,null]}),
            serde_json::json!({"kind":"finish"}),
            serde_json::json!({"kind":"data","value":[id,attribute]}),
            serde_json::json!({"kind":"finish"}),
        ]
    );
}
#[test]
fn containment_errors_remain_errors_and_all_failed_channels_are_closed() {
    let (dom, wire) = environment();
    dom.in_scope(ScopeId::ROOT,|| {
        wire.borrow_mut().native_error=Some("read failed".into());
        assert!(matches!(ready(read("root",None)),Err(EvalError::InvalidJs(error)) if error=="read failed"));
        wire.borrow_mut().native_error=None;
        wire.borrow_mut().reply=Some(serde_json::json!({"inside":true}));
        assert!(matches!(ready(read("root",None)),Err(EvalError::Serialization(_))));
        wire.borrow_mut().fail_recv=true;
        assert!(matches!(ready(read("root",None)),Err(EvalError::Finished)));
        wire.borrow_mut().fail_recv=false;
        wire.borrow_mut().fail_send=true;
        assert!(matches!(ready(read("root",None)),Err(EvalError::Finished)));
    });
    assert_eq!(
        wire.borrow()
            .sent
            .iter()
            .filter(|message| message["kind"] == "finish")
            .count(),
        4
    );
}
#[test]
fn cancelled_pending_containment_read_releases_the_live_channel() {
    let (dom, wire) = environment();
    dom.in_scope(ScopeId::ROOT, || {
        let mut future = Box::pin(read("category", Some("data-menu-category-root")));
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
            serde_json::json!({"kind":"data","value":["category","data-menu-category-root"]}),
            serde_json::json!({"kind":"finish"}),
        ]
    );
}

#[test]
fn active_only_query_uses_nullable_root_and_returns_arbitrary_or_empty_identity() {
    let (dom, wire) = environment();
    let unusual_id = "opener\"};throw new Error('injected');//";
    dom.in_scope(ScopeId::ROOT, || {
        for id in [unusual_id, ""] {
            wire.borrow_mut().reply = Some(serde_json::json!({
                "connected": false, "inside": false, "active_id": id
            }));
            assert_eq!(ready(active_id()).unwrap(), id);
        }
    });
    let wire = wire.borrow();
    assert_eq!(
        wire.sent,
        vec![
            serde_json::json!({"kind":"data","value":[null,null]}),
            serde_json::json!({"kind":"finish"}),
            serde_json::json!({"kind":"data","value":[null,null]}),
            serde_json::json!({"kind":"finish"}),
        ]
    );
    assert_eq!(wire.scripts.len(), 2);
    assert_eq!(wire.scripts[0], wire.scripts[1]);
    assert!(!wire.scripts[0].contains(unusual_id));
}

#[test]
fn active_identity_failures_and_retirement_close_the_same_acknowledged_channel() {
    let (dom, wire) = environment();
    dom.in_scope(ScopeId::ROOT, || {
        wire.borrow_mut().native_error = Some("active read failed".into());
        assert!(matches!(ready(active_id()), Err(EvalError::InvalidJs(error)) if error == "active read failed"));
        wire.borrow_mut().native_error = None;
        wire.borrow_mut().reply = Some(serde_json::json!({"connected":false,"inside":false,"active_id":null}));
        assert!(matches!(ready(active_id()), Err(EvalError::Serialization(_))));
        wire.borrow_mut().reply = None;
        let mut future = Box::pin(active_id());
        assert!(future.as_mut().poll(&mut Context::from_waker(Waker::noop())).is_pending());
        drop(future);
    });
    assert_eq!(
        wire.borrow()
            .sent
            .iter()
            .filter(|message| message["kind"] == "finish")
            .count(),
        3
    );
}
