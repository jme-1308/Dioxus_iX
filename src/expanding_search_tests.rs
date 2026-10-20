// Source state transitions and native rendering adapters. SSR does not prove
// physical pointer behavior, geometry or computed token paint. The mounted
// backing below separately exercises Dioxus-owned focus delivery and lifetime.
use super::*;
use std::cell::RefCell;

fn attribute<'a>(opening: &'a str, name: &str) -> Option<&'a str> {
    let value = opening.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else if let Some(value) = value.strip_prefix('\'') {
        Some(value.split('\'').next().unwrap())
    } else {
        Some(
            value
                .split(|c: char| c.is_whitespace() || c == '>')
                .next()
                .unwrap(),
        )
    }
}
fn classes(opening: &str) -> Vec<&str> {
    attribute(opening, "class")
        .unwrap_or_default()
        .split_whitespace()
        .collect()
}
fn has_attribute(opening: &str, name: &str) -> bool {
    opening
        .match_indices(&format!(" {name}"))
        .any(|(index, _)| {
            opening[index + name.len() + 1..]
                .chars()
                .next()
                .is_none_or(|c| c.is_whitespace() || c == '=' || c == '>')
        })
}
fn openings(html: &str) -> impl Iterator<Item = &str> {
    html.match_indices('<').filter_map(move |(index, _)| {
        let rest = &html[index..];
        Some(&rest[..=rest.find('>')?])
    })
}
fn part<'a>(html: &'a str, marker: &str) -> (&'a str, &'a str) {
    let (start, opening) = html
        .match_indices('<')
        .find_map(|(index, _)| {
            let rest = &html[index..];
            let opening = &rest[..=rest.find('>')?];
            classes(opening)
                .contains(&marker)
                .then_some((index, opening))
        })
        .unwrap_or_else(|| panic!("missing {marker}: {html}"));
    let tag = opening[1..]
        .split(|c: char| c.is_whitespace() || c == '>')
        .next()
        .unwrap();
    let closing = format!("</{tag}>");
    let rest = &html[start..];
    let mut depth = 0usize;
    for (index, _) in rest.match_indices('<') {
        let current = &rest[index..];
        if current.starts_with(&format!("<{tag} ")) || current.starts_with(&format!("<{tag}>")) {
            depth += 1;
        } else if current.starts_with(&closing) {
            depth -= 1;
            if depth == 0 {
                return (opening, &rest[..index + closing.len()]);
            }
        }
    }
    panic!("unclosed {marker}: {rest}");
}
fn text_content(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text
}
fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn expanding_search_state_preserves_source_empty_toggle_blur_and_clear_contract() {
    let initial = ExpandingSearchState::default();
    assert!(!initial.expanded && !initial.changed);
    // Input and blur are separate source transitions: typing never expands the
    // field, and a nonempty controlled prop alone never marks it changed.
    for expanded in [false, true] {
        for (changed, value, expected_changed) in [
            (false, "", true),
            (false, "query", true),
            (true, "", false),
            (true, "query", true),
        ] {
            let mut state = ExpandingSearchState { expanded, changed };
            state.input(value);
            assert_eq!(state.expanded, expanded);
            assert_eq!(state.changed, expected_changed);
        }
        for changed in [false, true] {
            let mut state = ExpandingSearchState { expanded, changed };
            state.blur();
            assert_eq!(state.expanded, expanded && changed);
            assert_eq!(state.changed, changed);
            let mut state = ExpandingSearchState { expanded, changed };
            state.clear();
            assert_eq!(
                state.expanded, expanded,
                "clear preserves expansion until an unchanged blur"
            );
            assert!(!state.changed);
            let mut state = ExpandingSearchState { expanded, changed };
            state.expand();
            assert!(state.expanded);
            assert_eq!(state.changed, changed);
        }
    }
    let mut state = ExpandingSearchState::default();
    state.expand();
    state.input("");
    state.blur();
    assert!(
        state.expanded && state.changed,
        "first empty input counts as changed"
    );
    state.input("");
    state.blur();
    assert!(
        !state.expanded && !state.changed,
        "second empty input clears the changed flag"
    );
    state.expand();
    state.input("retained");
    state.blur();
    assert!(state.expanded && state.changed);
    state.clear();
    assert!(state.expanded && !state.changed);
    state.blur();
    assert!(!state.expanded && !state.changed);
}

