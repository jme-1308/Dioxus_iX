//! Native top-layer dialogs and popovers with Rust result and dismissal handling.
//!
//! Open dialogs with [`ModalTrigger`], which uses the HTML Invoker Commands API
//! (`command="show-modal"`). This invokes native modal behavior, including inert
//! background content, focus containment, Escape and return focus. It requires a
//! browser/WebView supporting invoker commands; there is no JavaScript fallback.
//! [`ModalHandle::close`] replaces the native dialog with a closed instance.
//! [`PopoverHandle::hide`] retains its native surface and nested component state.
//! Both restore registered Dioxus invokers through mounted element handles.
//!
//! Native top-layer visibility uses DOM methods on web and desktop. Programmatic
//! focus uses registered Dioxus mounted handles; native observations provide
//! candidate eligibility/current-focus facts only. Rust owns dismissal and hover state.
use dioxus::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(test)]
#[path = "modal_focus_tests.rs"]
mod modal_focus_tests;

#[cfg(test)]
#[path = "popover_lifetime_tests.rs"]
mod popover_lifetime_tests;

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
    invoker: Signal<Option<crate::overlay_runtime::MountedFocusTarget>>,
    restore_after_mount: Signal<Option<u64>>,
}

fn use_surface(prefix: &'static str) -> Surface {
    Surface {
        id: use_signal(|| format!("ix-{prefix}-{}", DIALOG_ID.fetch_add(1, Ordering::Relaxed))),
        generation: use_signal(|| 0),
        open: use_signal(|| false),
        node: use_signal(|| None),
        invoker: use_signal(|| None),
        restore_after_mount: use_signal(|| None),
    }
}

