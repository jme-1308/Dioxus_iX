//! Mounted return-focus lifetime tests. Native top-layer activation is covered
//! by the preview; these tests deliver the replacement mount explicitly.
use super::*;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::future::Future;

#[derive(Clone, Default)]
struct Fixture(Rc<State>);
#[derive(Default)]
struct State {
    modal: Cell<Option<ModalHandle>>,
    first: Cell<Option<Signal<bool>>>,
    second: Cell<Option<Signal<bool>>>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
fn fixture_root(fixture: Fixture) -> Element {
    let modal = use_modal();
    let first = use_signal(|| true);
    let second = use_signal(|| true);
    fixture.0.modal.set(Some(modal));
    fixture.0.first.set(Some(first));
    fixture.0.second.set(Some(second));
    rsx! {
        if first() { ModalTrigger { modal, "First opener" } }
        if second() { ModalTrigger { modal, "Second opener" } }
    }
}

#[derive(Clone)]
struct FocusMount {
    name: String,
    calls: Rc<RefCell<Vec<String>>>,
}
impl dioxus::html::RenderedElementBacking for FocusMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let mounted = self.clone();
        Box::pin(async move {
            mounted.calls.borrow_mut().push(mounted.name);
            Ok(())
        })
    }
}
struct TestMouse;
impl dioxus::html::InteractionLocation for TestMouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for TestMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for TestMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for TestMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for TestMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! modal_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for Converter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                let _=event; modal_converter!(@data $group,event)
            })*
        }
    };
    (@data Mounted,$event:ident) => {MountedData::new($event.downcast::<FocusMount>().unwrap().clone())};
    (@data Mouse,$event:ident) => {dioxus::html::MouseData::new(TestMouse)};
    (@data $other:ident,$event:ident) => {panic!("Unexpected modal focus test event")};
}
dioxus::html::with_html_event_groups!(modal_converter);

struct Harness {
    dom: VirtualDom,
    fixture: Fixture,
    ids: BTreeMap<String, dioxus::core::ElementId>,
}
impl Harness {
    fn new() -> Self {
        let fixture = Fixture::default();
        let mut dom = VirtualDom::new_with_props(fixture_root, fixture.clone());
        let edits = dom.rebuild_to_vec();
        let mut harness = Self {
            dom,
            fixture,
            ids: BTreeMap::new(),
        };
        harness.collect(edits);
        harness
    }
    fn collect(&mut self, edits: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in edits.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(id),
                    ..
                } if name == "id" => {
                    self.ids.insert(id, current);
                }
                _ => {}
            }
        }
    }
    fn settle(&mut self) {
        for _ in 0..4 {
            let edits = self.dom.render_immediate_to_vec();
            self.collect(edits);
        }
    }
    fn modal(&self) -> ModalHandle {
        self.fixture.0.modal.get().unwrap()
    }
    fn runtime(&self) -> crate::overlay_runtime::RuntimeHandle {
        self.modal().focus_runtime.peek().clone()
    }
    fn register(
        &self,
        name: &str,
        calls: &Rc<RefCell<Vec<String>>>,
    ) -> crate::overlay_runtime::MountedFocusTarget {
        let runtime = self.runtime();
        self.dom.in_scope(ScopeId::ROOT, || {
            runtime.register_item_mount(
                name,
                name,
                Rc::new(MountedData::new(FocusMount {
                    name: name.into(),
                    calls: calls.clone(),
                })),
            );
            runtime.capture_registered_focus(name).unwrap()
        })
    }
    fn remember_and_close(&self, target: crate::overlay_runtime::MountedFocusTarget) -> u64 {
        let modal = self.modal();
        self.dom.in_scope(ScopeId::ROOT, || {
            modal.surface.remember_invoker(Some(target));
            modal.opened();
            modal.close("confirmed");
        });
        assert_eq!(
            modal.result(),
            Some(ModalResult::Closed("confirmed".into()))
        );
        *modal.surface.generation.peek()
    }
    fn mounted(&self, generation: u64) {
        self.dom.in_scope(ScopeId::ROOT, || {
            self.modal()
                .surface
                .mounted(generation, Rc::new(MountedData::new(())))
        });
    }
    fn triggers(&self) -> Vec<String> {
        dioxus_ssr::render(&self.dom)
            .split("<button")
            .skip(1)
            .filter_map(|part| {
                let tag = part.split('>').next()?;
                tag.contains("commandfor=").then(|| {
                    tag.split("id=\"")
                        .nth(1)
                        .unwrap()
                        .split('"')
                        .next()
                        .unwrap()
                        .to_string()
                })
            })
            .collect()
    }
    fn dispatch<T: std::any::Any>(&self, name: &str, id: &str, payload: T) {
        let data: Rc<dyn std::any::Any> =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
        self.dom
            .runtime()
            .handle_event(name, Event::new(data, true), self.ids[id]);
    }
}

