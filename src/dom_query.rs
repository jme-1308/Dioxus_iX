//! Acknowledged, single-result DOM evaluations.
//!
//! Desktop alpha.2 can collect a completed evaluation before Rust polls its
//! return value. Receive the result on the live channel, then acknowledge it.
//! Dropping a query also releases that channel, including before its first input.
//! Observer streams and fire-and-forget cleanup use ordinary document::eval.

use dioxus::document::{Eval, EvalError};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::future::{Future, IntoFuture};
use std::pin::Pin;

pub(crate) struct OneShot {
    task: Eval,
    finished: bool,
}

pub(crate) fn eval(script: &str) -> OneShot {
    let script = format!(
        r#"
        const channel = dioxus;
        const cancelled = Symbol('cancelled DOM query');
        let wasCancelled = false;
        const bridge = {{
            recv: async () => {{
                if (wasCancelled) throw cancelled;
                const message = await channel.recv();
                if (message.kind === 'finish') {{ wasCancelled = true; throw cancelled; }}
                return message.value;
            }}
        }};
        try {{
            const value = await (async (dioxus) => {{ {script}
            }})(bridge);
            if (wasCancelled) return;
            channel.send({{value: value === undefined ? null : value, error: null}});
        }} catch (error) {{
            if (wasCancelled || error === cancelled) return;
            channel.send({{value: null, error: String(error)}});
        }}
        await channel.recv();
        "#
    );
    OneShot {
        task: dioxus::document::eval(&script),
        finished: false,
    }
}

impl OneShot {
    pub(crate) fn send(&self, value: impl Serialize) -> Result<(), EvalError> {
        #[derive(Serialize)]
        struct Input<T> {
            kind: &'static str,
            value: T,
        }
        self.task.send(Input {
            kind: "data",
            value,
        })
    }

    fn finish(&mut self) -> Result<(), EvalError> {
        if self.finished {
            return Ok(());
        }
        self.finished = true;
        self.task.send(serde_json::json!({"kind": "finish"}))
    }

    pub(crate) async fn join<T: DeserializeOwned>(mut self) -> Result<T, EvalError> {
        #[derive(Deserialize)]
        struct Reply {
            value: serde_json::Value,
            error: Option<String>,
        }
        let reply: Reply = self.task.recv().await?;
        self.finish()?;
        if let Some(error) = reply.error {
            return Err(EvalError::InvalidJs(error));
        }
        serde_json::from_value(reply.value).map_err(EvalError::Serialization)
    }
}

impl Drop for OneShot {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

impl IntoFuture for OneShot {
    type Output = Result<serde_json::Value, EvalError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.join())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::internal::generational_box::{Owner, UnsyncStorage};
    use dioxus::document::Evaluator;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::task::{Context, Poll, Waker};

    #[derive(Default)]
    struct State {
        sent: Vec<serde_json::Value>,
        replies: VecDeque<serde_json::Value>,
        direct_join_calls: usize,
    }

    struct Backend(Rc<RefCell<State>>);

    impl Evaluator for Backend {
        fn send(&self, value: serde_json::Value) -> Result<(), EvalError> {
            self.0.borrow_mut().sent.push(value);
            Ok(())
        }

        fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            self.0
                .borrow_mut()
                .replies
                .pop_front()
                .map_or(Poll::Pending, |value| Poll::Ready(Ok(value)))
        }

        fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<serde_json::Value, EvalError>> {
            self.0.borrow_mut().direct_join_calls += 1;
            Poll::Ready(Err(EvalError::Finished))
        }
    }

    fn query(
        reply: Option<serde_json::Value>,
    ) -> (Owner<UnsyncStorage>, Rc<RefCell<State>>, OneShot) {
        let owner = Owner::default();
        let state = Rc::new(RefCell::new(State::default()));
        state.borrow_mut().replies.extend(reply);
        let task = Eval::new(owner.insert(Box::new(Backend(state.clone())) as Box<dyn Evaluator>));
        (
            owner,
            state,
            OneShot {
                task,
                finished: false,
            },
        )
    }

    fn poll<T>(future: Pin<&mut impl Future<Output = T>>) -> Poll<T> {
        future.poll(&mut Context::from_waker(Waker::noop()))
    }

    #[test]
    fn receives_buffered_result_without_using_collected_return_value() {
        let (_owner, state, query) = query(Some(serde_json::json!({"value": 42, "error": null})));
        let mut result = std::pin::pin!(query.join::<u32>());
        assert!(matches!(poll(result.as_mut()), Poll::Ready(Ok(42))));
        let state = state.borrow();
        assert_eq!(state.direct_join_calls, 0);
        assert_eq!(state.sent, [serde_json::json!({"kind": "finish"})]);
    }

    #[test]
    fn cancellation_releases_pending_and_never_polled_queries() {
        let (_owner, state, query) = query(None);
        let mut future = Box::pin(query.join::<u32>());
        assert!(poll(future.as_mut()).is_pending());
        drop(future);
        assert_eq!(state.borrow().sent, [serde_json::json!({"kind": "finish"})]);
        let (_owner, state, query) = self::query(None);
        drop(query.join::<u32>());
        assert_eq!(state.borrow().sent, [serde_json::json!({"kind": "finish"})]);
    }

    #[test]
    fn inputs_are_sent_once_and_errors_release_the_channel() {
        let (_owner, state, query) = query(Some(
            serde_json::json!({"value": null, "error": "DOM failure"}),
        ));
        query.send(("focus", "input-1")).unwrap();
        let mut result = std::pin::pin!(query.join::<bool>());
        assert!(
            matches!(poll(result.as_mut()), Poll::Ready(Err(EvalError::InvalidJs(error))) if error == "DOM failure")
        );
        assert_eq!(
            state.borrow().sent,
            [
                serde_json::json!({"kind": "data", "value": ["focus", "input-1"]}),
                serde_json::json!({"kind": "finish"}),
            ]
        );
    }

    #[test]
    fn invalid_result_still_acknowledges_exactly_once() {
        let (_owner, state, query) = query(Some(
            serde_json::json!({"value": "wrong type", "error": null}),
        ));
        let mut result = std::pin::pin!(query.join::<bool>());
        assert!(matches!(
            poll(result.as_mut()),
            Poll::Ready(Err(EvalError::Serialization(_)))
        ));
        assert_eq!(state.borrow().sent.len(), 1);
    }
}
