//! Actual child-state and owner-callback regressions; native hidden focus is lead runtime evidence.
use super::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

#[derive(Clone)]
struct NewsConfig {
    show: bool,
    present: bool,
    epoch: u64,
    linked: Option<String>,
    label: String,
    close_label: String,
    more_label: String,
}
impl Default for NewsConfig {
    fn default() -> Self {
        Self {
            show: true,
            present: true,
            epoch: 0,
            linked: Some("details".into()),
            label: "Release bulletin".into(),
            close_label: "Close news".into(),
            more_label: "Show more".into(),
        }
    }
}
#[derive(Default)]
struct NewsFixture {
    config: Cell<Option<Signal<NewsConfig>>>,
    closes: Cell<usize>,
    more: RefCell<Vec<Option<String>>>,
    apply: Cell<bool>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
}
impl PartialEq for NewsFixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn PrivateNewsNote(data: Rc<NewsFixture>) -> Element {
    use_hook(|| data.mounts.set(data.mounts.get() + 1));
    let dropped = data.clone();
    use_drop(move || dropped.drops.set(dropped.drops.get() + 1));
    let mut note = use_signal(|| String::from("Prepare release review"));
    rsx! {crate::Input{id:"news-note",label:"Private release note",value:note(),onvaluechange:move|value|note.set(value)}}
}
fn news_fixture(data: Rc<NewsFixture>) -> Element {
    let mut config = use_signal(NewsConfig::default);
    data.config.set(Some(config));
    let current = config();
    let mut active = use_signal(|| String::from("summary"));
    let closed = data.clone();
    let more = data.clone();
    rsx! {
        if current.present {
            MenuAboutNews {key:"news-{current.epoch}",id:"news-owner",class:"local-news",label:current.label,show:current.show,
                close_label:current.close_label,show_more_label:current.more_label,active_about_tab_key:current.linked,
                onclose:move |_|{closed.closes.set(closed.closes.get()+1);if closed.apply.get(){config.write().show=false;}},
                onshowmore:move|key:Option<String>|{more.more.borrow_mut().push(key.clone());if more.apply.get(){if let Some(key)=key{active.set(key);}config.write().show=false;}},
                PrivateNewsNote{data:data.clone()}
            }
        }
        TabSet {items:vec![TabItem::new("summary","Summary"),TabItem::new("details","Details"),TabItem::new("second","Second release")],active_key:active(),
            TabPanel{tab_key:"summary","Summary panel"}
            TabPanel{tab_key:"details","First release details"}
            TabPanel{tab_key:"second","Second release details"}
        }
    }
}
struct NewsMouse;
impl dioxus::html::InteractionLocation for NewsMouse {
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
impl dioxus::html::InteractionElementOffset for NewsMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for NewsMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for NewsMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for NewsMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct NewsForm(String);
impl dioxus::html::HasFileData for NewsForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for NewsForm {
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
struct NewsConverter;
macro_rules! news_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* })=>{
        impl dioxus::html::HtmlEventConverter for NewsConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{let _=event;news_converter!(@data $group,event)})*
        }
    };
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(NewsMouse)};
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(NewsForm($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected news test event")};
}
dioxus::html::with_html_event_groups!(news_converter);
#[derive(Default)]
struct NewsTargets {
    ids: BTreeMap<String, dioxus::core::ElementId>,
    labels: BTreeMap<String, dioxus::core::ElementId>,
}
impl NewsTargets {
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
                    if name == "id" {
                        self.ids.insert(value, current);
                    } else if name == "aria-label" {
                        self.labels.insert(value, current);
                    }
                }
                _ => {}
            }
        }
    }
}
fn news_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else {
        Some(
            value
                .split(|ch: char| ch.is_whitespace() || ch == '>')
                .next()
                .unwrap(),
        )
    }
}
fn news_tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|tag| news_attribute(tag, "id") == Some(id))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn news_more(html: &str, targets: &NewsTargets) -> dioxus::core::ElementId {
    let footer = html.split("ix-about-news-footer").nth(1).unwrap();
    let button = footer
        .split('<')
        .find(|tag| tag.starts_with("button "))
        .unwrap();
    targets.ids[news_attribute(button, "id").unwrap()]
}
fn news_refresh(dom: &mut VirtualDom, targets: &mut NewsTargets) {
    for _ in 0..6 {
        targets.collect(dom.render_immediate_to_vec());
        dom.process_events();
    }
}
fn news_setup() -> (VirtualDom, Rc<NewsFixture>, NewsTargets) {
    dioxus::html::set_event_converter(Box::new(NewsConverter));
    let data = Rc::new(NewsFixture::default());
    let mut dom = VirtualDom::new_with_props(news_fixture, data.clone());
    let mut targets = NewsTargets::default();
    targets.collect(dom.rebuild_to_vec());
    (dom, data, targets)
}
fn news_event(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    value: impl std::any::Any,
) {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(value))) as Rc<dyn std::any::Any>;
    dom.runtime().handle_event(name, Event::new(data, true), id);
}
fn news_update(
    dom: &mut VirtualDom,
    data: &NewsFixture,
    targets: &mut NewsTargets,
    update: impl FnOnce(&mut NewsConfig),
) {
    dom.in_scope(ScopeId::APP, || {
        update(&mut data.config.get().unwrap().write())
    });
    news_refresh(dom, targets);
}

