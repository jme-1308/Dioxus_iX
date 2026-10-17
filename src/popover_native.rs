//! Native close execution for a retained Popover host. Rust owns admission,
//! hierarchy, visibility and callbacks; this module only executes DOM intent.
use super::*;

fn current(surface: Surface, generation: u64) -> bool {
    surface
        .generation
        .try_peek()
        .is_ok_and(|value| *value == generation)
        && surface.open.try_peek().is_ok_and(|value| !*value)
}

/// Close the retained native host without removing its descendant surfaces.
/// A command watermark prevents a delayed older close from hiding a newer open.
pub(super) async fn close(surface: Surface, generation: u64, restore_focus: bool) -> bool {
    if !current(surface, generation)
        || surface
            .node
            .try_peek()
            .ok()
            .is_none_or(|node| node.is_none())
    {
        return false;
    }
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    {
        use wasm_bindgen::JsCast;
        let Ok(element) = surface.element() else {
            return false;
        };
        let Ok(panel) = element.dyn_into::<web_sys::HtmlElement>() else {
            return false;
        };
        let native_current = || {
            panel.is_connected()
                && panel
                    .get_attribute("data-surface-generation")
                    .and_then(|value| value.parse::<u64>().ok())
                    .is_some_and(|value| value <= generation)
                && panel
                    .get_attribute("data-ix-surface-command")
                    .is_none_or(|value| value.parse::<u64>().is_ok_and(|value| value <= generation))
        };
        if !native_current() {
            return false;
        }
        let _ = panel.set_attribute("data-ix-surface-command", &generation.to_string());
        if !restore_focus {
            release_web(&panel);
        }
        if panel.matches(":popover-open").unwrap_or(false) && panel.hide_popover().is_err() {
            return false;
        }
        if !restore_focus {
            blur_trigger_web(&panel);
        }
        if !current(surface, generation)
            || !native_current()
            || panel.matches(":popover-open").unwrap_or(false)
        {
            return false;
        }
        // The source corrects native hidePopover focus after its next paint.
        if !crate::dom_query::eval("await new Promise(requestAnimationFrame); return true;")
            .join::<bool>()
            .await
            .is_ok_and(|value| value)
        {
            return false;
        }
        if !current(surface, generation)
            || !native_current()
            || panel.get_attribute("data-ix-surface-command").as_deref()
                != Some(generation.to_string().as_str())
        {
            return false;
        }
        if restore_focus {
            let native_invoker = surface
                .native_invoker
                .try_peek()
                .ok()
                .and_then(|value| value.clone());
            if let Some(invoker) = native_invoker.filter(|invoker| invoker.is_connected()) {
                let _ = invoker.focus();
            } else {
                let invoker = surface
                    .invoker
                    .try_peek()
                    .ok()
                    .and_then(|value| value.clone());
                if let Some(invoker) = invoker {
                    let _ = invoker.set_focus(true).await;
                }
            }
        } else {
            release_web(&panel);
            if !crate::dom_query::eval("await new Promise(requestAnimationFrame); return true;")
                .join::<bool>()
                .await
                .is_ok_and(|value| value)
            {
                return false;
            }
            if !current(surface, generation) || !native_current() {
                return false;
            }
            blur_trigger_web(&panel);
        }
        current(surface, generation)
            && native_current()
            && !panel.matches(":popover-open").unwrap_or(false)
    }
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    {
        let id = serde_json::to_string(&surface.id()).expect("string serialization");
        let guard = format!(
            r#"
            const panel=document.getElementById({id});
            const valid=()=>{{
                if(!panel?.isConnected)return false;
                const value=panel.getAttribute('data-surface-generation');
                if(value===null||!/^\d+$/.test(value)||Number(value)>{generation})return false;
                const command=panel.getAttribute('data-ix-surface-command');
                return command===null||(/^\d+$/.test(command)&&Number(command)<={generation});
            }};
            const trigger=()=>Array.from(document.querySelectorAll('[popovertarget]')).find(node=>node.getAttribute('popovertarget')==={id});
            const blurTrigger=()=>{{const target=trigger();if(target instanceof HTMLElement)target.blur();}};
            const release=()=>{{const active=document.activeElement;if(active instanceof HTMLElement&&panel.contains(active))active.blur();blurTrigger();}};
        "#
        );
        let script = format!(
            r#"
            {guard}
            if(!valid())return false;
            panel.setAttribute('data-ix-surface-command','{generation}');
            try{{
                if(!{restore_focus})release();
                if(panel.matches(':popover-open'))panel.hidePopover();
                if(!{restore_focus})blurTrigger();
                return valid()&&!panel.matches(':popover-open');
            }}catch{{return false;}}
        "#
        );
        if !crate::dom_query::eval(&script)
            .join::<bool>()
            .await
            .is_ok_and(|value| value)
            || !current(surface, generation)
        {
            return false;
        }
        let focus_target = surface
            .bridge_invoker
            .try_peek()
            .ok()
            .and_then(|value| value.clone());
        if restore_focus {
            if let Some(target) = focus_target {
                let target_id = serde_json::to_string(&target.id).expect("string serialization");
                let script = format!(
                    r#"
                    {guard}
                    await new Promise(requestAnimationFrame);
                    if(!valid()||panel.getAttribute('data-ix-surface-command')!=='{generation}'||panel.matches(':popover-open'))return false;
                    const target=document.getElementById({target_id});
                    if(target instanceof HTMLElement&&target.isConnected){{target.focus();if({temporary})target.removeAttribute('id');}}
                    return true;
                "#,
                    temporary = target.temporary
                );
                if !crate::dom_query::eval(&script)
                    .join::<bool>()
                    .await
                    .is_ok_and(|value| value)
                {
                    return false;
                }
            } else {
                let script = format!(
                    r#"{guard} await new Promise(requestAnimationFrame); return valid()&&panel.getAttribute('data-ix-surface-command')==='{generation}'&&!panel.matches(':popover-open');"#
                );
                if !crate::dom_query::eval(&script)
                    .join::<bool>()
                    .await
                    .is_ok_and(|value| value)
                    || !current(surface, generation)
                {
                    return false;
                }
                let invoker = surface
                    .invoker
                    .try_peek()
                    .ok()
                    .and_then(|value| value.clone());
                if let Some(invoker) = invoker {
                    let _ = invoker.set_focus(true).await;
                }
            }
        } else {
            let script = format!(
                r#"
                {guard}
                await new Promise(requestAnimationFrame);
                if(!valid()||panel.getAttribute('data-ix-surface-command')!=='{generation}'||panel.matches(':popover-open'))return false;
                release();
                await new Promise(requestAnimationFrame);
                if(!valid()||panel.getAttribute('data-ix-surface-command')!=='{generation}'||panel.matches(':popover-open'))return false;
                blurTrigger();return true;
            "#
            );
            if !crate::dom_query::eval(&script)
                .join::<bool>()
                .await
                .is_ok_and(|value| value)
            {
                return false;
            }
        }
        current(surface, generation)
    }
    #[cfg(not(any(
        all(feature = "web", target_arch = "wasm32"),
        all(feature = "desktop", not(target_arch = "wasm32"))
    )))]
    {
        let _ = (surface, generation, restore_focus);
        false
    }
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
fn blur_trigger_web(panel: &web_sys::HtmlElement) {
    use wasm_bindgen::JsCast;
    let Some(document) = panel.owner_document() else {
        return;
    };
    let Ok(triggers) = document.query_selector_all("[popovertarget]") else {
        return;
    };
    for index in 0..triggers.length() {
        let Some(trigger) = triggers
            .item(index)
            .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            continue;
        };
        if trigger.get_attribute("popovertarget").as_deref() == Some(panel.id().as_str()) {
            let _ = trigger.blur();
            break;
        }
    }
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
fn release_web(panel: &web_sys::HtmlElement) {
    use wasm_bindgen::JsCast;
    if let Some(active) = panel
        .owner_document()
        .and_then(|document| document.active_element())
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    {
        if panel.contains(Some(active.as_ref())) {
            let _ = active.blur();
        }
    }
    blur_trigger_web(panel);
}

