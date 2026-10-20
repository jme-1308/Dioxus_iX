//! Native action cards and retained notification-card composition.
use crate::{Card, CardAccordion, CardContent, CardTitle, CardVariant, Icon};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ACTION_CARD: AtomicUsize = AtomicUsize::new(1);

/// Compact native action button. Composed `children` are presentation content;
/// put independent inputs, links and buttons in [`Card`] or [`PushCard`] instead
/// of nesting interactive controls here. `passive` disables native activation.
/// Without an explicit accessible label, a nonempty heading names the button.
#[component]
pub fn ActionCard(
    #[props(default)] heading: String,
    #[props(default)] subheading: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] variant: CardVariant,
    #[props(default)] selected: bool,
    #[props(default)] passive: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default = VNode::empty())] children: Element,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] aria_label_icon: Option<String>,
) -> Element {
    let heading_id = use_hook(|| {
        format!(
            "ix-action-card-heading-{}",
            NEXT_ACTION_CARD.fetch_add(1, Ordering::Relaxed)
        )
    });
    let has_heading = !heading.trim().is_empty();
    let aria_label = aria_label.filter(|label| !label.trim().is_empty());
    let labelledby = (aria_label.is_none() && has_heading).then(|| heading_id.clone());
    let icon_label = aria_label_icon
        .or_else(|| (!has_heading).then(|| icon.as_deref().unwrap_or_default().replace('-', " ")));
    rsx! { button { id, class: "ix-card ix-card-{variant.class()} ix-action-card {class}", r#type: "button",
        disabled: passive, "aria-pressed": selected, "data-selected": selected,
        "aria-label": aria_label, "aria-labelledby": labelledby,
        onclick: move |event| { if !passive { onclick.call(event); } },
        if let Some(icon) = icon { Icon { name: icon, label: icon_label } }
        if has_heading { strong { id: heading_id, "{heading}" } }
        if let Some(subheading) = subheading { span { class: "ix-card-subheading", "{subheading}" } }
        {children}
    } }
}

/// Notification card with independent title actions and retained details.
/// By default it remains passive for compatibility: set `passive: false` to add
/// Card's separate native selection button. Title actions and the disclosure
/// remain usable in either mode. Omitting `notification` omits the count.
#[component]
pub fn PushCard(
    #[props(default)] heading: String,
    children: Element,
    #[props(default)] subheading: Option<String>,
    #[props(default)] notification: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] variant: CardVariant,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] aria_label_icon: Option<String>,
    #[props(default)] title_actions: Option<Element>,
    #[props(default = true)] passive: bool,
    #[props(default)] selected: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] onselect: EventHandler<()>,
    #[props(default = "Details".to_string())] details_label: String,
    #[props(default)] aria_label_expand_button: Option<String>,
    #[props(default = true)] reveal_content: bool,
) -> Element {
    let aria_label = aria_label
        .filter(|label| !label.trim().is_empty())
        .unwrap_or_else(|| {
            if heading.trim().is_empty() {
                "Select notification card".into()
            } else {
                format!("Select {heading}")
            }
        });
    rsx! { Card { id, variant, passive, selected, onclick, onselect, aria_label, class: "ix-push-card {class}",
        CardTitle { actions: title_actions.unwrap_or_else(VNode::empty),
            if let Some(icon) = icon { Icon { name: icon, label: aria_label_icon } } "{heading}"
        }
        CardContent { if let Some(text) = notification { strong { class: "ix-push-notification", "{text}" } } if let Some(text) = subheading { p { class: "ix-card-subheading", "{text}" } } }
        CardAccordion { variant, expanded, default_expanded, onexpand, label: details_label,
            aria_label: aria_label_expand_button, reveal_content, {children} }
    } }
}

#[cfg(test)]
mod tests;
