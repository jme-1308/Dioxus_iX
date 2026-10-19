//! Excluded visible Linux/WebView nested Header regression. DOM input is synthetic.
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

static TOKEN: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default, Serialize)]
struct Life {
    mounts: usize,
    drops: usize,
    native_mounts: usize,
    live: Vec<usize>,
}
#[derive(Clone, Default)]
struct Records(
    Rc<RefCell<BTreeMap<&'static str, Life>>>,
    Rc<RefCell<Vec<Value>>>,
);
impl PartialEq for Records {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Records {
    fn log(&self, value: Value) {
        let mut log = self.1.borrow_mut();
        log.push(value);
        if log.len() > 160 {
            log.remove(0);
        }
    }
}
#[derive(Clone, Copy, Default, PartialEq)]
enum Veto {
    #[default]
    None,
    Open,
    Close,
}
impl Veto {
    fn blocks(self, open: bool) -> bool {
        self == if open { Self::Open } else { Self::Close }
    }
}

#[component]
fn Retained(group: &'static str, records: Records, children: Element) -> Element {
    let mut count = use_signal(|| 0usize);
    let token = use_hook(|| TOKEN.fetch_add(1, Ordering::Relaxed));
    let committed = use_hook(|| Rc::new(Cell::new(false)));
    let committed_effect = committed.clone();
    let effect_records = records.clone();
    use_effect(move || {
        if !committed_effect.replace(true) {
            let mut all = effect_records.0.borrow_mut();
            let life = all.entry(group).or_default();
            life.mounts += 1;
            life.live.push(token);
        }
    });
    let drop_records = records.clone();
    use_drop(move || {
        if committed.get() {
            let mut all = drop_records.0.borrow_mut();
            let life = all.entry(group).or_default();
            life.drops += 1;
            life.live.retain(|item| *item != token);
        }
    });
    rsx! {div {id:"hnp-{group}-scope",class:"hnp-content","data-token":token.to_string(),"data-count":count().to_string(),
        onmounted:move |_| records.0.borrow_mut().entry(group).or_default().native_mounts += 1,
        input {id:"hnp-{group}-input",r#type:"text",aria_label:"{group} uncontrolled value",autocomplete:"off"}
        button {id:"hnp-{group}-count",r#type:"button",onclick:move |event|{event.prevent_default();count+=1;},"{group} count {count}"}
        {children}
    }}
}

#[component]
fn Owner(
    initial_small: bool,
    records: Records,
    force: Signal<ApplicationBreakpoint>,
    layer: Signal<bool>,
    child_layer: Signal<bool>,
    root_veto: Signal<Veto>,
    child_veto: Signal<Veto>,
    mut header_present: Signal<bool>,
    submenu_present: Signal<bool>,
    mut switch_requests: Signal<usize>,
) -> Element {
    let submenu = use_dropdown();
    let child_records = records.clone();
    let changed_records = records.clone();
    let root_records = records.clone();
    let accepted_records = records.clone();
    let item_records = records.clone();
    let header = header_present().then(|| rsx! {ApplicationHeader {
        name:"Nested identity probe",aria_label_more_menu_icon_button:"More nested probe actions",
        enable_top_layer:layer(),
        onopenappswitch:move |event:MouseEvent| {event.prevent_default();switch_requests+=1;},
        onbeforeoverflowchange:move |event:HeaderOverflowEvent| {
            let veto=root_veto.peek().blocks(event.open);if veto {event.prevent_default();}
            root_records.log(json!({"kind":"root-request","open":event.open,"previous":event.previous,"veto":veto}));
        },
        onoverflowchange:move |open| accepted_records.log(json!({"kind":"root-accepted","open":open})),
        actions:rsx! {Retained {group:"default",records:records.clone(),
            DropdownButton {label:"Nested probe menu",aria_label:"Nested probe menu",enable_top_layer:child_layer(),
                onbeforechange:move |event:DropdownChangeEvent| {
                    let veto=child_veto.peek().blocks(event.open);if veto {event.prevent_default();}
                    child_records.log(json!({"kind":"child-request","open":event.open,"previous":event.previous,"veto":veto}));
                },
                onchange:move |open| changed_records.log(json!({"kind":"child-accepted","open":open})),
                DropdownItem {label:"Nested probe leaf",onitemclick:move |event:DropdownItemClickEvent| {
                    event.prevent_default();
                    item_records.log(json!({"kind":"item-notification","label":event.label,"veto":event.default_prevented()}));
                }}
                DropdownItem {label:"Unavailable probe item",disabled:true}
                DropdownItem {label:"Nested probe submenu",dropdown:submenu}
            }
        }},
        overflow:rsx! {Retained {group:"overflow",records:records.clone(),
            button {id:"hnp-inside",r#type:"button","Inside ordinary action"}
            button {id:"hnp-inside-prevent",r#type:"button",onclick:move |event|event.prevent_default(),"Inside prevent click"}
            button {id:"hnp-remove-header",r#type:"button",onclick:move |event|{event.prevent_default();header_present.set(false);},"Unmount Header"}
        }},
    }});
    rsx! {div {id:"hnp-owner","data-initial-small":initial_small.to_string(),
        Application {force_breakpoint:force(),app_switch_config:Some(AppSwitchConfig::default()),header,p {"Native nested regression content."}}
        if submenu_present() {
            Dropdown {controller:submenu,label:"Detached probe surface",enable_top_layer:child_layer(),
                Retained {group:"detached",records,
                    DropdownItem {label:"Detached probe leaf"}
                    DropdownItem {label:"Detached probe second"}
                }
            }
        }
    }}
}

const STYLES: &str = r"body{margin:0}.hnp-toolbar{display:flex;flex-wrap:wrap;gap:.4rem;margin:.75rem 0}.hnp-content{display:inline-flex;align-items:center;gap:.25rem}.hnp-content input{width:5rem;min-width:0}.hnp-content button{font:inherit}#hnp-fixture{font:1rem/1.5 system-ui,sans-serif;padding:1rem}#hnp-stage{height:32rem;border:1px solid var(--ix-color-border)}#hnp-owner,#hnp-owner>.ix-application{height:100%}#hnp-metrics{max-height:8rem;overflow:auto;white-space:pre-wrap;font-size:.75rem}";

#[component]
fn Fixture() -> Element {
    let records = use_hook(Records::default);
    let mut force = use_signal(|| ApplicationBreakpoint::Large);
    let mut initial_small = use_signal(|| false);
    let mut generation = use_signal(|| 1usize);
    let mut layer = use_signal(|| false);
    let mut child_layer = use_signal(|| false);
    let mut root_veto = use_signal(Veto::default);
    let mut child_veto = use_signal(Veto::default);
    let mut header_present = use_signal(|| true);
    let mut owner_present = use_signal(|| true);
    let mut submenu_present = use_signal(|| true);
    let switch_requests = use_signal(|| 0usize);
    let mut snapshot = use_signal(|| "{}".to_string());
    let mut revision = use_signal(|| 0usize);
    let snapshot_records = records.clone();
    rsx! {div {id:"hnp-fixture","data-generation":generation().to_string(),
        style { {STYLES} }
        h1 {"Nested Header native regression"}
        p {"Synthetic input and actual native focus/presentation. Forced modes do not certify physical viewport or source global service behavior."}
        div {class:"hnp-toolbar",
            for (id,label,small) in [("fresh-large","Fresh Large owner",false),("fresh-small","Fresh Small owner",true)] {
                button {id:"hnp-{id}",r#type:"button",onclick:move |_|{force.set(if small{ApplicationBreakpoint::Small}else{ApplicationBreakpoint::Large});initial_small.set(small);header_present.set(true);owner_present.set(true);submenu_present.set(true);root_veto.set(Veto::None);child_veto.set(Veto::None);layer.set(false);child_layer.set(false);generation+=1;},"{label}"}
            }
            button {id:"hnp-large",r#type:"button",onclick:move |_|force.set(ApplicationBreakpoint::Large),"Large"}
            button {id:"hnp-small",r#type:"button",onclick:move |_|force.set(ApplicationBreakpoint::Small),"Small"}
            button {id:"hnp-layer",r#type:"button",onclick:move |_|layer.toggle(),"Toggle Header top layer"}
            button {id:"hnp-child-layer",r#type:"button",onclick:move |_|child_layer.toggle(),"Toggle child top layer"}
            for (id,veto,child) in [("root-open-veto",Veto::Open,false),("root-close-veto",Veto::Close,false),("child-open-veto",Veto::Open,true),("child-close-veto",Veto::Close,true)] {
                button {id:"hnp-{id}",r#type:"button",onclick:move |_|if child{child_veto.set(veto)}else{root_veto.set(veto)},"{id}"}
            }
            button {id:"hnp-veto-none",r#type:"button",onclick:move |_|{root_veto.set(Veto::None);child_veto.set(Veto::None);},"Clear request vetoes"}
            button {id:"hnp-submenu-toggle",r#type:"button",onclick:move |_|submenu_present.toggle(),"Unmount / remount submenu"}
            button {id:"hnp-header-toggle",r#type:"button",onclick:move |_|header_present.toggle(),"Unmount / remount Header"}
            button {id:"hnp-owner-toggle",r#type:"button",onclick:move |_|owner_present.toggle(),"Unmount / remount owner"}
            button {id:"hnp-outside-prevent",r#type:"button",onclick:move |event|event.prevent_default(),"Outside prevented click"}
            button {id:"hnp-snapshot",r#type:"button",onclick:move |_|{revision+=1;snapshot.set(json!({"revision":*revision.peek(),"groups":snapshot_records.0.borrow().clone(),"events":snapshot_records.1.borrow().clone(),"switch_requests":*switch_requests.peek()}).to_string());},"Capture committed records"}
        }
        div {id:"hnp-stage",
            if owner_present() {for key in [generation()] {Owner {key:"{key}",initial_small:initial_small(),records:records.clone(),force,layer,child_layer,root_veto,child_veto,header_present,submenu_present,switch_requests}}}
        }
        pre {id:"hnp-metrics","{snapshot}"}
    }}
}

const VERIFY: &str = r#"
try{
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms)),get=id=>document.getElementById('hnp-'+id),checks=[],defaults=[],itemHits=[],focusResults=[],bubbleKeys=new WeakSet();document.addEventListener('keydown',event=>bubbleKeys.add(event),{passive:true});
 const header=()=>get('stage')?.querySelector('header.ix-application-header'),headerRoot=()=>header()?.querySelector('.ix-header-layout'),more=()=>header()?.querySelector('[data-header-more-button]');
 const child=()=>get('default-scope')?.querySelector('[role="button"][data-ix-overlay-trigger]'),submenuTrigger=()=>[...get('default-scope')?.querySelectorAll('[data-ix-dropdown-item]')??[]].find(node=>node.textContent.trim()==='Nested probe submenu');
 const linkedRoot=()=>get('owner')?.querySelector('.ix-dropdown[aria-label="Detached probe surface"]')??get('owner')?.querySelector('.ix-dropdown[id^="ix-dropdown-surface-owner-"]');
 const childRoot=()=>child(),subRoot=()=>linkedRoot(),shown=node=>!!node&&node.getAttribute('data-ix-overlay-open')==='true';
 const host=node=>node?.querySelector(':scope > [data-ix-overlay-host]'),item=(root,label)=>[...host(root)?.querySelectorAll('[data-ix-dropdown-item]')??[]].find(node=>node.textContent.trim()===label);
 const activeItem=(root,owner)=>{const id=root?.getAttribute('data-ix-overlay-active-item'),node=id?document.getElementById(id):null;return node?.isConnected&&root.getAttribute('aria-activedescendant')===id&&root.getAttribute('data-ix-overlay-items-owner')===owner.getAttribute('data-ix-overlay-root')&&node.getAttribute('data-ix-dropdown-owner')===owner.getAttribute('data-ix-overlay-root')&&node.closest('[data-ix-overlay-host]')===host(owner)&&node.getAttribute('aria-disabled')!=='true'?node:null;};
 const nativePopovers=()=>[...document.querySelectorAll(':popover-open')].filter(node=>node.matches('[data-ix-overlay-host],[data-header-popup-host]'));
 const state=()=>({fixtureGeneration:get('fixture')?.dataset.generation,header:header()?.className,root:shown(headerRoot()),child:shown(childRoot()),submenu:shown(subRoot()),focus:document.activeElement?.id||document.activeElement?.getAttribute('aria-label')||document.activeElement?.tagName,popovers:nativePopovers().map(node=>node.id),metrics:get('metrics')?.textContent});
 const assert=(condition,message)=>{if(!condition)throw Error(message+' '+JSON.stringify(state()));};
 const wait=async(predicate,message)=>{const end=performance.now()+6000;while(performance.now()<end){if(predicate())return;await sleep(20);}throw Error('Timeout '+(typeof message==='function'?message():message)+' '+JSON.stringify(state()));};
 const click=node=>{assert(node?.isConnected,'connected click target');const event=new MouseEvent('click',{bubbles:true,cancelable:true,composed:true});node.dispatchEvent(event);defaults.push({type:'click',prevented:event.defaultPrevented,target:node.id||node.getAttribute('aria-label')});return event;};
 const centerHit=node=>{if(!(node instanceof HTMLElement)||!node.isConnected)return{ok:false};const r=node.getBoundingClientRect(),style=getComputedStyle(node),x=r.x+r.width/2,y=r.y+r.height/2,target=document.elementFromPoint(x,y);return{ok:!!r.width&&!!r.height&&node.getClientRects().length>0&&style.display!=='none'&&style.visibility==='visible'&&r.left>=0&&r.top>=0&&r.right<=document.documentElement.clientWidth&&r.bottom<=document.documentElement.clientHeight&&!!target&&(target===node||node.contains(target)),node:node.id,label:node.textContent.trim(),rect:{x:r.x,y:r.y,width:r.width,height:r.height},hit:target?.id||target?.tagName};};
 const hitDiagnostics=node=>{const describe=element=>{if(!(element instanceof Element))return null;const r=element.getBoundingClientRect(),s=getComputedStyle(element);return{id:element.id,tag:element.tagName,role:element.getAttribute('role'),hidden:element.hidden,inert:element.inert,rect:{x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom},styles:{display:s.display,visibility:s.visibility,position:s.position,overflow:s.overflow,overflowX:s.overflowX,overflowY:s.overflowY,transform:s.transform,translate:s.translate,contain:s.contain,filter:s.filter,pointerEvents:s.pointerEvents,clipPath:s.clipPath},dialog:element instanceof HTMLDialogElement?{open:element.open,popover:element.getAttribute('popover'),popoverOpen:element.matches(':popover-open')}:null};};const ancestors=[];let current=node;while(current instanceof Element&&ancestors.length<12){ancestors.push(describe(current));current=current.parentElement;}return{center:centerHit(node),nearestHost:describe(node?.closest('[data-ix-overlay-host],[data-header-popup-host]')),ancestors};};
 const clickItem=async(node,label)=>{await wait(()=>centerHit(node).ok,()=> 'actual visible item center hit '+label+' '+JSON.stringify(hitDiagnostics(node)));itemHits.push({label,...centerHit(node)});return click(node);};
 const settledFocus=async(node,label)=>{await wait(()=>document.activeElement===node,'settled focus '+label);await new Promise(requestAnimationFrame);await new Promise(requestAnimationFrame);await sleep(0);assert(document.activeElement===node,'focus remains after paint '+label);focusResults.push({label,node:node.id||node.tagName});};
 const key=(node,value)=>{assert(node?.isConnected,'connected key target');node.focus();const event=new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true,composed:true});node.dispatchEvent(event);defaults.push({key:value,prevented:event.defaultPrevented,reachedDocumentBubble:bubbleKeys.has(event),target:node.id||node.getAttribute('aria-label')});if(value==='Escape')assert(!bubbleKeys.has(event),'Escape stopped before document bubble');return event;};
 const capture=async()=>{const prior=get('metrics').textContent;click(get('snapshot'));await wait(()=>get('metrics').textContent!==prior,'new record snapshot');return JSON.parse(get('metrics').textContent);};
 const captureUntil=async(predicate,message)=>{const end=performance.now()+6000;let records;while(performance.now()<end){records=await capture();if(predicate(records))return records;await sleep(20);}throw Error('Timeout committed records '+message+' '+JSON.stringify(records));};
 const ready=async small=>{await wait(()=>headerRoot()?.getAttribute('data-ix-overlay-ready')==='true'&&header()?.classList.contains(small?'ix-app-small':'ix-app-large')&&childRoot()&&subRoot(),'known logical viewport and roots');await wait(()=>[headerRoot(),childRoot(),subRoot()].every(root=>root?.isConnected&&Number(root.getAttribute('data-ix-overlay-generation'))>0&&[...document.querySelectorAll('input[data-ix-overlay-transport][data-ix-overlay-native="true"]')].some(input=>input.getAttribute('data-ix-overlay-transport')===root.getAttribute('data-ix-overlay-owner'))&&document.documentElement.getAttribute('data-ix-overlay-registry-native')===root.getAttribute('data-ix-overlay-owner')),'shared bridge owns current connected roots');};
 const openRoot=async()=>{if(!shown(headerRoot()))click(more());await wait(()=>shown(headerRoot()),'Header open');};
 const openChild=async()=>{if(!shown(childRoot()))click(child());await wait(()=>shown(childRoot()),'child open');};
 const openSub=async()=>{if(!shown(subRoot()))await clickItem(submenuTrigger(),'linked submenu trigger');await wait(()=>shown(subRoot()),'detached submenu open');};
 const allOpen=async()=>{await openRoot();await openChild();await openSub();};
 const fresh=async small=>{const previous=get('fixture').dataset.generation;click(get(small?'fresh-small':'fresh-large'));await wait(()=>get('fixture').dataset.generation!==previous,'fresh owner generation');await ready(small);};
 await ready(false);let records=await captureUntil(records=>['default','overflow','detached'].every(group=>records.groups[group]?.mounts===1&&records.groups[group]?.drops===0&&records.groups[group]?.native_mounts===1),'one committed/native scope each');
 await openChild();await openSub();assert(!shown(headerRoot()),'wide child chain leaves Header closed');key(child(),'Escape');await wait(()=>!shown(childRoot())&&!shown(subRoot()),'wide Escape closes child chain');
 click(get('small'));await ready(true);await allOpen();let escape=key(child(),'Escape');assert(!escape.defaultPrevented,'Escape original default retained');await wait(()=>!shown(childRoot())&&!shown(subRoot())&&shown(headerRoot()),'large-to-small cached Escape retains Header');await settledFocus(child(),'large-to-small Escape returns DropdownButton trigger');checks.push('source-history large→small Escape');
 await fresh(true);await allOpen();escape=key(child(),'Escape');assert(!escape.defaultPrevented,'fresh-small Escape original default retained');await wait(()=>!shown(headerRoot())&&!shown(childRoot())&&!shown(subRoot()),'fresh-small Escape closes all levels');await settledFocus(document.body,'fresh-small Escape releases BODY focus');checks.push('fresh-small discovery Escape');
 click(get('child-open-veto'));await openRoot();click(child());await sleep(80);assert(!shown(childRoot())&&shown(headerRoot()),'child open request veto');const vetoEvents=(await capture()).events,vetoIndex=vetoEvents.findLastIndex(event=>event.kind==='child-request'&&event.open===true&&event.veto===true);assert(vetoIndex>=0&&vetoEvents.slice(0,vetoIndex).some(event=>event.kind==='root-accepted'&&event.open===true)&&!vetoEvents.slice(vetoIndex+1).some(event=>event.kind==='child-accepted'&&event.open===true),'vetoed child request follows accepted root but has no accepted child opening');click(get('veto-none'));await wait(()=>!shown(headerRoot())&&!shown(childRoot())&&!shown(subRoot()),'snapshot/clear outside closure committed before retry opening');await allOpen();
 key(child(),'Escape');await wait(()=>!shown(headerRoot()),'close before next veto');click(get('child-close-veto'));await openRoot();await openChild();click(more());await wait(()=>!shown(headerRoot())&&shown(childRoot()),'child-close veto survives accepted parent close');click(get('veto-none'));await sleep(40);
 await fresh(true);click(get('root-close-veto'));await allOpen();click(more());await wait(()=>shown(headerRoot())&&!shown(childRoot())&&!shown(subRoot()),'parent veto retains root but children close');checks.push('separate parent/child veto survivors');
 await fresh(true);await openRoot();const insideClick=click(get('inside-prevent'));assert(insideClick.defaultPrevented,'inside native click prevented');await sleep(60);assert(shown(headerRoot()),'prevented inside retains Header');const outsideClick=click(get('outside-prevent'));assert(outsideClick.defaultPrevented,'outside native click prevented');await wait(()=>!shown(headerRoot()),'prevented outside still dismisses');
 await allOpen();const leaf=item(childRoot(),'Nested probe leaf');const leafClick=await clickItem(leaf,'veto-notification child leaf');assert(leafClick.defaultPrevented,'child-host leaf click prevented');await wait(()=>shown(headerRoot())&&!shown(childRoot()),'internal item notification veto still closes child and retains Header');const notified=(await capture()).events.filter(event=>event.kind==='item-notification');assert(notified.length===1&&notified[0].label==='Nested probe leaf'&&notified[0].veto===true,'exactly one separately vetoed item notification');await wait(()=>!shown(headerRoot())&&!shown(childRoot())&&!shown(subRoot()),'notification snapshot outside closure committed before reopening');await openRoot();await openChild();await openSub();const detachedClick=await clickItem(item(subRoot(),'Detached probe leaf'),'detached leaf');assert(!detachedClick.defaultPrevented,'detached leaf click unprevented');await wait(()=>!shown(headerRoot())&&!shown(childRoot())&&!shown(subRoot()),'detached leaf closes chain');checks.push('inside/outside and child-host/detached leaf paths');
 await openRoot();const down=key(child(),'ArrowDown');assert(down.defaultPrevented,'child Down prevented');await wait(()=>shown(childRoot())&&document.activeElement===child(),'active-descendant keeps native trigger');await wait(()=>activeItem(childRoot(),childRoot())===item(childRoot(),'Nested probe leaf'),'Rust adapter Down active-item and ARIA ownership');key(child(),'End');await wait(()=>activeItem(childRoot(),childRoot())===submenuTrigger(),'Rust adapter End active-item and ARIA ownership');const right=key(child(),'ArrowRight');assert(!right.defaultPrevented,'submenu Right unprevented');await wait(()=>shown(subRoot()),'Right opens external submenu');await wait(()=>activeItem(subRoot(),subRoot())===item(subRoot(),'Detached probe leaf')&&activeItem(childRoot(),subRoot())===item(subRoot(),'Detached probe leaf'),'Rust adapter Right forwards active/ARIA to owned detached first item');const forwardedDown=key(child(),'ArrowDown');assert(forwardedDown.defaultPrevented,'forwarded submenu Down prevented');await wait(()=>activeItem(subRoot(),subRoot())===item(subRoot(),'Detached probe second')&&activeItem(childRoot(),subRoot())===item(subRoot(),'Detached probe second')&&document.activeElement===child(),'Rust adapter Down moves actual detached submenu item with trigger focus');const left=key(child(),'ArrowLeft');assert(!left.defaultPrevented,'Left unprevented');await wait(()=>shown(headerRoot())&&!shown(childRoot())&&!shown(subRoot()),'forwarded Left closes nested chain');checks.push('source key flags; Rust adapter owned active-item/ARIA and detached key forwarding (source ARIA mapping unobserved)');
 await fresh(true);click(get('layer'));click(get('child-layer'));await sleep(60);await allOpen();await wait(()=>nativePopovers().length===3,'three actual native top-layer surfaces');assert(nativePopovers().every(node=>node.getAttribute('popover')==='manual'&&(!(node instanceof HTMLDialogElement)||!node.open)),'manual nonmodal native presentation');key(child(),'Escape');await wait(()=>nativePopovers().length===0&&!shown(headerRoot()),'top-layer hierarchy Escape cleanup');checks.push('actual manual native popover membership');
 await openRoot();click(get('default-count'));await wait(()=>get('default-scope').dataset.count==='1','default local count');await openRoot();click(get('overflow-count'));await wait(()=>get('overflow-scope').dataset.count==='1','overflow local count');await allOpen();click(get('detached-count'));await wait(()=>get('detached-scope').dataset.count==='1','detached local count');key(child(),'Escape');await wait(()=>!shown(headerRoot()),'close after local count seeds');
 const inputs=Object.fromEntries(['default','overflow','detached'].map(group=>[group,get(group+'-input')])),scopes=Object.fromEntries(['default','overflow','detached'].map(group=>[group,get(group+'-scope')]));for(const[group,node]of Object.entries(inputs)){node.value=group+' retained';node.dispatchEvent(new Event('input',{bubbles:true}));}
 const beforeUnmount=await capture();click(get('large'));await ready(false);click(get('small'));await ready(true);await allOpen();for(const[group,node]of Object.entries(inputs))assert(get(group+'-input')===node&&get(group+'-scope')===scopes[group]&&node.value===group+' retained'&&scopes[group].dataset.count==='1','retained native/scope/local state '+group);
 click(get('remove-header'));await wait(()=>!header()&&nativePopovers().length===0&&!shown(subRoot()),'Header unmount closes descendant native overlays');const detachedInput=inputs.detached;const removed=await capture();assert(['default','overflow'].every(group=>removed.groups[group].drops===beforeUnmount.groups[group].drops+1)&&removed.groups.detached.drops===beforeUnmount.groups.detached.drops,'Header drops owned scopes but retains external scope');click(get('header-toggle'));await ready(true);assert(get('detached-input')===detachedInput,'external surface content scope persists Header remount');const restored=await captureUntil(records=>['default','overflow'].every(group=>records.groups[group].mounts===removed.groups[group].mounts+1),'Header remount commits fresh scopes');await allOpen();const stale=childRoot();click(get('owner-toggle'));await wait(()=>!get('owner')&&nativePopovers().length===0,'owner unmount cleans popovers');const dropped=await capture();assert(['default','overflow','detached'].every(group=>dropped.groups[group].drops===restored.groups[group].drops+1&&dropped.groups[group].live.length===0),'owner drops all remaining scopes');stale.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true,composed:true}));await sleep(40);assert(!get('owner')&&nativePopovers().length===0,'stale detached key cannot revive owner');click(get('owner-toggle'));await ready(true);records=await capture();assert(records.switch_requests===0,'nested actions never request app switch');checks.push('retained placement and Header/owner cleanup');
 return{ok:true,checks,defaults,itemHits,focusResults,records,input:'synthetic click/input/keydown with programmatic focus/value; actual LinuxWebView DOM/popovers',limits:['not physical/trusted browser default activation, AT or full Brand geometry','forced logical mode history is not source global viewport lifecycle proof','prior B18.1 opener/exact navigation requires its separate unchanged probe','cross-type Popover, explicit roving/open-child Tab and all pending races unverified']};
}catch(error){return{ok:false,error:String(error)}}
"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(100)).await;
            eprintln!("Nested Header native probe timed out");
            std::process::exit(1);
        });
    });
    rsx! {IxTheme {scheme:ColorScheme::Dark,div {onmounted:move |_|{let window=window.clone();spawn(async move{let query=dom_query::eval(VERIFY);let _=query.send(());match query.join::<Value>().await{Ok(result)if result["ok"]==true=>{println!("Nested Header native checks: {result}");window.close();},result=>{eprintln!("Nested Header native failure: {result:?}");std::process::exit(1);}}});},Fixture {}}}}
}
fn main() {
    let config = dioxus::desktop::Config::new().with_window(
        dioxus::desktop::WindowBuilder::new()
            .with_title("Nested Header native regression")
            .with_inner_size(dioxus::desktop::LogicalSize::new(1440.0, 1100.0))
            .with_visible(true),
    );
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}
