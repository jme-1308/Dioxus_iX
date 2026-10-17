//! Native Popover facts and execution. Rust owns candidate priority, cached
//! eligibility and focus intent; this module only queries and operates the DOM.
use super::*;
use crate::popover_focus::{self, FocusCandidate, PostCloseFocus, RustCloseFocusPlan};

#[derive(serde::Deserialize)]
pub(super) struct FocusSnapshot {
    pub candidates: Vec<FocusCandidate>,
    pub trigger: Option<String>,
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

// Eligibility is a native fact: mirror source focusable selectors, inert/hidden
// ancestry and Popover ownership. Rust orders these facts and selects a target.
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

/// Capture native candidates in document order and an explicitly configured
/// trigger. Opaque DOM tokens bind later execution to the same actual element;
/// they neither change public IDs nor preserve JavaScript application state.
pub(super) async fn snapshot(surface: Surface, generation: u64) -> Option<FocusSnapshot> {
    if !current(surface, generation, true) {
        return None;
    }
    let guard = guard(surface, generation, Some(true), false);
    let prefix = serde_json::to_string(&format!(
        "{}-focus-{}",
        surface.id(),
        DIALOG_ID.fetch_add(1, Ordering::Relaxed)
    ))
    .expect("string serialization");
    let id = serde_json::to_string(&surface.id()).expect("string serialization");
    let script = format!(
        r#"
        {guard}
        {CANDIDATES}
        if(!valid())return null;
        let index=0;
        const token=element=>{{
            let value=element.getAttribute('data-ix-popover-focus-token');
            if(!value){{value={prefix}+'-'+index++;element.setAttribute('data-ix-popover-focus-token',value);}}
            return value;
        }};
        const candidates=Array.from(panel.querySelectorAll(selector)).filter(eligible).map(element=>({{
            target:token(element),tab_index:element.tabIndex,in_header:!!element.closest('.ix-popover-header')&&panel.contains(element.closest('.ix-popover-header'))
        }}));
        const trigger=Array.from(document.querySelectorAll('[popovertarget]')).find(element=>element instanceof HTMLElement&&element.getAttribute('popovertarget')==={id});
        return {{candidates,trigger:trigger?token(trigger):null}};
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

async fn next_frame(surface: Surface, generation: u64, open: bool) -> bool {
    crate::dom_query::eval("await new Promise(requestAnimationFrame);return true;")
        .join::<bool>()
        .await
        .is_ok_and(|value| value)
        && current(surface, generation, open)
}

/// Re-query after the presentation frame, then execute only Rust's chosen
/// content/header target. The opening eligibility cache is not overwritten.
pub(super) async fn focus_open(surface: Surface, generation: u64, hover: bool) -> bool {
    if !next_frame(surface, generation, true).await {
        return false;
    }
    let Some(facts) = snapshot(surface, generation).await else {
        return false;
    };
    let Some(target) = popover_focus::opening_target(&facts.candidates, hover) else {
        return true;
    };
    let target = serde_json::to_string(&target).expect("string serialization");
    let guard = guard(surface, generation, Some(true), false);
    let script = format!(
        r#"
        {guard}
        {CANDIDATES}
        if(!valid())return false;
        const target=Array.from(panel.querySelectorAll('[data-ix-popover-focus-token]')).find(element=>element.getAttribute('data-ix-popover-focus-token')==={target});
        if(!target||!eligible(target))return false;
        target.focus();return valid();
    "#
    );
    crate::dom_query::eval(&script)
        .join::<bool>()
        .await
        .is_ok_and(|value| value)
        && current(surface, generation, true)
}

/// Execute the admitted Rust close plan on the retained host. Native watermark,
/// Rust generation and element tokens reject obsolete hides/focus corrections.
pub(super) async fn close(
    surface: Surface,
    generation: u64,
    plan: RustCloseFocusPlan,
    trigger: Option<String>,
) -> bool {
    if !current(surface, generation, false) {
        return false;
    }
    let token = serde_json::to_string(&trigger).expect("optional string serialization");
    let trigger_query = format!(
        r#"
        const trigger={token}===null?undefined:Array.from(document.querySelectorAll('[data-ix-popover-focus-token]')).find(element=>element.getAttribute('data-ix-popover-focus-token')==={token});
        const blurTrigger=()=>{{if(trigger instanceof HTMLElement)trigger.blur();}};
        const release=()=>{{const active=document.activeElement;if(active instanceof HTMLElement&&panel.contains(active))active.blur();blurTrigger();}};
    "#
    );
    // The first command may find an older native-open watermark. Stamp it before
    // hide; all later commands require this exact operation's watermark.
    let first_guard = guard(surface, generation, None, true);
    let script = format!(
        r#"
        {first_guard}
        {trigger_query}
        if(!valid())return false;
        panel.setAttribute('data-ix-surface-command','{generation}');
        try{{
            if({pre_release})release();
            if(panel.matches(':popover-open'))panel.hidePopover();
            if({blur_trigger})blurTrigger();
            return panel.isConnected&&!panel.matches(':popover-open')&&panel.getAttribute('data-ix-surface-command')==='{generation}';
        }}catch{{return false;}}
    "#,
        pre_release = plan.pre_release,
        blur_trigger = plan.blur_trigger
    );
    if !crate::dom_query::eval(&script)
        .join::<bool>()
        .await
        .is_ok_and(|value| value)
        || !current(surface, generation, false)
        || !next_frame(surface, generation, false).await
    {
        return false;
    }
    let guard = guard(surface, generation, Some(false), false);
    let operation = match plan.post_close {
        PostCloseFocus::None => "",
        PostCloseFocus::Restore => {
            "if(trigger instanceof HTMLElement&&trigger.isConnected)trigger.focus();"
        }
        PostCloseFocus::Release => "release();",
    };
    let script =
        format!(r#"{guard}{trigger_query}if(!valid())return false;{operation}return valid();"#);
    if !crate::dom_query::eval(&script)
        .join::<bool>()
        .await
        .is_ok_and(|value| value)
        || !current(surface, generation, false)
    {
        return false;
    }
    if plan.post_close == PostCloseFocus::Release {
        if !next_frame(surface, generation, false).await {
            return false;
        }
        let script = format!(
            r#"{guard}{trigger_query}if(!valid())return false;blurTrigger();return valid();"#
        );
        if !crate::dom_query::eval(&script)
            .join::<bool>()
            .await
            .is_ok_and(|value| value)
        {
            return false;
        }
    }
    current(surface, generation, false)
}
