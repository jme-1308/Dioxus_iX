//! Composed tile regions and native page headings.
use crate::Icon;
use dioxus::prelude::*;

/// Default-density tile height. Content can grow beyond this minimum.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileSize {
    Small,
    #[default]
    Medium,
    Big,
}
impl TileSize {
    fn class(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Big => "big",
        }
    }
}

/// Fixed-width information tile with independent composed regions and actions.
/// `header_content` and `subheader_content` take precedence over their matching
/// text props, including explicitly empty content. A header action renders even
/// without title text. Body children keep their scopes as regions and size change.
#[component]
pub fn Tile(
    children: Element,
    #[props(default)] header: Option<String>,
    #[props(default)] header_actions: Option<Element>,
    #[props(default)] subheader: Option<String>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] size: TileSize,
    #[props(default)] header_content: Option<Element>,
    #[props(default)] subheader_content: Option<Element>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let has_header = header.is_some() || header_content.is_some() || header_actions.is_some();
    let has_subheader = subheader.is_some() || subheader_content.is_some();
    rsx! { article { id, class: "ix-tile ix-tile-{size.class()} {class}", "aria-label": aria_label,
        if has_header { header { class: "ix-tile-header",
            span { if let Some(content) = header_content { {content} } else if let Some(text) = header { "{text}" } }
            {header_actions}
        } }
        if has_subheader { div { class: "ix-tile-subheader",
            if let Some(content) = subheader_content { {content} } else if let Some(text) = subheader { "{text}" }
        } }
        div { class: "ix-tile-content", {children} }
        if let Some(footer) = footer { footer { class: "ix-tile-footer", {footer} } }
    } }
}

/// Page-title presentation, keeping the same semantic h1 in both variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContentHeaderVariant {
    #[default]
    Primary,
    Secondary,
}
impl ContentHeaderVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
        }
    }
}

/// Page title, subtitle, independently composed title content and actions.
/// The native back button reports one `onback` request per activation. `ellipsis`
/// opts into truncation; ordinary headings wrap. Native title attributes remain
/// available on the title/subtitle for compatibility with existing callers.
#[component]
pub fn ContentHeader(
    title: String,
    #[props(default)] subtitle: Option<String>,
    #[props(default)] show_back: bool,
    #[props(default)] onback: EventHandler<()>,
    #[props(default)] header_content: Option<Element>,
    #[props(default)] ellipsis: bool,
    #[props(default = VNode::empty())] children: Element,
    #[props(default)] variant: ContentHeaderVariant,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default = "Go back".to_string())] aria_label_back_button: String,
) -> Element {
    rsx! { header { id, class: "ix-content-header {class}", "data-ellipsis": ellipsis, "data-variant": variant.class(),
        if show_back { button { class: "ix-layout-icon-button", r#type: "button", "aria-label": aria_label_back_button,
            onclick: move |_| onback.call(()), Icon { name: "arrow-left" } } }
        div { class: "ix-content-header-titles",
            div { class: "ix-content-header-title-line", h1 { title: title.clone(), "{title}" } {header_content} }
            if let Some(subtitle) = subtitle { p { title: subtitle.clone(), "{subtitle}" } }
        }
        div { class: "ix-content-header-actions", {children} }
    } }
}

#[cfg(test)]
mod tests;
