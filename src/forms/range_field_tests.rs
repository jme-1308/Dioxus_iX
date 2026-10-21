use super::*;
use dioxus::core::{ElementId, Mutation};
use std::cell::{Cell, RefCell};

#[derive(Clone)]
struct RootMount {
    id: ElementId,
    backing: Rc<Cell<usize>>,
}
impl dioxus::html::RenderedElementBacking for RootMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct RangeForm(String);
impl dioxus::html::HasFileData for RangeForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for RangeForm {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*
        }
    };
    (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<RootMount>().unwrap().clone())};
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(RangeForm($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("unexpected range-field test event")};
}
dioxus::html::with_html_event_groups!(converter);
fn listeners(edits: &dioxus::core::Mutations, name: &str) -> Vec<ElementId> {
    let mut current = None;
    let mut ids = Vec::new();
    for edit in &edits.edits {
        match edit {
            Mutation::SetId { id } | Mutation::PushId { id } => current = Some(*id),
            Mutation::NewEventListener { name: actual } if actual == name => {
                ids.push(current.expect("native listener has an explicit renderer target"));
            }
            Mutation::SetAttribute { .. }
            | Mutation::SetText { .. }
            | Mutation::NewEventListener { .. }
            | Mutation::RemoveEventListener { .. } => {}
            _ => current = None,
        }
    }
    ids
}
fn event(dom: &VirtualDom, name: &str, id: ElementId, payload: impl std::any::Any) {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event(name, Event::new(data, name != "mounted"), id);
}
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    tag.split_once(&format!(" {name}=\""))?
        .1
        .split_once('"')
        .map(|pair| pair.0)
}
fn root(html: &str) -> &str {
    html.split('<')
        .find(|tag| {
            tag.starts_with("div ")
                && attribute(tag, "class").is_some_and(|classes| {
                    classes
                        .split_whitespace()
                        .any(|class| class == "ix-range-field")
                })
        })
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
#[derive(Default)]
struct Capture {
    revision: Cell<usize>,
    visible: Cell<bool>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
    deliveries: RefCell<Vec<(usize, Rc<MountedData>)>>,
}
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
fn default_and_named(data: Rc<Capture>) -> Element {
    rsx! {
        RangeField { input {"aria-label":"Start"} input {"aria-label":"End"} }
        RangeField {id:"named-range",aria_label:"Inspection period",onmounted:move|event:MountedEvent|data.deliveries.borrow_mut().push((0,event.data())),
            input {"aria-label":"Start inspection"} input {"aria-label":"End inspection"}
        }
    }
}
#[test]
fn range_field_default_and_named_groups_forward_the_actual_native_root() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(default_and_named, data.clone());
    let edits = dom.rebuild_to_vec();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("role=\"group\"").count(), 2);
    assert_eq!(attribute(root(&html), "class"), Some("ix-range-field"));
    assert!(attribute(root(&html), "id").is_none());
    assert!(attribute(root(&html), "aria-label").is_none());
    assert!(!html.contains("ix-range-no-arrow"));
    assert!(
        html.contains("id=\"named-range\"") && html.contains("aria-label=\"Inspection period\"")
    );
    assert_eq!(html.matches("<input ").count(), 4);
    let roots = listeners(&edits, "mounted");
    assert_eq!(
        roots.len(),
        2,
        "one native mount per existing group div, without wrappers"
    );
    let backing = Rc::new(Cell::new(0));
    for &id in &roots {
        event(
            &dom,
            "mounted",
            id,
            RootMount {
                id,
                backing: backing.clone(),
            },
        );
    }
    let delivered = data.deliveries.borrow();
    assert_eq!(delivered.len(), 1);
    let native = delivered[0].1.downcast::<RootMount>().unwrap();
    assert_eq!(native.id, roots[1]);
    assert!(Rc::ptr_eq(&native.backing, &backing));
}
#[component]
fn Draft(data: Rc<Capture>) -> Element {
    use_hook({
        let data = data.clone();
        move || data.mounts.set(data.mounts.get() + 1)
    });
    use_drop(move || data.drops.set(data.drops.get() + 1));
    let mut draft = use_signal(|| String::from("Initial range note"));
    rsx! {input {id:"range-private-draft",value:draft(),oninput:move|event|draft.set(event.value())}}
}
fn changing(data: Rc<Capture>) -> Element {
    let revision = data.revision.get();
    let callback = data.clone();
    rsx! {if data.visible.get() {
        RangeField {id:format!("inspection-range-{revision}"),aria_label:format!("Inspection interval {revision}"),
            class:format!("consumer-range-{revision}"),hide_arrow:revision!=0,
            onmounted:move|event:MountedEvent|callback.deliveries.borrow_mut().push((revision,event.data())),
            Draft {data:data.clone()} input {"aria-label":"Range end"}
        }
    }}
}
#[test]
fn range_field_live_metadata_and_arrow_changes_keep_root_child_draft_and_current_callback() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    data.visible.set(true);
    let mut dom = VirtualDom::new_with_props(changing, data.clone());
    let initial = dom.rebuild_to_vec();
    let mounts = listeners(&initial, "mounted");
    assert_eq!(mounts.len(), 1);
    let id = mounts[0];
    let backing = Rc::new(Cell::new(0));
    event(
        &dom,
        "mounted",
        id,
        RootMount {
            id,
            backing: backing.clone(),
        },
    );
    event(
        &dom,
        "input",
        listeners(&initial, "input")[0],
        String::from("Retained private range note"),
    );
    dom.render_immediate_to_vec();
    data.revision.set(1);
    dom.mark_dirty(ScopeId::APP);
    let updated = dom.render_immediate_to_vec();
    assert!(
        listeners(&updated, "mounted").is_empty(),
        "normal prop updates preserve the actual group root"
    );
    let html = dioxus_ssr::render(&dom);
    let group = root(&html);
    assert_eq!(attribute(group, "role"), Some("group"));
    assert_eq!(attribute(group, "id"), Some("inspection-range-1"));
    assert_eq!(
        attribute(group, "aria-label"),
        Some("Inspection interval 1")
    );
    assert_eq!(
        attribute(group, "class"),
        Some("ix-range-field ix-range-no-arrow consumer-range-1")
    );
    assert!(html.contains("value=\"Retained private range note\""));
    assert_eq!(data.mounts.get(), 1);
    assert_eq!(data.drops.get(), 0);
    assert_eq!(
        data.deliveries.borrow().len(),
        1,
        "prop updates never replay a mount callback"
    );
    event(
        &dom,
        "mounted",
        id,
        RootMount {
            id,
            backing: backing.clone(),
        },
    );
    let deliveries = data.deliveries.borrow();
    assert_eq!(deliveries.len(), 2);
    assert_eq!(deliveries[0].0, 0);
    assert_eq!(deliveries[1].0, 1);
    for (_, node) in deliveries.iter() {
        let native = node.downcast::<RootMount>().unwrap();
        assert_eq!(native.id, id);
        assert!(Rc::ptr_eq(&native.backing, &backing));
    }
    drop(deliveries);
    data.visible.set(false);
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert_eq!(data.drops.get(), 1);
    assert!(!dioxus_ssr::render(&dom).contains("ix-range-field"));
}