impl Surface {
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
        if let Some(invoker) = self.invoker.peek().clone() {
            spawn(async move {
                // The prior native dialog has now been removed. Restoring before
                // this mount lets browser dialog teardown overwrite the focus.
                if self
                    .generation
                    .try_peek()
                    .is_ok_and(|current| *current == generation)
                    && self.open.try_peek().is_ok_and(|open| !*open)
                    && let Some(node) = invoker.live_node()
                {
                    let _ = node.set_focus(true).await;
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

    // The caller queues this after Dioxus publishes the current generation.
    // Native code checks that rendered value rather than maintaining another
    // command counter; Rust still validates ownership before and after the call.
    async fn present_popover(self, generation: u64, shown: bool) -> Result<(), NativeSurfaceError> {
        let current = || {
            self.generation
                .try_peek()
                .is_ok_and(|value| *value == generation)
                && self.open.try_peek().is_ok_and(|value| *value == shown)
        };
        if !current() {
            return Err(NativeSurfaceError::Superseded);
        }
        if !self.node.try_peek().is_ok_and(|node| node.is_some()) {
            return Err(NativeSurfaceError::NotMounted);
        }
        let id = self.id();
        let marker = generation.to_string();
        let result = crate::native_surface::present(
            &id,
            shown,
            Some(("data-surface-generation", &marker)),
            None,
        )
        .await;
        if !current() {
            return Err(NativeSurfaceError::Superseded);
        }
        match result {
            Ok(true) => Ok(()),
            Ok(false) => Err(NativeSurfaceError::Superseded),
            Err(error) => Err(error),
        }
    }

    // MountedData has no native showModal/showPopover operation. Keep the
    // primary binding shared across targets; Dioxus owns keyed teardown/focus.
    async fn native_open(self, modal: bool) -> Result<(), NativeSurfaceError> {
        if self.node.peek().is_none() {
            return Err(NativeSurfaceError::NotMounted);
        }
        if !cfg!(any(
            all(feature = "web", target_arch = "wasm32"),
            all(feature = "desktop", not(target_arch = "wasm32"))
        )) {
            return Err(NativeSurfaceError::UnsupportedRenderer);
        }
        let generation = *self.generation.peek();
        let id = self.id();
        let marker = generation.to_string();
        let result = crate::native_surface::open_modal(
            &id,
            Some(("data-surface-generation", &marker)),
            modal,
        )
        .await;
        // A native notification and close can arrive before the reply.
        // Neither a replaced element nor a late result may revive it.
        if !self
            .generation
            .try_peek()
            .is_ok_and(|current| *current == generation)
        {
            return Err(NativeSurfaceError::Superseded);
        }
        match result {
            Ok(true) => Ok(()),
            Ok(false) => Err(NativeSurfaceError::Superseded),
            Err(error) => Err(error),
        }
    }

    fn remember_invoker(mut self, invoker: Option<crate::overlay_runtime::MountedFocusTarget>) {
        self.invoker.set(invoker);
    }

    async fn contains_focus(self) -> bool {
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            crate::focus_containment::read(&self.id(), None)
                .await
                .is_ok_and(|facts| facts.inside)
        }
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        false
    }
}

// queue_effect is the pinned Dioxus post-render boundary. Keep this one waiter
// local to its Rust future; cancellation leaves no native command to execute.
async fn popover_after_render() {
    let state = Rc::new(std::cell::RefCell::new((false, None::<std::task::Waker>)));
    let rendered = state.clone();
    dioxus::core::queue_effect(move || {
        let waker = {
            let mut state = rendered.borrow_mut();
            state.0 = true;
            state.1.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    });
    dioxus::core::needs_update();
    std::future::poll_fn(move |context| {
        let mut state = state.borrow_mut();
        if state.0 {
            std::task::Poll::Ready(())
        } else {
            state.1 = Some(context.waker().clone());
            std::task::Poll::Pending
        }
    })
    .await;
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
    focus_runtime: Signal<crate::overlay_runtime::RuntimeHandle>,
}

/// Create a stable dialog controller in a component, following the rules of hooks.
pub fn use_modal() -> ModalHandle {
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    ModalHandle {
        surface: use_surface("modal"),
        non_blocking: use_signal(|| false),
        callbacks: use_signal(ModalCallbacks::default),
        result: use_signal(|| None),
        pending: use_signal(|| None),
        request_sequence: use_signal(|| 0),
        focus_runtime: use_signal(move || runtime),
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
        if self.surface.node.peek().is_none() {
            return Err(NativeSurfaceError::NotMounted);
        }
        let generation = *self.surface.generation.peek();
        // Pinned MountedData can focus a handle but cannot query the current
        // active element. Read only its existing ID; resolve the Dioxus mount
        // before native activation. Unregistered controls get no JS fallback.
        #[cfg(any(
            all(feature = "web", target_arch = "wasm32"),
            all(feature = "desktop", not(target_arch = "wasm32"))
        ))]
        {
            let id = crate::dom_query::eval("return document.activeElement?.id||'';")
                .join::<String>()
                .await
                .ok();
            self.surface.remember_invoker(
                id.as_deref()
                    .and_then(|id| self.focus_runtime.peek().capture_registered_focus(id)),
            );
        }
        if *self.surface.generation.peek() != generation {
            return Err(NativeSurfaceError::Superseded);
        }
        self.surface.native_open(!*self.non_blocking.peek()).await?;
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

    #[cfg(test)]
    pub(crate) fn surface_generation(self) -> u64 {
        *self.surface.generation.peek()
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
    let trigger_id =
        use_hook(|| format!("{id}-trigger-{}", DIALOG_ID.fetch_add(1, Ordering::Relaxed)));
    let binding = crate::dropdown::use_item_host("native-opt-in");
    let mounted = binding.clone();
    let mount_id = trigger_id.clone();
    let focus_id = trigger_id.clone();
    rsx! {
        button { id:trigger_id, class: "ix-button ix-button-{variant.as_str()}", r#type: "button", disabled, autofocus,
            "commandfor": if non_blocking { None } else { Some(id.clone()) },
            "command": if non_blocking { None } else { Some("show-modal") },
            "popovertarget": if non_blocking { Some(id.clone()) } else { None },
            "popovertargetaction": if non_blocking { Some("show") } else { None },
            "aria-haspopup": "dialog", "aria-controls": id,
            onmounted: move |event| mounted.mount(&mount_id,event.data()),
            onclick: move |_| modal.surface.remember_invoker(binding.runtime.capture_registered_focus(&focus_id)),
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
    #[props(default)] icon_label: Option<String>,
    #[props(default = "Close modal".to_string())] close_label: String,
    #[props(default)] oncloseclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let modal = try_consume_context::<ModalHandle>();
    rsx! {
        header { class: "ix-modal-header",
            if let Some(icon) = icon { span { class: "ix-modal-header-icon", style: icon_color.map(|name| format!("color:var({name})")), crate::Icon { name: icon, size: 24, label:icon_label } } }
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

/// Cancelable Popover controller request with an exact requested visibility.
/// Unlike a native toggle observation, a close request can target an already
/// closed Popover. Both callback adapters share the same cancellation state.
#[derive(Clone)]
pub struct PopoverChangeEvent {
    /// Requested visibility, including false for an already closed instance.
    pub open: bool,
    /// Visibility immediately before this request.
    pub previous: bool,
    event: ToggleEvent,
}
impl PopoverChangeEvent {
    /// Prevent this controller operation before dismissal/presentation begins.
    pub fn prevent_default(&self) {
        self.event.prevent_default();
    }
    /// Whether either request callback has prevented this operation.
    pub fn default_prevented(&self) -> bool {
        !self.event.default_action_enabled()
    }
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
    onshowchange: Signal<Option<EventHandler<ToggleEvent>>>,
    onbeforechange: Signal<Option<EventHandler<PopoverChangeEvent>>>,
    dispatching_change: Signal<bool>,
    opening: Signal<bool>,
    pending_opening_focus: Signal<Option<u64>>,
    focusable_at_open: Signal<bool>,
    focus_trigger: Signal<Option<crate::overlay_runtime::MountedFocusTarget>>,
    trigger_id: Signal<Option<String>>,
    trigger_anchor: Signal<Option<PopoverAnchor>>,
    overlay: Signal<Option<crate::overlay_runtime::NodeHandle>>,
    trigger_ownership: Signal<Option<(u64, Vec<crate::overlay_runtime::NodeHandle>)>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PopoverAnchor {
    ownership: u64,
    visible: bool,
}
impl PopoverAnchor {
    fn name(self) -> String {
        format!("--ix-popover-trigger-{}", self.ownership)
    }
}

// A caller can drop its show future while the Popover itself remains mounted.
// Release only that operation's admission flag, including after owner teardown.
struct PopoverOpeningGuard {
    popover: PopoverHandle,
    generation: u64,
}

impl Drop for PopoverOpeningGuard {
    fn drop(&mut self) {
        if self
            .popover
            .surface
            .generation
            .try_peek()
            .ok()
            .is_some_and(|value| *value == self.generation)
        {
            if let Ok(mut opening) = self.popover.opening.try_write() {
                *opening = false;
            }
        }
    }
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
        onshowchange: use_signal(|| None),
        onbeforechange: use_signal(|| None),
        dispatching_change: use_signal(|| false),
        opening: use_signal(|| false),
        pending_opening_focus: use_signal(|| None),
        focusable_at_open: use_signal(|| false),
        focus_trigger: use_signal(|| None),
        trigger_id: use_signal(|| None),
        trigger_anchor: use_signal(|| None),
        overlay: use_signal(|| None),
        trigger_ownership: use_signal(|| None),
    }
}

impl PopoverHandle {
    fn sync_surface_metadata(self) {
        if let Some(node) = self.overlay.peek().clone() {
            let id = self.id();
            node.set_surface_metadata(crate::overlay_runtime::SurfaceMetadata {
                trigger_id: self.trigger_id.peek().clone().unwrap_or_default(),
                items_host: id.clone(),
                popover_host: id,
            });
        }
    }
    fn update_trigger_anchor(mut self, ownership: u64, target: &Rc<MountedData>, visible: bool) {
        if !self.trigger_anchor.try_peek().is_ok_and(|anchor| {
            anchor
                .as_ref()
                .is_some_and(|anchor| anchor.ownership == ownership)
        }) || !self.surface.invoker.try_peek().is_ok_and(|invoker| {
            invoker
                .as_ref()
                .is_some_and(|invoker| invoker.matches_node(target))
        }) {
            return;
        }
        let next = Some(PopoverAnchor { ownership, visible });
        if self.trigger_anchor.peek().as_ref() != next.as_ref() {
            self.trigger_anchor.set(next);
        }
    }
    async fn refresh_trigger_anchor(self) {
        let Some(anchor) = *self.trigger_anchor.peek() else {
            return;
        };
        let target = self
            .surface
            .invoker
            .peek()
            .as_ref()
            .and_then(crate::overlay_runtime::MountedFocusTarget::live_node);
        let Some(target) = target else {
            return;
        };
        let visible = target
            .get_client_rect()
            .await
            .is_ok_and(|bounds| bounds.size.width > 0.0 && bounds.size.height > 0.0);
        self.update_trigger_anchor(anchor.ownership, &target, visible);
    }

    /// Show the real native top-layer popover. Dismissal restores its configured
    /// trigger when the opening had focusable content. Opening is cancelable.
    /// Success confirms native presentation; initial focus is applied asynchronously
    /// from the shared canonical observation through registered mounted handles.
    pub async fn show(self) -> Result<(), NativeSurfaceError> {
        self.show_native().await
    }

    async fn show_native(mut self) -> Result<(), NativeSurfaceError> {
        self.cancel_hover_hide();
        if self.is_open() || *self.opening.peek() {
            return Ok(());
        }
        if self.surface.node.peek().is_none() {
            return Err(NativeSurfaceError::NotMounted);
        }
        if !self.request_change(true) {
            return Err(NativeSurfaceError::Prevented);
        }
        let node = self.overlay.peek().clone();
        if let Some(node) = &node {
            node.cancel_registered_focus();
            if !node.prepare_present() {
                return Err(NativeSurfaceError::Superseded);
            }
        }
        let generation = self.surface.generation.peek().wrapping_add(1);
        self.surface.generation.set(generation);
        self.suppress_focus_open.set(false);
        self.surface.restore_after_mount.set(None);
        self.opening.set(true);
        self.pending_opening_focus.set(None);
        self.focusable_at_open.set(false);
        self.focus_trigger.set(None);
        let _opening = PopoverOpeningGuard {
            popover: self,
            generation,
        };
        // Publish the current generation through Dioxus before native work.
        // This replaces the private native command counter and source-like yield.
        popover_after_render().await;
        if *self.surface.generation.peek() != generation
            || node.as_ref().is_some_and(|node| !node.is_registered())
        {
            return Err(NativeSurfaceError::Superseded);
        }
        self.refresh_trigger_anchor().await;
        if !self
            .surface
            .generation
            .try_peek()
            .is_ok_and(|current| *current == generation)
            || node.as_ref().is_some_and(|node| !node.is_registered())
        {
            return Err(NativeSurfaceError::Superseded);
        }
        // Source records presentation before the native show call. Native
        // rejection is rolled back as an explicit Rust Result adaptation.
        self.surface.open.set(true);
        if let Some(node) = &node {
            node.did_present();
        }
        let result = self.surface.present_popover(generation, true).await;
        if *self.surface.generation.peek() != generation {
            return Err(NativeSurfaceError::Superseded);
        }
        if let Err(error) = result {
            self.opening.set(false);
            self.surface.open.set(false);
            if let Some(node) = node {
                node.sync_open(false);
            }
            return Err(error);
        }
        // Request the existing canonical scan after native promotion. Dioxus
        // composition supplies header roles; no Popover-specific DOM scan or
        // second focus executor is needed. A newer pointer/close cancels intent.
        self.pending_opening_focus.set(Some(generation));
        let requested = node
            .as_ref()
            .is_some_and(|node| node.request_focus_observation(generation));
        if !requested {
            self.pending_opening_focus.set(None);
        }

        if *self.surface.generation.peek() != generation || !*self.surface.open.peek() {
            return Err(NativeSurfaceError::Superseded);
        }
        self.opening.set(false);
        if let Some(node) = &node {
            node.queue_completion(Rc::new(move || {
                if self
                    .surface
                    .generation
                    .try_peek()
                    .is_ok_and(|value| *value == generation)
                    && self.surface.open.try_peek().is_ok_and(|open| *open)
                    && !*self.reported_open.peek()
                {
                    let mut reported = self.reported_open;
                    reported.set(true);
                    if let Some(callback) = *self.onshowchanged.peek() {
                        callback.call(true);
                    }
                }
            }));
        }
        if requested {
            Ok(())
        } else {
            Err(NativeSurfaceError::Rejected(
                "shared opening focus observation is unavailable".into(),
            ))
        }
    }

    fn observe_opening_focus(
        mut self,
        node: &crate::overlay_runtime::NodeHandle,
        observation: crate::overlay_runtime::SurfaceObservation,
    ) {
        if !node.is_registered()
            || !self
                .overlay
                .try_peek()
                .is_ok_and(|current| current.as_ref() == Some(node))
        {
            return;
        }
        match observation {
            crate::overlay_runtime::SurfaceObservation::RequestedFocus { request, focus } => {
                if !self
                    .pending_opening_focus
                    .try_peek()
                    .is_ok_and(|pending| *pending == Some(request))
                    || !self
                        .surface
                        .generation
                        .try_peek()
                        .is_ok_and(|generation| *generation == request)
                    || !self.surface.open.try_peek().is_ok_and(|open| *open)
                {
                    return;
                }
                self.pending_opening_focus.set(None);
                self.apply_opening_focus(request, &node.opening_focus_candidates(&focus));
            }
            crate::overlay_runtime::SurfaceObservation::Pointer => {
                if self.pending_opening_focus.peek().is_some() {
                    self.pending_opening_focus.set(None);
                    node.cancel_registered_focus();
                }
            }
            crate::overlay_runtime::SurfaceObservation::Focus(_) => {}
        }
    }

    fn apply_opening_focus(
        mut self,
        generation: u64,
        candidates: &[crate::popover_focus::FocusCandidate],
    ) {
        if !self
            .surface
            .generation
            .try_peek()
            .is_ok_and(|value| *value == generation)
            || !self.surface.open.try_peek().is_ok_and(|open| *open)
        {
            return;
        }
        let Some(node) = self
            .overlay
            .peek()
            .clone()
            .filter(crate::overlay_runtime::NodeHandle::is_registered)
        else {
            return;
        };
        let registered = candidates
            .iter()
            .filter(|candidate| node.capture_registered_focus(&candidate.target).is_some())
            .cloned()
            .collect::<Vec<_>>();
        self.focusable_at_open.set(!registered.is_empty());
        self.focus_trigger.set(
            self.trigger_id
                .peek()
                .as_deref()
                .and_then(|id| node.capture_registered_focus(id)),
        );
        if let Some(target) =
            crate::popover_focus::opening_target(&registered, *self.hover_mode.peek())
                .and_then(|id| node.capture_registered_focus(&id))
        {
            node.queue_registered_focus(target, Some(true));
        }
    }

    fn hover_focus_allowed(mut self) -> bool {
        if *self.suppress_focus_open.peek() {
            self.suppress_focus_open.set(false);
            false
        } else {
            true
        }
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
    /// Hide the retained native surface and restore invoker focus.
    pub fn hide(self) {
        self.hide_with_focus(true);
    }

    fn hide_with_focus(self, restore_focus: bool) {
        if let Some(node) = self.overlay.peek().clone() {
            node.request(
                false,
                crate::overlay_runtime::Reason::Trigger,
                restore_focus,
            );
        } else if self.allow_hide() && self.is_open() {
            self.commit_hide_with_focus(restore_focus);
        }
    }

    // Observe cancellation before shared coordinator child dismissal begins.
    fn allow_hide(self) -> bool {
        self.request_change(false)
    }

    fn request_change(mut self, open: bool) -> bool {
        if *self.dispatching_change.peek() {
            return false;
        }
        let event = Event::new(Rc::new(ToggleData::new(LocalToggle)), false);
        let request = PopoverChangeEvent {
            open,
            previous: self.is_open(),
            event: event.clone(),
        };
        self.dispatching_change.set(true);
        if let Some(callback) = *self.onbeforechange.peek() {
            callback.call(request);
        }
        if let Some(callback) = *self.onshowchange.peek() {
            callback.call(event.clone());
        }
        self.dispatching_change.set(false);
        event.default_action_enabled()
    }

    fn commit_hide_with_focus(mut self, restore_focus: bool) {
        self.pending_opening_focus.set(None);
        if !self.is_open() {
            return;
        }
        self.cancel_hover_hide();
        let node = self.overlay.peek().clone();
        if let Some(node) = &node {
            node.cancel_registered_focus();
        }
        let trigger = restore_focus
            .then(|| self.focus_trigger.peek().clone())
            .flatten();
        // Suppress only the focus event requested by explicit restoration.
        // An outside close must allow the next genuine trigger focus to open.
        self.suppress_focus_open
            .set(*self.hover_mode.peek() && restore_focus);
        self.surface.open.set(false);
        let generation = self.surface.generation.peek().wrapping_add(1);
        self.surface.generation.set(generation);
        self.surface.restore_after_mount.set(None);
        self.opening.set(false);
        self.reported_open.set(false);
        let complete = move || {
            if !self
                .surface
                .generation
                .try_peek()
                .is_ok_and(|value| *value == generation)
                || !self.surface.open.try_peek().is_ok_and(|open| !*open)
            {
                return;
            }
            if let Some(node) = self
                .overlay
                .peek()
                .clone()
                .filter(crate::overlay_runtime::NodeHandle::is_registered)
            {
                if let Some(trigger) = trigger.clone() {
                    node.queue_registered_focus(trigger, Some(false));
                }
                node.queue_completion(Rc::new(move || {
                    if self
                        .surface
                        .generation
                        .try_peek()
                        .is_ok_and(|value| *value == generation)
                        && self.surface.open.try_peek().is_ok_and(|open| !*open)
                        && let Some(callback) = *self.onshowchanged.peek()
                    {
                        callback.call(false);
                    }
                }));
            }
        };
        #[cfg(any(
            all(feature = "web", target_arch = "wasm32"),
            all(feature = "desktop", not(target_arch = "wasm32"))
        ))]
        {
            let hide = move || {
                // Hide only after Dioxus renders this generation. Its confirmed
                // completion still precedes registered mounted focus return.
                spawn(async move {
                    popover_after_render().await;
                    if self
                        .surface
                        .present_popover(generation, false)
                        .await
                        .is_ok()
                    {
                        complete();
                    }
                });
            };
            if let Some(node) = node {
                node.with_owner(hide);
            } else {
                hide();
            }
        }
        #[cfg(not(any(
            all(feature = "web", target_arch = "wasm32"),
            all(feature = "desktop", not(target_arch = "wasm32"))
        )))]
        complete();
    }
}

fn popover_overlay_callbacks(
    popover: PopoverHandle,
    live: Rc<std::cell::Cell<bool>>,
) -> crate::overlay_runtime::Callbacks {
    let read = live.clone();
    let before = live.clone();
    crate::overlay_runtime::Callbacks {
        read: Rc::new(move || {
            if !read.get() {
                return None;
            }
            popover.surface.open.try_peek().ok().map(|open| *open)
        }),
        before: Rc::new(move |change| before.get() && !change.open && popover.allow_hide()),
        apply: Rc::new(move |change| {
            if live.get() && !change.open {
                popover.commit_hide_with_focus(change.restore_focus);
            }
        }),
        key: None,
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

/// A Dioxus-owned Popover trigger's focus and CSS anchor metadata.
/// Apply `onmounted`, `onvisible` and `attributes()` to the same stable control.
/// Use `attributes_with_style()` to preserve a caller-authored style alongside
/// the binding's anchor metadata.
#[derive(Clone)]
pub struct PopoverTriggerBinding {
    /// Register the control's mounted handle and current owner.
    pub onmounted: EventHandler<MountedEvent>,
    /// Update the registered anchor's visible geometry through Dioxus.
    pub onvisible: EventHandler<VisibleEvent>,
    ownership: u64,
    id: String,
    popover: PopoverHandle,
}
impl PopoverTriggerBinding {
    /// CSS anchor declaration for advanced attribute composition.
    pub fn style(&self) -> String {
        format!("anchor-name:--ix-popover-trigger-{}", self.ownership)
    }
    /// Stable ID, anchor style and accessible overlay relationships.
    pub fn attributes(&self) -> Vec<Attribute> {
        self.attributes_with_style("")
    }
    /// Preserve caller styling while adding the binding's anchor and relationships.
    pub fn attributes_with_style(&self, style: impl Into<String>) -> Vec<Attribute> {
        let mut style = style.into();
        if !style.is_empty() {
            style.push(';');
        }
        style.push_str(&self.style());
        vec![
            Attribute::new("id", self.id.clone(), None, false),
            Attribute::new("style", style, None, false),
            Attribute::new("data-ix-overlay-trigger", self.popover.id(), None, false),
            Attribute::new("aria-controls", self.popover.id(), None, false),
            Attribute::new("aria-haspopup", "dialog", None, false),
            Attribute::new(
                "aria-expanded",
                self.popover.is_open().to_string(),
                None,
                false,
            ),
        ]
    }
}

/// Register an explicitly anchored custom Popover trigger.
/// Give the control the same stable `id`, attach both event fields and spread
/// `binding.attributes()`. Activation uses the controller's `show`/`hide` methods.
/// A replacement control must have a new component key. Dioxus owns anchor
/// styling, measurements, focus and cleanup; no arbitrary DOM discovery occurs.
pub fn use_popover_trigger_binding(
    popover: PopoverHandle,
    id: impl Into<String>,
) -> PopoverTriggerBinding {
    use_popover_trigger_registration(popover, id.into(), true)
}

/// Register a custom Popover trigger's mounted element for return focus/ownership.
/// Give the control the same stable `id` and attach this handler to `onmounted`.
/// This mount-only hook does not author an anchor; use
/// [`use_popover_trigger_binding`] for positioned custom triggers. A panel with
/// no visible explicitly anchored trigger centers. The calling scope must
/// unmount with the control; use a new component key for a replacement.
/// Custom content controls register separately with [`crate::use_focus_mount`].
pub fn use_popover_trigger_mount(
    popover: PopoverHandle,
    id: impl Into<String>,
) -> EventHandler<MountedEvent> {
    use_popover_trigger_registration(popover, id.into(), false).onmounted
}

fn use_popover_trigger_registration(
    popover: PopoverHandle,
    id: String,
    anchored: bool,
) -> PopoverTriggerBinding {
    let binding = crate::dropdown::use_item_host("native-opt-in");
    let ancestors = crate::overlay_runtime::use_overlay_ancestors();
    let ownership = use_hook(|| DIALOG_ID.fetch_add(1, Ordering::Relaxed) as u64);
    let mut trigger_ownership = popover.trigger_ownership;
    let current = trigger_ownership.peek().clone();
    if current
        .as_ref()
        .is_none_or(|(token, _)| *token == ownership)
        && current.as_ref() != Some(&(ownership, ancestors.clone()))
    {
        trigger_ownership.set(Some((ownership, ancestors.clone())));
    }
    let mounted = use_hook(|| Rc::new(std::cell::RefCell::new(None::<Rc<MountedData>>)));
    let cleanup = mounted.clone();
    use_drop(move || {
        if trigger_ownership.try_peek().is_ok_and(|current| {
            current
                .as_ref()
                .is_some_and(|(token, _)| *token == ownership)
        }) {
            trigger_ownership.set(None);
            let mut anchor = popover.trigger_anchor;
            if anchor.try_peek().is_ok_and(|anchor| {
                anchor
                    .as_ref()
                    .is_some_and(|anchor| anchor.ownership == ownership)
            }) {
                anchor.set(None);
            }
        }
        if let Some(own) = cleanup.borrow_mut().take()
            && popover.surface.invoker.try_peek().is_ok_and(|current| {
                current
                    .as_ref()
                    .is_some_and(|current| current.matches_node(&own))
            })
        {
            let mut popover = popover;
            popover.surface.invoker.set(None);
            popover.trigger_id.set(None);
            popover.sync_surface_metadata();
            popover.focus_trigger.set(None);
            if let Some(node) = popover.overlay.peek().clone() {
                node.cancel_registered_focus();
            }
        }
    });
    let visible_owner = mounted.clone();
    let binding_id = id.clone();
    let onmounted = use_callback(move |event: MountedEvent| {
        let mut popover = popover;
        let target = event.data();
        if popover
            .surface
            .invoker
            .peek()
            .as_ref()
            .is_some_and(|previous| !previous.matches_node(&target))
        {
            popover.focus_trigger.set(None);
            if let Some(node) = popover.overlay.peek().clone() {
                node.cancel_registered_focus();
            }
        }
        binding.mount(&id, target.clone());
        *mounted.borrow_mut() = Some(target.clone());
        popover
            .surface
            .remember_invoker(binding.runtime.capture_registered_focus(&id));
        popover.trigger_id.set(Some(id.clone()));
        popover.sync_surface_metadata();
        popover
            .trigger_anchor
            .set(anchored.then_some(PopoverAnchor {
                ownership,
                visible: false,
            }));
        if anchored {
            spawn(async move {
                popover.refresh_trigger_anchor().await;
            });
        }
        if trigger_ownership.peek().as_ref() != Some(&(ownership, ancestors.clone())) {
            trigger_ownership.set(Some((ownership, ancestors.clone())));
        }
    });
    let onvisible = use_callback(move |event: VisibleEvent| {
        if !anchored {
            return;
        }
        let Some(target) = visible_owner.borrow().clone() else {
            return;
        };
        if let Ok(bounds) = event.get_bounding_client_rect() {
            popover.update_trigger_anchor(
                ownership,
                &target,
                bounds.size.width > 0.0 && bounds.size.height > 0.0,
            );
        }
    });
    PopoverTriggerBinding {
        onmounted,
        onvisible,
        ownership,
        id: binding_id,
        popover,
    }
}

/// Native popover invoker with a Dioxus-authored CSS anchor and accessible relationship.
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
    let trigger_binding = use_popover_trigger_binding(popover, format!("{id}-trigger"));
    let mut hover_mode = popover.hover_mode;
    use_effect(use_reactive((&trigger_mode,), move |(mode,)| {
        hover_mode.set(mode == PopoverTriggerMode::Hover)
    }));
    rsx! {
        button { id:"{id}-trigger",class: "ix-button ix-button-{variant.as_str()}", r#type: "button", disabled,
            "popovertarget": id.clone(), "popovertargetaction": "toggle", "aria-haspopup": "dialog",
            "data-ix-overlay-trigger": id.clone(),
            "aria-controls": id.clone(), "aria-expanded": popover.is_open().to_string(), "aria-label": aria_label,
            onmounted: trigger_binding.onmounted,onvisible:trigger_binding.onvisible,
            onclick: move |event| {
                event.prevent_default();
                if trigger_mode == PopoverTriggerMode::Click {
                    if popover.is_open() { popover.hide(); }
                    else { spawn(async move { let _=popover.show_native().await; }); }
                }
            },
            onkeydown: move |event| if trigger_mode==PopoverTriggerMode::Click && (event.key()==Key::Enter || event.key()==Key::Character(" ".into())) {
                event.prevent_default();
                if popover.is_open() { popover.hide(); }
                else { spawn(async move { let _=popover.show_native().await; }); }
            },
            onmouseenter: move |_| if !disabled && trigger_mode == PopoverTriggerMode::Hover {
                popover.cancel_hover_hide();
                spawn(async move { let _ = popover.show_native().await; });
            },
            onmouseleave: move |_| popover.schedule_hover_hide(false),
            onfocus: move |_| if !disabled && trigger_mode == PopoverTriggerMode::Hover {
                popover.cancel_hover_hide();
                if !popover.hover_focus_allowed() { return; }
                spawn(async move { let _ = popover.show_native().await; });
            },
            onblur: move |_| {let mut suppressed=popover.suppress_focus_open;suppressed.set(false);popover.schedule_hover_hide(true);},
            ..trigger_binding.attributes(),
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

/// Native non-modal floating panel. Clicking its trigger toggles it. Escape and
/// `close_on_click_outside` dismissal use the shared Rust overlay coordinator.
/// CSS Anchor Positioning is required for placement; older engines center the panel.
/// Tab/Shift+Tab remain inside the visible panel. A canceled Rust dismissal keeps
/// the panel present. Public controller requests are cancelable before native work.
#[component]
pub fn Popover(
    popover: PopoverHandle,
    label: String,
    children: Element,
    #[props(default)] placement: PopoverPlacement,
    #[props(default)] has_spike: bool,
    #[props(default)] close_on_click_outside: bool,
    /// Typed controller request; called before the legacy toggle adapter.
    #[props(default)]
    onbeforechange: Option<EventHandler<PopoverChangeEvent>>,
    #[props(default)] onshowchange: Option<EventHandler<ToggleEvent>>,
    #[props(default)] onshowchanged: Option<EventHandler<bool>>,
) -> Element {
    use_context_provider(|| popover);
    let id = popover.id();
    let generation = (popover.surface.generation)();
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let close_behavior = if close_on_click_outside {
        crate::overlay_registry::CloseBehavior::Outside
    } else {
        crate::overlay_registry::CloseBehavior::None
    };
    let registration = use_hook(|| DIALOG_ID.fetch_add(1, Ordering::Relaxed) as u64);
    let live = use_hook(|| Rc::new(std::cell::Cell::new(true)));
    let node = use_hook({
        let runtime = runtime.clone();
        let id = id.clone();
        let live = live.clone();
        move || {
            runtime.register(
                id,
                registration,
                crate::overlay_registry::OverlayKind::Popover,
                close_behavior,
                popover_overlay_callbacks(popover, live),
            )
        }
    });
    let mut overlay_slot = popover.overlay;
    use_hook({
        let node = node.clone();
        move || overlay_slot.set(Some(node))
    });
    let surface_ancestors = crate::overlay_runtime::use_overlay_ancestors();
    let ancestors = popover
        .trigger_ownership
        .read()
        .as_ref()
        .map(|(_, ancestors)| ancestors.clone())
        .unwrap_or(surface_ancestors);
    node.set_ownership(ancestors.clone(), true);
    node.set_surface_metadata(crate::overlay_runtime::SurfaceMetadata {
        trigger_id: popover.trigger_id.read().clone().unwrap_or_default(),
        items_host: id.clone(),
        popover_host: id.clone(),
    });
    crate::overlay_runtime::use_overlay_scope_with_ancestors(Some(node.clone()), ancestors);
    use_context_provider(|| crate::overlay_runtime::PopoverFocusScope {
        owner: Some(node.clone()),
        header: false,
    });
    let observed_node = node.clone();
    node.set_surface_observer(Rc::new(move |observation| {
        popover.observe_opening_focus(&observed_node, observation);
    }));
    node.set_close_behavior(close_behavior);
    let mounted_node = node.clone();
    let keyboard_node = node.clone();
    let sync = node.clone();
    use_effect(move || sync.sync_open((popover.surface.open)()));
    let cleanup = node.clone();
    use_drop(move || {
        live.set(false);
        cleanup.unregister();
        if overlay_slot.try_peek().is_ok_and(|slot| {
            slot.as_ref()
                .is_some_and(|stored| stored.generation() == registration)
        }) {
            overlay_slot.set(None);
            let mut surface = popover.surface;
            surface.node.set(None);
            surface.open.set(false);
            let generation = surface.generation.peek().wrapping_add(1);
            surface.generation.set(generation);
            let mut popover = popover;
            popover.opening.set(false);
            popover.pending_opening_focus.set(None);
            popover.focusable_at_open.set(false);
            popover.focus_trigger.set(None);
            popover.suppress_focus_open.set(false);
            popover.reported_open.set(false);
            popover.onshowchanged.set(None);
            popover.onshowchange.set(None);
            popover.onbeforechange.set(None);
            popover.cancel_hover_hide();
            surface.invoker.set(None);
            popover.trigger_anchor.set(None);
        }
    });
    let mut callback = popover.onshowchanged;
    let mut before_callback = popover.onshowchange;
    let mut typed_callback = popover.onbeforechange;
    use_effect(use_reactive(
        (&onshowchanged, &onshowchange, &onbeforechange),
        move |(handler, before, typed)| {
            callback.set(handler);
            before_callback.set(before);
            typed_callback.set(typed);
        },
    ));
    let anchor = popover
        .trigger_anchor
        .read()
        .as_ref()
        .filter(|anchor| anchor.visible)
        .map(|anchor| anchor.name());
    let anchored_style = anchor
        .as_ref()
        .map(|anchor| {
            // The pinned interpreter preserves omitted inline declarations.
            // Replace the centered panel's explicit `none` when an anchor appears.
            format!(
                "position-anchor:{anchor};position-area:{};position-try-fallbacks:flip-block,flip-inline,flip-block flip-inline,--ix-popover-viewport",
                placement.as_str()
            )
        })
        // A centered panel has no anchor placement to retry. Chromium tests
        // overflow before the centering transform; the viewport try would
        // otherwise center it once and then apply that transform a second time.
        .unwrap_or_else(|| {
            "position-anchor:auto;position-area:none;position-try-fallbacks:none".into()
        });
    rsx! {
        crate::overlay_runtime::OverlayTransport {runtime:runtime.clone()}
            div { id: id.clone(), class: "ix-popover", role: "dialog", "aria-label": label.clone(), tabindex: "-1",
                "data-surface-generation": "{generation}",
                "data-ix-overlay-root":id.clone(),"data-ix-overlay-owner":runtime.owner_id(),
                "data-ix-overlay-generation":registration.to_string(),"data-ix-overlay-open":popover.is_open().to_string(),
                "data-ix-overlay-host":id.clone(),"data-ix-overlay-native-surface":"popover",
                "popover": "manual",
                "data-placement": placement.as_str(), "data-spike": has_spike.to_string(),
                "data-unanchored":anchor.is_none().then_some("true"),
                style: anchored_style,
                onkeydown: move |event: KeyboardEvent| {
                    let decision = keyboard_node.dioxus_popover_tab(&event, *popover.focusable_at_open.peek());
                    if decision.prevent { event.prevent_default(); }
                    if decision.stop { event.stop_propagation(); }
                },
                onmounted: move |event| {
                    mounted_node.set_mounted();
                    popover.surface.mounted(*popover.surface.generation.peek(), event.data());
                },
                ontoggle: move |_| { let instance=*popover.surface.generation.peek(); spawn(async move {
                    let Some(visible) = popover.surface.native_visibility(instance).await else { return; };
                    let mut surface = popover.surface; let previous = *popover.reported_open.peek();
                    // Retained hosts can deliver an older toggle after a newer
                    // Rust intent. Such an observation cannot revive a closed
                    // generation or cancel an opening still awaiting native work.
                    if (visible && !*surface.open.peek()) || (!visible && *popover.opening.peek()) { return; }
                    surface.open.set(visible);
                    if !visible {
                        let mut pending = popover.pending_opening_focus;
                        pending.set(None);
                    }
                    let mut reported = popover.reported_open;
                    if !(visible && *popover.opening.peek()) { reported.set(visible); }
                    if previous != visible && !(visible && *popover.opening.peek()) { if let Some(callback) = onshowchanged { callback.call(visible); } }
                }); },
                onmouseenter: move |_| popover.cancel_hover_hide(),
                onmouseleave: move |_| popover.schedule_hover_hide(false),
                onfocusin: move |_| popover.cancel_hover_hide(),
                onfocusout: move |_| popover.schedule_hover_hide(true),
                div { class: "ix-popover-body", {children.clone()} }
                if has_spike { span { class: "ix-popover-spike", "aria-hidden": "true" } }
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
    let scope =
        try_consume_context::<crate::overlay_runtime::PopoverFocusScope>().unwrap_or_default();
    use_context_provider(|| crate::overlay_runtime::PopoverFocusScope {
        owner: scope.owner,
        header: true,
    });
    rsx! {
        header { class: "ix-popover-header",
            if let Some(icon) = icon { span { style: icon_color.map(|name| format!("color:var({name})")), crate::Icon { name: icon, size: 24 } } }
            h3 { class: "ix-popover-title", {children} }
            {additional_items}
            if !hide_close { PopoverCloseControl { popover,label:close_label,oncloseclick } }
        }
    }
}

#[component]
fn PopoverCloseControl(
    popover: Option<PopoverHandle>,
    label: String,
    oncloseclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let id = use_hook(|| {
        format!(
            "ix-popover-close-{}",
            DIALOG_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let mounted = crate::dropdown::use_focus_mount(id.clone());
    rsx! {button {id,onmounted:mounted,class:"ix-dialog-close",r#type:"button","aria-label":label,disabled:popover.is_none(),
        onclick:move |event| {
            if let Some(callback)=oncloseclick {callback.call(event.clone());}
            if event.default_action_enabled()&&let Some(popover)=popover {popover.hide();}
        },
        crate::Icon {name:"close".to_string(),size:24}
    }}
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

// WebKit collapses auto-height dialog panels with a percentage max-height and
// zero flex basis. Let the panel size from content; the dialog still caps height.
pub(crate) const STYLES: &str = include_str!("styles/dialogs.css");

#[cfg(test)]
mod tests;
