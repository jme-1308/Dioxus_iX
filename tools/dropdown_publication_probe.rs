//! Excluded publication diagnostic. Rust owns the actual mixed-item fixture;
//! native scripts only deliver inputs and observe transport, DOM and RAF facts.
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
                .with_title("iX item publication")
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
    eprintln!("Run the desktop or web item publication probe");
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
            futures_timer::Delay::new(std::time::Duration::from_secs(90)).await;
            eprintln!("Item publication native timeout");
            std::process::exit(1);
        });
    });
    rsx! {IxTheme {scheme:ColorScheme::Dark,
        style {{STYLES}}
        main {id:"publication-probe",style:"padding:20px;min-height:900px",
            onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=dom_query::eval(VERIFY);let _=query.send(());
                    match query.join::<Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dropdown publication native observations: {value}");window.close();}
                        Ok(value)=>{eprintln!("Dropdown publication native failure JSON: {value}");std::process::exit(1);}
                        Err(error)=>{eprintln!("Dropdown publication native transport failure: {error:?}");std::process::exit(1);}
                    }
                });}
            },
            h1 {"Item publication checkpoints"}
            p {"Seven independent histories. Each input has twelve fixed native frame observations."}
            div {style:"display:flex;gap:6px;flex-wrap:wrap",
                for next in 1..=7 {button {id:format!("publication-fresh-{next}"),onclick:move |_|{selected.set(next);epoch+=1;},"Fresh P{next}"}}
            }
            input {id:"publication-mode",r#type:"hidden",oninput:move |event|{selected.set(event.value().parse::<usize>().unwrap_or(1).clamp(1,7));epoch+=1;}}
            button {id:"publication-outside","Outside keyboard target"}
            for current in [epoch()] {Fixture {key:"{current}",epoch:current,case_id:selected()}}
            NativeJournal {}
            pre {id:"publication-native-log","aria-label":"Native publication observations",style:"max-height:180px;overflow:auto;font-size:12px;white-space:pre-wrap"}
        }
    }}
}

