//! Standalone source-host option; the options-driven Select model is separate.
//!
//! Copyright (c) 2023-2026 Siemens AG. SPDX-License-Identifier: MIT.
use crate::dropdown::{checked_content_attribute, item_host_id, use_item_host};
use dioxus::prelude::*;

/// A rendered Select item can also participate in a generic Dropdown. Selection
/// is controlled by `selected`; activation never toggles it automatically.
///
/// The outer host and inner option are distinct. Generic Dropdown keyboard
/// activation clicks the host, invoking `onclick` without an inner `onitemclick`.
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
    #[props(default)] onitemclick: EventHandler<String>,
    #[props(default)] onvaluechange: EventHandler<(String, String)>,
    #[props(default)] onlabelchange: EventHandler<(Option<String>, Option<String>)>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let binding = use_item_host("select");
    let host_id = item_host_id(&attributes, &binding.host_id);
    let mounted_host = binding.clone();
    let dropdown_key = binding.clone();
    let mounted_id = host_id.clone();
    let active = binding.active();
    let roving_tabindex = binding.tabindex();
    let checked_attribute =
        serde_json::to_string(&checked_content_attribute(&attributes)).unwrap_or_default();
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
    let attributes = attributes
        .into_iter()
        .filter(|attribute| roving_tabindex.is_none() || attribute.name != "tabindex")
        .collect::<Vec<_>>();
    rsx! {
        div {
            id: host_id,
            class: "ix-select-item",
            role: "presentation", "aria-hidden": "true", "aria-label": aria_label,
            "data-ix-item-kind": "select", "data-ix-item-key": binding.key,
            "data-ix-item-runtime": binding.runtime.owner_id(),
            "data-ix-item-focusable": "true", "data-active": active.to_string(),
            "value": value, "label": label, "disabled": disabled,
            "data-ix-reflect-checked": checked_attribute,
            tabindex: roving_tabindex,
            onmounted: move |event| mounted_host.mount(&mounted_id,event.data()),
            onclick: move |event| onclick.call(event),
            onkeydown: move |event: KeyboardEvent| { dropdown_key.navigation_key(event); },
            ..attributes,
            div {
                class: if selected { "ix-dropdown-item ix-select-item-option select-item-checked" } else { "ix-dropdown-item ix-select-item-option" },
                role: "option", "aria-hidden": "true", "checked": selected, "disabled": disabled,
                "data-active": active.to_string(),
                "data-ix-reflect-checked": serde_json::to_string(&selected.then_some("")).unwrap_or_default(),
                "data-ix-item-part": "select-option", "data-ix-descendant-kind": "dropdown", "data-ix-item-focusable": "false",
                onclick: move |_| { if !disabled { onitemclick.call(inner_value.clone()); } },
                onkeydown: move |event: KeyboardEvent| {
                    if !disabled && (event.key() == Key::Enter || event.key() == Key::Character(" ".into())) {
                        onitemclick.call(key_value.clone());
                    }
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
