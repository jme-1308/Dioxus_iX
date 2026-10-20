//! Native Rust action, status, and typography primitives.
//!
//! Behavioral reference: Siemens iX (MIT). No upstream JavaScript runtime is used.
use dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};

macro_rules! action_enum {
    ($name:ident, $default:ident, $($variant:ident => $value:literal),+ $(,)?) => {
        #[doc = concat!("Supported ", stringify!($name), " values.")]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name { $(#[doc = $value] $variant),+ }
        impl Default for $name { fn default() -> Self { Self::$default } }
        impl $name { #[doc = "The corresponding upstream token."] pub const fn as_str(self) -> &'static str { match self { $(Self::$variant => $value),+ } } }
    };
}

action_enum!(ButtonVariant, Primary,
    Primary => "primary", Secondary => "secondary", Tertiary => "tertiary",
    SubtlePrimary => "subtle-primary", SubtleSecondary => "subtle-secondary", SubtleTertiary => "subtle-tertiary",
    DangerPrimary => "danger-primary", DangerSecondary => "danger-secondary", DangerTertiary => "danger-tertiary");
action_enum!(ToggleButtonVariant, SubtlePrimary,
    Primary => "primary", Secondary => "secondary", Tertiary => "tertiary",
    SubtlePrimary => "subtle-primary", SubtleSecondary => "subtle-secondary", SubtleTertiary => "subtle-tertiary");
impl TryFrom<ButtonVariant> for ToggleButtonVariant {
    type Error = ButtonVariant;

    fn try_from(value: ButtonVariant) -> Result<Self, Self::Error> {
        match value {
            ButtonVariant::Primary => Ok(Self::Primary),
            ButtonVariant::Secondary => Ok(Self::Secondary),
            ButtonVariant::Tertiary => Ok(Self::Tertiary),
            ButtonVariant::SubtlePrimary => Ok(Self::SubtlePrimary),
            ButtonVariant::SubtleSecondary => Ok(Self::SubtleSecondary),
            ButtonVariant::SubtleTertiary => Ok(Self::SubtleTertiary),
            other => Err(other),
        }
    }
}
action_enum!(ButtonType, Button, Button => "button", Submit => "submit", Reset => "reset");
action_enum!(ButtonAlignment, Center, Center => "center", Start => "start");
action_enum!(ButtonIconSize, Twenty, Twelve => "12", Sixteen => "16", Twenty => "20", TwentyFour => "24");
impl ButtonIconSize {
    /// Glyph size in pixels.
    pub const fn pixels(self) -> u32 {
        match self {
            Self::Twelve => 12,
            Self::Sixteen => 16,
            Self::Twenty => 20,
            Self::TwentyFour => 24,
        }
    }
}
action_enum!(SpinnerSize, Md, Xxs => "xxs", Xs => "xs", Sm => "sm", Md => "md", Xxl => "xxl");
impl SpinnerSize {
    /// Nominal normal-density diameter in pixels; theme tokens determine rendered size.
    pub const fn pixels(self) -> u32 {
        match self {
            Self::Xxs => 12,
            Self::Xs => 16,
            Self::Sm => 20,
            Self::Md => 32,
            Self::Xxl => 96,
        }
    }
}
action_enum!(SpinnerVariant, Secondary, Primary => "primary", Secondary => "secondary");
action_enum!(StatusVariant, Primary, Primary => "primary", Alarm => "alarm", Critical => "critical", Warning => "warning", Info => "info", Neutral => "neutral", Success => "success", Custom => "custom", Error => "error");
impl StatusVariant {
    fn status_icon(self, outline: bool) -> &'static str {
        match self {
            Self::Alarm => {
                if outline {
                    "alarm"
                } else {
                    "alarm-filled"
                }
            }
            Self::Error => {
                if outline {
                    "error"
                } else {
                    "error-filled"
                }
            }
            Self::Critical => {
                if outline {
                    "warning-rhomb"
                } else {
                    "warning-rhomb-filled"
                }
            }
            Self::Warning => {
                if outline {
                    "warning"
                } else {
                    "warning-filled"
                }
            }
            Self::Success => {
                if outline {
                    "success"
                } else {
                    "success-filled"
                }
            }
            _ => {
                if outline {
                    "info"
                } else {
                    "info-filled"
                }
            }
        }
    }
    fn status_plate(self) -> &'static str {
        match self {
            Self::Critical => "rhomb-filled",
            Self::Warning => "triangle-filled",
            _ => "circle-filled",
        }
    }
}

/// A native button or styled anchor. Loading prevents activation just like disabled.
///
/// Regular buttons consume the source `--ix-button-*` sizing and state variables,
/// with `--si-sys-*` semantic tokens supplied by an enclosing [`crate::IxTheme`].
/// The `hover` and `active` classes expose the source visual-state aliases;
/// `active` locks the pressed appearance without introducing a toggle state.
/// Missing Brand values retain provisional public-source fallbacks.
#[component]
pub fn Button(
    children: Element,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default)] button_type: ButtonType,
    #[props(default)] alignment: ButtonAlignment,
    #[props(default)] icon_size: ButtonIconSize,
    #[props(default)] id: Option<String>,
    icon: Option<String>,
    icon_right: Option<String>,
    href: Option<String>,
    #[props(default = "_self".to_string())] target: String,
    rel: Option<String>,
    form: Option<String>,
    aria_label: Option<String>,
    aria_describedby: Option<String>,
    #[props(default)] class: String,
    #[props(default)] onmounted: Option<EventHandler<MountedEvent>>,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let binding = crate::dropdown::use_item_host("native-opt-in");
    let actual_id = id
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| binding.host_id.clone());
    let is_anchor = href.is_some();
    let mounted = use_hook(|| Rc::new(RefCell::new(None::<(bool, Rc<MountedData>)>)));
    let previous_mount = mounted.borrow().clone();
    if let Some((mounted_anchor, node)) = previous_mount {
        if mounted_anchor == is_anchor {
            binding.mount(&actual_id, node);
        } else {
            *mounted.borrow_mut() = None;
        }
    }
    let blocked = disabled || loading;
    let class = format!(
        "ix-button ix-button-standard ix-button-tokenized ix-button-{} ix-align-{} {}",
        variant.as_str(),
        alignment.as_str(),
        class
    );
    let loading_size = match (
        icon.as_deref().is_some_and(|name| !name.is_empty()),
        icon_size,
    ) {
        (true, ButtonIconSize::Twelve) => SpinnerSize::Xxs,
        (true, ButtonIconSize::Sixteen) => SpinnerSize::Xs,
        _ => SpinnerSize::Sm,
    };
    let content = rsx! {
        if loading { span { class: "ix-button-loading", Spinner { size: loading_size, hide_track: true } } }
        else if let Some(name) = icon.filter(|name| !name.is_empty()) { span { class: "ix-button-icon-start", crate::Icon { name, size: icon_size.pixels() } } }
        span { class: "ix-button-content", {children} }
        if let Some(name) = icon_right.filter(|name| !name.is_empty()) { span { class: "ix-button-icon-end", crate::Icon { name, size: icon_size.pixels() } } }
    };
    let mount_id = actual_id.clone();
    let mount_binding = binding.clone();
    let mount_handle = mounted;
    if let Some(href) = href {
        rsx! { a {
            id: actual_id,
            class, role: "button", href: if blocked { None } else { Some(href) },
            target, rel: rel.unwrap_or_else(|| "noopener noreferrer".into()),
            aria_label, aria_describedby, aria_disabled: blocked, aria_busy: loading,
            tabindex: if blocked { -1 } else { 0 },
            onmounted: move |event| {
                let node = event.data();
                *mount_handle.borrow_mut() = Some((true, node.clone()));
                mount_binding.mount(&mount_id, node);
                if let Some(handler) = onmounted {
                    handler.call(event);
                }
            },
            onclick: move |event| { if blocked { event.prevent_default(); event.stop_propagation(); } else if let Some(handler) = onclick { handler.call(event); } },
            {content}
        } }
    } else {
        rsx! { button {
            id: actual_id,
            class, r#type: button_type.as_str(), form, disabled: blocked,
            tabindex: if blocked { -1 } else { 0 },
            aria_label, aria_describedby, aria_disabled: blocked, aria_busy: loading,
            onmounted: move |event| {
                let node = event.data();
                *mount_handle.borrow_mut() = Some((false, node.clone()));
                mount_binding.mount(&mount_id, node);
                if let Some(handler) = onmounted {
                    handler.call(event);
                }
            },
            onclick: move |event| { if !blocked { if let Some(handler) = onclick { handler.call(event); } } },
            {content}
        } }
    }
}