#[test]
fn expanding_search_initial_markup_retains_native_input_for_presets_and_all_variants() {
    let html = render_fixture(|| {
        rsx! {
            ExpandingSearch { class: "default-search" }
            ExpandingSearch { class: "preset-search", value: "Existing query", full_width: true }
            ExpandingSearch { class: "custom-search", icon: "add", placeholder: "Find equipment",
                aria_label_search_icon_button: "Start equipment search", aria_label_search_input: "Equipment search", aria_label_clear_icon_button: "Remove query" }
            ExpandingSearch { class: "empty-icon-search", icon: "" }
            for variant in [crate::ButtonVariant::Primary, crate::ButtonVariant::Secondary, crate::ButtonVariant::Tertiary,
                crate::ButtonVariant::DangerPrimary, crate::ButtonVariant::DangerSecondary, crate::ButtonVariant::DangerTertiary,
                crate::ButtonVariant::SubtlePrimary, crate::ButtonVariant::SubtleSecondary, crate::ButtonVariant::SubtleTertiary] {
                ExpandingSearch { class: "variant-{variant.as_str()}", variant }
            }
            Input { class: "input-peer" }
        }
    });
    for marker in [
        "default-search",
        "preset-search",
        "custom-search",
        "empty-icon-search",
        "variant-primary",
        "variant-secondary",
        "variant-tertiary",
        "variant-danger-primary",
        "variant-danger-secondary",
        "variant-danger-tertiary",
        "variant-subtle-primary",
        "variant-subtle-secondary",
        "variant-subtle-tertiary",
    ] {
        let (root, body) = part(&html, marker);
        assert!(classes(root).contains(&"ix-expanding-search"));
        assert_eq!(attribute(root, "data-expanded"), Some("false"));
        assert_eq!(attribute(root, "data-changed"), Some("false"));
        assert!(!classes(root).contains(&"expanded"));
        assert_eq!(
            classes(root).contains(&"fullWidth"),
            marker == "preset-search"
        );
        let (host, host_body) = part(body, "ix-search-trigger-host");
        assert!(host.starts_with("<span "));
        assert_eq!(attribute(host, "data-expanded"), Some("false"));
        let (button, button_body) = part(host_body, "ix-search-trigger");
        assert!(button.starts_with("<button "));
        for class in [
            "ix-button-tokenized",
            "ix-icon-button-native",
            "ix-search-button",
            "ix-icon-size-24",
        ] {
            assert!(classes(button).contains(&class));
        }
        let variant = marker.strip_prefix("variant-").unwrap_or("tertiary");
        assert!(classes(button).contains(&format!("ix-button-{variant}").as_str()));
        assert_eq!(attribute(button, "type"), Some("button"));
        assert_eq!(attribute(button, "tabindex"), Some("0"));
        assert_eq!(
            attribute(button, "aria-label"),
            Some(if marker == "custom-search" {
                "Start equipment search"
            } else {
                "Open search"
            })
        );
        assert_eq!(attribute(button, "aria-expanded"), Some("false"));
        assert_eq!(
            button_body.contains("ix-icon-button-glyph"),
            marker != "empty-icon-search"
        );
        assert_eq!(
            text_content(button_body),
            "",
            "search icon button has no invented visible text"
        );
        if marker != "empty-icon-search" {
            assert!(
                button_body.contains(
                    crate::icons::icon_svg(if marker == "custom-search" {
                        "add"
                    } else {
                        "search"
                    })
                    .unwrap()
                )
            );
        }
        let (container, container_body) = part(body, "ix-search-input-container");
        assert_eq!(attribute(container, "data-expanded"), Some("false"));
        let native = openings(container_body)
            .find(|opening| opening.starts_with("<input "))
            .unwrap();
        assert!(classes(native).contains(&"ix-search-input"));
        assert_eq!(attribute(native, "type"), Some("text"));
        assert_eq!(attribute(native, "tabindex"), Some("-1"));
        assert_eq!(
            attribute(native, "aria-label"),
            Some(if marker == "custom-search" {
                "Equipment search"
            } else {
                "Search input"
            })
        );
        assert_eq!(
            attribute(native, "placeholder"),
            Some(if marker == "custom-search" {
                "Find equipment"
            } else {
                "Enter text here"
            })
        );
        assert_eq!(
            attribute(native, "value"),
            Some(if marker == "preset-search" {
                "Existing query"
            } else {
                ""
            })
        );
        assert!(!has_attribute(native, "disabled"));
        assert!(
            !body.contains("ix-search-clear"),
            "initial value does not fabricate changed state or a clear button"
        );
    }
    assert!(!classes(part(&html, "input-peer").0).contains(&"ix-expanding-search"));
}

