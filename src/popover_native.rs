//! Native Popover eligibility observation and guarded show/hide plumbing.
//! Registered Dioxus mounted handles execute every focus request.
use super::*;
use crate::popover_focus::FocusCandidate;

#[derive(serde::Deserialize)]
pub(super) struct FocusSnapshot {
    pub candidates: Vec<FocusCandidate>,
}

fn current(surface: Surface, generation: u64, open: bool) -> bool {
    surface
        .generation
        .try_peek()
        .is_ok_and(|value| *value == generation)
        && surface.open.try_peek().is_ok_and(|value| *value == open)
}

fn guard(surface: Surface, generation: u64, open: Option<bool>, older_command: bool) -> String {
    let id = serde_json::to_string(&surface.id()).expect("string serialization");
    let visibility = open.map_or_else(String::new, |open| {
        format!("||panel.matches(':popover-open')!=={open}")
    });
    format!(
        r#"
        const panel=document.getElementById({id});
        const valid=()=>{{
            if(!panel?.isConnected{visibility})return false;
            const published=panel.getAttribute('data-surface-generation');
            if(published===null||!/^\d+$/.test(published)||Number(published)>{generation})return false;
            const command=panel.getAttribute('data-ix-surface-command');
            return {older_command}?(command===null||(/^\d+$/.test(command)&&Number(command)<={generation})):command==='{generation}';
        }};
    "#
    )
}

// Pinned MountedData exposes focus but no descendant focusable enumeration.
// Observe existing IDs and eligibility only; Rust filters registered handles and
// chooses the target. Unregistered controls are never a JavaScript fallback.
const CANDIDATES: &str = r#"
    const selector='[tabindex],button,a[href],area[href],audio,input,select,textarea,iframe,object,embed,video,[contenteditable]';
    const eligible=element=>{
        if(!(element instanceof HTMLElement)||element.matches('[tabindex^="-"],[hidden],[disabled],:disabled')||element.closest('[inert]'))return false;
        if(element.closest('[data-ix-overlay-native-surface="popover"]')!==panel)return false;
        const style=getComputedStyle(element);
        if(style.visibility==='hidden'||style.visibility==='collapse')return false;
        if(typeof element.checkVisibility==='function')return element.checkVisibility();
        for(let node=element;node;node=node.parentElement){if(getComputedStyle(node).display==='none')return false;}
        return element.getClientRects().length>0;
    };
"#;

/// Observe existing candidate IDs in document order without modifying the DOM.
pub(super) async fn snapshot(surface: Surface, generation: u64) -> Option<FocusSnapshot> {
    if !current(surface, generation, true) {
        return None;
    }
    let guard = guard(surface, generation, Some(true), false);
    let script = format!(
        r#"
        {guard}
        {CANDIDATES}
        if(!valid())return null;
        const candidates=Array.from(panel.querySelectorAll(selector)).filter(element=>element.id&&eligible(element)).map(element=>({{
            target:element.id,tab_index:element.tabIndex,in_header:!!element.closest('.ix-popover-header')&&panel.contains(element.closest('.ix-popover-header'))
        }}));
        return {{candidates}};
    "#
    );
    let result = crate::dom_query::eval(&script)
        .join::<Option<FocusSnapshot>>()
        .await
        .ok()
        .flatten();
    current(surface, generation, true)
        .then_some(result)
        .flatten()
}

/// Hide the retained native surface. Focus is owned by the Rust mounted executor.
pub(super) async fn close(surface: Surface, generation: u64) -> bool {
    if !current(surface, generation, false) {
        return false;
    }
    let guard = guard(surface, generation, None, true);
    let script = format!(
        r#"{guard}
        if(!valid())return false;
        panel.setAttribute('data-ix-surface-command','{generation}');
        try{{
            if(panel.matches(':popover-open'))panel.hidePopover();
            return panel.isConnected&&!panel.matches(':popover-open')&&panel.getAttribute('data-ix-surface-command')==='{generation}';
        }}catch{{return false;}}
    "#
    );
    crate::dom_query::eval(&script)
        .join::<bool>()
        .await
        .is_ok_and(|value| value)
        && current(surface, generation, false)
}
