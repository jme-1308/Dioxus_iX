//! Source-oriented contextual tooltips, with Rust timing and placement.
//!
//! Copyright (c) 2025-2026 Siemens AG. SPDX-License-Identifier: MIT.
//! Dioxus owns registered anchors, events, mounted measurements and placement.
//! A root-owned native fact stream supplies viewport notifications; the panel
//! binding only covers native popover methods.

use dioxus::prelude::*;
use serde::Deserialize;
use std::{cell::RefCell, rc::Rc, time::Duration};

/// Preferred side. The surface flips when that side does not fit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPlacement {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
}

impl TooltipPlacement {
    fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
        }
    }
    fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct Metrics {
    anchor: Rect,
    width: f64,
    height: f64,
    viewport_width: f64,
    viewport_height: f64,
    clips: Vec<Rect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f64,
    y: f64,
    arrow: f64,
    side: TooltipPlacement,
}

fn position(metrics: &Metrics, preferred: TooltipPlacement) -> Option<Position> {
    let a = metrics.anchor;
    if [
        a.x,
        a.y,
        a.width,
        a.height,
        metrics.width,
        metrics.height,
        metrics.viewport_width,
        metrics.viewport_height,
    ]
    .iter()
    .any(|n| !n.is_finite())
        || a.width <= 0.0
        || a.height <= 0.0
    {
        return None;
    }
    let mut bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: metrics.viewport_width,
        height: metrics.viewport_height,
    };
    for clip in &metrics.clips {
        let right = (bounds.x + bounds.width).min(clip.x + clip.width);
        let bottom = (bounds.y + bounds.height).min(clip.y + clip.height);
        bounds.x = bounds.x.max(clip.x);
        bounds.y = bounds.y.max(clip.y);
        bounds.width = right - bounds.x;
        bounds.height = bottom - bounds.y;
    }
    if bounds.width <= 0.0
        || bounds.height <= 0.0
        || a.x + a.width <= bounds.x
        || a.y + a.height <= bounds.y
        || a.x >= bounds.x + bounds.width
        || a.y >= bounds.y + bounds.height
    {
        return None;
    }
    let w = metrics.width.max(0.0);
    let h = metrics.height.max(0.0);
    let candidate = |side| match side {
        TooltipPlacement::Top => ((a.x + a.width / 2.0 - w / 2.0), a.y - h - 12.0),
        TooltipPlacement::Bottom => ((a.x + a.width / 2.0 - w / 2.0), a.y + a.height + 12.0),
        TooltipPlacement::Left => (a.x - w - 12.0, a.y + a.height / 2.0 - h / 2.0),
        TooltipPlacement::Right => (a.x + a.width + 12.0, a.y + a.height / 2.0 - h / 2.0),
    };
    let cross = match preferred {
        TooltipPlacement::Top | TooltipPlacement::Bottom => {
            [TooltipPlacement::Right, TooltipPlacement::Left]
        }
        _ => [TooltipPlacement::Bottom, TooltipPlacement::Top],
    };
    let sides = [preferred, preferred.opposite(), cross[0], cross[1]];
    let mut side = preferred;
    for choice in sides {
        let (x, y) = candidate(choice);
        let fits = match choice {
            TooltipPlacement::Top | TooltipPlacement::Bottom => {
                y >= 10.0 && y + h <= metrics.viewport_height - 10.0
            }
            _ => x >= 10.0 && x + w <= metrics.viewport_width - 10.0,
        };
        if fits {
            side = choice;
            break;
        }
    }
    let (x, y) = candidate(side);
    let x = x.clamp(10.0, (metrics.viewport_width - w - 10.0).max(10.0));
    let y = y.clamp(10.0, (metrics.viewport_height - h - 10.0).max(10.0));
    let (center, size) = match side {
        TooltipPlacement::Top | TooltipPlacement::Bottom => (a.x + a.width / 2.0 - x - 6.0, w),
        _ => (a.y + a.height / 2.0 - y - 6.0, h),
    };
    Some(Position {
        x,
        y,
        side,
        arrow: center.clamp(6.0, (size - 18.0).max(6.0)),
    })
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Lifecycle {
    generation: u64,
    pending: Option<String>,
    visible: Option<String>,
}