#[derive(Clone)]
struct SearchLiveState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    requests: Rc<RefCell<Vec<String>>>,
}
fn live_search(state: SearchLiveState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let changes = state.requests.clone();
    let searches = state.requests.clone();
    rsx! {
        ExpandingSearch { class: "live-search", value: if current == 1 { "external query" } else { "" },
            placeholder: if current == 2 { "Revised placeholder" } else { "Enter text here" }, full_width: current == 1,
            disabled: current == 2, variant: if current == 1 { crate::ButtonVariant::DangerPrimary } else { crate::ButtonVariant::Tertiary },
            icon: match current { 1 => Some("add".to_string()), 2 => Some(String::new()), _ => None },
            aria_label_search_icon_button: if current == 1 { Some("Custom open".to_string()) } else { None },
            aria_label_search_input: if current == 1 { "Custom input" } else { "Search input" },
            aria_label_clear_icon_button: "Custom clear",
            onvaluechange: move |value| changes.borrow_mut().push(format!("change:{value}")),
            onsearch: move |value| searches.borrow_mut().push(format!("submit:{value}")),
        }
    }
}

#[test]
fn expanding_search_external_props_restore_native_attributes_without_changing_source_flags() {
    let state = SearchLiveState {
        stage: Rc::new(RefCell::new(None)),
        requests: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_search, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    for current in [0, 1, 2, 0] {
        dom.in_scope(ScopeId::ROOT, || stage.set(current));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        let (root, body) = part(&html, "live-search");
        assert_eq!(attribute(root, "data-expanded"), Some("false"));
        assert_eq!(attribute(root, "data-changed"), Some("false"));
        assert_eq!(classes(root).contains(&"fullWidth"), current == 1);
        let (button, button_body) = part(body, "ix-search-trigger");
        assert_eq!(
            attribute(button, "aria-label"),
            Some(if current == 1 {
                "Custom open"
            } else {
                "Open search"
            })
        );
        assert_eq!(has_attribute(button, "disabled"), current == 2);
        assert_eq!(
            attribute(button, "tabindex"),
            Some(if current == 2 { "-1" } else { "0" })
        );
        assert!(classes(button).contains(&if current == 1 {
            "ix-button-danger-primary"
        } else {
            "ix-button-tertiary"
        }));
        assert_eq!(button_body.contains("ix-icon-button-glyph"), current != 2);
        if current != 2 {
            assert!(button_body.contains(
                crate::icons::icon_svg(if current == 1 { "add" } else { "search" }).unwrap()
            ));
        }
        let (_, container) = part(body, "ix-search-input-container");
        let input = openings(container)
            .find(|opening| opening.starts_with("<input "))
            .unwrap();
        assert_eq!(
            attribute(input, "value"),
            Some(if current == 1 { "external query" } else { "" })
        );
        assert_eq!(attribute(input, "type"), Some("text"));
        assert_eq!(attribute(input, "tabindex"), Some("-1"));
        assert_eq!(
            attribute(input, "aria-label"),
            Some(if current == 1 {
                "Custom input"
            } else {
                "Search input"
            })
        );
        assert_eq!(
            attribute(input, "placeholder"),
            Some(if current == 2 {
                "Revised placeholder"
            } else {
                "Enter text here"
            })
        );
        assert_eq!(has_attribute(input, "disabled"), current == 2);
        assert!(!body.contains("ix-search-clear"));
        assert!(
            state.requests.borrow().is_empty(),
            "prop updates do not emit source value requests or the optional Rust Enter adapter"
        );
    }
}

#[derive(Default)]
struct SearchFocusFixture {
    disabled: std::cell::Cell<bool>,
    hidden: std::cell::Cell<bool>,
    epoch: std::cell::Cell<usize>,
    focuses: Rc<RefCell<Vec<usize>>>,
    changes: RefCell<Vec<String>>,
    searches: RefCell<Vec<String>>,
}
fn search_focus_fixture(fixture: Rc<SearchFocusFixture>) -> Element {
    let mut value = use_signal(String::new);
    let changes = fixture.clone();
    let searches = fixture.clone();
    rsx! {
        if !fixture.hidden.get() {
            ExpandingSearch { key:"{fixture.epoch.get()}",value:value(),disabled:fixture.disabled.get(),
                onvaluechange:move|next:String|{changes.changes.borrow_mut().push(next.clone());value.set(next);},
                onsearch:move|query|searches.searches.borrow_mut().push(query),
            }
        }
    }
}
#[derive(Clone)]
struct SearchMounted {
    epoch: usize,
    focuses: Rc<RefCell<Vec<usize>>>,
}
impl dioxus::html::RenderedElementBacking for SearchMounted {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let mount = self.clone();
        Box::pin(async move {
            mount.focuses.borrow_mut().push(mount.epoch);
            Ok(())
        })
    }
}
struct SearchMouse;
impl dioxus::html::InteractionLocation for SearchMouse {
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
impl dioxus::html::InteractionElementOffset for SearchMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for SearchMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for SearchMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for SearchMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct SearchForm(String);
impl dioxus::html::HasFileData for SearchForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for SearchForm {
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
struct SearchFocus;
impl dioxus::html::HasFocusData for SearchFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct SearchKey;
impl dioxus::html::ModifiersInteraction for SearchKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for SearchKey {
    fn key(&self) -> Key {
        Key::Enter
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Enter
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
struct SearchConverter;
macro_rules! search_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for SearchConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                let _=event;search_converter!(@data $group,event)
            })*
        }
    };
    (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<SearchMounted>().unwrap().clone())};
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(SearchMouse)};
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(SearchForm($event.downcast::<String>().unwrap().clone()))};
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(SearchFocus)};
    (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new(SearchKey)};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected expanding-search event")};
}
dioxus::html::with_html_event_groups!(search_converter);
struct SearchFocusHarness {
    dom: VirtualDom,
    fixture: Rc<SearchFocusFixture>,
    labels: std::collections::BTreeMap<String, dioxus::core::ElementId>,
}
impl SearchFocusHarness {
    fn new() -> Self {
        let fixture = Rc::new(SearchFocusFixture::default());
        let mut dom = VirtualDom::new_with_props(search_focus_fixture, fixture.clone());
        let mutations = dom.rebuild_to_vec();
        let mut h = Self {
            dom,
            fixture,
            labels: Default::default(),
        };
        h.collect(mutations);
        h.settle();
        h
    }
    fn collect(&mut self, mutations: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(label),
                    ..
                } if name == "aria-label" => {
                    self.labels.insert(label, current);
                }
                dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                    let mounted = SearchMounted {
                        epoch: self.fixture.epoch.get(),
                        focuses: self.fixture.focuses.clone(),
                    };
                    self.event("mounted", current, Box::new(mounted));
                }
                _ => {}
            }
        }
    }
    fn event(
        &self,
        event: &str,
        id: dioxus::core::ElementId,
        data: Box<dyn std::any::Any>,
    ) -> bool {
        let data = Event::new(
            Rc::new(dioxus::html::PlatformEventData::new(data)) as Rc<dyn std::any::Any>,
            event != "mounted",
        );
        self.dom.runtime().handle_event(event, data.clone(), id);
        data.default_action_enabled()
    }
    fn click(&self, label: &str) {
        self.event("click", self.labels[label], Box::new(()));
    }
    fn input(&self, value: &str) {
        self.event(
            "input",
            self.labels["Search input"],
            Box::new(value.to_owned()),
        );
    }
    fn blur(&self) {
        self.event("blur", self.labels["Search input"], Box::new(()));
    }
    fn render(&mut self) {
        let mutations = self.dom.render_immediate_to_vec();
        self.collect(mutations);
    }
    fn refresh(&mut self) {
        self.dom.mark_dirty(ScopeId::APP);
        self.render();
    }
    fn settle(&mut self) {
        for _ in 0..4 {
            self.render();
            self.dom.process_events();
        }
    }
    fn expanded(&self) -> bool {
        attribute(
            part(&dioxus_ssr::render(&self.dom), "ix-expanding-search").0,
            "data-expanded",
        ) == Some("true")
    }
}

