//! Joined primary actions composed with the shared Dioxus Dropdown.
use crate::{
    Button, ButtonVariant, Dropdown, DropdownChangeEvent, DropdownCloseBehavior, DropdownItem,
    DropdownNavigationMode, DropdownPlacement, DropdownTrigger, IconButton, use_dropdown,
};
use dioxus::prelude::*;

/// An alternative action in a split button. `value` is returned to `onselect`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitButtonItem {
    /// Stable action identifier, unique among the supplied items.
    pub value: String,
    /// Visible action label.
    pub label: String,
    /// Optional leading icon.
    pub icon: Option<String>,
    /// Prevent activation of this item.
    pub disabled: bool,
}
impl SplitButtonItem {
    /// Construct an enabled action.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }
}

/// Which ordinary menu interactions dismiss the alternatives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitCloseBehavior {
    #[default]
    Both,
    Inside,
    Outside,
    None,
}
impl SplitCloseBehavior {
    /// The corresponding upstream token.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Both => "both",
            Self::Inside => "inside",
            Self::Outside => "outside",
            Self::None => "none",
        }
    }
}
impl From<SplitCloseBehavior> for DropdownCloseBehavior {
    fn from(value: SplitCloseBehavior) -> Self {
        match value {
            SplitCloseBehavior::Both => Self::Both,
            SplitCloseBehavior::Inside => Self::Inside,
            SplitCloseBehavior::Outside => Self::Outside,
            SplitCloseBehavior::None => Self::None,
        }
    }
}

