//! Excluded linked-submenu diagnostic. Application intent and public commands
//! live in Rust; native scripts only deliver original input and observe facts.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc};

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX linked submenu histories")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1100.0, 950.0)),
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
    eprintln!("Run the desktop or web linked submenu probe");
}

const CASES: [usize; 11] = [1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12];
fn parent_roving(case_id: usize) -> bool {
    matches!(case_id, 3 | 4 | 5 | 6 | 12)
}
fn child_roving(case_id: usize) -> bool {
    matches!(case_id, 3..=7)
}
fn trigger_kind(case_id: usize) -> &'static str {
    match case_id {
        2 => "select",
        3 => "menu",
        4 => "category",
        5 => "native-opt-in",
        _ => "dropdown",
    }
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

#[component]
fn Probe() -> Element {
    let mut epoch = use_signal(|| 0_usize);
    let mut selected = use_signal(|| 1_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(120)).await;
            eprintln!("Linked submenu native timeout");
            std::process::exit(1);
        });
    });
    rsx! {IxTheme{scheme:ColorScheme::Dark,
        style{{STYLES}}
        main{id:"linked-probe",style:"padding:20px;min-height:900px",
            onmounted:move |_|{
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move{let query=dom_query::eval(VERIFY);let _=query.send(());match query.join::<Value>().await{
                    Ok(value) if value["ok"]==true=>{println!("Dropdown linked native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Dropdown linked native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Dropdown linked native transport failure: {error:?}");std::process::exit(1);}
                }});}
            },
            h1{"Linked submenu histories"}
            p{"Eleven public-input histories. Each input has twelve fixed native frame observations; source callback case L08 remains unsupported."}
            div{style:"display:flex;gap:6px;flex-wrap:wrap",for next in CASES{button{id:format!("linked-fresh-{next}"),onclick:move |_|{selected.set(next);epoch+=1;},"Fresh L{next:02}"}}}
            input{id:"linked-mode",r#type:"hidden",oninput:move |event|{let value=event.value().parse::<usize>().unwrap_or(1);selected.set(if CASES.contains(&value){value}else{1});epoch+=1;}}
            button{id:"linked-outside","Outside keyboard target"}
            for current in [epoch()]{Fixture{key:"{current}",epoch:current,case_id:selected()}}
            NativeJournal{}
            pre{id:"linked-native-log","aria-label":"Native linked submenu observations",style:"max-height:160px;overflow:auto;white-space:pre-wrap;font-size:12px"}
        }
    }}
}

#[component]
fn Fixture(epoch: usize, case_id: usize) -> Element {
    let parent = use_dropdown();
    let child_a = use_dropdown();
    let child_b = use_dropdown();
    let rows = use_signal(Vec::<Value>::new);
    rsx! {section{id:"linked-fixture","data-epoch":epoch.to_string(),"data-case":case_id.to_string(),"data-parent-roving":parent_roving(case_id).to_string(),"data-child-roving":child_roving(case_id).to_string(),
        div{id:"linked-owner",
            div{id:"linked-parent-trigger-slot",DropdownTrigger{dropdown:parent,"Open parent"}}
            Application{force_breakpoint:ApplicationBreakpoint::Large,class:"linked-application",
                menu:rsx!{Menu{label:"Linked real Menu context",start_expanded:false,pinned:false,
                    Dropdown{controller:parent,label:"Parent linked hosts",class:"linked-parent",navigation_mode:if parent_roving(case_id){DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},
                        close_behavior:DropdownCloseBehavior::None,enable_top_layer:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
                        onbeforechange:move |event:DropdownChangeEvent|record(rows,"request",json!({"role":"parent","open":event.open,"previous":event.previous,"veto":false,"current":parent.is_open()})),
                        onchange:move |open|record(rows,"completion",json!({"role":"parent","open":open,"current":parent.is_open()})),
                        div{id:"linked-parent-content","data-linked-role":"parent-content",
                            LinkedHost{kind:trigger_kind(case_id),name:"a",rows}
                            DropdownItem{id:"linked-parent-leaf","data-linked-role":"parent-leaf",label:"Ordinary parent leaf",onitemclick:move |_|record(rows,"item-click",json!({"role":"parent-leaf"}))}
                            if case_id==12{LinkedHost{kind:"dropdown",name:"b",rows}}
                        }
                    }
                }},
                div{}
            }
            ChildSurface{name:"a",controller:child_a,case_id,rows}
            if case_id==12{ChildSurface{name:"b",controller:child_b,case_id,rows}}
        }
        input{id:"linked-command",r#type:"hidden",oninput:move |event|{
            let action=event.value();record(rows,"command-before",json!({"action":action,"parent":parent.is_open(),"childA":child_a.is_open(),"childB":child_b.is_open()}));
            match action.as_str(){"parent-open"=>parent.open(),"parent-close"=>parent.close(false),"child-a-open"=>child_a.open(),"child-a-close"=>child_a.close(false),"child-b-open"=>child_b.open(),_=>{}}
            record(rows,"command-return",json!({"action":action,"parent":parent.is_open(),"childA":child_a.is_open(),"childB":child_b.is_open()}));
        }}
        div{style:"display:flex;gap:6px;flex-wrap:wrap",
            button{id:"linked-public-child-open",onmousedown:move |event|event.prevent_default(),onclick:move |_|child_a.open(),"Public child open()"}
            button{id:"linked-public-child-close",onmousedown:move |event|event.prevent_default(),onclick:move |_|child_a.close(false),"Public child close(false)"}
        }
        input{id:"linked-rust-json",r#type:"hidden",value:serde_json::to_string(&rows()).unwrap_or_default()}
    }}
}

