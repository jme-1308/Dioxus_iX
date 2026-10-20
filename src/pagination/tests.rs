use super::*;
use std::{cell::RefCell, collections::BTreeMap};

#[test]
fn pagination_empty_and_small_ranges_are_exact() {
    assert!(page_tokens(0, 5).is_empty());
    assert_eq!(page_tokens(1, 20), vec![PageToken::Page(0)]);
    assert_eq!(
        page_tokens(7, 3),
        (0..7).map(PageToken::Page).collect::<Vec<_>>()
    );
}
#[test]
fn pagination_windows_preserve_edges_current_and_valid_jump_targets() {
    for count in 8..80 {
        for selected in 0..count + 2 {
            let tokens = page_tokens(count, selected);
            assert_eq!(tokens.first(), Some(&PageToken::Page(0)));
            assert_eq!(tokens.last(), Some(&PageToken::Page(count - 1)));
            assert!(tokens.contains(&PageToken::Page(selected.min(count - 1))));
            assert!(tokens.len() <= 7);
            let indices: Vec<_> = tokens
                .iter()
                .map(|token| match *token {
                    PageToken::Page(index) | PageToken::Jump(index) => index,
                })
                .collect();
            assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
            assert!(indices.iter().all(|index| *index < count));
        }
    }
}

#[derive(Clone)]
struct Config {
    count: usize,
    selected: usize,
    advanced: bool,
    labels: PaginationLabels,
    item_count: usize,
    options: Vec<usize>,
    hide_size: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            count: 17,
            selected: 0,
            advanced: false,
            labels: PaginationLabels::default(),
            item_count: 15,
            options: vec![10, 15, 20, 40, 100],
            hide_size: false,
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    apply: Cell<bool>,
    veto: Cell<bool>,
    before: RefCell<Vec<(usize, usize)>>,
    accepted: RefCell<Vec<usize>>,
    sizes: RefCell<Vec<usize>>,
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let before = data.clone();
    let accepted = data.clone();
    rsx! { Pagination {
        count: config.count, selected_page: config.selected, advanced: config.advanced,
        labels: config.labels, label: "Equipment pages", item_count: config.item_count,
        item_count_options: config.options, hide_item_count: config.hide_size,
        onbeforepage: move |event: PaginationChangeEvent| {
            before.before.borrow_mut().push((event.previous, event.next));
            if before.veto.get() { event.clone().prevent_default(); assert!(event.default_prevented()); }
        },
        onpage: move |page| {
            accepted.accepted.borrow_mut().push(page);
            if accepted.apply.get() { accepted.config.borrow_mut().selected = page; }
        },
        onitemcount: move |size| data.sizes.borrow_mut().push(size),
    } }
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
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
struct RawKey(Key);
impl dioxus::html::ModifiersInteraction for RawKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for RawKey {
    fn key(&self) -> Key {
        self.0.clone()
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Unidentified
    }
    fn location(&self) -> dioxus::html::Location {
        dioxus::html::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
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
                $(fn $convert(&self, event: &dioxus::html::PlatformEventData) -> dioxus::html::$data {
                    let _ = event; converter!(@data $group, event)
                })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Form,$event:ident) => { dioxus::html::FormData::new(RawForm($event.downcast::<String>().unwrap().clone())) };
        (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RawKey>().unwrap().clone()) };
        (@data Focus,$event:ident) => { dioxus::html::FocusData::new(Focus) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected pagination test event") };
    }
dioxus::html::with_html_event_groups!(converter);
#[derive(Default)]
struct Targets(BTreeMap<usize, BTreeMap<String, String>>);
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
                    self.0.entry(current.raw()).or_default().insert(name, value);
                }
                dioxus::core::Mutation::Remove => {
                    self.0.remove(&current.raw());
                }
                _ => {}
            }
        }
    }
    fn label(&self, label: &str) -> dioxus::core::ElementId {
        dioxus::core::ElementId::from_raw(
            self.0
                .iter()
                .find_map(|(id, attrs)| {
                    (attrs.get("aria-label").map(String::as_str) == Some(label)).then_some(*id)
                })
                .unwrap_or_else(|| panic!("Missing {label}")),
        )
    }
}
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: Box<dyn std::any::Any>,
) -> bool {
    let event = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(payload)) as Rc<dyn std::any::Any>,
        true,
    );
    dom.runtime().handle_event(name, event.clone(), id);
    event.default_action_enabled()
}
fn settle(dom: &mut VirtualDom, targets: &mut Targets) {
    for _ in 0..3 {
        targets.collect(dom.render_immediate_to_vec());
        dom.process_events();
    }
}
fn refresh(dom: &mut VirtualDom, targets: &mut Targets) {
    dom.mark_dirty(ScopeId::APP);
    settle(dom, targets);
}
fn input(dom: &mut VirtualDom, targets: &mut Targets, text: &str) {
    dispatch(
        dom,
        "input",
        targets.label("Page selection"),
        Box::new(text.to_string()),
    );
    settle(dom, targets);
}
fn value(html: &str, text: &str) -> bool {
    html.contains(&format!("value=\"{text}\"")) || html.contains(&format!("value={text}"))
}

