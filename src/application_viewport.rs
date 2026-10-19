// SPDX-FileCopyrightText: 2023 Siemens AG
// SPDX-License-Identifier: MIT

//! Viewport queries adapted from pinned iX utils/breakpoints.ts and
//! utils/application-layout/service.ts. Rust resolves raw matchMedia facts.
//! Owners are isolated: this does not reproduce the upstream global service or
//! Menu.pinned's global override. Applications pass Some(supported), while
//! descendant Headers pass (None, None) to reuse their nearest owner. SSR starts
//! Large unless forced. Empty supported lists safely fall back to Large.

use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(any(feature = "web", feature = "desktop", test))]
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ApplicationBreakpoint;

const DEFAULT_SUPPORTED: [ApplicationBreakpoint; 3] = [
    ApplicationBreakpoint::Small,
    ApplicationBreakpoint::Medium,
    ApplicationBreakpoint::Large,
];
#[cfg(any(feature = "web", feature = "desktop", test))]
static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "web", feature = "desktop", test),
    derive(serde::Deserialize)
)]
struct MediaFacts {
    sm: bool,
    md: bool,
    lg: bool,
    available: bool,
}
impl MediaFacts {
    fn matches(self, breakpoint: ApplicationBreakpoint) -> bool {
        match breakpoint {
            ApplicationBreakpoint::Small => self.sm,
            ApplicationBreakpoint::Medium => self.md,
            ApplicationBreakpoint::Large => self.lg,
        }
    }
}

#[cfg(any(feature = "web", feature = "desktop", test))]
#[derive(serde::Deserialize)]
struct Observation {
    owner: u64,
    generation: u64,
    sequence: u64,
    #[serde(default)]
    width: f64,
    #[serde(default)]
    height: f64,
    #[serde(flatten)]
    facts: MediaFacts,
}

fn resolve(
    force: Option<ApplicationBreakpoint>,
    supported: Option<&[ApplicationBreakpoint]>,
    facts: Option<MediaFacts>,
) -> ApplicationBreakpoint {
    if let Some(force) = force {
        return force;
    }
    let Some(facts) = facts.filter(|facts| facts.available) else {
        return ApplicationBreakpoint::Large;
    };
    let supported = supported.unwrap_or(&DEFAULT_SUPPORTED);
    if let Some(found) = supported.iter().rev().find(|&&item| facts.matches(item)) {
        return *found;
    }
    if supported.contains(&ApplicationBreakpoint::Large) {
        supported.first().copied()
    } else {
        supported.last().copied()
    }
    .unwrap_or(ApplicationBreakpoint::Large)
}

struct ViewportModel {
    #[cfg(any(feature = "web", feature = "desktop", test))]
    owner: u64,
    generation: u64,
    #[cfg(any(feature = "web", feature = "desktop", test))]
    sequence: u64,
    alive: bool,
    force: Option<ApplicationBreakpoint>,
    supported: Option<Vec<ApplicationBreakpoint>>,
    facts: Option<MediaFacts>,
    viewport_size: Option<(f64, f64)>,
    #[cfg(any(feature = "web", feature = "desktop"))]
    eval: Option<dioxus::document::Eval>,
    #[cfg(any(feature = "web", feature = "desktop"))]
    task: Option<dioxus::core::Task>,
}
impl ViewportModel {
    fn new(
        force: Option<ApplicationBreakpoint>,
        supported: Option<Vec<ApplicationBreakpoint>>,
    ) -> Self {
        Self {
            #[cfg(any(feature = "web", feature = "desktop", test))]
            owner: NEXT_OWNER.fetch_add(1, Ordering::Relaxed),
            generation: 1,
            #[cfg(any(feature = "web", feature = "desktop", test))]
            sequence: 0,
            alive: true,
            force,
            supported,
            facts: None,
            viewport_size: None,
            #[cfg(any(feature = "web", feature = "desktop"))]
            eval: None,
            #[cfg(any(feature = "web", feature = "desktop"))]
            task: None,
        }
    }

    fn breakpoint(&self) -> ApplicationBreakpoint {
        resolve(self.force, self.supported.as_deref(), self.facts)
    }