impl Lifecycle {
    fn enter(&mut self, anchor: String) -> Option<u64> {
        if self.pending.as_ref() == Some(&anchor) {
            return None;
        }
        self.generation = self.generation.wrapping_add(1);
        self.pending = None;
        if self.visible.as_ref() == Some(&anchor) {
            return None;
        }
        self.pending = Some(anchor);
        Some(self.generation)
    }
    fn leave(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.pending = None;
        self.generation
    }
    fn show(&mut self, generation: u64, anchor: String) -> bool {
        if self.generation != generation || self.pending.as_ref() != Some(&anchor) {
            return false;
        }
        self.pending = None;
        self.visible = Some(anchor);
        true
    }
    fn hide(&mut self, generation: u64) {
        if self.generation == generation {
            self.visible = None;
            self.pending = None;
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Config {
    placement: TooltipPlacement,
    show_delay: u64,
    hide_delay: u64,
    interactive: bool,
}
impl Config {
    fn hide_delay(self) -> u64 {
        if self.interactive && self.hide_delay == 50 {
            150
        } else {
            self.hide_delay
        }
    }
}

#[derive(Clone)]
struct Anchor {
    id: String,
    mount: Option<Rc<MountedData>>,
    intersecting: bool,
}
static NEXT_PANEL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
struct Panel {
    token: u64,
    mount: Option<Rc<MountedData>>,
    config: Config,
    onerror: EventHandler<String>,
}
struct TooltipModel {
    alive: bool,
    id: String,
    next: u64,
    anchors: std::collections::BTreeMap<u64, Anchor>,
    panel: Option<Panel>,
    lifecycle: Lifecycle,
    coordinates: Option<Position>,
    viewport: (f64, f64),
    measurement: u64,
}
impl TooltipModel {
    fn invalidate(&mut self) {
        self.measurement = self.measurement.wrapping_add(1);
    }
    fn hide(&mut self) {
        let generation = self.lifecycle.leave();
        self.lifecycle.hide(generation);
        self.coordinates = None;
        self.invalidate();
    }
    fn current_anchor(&self) -> Option<u64> {
        self.lifecycle.visible.as_ref()?.parse().ok()
    }
    fn remove_anchor(&mut self, token: u64) {
        self.anchors.remove(&token);
        let key = token.to_string();
        if self.lifecycle.visible.as_ref() == Some(&key)
            || self.lifecycle.pending.as_ref() == Some(&key)
        {
            self.hide();
        }
    }
}

/// Shared tooltip state. Register anchors through [`use_tooltip_anchor`] and
/// pass a clone to [`TooltipFor`]. Dioxus owns all element handles and events.
/// Keep its ID stable; use a keyed remount to replace a controller or its ID.
#[derive(Clone)]
pub struct TooltipController {
    model: Rc<RefCell<TooltipModel>>,
    revision: Signal<u64>,
}
impl PartialEq for TooltipController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.model, &other.model)
    }
}
impl TooltipController {
    fn notify(&self) {
        let mut revision = self.revision;
        if let Ok(mut value) = revision.try_write() {
            *value = value.wrapping_add(1);
        }
    }
    fn dismiss(&self) {
        self.model.borrow_mut().hide();
        self.notify();
    }
    fn enter(&self, token: u64) {
        let request = {
            let mut model = self.model.borrow_mut();
            if !model.alive
                || !model
                    .anchors
                    .get(&token)
                    .is_some_and(|a| a.mount.is_some() && a.intersecting)
            {
                return;
            }
            let Some(panel) = model.panel.as_ref() else {
                return;
            };
            let (panel_token, delay) = (panel.token, panel.config.show_delay);
            model
                .lifecycle
                .enter(token.to_string())
                .map(|generation| (generation, panel_token, delay))
        };
        let Some((generation, panel_token, delay)) = request else {
            return;
        };
        let controller = self.clone();
        spawn(async move {
            if delay > 0 {
                futures_timer::Delay::new(Duration::from_millis(delay)).await;
            }
            let shown = {
                let mut model = controller.model.borrow_mut();
                if !model.alive
                    || model.panel.as_ref().map(|p| p.token) != Some(panel_token)
                    || !model
                        .anchors
                        .get(&token)
                        .is_some_and(|a| a.mount.is_some() && a.intersecting)
                {
                    return;
                }
                let shown = model.lifecycle.show(generation, token.to_string());
                if shown {
                    model.coordinates = None;
                    model.invalidate();
                }
                shown
            };
            if shown {
                controller.notify();
                controller.measure();
            }
        });
    }
    fn leave(&self, token: Option<u64>) {
        let request = {
            let mut model = self.model.borrow_mut();
            if let Some(token) = token {
                let key = token.to_string();
                if model.lifecycle.visible.as_ref() != Some(&key)
                    && model.lifecycle.pending.as_ref() != Some(&key)
                {
                    return;
                }
            }
            let Some(panel) = model.panel.as_ref() else {
                return;
            };
            let (panel_token, delay) = (panel.token, panel.config.hide_delay());
            (model.lifecycle.leave(), panel_token, delay)
        };
        let controller = self.clone();
        spawn(async move {
            if request.2 > 0 {
                futures_timer::Delay::new(Duration::from_millis(request.2)).await;
            }
            {
                let mut model = controller.model.borrow_mut();
                if !model.alive || model.panel.as_ref().map(|p| p.token) != Some(request.1) {
                    return;
                }
                model.lifecycle.hide(request.0);
                if model.lifecycle.visible.is_none() {
                    model.coordinates = None;
                    model.invalidate();
                }
            }
            controller.notify();
        });
    }
    fn retain(&self) {
        let mut model = self.model.borrow_mut();
        if model.panel.as_ref().is_some_and(|p| p.config.interactive) {
            model.lifecycle.leave();
        }
    }
    fn measure(&self) {
        let snapshot = {
            let mut model = self.model.borrow_mut();
            let Some(token) = model.current_anchor() else {
                return;
            };
            let Some(anchor) = model.anchors.get(&token).and_then(|a| a.mount.clone()) else {
                return;
            };
            let Some(panel) = model.panel.as_ref() else {
                return;
            };
            let Some(panel_mount) = panel.mount.clone() else {
                return;
            };
            let panel_token = panel.token;
            let placement = panel.config.placement;
            model.invalidate();
            (
                model.measurement,
                token,
                panel_token,
                anchor,
                panel_mount,
                model.viewport,
                placement,
            )
        };
        let controller = self.clone();
        spawn(async move {
            let anchor = snapshot.3.get_client_rect().await;
            let panel = snapshot.4.get_client_rect().await;
            let mut model = controller.model.borrow_mut();
            if !model.alive
                || model.measurement != snapshot.0
                || model.current_anchor() != Some(snapshot.1)
                || model.panel.as_ref().map(|p| p.token) != Some(snapshot.2)
                || !model
                    .anchors
                    .get(&snapshot.1)
                    .and_then(|a| a.mount.as_ref())
                    .is_some_and(|a| Rc::ptr_eq(a, &snapshot.3))
                || !model
                    .panel
                    .as_ref()
                    .and_then(|p| p.mount.as_ref())
                    .is_some_and(|p| Rc::ptr_eq(p, &snapshot.4))
            {
                return;
            }
            match (anchor, panel) {
                (Ok(anchor), Ok(panel)) => {
                    // onvisible owns native clipping facts; no handwritten ancestor traversal.
                    let metrics = Metrics {
                        anchor: Rect {
                            x: anchor.origin.x,
                            y: anchor.origin.y,
                            width: anchor.width(),
                            height: anchor.height(),
                        },
                        width: panel.width(),
                        height: panel.height(),
                        viewport_width: snapshot.5.0,
                        viewport_height: snapshot.5.1,
                        clips: Vec::new(),
                    };
                    if snapshot.5.0 <= 0.0 || snapshot.5.1 <= 0.0 {
                        return;
                    }
                    if let Some(next) = position(&metrics, snapshot.6) {
                        if model.coordinates == Some(next) {
                            return;
                        }
                        model.coordinates = Some(next);
                    } else {
                        model.hide();
                    }
                    drop(model);
                    controller.notify();
                }
                _ => {
                    let error = model.panel.as_ref().map(|p| p.onerror);
                    model.hide();
                    drop(model);
                    controller.notify();
                    if let Some(error) = error {
                        error.call("Tooltip mounted measurements are unavailable".into());
                    }
                }
            }
        });
    }
}

