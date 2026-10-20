//! Declarative preset messages composed from the existing native modal parts.
//! Copyright (c) 2023 Siemens AG. SPDX-License-Identifier: MIT.
use crate::{
    Modal, ModalContent, ModalDismissRequest, ModalFooter, ModalHandle, ModalHeader, ModalResult,
    ModalSize,
};
use dioxus::prelude::*;

/// Standard message intent. The title and message should convey its meaning;
/// the preset pictogram remains decorative unless `icon_label` is provided.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageDialogVariant {
    #[default]
    Info,
    Warning,
    Error,
    Success,
    Question,
}
impl MessageDialogVariant {
    const fn icon(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Success => "success",
            Self::Question => "question",
        }
    }
    const fn color(self) -> &'static str {
        match self {
            Self::Info => "--ix-color-info-text",
            Self::Warning => "--ix-color-warning-text",
            Self::Error => "--ix-color-danger-text",
            Self::Success => "--ix-color-success",
            Self::Question => "--ix-color-text",
        }
    }
}

/// A native modal with a preset message header and explicit Rust actions.
///
/// Create `modal` with [`crate::use_modal`] and open it with [`crate::ModalTrigger`].
/// Compose [`crate::ModalClose`] or ordinary controls in `actions`; their existing
/// [`ModalResult`] values and dismissal guards are forwarded without remapping.
/// Optional children add content after the described message. Native activation,
/// dismissal, focus and lifecycle remain owned by [`Modal`].
#[component]
pub fn MessageDialog(
    modal: ModalHandle,
    title: String,
    message: String,
    actions: Element,
    #[props(default)] variant: MessageDialogVariant,
    #[props(default)] children: Option<Element>,
    #[props(default)] icon_label: Option<String>,
    #[props(default)] size: ModalSize,
    #[props(default)] centered: bool,
    #[props(default)] hide_backdrop: bool,
    #[props(default)] close_on_backdrop_click: bool,
    #[props(default)] disable_animation: bool,
    #[props(default)] is_non_blocking: bool,
    #[props(default)] hide_close: bool,
    #[props(default = "Close modal".to_string())] close_label: String,
    #[props(default)] oncloseclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] ondismissrequest: Option<EventHandler<ModalDismissRequest>>,
    #[props(default)] onresult: Option<EventHandler<ModalResult>>,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
) -> Element {
    let description_id = format!("{}-message", modal.id());
    rsx! { Modal {
        modal, label:title.clone(), aria_describedby:description_id.clone(),
        size, centered, hide_backdrop, close_on_backdrop_click, disable_animation, is_non_blocking,
        ondismissrequest, onresult, onopenchange,
        ModalHeader {
            hide_close, icon:variant.icon(), icon_color:variant.color(), icon_label,
            close_label, oncloseclick, "{title}"
        }
        ModalContent {
            div {id:description_id,class:"ix-message-dialog-message","{message}"}
            {children}
        }
        ModalFooter {{actions}}
    }}
}
