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

fn retain_native_action_attributes(attributes: &mut Vec<Attribute>, owned: &[&str]) {
    // Optional typed props remain authoritative even when their value is None.
    attributes.retain(|attribute| !owned.contains(&attribute.name));
}

/// A native button or styled anchor. Loading prevents activation just like disabled.
///
/// Regular buttons consume the source `--ix-button-*` sizing and state variables,
/// with `--si-sys-*` semantic tokens supplied by an enclosing [`crate::IxTheme`].
/// The `hover` and `active` classes expose the source visual-state aliases;
/// `active` locks the pressed appearance without introducing a toggle state.
/// Missing Brand values retain provisional public-source fallbacks.
/// Global attributes and unowned events apply to the native control. Its class,
/// id, navigation/form fields, native state, typed ARIA and click/mount handlers
/// remain controlled by props, even when an optional prop is omitted.
#[component]
pub fn Button(
    children: Element,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    /// Suspend interaction without disabled styling; false removes native inert.
    #[props(default)]
    inert: bool,
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
    /// Native global metadata and events not owned by this component. Use typed
    /// props for owned attributes, including optional values; raw aliases cannot
    /// replace native state, classes, identity or internal activation handlers.
    #[props(extends = GlobalAttributes)]
    mut attributes: Vec<Attribute>,
) -> Element {
    retain_native_action_attributes(
        &mut attributes,
        &[
            "class",
            "id",
            "role",
            "href",
            "target",
            "rel",
            "form",
            "type",
            "disabled",
            "inert",
            "tabindex",
            "aria-disabled",
            "aria-busy",
            "aria-label",
            "aria-describedby",
            "onclick",
            "onmounted",
        ],
    );
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
            id: actual_id, inert: inert.then_some("true"),
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
            onclick: move |event| { if blocked || inert { event.prevent_default(); event.stop_propagation(); } else if let Some(handler) = onclick { handler.call(event); } },
            ..attributes,
            {content}
        } }
    } else {
        rsx! { button {
            id: actual_id, inert: inert.then_some("true"),
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
            onclick: move |event| { if inert { event.prevent_default(); event.stop_propagation(); } else if !blocked { if let Some(handler) = onclick { handler.call(event); } } },
            ..attributes,
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
/// Global attributes and unowned events apply to the button. Its class, id,
/// style, title/name, native state and typed events remain owned by props, even
/// when omitted. Use `icon_color` or `class` for styling instead of raw `style`.
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
    /// Suspend interaction without disabled styling; false removes native inert.
    #[props(default)]
    inert: bool,
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
    /// Native global metadata and events not owned by this component. Use typed
    /// props for owned attributes, including optional values; raw aliases cannot
    /// replace native state, classes, identity or internal activation handlers.
    #[props(extends = GlobalAttributes)]
    mut attributes: Vec<Attribute>,
) -> Element {
    retain_native_action_attributes(
        &mut attributes,
        &[
            "class",
            "id",
            "style",
            "title",
            "type",
            "disabled",
            "inert",
            "tabindex",
            "aria-disabled",
            "aria-busy",
            "aria-label",
            "aria-pressed",
            "aria-expanded",
            "aria-hidden",
            "onclick",
            "onmounted",
            "onkeydown",
            "onmousedown",
        ],
    );
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
        id: actual_id, inert: inert.then_some("true"),
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
        onclick: move |event| { if inert { event.prevent_default(); event.stop_propagation(); } else if !blocked { if let Some(handler) = onclick { handler.call(event); } } },
        ..native_events,
        ..attributes,
        if loading { Spinner { size: loading_size, hide_track: true } }
        else if !icon.is_empty() { crate::Icon { name: icon, size: size.pixels(), class: "ix-icon-button-glyph" } }
    } }
}

