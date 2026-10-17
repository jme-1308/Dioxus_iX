//! Native top-layer dialogs and popovers with Rust result and dismissal handling.
//!
//! Open dialogs with [`ModalTrigger`], which uses the HTML Invoker Commands API
//! (`command="show-modal"`). This invokes native modal behavior, including inert
//! background content, focus containment, Escape and return focus. It requires a
//! browser/WebView supporting invoker commands; there is no JavaScript fallback.
//! [`ModalHandle::close`] and [`PopoverHandle::hide`] replace the native surface
//! with a closed instance and restore the registered invoker's focus in Rust.
//!
//! Imperative opening, hover activation and popover keyboard containment use
//! native DOM methods: typed Rust bindings on web and a small embedded native
//! DOM bridge on desktop. Rust owns visibility, dismissal and timer state.
use dioxus::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static DIALOG_ID: AtomicUsize = AtomicUsize::new(1);

/// An imperative top-layer operation could not be completed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeSurfaceError {
    /// The surface has not received its mounted element yet.
    NotMounted,
    /// This renderer does not expose native DOM activation through Rust.
    UnsupportedRenderer,
    /// A cancelable opening callback prevented the native operation.
    Prevented,
    /// The controller was closed/replaced while native activation was in flight.
    Superseded,
    /// The browser rejected the operation, for example for a disconnected node.
    Rejected(String),
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[derive(Clone, serde::Deserialize)]
struct BridgeFocusTarget {
    id: String,
    temporary: bool,
}

impl std::fmt::Display for NativeSurfaceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMounted => formatter.write_str("surface is not mounted"),
            Self::UnsupportedRenderer => {
                formatter.write_str("renderer does not expose native surface activation")
            }
            Self::Prevented => formatter.write_str("surface opening was prevented"),
            Self::Superseded => formatter.write_str("surface opening was superseded"),
            Self::Rejected(reason) => write!(formatter, "native surface opening failed: {reason}"),
        }
    }
}

impl std::error::Error for NativeSurfaceError {}

#[derive(Clone, Copy, PartialEq)]
struct Surface {
    id: Signal<String>,
    generation: Signal<u64>,
    open: Signal<bool>,
    node: Signal<Option<Rc<MountedData>>>,
    invoker: Signal<Option<Rc<MountedData>>>,
    restore_after_mount: Signal<Option<u64>>,
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    native_invoker: Signal<Option<web_sys::HtmlElement>>,
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    bridge_invoker: Signal<Option<BridgeFocusTarget>>,
}

fn use_surface(prefix: &'static str) -> Surface {
    let surface = Surface {
        id: use_signal(|| format!("ix-{prefix}-{}", DIALOG_ID.fetch_add(1, Ordering::Relaxed))),
        generation: use_signal(|| 0),
        open: use_signal(|| false),
        node: use_signal(|| None),
        invoker: use_signal(|| None),
        restore_after_mount: use_signal(|| None),
        #[cfg(all(feature = "web", target_arch = "wasm32"))]
        native_invoker: use_signal(|| None),
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        bridge_invoker: use_signal(|| None),
    };
    use_drop(move || surface.cleanup_temporary_focus_id());
    surface
}