#[test]
fn expanding_search_focus_follows_rendered_open_clear_and_controlled_events() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SearchConverter));
    let mut h = SearchFocusHarness::new();
    assert!(h.fixture.focuses.borrow().is_empty());
    h.click("Open search");
    assert!(
        h.fixture.focuses.borrow().is_empty(),
        "activation waits for the expanded render"
    );
    h.render();
    assert!(h.expanded());
    h.settle();
    assert_eq!(*h.fixture.focuses.borrow(), vec![0]);
    h.input("Motor 42");
    h.settle();
    assert_eq!(*h.fixture.changes.borrow(), vec!["Motor 42"]);
    assert!(!h.event("keydown", h.labels["Search input"], Box::new(())));
    assert_eq!(*h.fixture.searches.borrow(), vec!["Motor 42"]);
    h.blur();
    h.settle();
    assert!(h.expanded(), "a changed search remains open on blur");
    h.click("Clear search");
    assert_eq!(
        h.fixture.focuses.borrow().len(),
        1,
        "clear focus follows its render"
    );
    h.settle();
    assert_eq!(*h.fixture.changes.borrow(), vec!["Motor 42", ""]);
    assert_eq!(*h.fixture.focuses.borrow(), vec![0, 0]);
    assert!(h.expanded());
    let html = dioxus_ssr::render(&h.dom);
    let input = openings(&html)
        .find(|tag| classes(tag).contains(&"ix-search-input"))
        .unwrap();
    assert_eq!(attribute(input, "value"), Some(""));
    assert!(!html.contains("ix-search-clear-host"));
    h.blur();
    h.settle();
    assert!(!h.expanded(), "cleared unchanged search collapses on blur");
    assert_eq!(*h.fixture.focuses.borrow(), vec![0, 0]);
}

