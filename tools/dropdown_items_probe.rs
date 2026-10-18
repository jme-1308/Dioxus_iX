//! Excluded mixed Dropdown-item public fixture. Rust owns all controls and
//! callbacks; native scripts only observe DOM and deliver real event objects.
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
                .with_title("iX mixed Dropdown items")
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
    eprintln!("Run the desktop or web mixed Dropdown items probe");
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
    EligibleActive,
    EligibleRoving,
    CategoryActive,
    CategoryRoving,
    ActivateActive,
    ActivateRoving,
    Native,
    Restore,
    CategoryOwn,
    Reorder,
}
impl Mode {
    const ALL: [Self; 12] = [
        Self::Active,
        Self::Roving,
        Self::EligibleActive,
        Self::EligibleRoving,
        Self::CategoryActive,
        Self::CategoryRoving,
        Self::ActivateActive,
        Self::ActivateRoving,
        Self::Native,
        Self::Restore,
        Self::CategoryOwn,
        Self::Reorder,
    ];
    fn key(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Roving => "roving",
            Self::EligibleActive => "eligible-active",
            Self::EligibleRoving => "eligible-roving",
            Self::CategoryActive => "category-active",
            Self::CategoryRoving => "category-roving",
            Self::ActivateActive => "activate-active",
            Self::ActivateRoving => "activate-roving",
            Self::Native => "native",
            Self::Restore => "restore",
            Self::CategoryOwn => "category-own",
            Self::Reorder => "reorder",
        }
    }
    fn parse(value: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|mode| mode.key() == value)
            .unwrap_or(Self::Active)
    }
    fn roving(self) -> bool {
        matches!(
            self,
            Self::Roving
                | Self::EligibleRoving
                | Self::CategoryRoving
                | Self::ActivateRoving
                | Self::Native
                | Self::Restore
                | Self::Reorder
        )
    }
    fn eligibility(self) -> bool {
        matches!(self, Self::EligibleActive | Self::EligibleRoving)
    }
    fn category_first(self) -> bool {
        matches!(
            self,
            Self::CategoryActive | Self::CategoryRoving | Self::CategoryOwn
        )
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
            futures_timer::Delay::new(std::time::Duration::from_secs(90)).await;
            eprintln!("Mixed items native timeout");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        style { {STYLES} }
        main { id:"items-probe",style:"padding:20px;min-height:900px",
            onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move{
                    let query=dom_query::eval(VERIFY);let _=query.send(());
                    match query.join::<Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dropdown items native observations: {value}");window.close();}
                        Ok(value)=>{eprintln!("Dropdown items native failure JSON: {value}");std::process::exit(1);}
                        Err(error)=>{eprintln!("Dropdown items native transport failure: {error:?}");std::process::exit(1);}
                    }
                });}
            },
            h1 {"Mixed Dropdown items"}
            p {"Actual component membership, original keyboard events and retained native attributes."}
            div {style:"display:flex;gap:6px;flex-wrap:wrap",
                for next in Mode::ALL { button {id:format!("items-mode-{}",next.key()),onclick:move |_|{mode.set(next);epoch+=1;},{format!("Fresh {}",next.key())}} }
            }
            input {id:"items-mode",r#type:"hidden",oninput:move |event|{mode.set(Mode::parse(&event.value()));epoch+=1;}}
            button {id:"items-outside","Outside keyboard target"}
            for current in [epoch()] { Fixture {key:"{current}",epoch:current,mode:mode()} }
            NativeJournal {}
            pre {id:"items-native-journal",style:"max-height:180px;overflow:auto;font-size:12px;white-space:pre-wrap","aria-label":"Native mixed item observations"}
        }
    } }
}