impl Surface {
    fn cleanup_native_position(self) {
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            let id = serde_json::to_string(&self.id()).expect("string serialization");
            let generation = serde_json::to_string(&self.generation.peek().to_string())
                .expect("string serialization");
            let script = format!(
                "const panel=document.getElementById({id});if(panel&&panel.matches('.ix-popover')&&!panel.matches(':popover-open')&&panel.getAttribute('data-surface-generation')==={generation}){{panel.style.setProperty('position-anchor','--'+{id});panel.removeAttribute('data-unanchored');}}"
            );
            let _ = dioxus::document::eval(&script);
        }
    }

    fn cleanup_temporary_focus_id(self) {
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        if let Some(target) = self
            .bridge_invoker
            .peek()
            .clone()
            .filter(|target| target.temporary)
        {
            let id = serde_json::to_string(&target.id).expect("string serialization");
            let script = format!(
                "const target=document.getElementById({id});if(target)target.removeAttribute('id');"
            );
            let _ = dioxus::document::eval(&script);
        }
    }

    fn id(self) -> String {
        self.id.read().clone()
    }

    fn replace(mut self, return_focus: bool) {
        self.open.set(false);
        let next = self.generation.peek().wrapping_add(1);
        self.restore_after_mount.set(return_focus.then_some(next));
        self.generation.set(next);
    }

    fn mounted(mut self, generation: u64, node: Rc<MountedData>) {
        if *self.generation.peek() != generation {
            return;
        }
        self.node.set(Some(node));
        if *self.restore_after_mount.peek() != Some(generation) {
            return;
        }
        self.restore_after_mount.set(None);
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        if let Some(target) = self.bridge_invoker.peek().clone() {
            spawn(async move {
                if *self.generation.peek() != generation || *self.open.peek() {
                    return;
                }
                let id = serde_json::to_string(&target.id).expect("string serialization");
                let script = format!(
                    "const target=document.getElementById({id});if(target instanceof HTMLElement){{target.focus();if({})target.removeAttribute('id');}}",
                    target.temporary
                );
                let _ = crate::dom_query::eval(&script).await;
            });
            return;
        }
        #[cfg(all(feature = "web", target_arch = "wasm32"))]
        if let Some(invoker) = self.native_invoker.peek().clone() {
            spawn(async move {
                if *self.generation.peek() == generation && !*self.open.peek() {
                    let _ = invoker.focus();
                }
            });
            return;
        }
        if let Some(invoker) = self.invoker.peek().clone() {
            spawn(async move {
                // The prior native dialog has now been removed. Restoring before
                // this mount lets browser dialog teardown overwrite the focus.
                if *self.generation.peek() == generation && !*self.open.peek() {
                    let _ = invoker.set_focus(true).await;
                }
            });
        }
    }

    async fn native_visibility(self, generation: u64) -> Option<bool> {
        let node = self.node.peek().clone()?;
        let bounds = node.get_client_rect().await.ok()?;
        if *self.generation.peek() != generation {
            return None;
        }
        Some(bounds.size.width > 0.0 && bounds.size.height > 0.0)
    }

    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    fn element(self) -> Result<web_sys::Element, NativeSurfaceError> {
        self.node
            .peek()
            .as_ref()
            .ok_or(NativeSurfaceError::NotMounted)?
            .downcast::<web_sys::Element>()
            .cloned()
            .ok_or(NativeSurfaceError::UnsupportedRenderer)
    }

    async fn native_open(
        self,
        modal: bool,
        remember_focus: bool,
    ) -> Result<(), NativeSurfaceError> {
        #[cfg(all(feature = "web", target_arch = "wasm32"))]
        {
            use wasm_bindgen::JsCast;
            let element = self.element()?;
            let active = element
                .owner_document()
                .and_then(|document| document.active_element())
                .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
            if remember_focus {
                let mut surface = self;
                surface.native_invoker.set(active.clone());
                surface.invoker.set(None);
            }
            if modal {
                let dialog = element
                    .dyn_into::<web_sys::HtmlDialogElement>()
                    .map_err(|_| NativeSurfaceError::UnsupportedRenderer)?;
                dialog
                    .show_modal()
                    .map_err(|error| NativeSurfaceError::Rejected(format!("{error:?}")))?;
                if !dialog.open() {
                    return Err(NativeSurfaceError::Prevented);
                }
            } else {
                let panel = element
                    .dyn_into::<web_sys::HtmlElement>()
                    .map_err(|_| NativeSurfaceError::UnsupportedRenderer)?;
                let options = web_sys::ShowPopoverOptions::new();
                if panel.matches(".ix-popover").unwrap_or(false) {
                    let explicit = panel
                        .owner_document()
                        .and_then(|document| document.query_selector_all("[popovertarget]").ok())
                        .and_then(|triggers| {
                            (0..triggers.length())
                                .filter_map(|index| triggers.item(index))
                                .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                                .find(|trigger| {
                                    trigger.get_attribute("popovertarget").as_deref()
                                        == Some(&self.id())
                                })
                        });
                    let has_explicit = explicit.is_some();
                    let saved = self.native_invoker.peek().clone();
                    let source = explicit
                        .or(if remember_focus { active } else { saved })
                        .filter(|source| {
                            source.is_connected()
                                && !matches!(source.tag_name().as_str(), "BODY" | "HTML")
                                && !panel.contains(Some(source.as_ref()))
                                && source.get_bounding_client_rect().width() > 0.0
                                && source.get_bounding_client_rect().height() > 0.0
                        });
                    if let Some(source) = source {
                        let _ = panel.remove_attribute("data-unanchored");
                        if !has_explicit {
                            let _ = panel.style().set_property("position-anchor", "auto");
                        }
                        options.set_source(&source);
                    } else {
                        let _ = panel.set_attribute("data-unanchored", "true");
                    }
                }
                panel
                    .show_popover_with_options(&options)
                    .map_err(|error| NativeSurfaceError::Rejected(format!("{error:?}")))?;
                if !panel.matches(":popover-open").unwrap_or(false) {
                    return Err(NativeSurfaceError::Prevented);
                }
            }
            Ok(())
        }
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        {
            if self.node.peek().is_none() {
                return Err(NativeSurfaceError::NotMounted);
            }
            #[derive(serde::Deserialize)]
            struct NativeReply {
                opened: bool,
                error: Option<String>,
                focus_target: Option<BridgeFocusTarget>,
                #[serde(default)]
                superseded: bool,
            }
            let generation = *self.generation.peek();
            let expected_generation =
                serde_json::to_string(&generation.to_string()).expect("string serialization");
            let id = serde_json::to_string(&self.id()).expect("string serialization");
            let focus_id = serde_json::to_string(&format!("{}-return-{generation}", self.id()))
                .expect("string serialization");
            let saved_focus_id = serde_json::to_string(
                &self
                    .bridge_invoker
                    .peek()
                    .as_ref()
                    .map(|target| target.id.clone()),
            )
            .expect("optional string serialization");
            let script = format!(
                r#"
                const element=document.getElementById({id});
                if(!element)return {{opened:false,error:'surface is not mounted',focus_target:null}};
                if(element.getAttribute('data-surface-generation')!=={expected_generation})return {{opened:false,error:null,focus_target:null,superseded:true}};
                const active=document.activeElement;
                try{{
                    if({modal})element.showModal();else{{
                        let source;
                        if(element.matches('.ix-popover')){{
                            const explicit=Array.from(document.querySelectorAll('[popovertarget]')).find(trigger=>trigger.getAttribute('popovertarget')==={id});
                            source=explicit||({remember_focus}?active:document.getElementById({saved_focus_id}));
                            if(source instanceof HTMLElement&&source.isConnected&&!['BODY','HTML'].includes(source.tagName)&&!element.contains(source)&&source.getBoundingClientRect().width>0&&source.getBoundingClientRect().height>0){{
                                element.removeAttribute('data-unanchored');
                                if(!explicit)element.style.setProperty('position-anchor','auto');
                            }}else{{source=undefined;element.setAttribute('data-unanchored','true');}}
                        }}
                        element.showPopover(source?{{source}}:undefined);
                    }}
                    const opened={modal}?element.open:element.matches(':popover-open');
                    let focus_target=null;
                    if(opened&&{remember_focus}&&active instanceof HTMLElement){{
                        const temporary=!active.id;
                        if(temporary)active.id={focus_id};
                        focus_target={{id:active.id,temporary}};
                    }}
                    return {{opened,error:null,focus_target}};
                }}catch(error){{return {{opened:false,error:String(error),focus_target:null}};}}
            "#
            );
            let reply = crate::dom_query::eval(&script)
                .join::<NativeReply>()
                .await
                .map_err(|error| NativeSurfaceError::Rejected(error.to_string()))?;
            // Native dispatch and its response cross the desktop event loop.
            // Neither a newer element with the same stable ID nor a late reply
            // may revive a closed generation or overwrite its focus target.
            if *self.generation.peek() != generation || reply.superseded {
                return Err(NativeSurfaceError::Superseded);
            }
            if let Some(error) = reply.error {
                return Err(NativeSurfaceError::Rejected(error));
            }
            if !reply.opened {
                return Err(NativeSurfaceError::Prevented);
            }
            if remember_focus {
                let mut surface = self;
                surface.bridge_invoker.set(reply.focus_target);
                surface.invoker.set(None);
            }
            Ok(())
        }
        #[cfg(not(any(
            all(feature = "web", target_arch = "wasm32"),
            all(feature = "desktop", not(target_arch = "wasm32"))
        )))]
        {
            let _ = (modal, remember_focus);
            if self.node.peek().is_none() {
                Err(NativeSurfaceError::NotMounted)
            } else {
                Err(NativeSurfaceError::UnsupportedRenderer)
            }
        }
    }

    fn remember_invoker(mut self, invoker: Option<Rc<MountedData>>) {
        self.invoker.set(invoker);
        #[cfg(all(feature = "web", target_arch = "wasm32"))]
        self.native_invoker.set(None);
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        self.bridge_invoker.set(None);
    }

    async fn advance_focus(self, backwards: bool) {
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            // This bridge only resolves native focusable descendants and focuses
            // the adjacent element. It installs no listeners or application state.
            let id = serde_json::to_string(&self.id()).expect("string serialization");
            let script = format!(
                r#"
                const panel=document.getElementById({id});if(!panel)return;
                const elements=Array.from(panel.querySelectorAll('button,a[href],input:not([type=hidden]),select,textarea,[tabindex],[contenteditable=true]'))
                    .filter(element=>element instanceof HTMLElement&&element.tabIndex>=0&&!element.matches(':disabled')&&!element.closest('[inert]')&&element.getClientRects().length>0)
                    .sort((a,b)=>(a.tabIndex>0?a.tabIndex:2147483647)-(b.tabIndex>0?b.tabIndex:2147483647));
                if(elements.length===0){{panel.focus();return;}}
                const index=elements.indexOf(document.activeElement);
                const next=index<0?({backwards}?elements.length-1:0):(index+({backwards}?-1:1)+elements.length)%elements.length;
                elements[next].focus();
            "#
            );
            let _ = crate::dom_query::eval(&script).await;
        }
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        let _ = backwards;
    }

    async fn contains_focus(self) -> bool {
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            let id = serde_json::to_string(&self.id()).expect("string serialization");
            let script = format!(
                "const panel=document.getElementById({id});return !!panel&&panel.contains(document.activeElement);"
            );
            crate::dom_query::eval(&script)
                .join::<bool>()
                .await
                .unwrap_or(false)
        }
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        false
    }
}

/// Upstream modal width presets, including full-width and full-screen layouts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModalSize {
    /// 360px dialog, the upstream default.
    #[default]
    Xs,
    /// 480px dialog.
    Sm,
    /// 600px dialog.
    Md,
    /// 720px dialog.
    Lg,
    /// 840px dialog.
    Xl,
    /// 95% viewport width.
    FullWidth,
    /// Full viewport dimensions.
    FullScreen,
}

impl ModalSize {
    /// Matching upstream size token.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "360",
            Self::Sm => "480",
            Self::Md => "600",
            Self::Lg => "720",
            Self::Xl => "840",
            Self::FullWidth => "full-width",
            Self::FullScreen => "full-screen",
        }
    }
}

/// Why a user or application requested dismissal rather than confirming a result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalDismissReason {
    /// Native Escape/close request.
    Escape,
    /// Pointer down and up both occurred outside the panel.
    Backdrop,
    /// Header close button.
    Header,
    /// Footer Cancel button.
    Cancel,
    /// An application-defined reason.
    Programmatic(String),
}

/// Completed modal operation. Browser-owned `method="dialog"` submissions without
/// a Rust handler report `NativeClosed`; use `handle.close(payload)` for a payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalResult {
    /// Application confirmation with its payload.
    Closed(String),
    /// Approved dismissal with its reason.
    Dismissed(ModalDismissReason),
    /// Browser-native close not initiated by this Rust controller.
    NativeClosed,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct ModalCallbacks {
    onclose: Option<EventHandler<String>>,
    ondismiss: Option<EventHandler<ModalDismissReason>>,
    onresult: Option<EventHandler<ModalResult>>,
    ondismissrequest: Option<EventHandler<ModalDismissRequest>>,
    onopenchange: Option<EventHandler<bool>>,
}

/// Controller for a declaratively opened native dialog. Create with [`use_modal`].
/// Both imperative opening and [`ModalTrigger`] retain native top-layer modality.
#[derive(Clone, Copy, PartialEq)]
pub struct ModalHandle {
    surface: Surface,
    non_blocking: Signal<bool>,
    callbacks: Signal<ModalCallbacks>,
    result: Signal<Option<ModalResult>>,
    pending: Signal<Option<u64>>,
    request_sequence: Signal<u64>,
}

/// Create a stable dialog controller in a component, following the rules of hooks.
pub fn use_modal() -> ModalHandle {
    ModalHandle {
        surface: use_surface("modal"),
        non_blocking: use_signal(|| false),
        callbacks: use_signal(ModalCallbacks::default),
        result: use_signal(|| None),
        pending: use_signal(|| None),
        request_sequence: use_signal(|| 0),
    }
}

