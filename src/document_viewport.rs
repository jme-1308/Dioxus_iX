// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Shared native viewport facts, one stream per Document in a Dioxus root. Pinned alpha.2
//! MountedData measures elements but exposes neither the viewport/visual viewport
//! box, CSS media matches or document/window scroll, resize or Escape subscriptions. This adapter
//! supplies only those missing facts; Rust owns subscribers and their lifetime.

use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

/// Raw CSS em query matches; width in pixels never substitutes for these facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct DocumentMediaFacts {
    pub(crate) sm: bool,
    pub(crate) md: bool,
    pub(crate) lg: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
pub(crate) struct DocumentViewportFacts {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) visual_left: f64,
    pub(crate) visual_top: f64,
    pub(crate) visual_width: f64,
    pub(crate) visual_height: f64,
    #[serde(default)]
    pub(crate) media: Option<DocumentMediaFacts>,
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
        let media = self.facts().and_then(|facts| facts.media);
        self.test_facts(DocumentViewportFacts {
            width,
            height,
            visual_left: 0.0,
            visual_top: 0.0,
            visual_width: width,
            visual_height: height,
            media,
        });
    }
    #[cfg(test)]
    pub(crate) fn test_facts(&self, facts: DocumentViewportFacts) {
        self.test_event(NativeEvent::Layout { facts });
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
// Pinned Dioxus has no MediaQueryList API. Preserve the upstream CSS em
// semantics once per shared stream instead of deriving matches from px width.
const queries=typeof window.matchMedia==='function'?{{
  sm:window.matchMedia('(max-width: 48em)'),
  md:window.matchMedia('(min-width: 48.0625em) and (max-width: 80em)'),
  lg:window.matchMedia('(min-width: 80.0625em)')
}}:null;
const layout=()=>{{const v=window.visualViewport;dioxus.send({{epoch,sequence:++sequence,kind:'layout',
  width:document.documentElement.clientWidth,height:document.documentElement.clientHeight,
  visual_left:v?.offsetLeft??0,visual_top:v?.offsetTop??0,
  visual_width:v?.width??document.documentElement.clientWidth,visual_height:v?.height??document.documentElement.clientHeight,
  media:queries?{{sm:queries.sm.matches,md:queries.md.matches,lg:queries.lg.matches}}:null}});}};
for(const query of Object.values(queries??{{}}))query.addEventListener('change',layout,{{signal}});
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
mod tests;
