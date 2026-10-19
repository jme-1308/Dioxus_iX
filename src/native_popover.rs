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
                let id = serde_json::to_string(&id).expect("surface ID serialization");
                let marker = serde_json::to_string(&marker).expect("surface marker serialization");
                let script = format!(
                    r#"const panel=document.getElementById({id});
                    if(!panel?.isConnected||panel.getAttribute('data-ix-native-popover')!=={marker})return;
                    try{{
                        const open=panel.matches(':popover-open');
                        if({shown}&&!open&&panel.hasAttribute('popover')){{
                            const inert=panel.inert;panel.inert=true;
                            try{{panel.showPopover();}}finally{{panel.inert=inert;}}
                        }}else if(!{shown}&&open)panel.hidePopover();
                    }}catch{{}}"#
                );
                let _ = dioxus::document::eval(&script);
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
