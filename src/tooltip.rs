//! Source-oriented contextual tooltips, with Rust timing and placement.
//!
//! Copyright (c) 2025-2026 Siemens AG. SPDX-License-Identifier: MIT.
//! Native popover activation, DOM events and measurements use the approved DOM
//! bridge; this module owns visibility, deadlines, flipping and arrow geometry.

use dioxus::prelude::*;
use serde::Deserialize;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

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

#[derive(Deserialize)]
struct Observation {
    kind: String,
    #[serde(default)]
    anchor: String,
    metrics: Option<Metrics>,
    error: Option<String>,
}

/// Tooltip bound to external DOM anchors. IDs need no CSS escaping. Optional
/// selectors may match multiple current or subsequently mounted anchors.
/// The component adds its ID to each anchor's `aria-describedby`, preserving
/// other descriptions and removing only its own token during cleanup.
#[component]
pub fn TooltipFor(
    id: String,
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
    render_tooltip(
        TooltipForProps {
            id,
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
    let mut lifecycle = use_signal(|| initial);
    let mut coordinates = use_signal(|| None::<Position>);
    let settings =
        use_memo(use_reactive!(|placement,
                                show_delay,
                                hide_delay,
                                interactive| {
            Config {
                placement,
                show_delay,
                hide_delay,
                interactive,
            }
        }));
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let transport = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let drop_alive = alive.clone();
    let drop_transport = transport.clone();
    use_drop(move || {
        drop_alive.set(false);
        if let Some(task) = drop_transport.borrow_mut().take() {
            let _ = task.send("finish");
        }
    });
    let visible = lifecycle.read().visible.clone();
    let style = coordinates()
        .map(|p| {
            format!(
                "left:{}px;top:{}px;--ix-tooltip-arrow:{}px",
                p.x, p.y, p.arrow
            )
        })
        .unwrap_or_default();
    let side = coordinates().map(|p| p.side).unwrap_or(placement);
    let selectors = serde_json::to_string(&anchor_selectors).unwrap_or_else(|_| "[]".into());
    let mount_id = id.clone();
    rsx! {
        div {
            id, class: "ix-tooltip {class}", role: "tooltip", popover: "manual",
            "data-open": visible.is_some().to_string(),
            "data-anchor": visible.unwrap_or_default(),
            "data-anchor-id": anchor_id,
            "data-anchor-selectors": selectors,
            "data-placement": side.as_str(),
            "data-interactive": interactive.to_string(),
            "data-animation-frame": animation_frame.to_string(),
            "data-positioned": coordinates().is_some().to_string(),
            // HTML inert is a presence attribute. Dioxus alpha serializes a
            // boolean false as inert="false", which still disables controls.
            inert: lifecycle.read().visible.is_none().then_some("true"), style,
            onclick: move |event| event.stop_propagation(),
            onmounted: move |_| {
                let id = mount_id.clone(); let alive = alive.clone(); let transport = transport.clone();
                spawn(async move {
                    let mut stream = dioxus::document::eval(OBSERVE);
                    if stream.send(id).is_err() { return; }
                    *transport.borrow_mut() = Some(stream);
                    while let Ok(observation) = stream.recv::<Observation>().await {
                        if !alive.get() { break; }
                        let config = *settings.peek();
                        match observation.kind.as_str() {
                            "enter" => {
                                let anchor = observation.anchor;
                                let generation = lifecycle.write().enter(anchor.clone());
                                if let Some(generation) = generation {
                                    let alive = alive.clone();
                                    spawn(async move {
                                        if config.show_delay > 0 { futures_timer::Delay::new(Duration::from_millis(config.show_delay)).await; }
                                        if alive.get() && lifecycle.write().show(generation, anchor) { coordinates.set(None); }
                                    });
                                }
                            }
                            "leave" => {
                                let generation = lifecycle.write().leave(); let alive = alive.clone();
                                spawn(async move {
                                    futures_timer::Delay::new(Duration::from_millis(config.hide_delay())).await;
                                    if alive.get() { lifecycle.write().hide(generation); }
                                });
                            }
                            "retain" if config.interactive => { lifecycle.write().leave(); }
                            "escape" | "missing" => { let mut state = lifecycle.write(); let generation = state.leave(); state.hide(generation); }
                            "measure" => {
                                if lifecycle.peek().visible.as_ref() != Some(&observation.anchor) { continue; }
                                if let Some(metrics) = observation.metrics {
                                    if let Some(result) = position(&metrics, config.placement) { if *coordinates.peek() != Some(result) { coordinates.set(Some(result)); } }
                                    else { let mut state = lifecycle.write(); let generation = state.leave(); state.hide(generation); }
                                }
                            }
                            "error" => {
                                let mut state = lifecycle.write(); let generation = state.leave(); state.hide(generation); drop(state);
                                onnativeerror.call(observation.error.unwrap_or_else(|| "Tooltip DOM bridge failed".into()));
                            }
                            _ => {}
                        }
                    }
                });
            },
            div { class: "ix-tooltip-content",
                if title.is_some() || title_icon.is_some() {
                    div { class: "ix-tooltip-title",
                        if let Some(icon) = title_icon { crate::Icon { name: icon, size: 16 } }
                        if let Some(title) = title { strong { "{title}" } }
                    }
                }
                {children}
            }
            span { class: "ix-tooltip-arrow", "aria-hidden": "true" }
        }
    }
}

// This bridge forwards DOM events and measurements. It does not decide tooltip
// delays, visibility, placement or interaction state. Native activation mirrors
// Rust's data-open attribute. The receive channel keeps cleanup alive until drop.
const OBSERVE: &str = r#"
const id=await dioxus.recv(), panel=document.getElementById(id);
if(!panel)return;
const send=(kind,extra={})=>dioxus.send({kind,...extra});
const bindings=new Map(), anchors=new Map();let sequence=0,frame=0,raf=0,disposed=false,resolveRemoval;
const removed=new Promise(resolve=>{resolveRemoval=resolve});
const listen=(node,name,fn,options)=>{node.addEventListener(name,fn,options);return()=>node.removeEventListener(name,fn,options)};
const rect=node=>{const r=node.getBoundingClientRect();return{x:r.left,y:r.top,width:r.width,height:r.height}};
const measure=()=>{
 frame=0;if(disposed||panel.dataset.open!=='true')return;
 const token=panel.dataset.anchor,target=anchors.get(token);
 if(!target?.isConnected){send('missing');return;}
 const clips=[];
 for(let p=target.parentElement;p;p=p.parentElement){const s=getComputedStyle(p);if(/hidden|clip|scroll|auto/.test(s.overflow+s.overflowX+s.overflowY))clips.push(rect(p));}
 const r=rect(panel);send('measure',{anchor:token,metrics:{anchor:rect(target),width:r.width,height:r.height,viewport_width:document.documentElement.clientWidth,viewport_height:document.documentElement.clientHeight,clips}});
};
const schedule=()=>{if(!frame&&!disposed)frame=requestAnimationFrame(measure)};
const ro=new ResizeObserver(schedule);ro.observe(panel);
const restore=(node,entry)=>{
 entry.cleanups.forEach(fn=>fn());ro.unobserve(node);anchors.delete(entry.token);
 if(entry.added){const values=(node.getAttribute('aria-describedby')||'').split(/\s+/).filter(value=>value&&value!==id);if(values.length)node.setAttribute('aria-describedby',values.join(' '));else node.removeAttribute('aria-describedby');}
};
const bind=()=>{
 if(disposed)return;
 const targets=new Set(); const anchorId=panel.dataset.anchorId;
 if(anchorId){const node=document.getElementById(anchorId);if(node)targets.add(node);}
 try{for(const selector of JSON.parse(panel.dataset.anchorSelectors||'[]'))for(const node of panel.getRootNode().querySelectorAll(selector))targets.add(node);}catch(error){send('error',{error:String(error)});}
 for(const [node,entry]of bindings)if(!targets.has(node)||!node.isConnected){restore(node,entry);bindings.delete(node);}
 for(const node of targets){if(bindings.has(node))continue;const token=String(++sequence);anchors.set(token,node);
 const descriptions=(node.getAttribute('aria-describedby')||'').split(/\s+/).filter(Boolean),added=!descriptions.includes(id);if(added)node.setAttribute('aria-describedby',[...descriptions,id].join(' '));
 const cleanups=[listen(node,'mouseenter',()=>send('enter',{anchor:token})),listen(node,'mouseleave',()=>send('leave',{anchor:token})),listen(node,'focusin',()=>send('enter',{anchor:token})),listen(node,'focusout',()=>send('leave',{anchor:token}))];
 bindings.set(node,{token,added,cleanups});ro.observe(node);}
 schedule();
};
const sync=()=>{
 if(disposed)return;
 try{if(panel.dataset.open==='true'){if(!panel.matches(':popover-open')){if(typeof panel.showPopover!=='function')throw new Error('Native popovers are unavailable');panel.showPopover();}schedule();}
 else if(panel.matches(':popover-open'))panel.hidePopover();}catch(error){send('error',{error:String(error)});}
};
const loop=()=>{raf=0;if(disposed)return;if(panel.dataset.animationFrame==='true'&&panel.dataset.open==='true'){schedule();raf=requestAnimationFrame(loop)}};
const local=new MutationObserver(records=>{if(records.some(r=>r.attributeName==='data-anchor-id'||r.attributeName==='data-anchor-selectors'))bind();sync();if(!raf)loop();});
local.observe(panel,{attributes:true,attributeFilter:['data-open','data-anchor','data-anchor-id','data-anchor-selectors','data-animation-frame']});
const body=new MutationObserver(()=>{if(!panel.isConnected){cleanup();return;}bind();});
body.observe(document.body,{childList:true,subtree:true,attributes:true,attributeFilter:['data-ix-tooltip']});
const cleanupListeners=[listen(window,'resize',schedule),listen(document,'scroll',schedule,true),listen(document,'keydown',event=>{if(event.key==='Escape')send('escape')},true),listen(panel,'mouseenter',()=>{if(panel.dataset.interactive==='true')send('retain')}),listen(panel,'focusin',()=>{if(panel.dataset.interactive==='true')send('retain')}),listen(panel,'mouseleave',()=>send('leave')),listen(panel,'focusout',()=>send('leave'))];
if(window.visualViewport){cleanupListeners.push(listen(window.visualViewport,'resize',schedule),listen(window.visualViewport,'scroll',schedule));}
const cleanup=()=>{if(disposed)return;disposed=true;cancelAnimationFrame(frame);cancelAnimationFrame(raf);ro.disconnect();local.disconnect();body.disconnect();cleanupListeners.forEach(fn=>fn());for(const[node,entry]of bindings)restore(node,entry);bindings.clear();if(panel.isConnected&&panel.matches(':popover-open'))panel.hidePopover();resolveRemoval();};
bind();sync();loop();try{await Promise.race([dioxus.recv(),removed]);}finally{cleanup();}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_tooltip_markup_does_not_make_interactive_content_inert() {
        fn opened() -> Element {
            render_tooltip(
                TooltipForProps {
                    id: "open-tooltip".into(),
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
