use super::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

// The backing remembers the renderer element that delivered the mount. This
// distinguishes the public interactive target from wrappers and form mirrors.
struct ChoiceMounted(dioxus::core::ElementId);
impl dioxus::html::RenderedElementBacking for ChoiceMounted {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct ChoiceMountConverter;
macro_rules! choice_mount_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for ChoiceMountConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                let _=event;
                choice_mount_converter!(@data $group,event)
            })*
        }
    };
    (@data Mounted,$event:ident) => {
        MountedData::new(ChoiceMounted(*$event.downcast::<dioxus::core::ElementId>().unwrap()))
    };
    (@data $other:ident,$event:ident) => {panic!("unexpected choice-mount test event")};
}
dioxus::html::with_html_event_groups!(choice_mount_converter);

const CHOICE_IDS: [&str; 4] = ["mount-check", "mount-radio", "mount-toggle", "mount-range"];
#[derive(Default)]
struct ChoiceMountData {
    runtime: RefCell<Option<crate::overlay_runtime::RuntimeHandle>>,
    nodes: RefCell<BTreeMap<String, Rc<MountedData>>>,
    deliveries: RefCell<Vec<(String, usize)>>,
    epoch: Cell<usize>,
    callback_revision: Cell<usize>,
    visible: Cell<bool>,
}
impl PartialEq for ChoiceMountData {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
fn choice_mounted(data: &ChoiceMountData, key: String, revision: usize, event: MountedEvent) {
    let node = event.data();
    if CHOICE_IDS.contains(&key.as_str()) {
        let registered = data
            .runtime
            .borrow()
            .as_ref()
            .unwrap()
            .capture_registered_focus(&key)
            .expect("surface registration precedes the public callback")
            .live_node()
            .unwrap();
        assert!(Rc::ptr_eq(&node, &registered));
    }
    data.nodes.borrow_mut().insert(key.clone(), node);
    data.deliveries.borrow_mut().push((key, revision));
}
#[component]
fn MountedChoices(data: Rc<ChoiceMountData>, revision: usize) -> Element {
    let checkbox = data.clone();
    let radio = data.clone();
    let toggle = data.clone();
    let slider = data.clone();
    let group = data.clone();
    rsx! {
        Checkbox { id:CHOICE_IDS[0],label:"Preparation",name:"prepared",form:"work-order",required:true,checked:true,
            onmounted:move|event|choice_mounted(&checkbox,CHOICE_IDS[0].into(),revision,event) }
        Radio { id:CHOICE_IDS[1],label:"Morning",name:"standalone-shift",form:"work-order",value:"morning",required:true,checked:true,
            onmounted:move|event|choice_mounted(&radio,CHOICE_IDS[1].into(),revision,event) }
        Toggle { id:CHOICE_IDS[2],aria_label:"Urgent",name:"urgent",form:"work-order",required:true,checked:true,
            onmounted:move|event|choice_mounted(&toggle,CHOICE_IDS[2].into(),revision,event) }
        Slider { id:CHOICE_IDS[3],label:"Priority",aria_label:"Work priority",name:"priority",form:"work-order",value:3.,min:1.,max:5.,step:0.5,show_value:true,
            onmounted:move|event|choice_mounted(&slider,CHOICE_IDS[3].into(),revision,event) }
        RadioGroup { label:"Shift",name:"shift",form:"work-order",value:"day",required:true,
            options:vec![SelectOption::new("day","Day"),SelectOption::new("night","Night").disabled()],
            onoptionmounted:move|(value,event)|choice_mounted(&group,format!("group:{value}"),revision,event) }
    }
}
fn choice_mount_fixture(data: Rc<ChoiceMountData>) -> Element {
    *data.runtime.borrow_mut() = Some(crate::overlay_runtime::use_overlay_runtime());
    rsx! { crate::Pane { heading:"Mounted choices",expanded:true,
        if data.visible.get() { MountedChoices { key:"{data.epoch.get()}",data:data.clone(),revision:data.callback_revision.get() } }
    } }
}
struct ChoiceMountHarness {
    dom: VirtualDom,
    data: Rc<ChoiceMountData>,
    ids: BTreeMap<String, dioxus::core::ElementId>,
    values: BTreeMap<String, dioxus::core::ElementId>,
}
impl ChoiceMountHarness {
    fn new() -> Self {
        let data = Rc::new(ChoiceMountData::default());
        data.visible.set(true);
        let mut dom = VirtualDom::new_with_props(choice_mount_fixture, data.clone());
        let changes = dom.rebuild_to_vec();
        let mut result = Self {
            dom,
            data,
            ids: BTreeMap::new(),
            values: BTreeMap::new(),
        };
        result.mount(changes);
        result
    }
    fn mount(&mut self, mut changes: dioxus::core::Mutations) {
        for _ in 0..8 {
            let mut current = dioxus::core::ElementId::ROOT;
            for edit in &changes.edits {
                match edit {
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = *id,
                    dioxus::core::Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } if name == "id" => {
                        self.ids.insert(value.clone(), current);
                    }
                    dioxus::core::Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } if name == "value" => {
                        self.values.insert(value.clone(), current);
                    }
                    dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                        let data: Rc<dyn std::any::Any> =
                            Rc::new(dioxus::html::PlatformEventData::new(Box::new(current)));
                        self.dom.runtime().handle_event(
                            "mounted",
                            Event::new(data, false),
                            current,
                        );
                    }
                    _ => {}
                }
            }
            changes = self.dom.render_immediate_to_vec();
        }
    }
    fn refresh(&mut self) {
        self.dom.mark_dirty(ScopeId::APP);
        let changes = self.dom.render_immediate_to_vec();
        self.mount(changes);
    }
}
fn tag_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(quoted) = value.strip_prefix('"') {
        quoted.split_once('"').map(|pair| pair.0)
    } else {
        // Pinned dioxus-ssr writes dynamic numeric attributes without quotes.
        value
            .split(|character: char| character.is_whitespace() || character == '>')
            .next()
    }
}

