//! Static profile faces and explicitly composed Dioxus profile menus.
use crate::{
    ButtonVariant, Divider, Dropdown, DropdownChangeEvent, DropdownNavigationMode,
    DropdownPlacement, DropdownTrigger, Icon, TooltipAnchorBinding, TooltipController, TooltipFor,
    use_dropdown, use_tooltip, use_tooltip_anchor,
};
use dioxus::prelude::*;

/// Profile image, initials, or generic user icon, with an explicitly opted-in
/// profile menu. Static rendering remains the default. `menu` or supplied
/// `children` enables the shared persistent Dropdown; children own their actions.
/// No surrounding Header or arbitrary DOM content is discovered. The menu
/// defaults to bottom-start placement, following the upstream Dropdown default.
///
/// Failed images fall back only for that source; a changed image can load
/// normally. The shared styled tooltip uses `tooltip` or username; an explicit
/// empty tooltip suppresses it. `aria_label_tooltip` optionally names that tooltip.
/// Opening the menu dismisses it until the trigger is hovered or focused again.
#[component]
pub fn Avatar(
    username: Option<String>,
    initials: Option<String>,
    image: Option<String>,
    #[props(default = 32)] size: u32,
    tooltip: Option<String>,
    #[props(default)] aria_label_tooltip: Option<String>,
    #[props(default)] menu: bool,
    #[props(default)] extra: Option<String>,
    #[props(default)] children: Option<Element>,
    #[props(default)] disabled: bool,
    #[props(default)] aria_label: Option<String>,
    #[props(default="User menu".to_string())] menu_label: String,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] enable_top_layer: bool,
    #[props(default=DropdownPlacement::BottomStart)] placement: DropdownPlacement,
    #[props(default)] navigation_mode: DropdownNavigationMode,
    #[props(default)] onbeforechange: EventHandler<DropdownChangeEvent>,
    #[props(default)] onchange: EventHandler<bool>,
) -> Element {
    let failed_source = use_signal(|| None::<String>);
    let label = aria_label
        .filter(|label| !label.trim().is_empty())
        .or_else(|| username.clone().filter(|name| !name.trim().is_empty()))
        .unwrap_or_else(|| "User profile".into());
    let text = tooltip
        .or_else(|| username.clone())
        .filter(|text| !text.trim().is_empty());
    if menu || children.is_some() {
        rsx! {AvatarMenu{username,initials,image,size,text,aria_label_tooltip,extra,children,disabled,label,menu_label,id,class,enable_top_layer,placement,navigation_mode,onbeforechange,onchange,failed_source}}
    } else {
        rsx! {AvatarStatic{image,initials,size,label,text,aria_label_tooltip,id,class,failed_source}}
    }
}

