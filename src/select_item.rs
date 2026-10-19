//! Standalone source-host option; the options-driven Select model is separate.
//!
//! Copyright (c) 2023-2026 Siemens AG. SPDX-License-Identifier: MIT.
use crate::dropdown::{ItemDescription, item_host_id, use_item_host};
use dioxus::prelude::*;

/// A rendered Select item can also participate in a generic Dropdown. Selection
/// is controlled by `selected`; activation never toggles it automatically.
///
/// The outer host and inner option are distinct. Enter/Space activation invokes
/// the semantic `onitemclick` notification with the current Rust value; generic
/// Dropdown activation uses the same action and owns its dismissal.
/// Pointer activation of the inner option invokes `onitemclick` with its Rust
/// value, then bubbles the original click to `onclick`. The value is a typed
/// adapter for the source's bubbling inner-element payload.
#[component]
pub fn SelectItem(
    value: String,
    #[props(default)] label: Option<String>,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] onfocus: EventHandler<FocusEvent>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
    #[props(default)] onitemclick: EventHandler<String>,
    #[props(default)] onvaluechange: EventHandler<(String, String)>,
    #[props(default)] onlabelchange: EventHandler<(Option<String>, Option<String>)>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let binding = use_item_host("select");
    let host_id = item_host_id(&attributes, &binding.host_id);
    let mounted_host = binding.clone();
    let dropdown_key = binding.clone();
    let option_key = binding.clone();
    let generic = binding.generic;
    let focus_host = binding.clone();
    let blur_host = binding.clone();
    let focus_option = binding.clone();
    let blur_option = binding.clone();
    let mounted_id = host_id.clone();
    let active = binding.active();
    let option_active = binding.runtime.part_active(
        &binding.key,
        crate::dropdown_inventory::HostPartKind::SelectOption,
    );
    let roving_tabindex = binding.tabindex();
    binding.describe(
        &host_id,
        &attributes,
        ItemDescription {
            disabled,
            parts: vec![crate::dropdown_inventory::HostPartFacts {
                kind: crate::dropdown_inventory::HostPartKind::SelectOption,
                capture: format!("{}-option", binding.key),
                source_focusable: false,
            }],
            ..Default::default()
        },
    );
    let aria_label = attributes
        .iter()
        .rev()
        .find_map(|attribute| {
            if attribute.name == "aria-label" {
                if let dioxus::core::AttributeValue::Text(value) = &attribute.value {
                    return Some(value.clone());
                }
            }
            None
        })
        .or_else(|| label.clone());
    let text = label
        .as_ref()
        .filter(|label| !label.is_empty())
        .unwrap_or(&value)
        .clone();
    let previous =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new((value.clone(), label.clone()))));
    use_effect(use_reactive((&value, &label), move |(value, label)| {
        let (old_value, old_label) = previous.replace((value.clone(), label.clone()));
        if old_value != value {
            onvaluechange.call((value, old_value));
        }
        if old_label != label {
            onlabelchange.call((label, old_label));
        }
    }));
    let inner_value = value.clone();
    let key_value = value.clone();
    let keyboard_action = use_callback(move |event: KeyboardEvent| {
        if !event.default_action_enabled()
            || disabled
            || !(event.key() == Key::Enter || event.key() == Key::Character(" ".into()))
        {
            return;
        }
        onitemclick.call(key_value.clone());
    });
    binding.set_keyboard_action(&host_id, keyboard_action);
    let standalone_action = use_callback(move |event: KeyboardEvent| {
        if !event.default_action_enabled()
            || disabled
            || !(event.key() == Key::Enter || event.key() == Key::Character(" ".into()))
        {
            return;
        }
        keyboard_action.call(event.clone());
        event.prevent_default();
    });
    let mut attributes = attributes
        .into_iter()
        .filter(|attribute| roving_tabindex.is_none() || attribute.name != "tabindex")
        .collect::<Vec<_>>();
    let class = crate::dropdown::item_host_class(
        &mut attributes,
        if active {
            "ix-select-item ix-focused"
        } else {
            "ix-select-item"
        },
    );
    rsx! {
        div {
            id: host_id,
            class,
            role: "presentation", "aria-hidden": "true", "aria-label": aria_label,
            "data-ix-item-kind": "select", "data-ix-item-key": binding.key,
            "data-ix-item-runtime": binding.runtime.owner_id(),
            "data-ix-item-focusable": "true", "data-active": active.to_string(),
            "value": value, "label": label, "disabled": disabled,
            tabindex: roving_tabindex,
            onmounted: move |event| mounted_host.mount(&mounted_id,event.data()),
            onfocus: move |event| { focus_host.focus(true,None); onfocus.call(event); },
            onblur: move |event| { blur_host.blur(); onblur.call(event); },
            onclick: move |event| onclick.call(event),
            onkeydown: move |event: KeyboardEvent| {
                if !dropdown_key.navigation_key(event.clone()) && !generic { standalone_action.call(event); }
            },
            ..attributes,
            div {
                class: format!("ix-dropdown-item ix-select-item-option{}{}",
                    if selected { " select-item-checked" } else { "" },
                    if option_active { " ix-focused" } else { "" }),
                role: "option", "aria-hidden": "true", "checked": selected, "disabled": disabled,
                "data-active": option_active.to_string(),
                "data-ix-item-part": "select-option", "data-ix-descendant-kind": "dropdown", "data-ix-item-focusable": "false",
                onfocus: move |_| focus_option.focus(true,Some(crate::dropdown_inventory::HostPartKind::SelectOption)),
                onblur: move |_| blur_option.blur(),
                onclick: move |_| { if !disabled { onitemclick.call(inner_value.clone()); } },
                onkeydown: move |event: KeyboardEvent| {
                    if !event.default_action_enabled() { return; }
                    if option_key.navigation_key(event.clone()) {
                        event.stop_propagation();
                        return;
                    }
                    if !generic { standalone_action.call(event); }
                },
                span { "{text}" }
                if selected { crate::Icon { name: "single-check", size: 16 } }
            }
            crate::overlay_runtime::OverlayTransport { runtime: binding.runtime.clone() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    struct TestKey(Key);
    impl dioxus::html::ModifiersInteraction for TestKey {
        fn modifiers(&self) -> Modifiers {
            Modifiers::empty()
        }
    }
    impl dioxus::html::HasKeyboardData for TestKey {
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
    struct Converter;
    macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*
            }
        };
        (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new(TestKey($event.downcast::<Key>().unwrap().clone()))};
        (@data $other:ident,$event:ident)=>{panic!("unexpected Select item test event")};
    }
    dioxus::html::with_html_event_groups!(converter);

    #[test]
    fn host_and_inner_keyboard_activation_emit_one_semantic_value_without_mouse_events() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        type Capture = (Rc<RefCell<Vec<String>>>, Rc<RefCell<usize>>);
        fn app(capture: Capture) -> Element {
            let host_click = capture.1.clone();
            let enabled = capture.0.clone();
            let disabled = capture.0.clone();
            rsx! {
                SelectItem {id:"enabled",value:"value",onclick:move |_|*host_click.borrow_mut()+=1,
                    onitemclick:move |value|enabled.borrow_mut().push(value)}
                SelectItem {id:"disabled",value:"disabled",disabled:true,
                    onitemclick:move |value|disabled.borrow_mut().push(value)}
            }
        }
        let values = Rc::new(RefCell::new(Vec::new()));
        let mouse = Rc::new(RefCell::new(0));
        let mut dom = VirtualDom::new_with_props(app, (values.clone(), mouse.clone()));
        let mut current = dioxus::core::ElementId::ROOT;
        let mut host = String::new();
        let mut targets = std::collections::BTreeMap::new();
        for edit in dom.rebuild_to_vec().edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "id" && (value == "enabled" || value == "disabled") => {
                    host = value.clone();
                    targets.insert(value, current);
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "class" && value.contains("ix-select-item-option") => {
                    targets.insert(format!("{host}-option"), current);
                }
                _ => {}
            }
        }
        let key = |target: &str, key: Key, prevented: bool| {
            let data: Rc<dyn std::any::Any> =
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(key)));
            let event = Event::new(data, true);
            if prevented {
                event.prevent_default();
            }
            dom.runtime()
                .handle_event("keydown", event.clone(), targets[target]);
            !event.default_action_enabled()
        };
        assert!(key("enabled", Key::Enter, false));
        assert!(key("enabled-option", Key::Enter, false));
        assert!(key("enabled-option", Key::Character(" ".into()), false));
        assert_eq!(
            *values.borrow(),
            ["value", "value", "value"],
            "inner bubbling cannot invoke the host action twice"
        );
        assert_eq!(
            *mouse.borrow(),
            0,
            "keyboard does not fabricate a MouseEvent"
        );
        assert!(!key("disabled-option", Key::Enter, false));
        assert!(key("enabled-option", Key::Enter, true));
        assert_eq!(
            values.borrow().len(),
            3,
            "disabled and prevented events have no semantic action"
        );
    }

    #[test]
    fn standalone_select_host_and_inner_option_keep_distinct_source_roles() {
        fn app() -> Element {
            rsx! {
                SelectItem { value: "value-only", label: "", selected: true, tabindex: "5", "data-consumer": "kept" }
                SelectItem { value: "disabled-value", label: "Disabled label", disabled: true }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("data-ix-item-kind=\"select\"").count(), 2);
        assert!(
            !html.contains("data-ix-item-kind=\"dropdown\""),
            "generated inner options are not outer light-DOM DropdownItems"
        );
        assert!(
            html.split('>')
                .filter(|tag| tag.contains("data-ix-item-kind=\"select\""))
                .all(|tag| tag.contains("role=\"presentation\""))
        );
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        assert!(
            html.contains("value-only</span>"),
            "empty labels fall back to the submitted value"
        );
        assert!(html.contains("tabindex=\"5\"") && html.contains("data-consumer=\"kept\""));
        assert!(html.contains("select-item-checked"));
        assert!(
            !html.contains("aria-selected=\"true\""),
            "selected and source visible focus are different"
        );
    }

    type WatchState = Rc<RefCell<Option<Signal<(String, Option<String>)>>>>;

    #[derive(Clone, PartialEq, Props)]
    struct WatchProps {
        state: WatchState,
        changes: Rc<RefCell<Vec<String>>>,
    }
    fn watch_root(props: WatchProps) -> Element {
        let state = use_signal(|| ("first".to_string(), None::<String>));
        *props.state.borrow_mut() = Some(state);
        let values = props.changes.clone();
        let labels = props.changes.clone();
        rsx! { SelectItem { value: state().0, label: state().1,
        onvaluechange: move |(new,old)| values.borrow_mut().push(format!("value:{old}->{new}")),
        onlabelchange: move |(new,old)| labels.borrow_mut().push(format!("label:{old:?}->{new:?}")) } }
    }
    #[test]
    fn select_item_prop_watchers_skip_initial_render_and_keep_old_new_values() {
        let state = Rc::new(RefCell::new(None));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let mut dom = VirtualDom::new_with_props(
            watch_root,
            WatchProps {
                state: state.clone(),
                changes: changes.clone(),
            },
        );
        dom.rebuild_in_place();
        for _ in 0..3 {
            dom.render_immediate_to_vec();
        }
        assert!(changes.borrow().is_empty());
        dom.in_runtime(|| {
            state
                .borrow()
                .unwrap()
                .set(("second".into(), Some("Label".into())))
        });
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        assert_eq!(
            *changes.borrow(),
            ["value:first->second", "label:None->Some(\"Label\")"]
        );
    }
}
