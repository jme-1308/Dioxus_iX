//! Guarded native Popover hide plumbing; no candidate discovery.
//! Registered Dioxus mounted handles execute every focus request.
use super::*;

fn current(surface: Surface, generation: u64, open: bool) -> bool {
    surface
        .generation
        .try_peek()
        .is_ok_and(|value| *value == generation)
        && surface.open.try_peek().is_ok_and(|value| *value == open)
}

fn guard(surface: Surface, generation: u64) -> String {
    let id = serde_json::to_string(&surface.id()).expect("string serialization");
    format!(
        r#"
        const panel=document.getElementById({id});
        const valid=()=>{{
            if(!panel?.isConnected)return false;
            const published=panel.getAttribute('data-surface-generation');
            if(published===null||!/^\d+$/.test(published)||Number(published)>{generation})return false;
            const command=panel.getAttribute('data-ix-surface-command');
            return command===null||(/^\d+$/.test(command)&&Number(command)<={generation});
        }};
    "#
    )
}

/// Hide the retained native surface. Focus is owned by the Rust mounted executor.
/// Pinned MountedData has no hidePopover operation; this bridge performs only
/// that missing top-layer operation, with no candidate discovery or focus path.
pub(super) async fn close(surface: Surface, generation: u64) -> bool {
    if !current(surface, generation, false) {
        return false;
    }
    let guard = guard(surface, generation);
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