fn tag_with_id<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|tag| tag_attribute(tag, "id") == Some(id))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}

#[test]
fn choice_mount_callbacks_target_registered_interactive_controls_and_group_values() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ChoiceMountConverter));
    let h = ChoiceMountHarness::new();
    assert_eq!(
        h.data.deliveries.borrow().len(),
        6,
        "wrappers and native mirrors never deliver public mounts"
    );
    let html = dioxus_ssr::render(&h.dom);
    for (index, id) in CHOICE_IDS.iter().enumerate() {
        let nodes = h.data.nodes.borrow();
        let node = nodes[*id].downcast::<ChoiceMounted>().unwrap();
        assert_eq!(node.0, h.ids[*id]);
        let tag = tag_with_id(&html, id);
        assert!(tag.starts_with(if index == 0 || index == 2 {
            "button "
        } else {
            "input "
        }));
        assert_eq!(
            tag_attribute(tag, "form"),
            if index == 0 || index == 2 {
                None
            } else {
                Some("work-order")
            }
        );
        if index == 0 || index == 2 {
            assert_eq!(
                tag_attribute(tag, "name"),
                None,
                "successful form values belong to the hidden mirror"
            );
        }
    }
    for value in ["day", "night"] {
        assert_eq!(
            h.data.nodes.borrow()[&format!("group:{value}")]
                .downcast::<ChoiceMounted>()
                .unwrap()
                .0,
            h.values[value]
        );
        let tag = html
            .split('<')
            .find(|tag| tag.starts_with("input ") && tag_attribute(tag, "value") == Some(value))
            .unwrap();
        assert_eq!(tag_attribute(tag, "type"), Some("radio"));
        assert_eq!(tag_attribute(tag, "name"), Some("shift"));
        assert_eq!(tag_attribute(tag, "form"), Some("work-order"));
        assert!(tag.split('>').next().unwrap().contains(" required"));
        if value == "day" {
            assert!(tag.split('>').next().unwrap().contains(" checked"));
        } else {
            assert!(tag.split('>').next().unwrap().contains(" disabled"));
        }
    }
    let range = tag_with_id(&html, CHOICE_IDS[3]);
    for (name, value) in [
        ("type", "range"),
        ("name", "priority"),
        ("aria-label", "Work priority"),
        ("min", "1"),
        ("max", "5"),
        ("step", "0.5"),
        ("value", "3"),
    ] {
        assert_eq!(tag_attribute(range, name), Some(value));
    }
    assert!(
        html.contains("for=\"mount-range\""),
        "visible label and output retain the explicit range ID"
    );
    assert_eq!(
        html.split('<')
            .filter(
                |tag| tag.starts_with("input ") && tag_attribute(tag, "name") == Some("priority")
            )
            .count(),
        1,
        "range is its own successful native control"
    );
    for name in ["prepared", "urgent"] {
        let mirror = html
            .split('<')
            .find(|tag| tag.starts_with("input ") && tag_attribute(tag, "name") == Some(name))
            .unwrap();
        assert_eq!(tag_attribute(mirror, "aria-hidden"), Some("true"));
        assert_eq!(tag_attribute(mirror, "form"), Some("work-order"));
    }
}