/// Apply the caller's admitted opening-focus intent after native presentation.
/// An interactive descendant owned by another overlay is never selected.
pub(super) async fn focus_open(surface: Surface, generation: u64) -> bool {
    let current_open = || {
        surface
            .generation
            .try_peek()
            .is_ok_and(|value| *value == generation)
            && surface.open.try_peek().is_ok_and(|value| *value)
    };
    if !current_open()
        || surface
            .node
            .try_peek()
            .ok()
            .is_none_or(|node| node.is_none())
    {
        return false;
    }
    #[cfg(any(
        all(feature = "web", target_arch = "wasm32"),
        all(feature = "desktop", not(target_arch = "wasm32"))
    ))]
    {
        // Split the paint wait from execution so Rust can reject a newer intent
        // before sending the native focus command over the desktop bridge.
        if !crate::dom_query::eval("await new Promise(requestAnimationFrame); return true;")
            .join::<bool>()
            .await
            .is_ok_and(|value| value)
            || !current_open()
        {
            return false;
        }
        let id = serde_json::to_string(&surface.id()).expect("string serialization");
        let script = format!(
            r#"
            const panel=document.getElementById({id});
            if(!panel?.isConnected||!panel.matches(':popover-open')||panel.getAttribute('data-ix-surface-command')!=='{generation}')return false;
            const stamped=panel.getAttribute('data-surface-generation');
            if(stamped===null||!/^\d+$/.test(stamped)||Number(stamped)>{generation})return false;
            const candidates=Array.from(panel.querySelectorAll('button,a[href],input:not([type=hidden]),select,textarea,[tabindex],[contenteditable=true]'))
                .filter(element=>{{
                    if(!(element instanceof HTMLElement)||element.tabIndex<0||element.matches(':disabled')||element.closest('[inert],[hidden]'))return false;
                    if(element.closest('[data-ix-overlay-host]')!==panel)return false;
                    const box=element.getBoundingClientRect(),style=getComputedStyle(element);
                    return box.width>0&&box.height>0&&style.visibility!=='hidden'&&style.visibility!=='collapse'&&style.display!=='none';
                }})
                .sort((a,b)=>(a.tabIndex>0?a.tabIndex:2147483647)-(b.tabIndex>0?b.tabIndex:2147483647));
            if(candidates.length)candidates[0].focus();
            return panel.isConnected&&panel.matches(':popover-open')&&panel.getAttribute('data-ix-surface-command')==='{generation}';
        "#
        );
        crate::dom_query::eval(&script)
            .join::<bool>()
            .await
            .is_ok_and(|value| value)
            && current_open()
    }
    #[cfg(not(any(
        all(feature = "web", target_arch = "wasm32"),
        all(feature = "desktop", not(target_arch = "wasm32"))
    )))]
    {
        let _ = (surface, generation);
        false
    }
}
