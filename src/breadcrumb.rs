//! Explicit breadcrumb definitions and retained Dioxus dropdown composition.
use crate::{
    ButtonVariant, Dropdown, DropdownItem, DropdownNavigationMode, Icon, use_dropdown,
    use_dropdown_trigger_binding,
};
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Stable breadcrumb definition for managed truncation and next-item menus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreadcrumbEntry {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
    pub href: Option<String>,
    pub target: Option<String>,
    pub rel: Option<String>,
}
impl BreadcrumbEntry {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: None,
            href: None,
            target: None,
            rel: None,
        }
    }
}

/// Accessible labels for previous-level and current-location child menus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreadcrumbLabels {
    pub previous_button: String,
    pub next_button: String,
}
impl Default for BreadcrumbLabels {
    fn default() -> Self {
        Self {
            previous_button: "Show previous breadcrumb items".into(),
            next_button: "Show next breadcrumb items".into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreadcrumbNavigationSource {
    Item,
    Previous,
    Next,
}

/// A synchronous navigation request. Clones share cancellation for this request.
/// Cancelling a visible link also prevents its native browser navigation.
#[derive(Clone, Debug)]
pub struct BreadcrumbNavigateEvent {
    pub item: BreadcrumbEntry,
    pub source: BreadcrumbNavigationSource,
    prevented: Rc<Cell<bool>>,
}
impl BreadcrumbNavigateEvent {
    fn new(item: BreadcrumbEntry, source: BreadcrumbNavigationSource) -> Self {
        Self {
            item,
            source,
            prevented: Rc::new(Cell::new(false)),
        }
    }
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}
fn hidden_breadcrumb_count(count: usize, visible: usize) -> usize {
    count.saturating_sub(visible.max(1))
}

/// Ordered breadcrumb navigation. Pass `items` for explicit managed truncation
/// and child menus, or compose children and mark a final [`BreadcrumbItem`]
/// `current`. Stable unique keys retain managed menu/item ownership and order.
///
/// Visible anchors preserve native href/target/rel and modifier behavior unless
/// `onbeforenavigate` vetoes them. Menu entries are Rust navigation actions: their
/// href is included in the callback metadata but is not executed implicitly.
/// Accepted requests call `onitem` or `onnext` once. A consumer replacing the
/// current location owns focus into its new content.
#[component]
pub fn Breadcrumb(
    #[props(default = VNode::empty())] children: Element,
    #[props(default = "Breadcrumb".to_string())] label: String,
    #[props(default)] items: Vec<BreadcrumbEntry>,
    #[props(default)] next_items: Vec<BreadcrumbEntry>,
    #[props(default = 9)] visible_item_count: usize,
    #[props(default)] onitem: EventHandler<BreadcrumbEntry>,
    #[props(default)] onnext: EventHandler<BreadcrumbEntry>,
    #[props(default)] labels: BreadcrumbLabels,
    #[props(default)] subtle: bool,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] onbeforenavigate: EventHandler<BreadcrumbNavigateEvent>,
) -> Element {
    let hidden = hidden_breadcrumb_count(items.len(), visible_item_count);
    let request = use_callback(
        move |(item, source): (BreadcrumbEntry, BreadcrumbNavigationSource)| {
            let event = BreadcrumbNavigateEvent::new(item, source);
            onbeforenavigate.call(event.clone());
            if event.default_prevented() {
                return false;
            }
            if source == BreadcrumbNavigationSource::Next {
                onnext.call(event.item);
            } else {
                onitem.call(event.item);
            }
            true
        },
    );
    rsx! {
        nav { class: "ix-breadcrumb", "aria-label": label, "data-subtle": subtle,
            ol {
                if hidden > 0 {
                    BreadcrumbMenu {
                        label: "...", aria_label: labels.previous_button,
                        entries: items.iter().take(hidden).cloned().collect(),
                        source: BreadcrumbNavigationSource::Previous, current: false,
                        subtle, enable_top_layer, onrequest: request,
                    }
                }
                for (position, item) in items.iter().enumerate().skip(hidden) {
                    if position + 1 == items.len() && !next_items.is_empty() {
                        BreadcrumbMenu {
                            key: "{item.key}", label: item.label.clone(), icon: item.icon.clone(),
                            aria_label: labels.next_button.clone(), entries: next_items.clone(),
                            source: BreadcrumbNavigationSource::Next, current: true,
                            subtle, enable_top_layer, onrequest: request,
                        }
                    } else {
                        BreadcrumbItem {
                            key: "{item.key}", label: item.label.clone(), icon: item.icon.clone(),
                            href: item.href.clone(), target: item.target.clone(), rel: item.rel.clone(),
                            current: position + 1 == items.len(), subtle,
                            onclick: { let item = item.clone(); move |event: MouseEvent| {
                                if !request.call((item.clone(), BreadcrumbNavigationSource::Item)) { event.prevent_default(); }
                            } },
                        }
                    }
                }
                {children}
            }
        }
    }
}

#[component]
fn BreadcrumbMenu(
    label: String,
    aria_label: String,
    #[props(default)] icon: Option<String>,
    entries: Vec<BreadcrumbEntry>,
    source: BreadcrumbNavigationSource,
    current: bool,
    subtle: bool,
    enable_top_layer: bool,
    onrequest: Callback<(BreadcrumbEntry, BreadcrumbNavigationSource), bool>,
) -> Element {
    static NEXT_MENU: AtomicUsize = AtomicUsize::new(1);
    let trigger_id = use_hook(|| {
        format!(
            "ix-breadcrumb-trigger-{}",
            NEXT_MENU.fetch_add(1, Ordering::Relaxed)
        )
    });
    let dropdown = use_dropdown();
    let binding = use_dropdown_trigger_binding(dropdown);
    let variant = if subtle {
        ButtonVariant::SubtlePrimary
    } else {
        ButtonVariant::Tertiary
    };
    let menu_kind = if current { "next" } else { "previous" };
    rsx! {
        li { class: "ix-breadcrumb-item", "data-subtle": subtle,
            div { class: "ix-breadcrumb-menu", "data-subtle": subtle,
                button {
                    id: trigger_id.clone(), class: "ix-dropdown-trigger ix-button ix-button-{variant.as_str()}", r#type: "button",
                    "data-breadcrumb-menu": menu_kind, "aria-label": aria_label,
                    "aria-current": current.then_some("page"), "aria-haspopup": "menu",
                    "aria-expanded": dropdown.is_open(), "aria-controls": dropdown.menu_id(),
                    "aria-activedescendant": dropdown.active_descendant(),
                    onmounted: binding.onmounted, onvisible: binding.onvisible,
                    onclick: binding.onclick, onkeydown: binding.onkeydown,
                    ..binding.attributes(),
                    if let Some(icon) = icon { Icon { name: icon, size: 16 } }
                    "{label}"
                    Icon { name: "chevron-down-small", size: 16 }
                }
                Dropdown {
                    controller: dropdown, trigger: trigger_id.clone(), label, enable_top_layer,
                    navigation_mode: DropdownNavigationMode::RovingTabindex,
                    class: "ix-breadcrumb-menu-surface",
                    for (position, item) in entries.into_iter().enumerate() {
                        BreadcrumbMenuEntry { key: "{item.key}", item, source, order: position, onrequest }
                    }
                }
            }
            span { class: "ix-breadcrumb-separator", "aria-hidden": "true", Icon { name: "chevron-right", size: 16 } }
        }
    }
}

#[component]
fn BreadcrumbMenuEntry(
    item: BreadcrumbEntry,
    source: BreadcrumbNavigationSource,
    order: usize,
    onrequest: Callback<(BreadcrumbEntry, BreadcrumbNavigationSource), bool>,
) -> Element {
    // Dropdown's generic item event owns both click and keyboard activation.
    // Its compatibility mouse callback cancels the original native click only
    // when this same request was vetoed, preserving normal menu dismissal.
    let vetoed = use_hook(|| Rc::new(Cell::new(false)));
    let native_veto = vetoed.clone();
    rsx! {
        DropdownItem {
            label: item.label.clone(), icon: item.icon.clone(), order,
            "data-breadcrumb-entry": item.key.clone(),
            onitemclick: move |event: crate::DropdownItemClickEvent| {
                let accepted = onrequest.call((item.clone(), source));
                vetoed.set(!accepted);
                if !accepted { event.prevent_default(); }
            },
            onclick: move |event: MouseEvent| if native_veto.replace(false) { event.prevent_default(); },
        }
    }
}

/// Breadcrumb link or application navigation action. A current location is
/// noninteractive. Anchors retain native metadata and defaults; `onclick` may
/// prevent navigation normally.
#[component]
pub fn BreadcrumbItem(
    label: String,
    #[props(default)] href: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] current: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] target: Option<String>,
    #[props(default)] rel: Option<String>,
    #[props(default)] subtle: bool,
) -> Element {
    rsx! { li { class: "ix-breadcrumb-item", "data-subtle": subtle,
        if current { span { "aria-current": "page", if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        else if let Some(href) = href { a { href, target, rel, onclick: move |event| onclick.call(event), if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        else { button { r#type: "button", onclick: move |event| onclick.call(event), if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        span { class: "ix-breadcrumb-separator", "aria-hidden": "true", Icon { name: "chevron-right", size: 16 } }
    } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::BTreeMap};

    #[test]
    fn breadcrumb_truncation_keeps_the_current_location_visible() {
        assert_eq!(hidden_breadcrumb_count(5, 3), 2);
        assert_eq!(hidden_breadcrumb_count(3, 9), 0);
        assert_eq!(hidden_breadcrumb_count(3, 0), 2);
        assert_eq!(hidden_breadcrumb_count(0, 3), 0);
    }
    #[derive(Clone)]
    struct Config {
        items: Vec<BreadcrumbEntry>,
        next: Vec<BreadcrumbEntry>,
        visible: usize,
        subtle: bool,
        top: bool,
        labels: BreadcrumbLabels,
    }
    impl Default for Config {
        fn default() -> Self {
            let mut items: Vec<_> = [
                ("root", "Facilities"),
                ("north", "North plant"),
                ("assembly", "Assembly"),
                ("line", "Line 01"),
                ("cell", "Cell A"),
            ]
            .into_iter()
            .map(|(key, label)| BreadcrumbEntry::new(key, label))
            .collect();
            items[2].href = Some("#assembly".into());
            items[2].target = Some("_blank".into());
            items[2].rel = Some("noopener".into());
            Self {
                items,
                next: vec![
                    BreadcrumbEntry::new("motor-1", "Motor 001"),
                    BreadcrumbEntry::new("motor-2", "Motor 002"),
                ],
                visible: 3,
                subtle: false,
                top: true,
                labels: BreadcrumbLabels::default(),
            }
        }
    }
    #[derive(Default)]
    struct Fixture {
        config: RefCell<Config>,
        veto: Cell<bool>,
        before: RefCell<Vec<(String, BreadcrumbNavigationSource)>>,
        items: RefCell<Vec<BreadcrumbEntry>>,
        next: RefCell<Vec<BreadcrumbEntry>>,
    }
    fn fixture(data: Rc<Fixture>) -> Element {
        let config = data.config.borrow().clone();
        let before = data.clone();
        let item = data.clone();
        rsx! { Breadcrumb {
            items: config.items, next_items: config.next, visible_item_count: config.visible,
            labels: config.labels, subtle: config.subtle, enable_top_layer: config.top,
            onbeforenavigate: move |event: BreadcrumbNavigateEvent| {
                before.before.borrow_mut().push((event.item.key.clone(),event.source));
                if before.veto.get() { event.clone().prevent_default(); assert!(event.default_prevented()); }
            },
            onitem: move |entry| item.items.borrow_mut().push(entry),
            onnext: move |entry| data.next.borrow_mut().push(entry),
        } }
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
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event;converter!(@data $group,event) })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RawKey>().unwrap().clone()) };
        (@data Mounted,$event:ident) => { MountedData::new(()) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected breadcrumb test event") };
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
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
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
    fn mount(dom: &mut VirtualDom, targets: &mut Targets) {
        for id in targets.mounted.clone() {
            dispatch(dom, "mounted", id, Box::new(()));
        }
        settle(dom, targets);
    }
    fn opening<'a>(html: &'a str, name: &str, value: &str) -> &'a str {
        html.split('<')
            .map(|tag| tag.split('>').next().unwrap())
            .find(|tag| tag.contains(&format!("{name}=\"{value}\"")))
            .unwrap_or_else(|| panic!("Missing {name}={value}: {html}"))
    }
    fn expanded(dom: &VirtualDom, kind: &str) -> bool {
        let html = dioxus_ssr::render(dom);
        let trigger = opening(&html, "data-breadcrumb-menu", kind);
        trigger.contains("aria-expanded=\"true\"")
            || trigger
                .split_ascii_whitespace()
                .any(|attribute| attribute == "aria-expanded=true")
    }

    #[test]
    fn breadcrumb_visible_native_links_preserve_metadata_and_veto_only_the_rejected_request() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let mut dom = VirtualDom::new_with_props(fixture, data.clone());
        let mut targets = Targets::default();
        targets.collect(dom.rebuild_to_vec());
        settle(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        let anchor = opening(&html, "href", "#assembly");
        assert!(
            anchor.starts_with("a ")
                && anchor.contains("target=\"_blank\"")
                && anchor.contains("rel=\"noopener\"")
        );
        assert_eq!(html.matches("aria-current=\"page\"").count(), 1);
        assert!(opening(&html, "data-breadcrumb-menu", "next").contains("aria-current=\"page\""));
        data.veto.set(true);
        assert!(!dispatch(
            &dom,
            "click",
            targets.attr("href", "#assembly"),
            Box::new(())
        ));
        assert!(data.items.borrow().is_empty());
        data.veto.set(false);
        assert!(
            dispatch(
                &dom,
                "click",
                targets.attr("href", "#assembly"),
                Box::new(())
            ),
            "accepted link retains native default"
        );
        let entries = data.items.borrow();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].target.as_deref(), Some("_blank"));
        assert_eq!(
            *data.before.borrow(),
            [
                ("assembly".into(), BreadcrumbNavigationSource::Item),
                ("assembly".into(), BreadcrumbNavigationSource::Item)
            ]
        );
    }

    #[test]
    fn breadcrumb_retained_menus_use_distinct_requests_and_actual_open_state() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let mut dom = VirtualDom::new_with_props(fixture, data.clone());
        let mut targets = Targets::default();
        targets.collect(dom.rebuild_to_vec());
        mount(&mut dom, &mut targets);
        let previous = targets.attr("data-breadcrumb-menu", "previous");
        assert!(
            targets.mounted.contains(&previous),
            "actual previous trigger owns a mounted listener"
        );
        assert!(!expanded(&dom, "previous"));
        dispatch(&dom, "click", previous, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(
            expanded(&dom, "previous"),
            "mounted native trigger opens its actual menu"
        );
        data.veto.set(true);
        assert!(!dispatch(
            &dom,
            "click",
            targets.attr("data-breadcrumb-entry", "root"),
            Box::new(())
        ));
        settle(&mut dom, &mut targets);
        assert!(expanded(&dom, "previous"));
        assert!(data.items.borrow().is_empty());
        data.veto.set(false);
        dispatch(
            &dom,
            "click",
            targets.attr("data-breadcrumb-entry", "root"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        assert!(!expanded(&dom, "previous"));
        assert_eq!(data.items.borrow()[0].key, "root");
        let next = targets.attr("data-breadcrumb-menu", "next");
        dispatch(&dom, "click", next, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(expanded(&dom, "next"));
        dispatch(
            &dom,
            "keydown",
            targets.attr("data-breadcrumb-entry", "motor-1"),
            Box::new(RawKey(Key::Enter)),
        );
        settle(&mut dom, &mut targets);
        assert_eq!(data.next.borrow().len(), 1);
        assert_eq!(data.next.borrow()[0].key, "motor-1");
        assert_eq!(
            *data.before.borrow(),
            [
                ("root".into(), BreadcrumbNavigationSource::Previous),
                ("root".into(), BreadcrumbNavigationSource::Previous),
                ("motor-1".into(), BreadcrumbNavigationSource::Next)
            ]
        );
        assert_eq!(
            dioxus_ssr::render(&dom)
                .matches("aria-current=\"page\"")
                .count(),
            1
        );
    }

    #[test]
    fn breadcrumb_labels_layering_order_visibility_and_legacy_children_remain_explicit() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let mut dom = VirtualDom::new_with_props(fixture, data.clone());
        let mut targets = Targets::default();
        targets.collect(dom.rebuild_to_vec());
        mount(&mut dom, &mut targets);
        let trigger = targets.attr("data-breadcrumb-menu", "previous");
        let first = dioxus_ssr::render(&dom);
        assert_eq!(first.matches("popover=\"manual\"").count(), 2);
        assert!(
            opening(&first, "data-breadcrumb-menu", "next")
                .contains("aria-controls=\"ix-dropdown-")
        );
        {
            let mut config = data.config.borrow_mut();
            config.items.swap(0, 1);
            config.visible = 0;
            config.subtle = true;
            config.top = false;
            config.labels = BreadcrumbLabels {
                previous_button: "Vorherige Ebenen".into(),
                next_button: "Untere Ebenen".into(),
            };
        }
        refresh(&mut dom, &mut targets);
        let changed = dioxus_ssr::render(&dom);
        assert_eq!(
            targets.attr("data-breadcrumb-menu", "previous"),
            trigger,
            "options/reorder retain native trigger ownership"
        );
        assert!(
            changed.contains("Vorherige Ebenen")
                && changed.contains("Untere Ebenen")
                && changed.contains("ix-button-subtle-primary")
        );
        assert!(!changed.contains("popover=\"manual\""));
        assert!(
            changed.find("data-breadcrumb-entry=\"north\"")
                < changed.find("data-breadcrumb-entry=\"root\"")
        );
        assert_eq!(changed.matches("aria-current=\"page\"").count(), 1);
        fn legacy() -> Element {
            rsx! { Breadcrumb {
                BreadcrumbItem{label:"Old parent",href:"#legacy",target:"_blank",rel:"noreferrer"}
                BreadcrumbItem{label:"Current",current:true}
            } }
        }
        let mut old = VirtualDom::new(legacy);
        old.rebuild_in_place();
        let old = dioxus_ssr::render(&old);
        assert!(old.contains("Old parent") && old.contains("Current"));
        assert_eq!(old.matches("aria-current=\"page\"").count(), 1);
        assert!(opening(&old, "href", "#legacy").contains("rel=\"noreferrer\""));
        assert!(opening(&old, "aria-current", "page").starts_with("span "));
    }
}
