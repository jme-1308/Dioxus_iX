//! Excluded G04.6 long-menu focus/nearest-scroll fixture. Application controls
//! and lifecycle stay in Rust; the native driver records actual DOM facts.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX Dropdown nearest-scroll probe")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1000.0, 850.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Probe);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _fixture = Probe;
    eprintln!("Run the desktop or web Dropdown scroll probe");
}

fn record(mut rows: Signal<Vec<Value>>, kind: &'static str, details: Value) {
    if let Ok(mut values) = rows.try_write() {
        let seq = values
            .last()
            .and_then(|row| row["seq"].as_u64())
            .unwrap_or(0)
            + 1;
        values.push(json!({"seq":seq,"kind":kind,"details":details}));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Active,
    Roving,
    Nested,
    Disabled,
    NoFocus,
    Category,
}
impl Mode {
    const ALL: [Self; 6] = [
        Self::Active,
        Self::Roving,
        Self::Nested,
        Self::Disabled,
        Self::NoFocus,
        Self::Category,
    ];
    fn key(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Roving => "roving",
            Self::Nested => "nested",
            Self::Disabled => "disabled",
            Self::NoFocus => "no-focus",
            Self::Category => "category",
        }
    }
    fn parse(value: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|mode| mode.key() == value)
            .unwrap_or(Self::Active)
    }
}

#[component]
fn Probe() -> Element {
    let mut epoch = use_signal(|| 0_usize);
    let mut mode = use_signal(|| Mode::Active);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(60)).await;
            eprintln!("Dropdown scroll native timeout");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        style { {STYLES} }
        main { id: "scroll-probe", style: "padding:20px;min-height:800px",
            onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                { let window=window.clone(); spawn(async move {
                    let query=dom_query::eval(VERIFY); let _=query.send(());
                    match query.join::<Value>().await {
                        Ok(value) if value["ok"]==true => { println!("Dropdown scroll native observations: {value}"); window.close(); }
                        Ok(value) => { eprintln!("Dropdown scroll native failure JSON: {value}"); std::process::exit(1); }
                        Err(error) => { eprintln!("Dropdown scroll native transport failure: {error:?}"); std::process::exit(1); }
                    }
                }); }
            },
            h1 { "Dropdown focus and nearest scrolling" }
            p { "Actual Rust items and native scrolling. The reset diagnostic is a separate public scrollTop intervention." }
            div { style: "display:flex;gap:8px;flex-wrap:wrap",
                for next in Mode::ALL { button { id: format!("scroll-mode-{}",next.key()), onclick: move |_| {mode.set(next);epoch+=1;}, {format!("Fresh {}",next.key())} } }
            }
            input { id: "scroll-mode", r#type: "hidden", oninput: move |event| {mode.set(Mode::parse(&event.value()));epoch+=1;} }
            button { id: "scroll-outside", "Outside native focus target" }
            for current in [epoch()] { Fixture { key: "{current}", epoch: current, mode: mode() } }
            NativeJournal {}
            pre { id: "scroll-native-journal", "aria-label": "Passive native focus and scrolling observations", style: "max-height:200px;overflow:auto;white-space:pre-wrap;font-size:12px" }
        }
    } }
}