// A stable Rust binding identity is independent of the caller's optional DOM ID.
// Tooltip measurement uses the mounted handle rather than looking up that ID.
fn use_avatar_tooltip() -> (String, TooltipController, TooltipAnchorBinding) {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    let id = use_hook(|| {
        format!(
            "ix-avatar-tooltip-{}",
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    });
    let controller = use_tooltip(id.clone());
    let binding = use_tooltip_anchor(controller.clone(), format!("{id}-anchor"));
    (id, controller, binding)
}

#[component]
fn AvatarStatic(
    image: Option<String>,
    initials: Option<String>,
    size: u32,
    failed_source: Signal<Option<String>>,
    label: String,
    text: Option<String>,
    aria_label_tooltip: Option<String>,
    id: Option<String>,
    class: String,
) -> Element {
    let (tooltip_id, controller, binding) = use_avatar_tooltip();
    let description = text.as_ref().and_then(|_| binding.describedby(""));
    rsx! {
        span {id,class:"ix-avatar {class}",style:"--ix-avatar-size:{size}px",role:"img","aria-label":label,"aria-describedby":description,
            onmounted:binding.onmounted,onresize:binding.onresize,onvisible:binding.onvisible,
            onmouseenter:binding.onmouseenter,onmouseleave:binding.onmouseleave,
            onfocusin:binding.onfocusin,onfocusout:binding.onfocusout,onkeydown:binding.onkeydown,
            AvatarFaceContent {image,initials,failed_source}
        }
        if let Some(text)=text {TooltipFor {id:tooltip_id,controller,aria_label:aria_label_tooltip,"{text}"}}
    }
}

#[component]
fn AvatarFace(
    image: Option<String>,
    initials: Option<String>,
    size: u32,
    failed_source: Signal<Option<String>>,
    #[props(default)] decorative: bool,
) -> Element {
    rsx! {span{class:"ix-avatar",style:"--ix-avatar-size:{size}px",role:if decorative{"presentation"}else{"img"},"aria-hidden":decorative.then_some("true"),
        AvatarFaceContent {image,initials,failed_source}
    }}
}

#[component]
fn AvatarFaceContent(
    image: Option<String>,
    initials: Option<String>,
    mut failed_source: Signal<Option<String>>,
) -> Element {
    let visible_image = image.filter(|source| failed_source().as_ref() != Some(source));
    rsx! {
        if let Some(source)=visible_image{img{key:"{source}",src:source.clone(),alt:"",onerror:move |_|failed_source.set(Some(source.clone()))}}
        else if let Some(initials)=initials{span{"aria-hidden":"true","{initials}"}}
        else{Icon{name:"user",size:20}}
    }
}

#[component]
fn AvatarMenu(
    username: Option<String>,
    initials: Option<String>,
    image: Option<String>,
    size: u32,
    text: Option<String>,
    aria_label_tooltip: Option<String>,
    extra: Option<String>,
    children: Option<Element>,
    disabled: bool,
    label: String,
    menu_label: String,
    id: Option<String>,
    class: String,
    enable_top_layer: bool,
    placement: DropdownPlacement,
    navigation_mode: DropdownNavigationMode,
    onbeforechange: EventHandler<DropdownChangeEvent>,
    onchange: EventHandler<bool>,
    failed_source: Signal<Option<String>>,
) -> Element {
    let dropdown = use_dropdown();
    let open = dropdown.is_open();
    let (tooltip_id, controller, binding) = use_avatar_tooltip();
    let description = (!open && text.is_some())
        .then(|| binding.describedby(""))
        .flatten();
    let enter = binding.onmouseenter;
    let focus = binding.onfocusin;
    let profile = username.filter(|name| !name.trim().is_empty());
    let divider = profile.is_some() && children.is_some();
    rsx! {div{id,class:"ix-avatar-menu {class}",style:"--ix-avatar-size:{size}px",
        onmounted:binding.onmounted,onresize:binding.onresize,onvisible:binding.onvisible,
        onmouseenter:move|event|if !dropdown.is_open(){enter.call(event)},onmouseleave:binding.onmouseleave,
        onfocusin:move|event|if !dropdown.is_open(){focus.call(event)},onfocusout:binding.onfocusout,onkeydown:binding.onkeydown,
        DropdownTrigger{dropdown,disabled,aria_label:label,aria_describedby:description,variant:ButtonVariant::SubtleTertiary,class:"ix-avatar-trigger",
            AvatarFace{image:image.clone(),initials:initials.clone(),size,failed_source,decorative:true}
        }
        Dropdown{controller:dropdown,label:menu_label,disabled,placement,navigation_mode,enable_top_layer,onbeforechange,onchange,class:"ix-avatar-menu-surface",
            if let Some(username)=profile{div{class:"ix-avatar-profile",onclick:move|event:MouseEvent|event.prevent_default(),
                AvatarFace{image,initials,size,failed_source,decorative:true}
                div{class:"ix-avatar-profile-text",span{class:"ix-avatar-username","{username}"}if let Some(extra)=extra{span{class:"ix-avatar-extra","{extra}"}}}
            }}
            if divider{div{class:"ix-avatar-divider",onclick:move|event:MouseEvent|event.prevent_default(),Divider{}}}
            {children}
        }
        if !open {if let Some(text)=text {TooltipFor {id:tooltip_id,controller,aria_label:aria_label_tooltip,"{text}"}}}
    }}
}

#[cfg(test)]
mod tests;
