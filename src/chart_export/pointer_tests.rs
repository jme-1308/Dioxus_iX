use super::*;
use dioxus::{
    document::{Document, Eval, EvalError, Evaluator},
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
    reply: serde_json::Value,
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
        Poll::Ready(if wire.fail_recv {
            Err(EvalError::Finished)
        } else {
            Ok(serde_json::json!({"value":wire.reply,"error":null}))
        })
    }
    fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
        panic!("chart operations receive and acknowledge the live one-shot channel")
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
    let wire = Rc::new(RefCell::new(Wire {
        reply: serde_json::json!({}),
        ..Default::default()
    }));
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
        Poll::Pending => panic!("mock chart reply should be immediately available"),
    }
}
#[test]
fn capture_and_release_share_transport_and_serialize_element_identity_as_data() {
    let (dom, wire) = environment();
    let id = "plot\"};throw new Error('injected');//";
    dom.in_scope(ScopeId::ROOT, || {
        assert_eq!(ready(capture_chart_pointer(id, 37)), Ok(()));
        ready(release_chart_pointer(id, 37));
        ready(release_chart_pointer(id, 37));
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
            serde_json::json!({"kind":"data","value":{"element_id":id,"pointer_id":37,"capture":true}}),
            serde_json::json!({"kind":"finish"}),
            serde_json::json!({"kind":"data","value":{"element_id":id,"pointer_id":37,"capture":false}}),
            serde_json::json!({"kind":"finish"}),
            serde_json::json!({"kind":"data","value":{"element_id":id,"pointer_id":37,"capture":false}}),
            serde_json::json!({"kind":"finish"}),
        ]
    );
}
#[test]
fn capture_reports_native_errors_while_release_remains_best_effort() {
    let (dom, wire) = environment();
    for message in ["The chart is no longer mounted.", "Invalid pointer id"] {
        wire.borrow_mut().reply = serde_json::json!({"error":message});
        dom.in_scope(ScopeId::ROOT, || {
            assert_eq!(
                ready(capture_chart_pointer("chart", -1)),
                Err(ChartExportError::Renderer(message.into()))
            );
            ready(release_chart_pointer("chart", -1));
        });
    }
    assert_eq!(
        wire.borrow()
            .sent
            .iter()
            .filter(|value| value["kind"] == "finish")
            .count(),
        4
    );
}
#[test]
fn pointer_transport_failures_report_capture_errors_and_never_escape_release() {
    let (dom, wire) = environment();
    for fail_send in [true, false] {
        wire.borrow_mut().fail_send = fail_send;
        wire.borrow_mut().fail_recv = !fail_send;
        dom.in_scope(ScopeId::ROOT, || {
            assert!(matches!(
                ready(capture_chart_pointer("chart", 9)),
                Err(ChartExportError::Renderer(_))
            ));
            ready(release_chart_pointer("chart", 9));
        });
    }
    assert_eq!(wire.borrow().scripts.len(), 4);
    assert_eq!(
        wire.borrow()
            .sent
            .iter()
            .filter(|value| value["kind"] == "finish")
            .count(),
        4,
        "failed operations still close their channel"
    );
}