#[component]
fn Fixture(epoch: usize, case_id: usize) -> Element {
    let dropdown = use_dropdown();
    let rows = use_signal(Vec::<Value>::new);
    rsx! {section {id:"publication-fixture","data-epoch":epoch.to_string(),"data-case":case_id.to_string(),
        div {id:"publication-owner",
            DropdownTrigger {dropdown,"Open mixed menu"}
            Application {force_breakpoint:ApplicationBreakpoint::Large,class:"publication-application",
                menu:rsx! {Menu {label:"Publication Menu context",start_expanded:false,pinned:false,
                    Dropdown {controller:dropdown,label:"Publication mixed members",class:"publication-dropdown",navigation_mode:if case_id==7 {DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},close_behavior:DropdownCloseBehavior::None,enable_top_layer:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
                        onbeforechange:move |event:DropdownChangeEvent|record(rows,"request",json!({"open":event.open,"previous":event.previous})),
                        onchange:move |open|record(rows,"completion",json!({"open":open})),
                        div {id:"publication-members",
                            Member {role:"dropdown-a",rows}
                            Member {role:"select-b",rows}
                            Member {role:"menu-c",rows}
                            Member {role:"native-d",rows}
                            Member {role:"native-control",rows}
                            Member {role:"dropdown-z",rows}
                            Member {role:"category-e",rows}
                        }
                    }
                }},
                div {}
            }
        }
        input {id:"publication-rust-json",r#type:"hidden",value:serde_json::to_string(&rows()).unwrap_or_default()}
    }}
}

#[component]
fn Member(role: &'static str, rows: Signal<Vec<Value>>) -> Element {
    rsx! {div {"data-publication-role":role,class:"publication-member",
        onclick:move |_|record(rows,"member-click",json!({"role":role})),
        match role {
            "dropdown-a"=>rsx!{DropdownItem {label:"Dropdown A",onitemclick:move |_|record(rows,"item-click",json!({"role":role}))}},
            "select-b"=>rsx!{SelectItem {value:"select-b",label:"Select B",onclick:move |_|record(rows,"select-click",json!({"role":role})),onitemclick:move |value|record(rows,"select-item-click",json!({"role":role,"value":value}))}},
            "menu-c"=>rsx!{MenuItem {label:"Menu C",close_on_select:false,onclick:move |_|record(rows,"menu-click",json!({"role":role}))}},
            "native-d"=>rsx!{button {id:"publication-native-d",r#type:"button","data-ix-roving-item":"",onclick:move |_|record(rows,"native-click",json!({"role":role})),"Marked native D"}},
            "native-control"=>rsx!{button {id:"publication-unmarked",r#type:"button",onclick:move |_|record(rows,"unmarked-click",json!({"role":role})),"Unmarked native control"}},
            "dropdown-z"=>rsx!{DropdownItem {label:"Dropdown Z",onitemclick:move |_|record(rows,"item-click",json!({"role":role}))}},
            "category-e"=>rsx!{MenuCategory {label:"Category E",
                MenuItem {id:"publication-category-child-0","data-publication-child":"category-child-0",label:"Category child one",close_on_select:false}
                MenuItem {id:"publication-category-child-1","data-publication-child":"category-child-1",label:"Category child two",close_on_select:false}
            }},
            _=>VNode::empty()
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
#publication-owner .publication-application{height:560px}
#publication-owner nav.ix-menu{position:relative;inset:auto;height:540px;width:52px;overflow:visible}
#publication-owner .publication-dropdown>.ix-dropdown-root-menu{max-height:440px;width:max-content}
#publication-members{width:320px;max-height:360px;overflow:auto}
.publication-member{min-height:36px}
.publication-member>[data-ix-item-kind],.publication-member>button{box-sizing:border-box;min-height:36px;width:100%}
"#;

const OBSERVE: &str = r#"
const probe=document.getElementById('publication-probe');if(!probe)return;
const rows=[],objects=new WeakMap(),events=new WeakMap(),phaseReports=new WeakMap(),abort=new AbortController();let sequence=0,objectSequence=0,eventSequence=0,context='bootstrap',alive=true;
const object=node=>{if(!node)return null;if(!objects.has(node))objects.set(node,++objectSequence);return objects.get(node);};
const eventId=event=>{if(!events.has(event))events.set(event,++eventSequence);return events.get(event);};
const node=el=>el instanceof Element?{object:object(el),id:el.id,tag:el.tagName,role:el.closest('[data-publication-child]')?.dataset.publicationChild??el.closest('[data-publication-role]')?.dataset.publicationRole,connected:el.isConnected}:null;
const root=()=>document.querySelector('#publication-owner .publication-dropdown[data-ix-overlay-root]');
const trigger=()=>document.querySelector('#publication-owner>[data-ix-overlay-trigger]');
const attributes=el=>el?{node:node(el),active:el.getAttribute('data-active'),focused:el.classList.contains('ix-focused'),tabindex:el.getAttribute('tabindex'),aria:el.getAttribute('aria-activedescendant'),ariaSelected:el.getAttribute('aria-selected'),checked:el.getAttribute('checked'),selected:el.getAttribute('selected'),selectedProperty:typeof el.selected==='boolean'?el.selected:null,checkedProperty:typeof el.checked==='boolean'?el.checked:null,class:el.getAttribute('class'),hidden:el.hasAttribute('hidden'),kind:el.dataset.ixItemKind,key:el.dataset.ixItemKey}:null;
const snapshot=()=>{const r=root(),host=r?.querySelector(':scope>dialog');let native=false;try{native=!!host?.matches(':popover-open');}catch{}return{epoch:document.getElementById('publication-fixture')?.dataset.epoch,case:document.getElementById('publication-fixture')?.dataset.case,root:node(r),owner:r?.dataset.ixOverlayOwner,generation:r?.dataset.ixOverlayGeneration,logical:r?.dataset.ixOverlayOpen,native,mode:r?.dataset.ixOverlayMode,published:r?.dataset.ixOverlayActiveItem,trigger:attributes(trigger()),focus:node(document.activeElement),members:[...document.querySelectorAll('#publication-members>.publication-member')].map(wrapper=>({role:wrapper.dataset.publicationRole,host:attributes(wrapper.firstElementChild),controls:[...wrapper.firstElementChild?.querySelectorAll('[data-ix-item-control],.ix-select-item-option')??[]].map(attributes)})),rust:JSON.parse(document.getElementById('publication-rust-json')?.value||'[]')};};
const publish=row=>{const el=document.getElementById('publication-native-log');if(el)el.textContent=JSON.stringify({context,last:row});};
const record=(kind,details={},withState=false)=>{const row={seq:++sequence,context,kind,time:performance.now(),...details,...(withState?{state:snapshot()}:{})};rows.push(row);if(rows.length>2400)rows.shift();publish(row);return row;};
const compactReport=report=>({owner:report.owner,sequence:report.sequence,phase:report.item_focus_phase,job:report.item_focus_sequence,root:report.item_focus_root,generation:report.item_focus_generation,target:report.item_focus_target,capture:report.item_focus_capture,currentTarget:node(document.getElementById(report.item_focus_target??''))});
const phaseEvent=event=>{if(phaseReports.has(event))return phaseReports.get(event);let report=null;if(event.target.matches?.('input[data-ix-overlay-transport]')){try{const value=JSON.parse(event.target.value);if(value.item_focus_phase)report=compactReport(value);}catch{}}phaseReports.set(event,report);return report;};
for(const type of ['keydown','keyup','click','focusin','focusout']){
 document.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('document-capture',{event:eventId(event),type,key:event.key,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented},true);},{capture:true,passive:true,signal:abort.signal});
 window.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('window-bubble',{event:eventId(event),type,key:event.key,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented},true);},{passive:true,signal:abort.signal});
}
document.addEventListener('change',event=>{const report=phaseEvent(event);if(!report)return;const id=eventId(event),observedContext=context;record('item-focus-change-capture',{event:id,observedContext,report,defaultPrevented:event.defaultPrevented},true);queueMicrotask(()=>{if(alive)record('item-focus-queued-capture-microtask',{event:id,observedContext,report,defaultPrevented:event.defaultPrevented},true);});setTimeout(()=>{if(alive)record('item-focus-post-change-task',{event:id,observedContext,report,defaultPrevented:event.defaultPrevented},true);},0);},{capture:true,passive:true,signal:abort.signal});
window.addEventListener('change',event=>{const report=phaseEvent(event);if(report)record('item-focus-change-window',{event:eventId(event),report,defaultPrevented:event.defaultPrevented},true);},{passive:true,signal:abort.signal});
const observedAttributes=['data-active','class','data-ix-overlay-active-item','aria-activedescendant','aria-selected','selected','checked','tabindex'];
const observer=new MutationObserver(records=>{const changes=records.filter(change=>change.target.closest?.('#publication-owner')&&(change.target.matches('[data-ix-overlay-root],[data-ix-overlay-trigger],[data-ix-item-kind],[data-ix-item-control],.ix-select-item-option')||change.target.closest('[data-ix-item-kind]'))).map(change=>({node:node(change.target),attribute:change.attributeName,oldValue:change.oldValue,currentValue:change.target.getAttribute(change.attributeName)}));if(changes.length)record('marker-mutation-delivery',{changes},true);});
observer.observe(probe,{attributes:true,subtree:true,attributeOldValue:true,attributeFilter:observedAttributes});
window.__ixPublicationProbe={rows,snapshot,node,eventId,record,setContext:value=>{context=value;record('context',{},true);}};record('journal-ready');
try{await dioxus.recv();}finally{alive=false;abort.abort();observer.disconnect();delete window.__ixPublicationProbe;}
"#;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
await dioxus.recv();
const observations=[],assertions=[],driver=[];let context='bootstrap';
const get=id=>document.getElementById('publication-'+id),api=()=>window.__ixPublicationProbe,sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const root=()=>document.querySelector('#publication-owner .publication-dropdown[data-ix-overlay-root]'),trigger=()=>document.querySelector('#publication-owner>[data-ix-overlay-trigger]');
const state=()=>api()?.snapshot(),record=(kind,details={})=>{const row={context,kind,time:performance.now(),...details,state:state()};driver.push(row);return row;};
const assert=(ok,label)=>{assertions.push({context,label,ok:!!ok});if(!ok)throw Error(label+' '+JSON.stringify(state()));};
const wait=async(fn,label,ms=6000)=>{const end=performance.now()+ms;while(performance.now()<end){if(fn())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify(state()));};
const frame=()=>new Promise(resolve=>requestAnimationFrame(resolve));
async function fixedFrames(count,label,event=null){for(let index=1;index<=count;index++){const timestamp=await frame();api().record('fixed-observer-frame',{label,event,index,timestamp},true);await Promise.resolve();api().record('fixed-observer-frame-microtask',{label,event,index,timestamp},true);}}
const key=(target,value,label)=>{assert(target?.isConnected,'connected actual key target');const event=new KeyboardEvent('keydown',{key:value,bubbles:true,composed:true,cancelable:true}),id=api().eventId(event);api().record('original-key-before',{label,event:id,key:value,target:api().node(target)},true);target.dispatchEvent(event);api().record('original-key-return',{label,event:id,key:value,target:api().node(target),defaultPrevented:event.defaultPrevented},true);return{event:id,key:value,target:api().node(target),defaultPrevented:event.defaultPrevented};};
const input=()=>{if(root()?.dataset.ixOverlayMode!=='roving-tabindex')return trigger();const active=document.activeElement;if(root()?.contains(active))return active;return document.querySelector('#publication-members [data-ix-item-kind][tabindex="0"],#publication-members [data-ix-roving-item][tabindex="0"]')??trigger();};
async function fresh(id){context='P'+id;const old=get('fixture');get('mode').value=String(id);get('mode').dispatchEvent(new Event('input',{bubbles:true}));await wait(()=>get('fixture')!==old&&get('fixture')?.dataset.case===String(id)&&root()?.dataset.ixOverlayGeneration&&root()?.dataset.ixOverlayOwner&&api(),'fresh Rust publication fixture');api().setContext(context);await fixedFrames(4,'fixture-bind');get('outside').focus();key(get('outside'),'Tab','outside-keyboard-setup');trigger().focus();trigger().click();await wait(()=>state().logical==='true'&&state().native,'actual registered trigger opens native Root');await fixedFrames(6,'actual-open-setup');if(id===7){const initialized=key(trigger(),'ArrowDown','actual-roving-setup');await Promise.resolve();api().record('key-return-microtask',{event:initialized.event,label:'actual-roving-setup'},true);await fixedFrames(12,'actual-roving-setup',initialized.event);}record('fixture-ready',{setupAdapter:'source public show property versus Rust registered-trigger opening; own setup requests/focus excluded'});}
async function operation(value,index){const label=context+'-key'+index,cursor=api().rows.at(-1)?.seq??0,before=state(),original=key(input(),value,label);await Promise.resolve();api().record('key-return-microtask',{label,event:original.event},true);await fixedFrames(12,label,original.event);api().record('fixed-frame-drain-end',{label,event:original.event},true);return{label,cursor,before,original,state:state(),journal:api().rows.filter(row=>row.context===context&&row.seq>cursor)};}
try{
 await wait(()=>api(),'passive native publication journal');
 const matrix=[['Home'],['Home','ArrowDown','ArrowDown','ArrowDown'],['Home','ArrowDown','ArrowDown','ArrowDown','End'],['Home'],['Home','ArrowDown'],['Home','ArrowDown','ArrowDown'],['Home','ArrowDown']];
 for(let index=0;index<matrix.length;index++){const id=index+1;await fresh(id);const cursor=api().rows.at(-1)?.seq??0,operations=[];for(let keyIndex=0;keyIndex<matrix[index].length;keyIndex++)operations.push(await operation(matrix[index][keyIndex],keyIndex+1));observations.push({case:id,label:context,operations,state:state(),journal:api().rows.filter(row=>row.context===context&&row.seq>cursor)});record('context-complete',{case:id});}
 assert(observations.length===7,'seven independent fixed-frame publication histories');return{ok:true,observations,assertions,driver,limits:['supplemental fixed12 observers; unchanged old12/65key artifact remains separate','no expected-marker waits or later activation input','item-focus phase cancellation is an observed authorization proxy, not an exact private Rust visual-write timestamp','capture microtask can precede later listeners; final task and page observations stay distinct','MutationObserver reports delivery after mutation, with current values possibly coalesced','opaque captured tuple token versus current target actualobject are different facts','source shadow versus native declarative markers/inner rendering remain different layers','no first-frame/runtime/renderer/physical/AT/platform parity claim']};
}catch(error){return{ok:false,error:String(error),context,state:state(),observations,assertions,driver,journal:api()?.rows??[]};}
"#;
