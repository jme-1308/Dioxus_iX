//! Native Rust action, status, and typography primitives.
//!
//! Behavioral reference: Siemens iX (MIT). No upstream JavaScript runtime is used.
use dioxus::prelude::*;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

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
    /// Spinner diameter in pixels.
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
    icon: Option<String>,
    icon_right: Option<String>,
    href: Option<String>,
    #[props(default = "_self".to_string())] target: String,
    rel: Option<String>,
    form: Option<String>,
    aria_label: Option<String>,
    aria_describedby: Option<String>,
    #[props(default)] class: String,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let blocked = disabled || loading;
    let class = format!(
        "ix-button ix-button-standard ix-button-{} ix-align-{} {}",
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
    if let Some(href) = href {
        rsx! { a {
            class, role: "button", href: if blocked { None } else { Some(href) },
            target, rel: rel.unwrap_or_else(|| "noopener noreferrer".into()),
            aria_label, aria_describedby, aria_disabled: blocked, aria_busy: loading,
            tabindex: if blocked { -1 } else { 0 },
            onclick: move |event| { if blocked { event.prevent_default(); event.stop_propagation(); } else if let Some(handler) = onclick { handler.call(event); } },
            {content}
        } }
    } else {
        rsx! { button {
            class, r#type: button_type.as_str(), form, disabled: blocked,
            tabindex: if blocked { -1 } else { 0 },
            aria_label, aria_describedby, aria_disabled: blocked, aria_busy: loading,
            onclick: move |event| { if !blocked { if let Some(handler) = onclick { handler.call(event); } } },
            {content}
        } }
    }
}

/// An icon-only action with an explicitly required accessible name.
#[component]
pub fn IconButton(
    icon: String,
    label: String,
    #[props(default = ButtonVariant::SubtlePrimary)] variant: ButtonVariant,
    #[props(default)] size: ButtonIconSize,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default)] oval: bool,
    #[props(default)] button_type: ButtonType,
    #[props(default)] class: String,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let blocked = disabled || loading;
    rsx! { button {
        class: "ix-button ix-icon-button ix-button-{variant.as_str()} ix-icon-size-{size.as_str()} {class}",
        class: if oval { "ix-oval" }, r#type: button_type.as_str(),
        disabled: blocked, aria_label: label.clone(), title: label, aria_busy: loading,
        onclick: move |event| { if !blocked { if let Some(handler) = onclick { handler.call(event); } } },
        if loading { Spinner { size: match size { ButtonIconSize::Twelve => SpinnerSize::Xxs, ButtonIconSize::Sixteen => SpinnerSize::Xs, _ => SpinnerSize::Sm }, hide_track: true } }
        else { crate::Icon { name: icon, size: size.pixels() } }
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
#[component]
pub fn ToggleButton(
    children: Element,
    #[props(default)] pressed: bool,
    #[props(default = ButtonVariant::SubtlePrimary)] variant: ButtonVariant,
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
        class: "ix-button ix-button-{variant.as_str()} {class}", class: if pressed { "ix-selected" },
        r#type: "button", disabled: blocked, aria_pressed: pressed, aria_busy: loading, aria_label,
        onclick: move |_| { if !blocked { onpressedchange.call(!pressed); } },
        if loading { Spinner { size: SpinnerSize::Sm, hide_track: true } }
        else if let Some(name) = icon { crate::Icon { name, size: 20 } }
        span { class: "ix-button-content", {children} }
        if let Some(name) = icon_right { crate::Icon { name, size: 20 } }
    } }
}

/// Icon toggle; the accessible label stays constant as its pressed state changes.
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
    onpressedchange: EventHandler<bool>,
) -> Element {
    let blocked = disabled || loading;
    rsx! { button {
        class: "ix-button ix-icon-button ix-button-{variant.as_str()} ix-icon-size-{size.as_str()}",
        class: if pressed { "ix-selected" }, class: if oval { "ix-oval" },
        r#type: "button", disabled: blocked, aria_pressed: pressed, aria_label: label.clone(),
        title: label, aria_busy: loading,
        onclick: move |_| { if !blocked { onpressedchange.call(!pressed); } },
        if loading { Spinner { size: SpinnerSize::Sm, hide_track: true } }
        else { crate::Icon { name: icon, size: size.pixels() } }
    } }
}