#[component]
fn Fixture(epoch: usize, mode: Mode) -> Element {
    let dropdown = use_dropdown();
    let mut mounted = use_signal(|| true);
    let mut replacement = use_signal(|| 0_usize);
    let rows = use_signal(Vec::<Value>::new);
    rsx! { section { id: "scroll-fixture", "data-epoch": epoch.to_string(), "data-mode": mode.key(),
        div { style: "display:flex;gap:8px;margin:12px 0",
            button { id: "scroll-scope-toggle", onclick: move |_| mounted.toggle(), "Toggle owner scope" }
            button { id: "scroll-item-replace", onclick: move |_| replacement+=1, "Replace last Rust item" }
            input { id: "scroll-mount", r#type: "hidden", oninput: move |event| mounted.set(event.value()=="true") }
            input { id: "scroll-replace", r#type: "hidden", oninput: move |_| replacement+=1 }
        }
        if mounted() {
            Owner { dropdown, epoch, mode, replacement: replacement(), rows }
        }
        pre { id: "scroll-rust-log", style: "max-height:100px;overflow:auto;font-size:12px", "{rows():?}" }
        input { id: "scroll-rust-json", r#type: "hidden", value: serde_json::to_string(&rows()).unwrap_or_default() }
    } }
}

#[component]
fn Owner(
    dropdown: DropdownHandle,
    epoch: usize,
    mode: Mode,
    replacement: usize,
    rows: Signal<Vec<Value>>,
) -> Element {
    use_effect(move || record(rows, "owner-mounted", json!({"epoch":epoch})));
    use_drop(move || record(rows, "owner-dropped", json!({"epoch":epoch})));
    let content = rsx! {
        Dropdown { controller: dropdown, label: "Long menu", class: "scroll-probe-dropdown",
            navigation_mode: if mode==Mode::Roving {DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},
            enable_top_layer: true, suppress_automatic_placement: true, suppress_trigger_visibility_check: true,
            disable_focus_handling: mode==Mode::NoFocus,
            onbeforechange: move |event: DropdownChangeEvent| record(rows,"request",json!({"open":event.open,"previous":event.previous})),
            onchange: move |open| record(rows,"completion",json!({"open":open})),
            div { id: "scroll-outer", class: if mode==Mode::Nested {"scroll-outer nested"}else{"scroll-outer"},
                if mode==Mode::Nested { div { class: "scroll-outer-spacer", "Outer leading spacer" } }
                div { id: "scroll-inner", class: "scroll-inner",
                    if mode==Mode::Category { MenuCategory { label: "Scroll category",
                        for index in 0..18 { MenuItem { label: "Category item {index:02}" } }
                        div { class: "scroll-trailing", "Category trailing non-focusable spacer" }
                    } }
                    for index in 0..18 {
                        { let item_key = if index==17 {format!("last-{replacement}")}else{index.to_string()}; rsx! { ScrollItem { key: "{item_key}", index, disabled: mode==Mode::Disabled&&index==17, epoch, replacement, rows } } }
                    }
                    div { class: "scroll-trailing", "Trailing non-focusable spacer" }
                }
                if mode==Mode::Nested { div { class: "scroll-outer-spacer", "Outer trailing spacer" } }
            }
        }
    };
    rsx! { div { id: "scroll-owner", "data-owner-epoch": epoch.to_string(),
        DropdownTrigger { dropdown, "Open long menu" }
        if mode==Mode::Category { Application { force_breakpoint: ApplicationBreakpoint::Large, class: "scroll-category-application",
            menu: rsx! { Menu { label: "Category scroll context", start_expanded: false, pinned: false, {content} } },
            div {}
        } }
        else { {content} }
    } }
}

#[component]
fn ScrollItem(
    index: usize,
    disabled: bool,
    epoch: usize,
    replacement: usize,
    rows: Signal<Vec<Value>>,
) -> Element {
    use_drop(move || {
        record(
            rows,
            "item-dropped",
            json!({"epoch":epoch,"index":index,"replacement":replacement}),
        )
    });
    rsx! { div { class: "scroll-item", "data-scroll-index": index.to_string(), "data-item-replacement": replacement.to_string(),
        DropdownItem { label: "Item {index:02}", disabled }
    } }
}

#[component]
fn NativeJournal() -> Element {
    let transport = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let dropped = transport.clone();
    use_drop(move || {
        if let Some(task) = dropped.borrow_mut().take() {
            let _ = task.send("finish");
        }
    });
    use_effect(move || {
        *transport.borrow_mut() = Some(dioxus::document::eval(OBSERVE));
    });
    VNode::empty()
}

const STYLES: &str = r#"
#scroll-owner{margin-top:12px}
#scroll-owner .scroll-probe-dropdown>.ix-dropdown-root-menu{max-height:600px;overflow:visible;padding:0;border:0;min-width:280px}
.scroll-inner{box-sizing:content-box;width:280px;height:160px;overflow:auto;scroll-behavior:auto;overflow-anchor:none;padding:0;border:0;margin:0}
.scroll-item,.scroll-item>.ix-dropdown-item{box-sizing:border-box;height:40px;min-height:40px;max-height:40px;margin:0}
.scroll-item>.ix-dropdown-item{padding:8px 12px;width:100%}
.scroll-trailing{box-sizing:border-box;height:120px;min-height:120px;padding:8px;background:#ffffff08}
.scroll-outer.nested{box-sizing:content-box;width:320px;height:200px;overflow:auto;scroll-behavior:auto;overflow-anchor:none}
.scroll-outer-spacer{height:80px;min-height:80px;box-sizing:border-box;padding:8px;background:#ffffff08}
#scroll-owner nav.ix-menu{position:relative;inset:auto;height:auto;width:360px;overflow:visible}
#scroll-owner .scroll-category-application{height:600px}
#scroll-owner .scroll-category-application nav.ix-menu{height:560px;width:52px}
#scroll-owner .ix-menu-category>[role=menuitem],#scroll-owner .ix-menu-category [data-menu-category-child]{height:40px;min-height:40px;box-sizing:border-box}
#scroll-owner .ix-menu-category-flyout>[data-menu-category-body]{height:160px;max-height:160px;min-height:160px;overflow:auto;scroll-behavior:auto}
"#;

const OBSERVE: &str = r#"
const outer=document.getElementById('scroll-probe');if(!outer)return;
const rows=[],ids=new WeakMap(),events=new WeakMap(),abort=new AbortController();let nextId=0,seq=0,eventSeq=0,alive=true,scenario='bootstrap';
const identity=el=>{if(!el)return null;if(!ids.has(el))ids.set(el,++nextId);return ids.get(el);};
const node=el=>el?{object:identity(el),id:el.id,tag:el.tagName,index:el.closest?.('[data-scroll-index]')?.dataset.scrollIndex,label:el.getAttribute?.('data-ix-dropdown-label')??el.getAttribute?.('aria-label'),connected:el.isConnected}:null;
const rect=el=>{const r=el?.getBoundingClientRect();return r?{x:r.x,y:r.y,width:r.width,height:r.height,top:r.top,bottom:r.bottom,left:r.left,right:r.right}:null;};
const active=()=>{const list=[];let el=document.activeElement;while(el){list.push(node(el));el=el.shadowRoot?.activeElement;}return list;};
const box=el=>el?{node:node(el),rect:rect(el),scrollTop:el.scrollTop,scrollLeft:el.scrollLeft,scrollHeight:el.scrollHeight,scrollWidth:el.scrollWidth,clientHeight:el.clientHeight,clientWidth:el.clientWidth,overflowY:getComputedStyle(el).overflowY}:null;
const root=()=>document.querySelector('#scroll-owner .scroll-probe-dropdown[data-ix-overlay-root]');
const snapshot=()=>{const r=root(),host=r?.querySelector('dialog'),trigger=document.querySelector('#scroll-owner [data-ix-overlay-trigger]:not([data-ix-overlay-root])');let native=false;try{native=!!host?.matches(':popover-open');}catch{}return{epoch:document.getElementById('scroll-fixture')?.dataset.epoch,mode:document.getElementById('scroll-fixture')?.dataset.mode,root:node(r),generation:r?.dataset.ixOverlayGeneration,owner:r?.dataset.ixOverlayOwner,logical:r?.dataset.ixOverlayOpen,native,host:box(host),trigger:node(trigger),activeDescendant:trigger?.getAttribute('aria-activedescendant'),publishedActive:r?.dataset.ixOverlayActiveItem,focusTarget:r?.dataset.ixOverlayFocusTarget,focusVersion:r?.dataset.ixOverlayFocusVersion,focus:active(),inner:box(document.getElementById('scroll-inner')),outer:box(document.getElementById('scroll-outer')),document:box(document.scrollingElement),viewport:{width:innerWidth,height:innerHeight},items:[...document.querySelectorAll('#scroll-inner>[data-scroll-index]>[data-ix-dropdown-item]')].map(el=>({node:node(el),rect:rect(el),tabindex:el.tabIndex,disabled:el.hasAttribute('disabled'),active:el.dataset.active})),category:[...document.querySelectorAll('#scroll-owner .ix-menu-category')].map(el=>({node:node(el),parent:node(el.querySelector(':scope>[role=menuitem]')),popup:el.dataset.menuCategoryPopupOpen,body:box(el.querySelector('[data-menu-category-body]')),inventory:[...el.querySelectorAll('[data-menu-category-child]')].map(child=>({node:node(child),rect:rect(child),tabindex:child.tabIndex}))})),rust:JSON.parse(document.getElementById('scroll-rust-json')?.value||'[]')};};
const publish=()=>{const el=document.getElementById('scroll-native-journal');if(el)el.textContent=JSON.stringify({scenario,rows:rows.slice(-24)});};
const record=(kind,detail={})=>{const row={seq:++seq,scenario,kind,time:performance.now(),...detail,state:snapshot()};rows.push(row);if(rows.length>500)rows.shift();publish();return row;};
const eventId=e=>{if(!events.has(e))events.set(e,++eventSeq);return events.get(e);};
for(const type of ['focusin','focusout','scroll'])document.addEventListener(type,event=>{if(type==='scroll'&&!outer.contains(event.target)&&event.target!==document)return;record('native-'+type,{event:eventId(event),target:node(event.composedPath()[0]),trusted:event.isTrusted});},{capture:true,passive:true,signal:abort.signal});
for(const type of ['keydown','click']){
 document.addEventListener(type,event=>{if(!outer.contains(event.target))return;const id=eventId(event);record('document-capture',{event:id,type,key:event.key,alt:event.altKey,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented});queueMicrotask(()=>{if(alive)record('queued-capture-microtask',{event:id,type,key:event.key,defaultPrevented:event.defaultPrevented});});let count=0;const frame=time=>{if(!alive)return;record('event-frame',{event:id,type,key:event.key,frame:++count,frameTime:time,defaultPrevented:event.defaultPrevented});if(count<5)requestAnimationFrame(frame);};requestAnimationFrame(frame);},{capture:true,passive:true,signal:abort.signal});
 window.addEventListener(type,event=>{if(!outer.contains(event.target))return;record('window-bubble',{event:eventId(event),type,key:event.key,defaultPrevented:event.defaultPrevented});},{passive:true,signal:abort.signal});
}
window.__ixScrollProbe={rows,snapshot,node,rect,record,setScenario:value=>{scenario=value;record('scenario');}};outer.dataset.nativeJournal='true';record('journal-ready');
try{await dioxus.recv();}finally{alive=false;abort.abort();delete window.__ixScrollProbe;delete outer.dataset.nativeJournal;}
"#;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
await dioxus.recv();
const observations=[],assertions=[],driver=[];let scenario='bootstrap';
const get=id=>document.getElementById('scroll-'+id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const frames=async(n=5)=>{for(let i=0;i<n;i++)await new Promise(requestAnimationFrame);await sleep(0);};
const api=()=>window.__ixScrollProbe,root=()=>document.querySelector('#scroll-owner .scroll-probe-dropdown[data-ix-overlay-root]'),trigger=()=>document.querySelector('#scroll-owner [data-ix-overlay-trigger]:not([data-ix-overlay-root])');
const item=index=>document.querySelector('#scroll-inner>[data-scroll-index="'+index+'"]>[data-ix-dropdown-item]');
const snapshot=()=>api()?.snapshot(),record=(kind,detail={})=>{const row={scenario,kind,time:performance.now(),...detail,state:snapshot()};driver.push(row);return row;};
const retained=el=>{const host=el?.querySelector?.('dialog');let native=false;try{native=!!host?.matches(':popover-open');}catch{}return{node:api().node(el),rect:api().rect(el),host:api().node(host),hostRect:api().rect(host),native,active:document.activeElement===el};};
const assert=(ok,label)=>{assertions.push({scenario,label,ok:!!ok});if(!ok)throw Error(label+' '+JSON.stringify(snapshot()));};
const wait=async(fn,label,ms=4500)=>{const end=performance.now()+ms;while(performance.now()<end){if(fn())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify(snapshot()));};
const control=(id,value)=>{const el=get(id);el.value=value;el.dispatchEvent(new Event('input',{bubbles:true}));record('Rust-control',{id,value});};
const key=(target,value,alt=false)=>{const e=new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,composed:true,cancelable:true,altKey:alt});record('driver-key',{key:value,alt,target:api().node(target),trusted:e.isTrusted});target.dispatchEvent(e);record('driver-key-return',{key:value,alt,defaultPrevented:e.defaultPrevented});return e;};
const fresh=async(label,mode='active')=>{scenario=label;const old=get('fixture');control('mode',mode);await wait(()=>get('fixture')!==old&&get('fixture')?.dataset.mode===mode&&root()?.dataset.ixOverlayOwner&&root()?.dataset.ixOverlayGeneration&&document.querySelector('[data-ix-overlay-transport]')&&api(),'fresh Rust fixture/native owner');api().setScenario(label);await frames(3);get('outside').focus();key(get('outside'),'Tab');trigger().focus();trigger().click();await wait(()=>snapshot()?.logical==='true'&&snapshot()?.native,'actual trigger native opening');await frames(5);assert(snapshot().items.length===18,'18 actual Rust DropdownItems');assert(get('inner').clientHeight===160&&get('inner').scrollHeight>get('inner').clientHeight,'real overflowing160px scrollbox');if(mode!=='category'){assert(Math.abs(item(0).getBoundingClientRect().height-40)<1&&Math.abs(item(17).getBoundingClientRect().top-item(0).getBoundingClientRect().top-680)<1&&Math.abs(get('inner').scrollHeight-840)<1,'actual18×40 plus120 geometry');}record('opened',{setupAdapter:'source propertyshow has no own request; port actual registered trigger click is public opening setup, excluded main-operation comparison'});};
const offsets=(inner=0,outer=0)=>{get('inner').scrollTop=inner;get('outer').scrollTop=outer;record('public-scroll-offset',{inner,outer});};
const seed=async()=>{if(get('fixture').dataset.mode==='roving'){trigger().focus();key(trigger(),'ArrowDown');await frames(8);assert(snapshot().items.some(item=>item.tabindex===0),'real roving item initialized');}offsets();if(get('fixture').dataset.mode!=='roving')trigger().focus();await frames(2);};
const operation=async(target,keys,detail={})=>{const cursor=api().rows.at(-1)?.seq??0,heldItems=snapshot().items.map(entry=>entry.node);record('operation-start',{cursor,keys,heldItems,origin:api().node(target),...detail});const events=keys.map(value=>key(!detail.fixedOrigin&&get('fixture').dataset.mode==='roving'?(document.querySelector('#scroll-inner>[data-scroll-index]>[data-ix-dropdown-item][tabindex="0"]')??target):target,value));await frames(8);const result={cursor,keys,heldItems,origin:api().node(target),original:events.map(e=>({key:e.key,defaultPrevented:e.defaultPrevented})),...detail};record('operation-settled',result);return result;};
const save=detail=>{const state=snapshot();observations.push({case:observations.length+1,label:scenario,...detail,state,journal:api().rows.filter(row=>row.scenario===scenario)});};
try{
 await wait(()=>api()&&get('probe')?.dataset.nativeJournal==='true','passive native journal');
 for(const mode of ['active','roving']){await fresh(mode+' offscreen End',mode);await seed();save(await operation(mode==='roving'?item(0):trigger(),['End']));}
 await fresh('visible Home then partial End');await seed();{const home=await operation(trigger(),['Home']);offsets(540);await frames(2);assert(Math.abs(item(17).getBoundingClientRect().bottom-get('inner').getBoundingClientRect().bottom-20)<1&&item(17).getBoundingClientRect().top>=get('inner').getBoundingClientRect().top,'actual last item bottom20px clipped');save({home,end:await operation(trigger(),['End']),initialPublicOffset:540});}
 await fresh('nested scrolling End','nested');await seed();save(await operation(trigger(),['End']));
 for(const mode of ['active','roving']){
  await fresh(mode+' same-turn End Home End',mode);await seed();
  const input=()=>mode==='roving'?(document.querySelector('#scroll-inner>[data-scroll-index]>[data-ix-dropdown-item][tabindex="0"]')??item(0)):trigger();
  const burst={...await operation(input(),['End','Home','End'],{sameTurn:true,label:'End-Home-End'}),state:snapshot()};
  const home={...await operation(input(),['Home'],{label:'drained-Home-before-Down-Down'}),state:snapshot()};
  const down={...await operation(input(),['ArrowDown','ArrowDown'],{sameTurn:true,fixedOrigin:true,label:'Down-Down-after-drained-Home'}),state:snapshot()};
  save({burst,downAfterHome:{home,down}});
 }
 await fresh('public reset after observed first focus before nearest','roving');await seed();{
  const held=item(17),cursor=api().rows.at(-1)?.seq??0;let intervention=null;
  const event=key(document.querySelector('#scroll-inner>[data-scroll-index]>[data-ix-dropdown-item][tabindex="0"]')??item(0),'End');
  await new Promise(resolve=>requestAnimationFrame(time=>{const focusObserved=api().rows.some(row=>row.seq>cursor&&row.kind==='native-focusin'&&row.target.object===api().node(held).object),actualFocused=document.activeElement===held;const before=snapshot();if(focusObserved&&actualFocused){get('inner').scrollTop=540;intervention={applied:true,time,focusObserved,actualFocused,before,after:snapshot()};api().record('public-scroll-reset-diagnostic',intervention);}else{intervention={applied:false,time,focusObserved,actualFocused,before,reason:'post-dispatch frame preceded actual native target focus; IPC boundary not repaired by delay'};api().record('public-scroll-reset-diagnostic-missed',intervention);}resolve();}));
  await frames(8);save({cursor,original:{key:event.key,defaultPrevented:event.defaultPrevented},intervention,held:api().node(held)});
 }
 await fresh('disabled last End then Alt ArrowDown','disabled');await seed();{assert(item(17).hasAttribute('disabled'),'last real item disabled before navigation');const end=await operation(trigger(),['End']);const cursor=api().rows.at(-1)?.seq??0,event=key(trigger(),'ArrowDown',true);await frames(6);save({end,alt:{cursor,defaultPrevented:event.defaultPrevented}});}
 await fresh('initial disable focus handling End','no-focus');await seed();save(await operation(trigger(),['End']));
 await fresh('Category helper and collapsed own navigation','category');await seed();{
  const home=await operation(trigger(),['Home']),parent=document.querySelector('#scroll-owner .ix-menu-category>[role=menuitem]');
  assert(parent&&document.querySelector('#scroll-owner nav.ix-menu')?.dataset.expanded==='false','actual collapsed public MenuCategory parent in Menu context');parent.focus();const enter=key(parent,'Enter');await wait(()=>document.querySelector('#scroll-owner .ix-menu-category')?.dataset.menuCategoryPopupOpen==='true','actual Category Enter opens native-owned popup');await frames(10);const body=document.querySelector('#scroll-owner [data-menu-category-body]'),owned=[...document.querySelectorAll('#scroll-owner [data-menu-category-child]')];assert(body&&owned.length===18,'real Category body and18actualMenuItems');body.scrollTop=0;await frames(2);const ownBefore=snapshot(),target=owned[0];const end=key(target,'End');await frames(10);save({home,own:{enterDefault:enter.defaultPrevented,endDefault:end.defaultPrevented,before:ownBefore,after:snapshot(),parent:api().node(parent),target:api().node(target)}});
 }
 await fresh('Rust owner scope drop after End before driver frame');await seed();{
  const held=item(17),owner=root(),cursor=api().rows.at(-1)?.seq??0,event=key(trigger(),'End');control('mount','false');await wait(()=>!get('owner')&&!owner.isConnected,'actual Rust owner scope drop');await frames(8);save({cursor,originalDefault:event.defaultPrevented,held:retained(held),owner:retained(owner),adapter:'Rust scoped unmount after public control IPC; actual drop chronology recorded, not synchronous source remove identity'});
 }
 await fresh('Rust item replacement after End before driver frame');await seed();{
  const held=item(17),cursor=api().rows.at(-1)?.seq??0,event=key(trigger(),'End');control('replace','next');await wait(()=>item(17)&&item(17)!==held&&!held.isConnected,'new actual Rust DropdownItem identity');await frames(8);save({cursor,originalDefault:event.defaultPrevented,held:retained(held),replacement:retained(item(17)),adapter:'Rust component replacement with new generated native ID; no public DropdownItem id prop, no native clone'});
 }
 assert(observations.length===12,'12 bounded raw histories');return{ok:true,observations,assertions,driver,journal:api().rows};
}catch(error){return{ok:false,error:String(error),scenario,state:snapshot(),observations,assertions,driver,journal:api()?.rows??[]};}
"#;