/// Primary action and retained alternatives, using shared Dropdown ownership,
/// keyboard navigation and dismissal. Composed `children` take precedence over
/// `items` and own their action callbacks. Legacy items notify `onselect` once.
///
/// `aria_label_button` names the primary action independently of its visible
/// text. Disabling all actions or the alternatives closes their menu; this
/// configuration change cannot be vetoed. Ordinary menu changes are cancelable.
#[component]
pub fn SplitButton(
    label: String,
    #[props(default)] items: Vec<SplitButtonItem>,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] disable_button: bool,
    #[props(default)] disable_dropdown_button: bool,
    #[props(default)] close_behavior: SplitCloseBehavior,
    icon: Option<String>,
    #[props(default = "chevron-down-small".to_string())] split_icon: String,
    #[props(default = "More actions".to_string())] dropdown_label: String,
    onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onselect: EventHandler<String>,
    #[props(default)] children: Option<Element>,
    #[props(default)] aria_label_button: Option<String>,
    #[props(default = "Alternative actions".to_string())] menu_label: String,
    #[props(default)] class: String,
    #[props(default)] id: Option<String>,
    #[props(default)] enable_top_layer: bool,
    #[props(default = DropdownPlacement::BottomEnd)] placement: DropdownPlacement,
    #[props(default = DropdownNavigationMode::RovingTabindex)]
    navigation_mode: DropdownNavigationMode,
    #[props(default)] onbeforechange: EventHandler<DropdownChangeEvent>,
    #[props(default)] onchange: EventHandler<bool>,
) -> Element {
    let dropdown = use_dropdown();
    let menu_disabled = disabled || disable_dropdown_button;
    rsx! { div { class: "ix-split-button {class}", id,
        if label.is_empty() && icon.is_some() {
            IconButton {
                variant, disabled: disabled || disable_button,
                icon: icon.clone().unwrap_or_default(),
                label: aria_label_button.unwrap_or_else(|| icon.as_deref().unwrap_or("Action").replace('-', " ")),
                onclick,
            }
        } else {
            Button { variant, disabled: disabled || disable_button, icon,
                aria_label: aria_label_button, onclick, "{label}" }
        }
        DropdownTrigger {
            dropdown, variant, disabled: menu_disabled, icon: split_icon,
            aria_label: dropdown_label, class: "ix-icon-button ix-split-trigger",
            children: VNode::empty(),
        }
        Dropdown {
            controller: dropdown, label: menu_label, disabled: menu_disabled,
            placement, navigation_mode, close_behavior: DropdownCloseBehavior::from(close_behavior),
            enable_top_layer, onchange,
            onbeforechange: move |event: DropdownChangeEvent| {
                if !menu_disabled || event.open { onbeforechange.call(event); }
            },
            if let Some(children) = children { {children} }
            else {
                for (order, item) in items.into_iter().enumerate() {
                    DropdownItem {
                        key: "{item.value}", label: item.label.clone(), icon: item.icon.clone(),
                        aria_label: if item.label.is_empty() { item.icon.as_ref().map(|name| name.replace('-', " ")) } else { None },
                        disabled: item.disabled, order, class: "ix-split-item",
                        onitemclick: move |_| onselect.call(item.value.clone()),
                    }
                }
            }
        }
    } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        collections::BTreeMap,
        rc::Rc,
    };

    #[derive(Clone, Default)]
    struct Config {
        main_disabled: bool,
        menu_disabled: bool,
        disabled: bool,
        composed: bool,
        icon_only: bool,
        top_layer: bool,
    }
    #[derive(Default)]
    struct Fixture {
        config: RefCell<Config>,
        veto: Cell<bool>,
        primary: Cell<usize>,
        selected: RefCell<Vec<String>>,
        custom: Cell<usize>,
        changes: RefCell<Vec<bool>>,
        before: RefCell<Vec<bool>>,
        note_mounts: Cell<usize>,
    }
    impl PartialEq for Fixture {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(self, other)
        }
    }
    type FocusLog = Rc<RefCell<Vec<String>>>;
    #[component]
    fn RetainedNote(data: Rc<Fixture>) -> Element {
        let mut note = use_signal(|| String::from("Inspect Motor 001"));
        rsx! {
            input {
                "data-split-note": String::from("note"), value: note(),
                onmounted: move |_| data.note_mounts.set(data.note_mounts.get()+1),
                oninput: move |event| note.set(event.value()),
            }
        }
    }
    fn fixture(data: Rc<Fixture>) -> Element {
        let config = data.config.borrow().clone();
        let primary = data.clone();
        let selected = data.clone();
        let before = data.clone();
        let changed = data.clone();
        let custom = data.clone();
        let mut items = vec![
            SplitButtonItem::new("one", "First"),
            SplitButtonItem::new("locked", "Locked"),
            SplitButtonItem::new("last", "Last"),
        ];
        items[1].disabled = true;
        let children = config.composed.then(|| {
            rsx! {
                RetainedNote { data: data.clone() }
                DropdownItem { label: "Custom action", order: 0,
                    onitemclick: move |_| custom.custom.set(custom.custom.get()+1),
                }
            }
        });
        rsx! { SplitButton {
            id: "split-fixture", class: "fixture-class",
            label: if config.icon_only { "" } else { "Create" }, icon: "add",
            aria_label_button: "Create work order", dropdown_label: "More work orders", menu_label: "Work orders",
            items, children, disabled: config.disabled, disable_button: config.main_disabled,
            disable_dropdown_button: config.menu_disabled, enable_top_layer: config.top_layer,
            onclick: move |event: MouseEvent| { primary.primary.set(primary.primary.get()+1); event.prevent_default(); },
            onselect: move |value| selected.selected.borrow_mut().push(value),
            onbeforechange: move |event: DropdownChangeEvent| {
                before.before.borrow_mut().push(event.open);
                if before.veto.get() { event.prevent_default(); }
            },
            onchange: move |value| changed.changes.borrow_mut().push(value),
        } }
    }
    #[derive(Clone)]
    struct FocusMount {
        label: String,
        focuses: Rc<RefCell<Vec<String>>>,
    }
    impl dioxus::html::RenderedElementBacking for FocusMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            assert!(focus);
            self.focuses.borrow_mut().push(self.label.clone());
            Box::pin(async { Ok(()) })
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
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event;converter!(@data $group,event) })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RawKey>().unwrap().clone()) };
        (@data Mounted,$event:ident) => { MountedData::new($event.downcast::<FocusMount>().unwrap().clone()) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected split-button test event") };
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
    fn mount(dom: &mut VirtualDom, targets: &mut Targets, focuses: Rc<RefCell<Vec<String>>>) {
        for id in targets.mounted.clone() {
            let attrs = targets.attrs.get(&id.raw());
            let label = attrs
                .and_then(|attrs| {
                    attrs
                        .get("data-ix-dropdown-label")
                        .or_else(|| attrs.get("aria-label"))
                })
                .cloned()
                .unwrap_or_default();
            dispatch(
                dom,
                "mounted",
                id,
                Box::new(FocusMount {
                    label,
                    focuses: focuses.clone(),
                }),
            );
        }
        settle(dom, targets);
    }

    fn open(dom: &VirtualDom) -> bool {
        let html = dioxus_ssr::render(dom);
        let tag = html
            .split('<')
            .find(|tag| {
                tag.starts_with("button ")
                    && tag
                        .split('>')
                        .next()
                        .unwrap()
                        .contains("aria-label=\"More work orders\"")
            })
            .unwrap();
        tag.split('>')
            .next()
            .unwrap()
            .contains("aria-expanded=\"true\"")
    }
    fn setup(data: Rc<Fixture>) -> (VirtualDom, Targets, FocusLog) {
        let mut dom = VirtualDom::new_with_props(fixture, data);
        let mut targets = Targets::default();
        targets.collect(dom.rebuild_to_vec());
        let focuses = Rc::new(RefCell::new(Vec::new()));
        mount(&mut dom, &mut targets, focuses.clone());
        (dom, targets, focuses)
    }
    #[test]
    fn split_menu_focus_wraps_and_skips_disabled_entries() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let (mut dom, mut targets, focuses) = setup(data.clone());
        let trigger = targets.attr("aria-label", "More work orders");
        assert!(!dispatch(
            &dom,
            "keydown",
            trigger,
            Box::new(RawKey(Key::ArrowDown))
        ));
        settle(&mut dom, &mut targets);
        assert!(open(&dom));
        assert_eq!(focuses.borrow().last().map(String::as_str), Some("First"));
        for (origin, key, expected) in [
            ("First", Key::ArrowDown, "Last"),
            ("Last", Key::ArrowDown, "First"),
            ("First", Key::ArrowUp, "Last"),
            ("Last", Key::Home, "First"),
            ("First", Key::End, "Last"),
        ] {
            dispatch(
                &dom,
                "keydown",
                targets.attr("data-ix-dropdown-label", origin),
                Box::new(RawKey(key)),
            );
            settle(&mut dom, &mut targets);
            assert_eq!(focuses.borrow().last().map(String::as_str), Some(expected));
        }
        dispatch(
            &dom,
            "keydown",
            targets.attr("data-ix-dropdown-label", "Last"),
            Box::new(RawKey(Key::Enter)),
        );
        settle(&mut dom, &mut targets);
        assert_eq!(*data.selected.borrow(), ["last"]);
        assert!(!open(&dom));
        dispatch(&dom, "keydown", trigger, Box::new(RawKey(Key::ArrowUp)));
        settle(&mut dom, &mut targets);
        assert_eq!(focuses.borrow().last().map(String::as_str), Some("Last"));
        dispatch(
            &dom,
            "keydown",
            targets.attr("data-ix-dropdown-label", "Last"),
            Box::new(RawKey(Key::Escape)),
        );
        settle(&mut dom, &mut targets);
        assert!(!open(&dom));
        assert_eq!(
            focuses.borrow().last().map(String::as_str),
            Some("More work orders")
        );
        dispatch(&dom, "click", trigger, Box::new(()));
        settle(&mut dom, &mut targets);
        dispatch(
            &dom,
            "click",
            targets.attr("data-ix-dropdown-label", "Locked"),
            Box::new(()),
        );
        assert_eq!(*data.selected.borrow(), ["last"]);
        dispatch(
            &dom,
            "click",
            targets.attr("data-ix-dropdown-label", "First"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        assert_eq!(*data.selected.borrow(), ["last", "one"]);
    }
    #[test]
    fn split_changes_are_cancelable_and_disabling_alternatives_closes_them() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let (mut dom, mut targets, _) = setup(data.clone());
        let trigger = targets.attr("aria-label", "More work orders");
        data.veto.set(true);
        dispatch(&dom, "click", trigger, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(!open(&dom));
        assert!(data.changes.borrow().is_empty());
        data.veto.set(false);
        data.config.borrow_mut().main_disabled = true;
        refresh(&mut dom, &mut targets);
        dispatch(&dom, "click", trigger, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(open(&dom));
        data.veto.set(true);
        dispatch(&dom, "click", trigger, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(open(&dom));
        let before = data.before.borrow().len();
        data.config.borrow_mut().menu_disabled = true;
        refresh(&mut dom, &mut targets);
        assert!(!open(&dom));
        assert_eq!(data.before.borrow().len(), before);
        assert_eq!(*data.changes.borrow(), [true, false]);
        dispatch(&dom, "click", trigger, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(!open(&dom));
        data.config.borrow_mut().menu_disabled = false;
        data.config.borrow_mut().disabled = true;
        refresh(&mut dom, &mut targets);
        dispatch(&dom, "click", trigger, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(!open(&dom));
        for (split, shared) in [
            (SplitCloseBehavior::Both, DropdownCloseBehavior::Both),
            (SplitCloseBehavior::Inside, DropdownCloseBehavior::Inside),
            (SplitCloseBehavior::Outside, DropdownCloseBehavior::Outside),
            (SplitCloseBehavior::None, DropdownCloseBehavior::None),
        ] {
            assert_eq!(DropdownCloseBehavior::from(split), shared);
        }
    }
    #[test]
    fn split_composition_labels_native_primary_event_and_retained_children() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        data.config.borrow_mut().composed = true;
        let (mut dom, mut targets, _) = setup(data.clone());
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("ix-split-button fixture-class"));
        assert!(html.contains("ix-icon-button ix-split-trigger"));
        assert!(html.contains("data-configured-placement=\"bottom-end\""));
        assert!(!html.contains("data-ix-dropdown-label=\"First\""));
        assert!(!dispatch(
            &dom,
            "click",
            targets.attr("aria-label", "Create work order"),
            Box::new(())
        ));
        assert_eq!(data.primary.get(), 1);
        let trigger = targets.attr("aria-label", "More work orders");
        dispatch(&dom, "click", trigger, Box::new(()));
        settle(&mut dom, &mut targets);
        dispatch(
            &dom,
            "click",
            targets.attr("data-ix-dropdown-label", "Custom action"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        assert_eq!(data.custom.get(), 1);
        assert!(data.selected.borrow().is_empty());
        assert!(!open(&dom));
        dispatch(&dom, "click", trigger, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(open(&dom));
        assert_eq!(data.note_mounts.get(), 1);
        data.config.borrow_mut().icon_only = true;
        data.config.borrow_mut().top_layer = true;
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("aria-label=\"Create work order\""));
        assert!(html.contains("popover=\"manual\""));
        assert_eq!(data.note_mounts.get(), 1);
        let mut empty = VirtualDom::new(
            || rsx! { SplitButton { label:"Empty", items:vec![SplitButtonItem::new("legacy","Legacy")], children:VNode::empty() } },
        );
        empty.rebuild_in_place();
        assert!(!dioxus_ssr::render(&empty).contains("Legacy"));
    }
}