#[test]
fn choice_mount_callbacks_renew_on_keyed_replacement_and_release_surface_registration() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ChoiceMountConverter));
    let mut h = ChoiceMountHarness::new();
    let original = h.data.nodes.borrow().clone();
    h.data.callback_revision.set(1);
    h.refresh();
    assert_eq!(
        h.data.deliveries.borrow().len(),
        6,
        "callback changes do not remount native controls"
    );
    h.data.epoch.set(1);
    h.refresh();
    assert_eq!(h.data.deliveries.borrow().len(), 12);
    for (key, node) in original {
        assert!(
            !Rc::ptr_eq(&node, &h.data.nodes.borrow()[&key]),
            "keyed replacement supplies fresh native handles"
        );
    }
    assert!(
        h.data.deliveries.borrow()[6..]
            .iter()
            .all(|(_, revision)| *revision == 1),
        "replacement calls the current consumer callbacks"
    );
    h.data.visible.set(false);
    h.refresh();
    for id in CHOICE_IDS {
        assert!(
            h.data
                .runtime
                .borrow()
                .as_ref()
                .unwrap()
                .capture_registered_focus(id)
                .is_none()
        );
    }
    assert_eq!(
        h.data.deliveries.borrow().len(),
        12,
        "unmount does not replay callbacks"
    );
}

fn generated_slider_fixture(label: String) -> Element {
    rsx! { Slider { label } }
}
fn render_generated_slider(label: &str) -> String {
    let mut dom = VirtualDom::new_with_props(generated_slider_fixture, label.to_owned());
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn slider_generated_id_retains_visible_label_and_default_accessible_name() {
    let labelled = render_generated_slider("Load");
    let range = labelled
        .split('<')
        .find(|tag| tag.starts_with("input ") && tag_attribute(tag, "type") == Some("range"))
        .unwrap();
    let id = tag_attribute(range, "id").unwrap();
    assert!(id.starts_with("ix-field-"));
    assert!(labelled.contains(&format!("for=\"{id}\"")));
    assert_eq!(tag_attribute(range, "aria-label"), Some("Load"));
    let unlabelled = render_generated_slider("");
    let range = unlabelled
        .split('<')
        .find(|tag| tag.starts_with("input ") && tag_attribute(tag, "type") == Some("range"))
        .unwrap();
    assert_eq!(tag_attribute(range, "aria-label"), Some("Slider"));
}
