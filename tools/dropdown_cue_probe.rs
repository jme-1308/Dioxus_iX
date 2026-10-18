//! Excluded supplemental cue diagnostic. Rust owns the actual consumer props;
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
        .with_custom_index(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>iX item cue</title></head><body><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string(),
        )
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX item cue")
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
    eprintln!("Run the desktop or web item cue probe");
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
            eprintln!("Item cue native timeout");
            std::process::exit(1);
        });
    });
    rsx! {IxTheme {scheme:ColorScheme::Dark,
        style {{STYLES}}
        main {id:"cue-probe",style:"padding:20px;min-height:900px",
            onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=dom_query::eval(VERIFY);let _=query.send(());
                    match query.join::<Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dropdown cue native observations: {value}");window.close();}
                        Ok(value)=>{eprintln!("Dropdown cue native failure JSON: {value}");std::process::exit(1);}
                        Err(error)=>{eprintln!("Dropdown cue native transport failure: {error:?}");std::process::exit(1);}
                    }
                });}
            },
            h1 {"Item cue checkpoints"}
            p {"Supplemental consumer race, adversarial cue perturbation, cleanup and nested focus redirect histories. Twelve fixed native frame observations; no expected-cue waits."}
            div {style:"display:flex;gap:6px;flex-wrap:wrap",
                for next in 1..=5 {button {id:format!("cue-fresh-{next}"),onclick:move |_|{selected.set(next);epoch+=1;},"Fresh C{next}"}}
            }
            input {id:"cue-mode",r#type:"hidden",oninput:move |event|{selected.set(event.value().parse::<usize>().unwrap_or(1).clamp(1,5));epoch+=1;}}
            button {id:"cue-outside","Outside keyboard target"}
            for current in [epoch()] {Fixture {key:"{current}",epoch:current,case_id:selected()}}
            NativeJournal {}
            pre {id:"cue-native-log","aria-label":"Native cue observations",style:"max-height:180px;overflow:auto;font-size:12px;white-space:pre-wrap"}
        }
    }}
}