impl ModalHandle {
    /// Open the real native modal (or configured non-blocking popover).
    /// Returns an explicit error when unmounted or unsupported by the renderer.
    /// Calling this on an already open modal has no effect.
    pub async fn open(self) -> Result<(), NativeSurfaceError> {
        if self.is_open() {
            return Ok(());
        }
        let generation = *self.surface.generation.peek();
        self.surface
            .native_open(!*self.non_blocking.peek(), true)
            .await?;
        if *self.surface.generation.peek() != generation {
            return Err(NativeSurfaceError::Superseded);
        }
        self.opened();
        Ok(())
    }

    fn opened(mut self) {
        if *self.surface.open.peek() {
            return;
        }
        self.surface.open.set(true);
        self.result.set(None);
        self.pending.set(None);
        if let Some(callback) = self.callbacks.peek().onopenchange {
            callback.call(true);
        }
    }
    /// Native dialog ID, also used by invoker controls.
    pub fn id(self) -> String {
        self.surface.id()
    }
    /// Most recently observed native visibility.
    pub fn is_open(self) -> bool {
        (self.surface.open)()
    }
    /// Whether an asynchronous dismissal decision is outstanding.
    pub fn is_dismiss_pending(self) -> bool {
        self.pending.read().is_some()
    }
    /// Last completed operation, retained while the dialog is closed.
    pub fn result(self) -> Option<ModalResult> {
        self.result.read().clone()
    }

    /// Private native-entry protocol identity; a replaced dialog cannot consume
    /// an observation from its previous DOM instance.
    pub(crate) fn surface_generation(self) -> u64 {
        *self.surface.generation.peek()
    }

    /// Finish the unguarded application-switch dismissal in its original native
    /// click task. The caller first validates its owned connected dialog and
    /// entry descriptor. An observed native-open fact can reconcile the earliest
    /// interaction before the renderer has delivered `beforetoggle`.
    pub(crate) fn dismiss_native_entry(self, generation: u64, native_open: bool) -> bool {
        let current = || {
            self.surface_generation() == generation
                && self.surface.node.peek().is_some()
                && self.pending.peek().is_none()
                && self.callbacks.peek().ondismissrequest.is_none()
        };
        if !native_open || !current() {
            return false;
        }
        if !self.is_open() {
            self.opened();
        }
        if !current() || !self.is_open() {
            return false;
        }
        let result = ModalResult::Dismissed(ModalDismissReason::Programmatic("app-switch".into()));
        self.finish(result.clone());
        self.surface_generation() == generation.wrapping_add(1)
            && self.result.peek().as_ref() == Some(&result)
    }

    fn finish(mut self, result: ModalResult) {
        if !self.is_open() {
            return;
        }
        self.pending.set(None);
        self.result.set(Some(result.clone()));
        let callbacks = *self.callbacks.peek();
        self.surface.replace(true);
        if let Some(callback) = callbacks.onopenchange {
            callback.call(false);
        }
        match &result {
            ModalResult::Closed(value) => {
                if let Some(callback) = callbacks.onclose {
                    callback.call(value.clone());
                }
            }
            ModalResult::Dismissed(reason) => {
                if let Some(callback) = callbacks.ondismiss {
                    callback.call(reason.clone());
                }
            }
            ModalResult::NativeClosed => {}
        }
        if let Some(callback) = callbacks.onresult {
            callback.call(result);
        }
    }

    /// Confirm and close, preserving the payload and restoring invoker focus.
    /// This bypasses the dismissal guard, matching upstream `closeModal`.
    pub fn close(self, value: impl Into<String>) {
        self.finish(ModalResult::Closed(value.into()));
    }

    /// Request dismissal. With `ondismissrequest`, the dialog stays modal until
    /// that request is explicitly allowed; repeated requests are coalesced.
    pub fn dismiss(mut self, reason: ModalDismissReason) {
        if !self.is_open() || self.pending.peek().is_some() {
            return;
        }
        let guard = self.callbacks.peek().ondismissrequest;
        if let Some(guard) = guard {
            let id = self.request_sequence.peek().wrapping_add(1);
            self.request_sequence.set(id);
            self.pending.set(Some(id));
            let generation = *self.surface.generation.peek();
            guard.call(ModalDismissRequest {
                modal: self,
                reason,
                id,
                generation,
            });
        } else {
            self.finish(ModalResult::Dismissed(reason));
        }
    }
}

/// Deferred dismissal decision. It may be retained across an async task or user
/// confirmation. Decisions from an already closed/reopened dialog are ignored.
#[derive(Clone, PartialEq)]
pub struct ModalDismissRequest {
    /// Why dismissal was requested.
    pub reason: ModalDismissReason,
    modal: ModalHandle,
    id: u64,
    generation: u64,
}

impl ModalDismissRequest {
    fn is_current(&self) -> bool {
        self.modal.is_open()
            && *self.modal.pending.peek() == Some(self.id)
            && *self.modal.surface.generation.peek() == self.generation
    }
    /// Approve this pending dismissal. Returns false for a stale decision.
    pub fn allow(self) -> bool {
        if !self.is_current() {
            return false;
        }
        self.modal.finish(ModalResult::Dismissed(self.reason));
        true
    }
    /// Reject dismissal and keep the dialog open. Returns false for a stale decision.
    pub fn deny(mut self) -> bool {
        if !self.is_current() {
            return false;
        }
        self.modal.pending.set(None);
        true
    }
}

/// Native modal invoker. `non_blocking` is configured on the corresponding [`Modal`].
#[component]
pub fn ModalTrigger(
    modal: ModalHandle,
    children: Element,
    #[props(default)] variant: crate::ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] autofocus: bool,
) -> Element {
    let id = modal.id();
    let non_blocking = (modal.non_blocking)();
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    rsx! {
        button { class: "ix-button ix-button-{variant.as_str()}", r#type: "button", disabled, autofocus,
            "commandfor": if non_blocking { None } else { Some(id.clone()) },
            "command": if non_blocking { None } else { Some("show-modal") },
            "popovertarget": if non_blocking { Some(id.clone()) } else { None },
            "popovertargetaction": if non_blocking { Some("show") } else { None },
            "aria-haspopup": "dialog", "aria-controls": id,
            onmounted: move |event| { trigger.set(Some(event.data())); },
            onclick: move |_| modal.surface.remember_invoker(trigger.peek().clone()),
            {children}
        }
    }
}

/// Dismiss/confirm control. Omit `modal` inside a [`Modal`] to use its controller.
#[component]
pub fn ModalClose(
    children: Element,
    #[props(default)] modal: Option<ModalHandle>,
    #[props(default)] dismiss: bool,
    #[props(default)] value: String,
    #[props(default)] disabled: bool,
    #[props(default)] autofocus: bool,
    #[props(default = crate::ButtonVariant::Secondary)] variant: crate::ButtonVariant,
) -> Element {
    let modal = modal.or_else(try_consume_context::<ModalHandle>);
    rsx! {
        button { class: "ix-button ix-button-{variant.as_str()}", r#type: "button", disabled: disabled || modal.is_none(), autofocus,
            onclick: move |_| if let Some(modal) = modal { if dismiss { modal.dismiss(ModalDismissReason::Cancel); } else { modal.close(value.clone()); } },
            {children}
        }
    }
}