#[component]
fn Fixture(epoch: usize, mode: Mode) -> Element {
    let dropdown = use_dropdown();
    let mut roving = use_signal(|| mode.roving());
    let mut reversed = use_signal(|| false);
    let rows = use_signal(Vec::<Value>::new);
    let command = use_callback(move |value: String| {
        record(rows, "public-command", json!({"value":value}));
        match value.as_str() {
            "ad" => roving.set(false),
            "roving" => roving.set(true),
            "reorder" => reversed.toggle(),
            "hide" => dropdown.close(false),
            "show" => dropdown.open(),
            _ => {}
        }
    });
    let mut order = vec![
        "dropdown-a",
        "select-b",
        "menu-c",
        "native-d",
        "native-control",
        "dropdown-z",
        "category-e",
    ];
    if mode.category_first() {
        order.rotate_right(1);
    }
    if mode == Mode::Native {
        order = vec![
            "native-d",
            "dropdown-a",
            "select-b",
            "menu-c",
            "native-control",
            "dropdown-z",
            "category-e",
        ];
    }
    if mode.eligibility() {
        let last = order.pop().expect("Category last");
        order.extend([
            "disabled-dropdown",
            "hidden-select",
            "negative-menu",
            "property-disabled-menu",
            "attribute-disabled-menu",
            "aria-disabled-dropdown",
            "inert-dropdown",
            "css-hidden-dropdown",
            "nested-owner",
        ]);
        order.push(last);
    }
    if reversed() {
        order.retain(|role| *role != "dropdown-z");
        order.insert(0, "dropdown-z");
    }
    rsx! {section {id:"items-fixture","data-epoch":epoch.to_string(),"data-mode":mode.key(),"data-roving":roving().to_string(),
        div {style:"display:flex;gap:6px;flex-wrap:wrap;margin:10px 0",
            button {id:"items-to-ad",onclick:move |_|command.call("ad".into()),"Change to AD"}
            button {id:"items-to-roving",onclick:move |_|command.call("roving".into()),"Change to roving"}
            button {id:"items-reorder",onclick:move |_|command.call("reorder".into()),"Reorder retained items"}
            button {id:"items-hide",onclick:move |_|command.call("hide".into()),"Public hide"}
            button {id:"items-show",onclick:move |_|command.call("show".into()),"Public show"}
            input {id:"items-command",r#type:"hidden",oninput:move |event|command.call(event.value())}
        }
        div {id:"items-owner",
            DropdownTrigger {dropdown,"Open mixed menu"}
            Application {force_breakpoint:ApplicationBreakpoint::Large,class:"items-application",
                menu:rsx! {Menu {label:"Mixed item Menu context",start_expanded:false,pinned:false,
                    Dropdown {controller:dropdown,label:"Mixed members",class:"items-dropdown",navigation_mode:if roving(){DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},close_behavior:DropdownCloseBehavior::None,enable_top_layer:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
                        onbeforechange:move |event:DropdownChangeEvent|record(rows,"request",json!({"open":event.open,"previous":event.previous})),
                        onchange:move |open|record(rows,"completion",json!({"open":open})),
                        div {id:"items-members",
                            for role in order { Member {key:"{role}",role,mode,rows} }
                        }
                    }
                }},
                div {}
            }
        }
        input {id:"items-rust-json",r#type:"hidden",value:serde_json::to_string(&rows()).unwrap_or_default()}
        pre {id:"items-rust-log",style:"max-height:100px;overflow:auto;font-size:12px","{rows():?}"}
    }}
}

#[component]
fn Member(role: &'static str, mode: Mode, rows: Signal<Vec<Value>>) -> Element {
    let original = matches!(mode, Mode::Restore | Mode::Reorder);
    let mut attributes = Vec::<Attribute>::new();
    if original {
        let tabindex = match role {
            "dropdown-a" => Some("5"),
            "select-b" => Some("-1"),
            "menu-c" => Some("0"),
            "dropdown-z" => Some("2"),
            _ => None,
        };
        if let Some(value) = tabindex {
            attributes.push(Attribute::new("tabindex", value, None, false));
        }
    }
    match role {
        "negative-menu" => attributes.push(Attribute::new("tabindex", "-1", None, false)),
        "attribute-disabled-menu" => attributes.push(Attribute::new("disabled", true, None, false)),
        "aria-disabled-dropdown" => {
            attributes.push(Attribute::new("aria-disabled", "true", None, false))
        }
        "inert-dropdown" => attributes.push(Attribute::new("inert", true, None, false)),
        "css-hidden-dropdown" => {
            attributes.push(Attribute::new("style", "display:none", None, false))
        }
        _ => {}
    }
    rsx! {div {"data-items-role":role,class:"items-member",
        onclick:move |_|record(rows,"member-click",json!({"role":role})),
        match role {
            "dropdown-a"=>rsx!{DropdownItem {label:"Dropdown A",attributes:attributes.clone(),onitemclick:move |_|record(rows,"item-click",json!({"role":role}))}},
            "select-b"=>rsx!{SelectItem {value:"select-b",label:"Select B",attributes:attributes.clone(),onclick:move |_|record(rows,"select-click",json!({"role":role})),onitemclick:move |value|record(rows,"select-item-click",json!({"role":role,"value":value}))}},
            "menu-c"=>rsx!{MenuItem {label:"Menu C",attributes:attributes.clone(),close_on_select:false,onclick:move |_|record(rows,"menu-click",json!({"role":role}))}},
            "native-d"=>rsx!{button {id:"items-native-d",r#type:"button","data-ix-roving-item":"",
                onclick:move |_|record(rows,"native-click",json!({"role":role})),"Marked native D"}},
            "native-control"=>rsx!{button {id:"items-unmarked",r#type:"button",onclick:move |_|record(rows,"unmarked-click",json!({"role":role})),"Unmarked native control"}},
            "dropdown-z"=>rsx!{DropdownItem {label:"Dropdown Z",attributes:attributes.clone(),onitemclick:move |_|record(rows,"item-click",json!({"role":role}))}},
            "category-e"=>rsx!{MenuCategory {label:"Category E",attributes:attributes.clone(),
                MenuItem {id:"items-category-child-0","data-items-child-role":"category-child-0",label:"Category child one",close_on_select:false}
                MenuItem {id:"items-category-child-1","data-items-child-role":"category-child-1",label:"Category child two",close_on_select:false}
            }},
            "disabled-dropdown"=>rsx!{DropdownItem {label:"Disabled Dropdown",disabled:true}},
            "hidden-select"=>rsx!{SelectItem {value:"hidden-select",label:"Hidden SelectItem",hidden:true}},
            "negative-menu"=>rsx!{MenuItem {label:"Negative Menu",attributes:attributes.clone(),close_on_select:false}},
            "property-disabled-menu"=>rsx!{MenuItem {label:"Property disabled Menu",disabled:true,close_on_select:false}},
            "attribute-disabled-menu"=>rsx!{MenuItem {label:"Attribute disabled Menu",attributes:attributes.clone(),disabled:true,close_on_select:false}},
            "aria-disabled-dropdown"=>rsx!{DropdownItem {label:"ARIA disabled Dropdown",attributes:attributes.clone()}},
            "inert-dropdown"=>rsx!{DropdownItem {label:"Inert Dropdown",attributes:attributes.clone()}},
            "css-hidden-dropdown"=>rsx!{DropdownItem {label:"CSS hidden Dropdown",attributes:attributes.clone()}},
            "nested-owner"=>rsx!{NestedMember {rows}},
            _=>VNode::empty()
        }
    }}
}

#[component]
fn NestedMember(rows: Signal<Vec<Value>>) -> Element {
    let child = use_dropdown();
    rsx! {
        div {"data-structural-owner":"nested",
            DropdownTrigger {dropdown:child,"Closed nested owner"}
            Dropdown {controller:child,label:"Nested light descendant",close_behavior:DropdownCloseBehavior::None,enable_top_layer:false,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
                div {"data-items-role":"nested-child",DropdownItem {label:"Nested light item",onitemclick:move |_|record(rows,"item-click",json!({"role":"nested-child"}))}}
            }
        }
    }
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
#items-owner .items-application{height:560px}
#items-owner nav.ix-menu{position:relative;inset:auto;height:540px;width:52px;overflow:visible}
#items-owner .items-dropdown>.ix-dropdown-root-menu{max-height:440px;width:max-content}
#items-members{width:320px;max-height:360px;overflow:auto}
.items-member{min-height:36px}
.items-member>[data-ix-item-kind],.items-member>button{box-sizing:border-box;min-height:36px;width:100%}
#items-native-journal{max-width:100%}
"#;

const OBSERVE: &str = r#"
const probe=document.getElementById('items-probe');if(!probe)return;
const rows=[],objects=new WeakMap(),events=new WeakMap(),abort=new AbortController();let sequence=0,objectId=0,eventSequence=0,scenario='bootstrap',alive=true;
const oid=node=>{if(!node)return null;if(!objects.has(node))objects.set(node,++objectId);return objects.get(node);};
const node=el=>el instanceof Element?{object:oid(el),id:el.id,tag:el.tagName,role:el.closest('[data-items-child-role]')?.dataset.itemsChildRole??el.closest('[data-items-role]')?.dataset.itemsRole,connected:el.isConnected}:null;
const rect=el=>{const r=el?.getBoundingClientRect();return r?{x:r.x,y:r.y,width:r.width,height:r.height,top:r.top,bottom:r.bottom}:null;};
const focus=()=>{const out=[];let el=document.activeElement;while(el){out.push(node(el));el=el.shadowRoot?.activeElement;}return out;};
const root=()=>document.querySelector('#items-owner .items-dropdown[data-ix-overlay-root]');
const candidate=wrapper=>{const el=wrapper?.firstElementChild;return el?.matches('[data-ix-item-kind],button')?el:null;};
const itemState=el=>({node:node(el),tabindex:el?.getAttribute('tabindex')??null,tabIndex:el?.tabIndex,disabledAttribute:el?.hasAttribute('disabled'),disabledProperty:typeof el?.disabled==='boolean'?el.disabled:null,hiddenAttribute:el?.hasAttribute('hidden'),inert:el?.inert,ariaDisabled:el?.getAttribute('aria-disabled'),display:el?getComputedStyle(el).display:null,visibility:el?getComputedStyle(el).visibility:null,rect:rect(el),active:el?.dataset.active,sourceFocused:el?.classList.contains('ix-focused'),actualFocused:!!el?.contains(document.activeElement),kind:el?.dataset.ixItemKind,key:el?.dataset.ixItemKey,owner:el?.getAttribute('data-ix-dropdown-owner'),marker:el?.hasAttribute('data-ix-roving-item'),nativeControls:[...el?.querySelectorAll('[data-ix-item-control]')??[]].map(control=>({node:node(control),disabled:control.disabled??null,tabindex:control.getAttribute('tabindex')}))});
const member=wrapper=>({role:wrapper.dataset.itemsRole,wrapper:node(wrapper),structural:!candidate(wrapper),...itemState(candidate(wrapper))});
const hostQuery='[data-ix-item-kind="dropdown"],[data-ix-item-kind="select"],[data-ix-item-kind="menu"],[data-ix-item-kind="category"]';
const inventory=r=>{const hosts=[...r?.querySelectorAll(hostQuery)??[]],roving=[...r?.querySelectorAll(hostQuery+',[data-ix-roving-item]')??[]];const ownEligible=el=>!el.hasAttribute('disabled')&&!el.hasAttribute('hidden'),ad=hosts.filter(el=>ownEligible(el)&&!el.getAttribute('tabindex')?.startsWith('-'));return{rawHosts:hosts.map(node),directAD:ad.map(node),directRoving:roving.filter(ownEligible).map(node),currentVisible:ad.filter(el=>el.classList.contains('ix-focused')).map(node),currentTab0:roving.filter(el=>ownEligible(el)&&el.getAttribute('tabindex')==='0').map(node),scope:'raw own-attribute selector inventory; no computed visibility/inert/nearest-owner filtering'};};
const snapshot=()=>{const r=root(),host=r?.querySelector(':scope>dialog'),trigger=document.querySelector('#items-owner>[data-ix-overlay-trigger]');let native=false;try{native=!!host?.matches(':popover-open');}catch{}return{epoch:document.getElementById('items-fixture')?.dataset.epoch,mode:document.getElementById('items-fixture')?.dataset.mode,root:node(r),owner:r?.dataset.ixOverlayOwner,generation:r?.dataset.ixOverlayGeneration,logical:r?.dataset.ixOverlayOpen,native,modeAttribute:r?.dataset.ixOverlayMode,trigger:node(trigger),activeDescendant:trigger?.getAttribute('aria-activedescendant'),publishedActive:r?.dataset.ixOverlayActiveItem,focus:focus(),inventory:inventory(r),members:[...document.querySelectorAll('#items-members [data-items-role]')].map(member),directOrder:[...document.querySelectorAll('#items-members>[data-items-role]')].map(el=>el.dataset.itemsRole),category:[...document.querySelectorAll('#items-members .ix-menu-category')].map(el=>({node:node(el),parent:itemState(el.querySelector(':scope>[role=menuitem]')),popup:el.dataset.menuCategoryPopupOpen,body:node(el.querySelector('[data-menu-category-body]')),children:[...el.querySelectorAll('[data-items-child-role]')].map(itemState),nativeChildren:[...el.querySelectorAll('[data-menu-category-child]')].map(node)})),rust:JSON.parse(document.getElementById('items-rust-json')?.value||'[]')};};
const publish=()=>{const el=document.getElementById('items-native-journal');if(el)el.textContent=JSON.stringify({scenario,rows:rows.slice(-24)});};
const record=(kind,details={})=>{const row={seq:++sequence,scenario,kind,time:performance.now(),...details,state:snapshot()};rows.push(row);if(rows.length>600)rows.shift();publish();return row;};
const eid=event=>{if(!events.has(event))events.set(event,++eventSequence);return events.get(event);};
for(const type of ['keydown','keyup','click','focusin','focusout']){
 document.addEventListener(type,event=>{if(!probe.contains(event.target))return;const id=eid(event);record('document-capture',{event:id,type,key:event.key,alt:event.altKey,target:node(event.composedPath()[0]),path:event.composedPath().filter(el=>el instanceof Element).map(node),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented});if(type==='keydown'||type==='click'){queueMicrotask(()=>{if(alive)record('queued-capture-microtask',{event:id,type,key:event.key,defaultPrevented:event.defaultPrevented});});let frame=0;const observe=time=>{if(!alive)return;record('event-frame',{event:id,type,key:event.key,frame:++frame,frameTime:time,defaultPrevented:event.defaultPrevented});if(frame<4)requestAnimationFrame(observe);};requestAnimationFrame(observe);}},{capture:true,passive:true,signal:abort.signal});
 window.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('window-bubble',{event:eid(event),type,key:event.key,target:node(event.composedPath()[0]),defaultPrevented:event.defaultPrevented,trusted:event.isTrusted});},{passive:true,signal:abort.signal});
}
document.addEventListener('input',event=>{if(!event.target.matches?.('input[data-ix-overlay-transport]'))return;try{const report=JSON.parse(event.target.value);if(report.kind==='key')record('raw-overlay-key-facts',{report});}catch{}},{capture:true,passive:true,signal:abort.signal});
window.__ixItemsProbe={rows,snapshot,node,record,eventId:eid,setScenario:value=>{scenario=value;record('scenario');}};probe.dataset.nativeJournal='true';record('journal-ready');
try{await dioxus.recv();}finally{alive=false;abort.abort();delete window.__ixItemsProbe;delete probe.dataset.nativeJournal;}
"#;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
await dioxus.recv();
const observations=[],assertions=[],driver=[];let scenario='bootstrap';
const get=id=>document.getElementById('items-'+id),api=()=>window.__ixItemsProbe,sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const root=()=>document.querySelector('#items-owner .items-dropdown[data-ix-overlay-root]'),trigger=()=>document.querySelector('#items-owner>[data-ix-overlay-trigger]');
const role=value=>document.querySelector('#items-members [data-items-role="'+value+'"]'),member=value=>{const el=role(value)?.firstElementChild;return el?.matches('[data-ix-item-kind],button')?el:null;};
const state=()=>api()?.snapshot(),record=(kind,details={})=>{const row={scenario,kind,time:performance.now(),...details,state:state()};driver.push(row);return row;};
const assert=(ok,label)=>{assertions.push({scenario,label,ok:!!ok});if(!ok)throw Error(label+' '+JSON.stringify(state()));};
const wait=async(fn,label,ms=6000)=>{const end=performance.now()+ms;while(performance.now()<end){if(fn())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify(state()));};
const frames=async(count=5)=>{for(let i=0;i<count;i++)await new Promise(requestAnimationFrame);await sleep(0);};
const control=(id,value)=>{get(id).value=value;get(id).dispatchEvent(new Event('input',{bubbles:true}));record('Rust-public-control',{id,value});};
const key=(target,value)=>{assert(target?.isConnected,'connected actual key target');const event=new KeyboardEvent('keydown',{key:value,bubbles:true,composed:true,cancelable:true});record('synthetic-key-before',{event:api().eventId(event),key:value,target:api().node(target),trusted:event.isTrusted});target.dispatchEvent(event);record('synthetic-key-return',{event:api().eventId(event),key:value,target:api().node(target),defaultPrevented:event.defaultPrevented});return{event:api().eventId(event),key:value,target:api().node(target),defaultPrevented:event.defaultPrevented};};
const input=()=>{if(root()?.dataset.ixOverlayMode!=='roving-tabindex')return trigger();const active=document.activeElement;if(root()?.contains(active))return active;return document.querySelector('#items-members [data-ix-item-kind][tabindex="0"],#items-members [data-ix-roving-item][tabindex="0"]')??trigger();};
async function fresh(label,mode){scenario=label;const old=get('fixture');control('mode',mode);await wait(()=>get('fixture')!==old&&get('fixture')?.dataset.mode===mode&&root()?.dataset.ixOverlayGeneration&&root()?.dataset.ixOverlayOwner&&document.querySelector('[data-ix-overlay-transport]')&&api(),'fresh Rust mixed owner');api().setScenario(label);await frames(4);const original=state();get('outside').focus();key(get('outside'),'Tab');trigger().focus();trigger().click();await wait(()=>state().logical==='true'&&state().native,'real trigger native Root opening');await frames(6);if(root().dataset.ixOverlayMode==='roving-tabindex'){key(trigger(),'ArrowDown');await frames(7);}assert(member('dropdown-a')&&member('select-b')&&member('menu-c')&&member('native-d')&&member('category-e'),'actual available component/native members');record('fixture-ready',{originalAttributes:original.members.map(row=>({role:row.role,node:row.node,tabindex:row.tabindex})),setupAdapter:'source public show property has no own request; Rust actual registered trigger opens and is excluded operation setup'});return{original,ready:state()};}
async function operation(keys,label){const cursor=api().rows.at(-1)?.seq??0,before=state();record('operation-before',{label,cursor});const results=[];for(const value of keys){results.push(key(input(),value));await frames(5);}const row={label,cursor,original:results,before,state:state(),journal:api().rows.filter(row=>row.scenario===scenario&&row.seq>cursor)};record('operation-settled',{label,cursor});return row;}
function save(caseId,label,operations,extra={}){observations.push({case:caseId,label,operations,state:state(),...extra});}
try{
 await wait(()=>api()&&get('probe')?.dataset.nativeJournal==='true','passive native journal');
 for(const roving of [false,true]){
  const mode=roving?'roving':'active';await fresh(mode+' mixed inventory walk',mode);const ops=[];
  for(const value of ['Home','ArrowDown','ArrowDown','ArrowDown','End','ArrowDown'])ops.push(await operation([value],'mixed-walk'));
  save(roving?2:1,scenario,ops);
 }
 for(const roving of [false,true]){
  const mode=roving?'eligible-roving':'eligible-active';await fresh(mode+' actual eligibility variants',mode);const before=state(),ops=[];
  for(const value of ['Home','ArrowDown','ArrowDown','End'])ops.push(await operation([value],'eligible-walk'));
  save(roving?4:3,scenario,ops,{eligibilityBefore:before});
 }
 for(const roving of [false,true]){
  const mode=roving?'category-roving':'category-active';await fresh(mode+' Category-first projection and continuity',mode);const ops=[];
  for(const value of ['Home','ArrowDown','ArrowDown'])ops.push(await operation([value],'category-first'));
  save(roving?6:5,scenario,ops);
 }
 for(const roving of [false,true]){
  const ops=[],subhistories=[];
  for(const selected of ['dropdown-a','select-b','menu-c','category-e']){
   const mode=roving?'activate-roving':'activate-active';const setup=await fresh(mode+' IX activation '+selected,mode);
   const count={'dropdown-a':0,'select-b':1,'menu-c':2}[selected],selection=selected==='category-e'?['End']:['Home',...Array(count).fill('ArrowDown')];
   const from=ops.length;ops.push(await operation(selection,'select-'+selected));ops.push(await operation(['Enter'],selected+'-activation'));ops.push(await operation([' '],selected+'-activation'));
   subhistories.push({role:selected,setup,from,to:ops.length,state:state()});
  }
  save(roving?8:7,(roving?'Roving':'AD')+' four IX activation subhistories',ops,{subhistories});
 }
 {
  const setup=await fresh('Roving marked native synthetic pass-through','native'),ops=[];
  for(const value of ['Home','Enter',' '])ops.push(await operation([value],'native-marker'));
  save(9,scenario,ops,{setup,classification:'Synthetic keyboard events have no browser default activation; raw pass-through/defaults and click absence only'});
 }
 {
  const setup=await fresh('Roving to AD exact attribute restoration','restore'),ops=[await operation(['End'],'before-mode-change')];
  const before=state();control('command','ad');await wait(()=>root()?.dataset.ixOverlayMode==='active-descendant','actual public navigation mode commit');await frames(8);
  save(10,scenario,ops,{originalAttributes:setup.original.members,modeChangeBefore:before,modeChangeAfter:state()});
 }
 {
  await fresh('Category projection entry and own child End','category-own');const ops=[await operation(['Home'],'outer-Home')];
  const parent=member('category-e')?.querySelector(':scope>[role=menuitem]');assert(parent,'real Category parent');const cursor=api().rows.at(-1)?.seq??0;
  const enter=key(parent,'Enter');await wait(()=>member('category-e')?.dataset.menuCategoryPopupOpen==='true','actual Category Enter popup');await frames(8);
  ops.push({label:'category-entry',cursor,original:[enter],state:state(),journal:api().rows.filter(row=>row.scenario===scenario&&row.seq>cursor)});
  const child=member('category-e').querySelector('[data-items-child-role="category-child-0"]');assert(child,'real Category child source host');const childCursor=api().rows.at(-1)?.seq??0;
  const end=key(child,'End');await frames(8);ops.push({label:'category-own-End',cursor:childCursor,original:[end],state:state(),journal:api().rows.filter(row=>row.scenario===scenario&&row.seq>childCursor)});
  save(11,scenario,ops);
 }
 {
  const setup=await fresh('Roving same-object reorder and hide restoration','reorder'),before=state();control('command','reorder');
  await wait(()=>get('members')?.firstElementChild?.dataset.itemsRole==='dropdown-z','real keyed Dropdown Z prepend');await frames(5);
  const reordered=state(),ops=[await operation(['Home','End'],'reordered-edges')];control('command','hide');await wait(()=>state().logical==='false'&&!state().native,'public Root hide native completion');await frames(8);
  save(12,scenario,ops,{originalAttributes:setup.original.members,reorderBefore:before,reordered,hidden:state()});
 }
 assert(observations.length===12,'12 raw mixed-item histories');return{ok:true,observations,assertions,driver,journal:api().rows,limits:['synthetic keyboard inputs do not certify browser default activation','source property show versus Rust trigger/handle setup is an explicit adapter','initial missing-API scaffold is preserved separately; this fixture uses actual SelectItem','candidate identity and raw eligibility are separate from native focus success','same-object Rust-keyed reorder and source native reparent are distinct renderer inputs; observed identities retained','no full chronology/Brand/physical/native/AT parity claim']};
}catch(error){return{ok:false,error:String(error),scenario,state:state(),observations,assertions,driver,journal:api()?.rows??[]};}
"#;