#[component]
fn Fixture(epoch: usize, case_id: usize) -> Element {
    let dropdown = use_dropdown();
    let rows = use_signal(Vec::<Value>::new);
    let mut consumer_revision = use_signal(|| 0_usize);
    let mut roving = use_signal(|| false);
    rsx! {section {id:"cue-fixture","data-epoch":epoch.to_string(),"data-case":case_id.to_string(),"data-consumer-revision":consumer_revision().to_string(),
        div {id:"cue-owner",
            DropdownTrigger {dropdown,"Open mixed menu"}
            Application {force_breakpoint:ApplicationBreakpoint::Large,class:"cue-application",
                menu:rsx! {Menu {label:"Cue Menu context",start_expanded:false,pinned:false,
                    Dropdown {controller:dropdown,label:"Cue mixed members",class:"cue-dropdown",navigation_mode:if roving() {DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},close_behavior:DropdownCloseBehavior::None,enable_top_layer:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
                        onbeforechange:move |event:DropdownChangeEvent|record(rows,"request",json!({"open":event.open,"previous":event.previous})),
                        onchange:move |open|record(rows,"completion",json!({"open":open})),
                        div {id:"cue-members",
                            Member {role:"dropdown-a",rows}
                            Member {role:"select-b",rows,consumer_revision:consumer_revision()}
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
        input {id:"cue-command",r#type:"hidden",oninput:move |event| {
            let command=serde_json::from_str::<Value>(&event.value()).unwrap_or(Value::Null);
            record(rows,"consumer-command",command.clone());
            match command["kind"].as_str() {
                Some("consumer-prop")=>consumer_revision+=1,
                Some("public-hide")=>dropdown.close(false),
                Some("mode-roving")=>roving.set(true),
                _=>{}
            }
        }}
        input {id:"cue-rust-json",r#type:"hidden",value:serde_json::to_string(&rows()).unwrap_or_default()}
    }}
}

#[component]
fn Member(
    role: &'static str,
    rows: Signal<Vec<Value>>,
    #[props(default)] consumer_revision: usize,
) -> Element {
    use_effect(use_reactive(
        (&role, &consumer_revision),
        move |(role, revision)| {
            if role == "select-b" {
                record(
                    rows,
                    "consumer-render-effect",
                    json!({"role":role,"revision":revision}),
                );
            }
        },
    ));
    rsx! {div {"data-cue-role":role,class:"cue-member",
        onclick:move |_|record(rows,"member-click",json!({"role":role})),
        match role {
            "dropdown-a"=>rsx!{DropdownItem {label:"Dropdown A",onitemclick:move |_|record(rows,"item-click",json!({"role":role}))}},
            "select-b"=>rsx!{SelectItem {value:"select-b",label:format!("Select B consumer {consumer_revision}"),"data-consumer-revision":consumer_revision.to_string(),onclick:move |_|record(rows,"select-click",json!({"role":role})),onitemclick:move |value|record(rows,"select-item-click",json!({"role":role,"value":value}))}},
            "menu-c"=>rsx!{MenuItem {label:"Menu C",close_on_select:false,onclick:move |_|record(rows,"menu-click",json!({"role":role}))}},
            "native-d"=>rsx!{button {id:"cue-native-d",r#type:"button","data-ix-roving-item":"",onclick:move |_|record(rows,"native-click",json!({"role":role})),"Marked native D"}},
            "native-control"=>rsx!{button {id:"cue-unmarked",r#type:"button",onclick:move |_|record(rows,"unmarked-click",json!({"role":role})),"Unmarked native control"}},
            "dropdown-z"=>rsx!{DropdownItem {label:"Dropdown Z",onitemclick:move |_|record(rows,"item-click",json!({"role":role}))}},
            "category-e"=>rsx!{MenuCategory {label:"Category E",
                MenuItem {id:"cue-category-child-0","data-cue-child":"category-child-0",label:"Category child one",close_on_select:false}
                MenuItem {id:"cue-category-child-1","data-cue-child":"category-child-1",label:"Category child two",close_on_select:false}
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
        let mut eval = dioxus::document::eval(OBSERVE);
        *transport.borrow_mut() = Some(eval);
        spawn(async move {
            while let Ok(value) = eval.recv::<Value>().await {
                eprintln!("Cue diagnostic progress: {value}");
            }
        });
    });
    VNode::empty()
}

const STYLES: &str = r#"
#cue-owner .cue-application{height:560px}
#cue-owner nav.ix-menu{position:relative;inset:auto;height:540px;width:52px;overflow:visible}
#cue-owner .cue-dropdown>.ix-dropdown-root-menu{max-height:440px;width:max-content}
#cue-members{width:320px;max-height:360px;overflow:auto}
.cue-member{min-height:36px}
.cue-member>[data-ix-item-kind],.cue-member>button{box-sizing:border-box;min-height:36px;width:100%}
"#;

const OBSERVE: &str = r#"
const probe=document.getElementById('cue-probe');if(!probe)return;
const rows=[],objects=new WeakMap(),events=new WeakMap(),phaseReports=new WeakMap(),abort=new AbortController();let sequence=0,contextRows=0,objectSequence=0,eventSequence=0,context='bootstrap',alive=true,race=null,detach=null,detachedPair=null;
const object=node=>{if(!node)return null;if(!objects.has(node))objects.set(node,++objectSequence);return objects.get(node);};
const eventId=event=>{if(!events.has(event))events.set(event,++eventSequence);return events.get(event);};
const node=el=>el instanceof Element?{object:object(el),id:el.id,tag:el.tagName,role:el.closest('[data-cue-child]')?.dataset.cueChild??el.closest('[data-cue-role]')?.dataset.cueRole,connected:el.isConnected}:null;
const root=()=>document.querySelector('#cue-owner .cue-dropdown[data-ix-overlay-root]');
const trigger=()=>document.querySelector('#cue-owner>[data-ix-overlay-trigger]');
const attributes=el=>el?{node:node(el),active:el.getAttribute('data-active'),focused:el.classList.contains('ix-focused'),tabindex:el.getAttribute('tabindex'),aria:el.getAttribute('aria-activedescendant'),ariaSelected:el.getAttribute('aria-selected'),checked:el.getAttribute('checked'),selected:el.getAttribute('selected'),selectedProperty:typeof el.selected==='boolean'?el.selected:null,checkedProperty:typeof el.checked==='boolean'?el.checked:null,consumerRevision:el.getAttribute('data-consumer-revision'),label:el.getAttribute('label'),text:el.textContent,class:el.getAttribute('class'),hidden:el.hasAttribute('hidden'),kind:el.dataset.ixItemKind,key:el.dataset.ixItemKey}:null;
const snapshot=()=>{const r=root(),host=r?.querySelector(':scope>dialog');let native=false;try{native=!!host?.matches(':popover-open');}catch{}return{detached:detachedPair?{original:attributes(detachedPair.original),replacement:attributes(detachedPair.replacement)}:null,epoch:document.getElementById('cue-fixture')?.dataset.epoch,case:document.getElementById('cue-fixture')?.dataset.case,consumerRevision:document.getElementById('cue-fixture')?.dataset.consumerRevision,root:node(r),owner:r?.dataset.ixOverlayOwner,generation:r?.dataset.ixOverlayGeneration,logical:r?.dataset.ixOverlayOpen,native,mode:r?.dataset.ixOverlayMode,published:r?.dataset.ixOverlayActiveItem,trigger:attributes(trigger()),focus:node(document.activeElement),members:[...document.querySelectorAll('#cue-members>.cue-member')].map(wrapper=>({role:wrapper.dataset.cueRole,host:attributes(wrapper.firstElementChild),controls:[...wrapper.firstElementChild?.querySelectorAll('[data-ix-item-control],.ix-select-item-option')??[]].map(attributes)})),rust:JSON.parse(document.getElementById('cue-rust-json')?.value||'[]')};};
const publish=row=>{const el=document.getElementById('cue-native-log');if(el)el.textContent=JSON.stringify({context,last:row});};
const record=(kind,details={},withState=false)=>{const row={seq:++sequence,contextRow:++contextRows,context,kind,time:performance.now(),...details,...(withState?{state:snapshot()}:{})};rows.push(row);if(rows.length>7000)rows.shift();if(details.report?.phase!=='retain')publish(row);if(kind==='journal-ready'||kind==='context'||sequence%1000===0||(context==='C5'&&contextRows<=100)||kind.startsWith('pending-native-detach-')){const report=details.report;try{dioxus.send({seq:sequence,contextRow:contextRows,context,kind,time:row.time,report:report?{phase:report.phase,job:report.job,root:report.root,generation:report.generation,target:report.target,capture:report.capture,itemCue:report.itemCue,rootCue:report.rootCue}:null});}catch{}}return row;};
const compactReport=report=>({itemCue:report.item_cue??null,rootCue:report.item_root_cue??null,liveHost:report.item_focus_host??null,owner:report.owner,sequence:report.sequence,phase:report.item_focus_phase,job:report.item_focus_sequence,root:report.item_focus_root,generation:report.item_focus_generation,target:report.item_focus_target,capture:report.item_focus_capture,currentTarget:node(document.getElementById(report.item_focus_target??''))});
const phaseEvent=event=>{if(phaseReports.has(event))return phaseReports.get(event);let report=null;if(event.target.matches?.('input[data-ix-overlay-transport]')){try{const value=JSON.parse(event.target.value);if(value.item_focus_phase||value.item_cue||value.item_root_cue)report=compactReport(value);}catch{}}phaseReports.set(event,report);return report;};
for(const type of ['keydown','keyup','click','focusin','focusout']){
 document.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('document-capture',{event:eventId(event),type,key:event.key,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented},true);},{capture:true,passive:true,signal:abort.signal});
 window.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('window-bubble',{event:eventId(event),type,key:event.key,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented},true);},{passive:true,signal:abort.signal});
}
document.addEventListener('change',event=>{const report=phaseEvent(event);if(!report)return;const id=eventId(event),observedContext=context;record('item-focus-change-capture',{event:id,observedContext,report,defaultPrevented:event.defaultPrevented},true);queueMicrotask(()=>{if(alive)record('item-focus-queued-capture-microtask',{event:id,observedContext,report,defaultPrevented:event.defaultPrevented},true);});setTimeout(()=>{if(alive)record('item-focus-post-change-task',{event:id,observedContext,report,defaultPrevented:event.defaultPrevented},true);},0);},{capture:true,passive:true,signal:abort.signal});
window.addEventListener('change',event=>{const report=phaseEvent(event);if(!report)return;record('item-focus-change-window',{event:eventId(event),report,defaultPrevented:event.defaultPrevented},true);if(detach&&report.phase==='arm'&&report.target===detach.element.id&&document.getElementById('cue-fixture')?.dataset.epoch===detach.epoch){const held=detach;detach=null;const original=held.element,replacement=original.cloneNode(true);detachedPair={original,replacement};record('pending-native-detach-before',{event:eventId(event),report,original:attributes(original),replacement:attributes(replacement)},true);original.parentNode.insertBefore(replacement,original);original.remove();observer.observe(original,{attributes:true,subtree:true,attributeOldValue:true,attributeFilter:observedAttributes});record('pending-native-detach-return',{event:eventId(event),report,original:attributes(original),replacement:attributes(replacement)},true);}if(race&&report.phase==='focus'&&report.target===race.target&&document.getElementById('cue-fixture')?.dataset.epoch===race.epoch){const armed=race;race=null;const input=document.getElementById('cue-command');record('consumer-race-before-command',{event:eventId(event),report,armed},true);input.value=JSON.stringify({kind:'consumer-prop',causeEvent:eventId(event),focusJob:report.job});input.dispatchEvent(new Event('input',{bubbles:true}));record('consumer-race-command-return',{event:eventId(event),report,armed},true);}},{passive:true,signal:abort.signal});
const observedAttributes=['data-active','class','data-ix-overlay-active-item','aria-activedescendant','aria-selected','selected','checked','tabindex','data-consumer-revision','label'];
const observer=new MutationObserver(records=>{const changes=records.filter(change=>(change.target.closest?.('#cue-owner')||detachedPair?.original===change.target||detachedPair?.original.contains(change.target))&&(change.target.matches('[data-ix-overlay-root],[data-ix-overlay-trigger],[data-ix-item-kind],[data-ix-item-control],.ix-select-item-option')||change.target.closest('[data-ix-item-kind]'))).map(change=>({node:node(change.target),attribute:change.attributeName,oldValue:change.oldValue,currentValue:change.target.getAttribute(change.attributeName)}));if(changes.length)record('marker-mutation-delivery',{changes},true);});
observer.observe(probe,{attributes:true,subtree:true,attributeOldValue:true,attributeFilter:observedAttributes});
window.__ixCueProbe={rows,snapshot,node,eventId,record,setContext:value=>{context=value;contextRows=0;race=null;detach=null;detachedPair=null;record('context',{},true);},armRace:target=>{race={target,epoch:document.getElementById('cue-fixture')?.dataset.epoch};record('consumer-race-armed',{race},true);},armDetach:element=>{detach={element,epoch:document.getElementById('cue-fixture')?.dataset.epoch};record('pending-native-detach-armed',{original:attributes(element)},true);}};record('journal-ready');
try{await dioxus.recv();}finally{alive=false;abort.abort();observer.disconnect();delete window.__ixCueProbe;}
"#;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
await dioxus.recv();
const observations=[],assertions=[],driver=[];let context='bootstrap';
const get=id=>document.getElementById('cue-'+id),api=()=>window.__ixCueProbe,sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const root=()=>document.querySelector('#cue-owner .cue-dropdown[data-ix-overlay-root]'),trigger=()=>document.querySelector('#cue-owner>[data-ix-overlay-trigger]');
const state=()=>api()?.snapshot(),record=(kind,details={})=>{const row={context,kind,time:performance.now(),...details,state:state()};driver.push(row);return row;};
const assert=(ok,label)=>{assertions.push({context,label,ok:!!ok});if(!ok)throw Error(label+' '+JSON.stringify(state()));};
const wait=async(fn,label,ms=6000)=>{const end=performance.now()+ms;while(performance.now()<end){if(fn())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify(state()));};
const frame=()=>new Promise(resolve=>requestAnimationFrame(resolve));
async function fixedFrames(count,label,event=null){for(let index=1;index<=count;index++){const timestamp=await frame();api().record('fixed-observer-frame',{label,event,index,timestamp},true);await Promise.resolve();api().record('fixed-observer-frame-microtask',{label,event,index,timestamp},true);}}
const key=(target,value,label)=>{assert(target?.isConnected,'connected actual key target');const event=new KeyboardEvent('keydown',{key:value,bubbles:true,composed:true,cancelable:true}),id=api().eventId(event);api().record('original-key-before',{label,event:id,key:value,target:api().node(target)},true);target.dispatchEvent(event);api().record('original-key-return',{label,event:id,key:value,target:api().node(target),defaultPrevented:event.defaultPrevented},true);return{event:id,key:value,target:api().node(target),defaultPrevented:event.defaultPrevented};};
const input=()=>{if(root()?.dataset.ixOverlayMode!=='roving-tabindex')return trigger();const active=document.activeElement;if(root()?.contains(active))return active;return document.querySelector('#cue-members [data-ix-item-kind][tabindex="0"],#cue-members [data-ix-roving-item][tabindex="0"]')??trigger();};
async function fresh(id){context='C'+id;const old=get('fixture');get('mode').value=String(id);get('mode').dispatchEvent(new Event('input',{bubbles:true}));await wait(()=>get('fixture')!==old&&get('fixture')?.dataset.case===String(id)&&root()?.dataset.ixOverlayGeneration&&root()?.dataset.ixOverlayOwner&&api(),'fresh supplemental Rust fixture');api().setContext(context);await fixedFrames(4,'fixture-bind');get('outside').focus();key(get('outside'),'Tab','outside-keyboard-setup');trigger().focus();trigger().click();await wait(()=>state().logical==='true'&&state().native,'actual registered trigger opens native Root');await fixedFrames(6,'actual-open-setup');record('fixture-ready',{setupAdapter:'registered trigger opens actual native surface; setup excluded'});}
const member=role=>document.querySelector('#cue-members>[data-cue-role="'+role+'"]')?.firstElementChild;
const command=value=>{const el=get('command');api().record('actual-consumer-command-before',{command:value},true);el.value=JSON.stringify(value);el.dispatchEvent(new Event('input',{bubbles:true}));api().record('actual-consumer-command-return',{command:value},true);};
const perturb=held=>{const changes=[];for(const [element,active]of held){assert(element instanceof HTMLElement&&element.isConnected,'actual connected held diagnostic cue object');changes.push({node:api().node(element),before:{active:element.getAttribute('data-active'),class:element.getAttribute('class')},injectedActive:active});element.setAttribute('data-active',String(active));element.classList.toggle('ix-focused',active);}api().record('adversarial-native-cue-perturbation',{classification:'Explicit diagnostic marker writes; not a source input, renderer commit, expected golden or focus policy',changes},true);};

async function operation(value,index){const label=context+'-key'+index,cursor=api().rows.at(-1)?.seq??0,before=state(),original=key(input(),value,label);await Promise.resolve();api().record('key-return-microtask',{label,event:original.event},true);await fixedFrames(12,label,original.event);api().record('fixed-frame-drain-end',{label,event:original.event},true);return{label,cursor,before,original,state:state(),journal:api().rows.filter(row=>row.context===context&&row.seq>cursor)};}
try{
 await wait(()=>api(),'passive native cue journal');
 // C1 changes real Rust consumer props during the actual focus transport.
 await fresh(1);let cursor=api().rows.at(-1)?.seq??0;const c1ops=[await operation('Home',1)],heldSelect=member('select-b');api().armRace(heldSelect.id);c1ops.push(await operation('ArrowDown',2));const c1rows=api().rows.filter(row=>row.context===context&&row.seq>cursor),raceCommand=c1rows.find(row=>row.kind==='consumer-race-command-return'),cueChanges=c1rows.filter(row=>row.kind==='marker-mutation-delivery').flatMap(row=>row.changes.map(change=>({seq:row.seq,...change}))),staleOverwrite=cueChanges.filter(change=>change.node?.object===api().node(heldSelect)?.object&&change.attribute==='data-active'&&change.oldValue==='true'&&change.currentValue==='false');observations.push({case:1,label:'C1 real consumer-prop render race',operations:c1ops,state:state(),sameActualSelect:member('select-b')===heldSelect,raceCommandObserved:!!raceCommand,consumerPropCommitObserved:heldSelect.getAttribute('data-consumer-revision')==='1',raceObserved:staleOverwrite.length>0,staleOverwrite,qualification:'Mutation delivery currentValue may coalesce; if raceObserved false, no stale renderer overwrite was proved',journal:c1rows});
 // C2 deterministically challenges current verdict reconciliation, separately.
 await fresh(2);cursor=api().rows.at(-1)?.seq??0;const c2ops=[await operation('Home',1),await operation('ArrowDown',2)],a=member('dropdown-a'),b=member('select-b'),inner=b.querySelector('.ix-select-item-option'),parent=member('category-e').querySelector('[data-ix-item-control]');const beforePerturb=record('adversarial-before');perturb([[a,true],[b,false],[inner,false],[parent,true]]);await fixedFrames(12,'adversarial-cue-reconciliation');observations.push({case:2,label:'C2 adversarial held-object stale cues',operations:c2ops,beforePerturb,state:state(),heldObjects:[a,b,inner,parent].map(api().node),journal:api().rows.filter(row=>row.context===context&&row.seq>cursor),classification:'Diagnostic corruption only; source policy and actual renderer timing not inferred'});
 // C3 uses actual modality/public commands, then challenges stale resurrection.
 await fresh(3);cursor=api().rows.at(-1)?.seq??0;const c3ops=[await operation('Home',1),await operation('ArrowDown',2)],cleanupSelect=member('select-b'),cleanupInner=cleanupSelect.querySelector('.ix-select-item-option');const pointer=new MouseEvent('mousedown',{bubbles:true,composed:true,cancelable:true});api().record('actual-pointer-modality-before',{event:api().eventId(pointer)},true);get('outside').dispatchEvent(pointer);api().record('actual-pointer-modality-return',{event:api().eventId(pointer),defaultPrevented:pointer.defaultPrevented},true);await fixedFrames(12,'pointer-clear');const pointerState=state();command({kind:'public-hide'});await fixedFrames(12,'public-hide');const hiddenState=state();command({kind:'mode-roving'});await fixedFrames(12,'closed-mode-change');const modeState=state();perturb([[cleanupSelect,true],[cleanupInner,true]]);await fixedFrames(12,'cleanup-stale-cue-reconciliation');observations.push({case:3,label:'C3 pointer hide mode stale resurrection diagnostic',operations:c3ops,pointerState,hiddenState,modeState,state:state(),sameActualSelect:member('select-b')===cleanupSelect,journal:api().rows.filter(row=>row.context===context&&row.seq>cursor),classification:'Actual mousedown + public close(false)/mode props; explicit marker perturbation only after cleanup; no source restoration equivalence claim'});
 // C4 redirects one native Menu focusin; outer attempted-focus publication remains observable.
 await fresh(4);cursor=api().rows.at(-1)?.seq??0;const c4ops=[await operation('Home',1),await operation('ArrowDown',2)],attemptedMenu=member('menu-c'),redirectNative=member('native-d'),redirectRows=[];let redirectArmed=true;const redirect=event=>{if(!redirectArmed||!event.composedPath().includes(attemptedMenu))return;redirectArmed=false;redirectRows.push(api().record('nested-focus-redirect-before',{originalFocusEvent:api().eventId(event),attempted:api().node(attemptedMenu),destination:api().node(redirectNative)},true));redirectNative.focus();redirectRows.push(api().record('nested-focus-redirect-return',{originalFocusEvent:api().eventId(event),attempted:api().node(attemptedMenu),destination:api().node(redirectNative)},true));};document.addEventListener('focusin',redirect,{capture:true});try{c4ops.push(await operation('ArrowDown',3));}finally{document.removeEventListener('focusin',redirect,true);}observations.push({case:4,label:'C4 one-shot Menu focusin redirect to native D',operations:c4ops,redirectObserved:!redirectArmed,redirectRows,state:state(),sameActualMenu:member('menu-c')===attemptedMenu,sameActualNative:member('native-d')===redirectNative,journal:api().rows.filter(row=>row.context===context&&row.seq>cursor),classification:'Public native focus redirect during original focusin; expected attempted-host final visual contract is static source-backed until paired actual source replay; final outcome is observational'});
 // C5 detaches only after native arm captured the original actual host.
 await fresh(5);cursor=api().rows.at(-1)?.seq??0;const detachedOriginal=member('dropdown-a');api().armDetach(detachedOriginal);const c5ops=[await operation('Home',1)],c5rows=api().rows.filter(row=>row.context===context&&row.seq>cursor);observations.push({case:5,label:'C5 admitted actual host detached with same-ID copied replacement',operations:c5ops,detachObserved:c5rows.some(row=>row.kind==='pending-native-detach-return'),originalHeld:api().node(detachedOriginal),replacementCurrent:api().node(member('dropdown-a')),sameId:detachedOriginal.id===member('dropdown-a')?.id,distinctActualObjects:detachedOriginal!==member('dropdown-a'),state:state(),journal:c5rows,classification:'Explicit diagnostic native reparent/replacement; clone made before first admitted original focus/visual phase, no marker injection/privategraph writes, no later input or goldenwait; native currentTarget by ID is replacement and is not the retained captured focus object'});
 assert(observations.length===5,'five supplemental cue histories');return{ok:true,observations,assertions,driver,limits:['old7/18 and old12/65 probes unchanged and separate','C1 real prop race reports actual qualification; stable output alone is not proof of overwritten renderer repair','C2/C3 direct marker writes are explicitly adversarial diagnostics, not source/native-default policy proof','fixed12 observations never wait for expected marker and never add a flushing key','transport capture microtask may precede later listeners; task and DOM mutation state remain separate','after-focus pure query requests remain raw; a request/ACK does not itself prove rendered DOM publication','Select own inner immediate cue is a Rust timing adapter vs source delayed render','no full first-frame/renderer/physical/AT/platform parity claim']};
}catch(error){return{ok:false,error:String(error),context,state:state(),observations,assertions,driver,journal:api()?.rows??[]};}
"#;