/// A native text link with disabled behavior and safe external-link defaults.
///
/// Omit `url` for a destination-free link. Native anchor keyboard behavior is
/// preserved; consumers can cancel navigation by calling `event.prevent_default()` in `onclick`.
/// Global metadata, ids, mount events and other unowned events reach the anchor.
/// Class, navigation fields, role/tabindex, disabled state and clicks remain
/// controlled by typed props, including omitted optional values.
#[component]
pub fn LinkButton(
    children: Element,
    url: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default = "_self".to_string())] target: String,
    #[props(default)] class: String,
    onclick: Option<EventHandler<MouseEvent>>,
    /// Native global metadata and events not owned by this component. Use typed
    /// props for owned attributes, including optional values; raw aliases cannot
    /// replace native state, classes, identity or internal activation handlers.
    #[props(extends = GlobalAttributes)]
    mut attributes: Vec<Attribute>,
) -> Element {
    retain_native_action_attributes(
        &mut attributes,
        &[
            "class",
            "href",
            "target",
            "rel",
            "role",
            "tabindex",
            "aria-disabled",
            "disabled",
            "onclick",
        ],
    );
    rsx! { a {
        class: "ix-link-button {class}", href: if disabled { None } else { url },
        target, rel: "noopener noreferrer", aria_disabled: disabled,
        role: disabled.then_some("link"), tabindex: disabled.then_some(-1),
        onclick: move |event| { if disabled { event.prevent_default(); event.stop_propagation(); } else if let Some(handler) = onclick { handler.call(event); } },
        ..attributes,
        crate::Icon {name:"chevron-right-small",size:16,class:"ix-link-button-icon"}
        span {class:"ix-link-button-label",{children}}
    } }
}

