//! Application account actions composed from the shared persistent Dropdown.
use crate::{
    Dropdown, DropdownChangeEvent, DropdownItemOptions, DropdownNavigationMode, DropdownPlacement,
    Icon, TooltipFor, TooltipPlacement, use_dropdown, use_dropdown_item_with_options,
    use_dropdown_trigger_binding, use_tooltip, use_tooltip_anchor,
};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(1);

/// Account trigger and persistent user actions, including optional logout.
/// Composed children retain their state when the menu closes. Native custom
/// controls register with `use_dropdown_item`; editable fields keep their own
/// editing keys. No surrounding DOM is discovered.
///
/// Placement defaults above the trigger for compatibility with the original
/// Rust sidebar. `tooltip_text` falls back to `top`; an explicit empty string
/// suppresses it. Opening the menu dismisses the shared styled tooltip.
#[component]
pub fn MenuAvatar(
    #[props(default = "User".to_string())] top: String,
    #[props(default)] bottom: Option<String>,
    #[props(default)] initials: Option<String>,
    #[props(default)] image: Option<String>,
    #[props(default)] children: Option<Element>,
    #[props(default)] hide_logout_button: bool,
    #[props(default = "Logout".to_string())] i18n_logout: String,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] tooltip_text: Option<String>,
    #[props(default)] aria_label_tooltip: Option<String>,
    #[props(default = DropdownPlacement::TopStart)] placement: DropdownPlacement,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] onbeforechange: EventHandler<DropdownChangeEvent>,
    #[props(default)] onchange: EventHandler<bool>,
    #[props(default)] onlogout: EventHandler<MouseEvent>,
) -> Element {
    let identity = use_hook(|| NEXT.fetch_add(1, Ordering::Relaxed));
    let trigger_id = format!("ix-menu-avatar-trigger-{identity}");
    let dropdown = use_dropdown();
    let trigger = use_dropdown_trigger_binding(dropdown);
    let open = dropdown.is_open();
    let actionable = children.is_some() || !hide_logout_button;
    let tooltip_id = format!("ix-menu-avatar-tooltip-{identity}");
    let tooltip = use_tooltip(tooltip_id.clone());
    // The stable binding identity is separate from the caller's optional ID;
    // measurement uses the actual Dioxus mounted wrapper, never an ID lookup.
    let anchor = use_tooltip_anchor(tooltip.clone(), format!("{tooltip_id}-anchor"));
    let text = tooltip_text.unwrap_or_else(|| top.clone());
    let show_tooltip = !open && !text.trim().is_empty();
    let description = show_tooltip.then(|| anchor.describedby("")).flatten();
    let enter = anchor.onmouseenter;
    let focus = anchor.onfocusin;
    let initials = initials.unwrap_or_else(|| {
        top.split_whitespace()
            .take(2)
            .filter_map(|word| word.chars().next())
            .collect()
    });
    rsx! {
        div {id,class:"ix-menu-avatar {class}",
            onmounted:anchor.onmounted,onvisible:anchor.onvisible,onresize:anchor.onresize,
            onmouseenter:move|event|if !dropdown.is_open(){enter.call(event)},onmouseleave:anchor.onmouseleave,
            onfocusin:move|event|if !dropdown.is_open(){focus.call(event)},onfocusout:anchor.onfocusout,onkeydown:anchor.onkeydown,
            button {id:trigger_id.clone(),class:"ix-menu-avatar-trigger",r#type:"button","aria-label":top.clone(),
                "aria-haspopup":actionable.then_some("menu"),"aria-expanded":(actionable&&open).to_string(),
                "aria-controls":actionable.then(||dropdown.menu_id()).flatten(),"aria-describedby":description,
                "aria-activedescendant":dropdown.active_descendant(),
                onmounted:trigger.onmounted,onvisible:trigger.onvisible,
                onkeydown:move|event|if actionable{trigger.onkeydown.call(event)},
                onclick:move|event|if actionable{trigger.onclick.call(event)},
                ..trigger.attributes(),
                span {class:"ix-menu-avatar-circle",if let Some(image)=image{img{src:image,alt:""}}else{"{initials}"}}
                span {class:"ix-menu-avatar-text",span{"{top}"}if let Some(bottom)=bottom{small{"{bottom}"}}}
            }
            Dropdown {controller:dropdown,trigger:trigger_id.clone(),label:"User menu",disabled:!actionable,
                placement,enable_top_layer,navigation_mode:DropdownNavigationMode::RovingTabindex,onbeforechange,onchange,class:"ix-menu-avatar-dropdown",
                {children}
                if !hide_logout_button {MenuAvatarItem{label:i18n_logout,icon:"log-out",onclick:onlogout}}
            }
            if show_tooltip {TooltipFor{id:tooltip_id,controller:tooltip,placement:TooltipPlacement::Right,aria_label:aria_label_tooltip,"{text}"}}
        }
    }
}

/// Native user-menu action. Accepted clicks close through the owning Dropdown;
/// calling `event.prevent_default()` in `onclick` keeps it open. Enter and Space
/// retain native button activation, while the shared item binding owns arrows,
/// focus registration, tab order and cleanup.
#[component]
pub fn MenuAvatarItem(
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    let identity = use_hook(|| NEXT.fetch_add(1, Ordering::Relaxed));
    let id = format!("ix-menu-avatar-item-{identity}");
    let item = use_dropdown_item_with_options(
        id.clone(),
        DropdownItemOptions {
            disabled,
            ..Default::default()
        },
    );
    rsx! {button{id,class:"ix-menu-avatar-item",r#type:"button",role:"menuitem",disabled,
        tabindex:item.tabindex,onmounted:item.onmounted,onkeydown:item.onkeydown,
        onclick:move|event|{if !disabled{onclick.call(event);}},
        ..item.attributes(),
        if let Some(icon)=icon{Icon{name:icon,size:24}}"{label}"
    }}
}
