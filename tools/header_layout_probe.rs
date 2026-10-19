//! Visible Linux/WebView B18.2 identity replay. All DOM stimuli are synthetic.
//! This excluded probe supplies content/observations, never Header behavior.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde::Serialize;
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;

static NEXT_SCOPE: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default, Serialize)]
struct Life {
    mounts: usize,
    drops: usize,
    native_mounts: usize,
    live: Vec<usize>,
}

#[derive(Clone, Default)]
struct Metrics(Rc<RefCell<BTreeMap<&'static str, Life>>>);

impl PartialEq for Metrics {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ContentMode {
    Elements,
    Empty,
    Text,
    Hidden,
}

#[component]
fn ControlPart(group: &'static str, part: &'static str, mut count: Signal<usize>) -> Element {
    let id = format!(
        "hlp-{group}-{}",
        if part == "input" { "input" } else { "count" }
    );
    let onmounted = use_focus_mount(id);
    if part == "input" {
        rsx! { input { id: "hlp-{group}-input", r#type: "text", "aria-label": "{group} uncontrolled value", autocomplete: "off", onmounted } }
    } else {
        rsx! { button { id: "hlp-{group}-count", r#type: "button", onmounted, onclick: move |_| count += 1, "{group} count {count}" } }
    }
}

#[component]
fn StatefulControl(
    group: &'static str,
    metrics: Metrics,
    #[props(default = ContentMode::Elements)] mode: ContentMode,
    #[props(default)] reversed: bool,
    #[props(default = true)] input_present: bool,
) -> Element {
    let count = use_signal(|| 0_usize);
    let scope = use_hook(|| NEXT_SCOPE.fetch_add(1, Ordering::Relaxed));
    let committed = use_hook(|| Rc::new(Cell::new(false)));
    let effect_committed = committed.clone();
    let effect_metrics = metrics.clone();
    use_effect(move || {
        if !effect_committed.replace(true) {
            let mut metrics = effect_metrics.0.borrow_mut();
            let life = metrics.entry(group).or_default();
            life.mounts += 1;
            life.live.push(scope);
        }
    });
    let cleanup_metrics = metrics.clone();
    use_drop(move || {
        if committed.get() {
            let mut metrics = cleanup_metrics.0.borrow_mut();
            let life = metrics.entry(group).or_default();
            life.drops += 1;
            life.live.retain(|token| *token != scope);
        }
    });
    match mode {
        ContentMode::Empty => rsx! {},
        ContentMode::Text => rsx! { "{group} text only" },
        ContentMode::Elements | ContentMode::Hidden => {
            let order = if reversed {
                ["count", "input"]
            } else {
                ["input", "count"]
            };
            rsx! {
                div { id: "hlp-{group}-control", class: "hlp-control", hidden: mode == ContentMode::Hidden,
                    "data-scope-token": scope.to_string(), "data-local-count": count().to_string(),
                    onmounted: move |_| metrics.0.borrow_mut().entry(group).or_default().native_mounts += 1,
                    for part in order.into_iter().filter(|part| *part != "input" || input_present) {
                        ControlPart { key: "{part}", group, part, count }
                    }
                }
            }
        }
    }
}

const STYLES: &str = r"
body{margin:0}.hlp-fixture{padding:1rem;font-family:system-ui,sans-serif}.hlp-toolbar{display:flex;flex-wrap:wrap;gap:.5rem;margin-bottom:1rem}.hlp-toolbar button,.hlp-control button{font:inherit;padding:.375rem .5rem;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);color:var(--ix-color-text);background:var(--ix-color-surface-1)}.hlp-control{display:inline-flex;align-items:center;gap:.375rem;min-width:0}.hlp-control[hidden]{display:none}.hlp-control input{box-sizing:border-box;min-width:0;width:7rem;height:2.25rem;padding:.375rem;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);font:inherit;color:var(--ix-color-text);background:var(--ix-color-surface-1)}.hlp-fixture :focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}#hlp-stage{height:27rem;min-width:0;position:relative;border:1px solid var(--ix-color-border)}#hlp-stage>.ix-application{height:100%}#hlp-metrics{white-space:pre-wrap;overflow-wrap:anywhere;max-height:12rem;overflow:auto;font-size:.75rem}
";

#[component]
fn Fixture() -> Element {
    let metrics = use_hook(Metrics::default);
    let mut snapshot = use_signal(|| "{}".to_string());
    let mut snapshot_revision = use_signal(|| 0_usize);
    let mut force = use_signal(|| ApplicationBreakpoint::Large);
    let mut top_layer = use_signal(|| false);
    let mut mode = use_signal(|| ContentMode::Elements);
    let mut reversed = use_signal(|| false);
    let mut input_present = use_signal(|| true);
    let mut header_present = use_signal(|| true);
    let mut app_present = use_signal(|| true);
    let mut veto = use_signal(|| false);
    let overflow_events = use_hook(|| Rc::new(RefCell::new(Vec::<Value>::new())));
    let snapshot_metrics = metrics.clone();
    let snapshot_events = overflow_events.clone();
    let request_events = overflow_events.clone();
    let changed_events = overflow_events.clone();
    let header = if header_present() {
        Some(rsx! {
            ApplicationHeader { name: "Header identity probe", aria_label_more_menu_icon_button: "Probe more",
                enable_top_layer: top_layer(),
                onbeforeoverflowchange: move |event: HeaderOverflowEvent| {
                    let vetoed = *veto.peek();
                    if vetoed { event.prevent_default(); }
                    request_events.borrow_mut().push(json!({"kind": "request", "open": event.open, "previous": event.previous, "veto": vetoed}));
                },
                onoverflowchange: move |open| changed_events.borrow_mut().push(json!({"kind": "accepted", "open": open})),
                actions: rsx! { StatefulControl { group: "default", metrics: metrics.clone(), reversed: reversed() } },
                secondary: rsx! { StatefulControl { group: "secondary", metrics: metrics.clone(), input_present: input_present() } },
                overflow: rsx! { StatefulControl { group: "overflow", metrics: metrics.clone(), mode: mode() } },
                overflow_present: matches!(mode(), ContentMode::Elements | ContentMode::Hidden),
                avatar: rsx! { StatefulControl { group: "avatar", metrics: metrics.clone() } },
            }
        })
    } else {
        None
    };
    let force_name = match force() {
        ApplicationBreakpoint::Small => "sm",
        ApplicationBreakpoint::Medium => "md",
        ApplicationBreakpoint::Large => "lg",
    };
    rsx! {
        style { {STYLES} }
        section { id: "hlp-fixture", class: "hlp-fixture", "data-force": force_name, "data-top-layer": top_layer().to_string(), "data-veto": veto().to_string(),
            h1 { "Header layout and identity probe" }
            p { "Synthetic native replay. Inputs have no bound value; each group has one component-local counter and post-commit lifetime record." }
            div { class: "hlp-toolbar",
                button { id: "hlp-lg", r#type: "button", onclick: move |_| force.set(ApplicationBreakpoint::Large), "Large" }
                button { id: "hlp-sm", r#type: "button", onclick: move |_| force.set(ApplicationBreakpoint::Small), "Small" }
                button { id: "hlp-md", r#type: "button", onclick: move |_| force.set(ApplicationBreakpoint::Medium), "Medium" }
                button { id: "hlp-layer", r#type: "button", onclick: move |_| top_layer.toggle(), "Toggle top layer" }
                button { id: "hlp-veto", r#type: "button", onclick: move |_| veto.toggle(), "Toggle overflow request veto" }
                button { id: "hlp-reorder", r#type: "button", onclick: move |_| reversed.toggle(), "Reverse keyed default children" }
                button { id: "hlp-child", r#type: "button", onclick: move |_| input_present.toggle(), "Remove / restore secondary input" }
                button { id: "hlp-elements", r#type: "button", onclick: move |_| mode.set(ContentMode::Elements), "Overflow elements" }
                button { id: "hlp-empty", r#type: "button", onclick: move |_| mode.set(ContentMode::Empty), "Overflow empty component" }
                button { id: "hlp-text", r#type: "button", onclick: move |_| mode.set(ContentMode::Text), "Overflow text only" }
                button { id: "hlp-hidden", r#type: "button", onclick: move |_| mode.set(ContentMode::Hidden), "Overflow hidden element" }
                button { id: "hlp-header", r#type: "button", onclick: move |_| header_present.toggle(), "Remove / restore Header" }
                button { id: "hlp-owner", r#type: "button", onclick: move |_| app_present.toggle(), "Remove / restore Application" }
                button { id: "hlp-snapshot", r#type: "button", onclick: move |_| {
                    snapshot_revision += 1;
                    snapshot.set(json!({"revision": *snapshot_revision.peek(), "groups": snapshot_metrics.0.borrow().clone(), "overflow_events": snapshot_events.borrow().clone()}).to_string());
                }, "Capture committed lifetimes" }
            }
            button { id: "hlp-before", r#type: "button", "Before Header" }
            div { id: "hlp-stage",
                if app_present() {
                    Application { force_breakpoint: force(), header,
                        p { "Application body" }
                        button { id: "hlp-body", r#type: "button", "Body focus target" }
                    }
                }
            }
            button { id: "hlp-after", r#type: "button", "After Header" }
            pre { id: "hlp-metrics", "{snapshot}" }
        }
    }
}

// Native markers are agreed with the private presenter's exclusive producer.
const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms)),get=id=>document.getElementById('hlp-'+id),header=()=>get('stage')?.querySelector('header.ix-application-header');
 const groups=['default','secondary','overflow','avatar'],control=group=>get(group+'-control'),input=group=>get(group+'-input'),count=group=>get(group+'-count');
 const root=()=>header()?.querySelector('[data-header-layout-native="true"]'),more=()=>header()?.querySelector('button[data-header-more-button]');
 const visible=node=>!!node&&node.getClientRects().length>0&&getComputedStyle(node).display!=='none'&&getComputedStyle(node).visibility!=='hidden';
 const opened=()=>root()?.dataset.open==='true'&&visible(input('overflow'));
 const popovers=()=>[...document.querySelectorAll(':popover-open')].filter(node=>header()?.contains(node));
 const active=()=>document.activeElement?.id||document.activeElement?.getAttribute('aria-label')||document.activeElement?.tagName;
 const snapshot=()=>({force:get('fixture')?.dataset.force,topLayer:get('fixture')?.dataset.topLayer,focus:active(),header:header()?.className,more:more()?{hidden:more().hidden,display:getComputedStyle(more()).display,ariaHidden:more().getAttribute('aria-hidden')}:null,popovers:popovers().map(node=>({tag:node.tagName,id:node.id,popover:node.getAttribute('popover')})),metrics:get('metrics')?.textContent,groups:Object.fromEntries(groups.map(group=>[group,{id:control(group)?.id,token:control(group)?.dataset.scopeToken,value:input(group)?.value,count:control(group)?.dataset.localCount,visible:visible(input(group))}]))});
 const assert=(ok,name)=>{if(!ok)throw Error(name+'; '+JSON.stringify(snapshot()))};
 const wait=async(test,name)=>{const until=performance.now()+5000;while(!test()){if(performance.now()>until)throw Error(name+'; '+JSON.stringify(snapshot()));await sleep(20)}};
 const click=id=>{const node=get(id);assert(node,'missing control '+id);node.focus();node.click();};
 const key=(node,value,shift=false)=>{node.focus();const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true,composed:true,shiftKey:shift});node.dispatchEvent(event);return event;};
 const captureSnapshot=async()=>{const previous=get('metrics').textContent;click('snapshot');await wait(()=>get('metrics').textContent!==previous,'fresh lifetime snapshot');return JSON.parse(get('metrics').textContent);};
 const captureLife=async()=>{return(await captureSnapshot()).groups;};
 const waitLife=async(test,name)=>{const until=performance.now()+5000;let life;do{life=await captureLife();if(test(life))return life;await sleep(20);}while(performance.now()<until);throw Error(name+'; '+JSON.stringify(snapshot()));};
 const close=async()=>{if(!opened()&&!popovers().length)return;const node=document.activeElement&&header()?.contains(document.activeElement)?document.activeElement:more();const event=key(node,'Escape');assert(!event.defaultPrevented,'source Escape leaves native key default enabled');assert(!escapedBubble.has(event),'source Escape stops external native bubble');await wait(()=>!opened()&&popovers().length===0,'Escape closes');await wait(()=>document.activeElement===more(),'Escape returns More focus');escapeResults.push({defaultPrevented:event.defaultPrevented,reachedDocumentBubble:escapedBubble.has(event),focus:active()});};
 const open=async(value=null)=>{await wait(()=>root()&&more(),'current native listener ready');if(value){const event=key(more(),value);assert(event.defaultPrevented,'source opening key canceled '+value);}else{more().focus();more().click();}await wait(opened,'overflow visible');};
 const force=async(value)=>{await close();click(value);await wait(()=>get('fixture').dataset.force===value&&header()?.classList.contains('ix-app-'+({sm:'small',md:'medium',lg:'large'}[value])),'force '+value);};
 const notes=[],checks=[],keyResults=[],escapeResults=[],escapedBubble=new WeakSet();
 document.addEventListener('keydown',event=>{if(event.key==='Escape')escapedBubble.add(event);});
 await wait(()=>header()&&more()&&groups.every(group=>control(group)),'mounted Header groups');
 await wait(root,'native Header layout readiness');
 let life=await waitLife(life=>groups.every(group=>life[group]?.mounts===1&&life[group]?.drops===0&&life[group]?.native_mounts===1&&life[group]?.live.length===1),'one committed/native instance per group');
 const refs=Object.fromEntries(groups.map(group=>[group,{node:input(group),root:control(group),scope:control(group).dataset.scopeToken}]));
 const seed=group=>{const node=input(group);node.value=group+' retained';node.dispatchEvent(new Event('input',{bubbles:true,cancelable:true}));node.setSelectionRange(1,4);};
 for(const group of ['default','secondary','avatar']){seed(group);count(group).click();}
 await open();seed('overflow');count('overflow').click();await wait(()=>!opened(),'pointer action closes');
 const retained=()=>{for(const group of groups){assert(input(group)===refs[group].node&&control(group)===refs[group].root&&control(group).dataset.scopeToken===refs[group].scope,'stable native/scope '+group);assert(input(group).value===group+' retained'&&control(group).dataset.localCount==='1','retained value/local state '+group);assert(document.querySelectorAll('#hlp-'+group+'-input').length===1,'no duplicate '+group);}};
 retained();checks.push('single committed scope/native instance and uncontrolled local state');
 click('veto');await wait(()=>get('fixture').dataset.veto==='true','request veto set');const vetoKey=key(more(),' ');assert(vetoKey.defaultPrevented,'vetoed source opening key still canceled');let transaction=await captureSnapshot();assert(transaction.overflow_events.at(-1)?.kind==='request'&&transaction.overflow_events.at(-1)?.veto===true&&!opened(),'veto request without accepted opening');click('veto');await wait(()=>get('fixture').dataset.veto==='false','request retry allowed');await open();await close();checks.push('public Rust overflow request veto and accepted-state retry adapter');
 for(const value of ['sm','md','lg']){await force(value);await open();await close();retained();}
 life=await captureLife();assert(groups.every(group=>life[group].mounts===1&&life[group].drops===0),'layout/open/close does not remount scopes');checks.push('forced small/medium/large retained native and component identity');
 const hostRefs=Object.fromEntries(['small','large'].map(mode=>[mode,header().querySelector('[data-header-popup-host="'+mode+'"]')]));assert(hostRefs.small&&hostRefs.large,'two stable native presentation hosts');click('layer');await wait(()=>get('fixture').dataset.topLayer==='true'&&root()?.dataset.topLayer==='true'&&hostRefs.large.getAttribute('popover')==='manual','top layer flag and native attribute');assert(!hostRefs.small.hasAttribute('popover'),'inactive host has no popover attribute');await open();assert(popovers().length===1&&popovers()[0]===hostRefs.large&&popovers()[0].getAttribute('popover')==='manual','actual one manual native top layer');await close();retained();
 for(const value of ['Home','End','ArrowUp','ArrowDown','Enter',' ']){await open(value);await wait(()=>document.activeElement===(['End','ArrowUp'].includes(value)?count('overflow'):input('overflow')),'source first/last opening focus '+value);keyResults.push({key:value,focused:active(),first:document.activeElement===input('overflow'),last:document.activeElement===count('overflow')});await close();}
 await force('sm');await wait(()=>hostRefs.small.getAttribute('popover')==='manual'&&!hostRefs.large.hasAttribute('popover'),'small active top-layer host only');assert(header().querySelector('[data-header-popup-host="small"]')===hostRefs.small&&header().querySelector('[data-header-popup-host="large"]')===hostRefs.large,'mode handoff keeps both native hosts');await open();assert(popovers().length===1&&popovers()[0]===hostRefs.small,'small actual top-layer host');const forwardTab=key(count('overflow'),'Tab');assert(forwardTab.defaultPrevented,'last popup Tab canceled');await wait(()=>document.activeElement===input('secondary'),'last popup Tab wraps first');const reverseTab=key(input('secondary'),'Tab',true);assert(reverseTab.defaultPrevented,'first popup reverse Tab canceled');await wait(()=>document.activeElement===count('overflow'),'first reverse Tab wraps last');await close();retained();await force('lg');
 checks.push('native top layer and synthetic source opening/default/Escape focus subset');
 click('reorder');await wait(()=>control('default').firstElementChild===count('default'),'keyed default reordered');retained();click('child');await wait(()=>!input('secondary'),'keyed secondary input removed');click('child');await wait(()=>input('secondary'),'keyed secondary input restored');assert(input('secondary')!==refs.secondary.node&&control('secondary')===refs.secondary.root,'intentional keyed child replacement preserves group scope');
 checks.push('real keyed reorder and intentional child removal after presentation changes');
 await force('lg');click('empty');await wait(()=>!control('overflow')&&!visible(more()),'empty component has no More element presence');click('text');await wait(()=>header().textContent.includes('overflow text only')&&!visible(more()),'text does not activate More');click('hidden');await wait(()=>control('overflow')?.hidden&&visible(more()),'hidden element counts for presence');click('elements');await wait(()=>control('overflow')&&!control('overflow').hidden&&visible(more()),'elements restored');assert(control('overflow').dataset.scopeToken===refs.overflow.scope&&control('overflow').dataset.localCount==='1','dynamic output preserves containing component local scope');
 life=await captureLife();assert(groups.every(group=>life[group].mounts===1&&life[group].drops===0),'dynamic output not scope remount');checks.push('empty/text/hidden native presence and dynamic output lifetime');
 await open();const oldMore=more();click('header');await wait(()=>!header()&&document.querySelectorAll(':popover-open').length===0,'Header teardown removes surface');life=await waitLife(life=>groups.every(group=>life[group].drops===1&&life[group].live.length===0),'Header scopes dropped once');oldMore.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));await sleep(40);assert(!header()&&document.querySelectorAll(':popover-open').length===0,'stale detached trigger cannot revive Header');click('header');await wait(()=>header()&&groups.every(group=>control(group)),'Header remount');life=await waitLife(life=>groups.every(group=>life[group].mounts===2&&life[group].drops===1),'fresh Header scopes committed');
 await open();click('owner');await wait(()=>!header()&&document.querySelectorAll(':popover-open').length===0,'Application teardown removes native surface');life=await waitLife(life=>groups.every(group=>life[group].drops===2&&life[group].live.length===0),'Application scopes dropped once');click('owner');await wait(()=>header()&&more(),'Application restored');life=await waitLife(life=>groups.every(group=>life[group].mounts===3&&life[group].drops===2),'fresh Application committed');checks.push('Header/Application teardown, stale detached trigger and fresh ownership');
 notes.push('Native input selection was seeded, but universal caret/focus preservation is not asserted. Intentional empty/keyed removal can replace native descendants while containing scopes survive.');
 return{ok:true,checks,keyResults,escapeResults,lifetimes:life,notes,input:'synthetic DOM click/input/keydown and programmatic value/selection/focus with real Rust/native presenter',limits:['not trusted browser default activation or physical/AT evidence','forced Application modes do not prove automatic viewport/em/global authority','empty/text/hidden corners are source-derived, not part of source39 runtime subset','nested overlays, full geometry/Brand and every pending async race remain unverified'],renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error)}}
"#;

#[component]
fn Probe() -> Element {
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(85)).await;
            eprintln!("B18.2 native Header layout probe timed out");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { onmounted: move |_| {
            let desktop=desktop.clone();spawn(async move {
                let query=dom_query::eval(VERIFY);let _=query.send(());
                match query.join::<Value>().await {
                    Ok(result) if result["ok"]==true => { println!("B18.2 native Header layout checks: {result}");desktop.close(); }
                    result => { eprintln!("B18.2 native Header layout probe failed: {result:?}");std::process::exit(1); }
                }
            });
        }, Fixture {} }
    } }
}

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX Header layout identity probe")
                .with_visible(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1440.0, 1100.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}