/// Native dialog surface with the upstream size, backdrop and non-blocking options.
///
/// No `open` attribute is authored: [`ModalTrigger`] invokes the browser's real
/// modal API. `ondismissrequest` replaces upstream `beforeDismiss` with an explicit
/// allow/deny request that can safely outlive an asynchronous operation.
#[component]
pub fn Modal(
    modal: ModalHandle,
    label: String,
    children: Element,
    #[props(default)] size: ModalSize,
    #[props(default)] centered: bool,
    #[props(default)] hide_backdrop: bool,
    #[props(default)] close_on_backdrop_click: bool,
    #[props(default)] disable_animation: bool,
    #[props(default)] is_non_blocking: bool,
    #[props(default)] aria_describedby: Option<String>,
    #[props(default)] ondialogclose: Option<EventHandler<String>>,
    #[props(default)] ondialogdismiss: Option<EventHandler<ModalDismissReason>>,
    #[props(default)] onresult: Option<EventHandler<ModalResult>>,
    #[props(default)] ondismissrequest: Option<EventHandler<ModalDismissRequest>>,
    #[props(default)] onopenchange: Option<EventHandler<bool>>,
) -> Element {
    use_context_provider(|| modal);
    let mut callbacks = modal.callbacks;
    let mut non_blocking = modal.non_blocking;
    use_effect(use_reactive(
        (
            &ondialogclose,
            &ondialogdismiss,
            &onresult,
            &ondismissrequest,
            &onopenchange,
            &is_non_blocking,
        ),
        move |(onclose, ondismiss, onresult, ondismissrequest, onopenchange, is_non_blocking)| {
            callbacks.set(ModalCallbacks {
                onclose,
                ondismiss,
                onresult,
                ondismissrequest,
                onopenchange,
            });
            non_blocking.set(is_non_blocking);
        },
    ));
    let id = modal.id();
    let generation = (modal.surface.generation)();
    let mut mouse_down_outside = use_signal(|| false);
    rsx! {
        for instance in [generation] {
            dialog { key: "{instance}", id: id.clone(), class: "ix-modal ix-modal-size-{size.as_str()}",
                "data-surface-generation": "{instance}",
                "aria-label": label.clone(), "aria-describedby": aria_describedby.clone(),
                "aria-modal": (!is_non_blocking).to_string(), "closedby": "closerequest",
                "popover": if is_non_blocking { Some("manual") } else { None },
                "data-centered": centered.to_string(), "data-hide-backdrop": hide_backdrop.to_string(), "data-no-animation": disable_animation.to_string(),
                onmounted: move |event| modal.surface.mounted(instance, event.data()),
                onbeforetoggle: move |_| {
                    if *modal.surface.generation.peek() != instance || *modal.surface.open.peek() { return; }
                    modal.opened();
                },
                ontoggle: move |_| {
                    spawn(async move {
                        let Some(visible) = modal.surface.native_visibility(instance).await else { return; };
                        let mut surface = modal.surface;
                        let previous = *surface.open.peek();
                        if previous == visible { return; }
                        surface.open.set(visible);
                        let callbacks = *modal.callbacks.peek();
                        if visible { let mut result = modal.result; result.set(None); }
                        else {
                            modal.surface.cleanup_temporary_focus_id();
                            let mut result = modal.result; result.set(Some(ModalResult::NativeClosed));
                            let mut pending = modal.pending; pending.set(None);
                            if let Some(callback) = callbacks.onresult { callback.call(ModalResult::NativeClosed); }
                        }
                        if let Some(callback) = callbacks.onopenchange { callback.call(visible); }
                    });
                },
                oncancel: move |event| { event.prevent_default(); modal.dismiss(ModalDismissReason::Escape); },
                onkeydown: move |event| if is_non_blocking && event.key() == Key::Escape { event.prevent_default(); event.stop_propagation(); modal.dismiss(ModalDismissReason::Escape); },
                onmousedown: move |event| {
                    if !close_on_backdrop_click || is_non_blocking { return; }
                    let point = event.client_coordinates();
                    let node = modal.surface.node.peek().clone();
                    spawn(async move { if let Some(node) = node { if let Ok(rect) = node.get_client_rect().await { mouse_down_outside.set(!point_in_bounds(point.x, point.y, rect.origin.x, rect.origin.y, rect.size.width, rect.size.height)); } } });
                },
                onmouseup: move |event| {
                    if !close_on_backdrop_click || is_non_blocking || !mouse_down_outside() { return; }
                    let point = event.client_coordinates(); let node = modal.surface.node.peek().clone();
                    spawn(async move { if let Some(node) = node { if let Ok(rect) = node.get_client_rect().await { if !point_in_bounds(point.x, point.y, rect.origin.x, rect.origin.y, rect.size.width, rect.size.height) { modal.dismiss(ModalDismissReason::Backdrop); } } } });
                },
                div { class: "ix-modal-panel", onmousedown: move |event| { mouse_down_outside.set(false); event.stop_propagation(); }, onmouseup: move |event| event.stop_propagation(),
                    {children.clone()}
                }
            }
        }
    }
}

fn point_in_bounds(x: f64, y: f64, left: f64, top: f64, width: f64, height: f64) -> bool {
    x >= left && x <= left + width && y >= top && y <= top + height
}

/// Modal title, optional leading pictogram and dismiss button.
#[component]
pub fn ModalHeader(
    children: Element,
    #[props(default)] hide_close: bool,
    #[props(default)] icon: Option<String>,
    #[props(default)] icon_color: Option<String>,
    #[props(default = "Close modal".to_string())] close_label: String,
    #[props(default)] oncloseclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let modal = try_consume_context::<ModalHandle>();
    rsx! {
        header { class: "ix-modal-header",
            if let Some(icon) = icon { span { class: "ix-modal-header-icon", style: icon_color.map(|name| format!("color:var({name})")), crate::Icon { name: icon, size: 24 } } }
            h2 { class: "ix-modal-title", {children} }
            if !hide_close {
                button { class: "ix-dialog-close", r#type: "button", "aria-label": close_label, disabled: modal.is_none(),
                    onclick: move |event| { if let Some(callback) = oncloseclick { callback.call(event.clone()); }
                    if event.default_action_enabled() { if let Some(modal) = modal { modal.dismiss(ModalDismissReason::Header); } } },
                    crate::Icon { name: "close".to_string(), size: 24 }
                }
            }
        }
    }
}

/// Scrollable modal message/form content.
#[component]
pub fn ModalContent(children: Element) -> Element {
    rsx! { div { class: "ix-modal-content", {children} } }
}

/// End-aligned modal actions.
#[component]
pub fn ModalFooter(children: Element) -> Element {
    rsx! { footer { class: "ix-modal-footer", {children} } }
}

/// Loading status to compose inside a modal while an application task runs.
#[component]
pub fn ModalLoading(children: Element) -> Element {
    rsx! { div { class: "ix-modal-loading", role: "status", "aria-busy": "true", crate::Spinner { variant: crate::SpinnerVariant::Primary } span { {children} } } }
}

/// Controller for a native declarative popover. Create with [`use_popover`].
#[derive(Clone, Copy, PartialEq)]
pub struct PopoverHandle {
    surface: Surface,
    onshowchanged: Signal<Option<EventHandler<bool>>>,
    reported_open: Signal<bool>,
    hover_mode: Signal<bool>,
    hover_sequence: Signal<u64>,
    suppress_focus_open: Signal<bool>,
    cancelled_native_close: Signal<bool>,
    restoring_native_close: Signal<bool>,
    onshowchange: Signal<Option<EventHandler<ToggleEvent>>>,
    dispatching_change: Signal<bool>,
}

/// Create a stable popover controller in a component.
pub fn use_popover() -> PopoverHandle {
    PopoverHandle {
        surface: use_surface("popover"),
        onshowchanged: use_signal(|| None),
        reported_open: use_signal(|| false),
        hover_mode: use_signal(|| false),
        hover_sequence: use_signal(|| 0),
        suppress_focus_open: use_signal(|| false),
        cancelled_native_close: use_signal(|| false),
        restoring_native_close: use_signal(|| false),
        onshowchange: use_signal(|| None),
        dispatching_change: use_signal(|| false),
    }
}

impl PopoverHandle {
    /// Show the real native top-layer popover, preserving the current focus target
    /// for restoration on dismissal. An opening callback may prevent this.
    pub async fn show(self) -> Result<(), NativeSurfaceError> {
        self.show_native(true).await
    }

    async fn show_native(mut self, remember_focus: bool) -> Result<(), NativeSurfaceError> {
        self.cancel_hover_hide();
        if self.is_open() {
            return Ok(());
        }
        let generation = *self.surface.generation.peek();
        self.surface.native_open(false, remember_focus).await?;
        if *self.surface.generation.peek() != generation {
            return Err(NativeSurfaceError::Superseded);
        }
        self.surface.open.set(true);
        Ok(())
    }

    fn cancel_hover_hide(mut self) {
        let next = self.hover_sequence.peek().wrapping_add(1);
        self.hover_sequence.set(next);
    }

    fn schedule_hover_hide(self, from_focus: bool) {
        if !*self.hover_mode.peek() {
            return;
        }
        self.cancel_hover_hide();
        let sequence = *self.hover_sequence.peek();
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_millis(150)).await;
            if *self.hover_sequence.peek() != sequence || !self.is_open() {
                return;
            }
            if from_focus && self.surface.contains_focus().await {
                return;
            }
            self.hide_with_focus(false);
        });
    }

    /// Native popover ID.
    pub fn id(self) -> String {
        self.surface.id()
    }
    /// Most recently observed native visibility.
    pub fn is_open(self) -> bool {
        (self.surface.open)()
    }
    /// Hide and restore invoker focus without an imperative DOM call.
    pub fn hide(self) {
        self.hide_with_focus(true);
    }

    fn hide_with_focus(mut self, restore_focus: bool) {
        let was_open = self.is_open();
        if !was_open || *self.dispatching_change.peek() {
            return;
        }
        let callback = *self.onshowchange.peek();
        if let Some(callback) = callback {
            let event = Event::new(Rc::new(ToggleData::new(LocalToggle)), false);
            self.dispatching_change.set(true);
            callback.call(event.clone());
            self.dispatching_change.set(false);
            if !event.default_action_enabled() {
                return;
            }
        }
        self.cancel_hover_hide();
        self.suppress_focus_open
            .set(restore_focus && *self.hover_mode.peek());
        self.cancelled_native_close.set(false);
        self.surface.replace(restore_focus);
        self.reported_open.set(false);
        if was_open {
            if let Some(callback) = *self.onshowchanged.peek() {
                callback.call(false);
            }
        }
    }
}