    // Only plain model data changes during render. Signal notification is queued
    // until after the render, so existing descendant readers are also updated.
    fn sync_inputs(
        &mut self,
        force: Option<ApplicationBreakpoint>,
        supported: Option<Vec<ApplicationBreakpoint>>,
    ) -> bool {
        if !self.alive || (self.force == force && self.supported == supported) {
            return false;
        }
        self.force = force;
        self.supported = supported;
        true
    }

    #[cfg(any(feature = "web", feature = "desktop", test))]
    fn observe(&mut self, observation: Observation) -> bool {
        if !self.alive
            || observation.owner != self.owner
            || observation.generation != self.generation
            || observation.sequence <= self.sequence
        {
            return false;
        }
        self.sequence = observation.sequence;
        let viewport_size = (observation.facts.available
            && observation.width.is_finite()
            && observation.width > 0.0
            && observation.height.is_finite()
            && observation.height > 0.0)
            .then_some((observation.width, observation.height));
        let changed = self.facts != Some(observation.facts) || self.viewport_size != viewport_size;
        self.facts = Some(observation.facts);
        self.viewport_size = viewport_size;
        changed
    }

    fn invalidate(&mut self) {
        self.alive = false;
        self.viewport_size = None;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[cfg(any(feature = "web", feature = "desktop"))]
fn observer_script(owner: u64, generation: u64) -> String {
    const SMALL_QUERY: &str = "(max-width: 48em)";
    const MEDIUM_QUERY: &str = "(min-width: 48.0625em) and (max-width: 80em)";
    const LARGE_QUERY: &str = "(min-width: 80.0625em)";
    format!(
        r#"
const owner={owner},generation={generation};let sequence=0;
if(typeof window==='undefined'||typeof window.matchMedia!=='function'){{
  dioxus.send({{owner,generation,sequence:++sequence,sm:false,md:false,lg:false,available:false}});return;
}}
const sm=window.matchMedia({SMALL_QUERY:?}),md=window.matchMedia({MEDIUM_QUERY:?}),lg=window.matchMedia({LARGE_QUERY:?});
// Pinned MountedData measures elements, but exposes no document viewport size.
// Reuse these existing resize/media subscriptions for the missing raw facts.
const report=()=>dioxus.send({{owner,generation,sequence:++sequence,sm:sm.matches,md:md.matches,lg:lg.matches,available:true,width:document.documentElement.clientWidth,height:document.documentElement.clientHeight}});
const unbind=[sm,md,lg].map(query=>{{
  if(typeof query.addEventListener==='function'){{query.addEventListener('change',report);return()=>query.removeEventListener('change',report);}}
  if(typeof query.addListener==='function'){{query.addListener(report);return()=>query.removeListener(report);}}
  return()=>{{}};
}});
window.addEventListener('resize',report,{{passive:true}});report();
try{{await dioxus.recv();}}finally{{for(const remove of unbind)remove();window.removeEventListener('resize',report);}}
"#
    )
}

#[derive(Clone)]
pub(crate) struct ApplicationViewport {
    model: Rc<RefCell<ViewportModel>>,
    revision: Signal<u64>,
}
impl ApplicationViewport {
    pub(crate) fn is_ready(&self) -> bool {
        let _ = (self.revision)();
        let model = self.model.borrow();
        model.force.is_some() || model.facts.is_some()
    }

    pub(crate) fn breakpoint(&self) -> ApplicationBreakpoint {
        let _ = (self.revision)();
        self.model.borrow().breakpoint()
    }

    /// Raw document viewport dimensions, unavailable before native observation
    /// (including SSR). Element-based layouts should use mounted measurements.
    pub(crate) fn viewport_size(&self) -> Option<(f64, f64)> {
        let _ = (self.revision)();
        self.model.borrow().viewport_size
    }
}

/// Some(supported) establishes an Application owner even below another owner.
/// (None, None) reuses an ancestor; without one it owns a standalone observer.
/// This ownership role stays fixed for the lifetime of this hook invocation.
pub(crate) fn use_application_viewport(
    force: Option<ApplicationBreakpoint>,
    supported: Option<Vec<ApplicationBreakpoint>>,
) -> ApplicationViewport {
    let inherited = use_hook(try_consume_context::<ApplicationViewport>);
    let owns = use_hook(|| inherited.is_none() || force.is_some() || supported.is_some());
    let revision = use_signal(|| 0_u64);
    let model = use_hook(|| {
        if owns {
            Rc::new(RefCell::new(ViewportModel::new(force, supported.clone())))
        } else {
            inherited
                .as_ref()
                .expect("inherited viewport")
                .model
                .clone()
        }
    });
    let revision = if owns {
        revision
    } else {
        inherited.as_ref().expect("inherited viewport").revision
    };
    let viewport = ApplicationViewport {
        model: model.clone(),
        revision,
    };
    use_hook(|| provide_context(viewport.clone()));

    if owns && model.borrow_mut().sync_inputs(force, supported) {
        let current = model.clone();
        dioxus::core::queue_effect(move || {
            let alive = current.borrow().alive;
            if alive {
                let mut revision = revision;
                let next = revision.peek().wrapping_add(1);
                revision.set(next);
            }
        });
    }

    #[cfg(any(feature = "web", feature = "desktop"))]
    use_effect({
        let model = model.clone();
        move || {
            if !owns || model.borrow().eval.is_some() {
                return;
            }
            let (owner, generation) = {
                let state = model.borrow();
                if !state.alive {
                    return;
                }
                (state.owner, state.generation)
            };
            let eval = dioxus::document::eval(&observer_script(owner, generation));
            let mut receiver = eval;
            let observed = model.clone();
            let mut revision = revision;
            let task = spawn(async move {
                while let Ok(observation) = receiver.recv::<Observation>().await {
                    let changed = observed.borrow_mut().observe(observation);
                    if changed {
                        let next = revision.peek().wrapping_add(1);
                        revision.set(next);
                    }
                }
            });
            let mut state = model.borrow_mut();
            state.eval = Some(eval);
            state.task = Some(task);
        }
    });
    use_drop({
        let model = model.clone();
        move || {
            if !owns {
                return;
            }
            #[cfg(any(feature = "web", feature = "desktop"))]
            let (eval, task) = {
                let mut state = model.borrow_mut();
                state.invalidate();
                (state.eval.take(), state.task.take())
            };
            #[cfg(not(any(feature = "web", feature = "desktop")))]
            model.borrow_mut().invalidate();
            #[cfg(any(feature = "web", feature = "desktop"))]
            {
                if let Some(eval) = eval {
                    let _ = eval.send("finish");
                }
                if let Some(task) = task {
                    task.cancel();
                }
            }
        }
    });
    viewport
}

#[cfg(test)]
mod tests {
    use super::*;
    use ApplicationBreakpoint::{Large, Medium, Small};

    fn facts(sm: bool, md: bool, lg: bool) -> MediaFacts {
        MediaFacts {
            sm,
            md,
            lg,
            available: true,
        }
    }

    #[test]
    fn viewport_resolves_force_initial_gap_and_supported_order() {
        assert_eq!(resolve(None, None, None), Large);
        assert_eq!(resolve(Some(Small), None, None), Small);
        assert_eq!(resolve(None, None, Some(facts(true, false, false))), Small);
        assert_eq!(resolve(None, None, Some(facts(false, true, false))), Medium);
        assert_eq!(resolve(None, None, Some(facts(false, false, true))), Large);
        assert_eq!(resolve(None, None, Some(facts(false, false, false))), Small);
        assert_eq!(
            resolve(
                None,
                Some(&[Small, Medium]),
                Some(facts(false, false, false))
            ),
            Medium
        );
        assert_eq!(
            resolve(
                None,
                Some(&[Medium, Large]),
                Some(facts(false, false, false))
            ),
            Medium
        );
        assert_eq!(
            resolve(None, Some(&[Large, Small]), Some(facts(true, false, true))),
            Small
        );
        assert_eq!(
            resolve(None, Some(&[]), Some(facts(true, false, false))),
            Large
        );
        assert_eq!(
            resolve(
                None,
                None,
                Some(MediaFacts {
                    available: false,
                    ..facts(true, false, false)
                })
            ),
            Large
        );
    }

    fn observation(model: &ViewportModel, sequence: u64, facts: MediaFacts) -> Observation {
        Observation {
            owner: model.owner,
            generation: model.generation,
            sequence,
            width: 0.0,
            height: 0.0,
            facts,
        }
    }

    fn sized_observation(
        model: &ViewportModel,
        sequence: u64,
        width: f64,
        height: f64,
    ) -> Observation {
        let mut report = observation(model, sequence, facts(false, true, false));
        report.width = width;
        report.height = height;
        report
    }

    #[test]
    fn viewport_resize_notifies_when_only_dimensions_change() {
        let mut model = ViewportModel::new(None, None);
        assert_eq!(model.viewport_size, None, "SSR has no document dimensions");
        assert!(model.observe(sized_observation(&model, 1, 1000.0, 700.0)));
        assert_eq!(model.breakpoint(), Medium);
        assert!(
            model.observe(sized_observation(&model, 2, 1100.0, 760.0)),
            "geometry changes must notify existing subscribers"
        );
        assert_eq!(model.breakpoint(), Medium);
        assert_eq!(model.viewport_size, Some((1100.0, 760.0)));
        assert!(!model.observe(sized_observation(&model, 3, 1100.0, 760.0)));
        assert_eq!(model.sequence, 3);
    }

    #[test]
    fn viewport_dimensions_reject_invalid_or_unavailable_native_facts() {
        let mut model = ViewportModel::new(Some(Small), None);
        assert!(model.observe(sized_observation(&model, 1, 1000.0, 700.0)));
        for (index, (width, height)) in [
            (0.0, 700.0),
            (-1.0, 700.0),
            (1000.0, 0.0),
            (f64::NAN, 700.0),
            (1000.0, f64::INFINITY),
        ]
        .into_iter()
        .enumerate()
        {
            model.observe(sized_observation(&model, index as u64 + 2, width, height));
            assert_eq!(model.viewport_size, None);
            assert_eq!(
                model.breakpoint(),
                Small,
                "geometry cannot override forced breakpoint"
            );
        }
        let mut unavailable = sized_observation(&model, 7, 1000.0, 700.0);
        unavailable.facts.available = false;
        model.observe(unavailable);
        assert_eq!(model.viewport_size, None);
        let missing: Observation = serde_json::from_value(serde_json::json!({
            "owner":model.owner,"generation":model.generation,"sequence":8,
            "sm":false,"md":true,"lg":false,"available":true
        }))
        .unwrap();
        model.observe(missing);
        assert_eq!(
            model.viewport_size, None,
            "missing dimensions are unavailable"
        );
    }

    #[test]
    fn viewport_dimensions_ignore_stale_owner_generation_and_sequence() {
        let mut model = ViewportModel::new(None, None);
        assert!(model.observe(sized_observation(&model, 2, 1000.0, 700.0)));
        let mut wrong_owner = sized_observation(&model, 3, 1200.0, 900.0);
        wrong_owner.owner += 1;
        assert!(!model.observe(wrong_owner));
        let mut wrong_generation = sized_observation(&model, 3, 1200.0, 900.0);
        wrong_generation.generation += 1;
        assert!(!model.observe(wrong_generation));
        assert!(!model.observe(sized_observation(&model, 1, 1200.0, 900.0)));
        assert_eq!(model.viewport_size, Some((1000.0, 700.0)));
        let late = sized_observation(&model, 3, 1200.0, 900.0);
        model.invalidate();
        assert!(!model.observe(late));
        assert_eq!(
            model.viewport_size, None,
            "owner teardown clears dimensions"
        );
    }

    #[test]
    fn viewport_observer_rejects_stale_reports_and_resolves_latest_owner_inputs() {
        let mut model = ViewportModel::new(Some(Large), None);
        let mut wrong = observation(&model, 1, facts(true, false, false));
        wrong.owner += 1;
        assert!(!model.observe(wrong));
        let mut wrong = observation(&model, 1, facts(true, false, false));
        wrong.generation += 1;
        assert!(!model.observe(wrong));
        assert_eq!(model.sequence, 0);
        assert!(model.observe(observation(&model, 2, facts(true, false, false))));
        assert_eq!(model.breakpoint(), Large);
        assert!(!model.observe(observation(&model, 1, facts(false, true, false))));
        assert!(model.sync_inputs(None, Some(vec![Small, Medium, Large])));
        assert_eq!(model.breakpoint(), Small);
        assert!(model.observe(observation(&model, 3, facts(false, true, false))));
        assert_eq!(model.breakpoint(), Medium);
        assert!(!model.observe(observation(&model, 4, facts(false, true, false))));
        assert_eq!(model.sequence, 4, "identical facts still consume sequence");
        model.invalidate();
        assert!(!model.observe(observation(&model, 5, facts(false, false, true))));
        assert!(!model.sync_inputs(Some(Large), None));
        assert_eq!(model.breakpoint(), Medium);
    }

    #[derive(Clone)]
    struct Capture(Rc<RefCell<Vec<(u64, ApplicationBreakpoint)>>>);
    impl PartialEq for Capture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    #[component]
    fn Consumer(capture: Capture) -> Element {
        let viewport = use_application_viewport(None, None);
        capture
            .0
            .borrow_mut()
            .push((viewport.model.borrow().owner, viewport.breakpoint()));
        rsx! { div {} }
    }
    #[component]
    fn NestedOwner(capture: Capture) -> Element {
        let viewport = use_application_viewport(None, Some(DEFAULT_SUPPORTED.to_vec()));
        capture
            .0
            .borrow_mut()
            .push((viewport.model.borrow().owner, viewport.breakpoint()));
        rsx! { Consumer {capture} }
    }
    #[component]
    fn Owner(capture: Capture) -> Element {
        let viewport = use_application_viewport(Some(Small), Some(DEFAULT_SUPPORTED.to_vec()));
        capture
            .0
            .borrow_mut()
            .push((viewport.model.borrow().owner, viewport.breakpoint()));
        rsx! { Consumer {capture:capture.clone()} NestedOwner {capture} }
    }

    #[test]
    fn viewport_consumers_share_nearest_owner_but_nested_application_is_isolated() {
        let capture = Capture(Rc::new(RefCell::new(vec![])));
        let mut dom = VirtualDom::new_with_props(
            Owner,
            OwnerProps {
                capture: capture.clone(),
            },
        );
        dom.rebuild_in_place();
        let records = capture.0.borrow();
        assert_eq!(records.len(), 4);
        assert_eq!(records[0], records[1]);
        assert_eq!(records[0].1, Small);
        assert_ne!(records[0].0, records[2].0);
        assert_eq!(records[2], records[3]);
        assert_eq!(
            records[2].1, Large,
            "nested auto does not inherit outer force"
        );
    }

    #[derive(Clone)]
    struct UpdatingCapture {
        force: Rc<RefCell<Option<Signal<Option<ApplicationBreakpoint>>>>>,
        values: Rc<RefCell<Vec<ApplicationBreakpoint>>>,
        owner: Rc<RefCell<Option<Rc<RefCell<ViewportModel>>>>>,
    }
    impl PartialEq for UpdatingCapture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.force, &other.force)
        }
    }
    #[component]
    fn UpdatingOwner(capture: UpdatingCapture) -> Element {
        let force = use_signal(|| Some(Small));
        *capture.force.borrow_mut() = Some(force);
        let viewport = use_application_viewport(force(), Some(DEFAULT_SUPPORTED.to_vec()));
        *capture.owner.borrow_mut() = Some(viewport.model.clone());
        let current = use_memo(move || viewport.breakpoint());
        capture.values.borrow_mut().push(current());
        rsx! { div {} }
    }

    #[test]
    fn viewport_input_changes_refresh_existing_memo_after_render_and_owner_drop_invalidates() {
        let capture = UpdatingCapture {
            force: Rc::new(RefCell::new(None)),
            values: Rc::new(RefCell::new(vec![])),
            owner: Rc::new(RefCell::new(None)),
        };
        let mut dom = VirtualDom::new_with_props(
            UpdatingOwner,
            UpdatingOwnerProps {
                capture: capture.clone(),
            },
        );
        dom.rebuild_in_place();
        let owner = capture.owner.borrow().as_ref().unwrap().clone();
        let mut force = capture.force.borrow().unwrap();
        assert_eq!(capture.values.borrow().last(), Some(&Small));
        for (requested, expected) in [(Some(Medium), Medium), (None, Large), (Some(Small), Small)] {
            dom.in_scope(ScopeId::ROOT, || force.set(requested));
            for _ in 0..4 {
                let _ = dom.render_immediate_to_vec();
            }
            assert_eq!(capture.values.borrow().last(), Some(&expected));
            assert!(Rc::ptr_eq(capture.owner.borrow().as_ref().unwrap(), &owner));
        }
        drop(dom);
        let mut owner = owner.borrow_mut();
        assert!(!owner.alive);
        let late = observation(&owner, 1, facts(false, true, false));
        assert!(!owner.observe(late));
    }
}