/// An icon-only action with an explicitly required accessible name.
///
/// Size, glyph, oval radius and state styles consume the source semantic tokens.
/// `icon_color` names a CSS custom property to use for the glyph; the enclosing
/// theme or a component rule can also supply `--ix-icon-button-color` directly.
/// Loading replaces the icon and prevents activation like disabled. Native form
/// controls and the required explicit name remain Rust accessibility adapters.
#[component]
pub fn IconButton(
    icon: String,
    label: String,
    #[props(default)] id: Option<String>,
    icon_color: Option<String>,
    #[props(default = ButtonVariant::SubtlePrimary)] variant: ButtonVariant,
    #[props(default)] size: ButtonIconSize,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default)] oval: bool,
    #[props(default)] button_type: ButtonType,
    #[props(default)] class: String,
    aria_pressed: Option<bool>,
    aria_expanded: Option<bool>,
    tabindex: Option<i32>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmousedown: Option<EventHandler<MouseEvent>>,
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    onmounted: Option<EventHandler<MountedEvent>>,
    aria_hidden: Option<bool>,
) -> Element {
    let binding = crate::dropdown::use_item_host("native-opt-in");
    let actual_id = id
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| binding.host_id.clone());
    let mounted = use_hook(|| Rc::new(RefCell::new(None::<Rc<MountedData>>)));
    if let Some(node) = mounted.borrow().clone() {
        binding.mount(&actual_id, node);
    }
    let blocked = disabled || loading;
    let loading_size = match (!icon.is_empty(), size) {
        (true, ButtonIconSize::Twelve) => SpinnerSize::Xxs,
        (true, ButtonIconSize::Sixteen) => SpinnerSize::Xs,
        _ => SpinnerSize::Sm,
    };
    let mut native_events = Vec::new();
    if let Some(handler) = onmousedown {
        native_events.push(Attribute::new(
            "onmousedown",
            dioxus::html::EventHandlerValue::into_platform_listener(handler),
            None,
            false,
        ));
    }
    if let Some(handler) = onkeydown {
        native_events.push(Attribute::new(
            "onkeydown",
            dioxus::html::EventHandlerValue::into_platform_listener(handler),
            None,
            false,
        ));
    }
    let mount_id = actual_id.clone();
    let mount_binding = binding.clone();
    let mount_handle = mounted;
    rsx! { button {
        id: actual_id,
        class: "ix-button ix-icon-button ix-button-tokenized ix-icon-button-native ix-button-{variant.as_str()} ix-icon-size-{size.as_str()} {class}",
        class: if oval { "ix-oval" }, r#type: button_type.as_str(),
        style: icon_color.map(|name| format!("--ix-icon-button-color:var({name})")),
        disabled: blocked, tabindex: if blocked { -1 } else { tabindex.unwrap_or(0) },
        aria_label: label.clone(), title: label, aria_busy: loading, aria_disabled: blocked, aria_pressed, aria_expanded, aria_hidden,
        onmounted: move |event| {
            let node = event.data();
            *mount_handle.borrow_mut() = Some(node.clone());
            mount_binding.mount(&mount_id, node);
            if let Some(handler) = onmounted {
                handler.call(event);
            }
        },
        onclick: move |event| { if !blocked { if let Some(handler) = onclick { handler.call(event); } } },
        ..native_events,
        if loading { Spinner { size: loading_size, hide_track: true } }
        else if !icon.is_empty() { crate::Icon { name: icon, size: size.pixels(), class: "ix-icon-button-glyph" } }
    } }
}

/// A text link with disabled behavior and safe external-link defaults.
#[component]
pub fn LinkButton(
    children: Element,
    url: String,
    #[props(default)] disabled: bool,
    #[props(default = "_self".to_string())] target: String,
    #[props(default)] class: String,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! { a {
        class: "ix-link-button {class}", href: if disabled { None } else { Some(url) },
        target, rel: "noopener noreferrer", aria_disabled: disabled,
        tabindex: if disabled { -1 } else { 0 },
        onclick: move |event| { if disabled { event.prevent_default(); event.stop_propagation(); } else if let Some(handler) = onclick { handler.call(event); } },
        {children}
    } }
}

/// Controlled toggle button; update `pressed` from `onpressedchange`.
///
/// Supports the six current source variants and semantic selected-state tokens.
/// A click requests the inverse of the supplied `pressed` value; it never changes
/// that value internally. Disabled and loading controls do not emit requests.
/// `active` locks the source host active appearance; `ix-toggle-active` exposes
/// the source inner active diagnostic alias independently of the selected state.
#[component]
pub fn ToggleButton(
    children: Element,
    #[props(default)] pressed: bool,
    #[props(default)] variant: ToggleButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    icon: Option<String>,
    icon_right: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
    onpressedchange: EventHandler<bool>,
) -> Element {
    let blocked = disabled || loading;
    rsx! { button {
        class: "ix-button ix-button-tokenized ix-button-selectable ix-toggle-button-native ix-button-{variant.as_str()} {class}",
        class: if pressed { "ix-selected" },
        r#type: "button", disabled: blocked, tabindex: if blocked { -1 } else { 0 },
        aria_pressed: pressed, aria_disabled: blocked, aria_busy: loading, aria_label,
        onclick: move |_| { if !blocked { onpressedchange.call(!pressed); } },
        if loading { span { class: "ix-button-loading", Spinner { size: SpinnerSize::Sm, hide_track: true } } }
        else if let Some(name) = icon.filter(|name| !name.is_empty()) {
            span { class: "ix-button-icon-start ix-toggle-button-icon", crate::Icon { name, size: 20, class: "ix-toggle-button-glyph" } }
        }
        span { class: "ix-button-content", {children} }
        if let Some(name) = icon_right.filter(|name| !name.is_empty()) {
            span { class: "ix-button-icon-end ix-toggle-button-icon", crate::Icon { name, size: 20, class: "ix-toggle-button-glyph" } }
        }
    } }
}

/// Controlled icon toggle; update `pressed` from `onpressedchange`.
///
/// Shares source control/glyph sizing and oval tokens with [`IconButton`], and
/// selected-state recipes with [`ToggleButton`]. The explicit accessible label
/// stays constant as `pressed` changes. Loading replaces the glyph and blocks
/// requests. `active` locks the source host active look; `ix-toggle-active`
/// exposes its separate inner active diagnostic alias.
#[component]
pub fn IconToggleButton(
    icon: String,
    label: String,
    #[props(default)] pressed: bool,
    #[props(default = ButtonVariant::SubtlePrimary)] variant: ButtonVariant,
    #[props(default)] size: ButtonIconSize,
    #[props(default)] oval: bool,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default)] class: String,
    onpressedchange: EventHandler<bool>,
) -> Element {
    let blocked = disabled || loading;
    let loading_size = match (!icon.is_empty(), size) {
        (true, ButtonIconSize::Twelve) => SpinnerSize::Xxs,
        (true, ButtonIconSize::Sixteen) => SpinnerSize::Xs,
        _ => SpinnerSize::Sm,
    };
    rsx! { button {
        class: "ix-button ix-icon-button ix-button-tokenized ix-icon-button-native ix-icon-toggle-button-native ix-button-selectable ix-button-{variant.as_str()} ix-icon-size-{size.as_str()} {class}",
        class: if pressed { "ix-selected" }, class: if oval { "ix-oval" },
        r#type: "button", disabled: blocked, tabindex: if blocked { -1 } else { 0 },
        aria_disabled: blocked, aria_pressed: pressed, aria_label: label.clone(),
        title: label, aria_busy: loading,
        onclick: move |_| { if !blocked { onpressedchange.call(!pressed); } },
        if loading { Spinner { size: loading_size, hide_track: true } }
        else if !icon.is_empty() { crate::Icon { name: icon, size: size.pixels(), class: "ix-icon-button-glyph" } }
    } }
}

pub use crate::split_button::{SplitButton, SplitButtonItem, SplitButtonProps, SplitCloseBehavior};

/// Animated busy state using iX rotation and clip-mask recipes.
/// Timing tokens can be overridden by the host's motion preference stylesheet.
#[component]
pub fn Spinner(
    #[props(default)] size: SpinnerSize,
    #[props(default)] variant: SpinnerVariant,
    #[props(default)] hide_track: bool,
    #[props(default = "Loading".to_string())] label: String,
    #[props(default)] class: String,
) -> Element {
    rsx! { span { class: "ix-spinner ix-spinner-{variant.as_str()} ix-spinner-{size.as_str()} {class}",
        class: if hide_track { "ix-spinner-no-track" },
        style: "--ix-spinner-size:{size.pixels()}px", role: "status", aria_busy: true, aria_label: label,
        div { class: "ix-spinner-ring {size.as_str()}",
            class: if variant == SpinnerVariant::Primary { "primary" },
            class: if hide_track { "hide-track" }, "aria-hidden": "true",
        }
    } }
}