struct LocalToggle;

impl dioxus::html::HasToggleData for LocalToggle {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// How a popover trigger opens its floating panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverTriggerMode {
    /// Activate by click or the native keyboard button action.
    #[default]
    Click,
    /// Open on pointer hover or focus; leaving both surfaces dismisses after 150ms.
    Hover,
}

/// Native popover invoker with an implicit CSS anchor and accessible relationship.
#[component]
pub fn PopoverTrigger(
    popover: PopoverHandle,
    children: Element,
    /// Exact accessible name for the native invoker, including an empty override.
    #[props(default)]
    aria_label: Option<String>,
    #[props(default)] variant: crate::ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] trigger_mode: PopoverTriggerMode,
) -> Element {
    let id = popover.id();
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    let mut hover_mode = popover.hover_mode;
    use_effect(use_reactive((&trigger_mode,), move |(mode,)| {
        hover_mode.set(mode == PopoverTriggerMode::Hover)
    }));
    rsx! {
        button { class: "ix-button ix-button-{variant.as_str()}", r#type: "button", disabled,
            "popovertarget": id.clone(), "popovertargetaction": "toggle", "aria-haspopup": "dialog",
            style: "anchor-name:--{id}",
            "aria-controls": id.clone(), "aria-expanded": popover.is_open().to_string(), "aria-label": aria_label,
            onmounted: move |event| { trigger.set(Some(event.data())); },
            onclick: move |event| {
                popover.surface.remember_invoker(trigger.peek().clone());
                if trigger_mode == PopoverTriggerMode::Hover {
                    event.prevent_default();
                    spawn(async move { let _ = popover.show_native(false).await; });
                } else if popover.is_open() { event.prevent_default(); popover.hide(); }
            },
            onmouseenter: move |_| if !disabled && trigger_mode == PopoverTriggerMode::Hover {
                popover.surface.remember_invoker(trigger.peek().clone());
                spawn(async move { let _ = popover.show_native(false).await; });
            },
            onmouseleave: move |_| popover.schedule_hover_hide(false),
            onfocus: move |_| if !disabled && trigger_mode == PopoverTriggerMode::Hover {
                let mut suppress = popover.suppress_focus_open;
                if *suppress.peek() { suppress.set(false); return; }
                popover.surface.remember_invoker(trigger.peek().clone());
                spawn(async move { let _ = popover.show_native(false).await; });
            },
            onblur: move |_| popover.schedule_hover_hide(true),
            {children}
        }
    }
}

/// Preferred side of the trigger. The browser may flip placement to stay onscreen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverPlacement {
    /// Above trigger.
    Top,
    /// Below trigger.
    #[default]
    Bottom,
    /// Left of trigger.
    Left,
    /// Right of trigger.
    Right,
}

impl PopoverPlacement {
    /// CSS anchor position-area token.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// Native non-modal floating panel. Clicking its trigger toggles it; Escape closes
/// while focus is inside. `close_on_click_outside` uses native auto-popover dismissal.
/// CSS Anchor Positioning is required for placement; older engines center the panel.
/// Tab/Shift+Tab remain inside the visible panel. Canceling native outside closure
/// restores the same popover through its native DOM API; the browser may briefly
/// remove it from the top layer before restoration.
#[component]
pub fn Popover(
    popover: PopoverHandle,
    label: String,
    children: Element,
    #[props(default)] placement: PopoverPlacement,
    #[props(default)] has_spike: bool,
    #[props(default)] close_on_click_outside: bool,
    #[props(default)] onshowchange: Option<EventHandler<ToggleEvent>>,
    #[props(default)] onshowchanged: Option<EventHandler<bool>>,
) -> Element {
    use_context_provider(|| popover);
    let id = popover.id();
    let generation = (popover.surface.generation)();
    let mut callback = popover.onshowchanged;
    let mut before_callback = popover.onshowchange;
    use_effect(use_reactive(
        (&onshowchanged, &onshowchange),
        move |(handler, before)| {
            callback.set(handler);
            before_callback.set(before);
        },
    ));
    rsx! {
        for instance in [generation] {
            div { key: "{instance}", id: id.clone(), class: "ix-popover", role: "dialog", "aria-label": label.clone(), tabindex: "-1",
                "data-surface-generation": "{instance}",
                "popover": if close_on_click_outside { "auto" } else { "manual" },
                "data-placement": placement.as_str(), "data-spike": has_spike.to_string(),
                style: "position-anchor:--{id};position-area:{placement.as_str()}",
                onmounted: move |event| popover.surface.mounted(instance, event.data()),
                onbeforetoggle: move |event| {
                    if !*popover.restoring_native_close.peek() {
                        if let Some(callback) = onshowchange {
                            let mut dispatching = popover.dispatching_change;
                            dispatching.set(true);
                            callback.call(event.clone());
                            dispatching.set(false);
                        }
                    }
                    if *popover.surface.open.peek() && !event.default_action_enabled() {
                        let mut cancelled = popover.cancelled_native_close;
                        cancelled.set(true);
                    }
                    if event.default_action_enabled() && *popover.surface.generation.peek() == instance && !*popover.surface.open.peek() {
                        let mut surface = popover.surface; surface.open.set(true);
                    }
                },
                ontoggle: move |_| { spawn(async move {
                    let Some(visible) = popover.surface.native_visibility(instance).await else { return; };
                    if !visible && *popover.cancelled_native_close.peek() {
                        let mut cancelled = popover.cancelled_native_close;
                        let mut restoring = popover.restoring_native_close;
                        cancelled.set(false);
                        restoring.set(true);
                        let restored = popover.surface.native_open(false, false).await.is_ok();
                        restoring.set(false);
                        if restored { return; }
                    }
                    let mut surface = popover.surface; let previous = *popover.reported_open.peek();
                    if !visible { popover.surface.cleanup_temporary_focus_id(); popover.surface.cleanup_native_position(); }
                    surface.open.set(visible);
                    let mut reported = popover.reported_open; reported.set(visible);
                    if previous != visible { if let Some(callback) = onshowchanged { callback.call(visible); } }
                }); },
                onkeydown: move |event| match event.key() {
                    Key::Escape => { event.prevent_default(); event.stop_propagation(); popover.hide(); }
                    Key::Tab if cfg!(any(feature = "web", feature = "desktop")) => {
                        event.prevent_default(); event.stop_propagation();
                        spawn(async move { popover.surface.advance_focus(event.modifiers().shift()).await; });
                    }
                    _ => {}
                },
                onmouseenter: move |_| popover.cancel_hover_hide(),
                onmouseleave: move |_| popover.schedule_hover_hide(false),
                onfocusin: move |_| popover.cancel_hover_hide(),
                onfocusout: move |_| popover.schedule_hover_hide(true),
                div { class: "ix-popover-body", {children.clone()} }
                if has_spike { span { class: "ix-popover-spike", "aria-hidden": "true" } }
            }
        }
    }
}

/// Popover title with optional icon, additional items and cancelable close action.
#[component]
pub fn PopoverHeader(
    children: Element,
    #[props(default)] icon: Option<String>,
    #[props(default)] icon_color: Option<String>,
    #[props(default)] additional_items: Option<Element>,
    #[props(default)] hide_close: bool,
    #[props(default = "Close".to_string())] close_label: String,
    #[props(default)] oncloseclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let popover = try_consume_context::<PopoverHandle>();
    rsx! {
        header { class: "ix-popover-header",
            if let Some(icon) = icon { span { style: icon_color.map(|name| format!("color:var({name})")), crate::Icon { name: icon, size: 24 } } }
            h3 { class: "ix-popover-title", {children} }
            {additional_items}
            if !hide_close { button { class: "ix-dialog-close", r#type: "button", "aria-label": close_label, disabled: popover.is_none(), autofocus: true,
                onclick: move |event| { if let Some(callback) = oncloseclick { callback.call(event.clone()); }
                    if event.default_action_enabled() { if let Some(popover) = popover { popover.hide(); } } },
                crate::Icon { name: "close".to_string(), size: 24 }
            } }
        }
    }
}

/// Scrollable body content. `no_padding` supports edge-to-edge content.
#[component]
pub fn PopoverContent(children: Element, #[props(default)] no_padding: bool) -> Element {
    rsx! { div { class: "ix-popover-content", "data-no-padding": no_padding.to_string(), {children} } }
}

/// Popover action orientation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverFooterAlignment {
    /// Actions in a row.
    #[default]
    Horizontal,
    /// Full-width actions stacked vertically.
    Vertical,
}

