//! Rust-owned loading workflow composed from the existing native modal parts.
//! Copyright (c) 2023 Siemens AG. SPDX-License-Identifier: MIT.
use crate::{Modal, ModalDismissRequest, ModalHandle, ModalLoading, ModalResult, ModalSize};
use dioxus::prelude::*;

/// A loading modal whose ordinary dismissal requests are denied by default.
///
/// Create `modal` with [`crate::use_modal`] and open with [`crate::ModalTrigger`].
/// Update `message` through normal Rust props; finish by calling
/// [`ModalHandle::close`], which bypasses the dismissal guard and preserves its
/// result payload. An optional `ondismissrequest` replaces the default denial
/// with the existing typed allow/deny policy. Optional children can compose
/// explicit task actions after the loading status. Native focus, modality and
/// lifecycle remain owned by [`Modal`]; no delay or removal timer is introduced.
#[component]
pub fn LoadingDialog(
    modal: ModalHandle,
    message: String,
    #[props(default = "Loading".to_string())] label: String,
    #[props(default)] centered: bool,
    #[props(default)] size: ModalSize,
    #[props(default)] children: Option<Element>,
    #[props(default)] ondismissrequest: Option<EventHandler<ModalDismissRequest>>,
    #[props(default)] onresult: Option<EventHandler<ModalResult>>,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
) -> Element {
    let description_id = format!("{}-loading-message", modal.id());
    rsx! { Modal {
        modal,label,centered,size,aria_describedby:description_id.clone(),onresult,onopenchange,
        ondismissrequest:move|request:ModalDismissRequest| {
            if let Some(guard)=ondismissrequest {guard.call(request);} else {request.deny();}
        },
        ModalLoading {span {id:description_id,"{message}"}}
        {children}
    }}
}

#[cfg(test)]
#[path = "loading_dialog_tests.rs"]
mod tests;
