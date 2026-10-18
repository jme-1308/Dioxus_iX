//! Excluded public trigger-ID rebind fixture. Rust owns consumer mutations;
//! native code delivers original inputs, observes objects, and moves one
//! explicitly diagnostic retained host without changing private metadata.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("iX public trigger rebinding")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0)),
                )
                .with_menu(None),
        )
        .launch(Probe);
}
#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Probe);
}
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _fixture = Probe;
    eprintln!("Run the desktop or web public trigger rebind probe");
}

const CASES: [usize; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12];
static NEXT_HOST: AtomicUsize = AtomicUsize::new(1);
#[derive(Clone)]
struct PublicIds {
    a: String,
    b: String,
    resident: String,
}
fn record(mut rows: Signal<Vec<Value>>, kind: &'static str, details: Value) {
    if let Ok(mut values) = rows.try_write() {
        let sequence = values
            .last()
            .and_then(|row| row["seq"].as_u64())
            .unwrap_or(0)
            + 1;
        values.push(json!({"seq":sequence,"kind":kind,"details":details}));
    }
}
fn request(
    rows: Signal<Vec<Value>>,
    name: &'static str,
    handle: DropdownHandle,
    event: DropdownChangeEvent,
) {
    record(
        rows,
        "request",
        json!({"name":name,"open":event.open,"previous":event.previous,
        "veto":event.default_prevented(),"current":handle.is_open()}),
    );
}
fn run_action(
    action: &str,
    mut target: Signal<Option<String>>,
    mut revision: Signal<usize>,
    parents: [DropdownHandle; 2],
    child: DropdownHandle,
    rows: Signal<Vec<Value>>,
    ids: Signal<PublicIds>,
) {
    let [a, b] = parents;
    record(
        rows,
        "command-before",
        json!({"action":action,"trigger":target(),"parentA":a.is_open(),"parentB":b.is_open(),"child":child.is_open()}),
    );
    match action {
        "trigger-a" => target.set(Some(ids().a)),
        "trigger-b" => target.set(Some(ids().b)),
        "trigger-none" => target.set(None),
        "trigger-empty" => target.set(Some(String::new())),
        "replace-a" => revision += 1,
        "parent-a-open" => a.open(),
        "parent-a-close" => a.close(false),
        "parent-b-open" => b.open(),
        "parent-b-close" => b.close(false),
        "child-open" => child.open(),
        "child-close" => child.close(false),
        "move-a" | "restore-a" => {
            let operation = action.to_string();
            spawn(async move {
                let evaluation = dioxus::document::eval(MOVE_HOST);
                let _ = evaluation.send(operation);
                let _ = evaluation.join::<Value>().await;
            });
        }
        _ => {}
    }
    record(
        rows,
        "command-return",
        json!({"action":action,"trigger":target(),"parentA":a.is_open(),"parentB":b.is_open(),"child":child.is_open()}),
    );
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
            futures_timer::Delay::new(std::time::Duration::from_secs(150)).await;
            eprintln!("Rebind probe watchdog timeout");
            std::process::exit(1);
        });
    });
    rsx! {IxTheme{scheme:ColorScheme::Dark,
        style{{STYLE}}
        main{id:"rebind-probe",onmounted:move |_|{
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{let evaluation=dom_query::eval(VERIFY);let _=evaluation.send(());match evaluation.join::<Value>().await{
                Ok(value)if value["ok"]==true=>{println!("Dropdown rebind native observations: {value}");window.close();}
                Ok(value)=>{eprintln!("Dropdown rebind native failure JSON: {value}");std::process::exit(1);}
                Err(error)=>{eprintln!("Dropdown rebind transport failure: {error:?}");std::process::exit(1);}
            }});}
        },
            h1{"Public trigger rebinding"}
            p{"Eleven public-input histories with twelve fixed native frames per operation. Boolean false R11 is source-only."}
            div{class:"rebind-controls",for next in CASES{button{id:format!("rebind-fresh-{next}"),onclick:move |_|{selected.set(next);epoch+=1;},"Fresh R{next:02}"}}}
            input{id:"rebind-mode",r#type:"hidden",oninput:move |event|{let next=event.value().parse::<usize>().unwrap_or(1);selected.set(if CASES.contains(&next){next}else{1});epoch+=1;}}
            button{id:"rebind-outside","Outside keyboard seed"}
            for current in [epoch()]{Fixture{key:"{current}",epoch:current,case_id:selected()}}
            NativeJournal{}
            pre{id:"rebind-native-result","Native journal is retained in memory; run result is published once."}
        }
    }}
}