/// Read-only semantic status label.
#[component]
pub fn Pill(
    children: Element,
    #[props(default)] variant: StatusVariant,
    #[props(default)] outline: bool,
    #[props(default)] align_left: bool,
    icon: Option<String>,
    aria_label_icon: Option<String>,
    background: Option<String>,
    color: Option<String>,
    tooltip: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let custom = custom_status_style(variant, background.as_deref(), color.as_deref());
    rsx! { span {
        class: "ix-pill ix-status-{variant.as_str()} {class}", class: if outline { "ix-outline" },
        class: if align_left { "ix-align-start" }, style: custom, title: tooltip,
        role: aria_label.as_ref().map(|_| "img"), aria_label,
        if let Some(name) = icon { crate::Icon { name, size: 16, label: aria_label_icon } }
        span { class: "ix-status-label", {children} }
    } }
}

/// Interactive tag with an independently operable close button.
/// `inactive` disables only the main chip; the close button remains available.
#[component]
pub fn Chip(
    children: Element,
    #[props(default)] variant: StatusVariant,
    #[props(default)] inactive: bool,
    #[props(default)] closable: bool,
    #[props(default)] outline: bool,
    #[props(default)] center_content: bool,
    icon: Option<String>,
    aria_label_icon: Option<String>,
    background: Option<String>,
    color: Option<String>,
    tooltip: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
    #[props(default = "Close chip".to_string())] close_label: String,
    onclick: Option<EventHandler<MouseEvent>>,
    onclose: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! { span {
        class: "ix-chip ix-status-{variant.as_str()} {class}", class: if outline { "ix-outline" },
        style: custom_status_style(variant, background.as_deref(), color.as_deref()), title: tooltip,
        button { class: "ix-chip-main", class: if center_content { "ix-center-content" },
            r#type: "button", disabled: inactive, aria_label,
            onclick: move |event| { if !inactive { if let Some(handler) = onclick { handler.call(event); } } },
            if let Some(name) = icon { crate::Icon { name, size: 20, label: aria_label_icon } }
            span { class: "ix-status-label", {children} }
        }
        if closable { button {
            class: "ix-chip-close", r#type: "button", aria_label: close_label,
            onclick: move |event| { event.stop_propagation(); if let Some(handler) = onclose { handler.call(event); } },
            crate::Icon { name: "close-small".to_string(), size: 16 }
        } }
    } }
}

/// Applied filter with a dismiss action. Read-only and disabled filters hide it.
#[component]
pub fn FilterChip(
    children: Element,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] hide_close_button: bool,
    #[props(default = "Remove filter".to_string())] close_label: String,
    onclose: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! { span {
        class: "ix-filter-chip", class: if disabled { "ix-disabled" }, aria_disabled: disabled,
        span { class: "ix-status-label", {children} }
        if !disabled && !readonly && !hide_close_button { button {
            class: "ix-filter-close", r#type: "button", aria_label: close_label,
            onclick: move |event| { event.stop_propagation(); event.prevent_default(); if let Some(handler) = onclose { handler.call(event); } },
            crate::Icon { name: "close-small".to_string(), size: 16 }
        } }
    } }
}

fn custom_status_style(
    variant: StatusVariant,
    background: Option<&str>,
    color: Option<&str>,
) -> String {
    if variant == StatusVariant::Custom {
        format!(
            "--ix-status-bg:{};--ix-status-fg:{}",
            background.unwrap_or("var(--ix-color-surface-2)"),
            color.unwrap_or("var(--ix-color-text)")
        )
    } else {
        String::new()
    }
}

action_enum!(BadgeType, Counter, Counter => "counter", Label => "label", Dot => "dot", StatusIcon => "status-icon");
action_enum!(BadgePosition, TopAfter, TopAfter => "top-after", BottomAfter => "bottom-after");

/// Format an iX counter: decimal values truncate and counts over 99 become `99+`.
/// Invalid and empty numeric strings do not display an indicator.
pub fn format_badge_counter(label: &str) -> Option<String> {
    let label = label.trim();
    if label == "99+" {
        return Some(label.to_string());
    }
    let digits = label.strip_prefix('-').unwrap_or(label);
    let mut parts = digits.split('.');
    let whole = parts.next()?;
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if let Some(fraction) = parts.next() {
        if fraction.is_empty()
            || !fraction.bytes().all(|byte| byte.is_ascii_digit())
            || parts.next().is_some()
        {
            return None;
        }
    }
    let value = label.parse::<f64>().ok()?.trunc();
    if value > 99.0 {
        Some("99+".into())
    } else {
        Some(format!("{}", if value == 0.0 { 0.0 } else { value }))
    }
}

/// Standalone or attached counter, dot, label, or status indicator.
/// For attached badges give the anchor an accessible description in its own props.
#[component]
pub fn Badge(
    #[props(default)] badge_type: BadgeType,
    #[props(default)] variant: StatusVariant,
    #[props(default)] outline: bool,
    #[props(default)] border: bool,
    #[props(default)] position: BadgePosition,
    #[props(default)] offset_x: i32,
    #[props(default)] offset_y: i32,
    #[props(default)] enable_animation: bool,
    #[props(default)] align_left: bool,
    label: Option<String>,
    aria_label: Option<String>,
    icon: Option<String>,
    background: Option<String>,
    color: Option<String>,
    tooltip: Option<String>,
    role: Option<String>,
    aria_label_icon: Option<String>,
    #[props(default)] class: String,
    children: Option<Element>,
) -> Element {
    let variant = if badge_type == BadgeType::StatusIcon {
        match variant {
            StatusVariant::Primary | StatusVariant::Neutral | StatusVariant::Custom => {
                StatusVariant::Info
            }
            _ => variant,
        }
    } else if variant == StatusVariant::Error {
        StatusVariant::Alarm
    } else {
        variant
    };
    let attached = children.is_some();
    let text = match badge_type {
        BadgeType::Counter => label.as_deref().and_then(format_badge_counter),
        BadgeType::Label => label
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string),
        _ => None,
    };
    let show = text.is_some() || matches!(badge_type, BadgeType::Dot | BadgeType::StatusIcon);
    let offset = if badge_type == BadgeType::Dot { 6 } else { 10 };
    let custom = custom_status_style(variant, background.as_deref(), color.as_deref());
    rsx! { span {
        class: "ix-badge ix-badge-{badge_type.as_str()} ix-status-{variant.as_str()} ix-badge-{position.as_str()} {class}",
        class: if attached { "ix-badge-attached" }, class: if outline { "ix-outline" },
        class: if border { "ix-badge-border" }, class: if enable_animation { "ix-badge-animated" },
        class: if align_left { "ix-align-start" },
        style: "{custom};--ix-badge-x:{offset_x - offset}px;--ix-badge-y:{offset_y - offset}px",
        title: tooltip,
        if let Some(children) = children { {children} }
        if show { span {
            class: "ix-badge-indicator", role: if attached { None } else { Some(role.unwrap_or_else(|| "status".into())) },
            aria_label: if attached { None } else { aria_label }, aria_hidden: attached,
            if badge_type == BadgeType::StatusIcon {
                if outline { crate::Icon { name: variant.status_icon(true).to_string(), size: 20 } }
                else { span { class: "ix-badge-status-stack",
                    span { class: "ix-badge-status-plate", crate::Icon { name: variant.status_plate().to_string(), size: 20 } }
                    span { class: "ix-badge-status-glyph", crate::Icon { name: variant.status_icon(false).to_string(), size: 20 } }
                } }
            }
            else {
                if badge_type == BadgeType::Label { if let Some(name) = icon { crate::Icon { name, size: 16, label: aria_label_icon } } }
                if let Some(text) = text { span { class: "ix-badge-label-text", "{text}" } }
            }
        } }
    } }
}

/// Profile image, initials, or a generic user icon. Failed images fall back.
#[component]
pub fn Avatar(
    username: Option<String>,
    initials: Option<String>,
    image: Option<String>,
    #[props(default = 32)] size: u32,
    tooltip: Option<String>,
) -> Element {
    let mut failed = use_signal(|| false);
    let label = username.clone().unwrap_or_else(|| "User profile".into());
    let title = tooltip.or(username);
    rsx! { span { class: "ix-avatar", style: "--ix-avatar-size:{size}px", role: "img", aria_label: label.clone(), title,
        if let Some(image) = image.filter(|_| !failed()) { img { src: image, alt: "", onerror: move |_| failed.set(true) } }
        else if let Some(initials) = initials { span { aria_hidden: true, "{initials}" } }
        else { crate::Icon { name: "user".to_string(), size: 20 } }
    } }
}

/// Horizontal or vertical content separator.
#[component]
pub fn Divider(#[props(default)] vertical: bool) -> Element {
    rsx! { div { class: "ix-divider", class: if vertical { "ix-divider-vertical" }, role: "separator", aria_orientation: if vertical { "vertical" } else { "horizontal" } } }
}

