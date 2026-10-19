// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Component-owned native popover presentation.
//!
//! Pinned dioxus-html 0.8.0-alpha.2 MountedData exposes focus, scrolling and
//! measurements, but no showPopover/hidePopover operation. DesktopElement also
//! keeps its DOM query handle private. This primary adapter supplies only those
//! native operations; Dioxus owns rendering, mounting, visibility intent and
//! cleanup. Browsers without the native Popover API are unsupported here.

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SURFACE: AtomicU64 = AtomicU64::new(1);

/// Perform only the native top-layer operation missing from MountedData. The
/// caller owns visibility and lifetime; a rendered marker rejects stale intents.
/// An explicit source is the native invoker. Other surfaces promote inertly so
/// the browser cannot override focus owned by their Dioxus mounted handles.
/// Returns true for confirmed visibility, or false for a stale guarded intent.
pub(crate) async fn present(
    id: &str,
    shown: bool,
    guard: Option<(&str, &str)>,
    source: Option<&str>,
) -> Result<bool, crate::NativeSurfaceError> {
    #[cfg(any(feature = "web", feature = "desktop"))]
    {
        let task = crate::dom_query::eval(PRESENT);
        task.send((id, shown, guard, source))
            .map_err(|error| crate::NativeSurfaceError::Rejected(error.to_string()))?;
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Reply {
            Applied(bool),
            Rejected(String),
        }
        match task.join::<Reply>().await {
            Ok(Reply::Applied(applied)) => Ok(applied),
            Ok(Reply::Rejected(error)) => Err(crate::NativeSurfaceError::Rejected(error)),
            Err(error) => Err(crate::NativeSurfaceError::Rejected(error.to_string())),
        }
    }
    #[cfg(not(any(feature = "web", feature = "desktop")))]
    {
        let _ = (id, shown, guard, source);
        Err(crate::NativeSurfaceError::UnsupportedRenderer)
    }
}

#[cfg(any(feature = "web", feature = "desktop"))]
const PRESENT: &str = r#"
const [id,shown,guard,source]=await dioxus.recv(),panel=document.getElementById(id);
if(guard&&(!panel?.isConnected||panel.getAttribute(guard[0])!==guard[1]))return false;
if(!panel?.isConnected)return 'Surface is not mounted';
const invoker=source?document.getElementById(source):null;
if(source&&!invoker)return 'Invoker is not mounted';
try {
    const open=panel.matches(':popover-open');
    if(shown&&!open){
        if(typeof panel.showPopover!=='function')return 'Popover API is unavailable';
        if(invoker)panel.showPopover({source:invoker});
        else{
            const inert=panel.inert;panel.inert=true;
            try{panel.showPopover();}finally{panel.inert=inert;}
        }
    }else if(!shown&&open)panel.hidePopover();
    if(!panel.isConnected)return guard?false:'Surface is not mounted';
    return panel.matches(':popover-open')===shown?true:'Native popover visibility did not match request';
}catch(error){return String(error);}
"#;

pub(crate) struct PopoverBinding {
    /// Render this marker on the exact element with the supplied ID. A late
    /// operation cannot affect a newer component or a different visibility intent.
    pub marker: String,
    pub onmounted: EventHandler<MountedEvent>,
}

pub(crate) fn use_native_popover(id: String, shown: bool) -> PopoverBinding {
    let instance = use_hook(|| NEXT_SURFACE.fetch_add(1, Ordering::Relaxed));
    let marker = format!("{instance}:{shown}");
    let mut mounts = use_signal(|| 0_u64);
    use_effect(use_reactive(
        (&id, &marker, &shown),
        move |(id, marker, shown)| {
            if mounts() == 0 {
                return;
            }
            #[cfg(any(
                all(feature = "web", target_arch = "wasm32"),
                all(feature = "desktop", not(target_arch = "wasm32"))
            ))]
            {
                spawn(async move {
                    let _ =
                        present(&id, shown, Some(("data-ix-native-popover", &marker)), None).await;
                });
            }
            #[cfg(not(any(
                all(feature = "web", target_arch = "wasm32"),
                all(feature = "desktop", not(target_arch = "wasm32"))
            )))]
            let _ = (id, marker, shown);
        },
    ));
    let onmounted = use_callback(move |_: MountedEvent| {
        mounts.with_mut(|revision| *revision = revision.wrapping_add(1));
    });
    PopoverBinding { marker, onmounted }
}