/// End-aligned action buttons with optional leading metadata.
#[component]
pub fn PopoverFooter(
    children: Element,
    #[props(default)] start: Option<Element>,
    #[props(default)] alignment: PopoverFooterAlignment,
) -> Element {
    rsx! { footer { class: "ix-popover-footer", "data-vertical": (alignment == PopoverFooterAlignment::Vertical).to_string(),
        div { class: "ix-popover-footer-start", {start} } div { class: "ix-popover-footer-actions", {children} }
    } }
}

/// Optional popover image. An absent URL renders nothing; pass descriptive `image_alt`
/// for content images and an empty string only when the image is decorative.
#[component]
pub fn PopoverImage(
    #[props(default)] image: Option<String>,
    #[props(default)] image_alt: String,
) -> Element {
    rsx! { if let Some(image) = image.filter(|value| !value.is_empty()) { img { class: "ix-popover-image", src: image, alt: image_alt } } }
}

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-modal{display:none;position:fixed;box-sizing:border-box;left:50%;top:40px;transform:translateX(-50%);margin:0;padding:24px;border:0;border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:inherit;box-shadow:0 12px 40px #0005;width:360px;max-width:calc(100vw - 32px);max-height:calc(100dvh - 80px);overflow:hidden}
.ix-theme .ix-modal[open],.ix-theme .ix-modal:popover-open{display:flex;flex-direction:column}.ix-theme .ix-modal::backdrop{background:#0008}.ix-theme .ix-modal[data-hide-backdrop=true]::backdrop{background:transparent}.ix-theme .ix-modal[data-centered=true]{top:50%;transform:translate(-50%,-50%)}
.ix-theme .ix-modal-size-480{width:480px}.ix-theme .ix-modal-size-600{width:600px}.ix-theme .ix-modal-size-720{width:720px}.ix-theme .ix-modal-size-840{width:840px}.ix-theme .ix-modal-size-full-width{width:95vw}.ix-theme .ix-modal-size-full-screen{inset:0;transform:none!important;width:100vw;height:100dvh;max-width:none;max-height:none;border-radius:0;box-shadow:none}
.ix-theme .ix-modal-panel{display:flex;flex-direction:column;min-height:0;max-height:100%;overflow:hidden;flex:1}.ix-theme .ix-modal-header{display:flex;align-items:start;gap:16px;flex:none;margin-bottom:16px}.ix-theme .ix-modal-header-icon{display:flex;align-items:center;min-height:32px}.ix-theme .ix-modal-title{flex:1;min-width:0;font:600 1.25rem/1.5 var(--ix-font);margin:0;overflow-wrap:anywhere}.ix-theme .ix-dialog-close{display:inline-flex;align-items:center;justify-content:center;width:32px;height:32px;flex:none;border:0;background:transparent;padding:4px;color:var(--ix-color-text-secondary);cursor:pointer;border-radius:var(--ix-radius)}.ix-theme .ix-dialog-close:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-dialog-close:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-theme .ix-dialog-close:disabled{opacity:.45;cursor:not-allowed}
.ix-theme .ix-modal-content{overflow:auto;min-height:0;flex:1;font-size:.875rem;line-height:1.5}.ix-theme .ix-modal-content>:first-child{margin-top:0}.ix-theme .ix-modal-content>:last-child{margin-bottom:0}.ix-theme .ix-modal-footer{display:flex;align-items:center;justify-content:flex-end;flex-wrap:wrap;gap:8px;flex:none;margin-top:24px}.ix-theme .ix-modal-loading{display:flex;align-items:center;gap:16px;min-height:48px;font-size:.875rem}.ix-theme .ix-modal:has(.ix-modal-header-icon) .ix-modal-content,.ix-theme .ix-modal:has(.ix-modal-header-icon) .ix-modal-footer{margin-inline-start:40px}
.ix-theme .ix-popover{display:none;position:fixed;box-sizing:border-box;inset:auto;margin:12px;padding:0;border:0;border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:inherit;width:400px;min-width:min(240px,calc(100vw - 32px));max-width:min(600px,calc(100vw - 32px));max-height:calc(100dvh - 32px);overflow:visible;box-shadow:0 8px 28px #0004;position-try-fallbacks:flip-block,flip-inline,flip-block flip-inline,--ix-popover-viewport}
.ix-theme .ix-popover:popover-open{display:block}.ix-theme .ix-popover-body{display:flex;flex-direction:column;max-height:calc(100dvh - 32px);overflow:hidden;border-radius:inherit;position:relative;z-index:1;background:inherit}.ix-theme .ix-popover-header{display:flex;align-items:center;gap:8px;padding:16px;flex:none}.ix-theme .ix-popover-title{font:600 1rem/1.5 var(--ix-font);margin:0;flex:1;min-width:0;overflow-wrap:anywhere}.ix-theme .ix-popover-content{padding:16px;overflow:auto;font-size:.875rem;line-height:1.5;flex:1;min-height:0}.ix-theme .ix-popover-content[data-no-padding=true]{padding:0}.ix-theme .ix-popover-image{display:block;width:100%;height:auto;max-height:240px;object-fit:cover;flex:none}
.ix-theme .ix-popover-footer{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:16px;padding:16px;flex:none;min-width:0}.ix-theme .ix-popover-footer-start{font-size:.75rem;color:var(--ix-color-text-secondary);min-width:0;overflow-wrap:anywhere}.ix-theme .ix-popover-footer-actions{display:flex;align-items:center;justify-content:end;flex-wrap:wrap;gap:8px;min-width:0;max-width:100%;margin-inline-start:auto}.ix-theme .ix-popover-footer-actions>.ix-button{max-width:100%;white-space:normal;overflow-wrap:anywhere;min-height:32px;height:auto}.ix-theme .ix-popover-footer[data-vertical=true] .ix-popover-footer-actions{align-items:stretch;flex-direction:column;width:100%}
.ix-theme .ix-popover[data-unanchored=true]{position-anchor:auto;position-area:none;inset:50% auto auto 50%;transform:translate(-50%,-50%);margin:0}
.ix-theme .ix-popover-spike{position:absolute;width:12px;height:12px;background:inherit;transform:rotate(45deg);z-index:0;pointer-events:none}.ix-theme .ix-popover[data-placement=bottom]>.ix-popover-spike{top:-6px;left:calc(50% - 6px)}.ix-theme .ix-popover[data-placement=top]>.ix-popover-spike{bottom:-6px;left:calc(50% - 6px)}.ix-theme .ix-popover[data-placement=left]>.ix-popover-spike{right:-6px;top:calc(50% - 6px)}.ix-theme .ix-popover[data-placement=right]>.ix-popover-spike{left:-6px;top:calc(50% - 6px)}
@position-try --ix-popover-viewport{position-area:none;inset:16px;margin:auto;width:min(400px,calc(100% - 32px));height:max-content;max-width:calc(100% - 32px);max-height:calc(100dvh - 32px)}
@supports not(position-anchor:--example){.ix-theme .ix-popover{inset:50% auto auto 50%;transform:translate(-50%,-50%);margin:0}}
@media(max-width:576px){.ix-theme .ix-modal{top:16px;max-height:calc(100dvh - 32px);padding:16px}.ix-theme .ix-modal[data-centered=true]{top:50%}.ix-theme .ix-modal.ix-modal-size-full-screen{inset:0;max-height:none}.ix-theme .ix-modal-footer>.ix-button{flex:1}.ix-theme .ix-modal:has(.ix-modal-header-icon) .ix-modal-content,.ix-theme .ix-modal:has(.ix-modal-header-icon) .ix-modal-footer{margin-inline-start:0}}
@media(prefers-reduced-motion:no-preference){.ix-theme .ix-modal[open]:not([data-no-animation=true]) .ix-modal-panel{animation:ix-modal-appear 140ms ease-out}@keyframes ix-modal-appear{from{transform:translateY(-8px);opacity:.5}to{transform:translateY(0);opacity:1}}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn backdrop_coordinates_keep_the_panel_edges_inside() {
        assert!(point_in_bounds(10.0, 20.0, 10.0, 20.0, 100.0, 80.0));
        assert!(point_in_bounds(110.0, 100.0, 10.0, 20.0, 100.0, 80.0));
        assert!(!point_in_bounds(9.9, 50.0, 10.0, 20.0, 100.0, 80.0));
        assert!(!point_in_bounds(50.0, 100.1, 10.0, 20.0, 100.0, 80.0));
    }

    fn rendering_fixture() -> Element {
        let modal = use_modal();
        let popover = use_popover();
        rsx! {
            ModalTrigger { modal, "Open modal" }
            Modal { modal, label: "Confirm resource creation",
                ModalHeader { "Create Resource" }
                ModalContent { "Resource details" }
                ModalFooter { ModalClose { value: "created", "Create" } }
            }
            PopoverTrigger { popover, "Open popover" }
            Popover { popover, label: "Release highlights", close_on_click_outside: true,
                PopoverHeader { "Release highlights" }
                PopoverContent { "Release details" }
            }
        }
    }

    #[test]
    fn native_invokers_reference_closed_accessibly_named_surfaces() {
        let mut dom = VirtualDom::new(rendering_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("command=\"show-modal\""));
        let target = html
            .split("commandfor=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert!(html.contains(&format!("id=\"{target}\"")));
        let dialog_tag = html
            .split("<dialog")
            .nth(1)
            .unwrap()
            .split('>')
            .next()
            .unwrap();
        assert!(dialog_tag.contains("aria-modal=\"true\""));
        assert!(dialog_tag.contains("aria-label=\"Confirm resource creation\""));
        assert!(dialog_tag.contains("data-surface-generation=\"0\""));
        assert!(
            !dialog_tag
                .split_whitespace()
                .any(|attribute| attribute == "open" || attribute.starts_with("open="))
        );
        let popover_target = html
            .split("popovertarget=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert!(html.contains(&format!("id=\"{popover_target}\"")));
        assert!(html.contains("popover=\"auto\""));
        assert!(html.contains("aria-label=\"Release highlights\""));
        assert_eq!(html.matches("data-surface-generation=\"0\"").count(), 2);
        assert!(!html.contains("<script"));
    }

    fn controller_fixture() -> Element {
        let controller = use_modal();
        let capture = consume_context::<Rc<RefCell<Option<ModalHandle>>>>();
        *capture.borrow_mut() = Some(controller);
        VNode::empty()
    }

    fn popover_controller_fixture() -> Element {
        let controller = use_popover();
        let capture = consume_context::<Rc<RefCell<Option<PopoverHandle>>>>();
        *capture.borrow_mut() = Some(controller);
        VNode::empty()
    }

    #[test]
    fn popover_hide_is_cancelable_and_reentrant_requests_are_coalesced() {
        let capture = Rc::new(RefCell::new(None));
        let mut dom =
            VirtualDom::new(popover_controller_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let mut popover: PopoverHandle = capture.borrow().unwrap();
        let completions = Rc::new(RefCell::new(Vec::new()));
        dom.in_scope(ScopeId::ROOT, || {
            popover.surface.open.set(true);
            popover.reported_open.set(true);
            let generation = *popover.surface.generation.peek();
            popover
                .onshowchange
                .set(Some(EventHandler::new(|event: ToggleEvent| {
                    event.prevent_default()
                })));
            let results = completions.clone();
            popover
                .onshowchanged
                .set(Some(EventHandler::new(move |visible| {
                    results.borrow_mut().push(visible)
                })));
            popover.hide();
            assert!(
                popover.is_open(),
                "a rejected hide must retain the live surface"
            );
            assert_eq!(*popover.surface.generation.peek(), generation);
            assert!(completions.borrow().is_empty());
            popover
                .onshowchange
                .set(Some(EventHandler::new(move |_| popover.hide())));
            popover.hide();
            assert!(!popover.is_open());
            assert_eq!(*popover.surface.generation.peek(), generation + 1);
            assert_eq!(
                &*completions.borrow(),
                &[false],
                "one completion for a reentrant hide"
            );
            popover.hide();
            assert_eq!(completions.borrow().len(), 1);
        });
    }

    #[test]
    fn imperative_opening_before_mount_returns_error_without_changing_controller_state() {
        use std::future::Future;
        use std::task::{Context, Poll, Waker};
        let capture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let modal: ModalHandle = capture.borrow().unwrap();
        dom.in_scope(ScopeId::ROOT, || {
            let mut operation = std::pin::pin!(modal.open());
            let mut context = Context::from_waker(Waker::noop());
            assert!(matches!(
                operation.as_mut().poll(&mut context),
                Poll::Ready(Err(NativeSurfaceError::NotMounted))
            ));
            assert!(!modal.is_open());
            assert!(modal.result().is_none());
        });
    }

    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    #[test]
    fn a_delayed_native_open_reply_cannot_restore_a_closed_generation_or_its_focus_target() {
        use dioxus::document::{Document, Eval, EvalError, Evaluator};
        use std::future::Future;
        use std::task::{Context, Poll, Waker};

        struct DelayedNativeReply {
            reply: Rc<RefCell<Option<serde_json::Value>>>,
            acknowledgements: Rc<std::cell::Cell<usize>>,
        }
        impl Evaluator for DelayedNativeReply {
            fn poll_join(
                &mut self,
                _: &mut Context<'_>,
            ) -> Poll<Result<serde_json::Value, EvalError>> {
                // The shared one-shot transport receives before acknowledging;
                // a completed desktop evaluation's direct return may be gone.
                Poll::Ready(Err(EvalError::Finished))
            }
            fn poll_recv(
                &mut self,
                _: &mut Context<'_>,
            ) -> Poll<Result<serde_json::Value, EvalError>> {
                match self.reply.borrow_mut().take() {
                    Some(reply) => Poll::Ready(Ok(serde_json::json!({
                        "value": reply, "error": null
                    }))),
                    None => Poll::Pending,
                }
            }
            fn send(&self, message: serde_json::Value) -> Result<(), EvalError> {
                assert_eq!(message, serde_json::json!({"kind": "finish"}));
                self.acknowledgements.set(self.acknowledgements.get() + 1);
                Ok(())
            }
        }
        struct DelayedDocument {
            owner: dioxus::signals::Owner,
            reply: Rc<RefCell<Option<serde_json::Value>>>,
            acknowledgements: Rc<std::cell::Cell<usize>>,
        }
        impl Document for DelayedDocument {
            fn eval(&self, _: String) -> Eval {
                Eval::new(self.owner.insert(Box::new(DelayedNativeReply {
                    reply: self.reply.clone(),
                    acknowledgements: self.acknowledgements.clone(),
                }) as Box<dyn Evaluator>))
            }
        }

        let capture = Rc::new(RefCell::new(None));
        let reply = Rc::new(RefCell::new(None));
        let acknowledgements = Rc::new(std::cell::Cell::new(0));
        let document: Rc<dyn Document> = Rc::new(DelayedDocument {
            owner: Default::default(),
            reply: reply.clone(),
            acknowledgements: acknowledgements.clone(),
        });
        let mut dom = VirtualDom::new(controller_fixture)
            .with_root_context(capture.clone())
            .with_root_context(document);
        dom.rebuild_in_place();
        let mut modal: ModalHandle = capture.borrow().unwrap();
        dom.in_scope(ScopeId::ROOT, || {
            modal.surface.node.set(Some(Rc::new(MountedData::new(()))));
            modal.surface.bridge_invoker.set(Some(BridgeFocusTarget {
                id: "retained-focus-target".into(),
                temporary: false,
            }));
            let mut operation = std::pin::pin!(modal.open());
            let mut context = Context::from_waker(Waker::noop());
            assert!(operation.as_mut().poll(&mut context).is_pending());
            assert_eq!(acknowledgements.get(), 0);
            // A native opening notification and application close can arrive
            // before the queued bridge result reaches this Rust task.
            modal.surface.open.set(true);
            modal.close("saved before native reply");
            *reply.borrow_mut() = Some(serde_json::json!({
                "opened": true, "error": null,
                "focus_target": { "id": "stale-focus-target", "temporary": false }
            }));
            assert!(matches!(
                operation.as_mut().poll(&mut context),
                Poll::Ready(Err(NativeSurfaceError::Superseded))
            ));
            assert_eq!(acknowledgements.get(), 1);
            assert!(!modal.is_open());
            assert_eq!(
                modal.result(),
                Some(ModalResult::Closed("saved before native reply".into()))
            );
            assert_eq!(
                modal.surface.bridge_invoker.peek().as_ref().unwrap().id,
                "retained-focus-target"
            );
        });
    }

    #[test]
    fn app_switch_native_dismiss_reconciles_early_open_and_rejects_duplicate_generation() {
        let capture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let mut modal: ModalHandle = capture.borrow().unwrap();
        dom.in_scope(ScopeId::ROOT, || {
            let generation = modal.surface_generation();
            assert!(!modal.dismiss_native_entry(generation, true), "unmounted");
            modal.surface.node.set(Some(Rc::new(MountedData::new(()))));
            assert!(
                !modal.dismiss_native_entry(generation, false),
                "native closed"
            );
            assert!(
                !modal.dismiss_native_entry(generation + 1, true),
                "wrong instance"
            );
            assert_eq!(modal.result(), None);

            // Native entry dispatch can precede the renderer's opening callback.
            assert!(!modal.is_open());
            assert!(modal.dismiss_native_entry(generation, true));
            assert!(!modal.is_open());
            assert_eq!(modal.surface_generation(), generation + 1);
            assert_eq!(
                modal.result(),
                Some(ModalResult::Dismissed(ModalDismissReason::Programmatic(
                    "app-switch".into()
                )))
            );
            assert!(
                !modal.dismiss_native_entry(generation, true),
                "already consumed"
            );
        });
    }

    #[test]
    fn app_switch_native_dismiss_rejects_guard_and_reentrant_open_supersession() {
        let capture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let mut modal: ModalHandle = capture.borrow().unwrap();
        dom.in_scope(ScopeId::ROOT, || {
            modal.surface.node.set(Some(Rc::new(MountedData::new(()))));
            modal.callbacks.set(ModalCallbacks {
                ondismissrequest: Some(EventHandler::new(|_| -> () {
                    panic!("private entry cannot use a deferred guard")
                })),
                ..Default::default()
            });
            let generation = modal.surface_generation();
            assert!(!modal.dismiss_native_entry(generation, true));
            assert!(!modal.is_open());
            assert!(!modal.is_dismiss_pending());

            modal.callbacks.set(ModalCallbacks {
                onopenchange: Some(EventHandler::new(move |open| {
                    if open {
                        modal.surface.replace(false);
                    }
                })),
                ..Default::default()
            });
            assert!(!modal.dismiss_native_entry(generation, true));
            assert_eq!(modal.surface_generation(), generation + 1);
            assert_eq!(
                modal.result(),
                None,
                "superseding callback was not overwritten"
            );
        });
    }

    #[test]
    fn app_switch_native_dismiss_orders_callbacks_and_rejects_pending_or_superseded_finish() {
        let capture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let mut modal: ModalHandle = capture.borrow().unwrap();
        let order = Rc::new(RefCell::new(Vec::new()));
        dom.in_scope(ScopeId::ROOT, || {
            modal.surface.node.set(Some(Rc::new(MountedData::new(()))));
            modal.opened();
            let visibility = order.clone();
            let dismissal = order.clone();
            let result = order.clone();
            modal.callbacks.set(ModalCallbacks {
                onopenchange: Some(EventHandler::new(move |open| {
                    assert!(!open);
                    visibility.borrow_mut().push("closed");
                })),
                ondismiss: Some(EventHandler::new(move |_| {
                    dismissal.borrow_mut().push("dismiss")
                })),
                onresult: Some(EventHandler::new(move |_| {
                    result.borrow_mut().push("result")
                })),
                ..Default::default()
            });
            assert!(modal.dismiss_native_entry(modal.surface_generation(), true));
            assert_eq!(&*order.borrow(), &["closed", "dismiss", "result"]);

            modal.callbacks.set(ModalCallbacks::default());
            modal.opened();
            modal.pending.set(Some(42));
            let generation = modal.surface_generation();
            assert!(!modal.dismiss_native_entry(generation, true));
            assert!(modal.is_open());
            assert_eq!(modal.result(), None);
            modal.pending.set(None);
            modal.callbacks.set(ModalCallbacks {
                onresult: Some(EventHandler::new(move |_| modal.surface.replace(false))),
                ..Default::default()
            });
            assert!(!modal.dismiss_native_entry(generation, true));
            assert_eq!(modal.surface_generation(), generation + 2);
        });
    }

    #[test]
    fn deferred_dismissal_is_coalesced_and_stale_decisions_cannot_close_a_reopened_dialog() {
        let capture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let mut modal: ModalHandle = capture.borrow().unwrap();
        let requests = Rc::new(RefCell::new(Vec::<ModalDismissRequest>::new()));
        dom.in_scope(ScopeId::ROOT, || {
            let received = requests.clone();
            modal.callbacks.set(ModalCallbacks {
                ondismissrequest: Some(EventHandler::new(move |request| {
                    received.borrow_mut().push(request)
                })),
                ..Default::default()
            });
            modal.surface.open.set(true);
            modal.dismiss(ModalDismissReason::Cancel);
            modal.dismiss(ModalDismissReason::Escape);
            assert_eq!(
                requests.borrow().len(),
                1,
                "one outstanding decision per dialog"
            );
            let request = requests.borrow_mut().pop().unwrap();
            assert!(modal.is_dismiss_pending());
            assert!(request.clone().deny());
            assert!(!modal.is_dismiss_pending());
            assert!(
                !request.allow(),
                "a denied request cannot later be accepted"
            );

            modal.dismiss(ModalDismissReason::Header);
            let stale_request = requests.borrow_mut().pop().unwrap();
            let id = modal.id();
            let old_generation = *modal.surface.generation.peek();
            modal.close("saved");
            assert_eq!(modal.result(), Some(ModalResult::Closed("saved".into())));
            let replacement_generation = *modal.surface.generation.peek();
            assert_eq!(
                *modal.surface.restore_after_mount.peek(),
                Some(replacement_generation)
            );
            modal
                .surface
                .mounted(old_generation, Rc::new(MountedData::new(())));
            assert!(
                modal.surface.node.peek().is_none(),
                "stale mount events must not replace the current surface reference"
            );
            assert_eq!(
                *modal.surface.restore_after_mount.peek(),
                Some(replacement_generation)
            );
            modal
                .surface
                .mounted(replacement_generation, Rc::new(MountedData::new(())));
            assert_eq!(
                *modal.surface.restore_after_mount.peek(),
                None,
                "focus restoration is consumed only by the closed replacement mount"
            );
            modal.surface.open.set(true);
            assert!(
                !stale_request.allow(),
                "a prior generation cannot close a newly opened surface"
            );
            assert!(modal.is_open());
            assert_eq!(
                modal.id(),
                id,
                "the declarative invoker's ID survives close/remount"
            );

            modal.dismiss(ModalDismissReason::Backdrop);
            assert!(requests.borrow_mut().pop().unwrap().allow());
            assert!(!modal.is_open());
            assert_eq!(
                modal.result(),
                Some(ModalResult::Dismissed(ModalDismissReason::Backdrop))
            );
            modal.callbacks.set(ModalCallbacks {
                ondismissrequest: Some(EventHandler::new(|request: ModalDismissRequest| {
                    assert!(request.allow());
                })),
                ..Default::default()
            });
            modal.surface.open.set(true);
            modal.dismiss(ModalDismissReason::Escape);
            assert!(!modal.is_open(), "guards may also approve synchronously");
        });
    }

    #[test]
    fn confirmation_bypasses_pending_guard_and_emits_each_payload_once() {
        let capture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(controller_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let mut modal: ModalHandle = capture.borrow().unwrap();
        let requests = Rc::new(RefCell::new(Vec::<ModalDismissRequest>::new()));
        let results = Rc::new(RefCell::new(Vec::<ModalResult>::new()));
        let confirmations = Rc::new(RefCell::new(Vec::<String>::new()));
        let dismissals = Rc::new(RefCell::new(Vec::<ModalDismissReason>::new()));
        dom.in_scope(ScopeId::ROOT, || {
            let received = requests.clone();
            let completed = results.clone();
            let confirmed = confirmations.clone();
            let dismissed = dismissals.clone();
            modal.callbacks.set(ModalCallbacks {
                ondismissrequest: Some(EventHandler::new(move |request| {
                    received.borrow_mut().push(request)
                })),
                onresult: Some(EventHandler::new(move |result| {
                    completed.borrow_mut().push(result)
                })),
                onclose: Some(EventHandler::new(move |payload| {
                    confirmed.borrow_mut().push(payload)
                })),
                ondismiss: Some(EventHandler::new(move |reason| {
                    dismissed.borrow_mut().push(reason)
                })),
                ..Default::default()
            });
            modal.surface.open.set(true);
            modal.dismiss(ModalDismissReason::Header);
            assert!(modal.is_dismiss_pending());
            modal.close("close payload!");
            assert!(!modal.is_dismiss_pending());
            assert!(!modal.is_open());
            assert!(!requests.borrow_mut().pop().unwrap().allow());
            modal.close("duplicate completion");
            modal.dismiss(ModalDismissReason::Cancel);
            assert_eq!(&*confirmations.borrow(), &["close payload!"]);
            assert_eq!(results.borrow().len(), 1);
            assert!(dismissals.borrow().is_empty());

            modal.surface.open.set(true);
            let reason = ModalDismissReason::Programmatic("dismiss payload".into());
            modal.dismiss(reason.clone());
            assert!(requests.borrow_mut().pop().unwrap().allow());
            assert!(!modal.is_open());
            assert_eq!(&*dismissals.borrow(), std::slice::from_ref(&reason));
            assert_eq!(
                &*results.borrow(),
                &[
                    ModalResult::Closed("close payload!".into()),
                    ModalResult::Dismissed(reason),
                ]
            );
        });
    }
}
