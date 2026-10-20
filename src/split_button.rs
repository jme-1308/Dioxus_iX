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
mod tests;