action_enum!(ProgressType, Linear, Linear => "linear", Circular => "circular");
action_enum!(ProgressSize, Md, Xs => "xs", Sm => "sm", Md => "md", Lg => "lg", Xl => "xl");
impl ProgressSize {
    /// Circular diameter in pixels.
    pub const fn pixels(self) -> u32 {
        match self {
            Self::Xs => 16,
            Self::Sm => 20,
            Self::Md => 32,
            Self::Lg => 48,
            Self::Xl => 64,
        }
    }
}
action_enum!(ProgressStatus, Default, Default => "default", Error => "error", Info => "info", Warning => "warning", Success => "success", Paused => "paused");
action_enum!(TextAlignment, Left, Left => "left", Center => "center", Right => "right");

/// Normalize finite progress input; invalid and inverted ranges are empty.
pub fn normalize_progress(value: f64, min: f64, max: f64) -> f64 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || max <= min {
        0.0
    } else {
        ((value - min) / (max - min) * 100.0).clamp(0.0, 100.0)
    }
}

/// Determinate linear or circular progress with semantic status and helper text.
/// Values are normalized to 0–100 for both paint and accessibility. Invalid
/// ranges render empty. Tooltip mode uses a native title; the helper remains
/// available as the progress bar's accessible description.
#[component]
pub fn ProgressIndicator(
    #[props(default)] value: f64,
    #[props(default)] min: f64,
    #[props(default = 100.0)] max: f64,
    #[props(default)] progress_type: ProgressType,
    #[props(default)] size: ProgressSize,
    #[props(default)] status: ProgressStatus,
    #[props(default)] text_alignment: TextAlignment,
    #[props(default)] show_text_as_tooltip: bool,
    #[props(default)] class: String,
    label: Option<String>,
    helper_text: Option<String>,
    children: Option<Element>,
) -> Element {
    let percent = normalize_progress(value, min, max);
    let offset = 100.0 - percent;
    let status_icon = match status {
        ProgressStatus::Error => Some("error"),
        ProgressStatus::Info => Some("info"),
        ProgressStatus::Warning => Some("warning"),
        ProgressStatus::Success => Some("success"),
        ProgressStatus::Paused => Some("circle-pause"),
        _ => None,
    };
    let title = if show_text_as_tooltip {
        helper_text.clone()
    } else {
        None
    };
    let accessible_label = label.clone().unwrap_or_else(|| "Progress".into());
    rsx! { div {
        class: "ix-progress ix-progress-{progress_type.as_str()} ix-progress-{size.as_str()} ix-progress-status-{status.as_str()} ix-progress-text-{text_alignment.as_str()} {class}",
        style: "text-align:{text_alignment.as_str()};--ix-progress-size:{size.pixels()}px", title,
        if let Some(label) = label { div { class: "ix-progress-label", "{label}" } }
        div { class: "ix-progress-content",
            if progress_type == ProgressType::Linear {
                div { class: "ix-progress-track", role: "progressbar", aria_label: accessible_label,
                    "aria-description": helper_text.clone(),
                    aria_valuemin: "0", aria_valuemax: "100", aria_valuenow: percent,
                    div { class: "ix-progress-fill", style: "width:{percent}%" }
                }
                if let Some(children) = children { div { class: "ix-progress-value", {children} } }
            } else {
                span { class: "ix-progress-circle", role: "progressbar", aria_label: accessible_label,
                    "aria-description": helper_text.clone(),
                    aria_valuemin: "0", aria_valuemax: "100", aria_valuenow: percent,
                    svg { view_box: "0 0 40 40", "aria-hidden": "true",
                        circle { class: "ix-progress-circle-track", cx: "20", cy: "20", r: "17" }
                        circle { class: "ix-progress-circle-fill", cx: "20", cy: "20", r: "17", path_length: "100", stroke_dasharray: "100", stroke_dashoffset: "{offset}" }
                    }
                    if matches!(size, ProgressSize::Lg | ProgressSize::Xl) { if let Some(children) = children.clone() { span { class: "ix-progress-circle-value", {children} } } }
                }
                if !matches!(size, ProgressSize::Lg | ProgressSize::Xl) { if let Some(children) = children { span { class: "ix-progress-value", {children} } } }
            }
        }
        if !show_text_as_tooltip { if let Some(helper) = helper_text {
            div { class: "ix-progress-helper",
                if let Some(name) = status_icon { crate::Icon { name: name.to_string(), size: 16 } }
                span { class: "ix-progress-helper-text", "{helper}" }
            }
        } }
    } }
}

action_enum!(TypographyFormat, Body,
    Body => "body", BodySbold => "body-sbold", BodyParagraph => "body-paragraph", BodyParagraphSbold => "body-paragraph-sbold",
    BodySm => "body-sm", BodySmSbold => "body-sm-sbold", BodyLg => "body-lg", BodyLgSbold => "body-lg-sbold",
    Display => "display", DisplaySbold => "display-sbold", DisplayLg => "display-lg", DisplayLgSbold => "display-lg-sbold",
    DisplayXl => "display-xl", DisplayXlSbold => "display-xl-sbold", DisplayXxl => "display-xxl", DisplayXxlSbold => "display-xxl-sbold",
    H6 => "h6", H5 => "h5", H5Bold => "h5-bold", H4 => "h4", H3 => "h3", H2 => "h2", H1 => "h1",
    Code => "code", CodeSm => "code-sm", CodeLg => "code-lg");
action_enum!(TypographyColor, Std,
    Std => "std", Soft => "soft", Weak => "weak", Alarm => "alarm", Contrast => "contrast",
    InvContrast => "inv-contrast", InvSoft => "inv-soft", InvStd => "inv-std", InvWeak => "inv-weak", Inverse => "inverse",
    PrimaryContrast => "primary-contrast", AlarmContrast => "alarm-contrast", CriticalContrast => "critical-contrast",
    InfoContrast => "info-contrast", NeutralContrast => "neutral-contrast", SuccessContrast => "success-contrast", WarningContrast => "warning-contrast");
action_enum!(TypographyDecoration, None, None => "none", Underline => "underline", LineThrough => "line-through");