#[test]
fn pagination_actual_buttons_use_controlled_requests_veto_and_native_boundaries() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.apply.set(true);
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets);
    dispatch(&dom, "click", targets.label("Previous page"), Box::new(()));
    assert!(
        data.before.borrow().is_empty(),
        "disabled previous cannot request"
    );
    dispatch(&dom, "click", targets.label("Next page"), Box::new(()));
    refresh(&mut dom, &mut targets);
    assert_eq!(*data.accepted.borrow(), [1]);
    dispatch(&dom, "click", targets.label("Page 17"), Box::new(()));
    refresh(&mut dom, &mut targets);
    assert_eq!(*data.accepted.borrow(), [1, 16]);
    dispatch(&dom, "click", targets.label("Next page"), Box::new(()));
    assert_eq!(data.before.borrow().len(), 2);
    data.veto.set(true);
    dispatch(&dom, "click", targets.label("Page 1"), Box::new(()));
    refresh(&mut dom, &mut targets);
    assert_eq!(data.before.borrow().last(), Some(&(16, 0)));
    assert_eq!(data.accepted.borrow().len(), 2);
    data.veto.set(false);
    data.apply.set(false);
    dispatch(&dom, "click", targets.label("Previous page"), Box::new(()));
    refresh(&mut dom, &mut targets);
    assert_eq!(data.accepted.borrow().last(), Some(&15));
    assert_eq!(
        data.config.borrow().selected,
        16,
        "held request keeps host authority"
    );
    assert!(dioxus_ssr::render(&dom).contains("aria-current=\"page\""));
}