/// Controlled toggle button; update `pressed` from `onpressedchange`.
///
/// Supports the six current source variants and semantic selected-state tokens.
/// A click requests the inverse of the supplied `pressed` value; it never changes
/// that value internally. Disabled and loading controls do not emit requests.
/// Global metadata, ids, mount events and unowned events reach the native button;
/// typed classes, pressed/disabled/loading state, name and activation stay owned.
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
    /// Native global metadata and events not owned by this component. Use typed
    /// props for owned attributes, including optional values; raw aliases cannot
    /// replace native state, classes, identity or internal activation handlers.
    #[props(extends = GlobalAttributes)]
    mut attributes: Vec<Attribute>,
) -> Element {
    retain_native_action_attributes(
        &mut attributes,
        &[
            "class",
            "type",
            "disabled",
            "tabindex",
            "aria-pressed",
            "aria-disabled",
            "aria-busy",
            "aria-label",
            "onclick",
        ],
    );
    let blocked = disabled || loading;
    rsx! { button {
        class: "ix-button ix-button-tokenized ix-button-selectable ix-toggle-button-native ix-button-{variant.as_str()} {class}",
        class: if pressed { "ix-selected" },
        r#type: "button", disabled: blocked, tabindex: if blocked { -1 } else { 0 },
        aria_pressed: pressed, aria_disabled: blocked, aria_busy: loading, aria_label,
        onclick: move |_| { if !blocked { onpressedchange.call(!pressed); } },
        ..attributes,
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
/// Global metadata, ids, mount events and unowned events reach the native button;
/// typed classes, title/name, pressed/disabled/loading state and activation stay owned.
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
    /// Native global metadata and events not owned by this component. Use typed
    /// props for owned attributes, including optional values; raw aliases cannot
    /// replace native state, classes, identity or internal activation handlers.
    #[props(extends = GlobalAttributes)]
    mut attributes: Vec<Attribute>,
) -> Element {
    retain_native_action_attributes(
        &mut attributes,
        &[
            "class",
            "title",
            "type",
            "disabled",
            "tabindex",
            "aria-pressed",
            "aria-disabled",
            "aria-busy",
            "aria-label",
            "onclick",
        ],
    );
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
        ..attributes,
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

static NEXT_STATUS_HELP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

/// Read-only semantic status label. Nonblank `tooltip` supplies shared styled
/// help and makes the status focusable by default; `tabindex` can override it.
/// Pass explicit Rust text, including the visible label when desired. Existing
/// description tokens and an explicit accessible name remain caller-owned.
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
    #[props(default)] id: Option<String>,
    #[props(default)] tabindex: Option<i64>,
    #[props(default)] aria_describedby: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let anchor_id = use_hook(|| {
        format!(
            "ix-pill-{}",
            NEXT_STATUS_HELP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    });
    let tooltip_id = format!("{anchor_id}-tooltip");
    let controller = crate::use_tooltip(tooltip_id.clone());
    // Registration identity is stable; geometry comes from the mounted handle,
    // independently of a caller's current DOM id.
    let binding = crate::use_tooltip_anchor(controller.clone(), anchor_id.clone());
    let root_id = id.unwrap_or(anchor_id);
    let text = tooltip.filter(|text| !text.trim().is_empty());
    let observations = if text.is_some() {
        Vec::<Attribute>::new()
            .onresize(binding.onresize)
            .onvisible(binding.onvisible)
    } else {
        Vec::new()
    };
    let root_tabindex = tabindex.or_else(|| text.as_ref().map(|_| 0));
    let describedby = binding.describedby(aria_describedby.as_deref().unwrap_or_default());
    let custom = custom_status_style(variant, background.as_deref(), color.as_deref());
    rsx! { span {
        class: "ix-pill ix-status-{variant.as_str()} {class}", class: if outline { "ix-outline" },
        class: if align_left { "ix-align-start" }, style: custom,
        id:root_id,tabindex:root_tabindex,aria_describedby:describedby,
        onmounted:binding.onmounted,
        onmouseenter:binding.onmouseenter,onmouseleave:binding.onmouseleave,
        onfocusin:binding.onfocusin,onfocusout:binding.onfocusout,onkeydown:binding.onkeydown,
        role: aria_label.as_ref().map(|_| "img"), aria_label,
        ..observations,
        if let Some(name) = icon { crate::Icon { name, size: 16, label: aria_label_icon } }
        span { class: "ix-status-label", {children} }
        if let Some(text)=text {crate::TooltipFor {id:tooltip_id,controller,"{text}"}}
    } }
}

/// Interactive tag with an independently operable close button.
/// `inactive` disables only the main chip; the close button remains available.
/// Nonblank `tooltip` describes both buttons with shared styled help. The main
/// button's Dioxus mount supplies geometry; the outer tag adds no Tab stop.
/// `id` applies to that main button, and existing description tokens are kept.
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
    #[props(default)] id: Option<String>,
    #[props(default)] aria_describedby: Option<String>,
    #[props(default)] class: String,
    #[props(default = "Close chip".to_string())] close_label: String,
    onclick: Option<EventHandler<MouseEvent>>,
    onclose: Option<EventHandler<MouseEvent>>,
) -> Element {
    let anchor_id = use_hook(|| {
        format!(
            "ix-chip-{}",
            NEXT_STATUS_HELP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    });
    let tooltip_id = format!("{anchor_id}-tooltip");
    let controller = crate::use_tooltip(tooltip_id.clone());
    let binding = crate::use_tooltip_anchor(controller.clone(), anchor_id.clone());
    let main_id = id.unwrap_or(anchor_id);
    let text = tooltip.filter(|text| !text.trim().is_empty());
    let observations = if text.is_some() {
        Vec::<Attribute>::new()
            .onresize(binding.onresize)
            .onvisible(binding.onvisible)
    } else {
        Vec::new()
    };
    let describedby = binding.describedby(aria_describedby.as_deref().unwrap_or_default());
    let close_describedby = describedby.clone();
    rsx! { span {
        class: "ix-chip ix-status-{variant.as_str()} {class}", class: if outline { "ix-outline" },
        style: custom_status_style(variant, background.as_deref(), color.as_deref()),
        onmouseenter:binding.onmouseenter,onmouseleave:binding.onmouseleave,
        onfocusin:binding.onfocusin,onfocusout:binding.onfocusout,onkeydown:binding.onkeydown,
        button { class: "ix-chip-main", class: if center_content { "ix-center-content" },
            r#type: "button", disabled: inactive, aria_label,id:main_id,aria_describedby:describedby,
            onmounted:binding.onmounted,
            onclick: move |event| { if !inactive { if let Some(handler) = onclick { handler.call(event); } } },
            ..observations,
            if let Some(name) = icon { crate::Icon { name, size: 20, label: aria_label_icon } }
            span { class: "ix-status-label", {children} }
        }
        if closable { button {
            class: "ix-chip-close", r#type: "button", aria_label: close_label,aria_describedby:close_describedby,
            onclick: move |event| { event.stop_propagation(); if let Some(handler) = onclose { handler.call(event); } },
            crate::Icon { name: "close-small".to_string(), size: 16 }
        } }
        if let Some(text)=text {crate::TooltipFor {id:tooltip_id,controller,"{text}"}}
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
/// Attached indicators are decorative. Supply `description_id` to render a
/// description target, then include that ID in the actual anchor's own
/// `aria_describedby`; existing tokens and native attributes remain caller-owned.
/// `tooltip` uses the shared styled tooltip, including for attached badges.
/// Standalone badges can also derive help from their label with
/// `tooltip_from_label` or an explicitly empty `tooltip`. Help makes their root
/// keyboard focusable by default; `tabindex` overrides that choice. The focused
/// root owns naming and role, including an explicitly named leading icon unless
/// a full `aria_label` overrides the name.
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
    #[props(default)] tooltip_from_label: bool,
    #[props(default)] description_id: Option<String>,
    #[props(default)] description: Option<String>,
    #[props(default)] tabindex: Option<i64>,
    #[props(default)] aria_describedby: Option<String>,
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
    static NEXT_BADGE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    let root_id = use_hook(|| {
        format!(
            "ix-badge-root-{}",
            NEXT_BADGE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    });
    let tooltip_id = format!("{root_id}-tooltip");
    let tooltip_owner = crate::use_tooltip(tooltip_id.clone());
    let binding = crate::use_tooltip_anchor(tooltip_owner.clone(), root_id.clone());
    let explicit_tooltip = tooltip
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let tooltip_text = if attached {
        explicit_tooltip
    } else if show && (tooltip.is_some() || tooltip_from_label) {
        explicit_tooltip.or_else(|| {
            text.clone().or_else(|| {
                aria_label
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            })
        })
    } else {
        None
    };
    let root_tabindex = (!attached)
        .then_some(tabindex.or_else(|| tooltip_text.as_ref().map(|_| 0)))
        .flatten();
    let root_focusable = root_tabindex.is_some();
    let root_role = root_focusable.then(|| role.clone().unwrap_or_else(|| "status".into()));
    let accessible_name = aria_label
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            let name = text.clone().or_else(|| tooltip_text.clone());
            let icon_name = (badge_type == BadgeType::Label && show && icon.is_some())
                .then(|| {
                    aria_label_icon
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                })
                .flatten();
            match (icon_name, name) {
                (Some(icon), Some(name)) => Some(format!("{icon} {name}")),
                (Some(icon), None) => Some(icon.to_string()),
                (None, name) => name,
            }
        });
    let root_description = if root_focusable {
        binding.describedby(aria_describedby.as_deref().unwrap_or_default())
    } else {
        None
    };
    let attached_description = if attached {
        description_id
            .filter(|id| !id.trim().is_empty())
            .and_then(|id| {
                description
                    .filter(|value| !value.trim().is_empty())
                    .or_else(|| text.clone())
                    .map(|value| (id, value))
            })
    } else {
        None
    };
    rsx! { span {
        id: root_id, tabindex: root_tabindex,
        role: root_role,
        aria_label: root_focusable.then(|| accessible_name.clone()).flatten(),
        aria_describedby: root_description,
        onmounted: binding.onmounted, onresize: binding.onresize, onvisible: binding.onvisible,
        onmouseenter: binding.onmouseenter, onmouseleave: binding.onmouseleave,
        onfocusin: binding.onfocusin, onfocusout: binding.onfocusout, onkeydown: binding.onkeydown,
        class: "ix-badge ix-badge-{badge_type.as_str()} ix-status-{variant.as_str()} ix-badge-{position.as_str()} {class}",
        class: if attached { "ix-badge-attached" }, class: if outline { "ix-outline" },
        class: if border { "ix-badge-border" }, class: if enable_animation { "ix-badge-animated" },
        class: if align_left { "ix-align-start" },
        style: "{custom};--ix-badge-x:{offset_x - offset}px;--ix-badge-y:{offset_y - offset}px",
        if let Some(children) = children { {children} }
        if show { span {
            class: "ix-badge-indicator", role: if attached || root_focusable { None } else { Some(role.unwrap_or_else(|| "status".into())) },
            aria_label: if attached || root_focusable { None } else { aria_label }, aria_hidden: attached || root_focusable,
            aria_describedby: if attached || root_focusable { None } else { aria_describedby },
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
        if let Some((id, text)) = attached_description { span { id, class: "ix-sr-only", "{text}" } }
        if let Some(text) = tooltip_text {
            crate::TooltipFor { id: tooltip_id, controller: tooltip_owner, class: "ix-badge-tooltip", "{text}" }
        }
    } }
}

pub use crate::avatar::{Avatar, AvatarProps};

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
/// ranges render empty. Composed helpers render alongside `helper_text` or alone.
/// Tooltip mode uses the shared styled tooltip when `helper_text` is nonempty;
/// content-only helpers stay inline. Keep independent actions/inputs inline,
/// rather than inside the noninteractive tooltip. The root defaults to tabindex
/// -1; opt into ordinary Tab navigation or use its mounted handle when useful.
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
    #[props(default)] id: Option<String>,
    #[props(default = -1)] tabindex: i64,
    #[props(default)] onmounted: Option<EventHandler<MountedEvent>>,
    label: Option<String>,
    helper_text: Option<String>,
    /// Additional Rust helper content. Use inline mode for independent controls.
    #[props(default)]
    helper_content: Option<Element>,
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
    static NEXT_HELPER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    let helper_id = use_hook(|| {
        format!(
            "ix-progress-helper-{}",
            NEXT_HELPER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    });
    let tooltip = crate::use_tooltip(helper_id.clone());
    let binding = crate::use_tooltip_anchor(tooltip.clone(), format!("{helper_id}-anchor"));
    let tooltip_mode =
        show_text_as_tooltip && helper_text.as_ref().is_some_and(|text| !text.is_empty());
    let observations = if tooltip_mode {
        Vec::<Attribute>::new()
            .onresize(binding.onresize)
            .onvisible(binding.onvisible)
    } else {
        Vec::new()
    };
    let composed = helper_content.is_some();
    let description = (tooltip_mode || composed).then(|| helper_id.clone());
    let mount_tooltip = binding.onmounted;
    // Preserve the plain helper's existing markup, including an explicitly empty
    // string. With composed content, an empty string leaves the slot on its own.
    let inline_text = helper_text
        .clone()
        .filter(|text| !composed || !text.is_empty());
    let helper = if let Some(helper) = inline_text {
        rsx! { div { class: if composed { "ix-progress-helper ix-progress-helper-composed" } else { "ix-progress-helper" },
            if let Some(name) = status_icon { crate::Icon { name: name.to_string(), size: 16 } }
            span { class: "ix-progress-helper-text", "{helper}" }
            if let Some(content) = helper_content {
                div { id: (!tooltip_mode).then(|| helper_id.clone()), class: "ix-progress-helper-content", {content} }
            }
        } }
    } else if let Some(content) = helper_content {
        rsx! { div { id: helper_id.clone(), class: "ix-progress-helper-content", {content} } }
    } else {
        VNode::empty()
    };
    let accessible_label = label.clone().unwrap_or_else(|| "Progress".into());
    rsx! { div {
        class: "ix-progress ix-progress-{progress_type.as_str()} ix-progress-{size.as_str()} ix-progress-status-{status.as_str()} ix-progress-text-{text_alignment.as_str()} {class}",
        id, tabindex, "aria-describedby": description,
        style: "text-align:{text_alignment.as_str()};--ix-progress-size:{size.pixels()}px",
        onmounted: move |event: MountedEvent| { mount_tooltip.call(event.clone()); if let Some(handler) = onmounted { handler.call(event); } },
        onmouseenter: binding.onmouseenter, onmouseleave: binding.onmouseleave,
        onfocusin: binding.onfocusin, onfocusout: binding.onfocusout, onkeydown: binding.onkeydown,
        ..observations,
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
        if tooltip_mode {
            crate::TooltipFor { id: helper_id, controller: tooltip, placement: crate::TooltipPlacement::Bottom,
                show_delay: 500, class: "ix-progress-helper-tooltip", {helper}
            }
        } else { {helper} }
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

pub(crate) const STYLES: &str = include_str!("styles/actions.css");

#[cfg(test)]
mod tests;

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

#[cfg(test)]
#[path = "actions/link_button_tests.rs"]
mod link_button_tests;

#[cfg(test)]
#[path = "actions/badge_composition_tests.rs"]
mod badge_composition_tests;

#[cfg(test)]
#[path = "actions/status_help_tests.rs"]
mod status_help_tests;

#[cfg(test)]
mod inert_tests;

#[cfg(test)]
mod native_attribute_tests;
