//! Test-only native presentation receipts and Dioxus visibility facts.
//! These fixtures execute no JavaScript and do not simulate unrelated DOM APIs.
use dioxus::document::{Document, Eval, EvalError, Evaluator, NoOpDocument};
use dioxus::html::{HasVisibleData, VisibleResult, geometry::PixelsRect};
use serde_json::Value;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    task::{Context, Poll},
};

#[derive(Default)]
struct Wire {
    requests: Vec<Value>,
    acknowledgements: usize,
}
#[derive(Clone, Default)]
pub(crate) struct NativeSurfaceFixture(Rc<RefCell<Wire>>);
impl NativeSurfaceFixture {
    pub(crate) fn document(&self) -> Rc<dyn Document> {
        Rc::new(SurfaceDocument {
            owner: Default::default(),
            wire: self.0.clone(),
        })
    }
    pub(crate) fn requests(&self) -> Vec<Value> {
        self.0.borrow().requests.clone()
    }
    pub(crate) fn acknowledgements(&self) -> usize {
        self.0.borrow().acknowledgements
    }
}
struct SurfaceDocument {
    owner: dioxus::signals::Owner,
    wire: Rc<RefCell<Wire>>,
}
impl Document for SurfaceDocument {
    fn eval(&self, script: String) -> Eval {
        // Recognize only the shared presentation protocol. Read-only streams
        // and all other native operations retain NoOpDocument's unsupported
        // behavior; this is not a blanket successful browser implementation.
        if !script.contains("const [id,shown,guard,source,mode]=await dioxus.recv()") {
            return NoOpDocument.eval(script);
        }
        Eval::new(self.owner.insert(Box::new(SurfaceReply {
            wire: self.wire.clone(),
            requested: Cell::new(false),
            finished: Cell::new(false),
        }) as Box<dyn Evaluator>))
    }
}
struct SurfaceReply {
    wire: Rc<RefCell<Wire>>,
    requested: Cell<bool>,
    finished: Cell<bool>,
}
impl Evaluator for SurfaceReply {
    fn send(&self, message: Value) -> Result<(), EvalError> {
        match message["kind"].as_str() {
            Some("data") => {
                assert!(
                    !self.requested.replace(true),
                    "one presentation operation per evaluation"
                );
                let request = message["value"].clone();
                let (id, shown, guard, source, mode): (
                    String,
                    bool,
                    Option<(String, String)>,
                    Option<String>,
                    String,
                ) = serde_json::from_value(request.clone())
                    .expect("typed native presentation request");
                assert!(!id.is_empty());
                assert_eq!(mode, "popover");
                assert!(source.is_none(), "these fixtures use registered Rust focus");
                let (attribute, marker) = guard.expect("current rendered ownership guard");
                match attribute.as_str() {
                    "data-surface-generation" => {
                        marker.parse::<u64>().expect("surface generation");
                    }
                    "data-ix-native-popover" => {
                        let parts: Vec<_> = marker.split(':').collect();
                        assert_eq!(parts.len(), 2);
                        parts[0].parse::<u64>().expect("native popover instance");
                        assert_eq!(parts[1].parse::<bool>().expect("visibility intent"), shown);
                    }
                    "data-ix-tooltip-native" => {
                        let parts: Vec<_> = marker.split(':').collect();
                        assert_eq!(parts.len(), 3);
                        parts[0].parse::<u64>().expect("panel token");
                        assert_eq!(parts[1].parse::<bool>().expect("visibility intent"), shown);
                        parts[2].parse::<u64>().expect("tooltip generation");
                    }
                    _ => panic!("unexpected native presentation guard: {attribute}"),
                }
                self.wire.borrow_mut().requests.push(request);
            }
            Some("finish") => {
                assert!(
                    !self.finished.replace(true),
                    "one channel cleanup acknowledgment"
                );
                self.wire.borrow_mut().acknowledgements += 1;
            }
            _ => panic!("unexpected presentation transport message"),
        }
        Ok(())
    }
    fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<Value, EvalError>> {
        assert!(self.requested.get(), "presentation data precedes its reply");
        Poll::Ready(Ok(serde_json::json!({"value":true,"error":null})))
    }
    fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<Value, EvalError>> {
        panic!("presentation uses the acknowledged receive channel")
    }
}

/// Visibility delivered by the renderer, with a finite usable observer root.
pub(crate) struct Visibility(pub bool);
impl HasVisibleData for Visibility {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn is_intersecting(&self) -> VisibleResult<bool> {
        Ok(self.0)
    }
    fn get_intersection_rect(&self) -> VisibleResult<PixelsRect> {
        use dioxus::html::geometry::euclid::{Point2D, Size2D};
        Ok(PixelsRect::new(
            Point2D::new(40., 60.),
            Size2D::new(if self.0 { 32. } else { 0. }, 32.),
        ))
    }
    fn get_root_bounds(&self) -> VisibleResult<PixelsRect> {
        use dioxus::html::geometry::euclid::{Point2D, Size2D};
        Ok(PixelsRect::new(
            Point2D::new(0., 0.),
            Size2D::new(1280., 800.),
        ))
    }
}