#[test]
fn news_close_and_linked_tab_keep_actual_child_input_until_owned_reset() {
    let _guard = crate::test_event_converter_guard();
    let (mut dom, data, mut targets) = news_setup();
    data.apply.set(true);
    let original_note = targets.ids["news-note"];
    news_event(
        &dom,
        "input",
        original_note,
        String::from("Retained release review"),
    );
    news_refresh(&mut dom, &mut targets);
    news_event(&dom, "click", targets.labels["Close news"], ());
    news_refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(news_attribute(news_tag(&html, "news-owner"), "hidden").is_some());
    assert_eq!(
        news_attribute(news_tag(&html, "news-note"), "value"),
        Some("Retained release review")
    );
    assert_eq!(data.closes.get(), 1);
    assert_eq!(data.mounts.get(), 1);
    assert_eq!(data.drops.get(), 0);
    news_update(&mut dom, &data, &mut targets, |config| {
        config.show = true;
        config.label = "New release bulletin".into();
        config.close_label = "Nachrichten schließen".into();
        config.more_label = "Mehr anzeigen".into();
        config.linked = Some("second".into());
    });
    let html = dioxus_ssr::render(&dom);
    assert!(news_attribute(news_tag(&html, "news-owner"), "hidden").is_none());
    assert_eq!(
        news_attribute(news_tag(&html, "news-owner"), "aria-label"),
        Some("New release bulletin")
    );
    assert_eq!(targets.ids["news-note"], original_note);
    assert!(html.contains("Nachrichten schließen") && html.contains("Mehr anzeigen"));
    news_event(&dom, "click", news_more(&html, &targets), ());
    news_refresh(&mut dom, &mut targets);
    assert_eq!(*data.more.borrow(), [Some(String::from("second"))]);
    assert_eq!(data.closes.get(), 1);
    let html = dioxus_ssr::render(&dom);
    assert!(news_attribute(news_tag(&html, "news-owner"), "hidden").is_some());
    let detail = html
        .split("<section")
        .skip(1)
        .find(|panel| panel.contains("Second release details"))
        .unwrap();
    assert!(
        news_attribute(detail.split('>').next().unwrap(), "hidden").is_none(),
        "Show more selected the actual current linked TabSet panel"
    );
    assert_eq!(
        news_attribute(news_tag(&html, "news-note"), "value"),
        Some("Retained release review")
    );
    assert_eq!(data.mounts.get(), 1);
    news_update(&mut dom, &data, &mut targets, |config| {
        config.show = true;
        config.epoch += 1;
    });
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        news_attribute(news_tag(&html, "news-note"), "value"),
        Some("Prepare release review")
    );
    assert_eq!(data.mounts.get(), 2);
    assert_eq!(data.drops.get(), 1);
}

#[test]
fn news_visibility_remains_controlled_and_unmount_drops_retained_child_scope() {
    let _guard = crate::test_event_converter_guard();
    let (mut dom, data, mut targets) = news_setup();
    news_event(&dom, "click", targets.labels["Close news"], ());
    news_refresh(&mut dom, &mut targets);
    assert_eq!(data.closes.get(), 1);
    let html = dioxus_ssr::render(&dom);
    assert!(
        news_attribute(news_tag(&html, "news-owner"), "hidden").is_none(),
        "callback alone must not mutate controlled show"
    );
    assert_eq!(
        news_attribute(news_tag(&html, "news-owner"), "class"),
        Some("ix-about-news local-news")
    );
    news_update(&mut dom, &data, &mut targets, |config| {
        config.show = false;
        config.linked = None;
    });
    let html = dioxus_ssr::render(&dom);
    assert!(news_attribute(news_tag(&html, "news-owner"), "hidden").is_some());
    assert!(!html.contains("ix-about-news-footer"));
    assert!(html.contains("news-note") && html.contains("Close news"));
    assert_eq!(data.mounts.get(), 1);
    assert_eq!(data.drops.get(), 0);
    news_update(&mut dom, &data, &mut targets, |config| {
        config.present = false
    });
    assert!(!dioxus_ssr::render(&dom).contains("news-owner"));
    assert_eq!(data.drops.get(), 1);
}