const HOST_STYLE: &str = "display:block;box-sizing:border-box;min-height:36px;margin:0";
#[component]
fn LinkedHost(kind: &'static str, name: &'static str, rows: Signal<Vec<Value>>) -> Element {
    let id = format!("linked-host-{name}");
    let role = format!("linked-{name}");
    rsx! {div{class:"linked-member-wrapper","data-linked-member":role.clone(),
        onclick:move |_|record(rows,"member-click",json!({"role":format!("linked-{name}")})),
        match kind{
            "select"=>rsx!{SelectItem{id:id.clone(),"data-linked-role":role.clone(),style:HOST_STYLE,value:role.clone(),label:"Linked Select",onclick:move |_|record(rows,"host-click",json!({"role":format!("linked-{name}"),"kind":"select"})),onitemclick:move |value|record(rows,"select-item-click",json!({"role":format!("linked-{name}"),"value":value}))}},
            "menu"=>rsx!{MenuItem{id:id.clone(),"data-linked-role":role.clone(),style:HOST_STYLE,label:"Linked MenuItem",close_on_select:false,onclick:move |_|record(rows,"host-click",json!({"role":format!("linked-{name}"),"kind":"menu"}))}},
            "category"=>rsx!{MenuCategory{id:id.clone(),"data-linked-role":role.clone(),style:HOST_STYLE,label:"Linked Category",
                MenuItem{id:"linked-category-child-0","data-linked-role":"category-child-0",label:"Category child one",close_on_select:false}
                MenuItem{id:"linked-category-child-1","data-linked-role":"category-child-1",label:"Category child two",close_on_select:false}
            }},
            "native-opt-in"=>rsx!{button{id:id.clone(),"data-linked-role":role.clone(),style:HOST_STYLE,r#type:"button","data-ix-roving-item":"",onclick:move |_|record(rows,"host-click",json!({"role":format!("linked-{name}"),"kind":"native-opt-in"})),"Linked native opt-in"}},
            _=>rsx!{DropdownItem{id:id.clone(),"data-linked-role":role.clone(),style:HOST_STYLE,label:format!("Linked Dropdown {name}"),onclick:move |_|record(rows,"host-click",json!({"role":format!("linked-{name}"),"kind":"dropdown"})),onitemclick:move |_|record(rows,"item-click",json!({"role":format!("linked-{name}")}))}},
        }
    }}
}

#[component]
fn ChildSurface(
    name: &'static str,
    controller: DropdownHandle,
    case_id: usize,
    rows: Signal<Vec<Value>>,
) -> Element {
    let roving = child_roving(case_id);
    rsx! {Dropdown{controller,trigger:format!("linked-host-{name}"),label:format!("External child {name}"),class:format!("linked-child linked-child-{name}"),
        navigation_mode:if roving{DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},
        close_behavior:DropdownCloseBehavior::None,enable_top_layer:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
        focus_checked_item:true,disable_focus_handling:case_id==7,
        onbeforechange:move |event:DropdownChangeEvent|{let veto=case_id==9&&event.open;if veto{event.prevent_default();}record(rows,"request",json!({"role":format!("child-{name}"),"open":event.open,"previous":event.previous,"veto":veto,"current":controller.is_open()}));},
        onchange:move |open|record(rows,"completion",json!({"role":format!("child-{name}"),"open":open,"current":controller.is_open()})),
        div{id:format!("linked-child-{name}-content"),"data-linked-role":format!("child-{name}-content"),class:"linked-child-content",
            DropdownItem{id:format!("linked-child-{name}-disabled"),"data-linked-role":format!("child-{name}-disabled"),label:"Disabled child item",disabled:true}
            DropdownItem{id:format!("linked-child-{name}-first"),"data-linked-role":format!("child-{name}-first"),label:"First child item",onitemclick:move |_|record(rows,"item-click",json!({"role":format!("child-{name}-first")}))}
            DropdownItem{id:format!("linked-child-{name}-checked"),"data-linked-role":format!("child-{name}-checked"),label:"Checked child item",checked:true,tabindex:roving.then_some("4"),onitemclick:move |_|record(rows,"item-click",json!({"role":format!("child-{name}-checked")}))}
            DropdownItem{id:format!("linked-child-{name}-last"),"data-linked-role":format!("child-{name}-last"),label:"Last child item",onitemclick:move |_|record(rows,"item-click",json!({"role":format!("child-{name}-last")}))}
        }
    }}
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
#linked-fixture{padding-top:12px;min-height:580px}
#linked-owner .linked-application{height:560px}
#linked-owner nav.ix-menu{position:relative;inset:auto;height:540px;width:52px;overflow:visible}
#linked-parent-content{width:320px;max-height:360px;overflow:auto}
#linked-parent-content [data-linked-role="parent-leaf"]{display:block;box-sizing:border-box;min-height:36px;margin:0}
.linked-child-content{width:260px;max-height:160px;overflow:auto}
.linked-child-content>[data-linked-role]{display:block;box-sizing:border-box;min-height:40px;margin:0}
#linked-owner .linked-parent>dialog,#linked-owner .linked-child>dialog{padding:0;max-height:440px}
"#;

const OBSERVE: &str = r#"
const probe=document.getElementById('linked-probe');if(!probe)return;
const rows=[],objects=new WeakMap(),events=new WeakMap(),queryReports=new WeakMap(),abort=new AbortController();let sequence=0,objectSequence=0,eventSequence=0,context='bootstrap',alive=true;
const object=el=>{if(!el)return null;if(!objects.has(el))objects.set(el,++objectSequence);return objects.get(el);};
const eventId=event=>{if(!events.has(event))events.set(event,++eventSequence);return events.get(event);};
const root=role=>document.querySelector('#linked-owner .linked-'+(role==='parent'?'parent':'child-'+role)+'[data-ix-overlay-root]');
const trigger=()=>document.querySelector('#linked-parent-trigger-slot>[data-ix-overlay-trigger]');
const role=el=>el?.getAttribute?.('data-linked-role')??el?.closest?.('[data-linked-role]')?.getAttribute('data-linked-role')??(el===trigger()?'root-trigger':el?.classList?.contains('linked-parent')?'parent-root':el?.classList?.contains('linked-child-a')?'child-a-root':el?.classList?.contains('linked-child-b')?'child-b-root':null);
const node=el=>el instanceof Element?{object:object(el),id:el.id,tag:el.tagName,role:role(el),connected:el.isConnected,part:el.getAttribute('data-ix-item-part'),control:el.hasAttribute('data-ix-item-control')}:null;
const deepActive=()=>{let el=document.activeElement;while(el?.shadowRoot?.activeElement)el=el.shadowRoot.activeElement;return el;};
const attributes=el=>{if(!(el instanceof Element))return null;const rect=el.getBoundingClientRect(),style=getComputedStyle(el);return{node:node(el),attrs:Object.fromEntries([...el.attributes].map(attr=>[attr.name,attr.value])),active:el.getAttribute('data-active'),focused:el.classList.contains('ix-focused'),triggerActive:el.classList.contains('ix-dropdown-submenu-trigger-active'),tabindex:el.getAttribute('tabindex'),aria:el.getAttribute('aria-activedescendant'),checked:el.hasAttribute('checked'),checkedProperty:typeof el.checked==='boolean'?el.checked:null,disabled:el.hasAttribute('disabled'),hidden:el.hasAttribute('hidden'),inert:el.hasAttribute('inert'),display:style.display,visibility:style.visibility,rect:{x:rect.x,y:rect.y,width:rect.width,height:rect.height},scrollTop:el.scrollTop,scrollLeft:el.scrollLeft,scrollHeight:el.scrollHeight,clientHeight:el.clientHeight};};
const surface=role=>{const r=root(role);if(!r)return null;const host=r.querySelector(':scope>dialog');let native=false;try{native=!!host?.matches(':popover-open');}catch{}return{root:attributes(r),panel:attributes(host),owner:r.dataset.ixOverlayOwner,generation:r.dataset.ixOverlayGeneration,mode:r.dataset.ixOverlayMode,logical:r.dataset.ixOverlayOpen,native,published:r.dataset.ixOverlayActiveItem,itemsOwner:r.dataset.ixOverlayItemsOwner};};
const categorySnapshot=el=>{let native=false;try{native=el.matches(':popover-open');}catch{}return{...attributes(el),native};};
const snapshot=()=>({epoch:document.getElementById('linked-fixture')?.dataset.epoch,case:document.getElementById('linked-fixture')?.dataset.case,parent:surface('parent'),childA:surface('a'),childB:surface('b'),trigger:attributes(trigger()),focus:node(document.activeElement),deepFocus:node(deepActive()),members:[...document.querySelectorAll('#linked-owner [data-linked-role]')].filter(el=>!el.getAttribute('data-linked-role').endsWith('content')).map(el=>({host:attributes(el),controls:[...el.querySelectorAll('[data-ix-item-control],[data-ix-item-part]')].filter(control=>control.closest('[data-ix-item-kind]')===el).map(attributes)})),contents:[document.getElementById('linked-parent-content'),document.getElementById('linked-child-a-content'),document.getElementById('linked-child-b-content')].filter(Boolean).map(attributes),category:[...document.querySelectorAll('#linked-owner [data-menu-category-root],#linked-owner [data-menu-category-panel],#linked-owner [data-menu-category-empty-popup]')].map(categorySnapshot),rust:JSON.parse(document.getElementById('linked-rust-json')?.value||'[]'),documentScroll:{x:scrollX,y:scrollY}});
const publish=row=>{const el=document.getElementById('linked-native-log');if(el)el.textContent=JSON.stringify({context,last:row});};
const record=(kind,details={},withState=false,publishDom=true)=>{const row={seq:++sequence,context,kind,time:performance.now(),...details,...(withState?{state:snapshot()}:{})};rows.push(row);if(rows.length>9000)rows.shift();if(publishDom)publish(row);return row;};
for(const type of ['keydown','keyup','click','focusin','focusout','scroll']){
 document.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('document-capture',{event:eventId(event),type,key:event.key,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,path:event.composedPath().filter(el=>el instanceof Element).map(node)},true);},{capture:true,passive:true,signal:abort.signal});
 window.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('window-bubble',{event:eventId(event),type,key:event.key,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented},true);},{passive:true,signal:abort.signal});
}
const queryReport=event=>{if(queryReports.has(event))return queryReports.get(event);let value=null;if(event.target.matches?.('input[data-ix-overlay-transport]'))try{const report=JSON.parse(event.target.value);if(report.descendant_focus)value={kind:'descendant-focus',owner:report.owner,...report.descendant_focus};else if(report.item_focus_phase)value={kind:'item-focus',owner:report.owner,phase:report.item_focus_phase,sequence:report.item_focus_sequence,root:report.item_focus_root,generation:report.item_focus_generation,target:report.item_focus_target,capture:report.item_focus_capture};}catch{}queryReports.set(event,value);return value;};
document.addEventListener('change',event=>{const report=queryReport(event);if(!report)return;const id=eventId(event),retain=report.phase==='retain';record(report.kind+'-change-capture',{event:id,report,defaultPrevented:event.defaultPrevented},!retain,!retain);if(!retain)queueMicrotask(()=>{if(alive)record(report.kind+'-queued-capture-microtask',{event:id,report,defaultPrevented:event.defaultPrevented},true);});},{capture:true,passive:true,signal:abort.signal});
window.addEventListener('change',event=>{const report=queryReport(event);if(report){const retain=report.phase==='retain';record(report.kind+'-change-window',{event:eventId(event),report,defaultPrevented:event.defaultPrevented},!retain,!retain);}},{passive:true,signal:abort.signal});
const observer=new MutationObserver(changes=>{const facts=changes.filter(change=>change.target.closest?.('#linked-owner')).map(change=>({node:node(change.target),attribute:change.attributeName,oldValue:change.oldValue,currentValue:change.target.getAttribute(change.attributeName)}));if(facts.length)record('native-mutation-delivery',{changes:facts},true);});
observer.observe(probe,{attributes:true,subtree:true,attributeOldValue:true,attributeFilter:['data-active','class','aria-activedescendant','aria-expanded','data-ix-overlay-active-item','data-ix-overlay-open','data-ix-overlay-items-owner','data-ix-overlay-trigger','hidden','inert','checked','tabindex']});
window.__ixLinkedProbe={rows,snapshot,node,eventId,record,deepActive,setContext:value=>{context=value;record('context',{},true);}};record('journal-ready');
try{await dioxus.recv();}finally{alive=false;abort.abort();observer.disconnect();delete window.__ixLinkedProbe;}
"#;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
await dioxus.recv();
const observations=[],assertions=[],driver=[];let context='bootstrap',heldParentTrigger=null,heldLinkA=null,heldIdentity=null;
const cases=[1,2,3,4,5,6,7,9,10,11,12],get=id=>document.getElementById('linked-'+id),api=()=>window.__ixLinkedProbe,sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const root=role=>document.querySelector('#linked-owner .linked-'+(role==='parent'?'parent':'child-'+role)+'[data-ix-overlay-root]'),trigger=()=>document.querySelector('#linked-parent-trigger-slot>[data-ix-overlay-trigger]');
const state=()=>api()?.snapshot(),record=(kind,details={})=>{const row={context,kind,time:performance.now(),...details,state:state()};driver.push(row);return row;};
const assert=(ok,label)=>{assertions.push({context,label,ok:!!ok});if(!ok)throw Error(label+' '+JSON.stringify(state()));};
const wait=async(fn,label,ms=8000)=>{const end=performance.now()+ms;while(performance.now()<end){if(fn())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify(state()));};
const frame=()=>new Promise(resolve=>requestAnimationFrame(resolve));
async function fixedFrames(count,label,event=null){for(let index=1;index<=count;index++){const timestamp=await frame();api().record('fixed-observer-frame',{label,event,index,timestamp},true);await Promise.resolve();api().record('fixed-observer-frame-microtask',{label,event,index,timestamp},true);}}
const key=(target,value,label)=>{assert(target instanceof HTMLElement&&target.isConnected,'connected actual key target');const event=new KeyboardEvent('keydown',{key:value,bubbles:true,composed:true,cancelable:true}),id=api().eventId(event);api().record('original-key-before',{label,event:id,key:value,target:api().node(target)},true);target.dispatchEvent(event);api().record('original-key-return',{label,event:id,key:value,target:api().node(target),defaultPrevented:event.defaultPrevented},true);return{event:id,key:value,target:api().node(target),defaultPrevented:event.defaultPrevented};};
const click=(target,label)=>{assert(target instanceof HTMLElement&&target.isConnected,'connected actual click target');const event=new MouseEvent('click',{bubbles:true,composed:true,cancelable:true,button:0}),id=api().eventId(event);api().record('original-click-before',{label,event:id,target:api().node(target)},true);target.dispatchEvent(event);api().record('original-click-return',{label,event:id,target:api().node(target),defaultPrevented:event.defaultPrevented},true);return{event:id,target:api().node(target),defaultPrevented:event.defaultPrevented};};
const command=action=>{api().record('public-command-before',{action},true);get('command').value=action;get('command').dispatchEvent(new Event('input',{bubbles:true}));api().record('public-command-return',{action},true);return{action};};
const identities=value=>({parent:value.parent.root.node.object,childA:value.childA.root.node.object,childB:value.childB?.root.node.object??null,trigger:value.trigger.node.object,members:value.members.map(member=>({host:member.host.node.object,controls:member.controls.map(control=>control.node.object)}))});
async function fresh(id){
 context='L'+String(id).padStart(2,'0');const old=get('fixture');get('mode').value=String(id);get('mode').dispatchEvent(new Event('input',{bubbles:true}));
 await wait(()=>get('fixture')!==old&&get('fixture')?.dataset.case===String(id)&&root('parent')?.dataset.ixOverlayOwner&&root('a')?.dataset.ixOverlayGeneration&&get('host-a')?.getAttribute('data-ix-overlay-trigger')===root('a')?.dataset.ixOverlayRoot&&(id!==12||get('host-b')?.getAttribute('data-ix-overlay-trigger')===root('b')?.dataset.ixOverlayRoot)&&api(),'fresh actual public-ID linked trigger binding');
 api().setContext(context);heldParentTrigger=trigger();heldLinkA=get('host-a');const cursor=api().rows.at(-1)?.seq??0;await fixedFrames(5,'fixture-bind');get('outside').focus();key(get('outside'),'Tab','outside-keyboard-setup');heldParentTrigger.focus();
 assert(state().parent.logical==='false'&&state().childA.logical==='false','fresh parent and child start closed');const before=state();heldIdentity=JSON.stringify(identities(before));
 const open=click(heldParentTrigger,'actual-parent-trigger-open');await Promise.resolve();api().record('parent-open-microtask',{event:open.event},true);await fixedFrames(12,'actual-parent-trigger-open',open.event);
 const opened=state();assert(opened.parent.logical==='true'&&opened.parent.native,'actual parent trigger opened native parent before navigation');
 const selection=key(heldParentTrigger,id===12?'End':'Home','actual-parent-selection');await Promise.resolve();api().record('selection-microtask',{event:selection.event},true);await fixedFrames(12,'actual-parent-selection',selection.event);
 record('fixture-ready',{case:id,setup:'actual parent click assigns child cache, then mode-correct original Home/End; no desired marker/focus wait'});
 return{before,open,opened,selection,selected:state(),journal:api().rows.filter(row=>row.context===context&&row.seq>cursor)};
}
async function operation(id,kind,value,index){
 const label=context+'-operation-'+index,cursor=api().rows.at(-1)?.seq??0,before=state();let original;
 if(kind==='key'){const target=get('fixture').dataset.parentRoving==='true'?api().deepActive():heldParentTrigger;original=key(target,value,label);}
 else if(kind==='click')original=click(heldLinkA,label);
 else original=command('child-a-open');
 api().record('operation-return',{label,kind,original},true);await Promise.resolve();api().record('operation-return-microtask',{label,event:original.event??null},true);
 await fixedFrames(12,label,original.event??null);api().record('fixed-frame-drain-end',{label,event:original.event??null},true);
 const after=state(),journal=api().rows.filter(row=>row.context===context&&row.seq>cursor);assert(JSON.stringify(identities(before))===heldIdentity&&JSON.stringify(identities(after))===heldIdentity,'same actual roots/member/control objects retained from fresh setup');
 return{label,kind,value,before,original,state:after,journal};
}
try{
 await wait(()=>api(),'passive linked submenu journal');
 for(const id of cases){const setup=await fresh(id),operations=[];let index=0;
  if(id===10)operations.push(await operation(id,'click',null,++index));else if(id===11)operations.push(await operation(id,'public',null,++index));
  else operations.push(await operation(id,'key','ArrowRight',++index));
  if(![6,7].includes(id))operations.push(await operation(id,'key','ArrowDown',++index));
  operations.push(await operation(id,'key','ArrowLeft',++index));
  observations.push({case:id,label:context,apiDiagnostic:id===11,setup,operations,state:state()});record('context-complete',{case:id});
 }
 assert(observations.length===11,'eleven approved public linked histories; source callback L08 omitted');
 return{ok:true,observations,assertions,driver,limits:['raw outcomes until actual source-derived comparison; setup/identity assertions are not parity','each original input has12 fixed RAF/microtasks, no desired-focus wait or corrective input','source child property.show versus Rust childHandle.open is explicit L11 API differential','actual child uses public trigger string ID, no private trigger metadata injection','actual Application/Menu and Category independent popup recorded; no alias to external child','source shadow/native generated control and root/controller capture adapters remain explicit','synthetic composed cancelable keys/clicks are not physical OS/AT/Brand/platform proof','phase log records actual native transport events; pure cue/work query cost unknown','retain query rows remain in array without DOM publication to avoid proven diagnostic feedback','parent/child current modes stay raw, including parent roving/child AD caseL12']};
}catch(error){return{ok:false,error:String(error),context,state:state(),observations,assertions,driver,journal:api()?.rows??[]};}
"#;