#[test]
fn modal_return_waits_for_matching_replacement_mount_and_restores_once() {
    let mut harness = Harness::new();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let target = harness.register("registered-opener", &calls);
    let generation = harness.remember_and_close(target);
    harness.settle();
    assert!(
        calls.borrow().is_empty(),
        "closing alone cannot race native dialog teardown"
    );
    harness.mounted(generation - 1);
    harness.settle();
    assert!(
        calls.borrow().is_empty(),
        "the prior dialog mount cannot restore focus"
    );
    harness.mounted(generation);
    harness.settle();
    assert_eq!(calls.borrow().as_slice(), ["registered-opener"]);
    assert!(harness.modal().surface.restore_after_mount.peek().is_none());
    harness.mounted(generation);
    harness.settle();
    assert_eq!(calls.borrow().len(), 1);
}

#[test]
fn modal_return_is_cancelled_by_reopen_or_new_surface_generation() {
    for reopen in [true, false] {
        let mut harness = Harness::new();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let target = harness.register("registered-opener", &calls);
        let generation = harness.remember_and_close(target);
        harness.mounted(generation);
        let modal = harness.modal();
        harness.dom.in_scope(ScopeId::ROOT, || {
            if reopen {
                modal.opened();
            } else {
                modal.surface.replace(false);
            }
        });
        harness.settle();
        assert!(
            calls.borrow().is_empty(),
            "obsolete close cannot steal focus after reopen/replacement"
        );
    }
}

#[test]
fn modal_return_rejects_replaced_unmounted_and_dead_renderer_destinations() {
    for replace in [true, false] {
        let mut harness = Harness::new();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let target = harness.register("registered-opener", &calls);
        let generation = harness.remember_and_close(target.clone());
        if replace {
            harness.register("registered-opener", &calls);
        } else {
            harness.runtime().unregister_item_cue("registered-opener");
        }
        assert!(target.live_node().is_none());
        harness.mounted(generation);
        harness.settle();
        assert!(
            calls.borrow().is_empty(),
            "same public ID never adopts another mounted object"
        );
    }
    let old = Harness::new();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let target = old.register("registered-opener", &calls);
    let retained_registry = old.runtime();
    drop(old);
    assert!(
        target.live_node().is_none(),
        "retaining a registry cannot retain its renderer"
    );
    let mut current = Harness::new();
    let generation = current.remember_and_close(target);
    current.mounted(generation);
    current.settle();
    assert!(calls.borrow().is_empty());
    drop(retained_registry);
}

#[test]
fn multiple_modal_triggers_capture_the_clicked_registered_opener_and_release_removed_controls() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let mut harness = Harness::new();
    let triggers = harness.triggers();
    assert_eq!(triggers.len(), 2);
    assert_ne!(
        triggers[0], triggers[1],
        "typed triggers have distinct Rust-owned IDs"
    );
    let calls = Rc::new(RefCell::new(Vec::new()));
    for id in &triggers {
        harness.dispatch(
            "mounted",
            id,
            FocusMount {
                name: id.clone(),
                calls: calls.clone(),
            },
        );
    }
    for (index, id) in triggers.iter().enumerate() {
        harness.dispatch("click", id, ());
        let modal = harness.modal();
        harness.dom.in_scope(ScopeId::ROOT, || modal.opened());
        if index == 1 {
            harness.dom.in_scope(ScopeId::ROOT, || {
                harness.fixture.0.first.get().unwrap().set(false)
            });
            harness.settle();
        }
        harness
            .dom
            .in_scope(ScopeId::ROOT, || modal.close("confirmed"));
        let generation = *modal.surface.generation.peek();
        harness.settle();
        assert_eq!(
            calls.borrow().len(),
            index,
            "no return before replacement mount"
        );
        harness.mounted(generation);
        harness.settle();
        assert_eq!(calls.borrow().last(), Some(id));
    }
    harness.dispatch("click", &triggers[1], ());
    let modal = harness.modal();
    harness.dom.in_scope(ScopeId::ROOT, || {
        modal.opened();
        harness.fixture.0.second.get().unwrap().set(false);
    });
    harness.settle();
    harness
        .dom
        .in_scope(ScopeId::ROOT, || modal.close("confirmed"));
    harness.mounted(*modal.surface.generation.peek());
    harness.settle();
    assert_eq!(
        calls.borrow().len(),
        2,
        "unmounted clicked opener cannot receive a return"
    );
}