#[test]
fn expanding_search_pending_focus_respects_current_state_and_component_lifetime() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SearchConverter));
    let mut h = SearchFocusHarness::new();
    h.click("Open search");
    h.blur();
    h.settle();
    assert!(!h.expanded());
    assert!(
        h.fixture.focuses.borrow().is_empty(),
        "a collapsed input is not focused"
    );
    h.click("Open search");
    h.fixture.disabled.set(true);
    h.refresh();
    h.settle();
    assert!(
        h.fixture.focuses.borrow().is_empty(),
        "latest disabled props guard queued focus"
    );
    h.fixture.disabled.set(false);
    h.refresh();
    h.settle();
    assert!(
        h.fixture.focuses.borrow().is_empty(),
        "reenabling does not replay focus"
    );
    h.click("Close search");
    h.fixture.hidden.set(true);
    h.refresh();
    h.settle();
    assert!(
        h.fixture.focuses.borrow().is_empty(),
        "Dioxus cancels the removed owner's queued effect"
    );
    h.fixture.hidden.set(false);
    h.fixture.epoch.set(1);
    h.refresh();
    h.settle();
    h.click("Open search");
    h.settle();
    assert_eq!(
        *h.fixture.focuses.borrow(),
        vec![1],
        "remounted search focuses its live input only"
    );
}