/// Create a tooltip owner. IDs are stable for the lifetime of this hook.
pub fn use_tooltip(id: impl Into<String>) -> TooltipController {
    use_tooltip_owner(id.into(), true)
}
fn use_tooltip_owner(id: String, native: bool) -> TooltipController {
    let revision = use_signal(|| 0_u64);
    let controller = use_hook(|| TooltipController {
        model: Rc::new(RefCell::new(TooltipModel {
            alive: true,
            id,
            next: 0,
            anchors: Default::default(),
            panel: None,
            lifecycle: Default::default(),
            coordinates: None,
            viewport: (0.0, 0.0),
            measurement: 0,
        })),
        revision,
    });
    let drop_controller = controller.clone();
    use_drop(move || {
        let mut model = drop_controller.model.borrow_mut();
        model.alive = false;
        model.hide();
        model.anchors.clear();
        model.panel = None;
    });
    let _ = (controller.revision)();
    let shown = {
        let model = controller.model.borrow();
        native && model.alive && model.panel.is_some() && model.lifecycle.visible.is_some()
    };
    let viewport = crate::document_viewport::use_document_viewport(shown);
    let escape_baseline = use_hook(|| Rc::new(std::cell::Cell::new(None::<u64>)));
    // Establish a fresh baseline at the opening render, before subscribing the
    // shown effect. A document Escape from an earlier opening cannot dismiss a
    // later tooltip, even if other viewport consumers kept the stream alive.
    if shown {
        if escape_baseline.get().is_none() {
            escape_baseline.set(Some(viewport.escape_revision()));
        }
    } else {
        escape_baseline.set(None);
    }
    let native_controller = controller.clone();
    use_effect(use_reactive((&shown,), move |(shown,)| {
        if !shown {
            return;
        }
        let escape = viewport.escape_revision();
        if escape_baseline
            .get()
            .is_some_and(|previous| previous != escape)
        {
            escape_baseline.set(Some(escape));
            native_controller.dismiss();
            return;
        }
        let _ = viewport.layout_revision();
        if let Some(facts) = viewport.facts() {
            native_controller.model.borrow_mut().viewport = (facts.width, facts.height);
            native_controller.measure();
        }
    }));
    controller
}

/// Events and ARIA composition for one mounted Dioxus anchor. Wire every field
/// to that element; keep its identity stable. Multiple anchors may share an owner.
/// Controller and anchor ID are captured once; changing either requires a keyed remount.
#[derive(Clone)]
pub struct TooltipAnchorBinding {
    pub onmounted: EventHandler<MountedEvent>,
    pub onresize: EventHandler<ResizeEvent>,
    pub onvisible: EventHandler<VisibleEvent>,
    pub onmouseenter: EventHandler<MouseEvent>,
    pub onmouseleave: EventHandler<MouseEvent>,
    pub onfocusin: EventHandler<FocusEvent>,
    pub onfocusout: EventHandler<FocusEvent>,
    pub onkeydown: EventHandler<KeyboardEvent>,
    controller: TooltipController,
    token: u64,
}
impl TooltipAnchorBinding {
    /// Merge the live panel ID with caller-owned tokens. After panel drop this
    /// removes only the tooltip's token; other descriptions retain their order.
    pub fn describedby(&self, existing: &str) -> Option<String> {
        if self.controller.model.borrow().alive {
            let _ = (self.controller.revision)();
        }
        let model = self.controller.model.borrow();
        let mut tokens: Vec<&str> = existing
            .split_whitespace()
            .filter(|token| *token != model.id)
            .collect();
        if model.alive && model.panel.is_some() && model.anchors.contains_key(&self.token) {
            tokens.push(&model.id);
        }
        (!tokens.is_empty()).then(|| tokens.join(" "))
    }
}