/// Text styles with semantic heading levels and code roles where appropriate.
#[component]
pub fn Typography(
    children: Element,
    #[props(default)] format: TypographyFormat,
    text_color: Option<TypographyColor>,
    #[props(default)] bold: bool,
    #[props(default)] text_decoration: TypographyDecoration,
    #[props(default)] class: String,
    id: Option<String>,
    role: Option<String>,
    aria_live: Option<String>,
    aria_hidden: Option<String>,
) -> Element {
    let heading = match format {
        TypographyFormat::H1 => Some(1),
        TypographyFormat::H2 => Some(2),
        TypographyFormat::H3 => Some(3),
        TypographyFormat::H4 => Some(4),
        TypographyFormat::H5 | TypographyFormat::H5Bold => Some(5),
        TypographyFormat::H6 => Some(6),
        _ => None,
    };
    let color_class = text_color
        .map(|color| format!(" ix-text-{}", color.as_str()))
        .unwrap_or_default();
    rsx! { div {
        class: "ix-typography ix-type-{format.as_str()}{color_class} {class}",
        class: if bold { "ix-type-bold" }, style: "text-decoration:{text_decoration.as_str()}",
        id, role: role.or_else(|| heading.map(|_| "heading".to_string())), aria_level: heading, "aria-hidden": aria_hidden, "aria-live": aria_live,
        {children}
    } }
}

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-button{box-sizing:border-box;display:inline-flex;align-items:center;justify-content:center;gap:8px;min-height:32px;min-width:64px;padding:4px 12px;border:1px solid transparent;border-radius:var(--ix-radius);font:600 14px/22px var(--ix-font);cursor:pointer;text-decoration:none;white-space:nowrap;vertical-align:middle;transition:background-color .12s,color .12s,border-color .12s}
.ix-theme .ix-button-content{overflow:hidden;text-overflow:ellipsis;min-width:0}.ix-theme .ix-align-start{justify-content:flex-start}
.ix-theme .ix-button-primary{background:var(--ix-color-primary);color:var(--ix-color-on-primary)}
.ix-theme .ix-button-secondary{background:color-mix(in srgb,var(--ix-color-primary) 10%,transparent);color:var(--ix-color-primary);border-color:var(--ix-color-primary)}
.ix-theme .ix-button-tertiary{background:transparent;color:var(--ix-color-primary)}
.ix-theme .ix-button-subtle-primary{background:var(--ix-color-surface-2);color:var(--ix-color-text)}
.ix-theme .ix-button-subtle-secondary{background:transparent;border-color:var(--ix-color-border);color:var(--ix-color-text)}
.ix-theme .ix-button-subtle-tertiary{background:transparent;color:var(--ix-color-text)}
.ix-theme .ix-button-danger-primary{background:var(--ix-color-danger);color:var(--ix-color-on-danger,#fff)}
.ix-theme .ix-button-danger-secondary{background:transparent;border-color:var(--ix-color-danger-text);color:var(--ix-color-danger-text)}
.ix-theme .ix-button-danger-tertiary{background:transparent;color:var(--ix-color-danger-text)}
.ix-theme .ix-button:hover:not(:disabled):not([aria-disabled=true]){filter:brightness(1.1)}
.ix-theme .ix-button-tertiary:hover:not(:disabled),.ix-theme .ix-button-subtle-tertiary:hover:not(:disabled){background:var(--ix-color-surface-2)}
.ix-theme .ix-button:active:not(:disabled),.ix-theme .ix-button.ix-selected{box-shadow:inset 0 0 0 2px currentColor;background:color-mix(in srgb,var(--ix-color-primary) 24%,var(--ix-color-surface-1))}
.ix-theme .ix-button-primary.ix-selected{color:var(--ix-color-primary)}
.ix-theme .ix-button:disabled,.ix-theme .ix-button[aria-disabled=true]{cursor:default;color:var(--ix-color-text-disabled,var(--ix-color-text-secondary));background:var(--ix-color-surface-1);border-color:transparent;opacity:.55}
.ix-theme .ix-button:focus-visible,.ix-theme .ix-link-button:focus-visible,.ix-theme .ix-chip button:focus-visible,.ix-theme .ix-filter-chip button:focus-visible,.ix-theme .ix-split-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}
.ix-theme .ix-icon-button{width:32px;min-width:32px;height:32px;padding:4px;flex-shrink:0}.ix-theme .ix-icon-size-12{width:24px;min-width:24px;height:24px;min-height:24px}.ix-theme .ix-icon-size-16{width:24px;min-width:24px;height:24px;min-height:24px}.ix-theme .ix-oval{border-radius:50%}
.ix-theme .ix-link-button{font:inherit;color:var(--ix-color-primary);text-decoration:underline;text-underline-offset:3px;cursor:pointer}.ix-theme .ix-link-button:hover{color:var(--ix-color-text)}.ix-theme .ix-link-button[aria-disabled=true]{color:var(--ix-color-text-secondary);cursor:default;text-decoration:none;opacity:.55}
.ix-theme .ix-split-button{display:inline-flex;position:relative;vertical-align:middle}.ix-theme .ix-split-button>.ix-button:first-child{border-top-right-radius:0;border-bottom-right-radius:0}.ix-theme .ix-split-trigger{border-top-left-radius:0;border-bottom-left-radius:0;border-left-color:currentColor}.ix-theme .ix-split-backdrop{position:fixed;inset:0;z-index:59;opacity:0;border:0;cursor:default}.ix-theme .ix-split-menu{position:absolute;top:calc(100% + 4px);left:0;min-width:180px;padding:4px;z-index:60;background:var(--ix-color-surface-2);box-shadow:0 4px 16px #0005;border-radius:var(--ix-radius)}.ix-theme .ix-split-item{display:flex;gap:8px;align-items:center;width:100%;padding:8px 12px;border:0;border-radius:var(--ix-radius);text-align:left;background:transparent;color:var(--ix-color-text);font:inherit;cursor:pointer}.ix-theme .ix-split-item:hover{background:var(--ix-color-surface-1)}.ix-theme .ix-split-item:disabled{opacity:.4;cursor:default}
.ix-theme .ix-status-primary{--ix-status-bg:var(--ix-color-primary);--ix-status-fg:var(--ix-color-on-primary)}.ix-theme .ix-status-alarm,.ix-theme .ix-status-error{--ix-status-bg:var(--ix-color-danger);--ix-status-fg:var(--ix-color-on-danger,#fff)}.ix-theme .ix-status-critical{--ix-status-bg:var(--ix-color-critical,#f89a00);--ix-status-fg:var(--ix-color-on-critical)}.ix-theme .ix-status-warning{--ix-status-bg:var(--ix-color-warning);--ix-status-fg:var(--ix-color-on-warning,#fff)}.ix-theme .ix-status-info{--ix-status-bg:var(--ix-color-info);--ix-status-fg:var(--ix-color-on-info,#fff)}.ix-theme .ix-status-neutral{--ix-status-bg:var(--ix-color-surface-2);--ix-status-fg:var(--ix-color-text)}.ix-theme .ix-status-success{--ix-status-bg:var(--ix-color-success);--ix-status-fg:var(--ix-color-on-success,#fff)}.ix-theme .ix-status-custom{--ix-status-bg:var(--ix-color-surface-2);--ix-status-fg:var(--ix-color-text)}
.ix-theme .ix-pill,.ix-theme .ix-chip,.ix-theme .ix-filter-chip{box-sizing:border-box;display:inline-flex;align-items:center;gap:6px;max-width:100%;vertical-align:middle;font:14px/20px var(--ix-font);border:1px solid transparent;border-radius:16px;overflow:hidden}.ix-theme .ix-pill{height:20px;min-height:20px;padding:0 8px;line-height:18px;justify-content:center;background:var(--ix-status-bg);color:var(--ix-status-fg)}.ix-theme .ix-status-label{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-outline{border-color:var(--ix-status-bg);color:var(--ix-status-bg);background:transparent}.ix-theme .ix-chip{min-height:32px;gap:0;background:var(--ix-status-bg);color:var(--ix-status-fg)}.ix-theme .ix-chip.ix-outline{background:transparent;color:var(--ix-status-bg)}.ix-theme .ix-chip button{font:inherit;color:inherit;background:transparent;border:0;cursor:pointer}.ix-theme .ix-chip-main{display:inline-flex;align-items:center;gap:6px;padding:5px 12px;min-width:0;min-height:30px;flex:1}.ix-theme .ix-chip-main:disabled{cursor:default}.ix-theme .ix-chip-close{display:inline-flex;align-items:center;justify-content:center;width:28px;height:28px;margin-right:1px;flex-shrink:0;border-radius:50%}.ix-theme .ix-chip button:hover:not(:disabled){background:color-mix(in srgb,currentColor 12%,transparent)}.ix-theme .ix-center-content{justify-content:center}.ix-theme .ix-status-custom.ix-outline{color:var(--ix-status-fg)}.ix-theme .ix-badge.ix-status-custom.ix-outline .ix-badge-indicator{color:var(--ix-status-fg)}
/* Outlined status labels keep readable text while icon and border retain status. */
.ix-theme .ix-pill.ix-outline{color:var(--ix-pill-outline--color,var(--ix-color-text))}
.ix-theme .ix-pill.ix-outline>.ix-icon{color:var(--ix-pill-outline-icon,var(--ix-status-bg))}
.ix-theme .ix-pill.ix-status-warning.ix-outline{--ix-pill-outline-icon:var(--ix-color-warning-text)}
.ix-theme .ix-pill.ix-status-alarm.ix-outline,.ix-theme .ix-pill.ix-status-error.ix-outline{--ix-pill-outline-icon:var(--ix-color-danger-text)}
.ix-theme .ix-pill.ix-status-info.ix-outline{--ix-pill-outline-icon:var(--ix-color-info-text)}
.ix-theme .ix-pill.ix-status-neutral.ix-outline{--ix-pill-outline-icon:var(--ix-color-text);border-color:var(--ix-color-border)}
.ix-theme .ix-pill.ix-status-custom.ix-outline{color:var(--ix-status-fg);--ix-pill-outline-icon:var(--ix-status-fg)}
.ix-theme .ix-chip.ix-outline{color:var(--ix-chip-outline--color,var(--ix-color-text))}
.ix-theme .ix-chip.ix-outline .ix-chip-main>.ix-icon{color:var(--ix-chip-outline-icon,var(--ix-status-bg))}
.ix-theme .ix-chip.ix-status-warning.ix-outline{--ix-chip-outline-icon:var(--ix-color-warning-text)}
.ix-theme .ix-chip.ix-status-alarm.ix-outline,.ix-theme .ix-chip.ix-status-error.ix-outline{--ix-chip-outline-icon:var(--ix-color-danger-text)}
.ix-theme .ix-chip.ix-status-info.ix-outline{--ix-chip-outline-icon:var(--ix-color-info-text)}
.ix-theme .ix-chip.ix-status-neutral.ix-outline{--ix-chip-outline-icon:var(--ix-color-text);border-color:var(--ix-color-border)}
.ix-theme .ix-chip.ix-status-custom.ix-outline{color:var(--ix-status-fg)}
/* Keep the native keyboard ring inside the rounded, clipped chip surface. */
.ix-theme .ix-chip button:focus-visible,.ix-theme .ix-filter-chip button:focus-visible{outline-offset:-3px}
.ix-theme .ix-filter-chip{background:var(--ix-color-surface-2);color:var(--ix-color-text);padding:3px 4px 3px 12px;min-height:32px}.ix-theme .ix-filter-chip.ix-disabled{opacity:.5}.ix-theme .ix-filter-close{display:inline-flex;align-items:center;justify-content:center;background:transparent;color:inherit;border:0;border-radius:50%;width:24px;height:24px;cursor:pointer}.ix-theme .ix-filter-close:hover{background:var(--ix-color-surface-1)}
.ix-theme .ix-badge{position:relative;display:inline-flex;vertical-align:middle}.ix-theme .ix-badge-indicator{box-sizing:border-box;display:inline-flex;align-items:center;justify-content:center;gap:4px;min-width:20px;height:20px;padding:0 4px;border-radius:10px;background:var(--ix-status-bg);color:var(--ix-status-fg);font:700 14px/20px var(--ix-font);white-space:nowrap;max-width:var(--ix-badge-max-width,160px);overflow:hidden;text-overflow:ellipsis}.ix-theme .ix-badge-label:not(.ix-badge-attached) .ix-badge-indicator{width:100%}.ix-theme .ix-badge-label-text{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-badge-label .ix-badge-indicator{height:20px;line-height:20px;border-radius:10px;padding:0 8px;font-weight:400}.ix-theme .ix-badge-dot .ix-badge-indicator{min-width:12px;width:12px;height:12px;padding:0;border-radius:50%}.ix-theme .ix-badge-status-icon .ix-badge-indicator{min-width:20px;width:20px;height:20px;padding:0;border-radius:0;background:transparent;color:var(--ix-status-bg);overflow:visible}.ix-theme .ix-badge-status-stack{position:relative;display:inline-flex;width:20px;height:20px}.ix-theme .ix-badge-status-plate,.ix-theme .ix-badge-status-glyph{position:absolute;inset:0}.ix-theme .ix-badge-status-plate{color:var(--ix-status-fg)}.ix-theme .ix-badge.ix-outline.ix-badge-status-icon .ix-badge-indicator{background:transparent;border:0}.ix-theme .ix-badge.ix-outline .ix-badge-indicator{border:1px solid var(--ix-status-bg);background:var(--ix-color-surface);color:var(--ix-status-bg)}.ix-theme .ix-badge-attached .ix-badge-indicator{position:absolute;right:var(--ix-badge-x);top:var(--ix-badge-y)}.ix-theme .ix-badge-bottom-after .ix-badge-indicator{top:auto;bottom:var(--ix-badge-y)}.ix-theme .ix-badge-border .ix-badge-indicator{outline:2px solid var(--ix-color-surface)}.ix-theme .ix-badge-animated .ix-badge-indicator{position:relative;overflow:visible;isolation:isolate}.ix-theme .ix-badge-attached.ix-badge-animated .ix-badge-indicator{position:absolute}.ix-theme .ix-badge-animated .ix-badge-indicator::after{content:"";position:absolute;inset:.5px;z-index:-1;pointer-events:none;box-sizing:border-box;border:0 solid var(--ix-status-bg);border-radius:inherit;opacity:0;animation:ix-badge-pulse-ring var(--ix-badge-animation-duration,2s) linear infinite}.ix-theme .ix-badge-status-icon.ix-badge-animated .ix-badge-indicator::after{inset:.4px;border:0;background:var(--ix-status-bg);border-radius:50%;transform:scale(.521);animation-name:ix-badge-pulse-status}@keyframes ix-badge-pulse-ring{0%,100%{inset:.5px;border-width:0;opacity:0}5%{inset:.5px;border-width:0;opacity:.5}45%{opacity:.4}80%{inset:-7.5px;border-width:8px;opacity:.1;animation-timing-function:ease-out}90%{inset:-9.5px;border-width:10px;opacity:0}}@keyframes ix-badge-pulse-status{0%,100%{transform:scale(.521);opacity:0}5%{transform:scale(.521);opacity:.5;animation-timing-function:cubic-bezier(.637,.43,.928,1.008)}45%{transform:scale(1);opacity:.4}80%{transform:scale(1.6);opacity:.1;animation-timing-function:ease-out}90%{transform:scale(1.8);opacity:0}}
.ix-theme .ix-avatar{display:inline-flex;align-items:center;justify-content:center;overflow:hidden;flex-shrink:0;vertical-align:middle;width:var(--ix-avatar-size);height:var(--ix-avatar-size);border-radius:50%;background:var(--ix-color-surface-2);color:var(--ix-color-text);font:600 14px/1 var(--ix-font)}.ix-theme .ix-avatar img{width:100%;height:100%;object-fit:cover}.ix-theme .ix-divider{height:1px;background:var(--ix-color-border);margin:16px 0;flex-shrink:0}.ix-theme .ix-divider-vertical{height:auto;min-height:24px;width:1px;margin:0 16px;align-self:stretch}
.ix-theme .ix-typography{display:block;position:relative;font:var(--si-sys-typography-body,14px/20px var(--ix-font));margin:0;overflow-wrap:break-word}.ix-theme .ix-type-body{font:var(--si-sys-typography-body,14px/20px var(--ix-font))}.ix-theme .ix-type-body-sbold{font:var(--si-sys-typography-body-sbold,600 14px/20px var(--ix-font))}.ix-theme .ix-type-body-paragraph{font:var(--si-sys-typography-body-paragraph,14px/24px var(--ix-font))}.ix-theme .ix-type-body-paragraph-sbold{font:var(--si-sys-typography-body-paragraph-sbold,600 14px/24px var(--ix-font))}.ix-theme .ix-type-body-sm{font:var(--si-sys-typography-body-sm,12px/16px var(--ix-font))}.ix-theme .ix-type-body-sm-sbold{font:var(--si-sys-typography-body-sm-sbold,600 12px/16px var(--ix-font))}.ix-theme .ix-type-body-lg{font:var(--si-sys-typography-body-lg,16px/24px var(--ix-font))}.ix-theme .ix-type-body-lg-sbold{font:var(--si-sys-typography-body-lg-sbold,600 16px/24px var(--ix-font))}.ix-theme .ix-type-display{font:var(--si-sys-typography-display,40px/48px var(--ix-font))}.ix-theme .ix-type-display-sbold{font:var(--si-sys-typography-display-sbold,600 40px/48px var(--ix-font))}.ix-theme .ix-type-display-lg{font:var(--si-sys-typography-display-lg,48px/56px var(--ix-font))}.ix-theme .ix-type-display-lg-sbold{font:var(--si-sys-typography-display-lg-sbold,600 48px/56px var(--ix-font))}.ix-theme .ix-type-display-xl{font:var(--si-sys-typography-display-xl,56px/64px var(--ix-font))}.ix-theme .ix-type-display-xl-sbold{font:var(--si-sys-typography-display-xl-sbold,600 56px/64px var(--ix-font))}.ix-theme .ix-type-display-xxl{font:var(--si-sys-typography-display-xxl,64px/72px var(--ix-font))}.ix-theme .ix-type-display-xxl-sbold{font:var(--si-sys-typography-display-xxl-sbold,600 64px/72px var(--ix-font))}.ix-theme .ix-type-h1{font:var(--si-sys-typography-h1,40px/48px var(--ix-font))}.ix-theme .ix-type-h2{font:var(--si-sys-typography-h2,32px/40px var(--ix-font))}.ix-theme .ix-type-h3{font:var(--si-sys-typography-h3,24px/32px var(--ix-font))}.ix-theme .ix-type-h4{font:var(--si-sys-typography-h4,20px/28px var(--ix-font))}.ix-theme .ix-type-h5{font:var(--si-sys-typography-h5,16px/24px var(--ix-font))}.ix-theme .ix-type-h5-bold{font:var(--si-sys-typography-h5-bold,600 16px/24px var(--ix-font))}.ix-theme .ix-type-h6{font:var(--si-sys-typography-h6,600 14px/20px var(--ix-font))}.ix-theme .ix-type-code{font:var(--si-sys-typography-code,14px/20px monospace)}.ix-theme .ix-type-code-sm{font:var(--si-sys-typography-code-sm,12px/16px monospace)}.ix-theme .ix-type-code-lg{font:var(--si-sys-typography-code-lg,16px/24px monospace)}.ix-theme .ix-typography{font-feature-settings:"clig" off,"liga" off;font-style:normal;letter-spacing:var(--si-ref-typography-letter-spacing-normal,normal);-webkit-font-smoothing:antialiased}.ix-theme .ix-type-bold{font-weight:var(--ix-typography-weight-bold--font-weight,var(--si-ref-typography-font-weight-bold,700))!important}.ix-theme .ix-text-std,.ix-theme .ix-text-contrast{color:var(--si-sys-color-text-primary,var(--ix-color-text))}.ix-theme .ix-text-soft{color:var(--si-sys-color-text-secondary,var(--ix-color-text-secondary))}.ix-theme .ix-text-weak{color:var(--si-sys-color-text-disabled,var(--ix-color-text-disabled,var(--ix-color-text-secondary)))}.ix-theme .ix-text-alarm{color:var(--si-sys-color-text-danger,var(--ix-color-danger-text))}.ix-theme .ix-text-alarm-contrast{color:var(--si-sys-color-text-on-danger,var(--ix-color-on-danger,#fff))}.ix-theme .ix-text-critical-contrast{color:var(--si-sys-color-text-on-critical,var(--ix-color-on-critical,#000028))}.ix-theme .ix-text-info-contrast{color:var(--si-sys-color-text-on-information,var(--ix-color-on-info,#fff))}.ix-theme .ix-text-inv-contrast,.ix-theme .ix-text-inv-soft,.ix-theme .ix-text-inv-std,.ix-theme .ix-text-inv-weak,.ix-theme .ix-text-inverse{color:var(--si-sys-color-text-inverse,var(--ix-color-on-primary))}.ix-theme .ix-text-neutral-contrast{color:var(--si-sys-color-text-on-neutral,var(--ix-color-text))}.ix-theme .ix-text-primary-contrast{color:var(--si-sys-color-text-on-accent,var(--ix-color-on-primary))}.ix-theme .ix-text-success-contrast{color:var(--si-sys-color-text-on-success,var(--ix-color-on-success,#fff))}.ix-theme .ix-text-warning-contrast{color:var(--si-sys-color-text-on-warning,var(--ix-color-on-warning,#000028))}
@media(prefers-reduced-motion:reduce){.ix-theme .ix-badge-animated .ix-badge-indicator::after{animation:none;opacity:0}.ix-theme .ix-button,.ix-theme .ix-progress-fill,.ix-theme .ix-progress-circle-fill{transition:none}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, collections::BTreeMap};
    #[test]
    fn badge_counter_matches_upstream_numeric_rules() {
        for (input, expected) in [
            ("99", Some("99")),
            ("100", Some("99+")),
            ("99+", Some("99+")),
            (" -3.8 ", Some("-3")),
            ("0", Some("0")),
            ("1e2", None),
            ("+1", None),
            (".5", None),
            ("1.", None),
            ("abc", None),
            ("", None),
        ] {
            assert_eq!(format_badge_counter(input).as_deref(), expected, "{input}");
        }
    }
    #[test]
    fn progress_handles_arbitrary_ranges_and_bad_inputs() {
        assert_eq!(normalize_progress(150.0, 100.0, 200.0), 50.0);
        assert_eq!(normalize_progress(-1.0, 0.0, 100.0), 0.0);
        assert_eq!(normalize_progress(200.0, 0.0, 100.0), 100.0);
        assert_eq!(normalize_progress(0.0, 1.0, 1.0), 0.0);
        assert_eq!(normalize_progress(f64::NAN, 0.0, 100.0), 0.0);
    }
    #[test]
    fn badge_status_icons_preserve_distinct_alarm_error_and_critical_anatomies() {
        assert_eq!(StatusVariant::Alarm.status_icon(false), "alarm-filled");
        assert_eq!(StatusVariant::Error.status_icon(true), "error");
        assert_eq!(StatusVariant::Critical.status_icon(true), "warning-rhomb");
        assert_eq!(StatusVariant::Critical.status_plate(), "rhomb-filled");
        assert_eq!(StatusVariant::Warning.status_plate(), "triangle-filled");
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Badge { badge_type: BadgeType::StatusIcon, variant: StatusVariant::Primary, role: "img", aria_label: "Information" }
                Badge { label: "3", variant: StatusVariant::Error }
                Chip { aria_label: "In review", "Info" }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("ix-status-info"));
        assert!(html.contains("ix-status-alarm"));
        assert!(html.contains("aria-label=\"Information\""));
        assert!(html.contains("aria-label=\"In review\""));
    }
    #[test]
    fn typography_omits_color_class_by_default_and_preserves_semantic_roles_and_variants() {
        for (color, expected) in [
            (TypographyColor::InvContrast, "inv-contrast"),
            (TypographyColor::InvSoft, "inv-soft"),
            (TypographyColor::InvStd, "inv-std"),
            (TypographyColor::InvWeak, "inv-weak"),
            (TypographyColor::Inverse, "inverse"),
            (TypographyColor::CriticalContrast, "critical-contrast"),
            (TypographyColor::WarningContrast, "warning-contrast"),
            (TypographyColor::InfoContrast, "info-contrast"),
            (TypographyColor::NeutralContrast, "neutral-contrast"),
        ] {
            assert_eq!(color.as_str(), expected);
        }
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Typography { class: "omitted-color", "Inherited" }
                Typography { text_color: TypographyColor::Soft, class: "direct-enum", "Soft" }
                Typography { text_color: Some(TypographyColor::InvContrast), class: "some-inv-contrast", "InvContrast" }
                Typography { text_color: TypographyColor::InvSoft, "InvSoft" }
                Typography { text_color: TypographyColor::InvStd, "InvStd" }
                Typography { text_color: TypographyColor::InvWeak, "InvWeak" }
                Typography { text_color: TypographyColor::Inverse, "InverseAlias" }
                Typography {
                    format: TypographyFormat::H5Bold,
                    text_color: TypographyColor::Weak,
                    bold: true,
                    text_decoration: TypographyDecoration::Underline,
                    "HeadingFive"
                }
                Typography {
                    format: TypographyFormat::CodeSm,
                    text_decoration: TypographyDecoration::LineThrough,
                    "CodeSample"
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let classes: Vec<Vec<_>> = html
            .split("class=\"")
            .skip(1)
            .map(|value| {
                value
                    .split('"')
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .collect()
            })
            .collect();
        let omitted = classes
            .iter()
            .find(|names| names.contains(&"omitted-color"))
            .unwrap();
        assert!(omitted.contains(&"ix-typography") && omitted.contains(&"ix-type-body"));
        assert!(omitted.iter().all(|name| !name.starts_with("ix-text-")));
        assert!(!html.contains("ix-text-std"));
        assert!(
            classes
                .iter()
                .any(|names| names.contains(&"direct-enum") && names.contains(&"ix-text-soft"))
        );
        assert!(
            classes
                .iter()
                .any(|names| names.contains(&"some-inv-contrast")
                    && names.contains(&"ix-text-inv-contrast"))
        );
        assert!(html.contains("ix-text-inv-soft"));
        assert!(html.contains("ix-text-inv-std"));
        assert!(html.contains("ix-text-inv-weak"));
        assert!(html.contains("ix-text-inverse"));
        assert!(html.contains("ix-type-h5-bold ix-text-weak"));
        assert!(html.contains("ix-type-bold"));
        assert!(html.contains("role=\"heading\""));
        assert!(html.contains("aria-level=5") || html.contains("aria-level=\"5\""));
        assert!(html.contains("style=\"text-decoration:underline\""));
        assert!(html.contains("ix-type-code-sm"));
        assert!(html.contains("style=\"text-decoration:line-through\""));
    }

    #[derive(Clone, Default)]
    struct ActionFocusMount {
        label: String,
        focuses: Rc<RefCell<Vec<String>>>,
    }
    impl dioxus::html::RenderedElementBacking for ActionFocusMount {
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
    struct ActionConverter;
    macro_rules! action_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for ActionConverter {
                $(fn $convert(&self, event: &dioxus::html::PlatformEventData) -> dioxus::html::$data {
                    let _ = event;
                    action_converter!(@data $group, event)
                })*
            }
        };
        (@data Mounted, $event:ident) => {
            if let Some(mock) = $event.downcast::<ActionFocusMount>() {
                MountedData::new(mock.clone())
            } else {
                MountedData::new(())
            }
        };
        (@data $other:ident, $event:ident) => {
            panic!("Unexpected action test event")
        };
    }
    dioxus::html::with_html_event_groups!(action_converter);

    type ActionUpdate = std::sync::Arc<dyn Fn() + Send + Sync>;

    #[derive(Clone, Default)]
    struct ActionMountState {
        node: Rc<RefCell<Option<crate::overlay_runtime::NodeHandle>>>,
        update: Rc<RefCell<Option<ActionUpdate>>>,
        visible: Rc<Cell<bool>>,
        button_id: Rc<RefCell<Option<String>>>,
        button_href: Rc<RefCell<Option<String>>>,
        icon_button_id: Rc<RefCell<Option<String>>>,
        button_mounts: Rc<Cell<usize>>,
        icon_button_mounts: Rc<Cell<usize>>,
    }
    impl PartialEq for ActionMountState {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.node, &other.node)
        }
    }
    fn action_mount_fixture(state: ActionMountState) -> Element {
        *state.update.borrow_mut() = Some(dioxus::core::schedule_update());
        let runtime = crate::overlay_runtime::use_overlay_runtime();
        let node = use_hook(|| {
            runtime.register(
                "actions-mount-test-owner".into(),
                1,
                crate::overlay_registry::OverlayKind::Dropdown,
                crate::overlay_registry::CloseBehavior::Both,
                crate::overlay_runtime::Callbacks {
                    read: Rc::new(|| Some(true)),
                    before: Rc::new(|_| true),
                    apply: Rc::new(|_| {}),
                    key: None,
                },
            )
        });
        *state.node.borrow_mut() = Some(node);
        let button_mounts = state.button_mounts.clone();
        let icon_button_mounts = state.icon_button_mounts.clone();
        rsx! {
            div {
                if state.visible.get() {
                    Button {
                        id: state.button_id.borrow().clone(),
                        href: state.button_href.borrow().clone(),
                        aria_label: "ActionButton",
                        onmounted: move |_| button_mounts.set(button_mounts.get() + 1),
                        "Action"
                    }
                    IconButton {
                        id: state.icon_button_id.borrow().clone(),
                        icon: "star",
                        label: "ActionIconButton",
                        onmounted: move |_| icon_button_mounts.set(icon_button_mounts.get() + 1),
                    }
                }
            }
        }
    }

    #[test]
    fn button_and_icon_button_render_default_and_custom_ids_across_button_and_anchor_modes() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Button { aria_label: "DefaultBtn", "Default" }
                Button { id: "", aria_label: "EmptyIdBtn", "Empty" }
                Button { id: "explicit-btn", aria_label: "ExplicitBtn", "Explicit" }
                Button { href: "https://example.com", aria_label: "DefaultAnchor", "Anchor" }
                Button { id: "explicit-anchor", href: "https://example.com", disabled: true, aria_label: "DisabledAnchor", "DisabledAnchor" }
                IconButton { icon: "star", label: "DefaultIcon" }
                IconButton { id: "", icon: "star", label: "EmptyIdIcon" }
                IconButton { id: "explicit-icon", icon: "star", label: "ExplicitIcon" }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"explicit-btn\""));
        assert!(html.contains("id=\"explicit-anchor\""));
        assert!(html.contains("id=\"explicit-icon\""));
        let generated_ids: Vec<_> = html
            .split("id=\"")
            .skip(1)
            .map(|part| part.split('"').next().unwrap())
            .filter(|id| id.starts_with("ix-source-native-opt-in-"))
            .collect();
        assert_eq!(
            generated_ids.len(),
            5,
            "omitted and empty IDs fall back to unique native-opt-in host IDs"
        );
    }

    #[test]
    fn button_and_icon_button_register_mounted_handles_update_ids_forward_onmounted_once_and_clean_up_on_drop()
     {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(ActionConverter));
        let state = ActionMountState {
            visible: Rc::new(Cell::new(true)),
            ..Default::default()
        };
        let mut dom = VirtualDom::new_with_props(action_mount_fixture, state.clone());
        let mut attrs: BTreeMap<usize, BTreeMap<String, String>> = BTreeMap::new();
        let collect =
            |mutations: dioxus::core::Mutations,
             attrs: &mut BTreeMap<usize, BTreeMap<String, String>>| {
                let mut current = dioxus::core::ElementId::ROOT;
                for edit in mutations.edits {
                    match edit {
                        dioxus::core::Mutation::SetId { id }
                        | dioxus::core::Mutation::PushId { id } => current = id,
                        dioxus::core::Mutation::SetAttribute {
                            name,
                            value: dioxus::core::AttributeValue::Text(text),
                            ..
                        } => {
                            attrs.entry(current.raw()).or_default().insert(name, text);
                        }
                        dioxus::core::Mutation::Remove => {
                            attrs.remove(&current.raw());
                        }
                        _ => {}
                    }
                }
            };
        collect(dom.rebuild_to_vec(), &mut attrs);
        let settle = |dom: &mut VirtualDom,
                      attrs: &mut BTreeMap<usize, BTreeMap<String, String>>| {
            for _ in 0..4 {
                let edits = dom.render_immediate_to_vec();
                collect(edits, attrs);
            }
        };
        let find_by_label =
            |attrs: &BTreeMap<usize, BTreeMap<String, String>>, label: &str| -> (usize, String) {
                attrs
                    .iter()
                    .find_map(|(raw, map)| {
                        (map.get("aria-label").map(String::as_str) == Some(label))
                            .then(|| (*raw, map.get("id").cloned().unwrap_or_default()))
                    })
                    .unwrap_or_else(|| panic!("missing element for {label}"))
            };
        let (btn_raw, initial_btn_id) = find_by_label(&attrs, "ActionButton");
        let (icon_raw, initial_icon_id) = find_by_label(&attrs, "ActionIconButton");
        assert!(initial_btn_id.starts_with("ix-source-native-opt-in-"));
        assert!(initial_icon_id.starts_with("ix-source-native-opt-in-"));

        let focuses = Rc::new(RefCell::new(Vec::<String>::new()));
        let dispatch_mount = |dom: &VirtualDom, raw: usize, label: &str| {
            let payload = ActionFocusMount {
                label: label.into(),
                focuses: focuses.clone(),
            };
            let data: Rc<dyn std::any::Any> =
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
            let event = Event::new(data, false);
            dom.runtime()
                .handle_event("mounted", event, dioxus::core::ElementId::from_raw(raw));
        };
        dispatch_mount(&dom, btn_raw, "button-node");
        dispatch_mount(&dom, icon_raw, "icon-button-node");
        settle(&mut dom, &mut attrs);

        assert_eq!(state.button_mounts.get(), 1);
        assert_eq!(
            state.icon_button_mounts.get(),
            1,
            "IconButton consumer onmounted must fire exactly once"
        );

        let node = state.node.borrow().clone().unwrap();
        let focus_target = |dom: &mut VirtualDom,
                            attrs: &mut BTreeMap<usize, BTreeMap<String, String>>,
                            target: &str|
         -> bool {
            let queued =
                dom.in_runtime(|| node.queue_item_focus(target.to_string(), true, Rc::new(|| {})));
            settle(dom, attrs);
            queued
        };
        assert!(focus_target(&mut dom, &mut attrs, &initial_btn_id));
        assert!(focus_target(&mut dom, &mut attrs, &initial_icon_id));
        assert_eq!(
            focuses.borrow().as_slice(),
            ["button-node", "icon-button-node"]
        );

        *state.button_id.borrow_mut() = Some("updated-btn-id".into());
        *state.icon_button_id.borrow_mut() = Some("updated-icon-id".into());
        state.update.borrow().as_ref().unwrap()();
        settle(&mut dom, &mut attrs);

        assert_eq!(state.button_mounts.get(), 1);
        assert_eq!(state.icon_button_mounts.get(), 1);
        assert!(
            !focus_target(&mut dom, &mut attrs, &initial_btn_id),
            "previous Button ID is superseded after prop update"
        );
        assert!(
            !focus_target(&mut dom, &mut attrs, &initial_icon_id),
            "previous IconButton ID is superseded after prop update"
        );
        assert!(focus_target(&mut dom, &mut attrs, "updated-btn-id"));
        assert!(focus_target(&mut dom, &mut attrs, "updated-icon-id"));
        assert_eq!(
            focuses.borrow().as_slice(),
            [
                "button-node",
                "icon-button-node",
                "button-node",
                "icon-button-node"
            ]
        );

        *state.button_href.borrow_mut() = Some("https://example.com".into());
        state.update.borrow().as_ref().unwrap()();
        settle(&mut dom, &mut attrs);
        let (anchor_raw, anchor_id) = find_by_label(&attrs, "ActionButton");
        assert_eq!(anchor_id, "updated-btn-id");
        dispatch_mount(&dom, anchor_raw, "anchor-node");
        settle(&mut dom, &mut attrs);
        assert_eq!(state.button_mounts.get(), 2);
        assert!(focus_target(&mut dom, &mut attrs, "updated-btn-id"));
        assert_eq!(
            focuses.borrow().last().map(String::as_str),
            Some("anchor-node")
        );

        state.visible.set(false);
        state.update.borrow().as_ref().unwrap()();
        settle(&mut dom, &mut attrs);
        assert!(
            !focus_target(&mut dom, &mut attrs, "updated-btn-id"),
            "Button use_drop unregisters its item mount"
        );
        assert!(
            !focus_target(&mut dom, &mut attrs, "updated-icon-id"),
            "IconButton use_drop unregisters its item mount"
        );
    }
}

#[cfg(test)]
#[path = "button_token_tests.rs"]
mod button_token_tests;

#[cfg(test)]
#[path = "icon_button_token_tests.rs"]
mod icon_button_token_tests;

#[cfg(test)]
#[path = "toggle_button_token_tests.rs"]
mod toggle_button_token_tests;

#[cfg(test)]
#[path = "icon_toggle_button_token_tests.rs"]
mod icon_toggle_button_token_tests;

#[cfg(test)]
#[path = "spinner_visual_tests.rs"]
mod spinner_visual_tests;

#[cfg(test)]
#[path = "progress_tests.rs"]
mod progress_tests;
