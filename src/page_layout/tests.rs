use super::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
#[derive(Clone)]
struct Config {
    mode: u8,
    size: TileSize,
    variant: ContentHeaderVariant,
    ellipsis: bool,
    show_back: bool,
    title: String,
    subtitle: Option<String>,
    back_label: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            mode: 0,
            size: TileSize::Medium,
            variant: ContentHeaderVariant::Primary,
            ellipsis: false,
            show_back: true,
            title: "Motor 001".into(),
            subtitle: Some("Plant A".into()),
            back_label: "Return to assets".into(),
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    tile_actions: Cell<usize>,
    sub_actions: Cell<usize>,
    footer_actions: Cell<usize>,
    back: Cell<usize>,
    page_actions: Cell<usize>,
    mounts: RefCell<Vec<String>>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn LocalField(id: String, data: Rc<Fixture>) -> Element {
    let mut note = use_signal(|| String::from("Initial note"));
    let mounted_id = id.clone();
    rsx! {input{id,value:note(),oninput:move|event|note.set(event.value()),onmounted:move|_|data.mounts.borrow_mut().push(mounted_id.clone())}}
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let action = data.clone();
    let sub = data.clone();
    let footer = data.clone();
    let back = data.clone();
    let page = data.clone();
    let header = if config.mode == 1 {
        None
    } else {
        Some(String::from("Fallback heading"))
    };
    let subheader = if config.mode == 1 {
        None
    } else {
        Some(String::from("Fallback subheading"))
    };
    let header_content = match config.mode {
        0 => Some(rsx! {span{"Composed condition"}}),
        3 => Some(VNode::empty()),
        _ => None,
    };
    let subheader_content = match config.mode {
        0 => Some(
            rsx! {span{"Local sensor"}button{id:String::from("tile-units"),r#type:"button",onclick:move|_|sub.sub_actions.set(sub.sub_actions.get()+1),"Units"}},
        ),
        3 => Some(VNode::empty()),
        _ => None,
    };
    let header_actions=(config.mode!=2).then(||rsx!{button{id:String::from("tile-inspect"),r#type:"button",onclick:move|_|action.tile_actions.set(action.tile_actions.get()+1),"Inspect"}});
    let footer_content=(config.mode!=2).then(||rsx!{button{id:String::from("tile-review"),r#type:"button",onclick:move|_|footer.footer_actions.set(footer.footer_actions.get()+1),"Review"}});
    rsx! {
        crate::layout::Tile{id:"condition",class:"custom-tile",aria_label:"Condition tile",header,subheader,header_content,subheader_content,header_actions,footer:footer_content,size:config.size,
            LocalField{id:"tile-note",data:data.clone()}
        }
        ContentHeader{id:"asset-header",class:"custom-page-header",title:config.title,subtitle:config.subtitle,show_back:config.show_back,aria_label_back_button:config.back_label,variant:config.variant,ellipsis:config.ellipsis,
            onback:move|_|back.back.set(back.back.get()+1),
            header_content:rsx!{LocalField{id:"header-note",data:data.clone()}},
            button{id:String::from("page-run"),r#type:"button",onclick:move|_|page.page_actions.set(page.page_actions.get()+1),"Run"}
        }
    }
}
struct RawForm(String);
impl dioxus::html::HasFileData for RawForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for RawForm {
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
struct Mouse;
impl dioxus::html::InteractionLocation for Mouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for Mouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for Mouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for Mouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for Mouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event;converter!(@data $group,event) })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Mounted,$event:ident) => { MountedData::new(()) };
        (@data Form,$event:ident) => { dioxus::html::FormData::new(RawForm($event.downcast::<String>().unwrap().clone())) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected page-layout test event") };
    }
dioxus::html::with_html_event_groups!(converter);
#[derive(Default)]
struct Targets {
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    mounted: Vec<dioxus::core::ElementId>,
}
impl Targets {
    fn collect(&mut self, changes: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in changes.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    self.attrs
                        .entry(current.raw())
                        .or_default()
                        .insert(name, value);
                }
                dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                    self.mounted.push(current)
                }
                _ => {}
            }
        }
    }
    fn attr(&self, name: &str, value: &str) -> dioxus::core::ElementId {
        dioxus::core::ElementId::from_raw(
            self.attrs
                .iter()
                .find_map(|(id, attrs)| {
                    (attrs.get(name).map(String::as_str) == Some(value)).then_some(*id)
                })
                .unwrap_or_else(|| panic!("Missing {name}={value}")),
        )
    }
}
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    data: Box<dyn std::any::Any>,
) -> bool {
    let event = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(data)) as Rc<dyn std::any::Any>,
        name != "mounted",
    );
    dom.runtime().handle_event(name, event.clone(), id);
    event.default_action_enabled()
}
fn settle(dom: &mut VirtualDom, targets: &mut Targets) {
    for _ in 0..4 {
        targets.collect(dom.render_immediate_to_vec());
        dom.process_events();
    }
}
fn refresh(dom: &mut VirtualDom, targets: &mut Targets) {
    dom.mark_dirty(ScopeId::APP);
    settle(dom, targets);
}

fn setup(data: Rc<Fixture>) -> (VirtualDom, Targets) {
    let mut dom = VirtualDom::new_with_props(fixture, data);
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    for id in targets.mounted.clone() {
        dispatch(&dom, "mounted", id, Box::new(()));
    }
    settle(&mut dom, &mut targets);
    (dom, targets)
}
fn tag<'a>(html: &'a str, name: &str, value: &str) -> &'a str {
    html.split('<')
        .map(|tag| tag.split('>').next().unwrap())
        .find(|tag| tag.contains(&format!("{name}=\"{value}\"")))
        .unwrap_or_else(|| panic!("Missing {name}={value}: {html}"))
}
fn true_attribute(tag: &str, name: &str) -> bool {
    tag.trim_end_matches('/')
        .split_ascii_whitespace()
        .any(|attribute| {
            attribute == name
                || attribute == format!("{name}=true")
                || attribute == format!("{name}=\"true\"")
        })
}
#[test]
fn page_layout_tile_composition_and_actions_only_header_preserve_body_state() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Composed condition"));
    assert!(html.contains("Local sensor"));
    assert!(!html.contains("Fallback heading"));
    assert!(!html.contains("Fallback subheading"));
    assert!(tag(&html, "id", "condition").contains("aria-label=\"Condition tile\""));
    dispatch(
        &dom,
        "input",
        targets.attr("id", "tile-note"),
        Box::new(String::from("Tools ready")),
    );
    dispatch(
        &dom,
        "click",
        targets.attr("id", "tile-inspect"),
        Box::new(()),
    );
    dispatch(
        &dom,
        "click",
        targets.attr("id", "tile-units"),
        Box::new(()),
    );
    dispatch(
        &dom,
        "click",
        targets.attr("id", "tile-review"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    assert_eq!(
        (
            data.tile_actions.get(),
            data.sub_actions.get(),
            data.footer_actions.get()
        ),
        (1, 1, 1)
    );
    assert_eq!(data.back.get(), 0);
    assert_eq!(data.page_actions.get(), 0);
    data.config.borrow_mut().mode = 1;
    data.config.borrow_mut().size = TileSize::Small;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(tag(&html, "id", "condition").contains("ix-tile-small"));
    assert!(html.contains("ix-tile-header"));
    assert!(html.contains("id=\"tile-inspect\""));
    assert!(!html.contains("Composed condition"));
    assert!(tag(&html, "id", "tile-note").contains("value=\"Tools ready\""));
    dispatch(
        &dom,
        "click",
        targets.attr("id", "tile-inspect"),
        Box::new(()),
    );
    assert_eq!(data.tile_actions.get(), 2);
    data.config.borrow_mut().mode = 2;
    data.config.borrow_mut().size = TileSize::Big;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Fallback heading"));
    assert!(html.contains("Fallback subheading"));
    assert!(!html.contains("id=\"tile-inspect\""));
    assert!(!html.contains("ix-tile-footer"));
    assert!(tag(&html, "id", "tile-note").contains("value=\"Tools ready\""));
    data.config.borrow_mut().mode = 3;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("Fallback heading"));
    assert!(!html.contains("Fallback subheading"));
    assert!(tag(&html, "id", "tile-note").contains("value=\"Tools ready\""));
    assert_eq!(
        data.mounts.borrow().as_slice(),
        ["tile-note", "header-note"]
    );
}
#[test]
fn page_layout_header_native_back_and_actions_keep_custom_state_through_presentation_changes() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let (mut dom, mut targets) = setup(data.clone());
    assert!(dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Return to assets"),
        Box::new(())
    ));
    assert_eq!(data.back.get(), 1);
    dispatch(&dom, "click", targets.attr("id", "page-run"), Box::new(()));
    assert_eq!(data.page_actions.get(), 1);
    assert_eq!(data.back.get(), 1);
    dispatch(
        &dom,
        "input",
        targets.attr("id", "header-note"),
        Box::new(String::from("Keep this header control")),
    );
    settle(&mut dom, &mut targets);
    data.config.borrow_mut().variant = ContentHeaderVariant::Secondary;
    data.config.borrow_mut().ellipsis = true;
    data.config.borrow_mut().title = "Motor 001 — maintenance preparation".into();
    data.config.borrow_mut().subtitle = None;
    data.config.borrow_mut().show_back = false;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(tag(&html, "id", "asset-header").contains("data-variant=\"secondary\""));
    assert!(true_attribute(
        tag(&html, "id", "asset-header"),
        "data-ellipsis"
    ));
    assert!(tag(&html, "title", "Motor 001 — maintenance preparation").starts_with("h1 "));
    assert!(!html.contains("aria-label=\"Return to assets\""));
    assert!(tag(&html, "id", "header-note").contains("value=\"Keep this header control\""));
    data.config.borrow_mut().show_back = true;
    data.config.borrow_mut().back_label = "Zurück zur Übersicht".into();
    data.config.borrow_mut().subtitle = Some("Line 2".into());
    data.config.borrow_mut().ellipsis = false;
    refresh(&mut dom, &mut targets);
    assert!(dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Zurück zur Übersicht"),
        Box::new(())
    ));
    assert_eq!(data.back.get(), 2);
    assert_eq!(data.page_actions.get(), 1);
    let html = dioxus_ssr::render(&dom);
    assert!(tag(&html, "id", "header-note").contains("value=\"Keep this header control\""));
    assert!(tag(&html, "title", "Line 2").starts_with("p "));
    assert_eq!(
        data.mounts.borrow().as_slice(),
        ["tile-note", "header-note"]
    );
}
#[test]
fn page_layout_legacy_imports_default_tile_size_and_primary_title_semantics_stay_available() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            crate::layout::Tile{"Default body"}
            crate::Tile{size:crate::TileSize::Small,"Small body"}
            crate::Tile{size:crate::layout::TileSize::Big,header:"Big tile",subheader:"Temperature",footer:rsx!{a{href:"#details","Details"}},"Big body"}
            crate::layout::ContentHeader{title:"Production",subtitle:"Line 1",show_back:true}
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    for size in ["medium", "small", "big"] {
        assert!(html.contains(&format!("ix-tile-{size}")));
    }
    assert!(html.contains("data-variant=\"primary\""));
    assert!(!true_attribute(
        tag(&html, "data-variant", "primary"),
        "data-ellipsis"
    ));
    assert!(tag(&html, "title", "Production").starts_with("h1 "));
    assert!(tag(&html, "title", "Line 1").starts_with("p "));
    assert!(html.contains("aria-label=\"Go back\""));
    assert!(html.contains("ix-tile-footer"));
}