pub fn use_tooltip_anchor(
    controller: TooltipController,
    anchor_id: impl Into<String>,
) -> TooltipAnchorBinding {
    let controller = use_hook(|| controller);
    let id = anchor_id.into();
    let token = use_hook(|| {
        let mut model = controller.model.borrow_mut();
        model.next += 1;
        let token = model.next;
        model.anchors.insert(
            token,
            Anchor {
                id,
                mount: None,
                intersecting: true,
            },
        );
        token
    });
    let drop_controller = controller.clone();
    use_drop(move || {
        drop_controller.model.borrow_mut().remove_anchor(token);
        drop_controller.notify();
    });
    let mounted = controller.clone();
    let resized = controller.clone();
    let visible = controller.clone();
    let entered = controller.clone();
    let left = controller.clone();
    let focused = controller.clone();
    let blurred = controller.clone();
    let keyed = controller.clone();
    TooltipAnchorBinding {
        onmounted: use_callback(move |event: MountedEvent| {
            if let Some(anchor) = mounted.model.borrow_mut().anchors.get_mut(&token) {
                anchor.mount = Some(event.data());
            }
            mounted.measure();
        }),
        onresize: use_callback(move |_: ResizeEvent| resized.measure()),
        onvisible: use_callback(move |event: VisibleEvent| {
            let intersects = event.is_intersecting().unwrap_or(true)
                && event
                    .get_intersection_rect()
                    .is_ok_and(|r| r.width() > 0.0 && r.height() > 0.0);
            {
                let mut model = visible.model.borrow_mut();
                if let Some(anchor) = model.anchors.get_mut(&token) {
                    anchor.intersecting = intersects;
                }
                if let Ok(bounds) = event.get_root_bounds() {
                    if bounds.width() > 0.0 && bounds.height() > 0.0 {
                        model.viewport = (bounds.width(), bounds.height());
                    }
                }
                if !intersects && model.current_anchor() == Some(token) {
                    model.hide();
                }
            }
            visible.notify();
            visible.measure();
        }),
        onmouseenter: use_callback(move |_: MouseEvent| entered.enter(token)),
        onmouseleave: use_callback(move |_: MouseEvent| left.leave(Some(token))),
        onfocusin: use_callback(move |_: FocusEvent| focused.enter(token)),
        onfocusout: use_callback(move |_: FocusEvent| blurred.leave(Some(token))),
        onkeydown: use_callback(move |event: KeyboardEvent| {
            if event.key() == Key::Escape {
                keyed.dismiss();
            }
        }),
        controller,
        token,
    }
}

/// Tooltip panel for explicitly registered Dioxus anchors. Pass the controller
/// returned by [`use_tooltip`] and wire [`use_tooltip_anchor`] on each anchor.
/// Legacy DOM selectors/IDs are unsupported and reported through `onnativeerror`;
/// arbitrary DOM discovery has no JavaScript fallback. Keep the controller/ID
/// stable for the panel lifetime; replace them with a keyed remount.
#[component]
pub fn TooltipFor(
    id: String,
    controller: Option<TooltipController>,
    #[props(default)] anchor_id: String,
    #[props(default)] anchor_selectors: Vec<String>,
    #[props(default)] placement: TooltipPlacement,
    #[props(default)] show_delay: u64,
    #[props(default = 50)] hide_delay: u64,
    #[props(default)] interactive: bool,
    #[props(default)] animation_frame: bool,
    title: Option<String>,
    title_icon: Option<String>,
    #[props(default)] class: String,
    #[props(default)] onnativeerror: EventHandler<String>,
    children: Element,
) -> Element {
    let own = use_tooltip_owner(id.clone(), controller.is_none());
    let controller = controller.unwrap_or(own);
    render_tooltip(
        TooltipForProps {
            id,
            controller: Some(controller),
            anchor_id,
            anchor_selectors,
            placement,
            show_delay,
            hide_delay,
            interactive,
            animation_frame,
            title,
            title_icon,
            class,
            onnativeerror,
            children,
        },
        Lifecycle::default(),
    )
}