#[component]
fn Fixture(epoch: usize, case_id: usize) -> Element {
    let a = use_dropdown();
    let b = use_dropdown();
    let child = use_dropdown();
    let resident = use_dropdown();
    let original_child = use_hook(|| child);
    let ids = use_signal(move || PublicIds {
        a: format!("rebind-link-a-{epoch}"),
        b: format!("rebind-link-b-{epoch}"),
        resident: format!("rebind-resident-b-link-{epoch}"),
    });
    let target = use_signal(move || Some(format!("rebind-link-a-{epoch}")));
    let revision = use_signal(|| 0_usize);
    let rows = use_signal(Vec::<Value>::new);
    use_effect(move || {
        record(
            rows,
            "consumer-prop-effect",
            json!({"trigger":target(),"hostRevision":revision()}),
        );
    });
    rsx! {section{id:"rebind-fixture","data-epoch":epoch.to_string(),"data-case":case_id.to_string(),
        "data-child-handle-retained":(child==original_child).to_string(),"data-trigger-value":serde_json::to_string(&target()).unwrap_or_default(),
        div{class:"rebind-controls",
            div{id:"rebind-parent-a-trigger-slot",DropdownTrigger{dropdown:a,"Parent A"}}
            div{id:"rebind-parent-b-trigger-slot",DropdownTrigger{dropdown:b,"Parent B"}}
        }
        div{id:"rebind-owner",
            Parent{name:"a",controller:a,resident:case_id==4,revision:revision(),epoch,rows}
            Parent{name:"b",controller:b,resident:case_id==4,revision:0,epoch,rows}
            if case_id==4{Child{name:"resident",controller:resident,trigger:Some(ids().resident),epoch,rows}}
            Child{name:"child",controller:child,trigger:target(),epoch,rows}
        }
        input{id:"rebind-command",r#type:"hidden",oninput:move |event|run_action(&event.value(),target,revision,[a,b],child,rows,ids)}
        div{class:"rebind-controls",for (action,label) in [
            ("trigger-a","Bind A"),("trigger-b","Bind B"),("trigger-none","Set None"),("trigger-empty","Set empty string"),
            ("replace-a","Replace actual A"),("move-a","Move actual A to B"),("restore-a","Restore actual A"),
            ("child-open","Public child open"),("child-close","Public child close"),
            ("parent-a-close","Public A close"),("parent-b-close","Public B close")]{
            button{id:format!("rebind-control-{action}"),onmousedown:move |event|event.prevent_default(),
                onclick:move |event|{event.stop_propagation();
                    #[cfg(feature="web")]
                    if let Some(native)=event.data().downcast::<web_sys::MouseEvent>(){native.stop_propagation();}
                    run_action(action,target,revision,[a,b],child,rows,ids);
                },"{label}"}
        }}
        input{id:"rebind-rust-json",r#type:"hidden",value:serde_json::to_string(&rows()).unwrap_or_default()}
    }}
}

#[component]
fn Parent(
    name: &'static str,
    controller: DropdownHandle,
    resident: bool,
    revision: usize,
    epoch: usize,
    rows: Signal<Vec<Value>>,
) -> Element {
    rsx! {Dropdown{controller,label:format!("Parent {name}"),class:format!("rebind-parent-{name}"),
        navigation_mode:DropdownNavigationMode::ActiveDescendant,close_behavior:DropdownCloseBehavior::None,
        enable_top_layer:true,disable_focus_trap:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
        onbeforechange:move |event|request(rows,if name=="a"{"parent-a"}else{"parent-b"},controller,event),
        onchange:move |open|record(rows,"completion",json!({"name":format!("parent-{name}"),"open":open,"current":controller.is_open()})),
        div{id:format!("rebind-parent-{name}-content"),"data-rebind-role":format!("parent-{name}-content"),class:"rebind-parent-content",
            div{id:format!("rebind-slot-{name}"),
                for current in [revision]{Host{key:"{name}-{current}",name,epoch,rows}}
                if name=="b"&&resident{DropdownItem{id:format!("rebind-resident-b-link-{epoch}"),"data-rebind-role":"resident-b-link",style:ROW,label:"Resident B link"}}
                DropdownItem{id:format!("rebind-leaf-{name}-{epoch}"),"data-rebind-role":format!("leaf-{name}"),style:ROW,label:format!("Leaf {name}"),
                    onitemclick:move |_|record(rows,"item-click",json!({"name":format!("leaf-{name}")}))}
            }
        }
    }}
}
#[component]
fn Host(name: &'static str, epoch: usize, rows: Signal<Vec<Value>>) -> Element {
    let instance = use_hook(|| NEXT_HOST.fetch_add(1, Ordering::Relaxed));
    rsx! {DropdownItem{id:format!("rebind-link-{name}-{epoch}"),"data-rebind-role":format!("link-{name}"),"data-host-instance":instance.to_string(),style:ROW,
    label:format!("Link {name}"),onclick:move |_|record(rows,"host-click",json!({"name":format!("link-{name}"),"instance":instance})),
    onitemclick:move |_|record(rows,"item-click",json!({"name":format!("link-{name}"),"instance":instance}))}}
}
#[component]
fn Child(
    name: &'static str,
    controller: DropdownHandle,
    trigger: Option<String>,
    epoch: usize,
    rows: Signal<Vec<Value>>,
) -> Element {
    rsx! {Dropdown{controller,trigger,label:format!("Child {name}"),class:format!("rebind-child-{name}"),
        navigation_mode:DropdownNavigationMode::ActiveDescendant,close_behavior:DropdownCloseBehavior::None,
        enable_top_layer:true,disable_focus_trap:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
        onbeforechange:move |event|request(rows,name,controller,event),
        onchange:move |open|record(rows,"completion",json!({"name":name,"open":open,"current":controller.is_open()})),
        div{id:format!("rebind-{name}-content"),"data-rebind-role":format!("{name}-content"),class:"rebind-child-content",
            for role in if name=="resident"{vec!["resident-first","resident-last"]}else{vec!["child-first","child-middle","child-last"]}{
                DropdownItem{id:format!("rebind-{role}-{epoch}"),"data-rebind-role":role,style:ROW,label:role,
                    onitemclick:move |_|record(rows,"item-click",json!({"name":role}))}
            }
        }
    }}
}

#[component]
fn NativeJournal() -> Element {
    let running = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let dropped = running.clone();
    use_drop(move || {
        if let Some(task) = dropped.borrow_mut().take() {
            let _ = task.send("finish");
        }
    });
    use_effect(move || {
        *running.borrow_mut() = Some(dioxus::document::eval(OBSERVE));
    });
    VNode::empty()
}
const ROW: &str = "display:block;box-sizing:border-box;min-height:36px;margin:0";
const STYLE: &str = r#"
#rebind-probe{padding:20px;min-height:920px;font-family:system-ui,sans-serif}
.rebind-controls{display:flex;gap:6px;flex-wrap:wrap;margin:8px 0}
#rebind-owner{min-height:500px}.rebind-parent-content{width:280px;max-height:240px;overflow:auto}
.rebind-child-content{width:240px;max-height:160px;overflow:auto}
#rebind-owner dialog{padding:0}#rebind-native-result{max-height:160px;overflow:auto;font-size:12px;white-space:pre-wrap}
"#;
const MOVE_HOST: &str = r#"
const operation=await dioxus.recv();
if(operation==='move-a'){
  const element=document.querySelector('#rebind-owner [data-rebind-role="link-a"]'),before=document.querySelector('#rebind-owner [data-rebind-role="link-b"]');
  if(element&&before){window.__ixRebindMoved={element,parent:element.parentNode,next:element.nextSibling};before.parentNode.insertBefore(element,before);}
}else if(operation==='restore-a'){
  const held=window.__ixRebindMoved;if(held?.element&&held.parent){held.parent.insertBefore(held.element,held.next?.parentNode===held.parent?held.next:null);delete window.__ixRebindMoved;}
}
return {operation};
"#;

const OBSERVE: &str = r#"
if(!document.getElementById('rebind-probe'))return;
const abort=new AbortController(),rows=[],errors=[],objects=new WeakMap(),events=new WeakMap();let serial=0,objectSerial=0,eventSerial=0,context='bootstrap',heldA=null;
const object=element=>{if(!element)return null;if(!objects.has(element))objects.set(element,++objectSerial);return objects.get(element);};
const eventId=event=>{if(!events.has(event))events.set(event,++eventSerial);return events.get(event);};
const root=name=>document.querySelector('#rebind-owner .rebind-'+(name.startsWith('parent-')?name:'child-'+name)+'[data-ix-overlay-root]');
const parentTrigger=name=>document.querySelector('#rebind-'+name+'-trigger-slot>[data-ix-overlay-trigger]');
const directRole=element=>element?.getAttribute?.('data-rebind-role')??(['parent-a','parent-b'].find(name=>element===parentTrigger(name))?.concat('-trigger'))??null;
const node=element=>element instanceof Element?{object:object(element),id:element.id,tag:element.tagName,role:directRole(element),connected:element.isConnected}:null;
const active=()=>{let element=document.activeElement;while(element?.shadowRoot?.activeElement)element=element.shadowRoot.activeElement;return element;};
const nativeOpen=element=>{try{return !!element?.matches(':popover-open');}catch{return false;}};
const attrs=element=>{if(!(element instanceof Element))return null;const r=element.getBoundingClientRect(),s=getComputedStyle(element);return{node:node(element),attrs:Object.fromEntries([...element.attributes].map(a=>[a.name,a.value])),focused:element.classList.contains('ix-focused'),active:element.getAttribute('data-active'),submenuActive:element.classList.contains('ix-dropdown-submenu-trigger-active'),tabindex:element.getAttribute('tabindex'),parent:node(element.parentElement),hostAncestors:(()=>{const out=[];for(let current=element.parentElement;current;current=current.parentElement)if(current.hasAttribute('data-ix-overlay-host'))out.push(current.getAttribute('data-ix-overlay-host'));return out;})(),rect:{x:r.x,y:r.y,width:r.width,height:r.height},display:s.display,visibility:s.visibility,scrollTop:element.scrollTop,scrollLeft:element.scrollLeft,scrollHeight:element.scrollHeight,clientHeight:element.clientHeight};};
const surface=name=>{const element=root(name);if(!element)return null;const panel=element.querySelector(':scope>dialog');return{root:attrs(element),panel:attrs(panel),owner:element.dataset.ixOverlayOwner,generation:element.dataset.ixOverlayGeneration,logical:element.dataset.ixOverlayOpen,native:nativeOpen(panel),declaration:element.getAttribute('data-ix-overlay-trigger-id'),published:element.dataset.ixOverlayActiveItem,itemsOwner:element.dataset.ixOverlayItemsOwner};};
const rust=()=>JSON.parse(document.getElementById('rebind-rust-json')?.value||'[]');
const snapshot=()=>({epoch:document.getElementById('rebind-fixture')?.dataset.epoch,case:document.getElementById('rebind-fixture')?.dataset.case,triggerValue:document.getElementById('rebind-fixture')?.dataset.triggerValue,handleRetained:document.getElementById('rebind-fixture')?.dataset.childHandleRetained,parentA:surface('parent-a'),parentB:surface('parent-b'),child:surface('child'),resident:surface('resident'),parentTriggers:['parent-a','parent-b'].map(name=>attrs(parentTrigger(name))),heldA:attrs(heldA),currentA:attrs(document.querySelector('#rebind-owner [data-rebind-role="link-a"]')),members:[...document.querySelectorAll('#rebind-owner [data-rebind-role]')].filter(element=>!element.getAttribute('data-rebind-role').endsWith('content')).map(attrs),focus:node(document.activeElement),deepFocus:node(active()),rust:rust(),documentScroll:{x:scrollX,y:scrollY}});
const push=(kind,detail={})=>{if(rows.length<20000)rows.push({seq:++serial,time:performance.now(),context,kind,...detail});};
window.addEventListener('error',event=>{const error={message:event.message,source:event.filename,line:event.lineno,column:event.colno};errors.push(error);push('native-error',{error});},{signal:abort.signal});
window.addEventListener('unhandledrejection',event=>{const error={reason:String(event.reason)};errors.push(error);push('native-unhandled-rejection',{error});},{signal:abort.signal});
const facts=()=>({focus:node(active()),callbackCount:rust().length,roots:['parent-a','parent-b','child','resident'].flatMap(name=>{const element=root(name);return element?[{name,object:object(element),generation:element.dataset.ixOverlayGeneration,open:element.dataset.ixOverlayOpen,native:nativeOpen(element.querySelector(':scope>dialog')),declaration:element.getAttribute('data-ix-overlay-trigger-id')}]:[]})});
for(const type of ['keydown','keyup','click','mousedown','mouseup','pointerdown','pointerup','focusin','focusout']){
 document.addEventListener(type,event=>{if(event.target?.matches?.('input[data-ix-overlay-transport]'))return;push('document-capture',{type,event:eventId(event),key:event.key||'',target:node(event.composedPath()[0]),path:event.composedPath().filter(el=>el instanceof Element).map(node),defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,facts:facts()});},{capture:true,signal:abort.signal});
 window.addEventListener(type,event=>{if(event.target?.matches?.('input[data-ix-overlay-transport]'))return;push('window-bubble',{type,event:eventId(event),key:event.key||'',target:node(event.composedPath()[0]),defaultPrevented:event.defaultPrevented,facts:facts()});},{signal:abort.signal});
}
for(const type of ['input','change'])document.addEventListener(type,event=>{
 if(!event.target?.matches?.('input[data-ix-overlay-transport]'))return;let report;try{report=JSON.parse(event.target.value);}catch{return;}
 if(!['click','key','modality'].includes(report.kind))return;
 const retained={owner:report.owner,sequence:report.sequence,kind:report.kind,phase:report.phase,default_prevented:report.default_prevented,key:report.key,target_id:report.target_id,triggers:report.triggers,hosts:report.hosts,nodes:(report.nodes||[]).map(node=>({id:node.id,generation:node.generation,root_capture:node.root_capture,trigger_capture:node.trigger_capture,configured_trigger_listener:node.configured_trigger_listener,trigger_on_path:node.trigger_on_path,on_trigger:node.on_trigger,trigger_ancestors:node.trigger_ancestors,assignment_ancestors:node.assignment_ancestors}))};
 push('native-report-capture',{type,event:eventId(event),report:retained,defaultPrevented:event.defaultPrevented});queueMicrotask(()=>push('native-report-queued-capture-microtask',{type,event:eventId(event),report:retained,defaultPrevented:event.defaultPrevented}));
},{capture:true,signal:abort.signal});
const observer=new MutationObserver(changes=>{const relevant=changes.filter(change=>change.target instanceof Element&&change.target.closest('#rebind-owner'));if(relevant.length)push('mutation',{changes:relevant.map(change=>({type:change.type,attribute:change.attributeName,old:change.oldValue,target:node(change.target),value:change.attributeName?change.target.getAttribute(change.attributeName):null})),facts:facts()});});
observer.observe(document.getElementById('rebind-probe'),{subtree:true,childList:true,attributes:true,attributeOldValue:true,attributeFilter:['id','class','tabindex','hidden','inert','style','aria-activedescendant','aria-expanded','data-active','data-ix-overlay-trigger','data-ix-overlay-trigger-id','data-ix-overlay-open','data-ix-overlay-items-owner']});
window.__ixRebind={rows,errors,object,eventId,node,attrs,root,parentTrigger,snapshot,push,holdA:element=>{heldA=element;},context:value=>{context=value;},sequence:()=>serial};document.getElementById('rebind-probe').dataset.nativeJournalReady='true';
try{await dioxus.recv();}finally{abort.abort();observer.disconnect();}
"#;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
const raf=()=>new Promise(resolve=>requestAnimationFrame(resolve)),assertions=[],driver=[],observations=[];
const assert=(value,message)=>{assertions.push({message,ok:!!value});if(!value)throw Error(message);};
const wait=async(predicate,message)=>{for(let index=0;index<300;index++){if(predicate())return;await raf();}throw Error('setup timeout: '+message);};
const member=role=>document.querySelector('#rebind-owner [data-rebind-role="'+role+'"]');
const mode=caseId=>{const input=document.getElementById('rebind-mode');input.value=String(caseId);input.dispatchEvent(new Event('input',{bubbles:true}));};
const command=action=>{const input=document.getElementById('rebind-command');input.value=action;input.dispatchEvent(new Event('input',{bubbles:true}));};
const plans={1:['B','click:A','click:B','click:B','click:A'],2:['click:A','B','key:A:Home','key:B:End','click:B','click:B','key:B:End'],3:['B','B','A','click:B','click:A'],4:['B','key:PA:Home','key:PA:ArrowRight','click:B','key:PB:Home','key:PB:ArrowRight'],5:['B','click:B','click:PA'],6:['B','click:B','click:PB'],7:['B','click:PB','click:PB','click:B','click:PB'],8:['replace','click:currentA','click:oldA','click:oldA','A','click:currentA','B','A','click:oldA','click:currentA'],9:['move','key:PB:Home','key:PB:ArrowRight','click:A','click:PA'],10:['none','click:A','key:A:End'],12:['empty','click:A','key:A:End']};
let journal;
const drain=async(label,event=null)=>{await Promise.resolve();journal.push('operation-return-microtask',{label,event,state:journal.snapshot()});for(let index=1;index<=12;index++){const timestamp=await raf();journal.push('fixed-observer-frame',{label,event,index,timestamp,state:journal.snapshot()});await Promise.resolve();journal.push('fixed-observer-frame-microtask',{label,event,index,timestamp,state:journal.snapshot()});}};
const dispatchClick=(element,label)=>{assert(element instanceof HTMLElement,label+' actual HTMLElement');const event=new MouseEvent('click',{bubbles:true,cancelable:true,composed:true});const original={event:journal.eventId(event),target:journal.node(element)};journal.push('original-click-before',{label,...original,state:journal.snapshot()});element.dispatchEvent(event);original.defaultPrevented=event.defaultPrevented;journal.push('original-click-return',{label,...original,state:journal.snapshot()});return original;};
const dispatchKey=(element,key,label)=>{assert(element instanceof HTMLElement,label+' actual HTMLElement');const event=new KeyboardEvent('keydown',{key,bubbles:true,cancelable:true,composed:true});const original={event:journal.eventId(event),key,target:journal.node(element)};journal.push('original-key-before',{label,...original,state:journal.snapshot()});element.dispatchEvent(event);original.defaultPrevented=event.defaultPrevented;journal.push('original-key-return',{label,...original,state:journal.snapshot()});return original;};
try{
 await wait(()=>window.__ixRebind,'journal ready');journal=window.__ixRebind;
 for(const caseId of [1,2,3,4,5,6,7,8,9,10,12]){
  const previousEpoch=document.getElementById('rebind-fixture')?.dataset.epoch;mode(caseId);await wait(()=>document.getElementById('rebind-fixture')?.dataset.epoch!==previousEpoch&&document.getElementById('rebind-fixture')?.dataset.case===String(caseId),'fresh epoch R'+caseId);journal.context('R'+String(caseId).padStart(2,'0'));
  await wait(()=>['parent-a','parent-b','child',...(caseId===4?['resident']:[])].every(name=>journal.root(name)?.dataset.ixOverlayOwner)&&member('link-a')?.getAttribute('data-ix-overlay-trigger')===journal.root('child')?.dataset.ixOverlayRoot&&(caseId!==4||member('resident-b-link')?.getAttribute('data-ix-overlay-trigger')===journal.root('resident')?.dataset.ixOverlayRoot),'initial actual public trigger binding');
  const held={A:member('link-a'),B:member('link-b'),PA:journal.parentTrigger('parent-a'),PB:journal.parentTrigger('parent-b')};
  held.oldA=held.A;journal.holdA(held.A);const roots=['parent-a','parent-b','child',...(caseId===4?['resident']:[])].map(name=>({name,object:journal.object(journal.root(name)),generation:journal.root(name).dataset.ixOverlayGeneration}));
  assert(held.A instanceof HTMLElement&&held.B instanceof HTMLElement,'R'+caseId+' actual A/B ready');assert(journal.snapshot().handleRetained==='true','R'+caseId+' actual child handle equality');
  const initial=journal.snapshot();assert([initial.parentA,initial.parentB,initial.child,...(initial.resident?[initial.resident]:[])].every(surface=>surface.logical==='false'&&!surface.native),'R'+caseId+' fresh surfaces initially logical/native closed');
  document.getElementById('rebind-outside').focus();const seed=dispatchKey(document.getElementById('rebind-outside'),'Tab','setup-keyboard');await drain('setup-keyboard',seed.event);
  held.PA.focus();let opened=dispatchClick(held.PA,'setup-parent-a-click');await drain('setup-parent-a-click',opened.event);opened=dispatchClick(held.PB,'setup-parent-b-click');await drain('setup-parent-b-click',opened.event);
  assert(journal.snapshot().parentA.logical==='true'&&journal.snapshot().parentA.native&&journal.snapshot().parentB.logical==='true'&&journal.snapshot().parentB.native,'R'+caseId+' both real parents opened');
  const setup=journal.snapshot(),operations=[];
  try{for(let index=0;index<plans[caseId].length;index++){
   const action=plans[caseId][index],label='R'+String(caseId).padStart(2,'0')+'-operation-'+(index+1),before=journal.snapshot(),from=journal.sequence()+1;let original=null;
   if(['A','B','none','empty'].includes(action))command({'A':'trigger-a','B':'trigger-b','none':'trigger-none','empty':'trigger-empty'}[action]);
   else if(action==='replace')command('replace-a');else if(action==='move')command('move-a');
   else{const [kind,target,key]=action.split(':'),element=target==='currentA'?member('link-a'):held[target];original=kind==='click'?dispatchClick(element,label):dispatchKey(element,key,label);}
   journal.push('operation-return',{label,action,original,state:journal.snapshot()});await drain(label,original?.event??null);const state=journal.snapshot();
   assert(roots.every(held=>journal.object(journal.root(held.name))===held.object&&journal.root(held.name).dataset.ixOverlayGeneration===held.generation),label+' actual roots and generations retained');
   assert(state.handleRetained==='true'&&journal.object(member('link-b'))===journal.object(held.B),label+' actual handle/B object retained');
   if(caseId!==8)assert(member('link-a')===held.A,label+' actual A object retained');
   if(action==='replace')assert(!held.oldA.isConnected&&member('link-a')!==held.oldA,label+' authentic keyed same-ID replacement');
   if(action==='move')assert(member('link-a')===held.A&&held.A.parentNode===held.B.parentNode&&held.A.nextSibling===held.B,label+' actual same-object move before B');
   operations.push({label,action,from,to:journal.sequence(),before,original,state,journal:journal.rows.filter(row=>row.seq>=from&&row.seq<=journal.sequence())});
  }}finally{if(caseId===9){command('restore-a');await drain('R09-finally-restore');assert(held.A.parentNode===document.getElementById('rebind-slot-a'),'R09 actual A restored');}}
  const finalState=journal.snapshot(),mainState=operations.at(-1)?.state??setup;observations.push({case:caseId,id:'R'+String(caseId).padStart(2,'0'),apiDiagnostic:caseId===10||caseId===12,replacementAdapter:caseId===8,nativeMoveDiagnostic:caseId===9,initial,setup,operations,state:caseId===9?mainState:finalState,restoredState:caseId===9?finalState:null});driver.push({case:caseId,operations:operations.length,rootIdentities:roots,originalA:journal.node(held.oldA),currentA:journal.node(member('link-a'))});
 }
 assert(observations.length===11,'all eleven supported histories recorded');
 const result={ok:true,observations,assertions,driver,errors:journal.errors,journalDropped:journal.rows.length>=20000,limits:['source booleanfalse R11 unsupported; None/empty raw API diagnostics','Rust Signal prop changes and source direct properties have separate render/async-resolution timing','fixed12 frames/microtasks, no expected marker/focus wait or corrective input','initial public binding readiness and actual-root/object assertions are setup, not parity','R08 genuine Rust scope/DOM replacement versus authentic source IX replacement is an explicit lifetime adapter; held-detached clicks synthetic diagnostics','R09 actual native reparent control retains object and restores finally; not VDOM remount','raw native receiver capture/transport facts do not expose private controller cache; actual subsequent inputs observe its effects','source suppressOverflowBehavior has no Rust public counterpart; geometry and CSS anchors remain adapters','synthetic inputs not trusted hardware/AT/physical Brand/platform proof','queued-capture microtasks may run between native listeners; no full Rust/IPC callback chronology inferred']};
 document.getElementById('rebind-native-result').textContent=JSON.stringify({ok:true,cases:observations.length,assertions:assertions.length});return result;
}catch(error){const result={ok:false,error:String(error),observations,assertions,driver,errors:journal?.errors,state:journal?.snapshot(),journal:journal?.rows};document.getElementById('rebind-native-result').textContent=JSON.stringify({ok:false,error:String(error)});return result;}
"#;