#[test]
fn pagination_actual_draft_commits_once_restores_and_discards_external_stale_values() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    {
        let mut config = data.config.borrow_mut();
        config.count = 10;
        config.selected = 2;
        config.advanced = true;
    }
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets);
    input(&mut dom, &mut targets, "9");
    assert!(value(&dioxus_ssr::render(&dom), "9"));
    assert!(
        !dispatch(
            &dom,
            "keydown",
            targets.label("Page selection"),
            Box::new(RawKey(Key::Enter))
        ),
        "Enter prevents parent form submission"
    );
    settle(&mut dom, &mut targets);
    assert_eq!(*data.before.borrow(), [(2, 8)]);
    assert_eq!(*data.accepted.borrow(), [8]);
    assert!(
        value(&dioxus_ssr::render(&dom), "3"),
        "controlled held request restores page"
    );
    dispatch(
        &dom,
        "change",
        targets.label("Page selection"),
        Box::new("9".to_string()),
    );
    dispatch(&dom, "blur", targets.label("Page selection"), Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(
        data.before.borrow().len(),
        1,
        "Enter/change/blur deduplicated"
    );
    data.veto.set(true);
    input(&mut dom, &mut targets, "4");
    dispatch(
        &dom,
        "change",
        targets.label("Page selection"),
        Box::new("4".to_string()),
    );
    settle(&mut dom, &mut targets);
    assert_eq!(data.before.borrow().last(), Some(&(2, 3)));
    assert_eq!(data.accepted.borrow().len(), 1);
    assert!(value(&dioxus_ssr::render(&dom), "3"));
    data.veto.set(false);
    for invalid in ["", "2.5", "NaN"] {
        input(&mut dom, &mut targets, invalid);
        dispatch(
            &dom,
            "change",
            targets.label("Page selection"),
            Box::new(invalid.to_string()),
        );
        settle(&mut dom, &mut targets);
    }
    assert_eq!(data.before.borrow().len(), 2);
    input(&mut dom, &mut targets, "7");
    assert!(!dispatch(
        &dom,
        "keydown",
        targets.label("Page selection"),
        Box::new(RawKey(Key::Escape))
    ));
    settle(&mut dom, &mut targets);
    assert!(value(&dioxus_ssr::render(&dom), "3"));
    assert_eq!(data.before.borrow().len(), 2);
    for (text, next) in [("-5", 0), ("200", 9)] {
        input(&mut dom, &mut targets, text);
        dispatch(
            &dom,
            "change",
            targets.label("Page selection"),
            Box::new(text.to_string()),
        );
        settle(&mut dom, &mut targets);
        assert_eq!(data.accepted.borrow().last(), Some(&next));
    }
    input(&mut dom, &mut targets, "8");
    {
        let mut config = data.config.borrow_mut();
        config.count = 2;
        config.selected = 99;
    }
    refresh(&mut dom, &mut targets);
    assert!(value(&dioxus_ssr::render(&dom), "2"));
    dispatch(&dom, "blur", targets.label("Page selection"), Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(
        data.accepted.borrow().len(),
        3,
        "external authority discards uncommitted draft"
    );
    data.config.borrow_mut().count = 0;
    refresh(&mut dom, &mut targets);
    assert!(value(&dioxus_ssr::render(&dom), "0"));
    dispatch(&dom, "click", targets.label("Previous page"), Box::new(()));
    dispatch(&dom, "click", targets.label("Next page"), Box::new(()));
    input(&mut dom, &mut targets, "1");
    dispatch(
        &dom,
        "keydown",
        targets.label("Page selection"),
        Box::new(RawKey(Key::Enter)),
    );
    assert_eq!(data.accepted.borrow().len(), 3);
}

#[test]
fn pagination_localized_labels_and_native_size_options_keep_actual_authority() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    {
        let mut config = data.config.borrow_mut();
        config.labels = PaginationLabels {
            page: "Seite".into(),
            of: "von".into(),
            items: "Einträge".into(),
            previous_page: "Vorherige Seite".into(),
            next_page: "Nächste Seite".into(),
            page_selection: "Seitenauswahl".into(),
            items_per_page: "Einträge pro Seite".into(),
            jump_to_page: "Zu Seite {page} springen".into(),
        };
        config.options = vec![0, 5, 5, 20];
        config.item_count = 15;
    }
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains("aria-label=\"Equipment pages\"") && html.contains("aria-label=\"Seite 1\"")
    );
    assert!(html.contains("Zu Seite 11 springen"));
    data.config.borrow_mut().advanced = true;
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains("Seitenauswahl")
            && html.contains("Einträge pro Seite")
            && html.contains("von 17")
    );
    assert_eq!(
        html.matches("<option").count(),
        3,
        "positive unique options include positive current count"
    );
    let current = html
        .split("<option")
        .skip(1)
        .map(|tag| tag.split('>').next().unwrap())
        .find(|tag| value(tag, "15"))
        .unwrap();
    assert!(
        current.contains("selected"),
        "native current option must be actually selected: {html}"
    );
    dispatch(
        &dom,
        "change",
        targets.label("Einträge pro Seite"),
        Box::new("20".to_string()),
    );
    dispatch(
        &dom,
        "change",
        targets.label("Einträge pro Seite"),
        Box::new("0".to_string()),
    );
    dispatch(
        &dom,
        "change",
        targets.label("Einträge pro Seite"),
        Box::new("99".to_string()),
    );
    assert_eq!(*data.sizes.borrow(), [20]);
    data.config.borrow_mut().hide_size = true;
    refresh(&mut dom, &mut targets);
    assert!(!dioxus_ssr::render(&dom).contains("<select"));
}