fn render_tooltip(props: TooltipForProps, initial: Lifecycle) -> Element {
    let TooltipForProps {
        id,
        controller,
        anchor_id,
        anchor_selectors,
        placement,
        show_delay,
        hide_delay,
        interactive,
        animation_frame,
        title,
        title_icon,
        class,
        onnativeerror,
        children,
    } = props;
    let controller = controller.expect("TooltipFor creates an owner");
    let controller = use_hook(|| controller);
    let panel_controller = controller.clone();
    let config = Config {
        placement,
        show_delay,
        hide_delay,
        interactive,
    };
    let token = use_hook(|| {
        let mut model = panel_controller.model.borrow_mut();
        let token = NEXT_PANEL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if model.id == id {
            model.hide();
            model.lifecycle = initial;
            model.panel = Some(Panel {
                token,
                mount: None,
                config,
                onerror: onnativeerror,
            });
        }
        token
    });
    let attachment = controller.clone();
    use_effect(use_reactive((&id, &config), move |(id, config)| {
        let mut model = attachment.model.borrow_mut();
        if model.id != id {
            drop(model);
            onnativeerror.call("TooltipFor ID must match its TooltipController".into());
            return;
        }
        if let Some(panel) = model.panel.as_mut().filter(|panel| panel.token == token) {
            panel.config = config;
            panel.onerror = onnativeerror;
        }
        drop(model);
        attachment.notify();
        attachment.measure();
    }));
    let dropped = controller.clone();
    use_drop(move || {
        let mut model = dropped.model.borrow_mut();
        if model.panel.as_ref().map(|p| p.token) == Some(token) {
            model.hide();
            model.panel = None;
            drop(model);
            dropped.notify();
        }
    });
    let unsupported = !anchor_id.is_empty() || !anchor_selectors.is_empty();
    use_effect(move || {
        if unsupported {
            onnativeerror.call("Tooltip DOM anchor IDs/selectors are unsupported; use use_tooltip_anchor and pass its controller".into());
        }
    });
    let _ = (controller.revision)();
    let (visible, coordinates, anchor, intent) = {
        let model = controller.model.borrow();
        (
            model.lifecycle.visible.is_some(),
            model.coordinates,
            model
                .current_anchor()
                .and_then(|key| model.anchors.get(&key))
                .map(|a| a.id.clone())
                .unwrap_or_default(),
            model.lifecycle.generation,
        )
    };
    let style = coordinates
        .map(|p| {
            format!(
                "left:{}px;top:{}px;--ix-tooltip-arrow:{}px",
                p.x, p.y, p.arrow
            )
        })
        .unwrap_or_default();
    let side = coordinates.map(|p| p.side).unwrap_or(placement);
    let marker = format!("{token}:{visible}:{intent}");
    let mut mount_revision = use_signal(|| 0_u64);
    let presentation = controller.clone();
    use_effect(use_reactive(
        (&id, &marker, &visible, &intent),
        move |(id, marker, visible, intent)| {
            if mount_revision() == 0 {
                return;
            }
            #[cfg(any(
                all(feature = "web", target_arch = "wasm32"),
                all(feature = "desktop", not(target_arch = "wasm32"))
            ))]
            {
                let controller = presentation.clone();
                spawn(async move {
                    // The shared binding covers MountedData's missing native
                    // popover methods and guards this exact panel and intent.
                    if let Err(error) = crate::native_popover::present(
                        &id,
                        visible,
                        Some(("data-ix-tooltip-native", &marker)),
                        None,
                    )
                    .await
                    {
                        let model = controller.model.borrow();
                        if model.panel.as_ref().map(|p| p.token) == Some(token)
                            && model.lifecycle.generation == intent
                        {
                            drop(model);
                            controller.dismiss();
                            onnativeerror.call(error.to_string());
                        }
                    } else {
                        controller.measure();
                    }
                });
            }
            #[cfg(not(any(
                all(feature = "web", target_arch = "wasm32"),
                all(feature = "desktop", not(target_arch = "wasm32"))
            )))]
            let _ = (&id, &marker, visible, intent, &presentation);
        },
    ));
    let sampling = controller.clone();
    let sample_epoch = use_hook(|| Rc::new(std::cell::Cell::new(0_u64)));
    let drop_epoch = sample_epoch.clone();
    use_drop(move || drop_epoch.set(drop_epoch.get().wrapping_add(1)));
    use_effect(use_reactive(
        (&visible, &animation_frame),
        move |(visible, animation_frame)| {
            sample_epoch.set(sample_epoch.get().wrapping_add(1));
            let epoch = sample_epoch.get();
            let sample_epoch = sample_epoch.clone();
            if !visible || !animation_frame {
                return;
            }
            let controller = sampling.clone();
            spawn(async move {
                loop {
                    // Opt-in Rust sampling approximates animated movement; it is not a
                    // browser animation-frame chronology contract.
                    futures_timer::Delay::new(Duration::from_millis(16)).await;
                    {
                        let model = controller.model.borrow();
                        if !model.alive
                            || model.panel.as_ref().map(|p| p.token) != Some(token)
                            || sample_epoch.get() != epoch
                            || model.lifecycle.visible.is_none()
                        {
                            break;
                        }
                    }
                    controller.measure();
                }
            });
        },
    ));
    let mounted = controller.clone();
    let resized = controller.clone();
    let seen = controller.clone();
    let retained = controller.clone();
    let focused = controller.clone();
    let left = controller.clone();
    let blurred = controller.clone();
    let keyed = controller.clone();
    rsx! {
        div {
            id,class:"ix-tooltip {class}",role:"tooltip",popover:"manual",
            "data-open":visible.to_string(),"data-anchor":anchor,"data-placement":side.as_str(),
            "data-interactive":interactive.to_string(),"data-animation-frame":animation_frame.to_string(),
            "data-positioned":coordinates.is_some().to_string(),"data-ix-tooltip-native":marker,
            inert:(!visible).then_some("true"),style,
            onclick:move |event|event.stop_propagation(),
            onmounted:move |event:MountedEvent|{if let Some(panel)=mounted.model.borrow_mut().panel.as_mut().filter(|p|p.token==token){panel.mount=Some(event.data());}mount_revision.with_mut(|value|*value+=1);mounted.measure();},
            onresize:move |_|resized.measure(),onvisible:move |_|seen.measure(),
            onmouseenter:move |event|{event.stop_propagation();retained.retain();},
            onfocusin:move |event|{event.stop_propagation();focused.retain();},
            onmouseleave:move |event|{event.stop_propagation();left.leave(None);},
            onfocusout:move |event|{event.stop_propagation();blurred.leave(None);},
            onkeydown:move |event:KeyboardEvent|{if event.key()==Key::Escape {keyed.dismiss();}},
            div {class:"ix-tooltip-content",
                if title.is_some()||title_icon.is_some() {
                    div {class:"ix-tooltip-title",
                        if let Some(icon)=title_icon {crate::Icon{name:icon,size:16}}
                        if let Some(title)=title {strong{"{title}"}}
                    }
                }
                {children}
            }
            span {class:"ix-tooltip-arrow","aria-hidden":"true"}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_tooltip_markup_does_not_make_interactive_content_inert() {
        fn opened() -> Element {
            render_tooltip(
                TooltipForProps {
                    id: "open-tooltip".into(),
                    controller: Some(use_tooltip("open-tooltip")),
                    anchor_id: "anchor".into(),
                    anchor_selectors: Vec::new(),
                    placement: TooltipPlacement::Top,
                    show_delay: 0,
                    hide_delay: 50,
                    interactive: true,
                    animation_frame: false,
                    title: None,
                    title_icon: None,
                    class: String::new(),
                    onnativeerror: EventHandler::default(),
                    children: rsx! { button { "Tooltip action" } },
                },
                Lifecycle {
                    visible: Some("anchor-token".into()),
                    ..Default::default()
                },
            )
        }
        let mut dom = VirtualDom::new(opened);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("role=\"tooltip\""), "{html}");
        assert!(html.contains("popover=\"manual\""), "{html}");
        assert!(html.contains("data-open=\"true\""), "{html}");
        assert!(html.contains("Tooltip action"), "{html}");
        assert!(!html.contains(" inert"), "Open tooltip is inert: {html}");

        fn closed() -> Element {
            rsx! { TooltipFor { id: "closed-tooltip", anchor_id: "anchor", "Description" } }
        }
        let mut closed_dom = VirtualDom::new(closed);
        closed_dom.rebuild_in_place();
        let closed = dioxus_ssr::render(&closed_dom);
        assert!(closed.contains("data-open=\"false\""), "{closed}");
        assert!(closed.contains(" inert"), "{closed}");
    }

    type OwnerCapture = Rc<
        RefCell<
            Option<(
                TooltipController,
                TooltipAnchorBinding,
                TooltipAnchorBinding,
                Signal<bool>,
            )>,
        >,
    >;
    type ViewportCapture = Rc<RefCell<Option<crate::document_viewport::DocumentViewport>>>;
    fn registered_owner_fixture() -> Element {
        let viewport = crate::document_viewport::use_document_viewport(false);
        if let Some(capture) = try_consume_context::<ViewportCapture>() {
            *capture.borrow_mut() = Some(viewport);
        }
        let capture = use_context::<OwnerCapture>();
        let show = use_signal(|| true);
        let controller = use_tooltip("registered-tooltip");
        let first = use_tooltip_anchor(controller.clone(), "first");
        let second = use_tooltip_anchor(controller.clone(), "second");
        *capture.borrow_mut() = Some((controller.clone(), first.clone(), second.clone(), show));
        rsx! {
            button {"aria-describedby":first.describedby("external description"),"First"}
            button {"aria-describedby":second.describedby("other"),"Second"}
            if show() { TooltipFor {id:"registered-tooltip",controller:controller.clone(),interactive:true,"Description"} }
        }
    }
    fn settle(dom: &mut VirtualDom) {
        for _ in 0..6 {
            dom.render_immediate_to_vec();
            dom.process_events();
        }
    }
    #[derive(Clone)]
    struct RectMount(Rect);
    impl dioxus::html::RenderedElementBacking for RectMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn get_client_rect(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                    >,
            >,
        > {
            let rect = self.0;
            Box::pin(async move {
                Ok(dioxus::html::geometry::PixelsRect::new(
                    dioxus::html::geometry::euclid::Point2D::new(rect.x, rect.y),
                    dioxus::html::geometry::euclid::Size2D::new(rect.width, rect.height),
                ))
            })
        }
    }
    fn mounted(rect: Rect) -> Rc<MountedData> {
        Rc::new(MountedData::new(RectMount(rect)))
    }

    #[test]
    fn registered_shared_anchors_preserve_descriptions_and_panel_drop_removes_only_own_token() {
        let capture: OwnerCapture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(registered_owner_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let (controller, first, second, mut show) = capture.borrow().clone().unwrap();
        assert_eq!(
            first.describedby("external description"),
            Some("external description registered-tooltip".into())
        );
        assert_eq!(
            second.describedby("other"),
            Some("other registered-tooltip".into())
        );
        dom.in_scope(ScopeId::ROOT, || show.set(false));
        settle(&mut dom);
        assert!(controller.model.borrow().panel.is_none());
        assert_eq!(
            first.describedby("external registered-tooltip description"),
            Some("external description".into())
        );
        assert_eq!(second.describedby("other"), Some("other".into()));
        dom.in_scope(ScopeId::ROOT, || show.set(true));
        settle(&mut dom);
        assert_eq!(
            second.describedby("other"),
            Some("other registered-tooltip".into())
        );
        drop(dom);
        let model = controller.model.borrow();
        assert!(!model.alive);
        assert!(model.anchors.is_empty());
        assert!(model.panel.is_none());
    }

    #[test]
    fn mounted_measurements_switch_shared_anchors_and_removed_anchor_cannot_reopen() {
        let capture: OwnerCapture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(registered_owner_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let (controller, first, second, _) = capture.borrow().clone().unwrap();
        {
            let mut model = controller.model.borrow_mut();
            model.viewport = (400.0, 300.0);
            model.anchors.get_mut(&first.token).unwrap().mount = Some(mounted(scene().anchor));
            model.anchors.get_mut(&second.token).unwrap().mount = Some(mounted(Rect {
                x: 200.0,
                ..scene().anchor
            }));
            model.panel.as_mut().unwrap().mount = Some(mounted(Rect {
                x: 0.0,
                y: 0.0,
                width: 120.0,
                height: 50.0,
            }));
        }
        dom.in_scope(ScopeId::ROOT, || controller.enter(first.token));
        settle(&mut dom);
        assert_eq!(
            controller.model.borrow().current_anchor(),
            Some(first.token)
        );
        assert_eq!(
            controller.model.borrow().coordinates,
            position(&scene(), TooltipPlacement::Top)
        );
        controller.model.borrow_mut().coordinates = None;
        dom.in_scope(ScopeId::ROOT, || controller.measure());
        controller
            .model
            .borrow_mut()
            .anchors
            .get_mut(&first.token)
            .unwrap()
            .mount = Some(mounted(Rect {
            x: 250.0,
            ..scene().anchor
        }));
        settle(&mut dom);
        assert!(
            controller.model.borrow().coordinates.is_none(),
            "old mounted result must not position a replacement anchor"
        );
        dom.in_scope(ScopeId::ROOT, || controller.measure());
        settle(&mut dom);
        assert_eq!(controller.model.borrow().coordinates.unwrap().x, 230.0);
        dom.in_scope(ScopeId::ROOT, || controller.enter(second.token));
        settle(&mut dom);
        assert_eq!(
            controller.model.borrow().current_anchor(),
            Some(second.token)
        );
        assert_eq!(controller.model.borrow().coordinates.unwrap().x, 180.0);
        controller.model.borrow_mut().remove_anchor(second.token);
        assert!(controller.model.borrow().lifecycle.visible.is_none());
        dom.in_scope(ScopeId::ROOT, || controller.enter(second.token));
        settle(&mut dom);
        assert!(controller.model.borrow().lifecycle.visible.is_none());
        dom.in_scope(ScopeId::ROOT, || controller.enter(first.token));
        settle(&mut dom);
        controller.dismiss();
        settle(&mut dom);
        assert!(controller.model.borrow().lifecycle.visible.is_none());
    }

    #[test]
    fn shared_viewport_remeasures_shown_tooltip_and_escape_does_not_replay_on_reopening() {
        let capture: OwnerCapture = Rc::new(RefCell::new(None));
        let viewport_capture: ViewportCapture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(registered_owner_fixture)
            .with_root_context(capture.clone())
            .with_root_context(viewport_capture.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let (controller, first, _, _) = capture.borrow().clone().unwrap();
        let viewport = viewport_capture.borrow().clone().unwrap();
        {
            let mut model = controller.model.borrow_mut();
            model.anchors.get_mut(&first.token).unwrap().mount = Some(mounted(scene().anchor));
            model.panel.as_mut().unwrap().mount = Some(mounted(Rect {
                width: 120.0,
                height: 50.0,
                ..Rect::default()
            }));
        }
        dom.in_scope(ScopeId::ROOT, || {
            viewport.test_layout(400.0, 300.0);
            viewport.test_escape();
        });
        settle(&mut dom);
        assert_eq!(
            controller.model.borrow().viewport,
            (0.0, 0.0),
            "a hidden owner does not consume layout facts"
        );
        dom.in_scope(ScopeId::ROOT, || controller.enter(first.token));
        settle(&mut dom);
        assert_eq!(
            controller.model.borrow().current_anchor(),
            Some(first.token)
        );
        assert_eq!(
            controller.model.borrow().coordinates,
            position(&scene(), TooltipPlacement::Top),
            "the Escape before opening is only a baseline"
        );

        controller
            .model
            .borrow_mut()
            .anchors
            .get_mut(&first.token)
            .unwrap()
            .mount = Some(mounted(Rect {
            x: 200.0,
            ..scene().anchor
        }));
        dom.in_scope(ScopeId::ROOT, || viewport.test_layout(400.0, 300.0));
        settle(&mut dom);
        assert_eq!(
            controller.model.borrow().coordinates.unwrap().x,
            180.0,
            "scroll invalidation remeasures even with unchanged viewport dimensions"
        );
        dom.in_scope(ScopeId::ROOT, || viewport.test_escape());
        settle(&mut dom);
        assert!(controller.model.borrow().lifecycle.visible.is_none());

        dom.in_scope(ScopeId::ROOT, || {
            viewport.test_layout(260.0, 200.0);
            viewport.test_escape();
        });
        settle(&mut dom);
        assert_eq!(controller.model.borrow().viewport, (400.0, 300.0));
        dom.in_scope(ScopeId::ROOT, || controller.enter(first.token));
        settle(&mut dom);
        assert_eq!(
            controller.model.borrow().current_anchor(),
            Some(first.token),
            "a later opening never replays earlier Escape revisions"
        );
        assert_eq!(controller.model.borrow().viewport, (260.0, 200.0));
        assert_eq!(controller.model.borrow().coordinates.unwrap().x, 130.0);
        dom.in_scope(ScopeId::ROOT, || viewport.test_escape());
        settle(&mut dom);
        assert!(
            controller.model.borrow().lifecycle.visible.is_none(),
            "a new Escape still dismisses the reopened tooltip"
        );
    }

    #[test]
    fn interactive_retention_invalidates_pending_close_and_panel_replacement_cancels_old_show() {
        let capture: OwnerCapture = Rc::new(RefCell::new(None));
        let mut dom = VirtualDom::new(registered_owner_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        settle(&mut dom);
        let (controller, first, _, mut show) = capture.borrow().clone().unwrap();
        let generation = {
            let mut model = controller.model.borrow_mut();
            model.lifecycle.enter(first.token.to_string()).unwrap()
        };
        {
            let mut model = controller.model.borrow_mut();
            assert!(model.lifecycle.show(generation, first.token.to_string()));
        }
        let close = controller.model.borrow_mut().lifecycle.leave();
        controller.retain();
        controller.model.borrow_mut().lifecycle.hide(close);
        assert_eq!(
            controller.model.borrow().current_anchor(),
            Some(first.token)
        );
        let old_panel = controller.model.borrow().panel.as_ref().unwrap().token;
        dom.in_scope(ScopeId::ROOT, || show.set(false));
        settle(&mut dom);
        dom.in_scope(ScopeId::ROOT, || show.set(true));
        settle(&mut dom);
        assert_ne!(
            controller.model.borrow().panel.as_ref().unwrap().token,
            old_panel
        );
        assert!(
            !controller
                .model
                .borrow_mut()
                .lifecycle
                .show(generation, first.token.to_string())
        );
        assert!(controller.model.borrow().lifecycle.visible.is_none());
    }

    fn scene() -> Metrics {
        Metrics {
            anchor: Rect {
                x: 100.0,
                y: 100.0,
                width: 80.0,
                height: 30.0,
            },
            width: 120.0,
            height: 50.0,
            viewport_width: 400.0,
            viewport_height: 300.0,
            clips: vec![],
        }
    }
    #[test]
    fn placement_flips_and_shifts_while_arrow_tracks_the_anchor() {
        let mut m = scene();
        let p = position(&m, TooltipPlacement::Top).unwrap();
        assert_eq!(
            (p.x, p.y, p.side, p.arrow),
            (80.0, 38.0, TooltipPlacement::Top, 54.0)
        );
        m.anchor.y = 5.0;
        assert_eq!(
            position(&m, TooltipPlacement::Top).unwrap().side,
            TooltipPlacement::Bottom
        );
        m.anchor.x = 370.0;
        let p = position(&m, TooltipPlacement::Bottom).unwrap();
        assert_eq!(p.x, 270.0);
        assert_eq!(p.arrow, 102.0);
    }
    #[test]
    fn clipped_or_removed_references_cannot_leave_visible_tooltips() {
        let mut m = scene();
        m.clips.push(Rect {
            x: 0.0,
            y: 0.0,
            width: 90.0,
            height: 300.0,
        });
        assert!(position(&m, TooltipPlacement::Top).is_none());
        m.clips.clear();
        m.anchor.width = 0.0;
        assert!(position(&m, TooltipPlacement::Top).is_none());
        m.anchor.width = f64::NAN;
        assert!(position(&m, TooltipPlacement::Top).is_none());
    }
    #[test]
    fn leaving_cancels_a_delayed_open_and_reentry_cancels_a_pending_close() {
        let mut state = Lifecycle::default();
        let open = state.enter("first".into()).unwrap();
        assert!(state.enter("first".into()).is_none());
        assert_eq!(state.generation, open);
        state.leave();
        assert!(!state.show(open, "first".into()));
        let open = state.enter("first".into()).unwrap();
        assert!(state.show(open, "first".into()));
        let close = state.leave();
        assert!(state.enter("first".into()).is_none());
        state.hide(close);
        assert_eq!(state.visible.as_deref(), Some("first"));
        let second = state.enter("second".into()).unwrap();
        assert!(state.show(second, "second".into()));
        assert_eq!(state.visible.as_deref(), Some("second"));
    }
    #[test]
    fn interactive_default_extends_only_the_source_default_hide_delay() {
        let config = Config {
            placement: TooltipPlacement::Top,
            show_delay: 500,
            hide_delay: 50,
            interactive: true,
        };
        assert_eq!(config.hide_delay(), 150);
        assert_eq!(
            Config {
                hide_delay: 75,
                ..config
            }
            .hide_delay(),
            75
        );
        assert_eq!(
            Config {
                interactive: false,
                ..config
            }
            .hide_delay(),
            50
        );
    }
}