/// An alternative action in a split button. `value` is returned to `onselect`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitButtonItem {
    /// Stable action identifier.
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
action_enum!(SplitCloseBehavior, Both, Both => "both", Inside => "inside", Outside => "outside", None => "none");

/// Primary action plus a keyboard-accessible menu of alternative actions.
#[component]
pub fn SplitButton(
    label: String,
    items: Vec<SplitButtonItem>,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] disable_button: bool,
    #[props(default)] disable_dropdown_button: bool,
    #[props(default)] close_behavior: SplitCloseBehavior,
    icon: Option<String>,
    #[props(default = "chevron-down-small".to_string())] split_icon: String,
    #[props(default = "More actions".to_string())] dropdown_label: String,
    onclick: Option<EventHandler<MouseEvent>>,
    onselect: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let mut focused = use_signal(|| 0usize);
    let item_refs = use_hook(|| Rc::new(RefCell::new(BTreeMap::<usize, Rc<MountedData>>::new())));
    let trigger_ref = use_hook(|| Rc::new(RefCell::new(None::<Rc<MountedData>>)));
    let register_item = {
        let item_refs = item_refs.clone();
        use_callback(move |(index, element): (usize, Rc<MountedData>)| {
            item_refs.borrow_mut().insert(index, element);
        })
    };
    let focus_item = use_callback(move |index: usize| {
        focused.set(index);
        let element = item_refs.borrow().get(&index).cloned();
        if let Some(element) = element {
            spawn(async move {
                let _ = element.set_focus(true).await;
            });
        }
    });
    let close_menu = {
        let trigger_ref = trigger_ref.clone();
        use_callback(move |_| {
            open.set(false);
            let element = trigger_ref.borrow().clone();
            if let Some(element) = element {
                spawn(async move {
                    let _ = element.set_focus(true).await;
                });
            }
        })
    };
    let enabled_indices: Vec<_> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| (!item.disabled).then_some(index))
        .collect();
    let first = enabled_indices.first().copied().unwrap_or(0);
    let last = enabled_indices.last().copied().unwrap_or(0);
    let close_inside = matches!(
        close_behavior,
        SplitCloseBehavior::Both | SplitCloseBehavior::Inside
    );
    let close_outside = matches!(
        close_behavior,
        SplitCloseBehavior::Both | SplitCloseBehavior::Outside
    );
    rsx! { div {
        class: "ix-split-button",
        onkeydown: move |event| { if event.key() == Key::Escape { close_menu.call(()); event.stop_propagation(); } },
        if label.is_empty() && icon.is_some() {
            IconButton { variant, disabled: disabled || disable_button, icon: icon.clone().unwrap_or_default(), label: icon.as_deref().unwrap_or("Action").replace('-', " "),
                onclick: move |event| { if let Some(handler) = onclick { handler.call(event); } } }
        } else {
            Button { variant, disabled: disabled || disable_button, icon, onclick: move |event| { if let Some(handler) = onclick { handler.call(event); } }, "{label}" }
        }
        button {
            class: "ix-button ix-icon-button ix-split-trigger ix-button-{variant.as_str()}",
            r#type: "button", disabled: disabled || disable_dropdown_button,
            aria_label: dropdown_label, aria_haspopup: "menu", aria_expanded: open(),
            onmounted: move |event| { *trigger_ref.borrow_mut() = Some(event.data()); },
            onclick: move |_| { focused.set(first); open.toggle(); },
            onkeydown: move |event| {
                match event.key() {
                    Key::ArrowDown => { event.prevent_default(); focused.set(first); open.set(true); },
                    Key::ArrowUp => { event.prevent_default(); focused.set(last); open.set(true); },
                    _ => {}
                }
            },
            crate::Icon { name: split_icon, size: 20 }
        }
        if open() && !disabled {
            if close_outside { button { class: "ix-split-backdrop", r#type: "button", tabindex: -1, aria_label: "Close actions", onclick: move |_| close_menu.call(()) } }
            div { class: "ix-split-menu", role: "menu", aria_label: "Alternative actions",
                onkeydown: move |event| {
                    let target = match event.key() {
                        Key::Home => Some(first), Key::End => Some(last),
                        Key::ArrowDown => next_split_index(&enabled_indices, focused(), true),
                        Key::ArrowUp => next_split_index(&enabled_indices, focused(), false),
                        _ => None,
                    };
                    if let Some(target) = target { event.prevent_default(); focus_item.call(target); }
                },
                for (index, item) in items.into_iter().enumerate() {
                    button {
                        key: "{item.value}", class: "ix-split-item", role: "menuitem", r#type: "button", disabled: item.disabled,
                        aria_label: if item.label.is_empty() { item.icon.as_ref().map(|name| name.replace('-', " ")) } else { None },
                        tabindex: if index == focused() { 0 } else { -1 },
                        onmounted: move |event| { register_item.call((index, event.data())); if index == focused() { focus_item.call(index); } },
                        onfocus: move |_| focused.set(index),
                        onclick: move |_| { onselect.call(item.value.clone()); if close_inside { close_menu.call(()); } },
                        if let Some(name) = item.icon.clone() { crate::Icon { name, size: 20 } }
                        "{item.label}"
                    }
                }
            }
        }
    } }
}

fn next_split_index(enabled: &[usize], current: usize, forward: bool) -> Option<usize> {
    if enabled.is_empty() {
        return None;
    }
    let position = enabled
        .iter()
        .position(|index| *index == current)
        .unwrap_or(0);
    let next = if forward {
        (position + 1) % enabled.len()
    } else {
        (position + enabled.len() - 1) % enabled.len()
    };
    Some(enabled[next])
}

/// Animated busy state, automatically respecting reduced-motion preferences.
#[component]
pub fn Spinner(
    #[props(default)] size: SpinnerSize,
    #[props(default)] variant: SpinnerVariant,
    #[props(default)] hide_track: bool,
    #[props(default = "Loading".to_string())] label: String,
) -> Element {
    rsx! { span { class: "ix-spinner ix-spinner-{variant.as_str()}",
        class: if hide_track { "ix-spinner-no-track" },
        style: "--ix-spinner-size:{size.pixels()}px", role: "status", aria_busy: true, aria_label: label,
        svg { view_box: "0 0 32 32", "aria-hidden": "true",
            circle { class: "ix-spinner-track", cx: "16", cy: "16", r: "13" }
            circle { class: "ix-spinner-arc", cx: "16", cy: "16", r: "13" }
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
        ProgressStatus::Paused => Some("pause"),
        _ => None,
    };
    let title = if show_text_as_tooltip {
        helper_text.clone()
    } else {
        None
    };
    let accessible_label = label.clone().unwrap_or_else(|| "Progress".into());
    rsx! { div {
        class: "ix-progress ix-progress-{progress_type.as_str()} ix-progress-{size.as_str()} ix-progress-status-{status.as_str()}",
        style: "text-align:{text_alignment.as_str()};--ix-progress-size:{size.pixels()}px", title,
        if let Some(label) = label { div { class: "ix-progress-label", "{label}" } }
        div { class: "ix-progress-content",
            if progress_type == ProgressType::Linear {
                div { class: "ix-progress-track", role: "progressbar", aria_label: accessible_label,
                    aria_valuemin: "0", aria_valuemax: "100", aria_valuenow: percent,
                    div { class: "ix-progress-fill", style: "width:{percent}%" }
                }
                if let Some(children) = children { div { class: "ix-progress-value", {children} } }
            } else {
                span { class: "ix-progress-circle", role: "progressbar", aria_label: accessible_label,
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
                "{helper}"
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
        role: heading.map(|_| "heading"), aria_level: heading,
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
.ix-theme .ix-spinner{display:inline-flex;flex-shrink:0;width:var(--ix-spinner-size);height:var(--ix-spinner-size);color:currentColor;vertical-align:middle}.ix-theme .ix-spinner-primary{color:var(--ix-color-primary)}.ix-theme .ix-spinner svg{width:100%;height:100%;animation:ix-spinner-turn 1s linear infinite}.ix-theme .ix-spinner circle{fill:none;stroke:currentColor;stroke-width:3}.ix-theme .ix-spinner-track{opacity:.2}.ix-theme .ix-spinner-no-track .ix-spinner-track{opacity:0}.ix-theme .ix-spinner-arc{stroke-dasharray:24 82;stroke-linecap:round}@keyframes ix-spinner-turn{to{transform:rotate(360deg)}}
.ix-theme .ix-status-primary{--ix-status-bg:var(--ix-color-primary);--ix-status-fg:var(--ix-color-on-primary)}.ix-theme .ix-status-alarm,.ix-theme .ix-status-error{--ix-status-bg:var(--ix-color-danger);--ix-status-fg:var(--ix-color-on-danger,#fff)}.ix-theme .ix-status-critical{--ix-status-bg:var(--ix-color-critical,#f89a00);--ix-status-fg:var(--ix-color-on-critical)}.ix-theme .ix-status-warning{--ix-status-bg:var(--ix-color-warning);--ix-status-fg:var(--ix-color-on-warning,#fff)}.ix-theme .ix-status-info{--ix-status-bg:var(--ix-color-info);--ix-status-fg:var(--ix-color-on-info,#fff)}.ix-theme .ix-status-neutral{--ix-status-bg:var(--ix-color-surface-2);--ix-status-fg:var(--ix-color-text)}.ix-theme .ix-status-success{--ix-status-bg:var(--ix-color-success);--ix-status-fg:var(--ix-color-on-success,#fff)}.ix-theme .ix-status-custom{--ix-status-bg:var(--ix-color-surface-2);--ix-status-fg:var(--ix-color-text)}
.ix-theme .ix-pill,.ix-theme .ix-chip,.ix-theme .ix-filter-chip{box-sizing:border-box;display:inline-flex;align-items:center;gap:6px;max-width:100%;vertical-align:middle;font:14px/20px var(--ix-font);border:1px solid transparent;border-radius:16px;overflow:hidden}.ix-theme .ix-pill{height:20px;min-height:20px;padding:0 8px;line-height:18px;justify-content:center;background:var(--ix-status-bg);color:var(--ix-status-fg)}.ix-theme .ix-status-label{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-outline{border-color:var(--ix-status-bg);color:var(--ix-status-bg);background:transparent}.ix-theme .ix-chip{min-height:32px;gap:0;background:var(--ix-status-bg);color:var(--ix-status-fg)}.ix-theme .ix-chip.ix-outline{background:transparent;color:var(--ix-status-bg)}.ix-theme .ix-chip button{font:inherit;color:inherit;background:transparent;border:0;cursor:pointer}.ix-theme .ix-chip-main{display:inline-flex;align-items:center;gap:6px;padding:5px 12px;min-width:0;min-height:30px;flex:1}.ix-theme .ix-chip-main:disabled{cursor:default}.ix-theme .ix-chip-close{display:inline-flex;align-items:center;justify-content:center;width:28px;height:28px;margin-right:1px;flex-shrink:0;border-radius:50%}.ix-theme .ix-chip button:hover:not(:disabled){background:color-mix(in srgb,currentColor 12%,transparent)}.ix-theme .ix-center-content{justify-content:center}.ix-theme .ix-status-custom.ix-outline{color:var(--ix-status-fg)}.ix-theme .ix-badge.ix-status-custom.ix-outline .ix-badge-indicator{color:var(--ix-status-fg)}
.ix-theme .ix-filter-chip{background:var(--ix-color-surface-2);color:var(--ix-color-text);padding:3px 4px 3px 12px;min-height:32px}.ix-theme .ix-filter-chip.ix-disabled{opacity:.5}.ix-theme .ix-filter-close{display:inline-flex;align-items:center;justify-content:center;background:transparent;color:inherit;border:0;border-radius:50%;width:24px;height:24px;cursor:pointer}.ix-theme .ix-filter-close:hover{background:var(--ix-color-surface-1)}
.ix-theme .ix-badge{position:relative;display:inline-flex;vertical-align:middle}.ix-theme .ix-badge-indicator{box-sizing:border-box;display:inline-flex;align-items:center;justify-content:center;gap:4px;min-width:20px;height:20px;padding:0 4px;border-radius:10px;background:var(--ix-status-bg);color:var(--ix-status-fg);font:700 14px/20px var(--ix-font);white-space:nowrap;max-width:var(--ix-badge-max-width,160px);overflow:hidden;text-overflow:ellipsis}.ix-theme .ix-badge-label:not(.ix-badge-attached) .ix-badge-indicator{width:100%}.ix-theme .ix-badge-label-text{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-badge-label .ix-badge-indicator{height:20px;line-height:20px;border-radius:10px;padding:0 8px;font-weight:400}.ix-theme .ix-badge-dot .ix-badge-indicator{min-width:12px;width:12px;height:12px;padding:0;border-radius:50%}.ix-theme .ix-badge-status-icon .ix-badge-indicator{min-width:20px;width:20px;height:20px;padding:0;border-radius:0;background:transparent;color:var(--ix-status-bg);overflow:visible}.ix-theme .ix-badge-status-stack{position:relative;display:inline-flex;width:20px;height:20px}.ix-theme .ix-badge-status-plate,.ix-theme .ix-badge-status-glyph{position:absolute;inset:0}.ix-theme .ix-badge-status-plate{color:var(--ix-status-fg)}.ix-theme .ix-badge.ix-outline.ix-badge-status-icon .ix-badge-indicator{background:transparent;border:0}.ix-theme .ix-badge.ix-outline .ix-badge-indicator{border:1px solid var(--ix-status-bg);background:var(--ix-color-surface);color:var(--ix-status-bg)}.ix-theme .ix-badge-attached .ix-badge-indicator{position:absolute;right:var(--ix-badge-x);top:var(--ix-badge-y)}.ix-theme .ix-badge-bottom-after .ix-badge-indicator{top:auto;bottom:var(--ix-badge-y)}.ix-theme .ix-badge-border .ix-badge-indicator{outline:2px solid var(--ix-color-surface)}.ix-theme .ix-badge-animated .ix-badge-indicator{position:relative;overflow:visible;isolation:isolate}.ix-theme .ix-badge-attached.ix-badge-animated .ix-badge-indicator{position:absolute}.ix-theme .ix-badge-animated .ix-badge-indicator::after{content:"";position:absolute;inset:.5px;z-index:-1;pointer-events:none;box-sizing:border-box;border:0 solid var(--ix-status-bg);border-radius:inherit;opacity:0;animation:ix-badge-pulse-ring var(--ix-badge-animation-duration,2s) linear infinite}.ix-theme .ix-badge-status-icon.ix-badge-animated .ix-badge-indicator::after{inset:.4px;border:0;background:var(--ix-status-bg);border-radius:50%;transform:scale(.521);animation-name:ix-badge-pulse-status}@keyframes ix-badge-pulse-ring{0%,100%{inset:.5px;border-width:0;opacity:0}5%{inset:.5px;border-width:0;opacity:.5}45%{opacity:.4}80%{inset:-7.5px;border-width:8px;opacity:.1;animation-timing-function:ease-out}90%{inset:-9.5px;border-width:10px;opacity:0}}@keyframes ix-badge-pulse-status{0%,100%{transform:scale(.521);opacity:0}5%{transform:scale(.521);opacity:.5;animation-timing-function:cubic-bezier(.637,.43,.928,1.008)}45%{transform:scale(1);opacity:.4}80%{transform:scale(1.6);opacity:.1;animation-timing-function:ease-out}90%{transform:scale(1.8);opacity:0}}
.ix-theme .ix-avatar{display:inline-flex;align-items:center;justify-content:center;overflow:hidden;flex-shrink:0;vertical-align:middle;width:var(--ix-avatar-size);height:var(--ix-avatar-size);border-radius:50%;background:var(--ix-color-surface-2);color:var(--ix-color-text);font:600 14px/1 var(--ix-font)}.ix-theme .ix-avatar img{width:100%;height:100%;object-fit:cover}.ix-theme .ix-divider{height:1px;background:var(--ix-color-border);margin:16px 0;flex-shrink:0}.ix-theme .ix-divider-vertical{height:auto;min-height:24px;width:1px;margin:0 16px;align-self:stretch}
.ix-theme .ix-progress{--ix-progress-color:var(--ix-color-primary);color:var(--ix-color-text);min-width:0}.ix-theme .ix-progress-status-error{--ix-progress-color:var(--ix-color-danger-text)}.ix-theme .ix-progress-status-success{--ix-progress-color:var(--ix-color-success)}.ix-theme .ix-progress-status-warning{--ix-progress-color:var(--ix-color-warning-text)}.ix-theme .ix-progress-status-info{--ix-progress-color:var(--ix-color-info-text)}.ix-theme .ix-progress-status-paused{--ix-progress-color:var(--ix-color-text-secondary)}.ix-theme .ix-progress-label{color:var(--ix-color-text-secondary);margin-bottom:8px;font-size:14px}.ix-theme .ix-progress-content{display:flex;align-items:center;gap:12px}.ix-theme .ix-progress-track{height:8px;flex:1;overflow:hidden;background:var(--ix-color-surface-2);border-radius:2px}.ix-theme .ix-progress-xs .ix-progress-track{height:2px}.ix-theme .ix-progress-sm .ix-progress-track{height:4px}.ix-theme .ix-progress-lg .ix-progress-track{height:16px}.ix-theme .ix-progress-xl .ix-progress-track{height:24px}.ix-theme .ix-progress-fill{height:100%;background:var(--ix-progress-color);transition:width .2s ease-out}.ix-theme .ix-progress-value{font-size:14px;white-space:nowrap;font-variant-numeric:tabular-nums}.ix-theme .ix-progress-helper{display:flex;align-items:center;gap:6px;margin-top:8px;font-size:12px;color:var(--ix-color-text-secondary)}.ix-theme .ix-progress-status-error .ix-progress-helper{color:var(--ix-color-danger-text)}.ix-theme .ix-progress-circle{position:relative;display:inline-flex;width:var(--ix-progress-size);height:var(--ix-progress-size);flex-shrink:0}.ix-theme .ix-progress-circle svg{width:100%;height:100%;transform:rotate(-90deg)}.ix-theme .ix-progress-circle circle{fill:none;stroke-width:3}.ix-theme .ix-progress-circle-track{stroke:var(--ix-color-surface-2)}.ix-theme .ix-progress-circle-fill{stroke:var(--ix-progress-color);transition:stroke-dashoffset .2s ease-out}.ix-theme .ix-progress-circle-value{position:absolute;inset:0;display:flex;align-items:center;justify-content:center;font-size:12px;font-variant-numeric:tabular-nums}.ix-theme .ix-progress-circular .ix-progress-content{justify-content:inherit}
.ix-theme .ix-typography{display:block;position:relative;font:var(--si-sys-typography-body,14px/20px var(--ix-font));margin:0;overflow-wrap:break-word}.ix-theme .ix-type-body{font:var(--si-sys-typography-body,14px/20px var(--ix-font))}.ix-theme .ix-type-body-sbold{font:var(--si-sys-typography-body-sbold,600 14px/20px var(--ix-font))}.ix-theme .ix-type-body-paragraph{font:var(--si-sys-typography-body-paragraph,14px/24px var(--ix-font))}.ix-theme .ix-type-body-paragraph-sbold{font:var(--si-sys-typography-body-paragraph-sbold,600 14px/24px var(--ix-font))}.ix-theme .ix-type-body-sm{font:var(--si-sys-typography-body-sm,12px/16px var(--ix-font))}.ix-theme .ix-type-body-sm-sbold{font:var(--si-sys-typography-body-sm-sbold,600 12px/16px var(--ix-font))}.ix-theme .ix-type-body-lg{font:var(--si-sys-typography-body-lg,16px/24px var(--ix-font))}.ix-theme .ix-type-body-lg-sbold{font:var(--si-sys-typography-body-lg-sbold,600 16px/24px var(--ix-font))}.ix-theme .ix-type-display{font:var(--si-sys-typography-display,40px/48px var(--ix-font))}.ix-theme .ix-type-display-sbold{font:var(--si-sys-typography-display-sbold,600 40px/48px var(--ix-font))}.ix-theme .ix-type-display-lg{font:var(--si-sys-typography-display-lg,48px/56px var(--ix-font))}.ix-theme .ix-type-display-lg-sbold{font:var(--si-sys-typography-display-lg-sbold,600 48px/56px var(--ix-font))}.ix-theme .ix-type-display-xl{font:var(--si-sys-typography-display-xl,56px/64px var(--ix-font))}.ix-theme .ix-type-display-xl-sbold{font:var(--si-sys-typography-display-xl-sbold,600 56px/64px var(--ix-font))}.ix-theme .ix-type-display-xxl{font:var(--si-sys-typography-display-xxl,64px/72px var(--ix-font))}.ix-theme .ix-type-display-xxl-sbold{font:var(--si-sys-typography-display-xxl-sbold,600 64px/72px var(--ix-font))}.ix-theme .ix-type-h1{font:var(--si-sys-typography-h1,40px/48px var(--ix-font))}.ix-theme .ix-type-h2{font:var(--si-sys-typography-h2,32px/40px var(--ix-font))}.ix-theme .ix-type-h3{font:var(--si-sys-typography-h3,24px/32px var(--ix-font))}.ix-theme .ix-type-h4{font:var(--si-sys-typography-h4,20px/28px var(--ix-font))}.ix-theme .ix-type-h5{font:var(--si-sys-typography-h5,16px/24px var(--ix-font))}.ix-theme .ix-type-h5-bold{font:var(--si-sys-typography-h5-bold,600 16px/24px var(--ix-font))}.ix-theme .ix-type-h6{font:var(--si-sys-typography-h6,600 14px/20px var(--ix-font))}.ix-theme .ix-type-code{font:var(--si-sys-typography-code,14px/20px monospace)}.ix-theme .ix-type-code-sm{font:var(--si-sys-typography-code-sm,12px/16px monospace)}.ix-theme .ix-type-code-lg{font:var(--si-sys-typography-code-lg,16px/24px monospace)}.ix-theme .ix-typography{font-feature-settings:"clig" off,"liga" off;font-style:normal;letter-spacing:var(--si-ref-typography-letter-spacing-normal,normal);-webkit-font-smoothing:antialiased}.ix-theme .ix-type-bold{font-weight:var(--ix-typography-weight-bold--font-weight,var(--si-ref-typography-font-weight-bold,700))!important}.ix-theme .ix-text-std,.ix-theme .ix-text-contrast{color:var(--si-sys-color-text-primary,var(--ix-color-text))}.ix-theme .ix-text-soft{color:var(--si-sys-color-text-secondary,var(--ix-color-text-secondary))}.ix-theme .ix-text-weak{color:var(--si-sys-color-text-disabled,var(--ix-color-text-disabled,var(--ix-color-text-secondary)))}.ix-theme .ix-text-alarm{color:var(--si-sys-color-text-danger,var(--ix-color-danger-text))}.ix-theme .ix-text-alarm-contrast{color:var(--si-sys-color-text-on-danger,var(--ix-color-on-danger,#fff))}.ix-theme .ix-text-critical-contrast{color:var(--si-sys-color-text-on-critical,var(--ix-color-on-critical,#000028))}.ix-theme .ix-text-info-contrast{color:var(--si-sys-color-text-on-information,var(--ix-color-on-info,#fff))}.ix-theme .ix-text-inv-contrast,.ix-theme .ix-text-inv-soft,.ix-theme .ix-text-inv-std,.ix-theme .ix-text-inv-weak,.ix-theme .ix-text-inverse{color:var(--si-sys-color-text-inverse,var(--ix-color-on-primary))}.ix-theme .ix-text-neutral-contrast{color:var(--si-sys-color-text-on-neutral,var(--ix-color-text))}.ix-theme .ix-text-primary-contrast{color:var(--si-sys-color-text-on-accent,var(--ix-color-on-primary))}.ix-theme .ix-text-success-contrast{color:var(--si-sys-color-text-on-success,var(--ix-color-on-success,#fff))}.ix-theme .ix-text-warning-contrast{color:var(--si-sys-color-text-on-warning,var(--ix-color-on-warning,#000028))}
@media(prefers-reduced-motion:reduce){.ix-theme .ix-spinner svg{animation-duration:3s}.ix-theme .ix-badge-animated .ix-badge-indicator::after{animation:none;opacity:0}.ix-theme .ix-button,.ix-theme .ix-progress-fill,.ix-theme .ix-progress-circle-fill{transition:none}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
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
    fn split_menu_focus_wraps_and_skips_disabled_entries() {
        let enabled = [0, 2, 4];
        assert_eq!(next_split_index(&enabled, 0, true), Some(2));
        assert_eq!(next_split_index(&enabled, 4, true), Some(0));
        assert_eq!(next_split_index(&enabled, 0, false), Some(4));
        assert_eq!(next_split_index(&[], 0, true), None);
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
}

#[cfg(test)]
#[path = "button_token_tests.rs"]
mod button_token_tests;
