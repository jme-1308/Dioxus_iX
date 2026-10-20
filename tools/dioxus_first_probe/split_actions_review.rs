//! Excluded actual-gallery shared SplitButton actions and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/split_work_orders.rs"]
mod gallery_split;

#[component]
pub fn SplitActionsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_split::STYLES} }
        IxTheme { class: "split-actions-review",
            h1 { "Shared split actions" }
            Button { id: "split-actions-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "split-actions-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_split::SplitWorkOrdersExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_SPLIT_ACTIONS_CHECK);
                        if let Err(error) = query.send("splitactions") {
                            eprintln!("Split actions native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native split actions: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native split actions failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Split actions native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.split-actions-review{box-sizing:border-box;min-height:100vh;padding:24px}.split-actions-review h1{font-size:28px;margin:0 0 24px}.split-actions-review h2{font-size:22px;margin:24px 0 16px}.split-actions-review h3{font-size:18px}.split-actions-review p{max-width:72ch;line-height:1.5}#split-actions-frame{margin-top:24px}@media(max-width:600px){.split-actions-review{padding:16px}}";

// Excluded driver JS observes actual composed SplitButton/Dropdown state and
// sends synthetic events with explicit native focus. Untrusted native-button
// Enter/Space and Tab cannot activate browser defaults: permission is recorded,
// then an explicit DOM click/focus exercises the action. DropdownItem keyboard
// actions use the actual shared semantic Rust handler, without fabricated clicks.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SPLIT_ACTIONS_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('split-work-orders'),split=()=>get('work-order-split');
 const primary=()=>split()?.querySelector('button[aria-label="Create maintenance work order"]');
 const trigger=()=>split()?.querySelector('button[aria-label="More work order actions"][aria-haspopup="menu"]');
 const menu=()=>get(trigger()?.getAttribute('aria-controls'));
 const opened=()=>trigger()?.getAttribute('aria-expanded')==='true'&&menu()&&!menu().hidden;
 const note=()=>get('work-order-note'),inspection=()=>get('work-order-inspection'),service=()=>get('work-order-service'),locked=()=>get('work-order-locked');
 const count=()=>Number(get('work-order-result')?.dataset.count);
 const inspections=()=>Number(get('work-order-inspections')?.dataset.count);
 const records=()=>[...get('work-order-records')?.querySelectorAll('tbody tr[data-order-id]')??[]];
 const recordField=(record,field)=>record?.querySelector('[data-field="'+field+'"]')?.textContent.trim();
 const latestRecord=()=>records().at(-1);
 const menuState=()=>get('work-order-menu-state')?.dataset.open;
 const checked=node=>node instanceof HTMLInputElement?node.checked:node.getAttribute('aria-checked')==='true';
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; result='+get('work-order-result')?.textContent+'; menu='+menuState()+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const key=(node,key,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key,code:key===' '?'Space':key,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setNote=async text=>{
  const node=note();assert(node&&opened(),'actual open retained note input exists');
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait('retained note draft applied',()=>note()===node&&note().value===text);
 };
 const open=async(edge='ArrowDown')=>{
  if(opened())return;
  await focus(trigger(),'shared split trigger');key(trigger(),edge);
  await wait('split menu opens with mounted enabled item focus',()=>opened()&&menuState()==='true'&&active()===(edge==='ArrowUp'?service():note()));
  assert(menu().getAttribute('role')==='menu'&&menu().getAttribute('aria-label')==='Work order actions','shared Dropdown supplies labeled actual menu');
 };
 const escape=async()=>{
  assert(opened(),'Escape has an open actual menu');key(active()&&menu().contains(active())?active():trigger(),'Escape');
  await wait('Escape closes split menu and returns its current trigger',()=>!opened()&&menuState()==='false'&&active()===trigger());
 };
 const choose=async(kind,method='Enter',closes=true)=>{
  const target=kind==='Inspection'?inspection():service(),before=count(),expectedNote=note().value;
  assert(opened()&&target,'actual composed action available');
  if(method==='click')target.click();else{await focus(target,kind+' action');key(target,method)}
  await wait(kind+' action appends exactly one real stored record',()=>count()===before+1&&records().length===before+1&&recordField(latestRecord(),'kind')===kind&&recordField(latestRecord(),'note')===expectedNote);
  await wait(kind+' respects configured close policy',()=>closes?!opened()&&menuState()==='false':opened()&&menuState()==='true');
  if(closes&&method!=='click')await wait('semantic keyboard action returns trigger focus',()=>active()===trigger());
 };
 const outside=async closes=>{
  const before=inspections();get('work-order-outside').focus();get('work-order-outside').click();
  await wait('outside Inspect asset action reaches its real target',()=>inspections()===before+1);
  await wait('outside click respects configured close policy',()=>closes?!opened()&&menuState()==='false':opened()&&menuState()==='true');
  assert(active()===get('work-order-outside'),'outside destination native focus is preserved');
 };
 const setPolicy=async value=>{
  assert(!opened(),'close policy changes start with menu closed');
  const node=get('work-order-close');node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));
  await wait('close policy '+value+' applied to actual surface',()=>node.value===value&&trigger()&&!opened());
 };
 const snapshot=()=>({count:count(),inspections:inspections(),open:opened(),publishedOpen:menuState(),note:note()?.value,policy:get('work-order-close')?.value,
  records:records().map(record=>({id:record.dataset.orderId,kind:recordField(record,'kind'),note:recordField(record,'note')})),
  disabled:checked(get('work-order-disabled')),mainDisabled:checked(get('work-order-main-disabled')),menuDisabled:checked(get('work-order-menu-disabled')),topLayer:checked(get('work-order-top-layer')),veto:checked(get('work-order-veto'))});

 await wait('actual work-order split gallery mounted',()=>host()&&primary()&&trigger()&&menu()&&note()&&get('work-order-records')&&get('work-order-result')&&get('work-order-inspections')&&get('work-order-menu-state'));
 await wait('native WebView ready for focus',()=>document.hasFocus());
 const initialNote=note().value,initialTop=checked(get('work-order-top-layer'));
 assert(initialNote==='Inspect Motor 001'&&note().maxLength===80&&count()===0&&records().length===0&&inspections()===0&&!opened()&&menuState()==='false','initial local records and actual menu are empty/closed');
 const reset=async()=>{
  get('work-order-reset').focus();get('work-order-reset').click();
  await wait('Restore work orders resets local records/note/options and owned menu',()=>count()===0&&records().length===0&&inspections()===0&&note()?.value===initialNote&&!opened()&&menuState()==='false'&&!primary().disabled&&!trigger().disabled&&!checked(get('work-order-disabled'))&&!checked(get('work-order-main-disabled'))&&!checked(get('work-order-menu-disabled'))&&!checked(get('work-order-veto'))&&checked(get('work-order-top-layer'))===initialTop&&get('work-order-close').value==='both');
 };

 primary().click();await wait('main pointer action adds a real maintenance order',()=>count()===1&&records().length===1&&recordField(latestRecord(),'kind')==='Maintenance'&&recordField(latestRecord(),'note')===initialNote);
 for(const value of ['Enter',' ']){
  await focus(primary(),'native primary work-order button');const before=count(),event=key(primary(),value);keyPermissions['main'+value]=!event.defaultPrevented;
  assert(!event.defaultPrevented,'native main button '+value+' default is not hijacked');
  await wait('untrusted native-button key does not fabricate an action',()=>count()===before);
  primary().click();await wait('explicit native click after '+value+' adds one order',()=>count()===before+1&&records().length===before+1&&recordField(latestRecord(),'kind')==='Maintenance');
 }
 snapshots.primary=snapshot();

 await open();assert(locked().getAttribute('aria-disabled')==='true','safety interlock action remains disabled');
 await focus(inspection(),'first shared action after registered editing field');
 key(active(),'End');await wait('End selects final enabled service action',()=>active()===service());
 key(active(),'Home');await wait('Home selects first inspection action',()=>active()===inspection());
 key(active(),'ArrowDown');await wait('ArrowDown selects next enabled action',()=>active()===service());
 key(active(),'ArrowUp');await wait('ArrowUp selects prior enabled action',()=>active()===inspection());
 const beforeLocked=count();locked().click();await wait('disabled composed action never creates a work order',()=>count()===beforeLocked);
 if(!opened())await open();
 await choose('Service',' ');await open('ArrowUp');await choose('Inspection','Enter');snapshots.navigation=snapshot();

 await open();const retained=note();await focus(retained,'actual custom work-order note');
 for(const value of ['Home','End','ArrowLeft','ArrowDown',' ']){
  const event=key(note(),value);keyPermissions['note'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native note '+value+' keeps native default');
 }
 await setNote('Inspect seal soon');await escape();
 await open();assert(note()===retained&&note().value==='Inspect seal soon','ordinary close/reopen retains actual custom native node and local Dioxus note state');
 await escape();const beforeNote=count();primary().click();
 await wait('primary action uses edited retained note in stored data',()=>count()===beforeNote+1&&recordField(latestRecord(),'note')==='Inspect seal soon');snapshots.retainedNote=snapshot();

 get('work-order-main-disabled').click();await wait('only main action disabled',()=>primary().disabled&&!trigger().disabled);
 const beforeMain=count();primary().click();await wait('disabled main action ignored',()=>count()===beforeMain);
 await open();await choose('Inspection');get('work-order-main-disabled').click();await wait('main action reenabled',()=>!primary().disabled);
 get('work-order-menu-disabled').click();await wait('only alternatives disabled',()=>trigger().disabled&&!primary().disabled);
 trigger().click();await wait('disabled split trigger cannot open',()=>!opened());
 const beforePrimary=count();primary().click();await wait('main remains usable with alternatives disabled',()=>count()===beforePrimary+1&&recordField(latestRecord(),'kind')==='Maintenance');
 get('work-order-menu-disabled').click();await wait('split trigger reenabled',()=>!trigger().disabled);await open();
 get('work-order-menu-disabled').click();await wait('disabling alternatives closes current shared menu',()=>trigger().disabled&&!opened()&&menuState()==='false');
 get('work-order-menu-disabled').click();get('work-order-disabled').click();await wait('Disable all reaches both native buttons',()=>primary().disabled&&trigger().disabled);
 const beforeAll=count();primary().click();trigger().click();await wait('fully disabled native buttons do not create orders or open',()=>count()===beforeAll&&!opened());snapshots.disabled=snapshot();
 await reset();

 await setPolicy('both');await open();await choose('Inspection','click',true);await open();await outside(true);
 await setPolicy('inside');await open();await choose('Service','click',true);await open();await outside(false);await escape();
 await setPolicy('outside');await open();await choose('Inspection','click',false);await outside(true);
 await setPolicy('none');await open();await choose('Service','click',false);await outside(false);await escape();
 assert(count()===4&&inspections()===4,'four close policies preserve real inside/outside actions exactly once');snapshots.closePolicies=snapshot();
 await reset();

 get('work-order-veto').click();await wait('menu change veto enabled',()=>checked(get('work-order-veto')));
 await focus(trigger(),'vetoed split trigger');key(trigger(),'ArrowDown');
 await wait('veto prevents accepted opening',()=>!opened()&&menuState()==='false'&&count()===0);
 get('work-order-veto').click();await wait('veto released for opening',()=>!checked(get('work-order-veto')));await setPolicy('inside');await open();
 get('work-order-veto').click();await wait('open menu veto enabled',()=>checked(get('work-order-veto'))&&opened());
 await focus(inspection(),'vetoed Escape item');key(active(),'Escape');await wait('veto keeps actual menu open on Escape',()=>opened()&&menuState()==='true');
 await choose('Inspection','Enter',false);await outside(false);
 assert(count()===1&&inspections()===1,'veto concerns state changes while actual actions remain usable');snapshots.veto=snapshot();
 get('work-order-veto').click();await wait('veto released',()=>!checked(get('work-order-veto')));if(opened())await escape();
 await reset();

 if(!checked(get('work-order-top-layer')))get('work-order-top-layer').click();
 await wait('top-layer option enabled',()=>checked(get('work-order-top-layer')));await open();
 await wait('shared split Dropdown uses actual native top layer',()=>menu()?.hasAttribute('popover')&&menu().matches(':popover-open'));
 await choose('Service','Enter');assert(count()===1&&recordField(latestRecord(),'kind')==='Service','top-layer semantic item creates one real order');snapshots.topLayer=snapshot();
 await open();await focus(note(),'note before external restore');await setNote('Discard restored note');const removedNote=note();
 await reset();assert(!removedNote.isConnected&&note()!==removedNote&&active()===get('work-order-reset'),'explicit Restore replaces only owned split content and preserves toolbar focus');
 primary().click();await wait('fresh main action after reset remains usable once',()=>count()===1&&records().length===1&&recordField(latestRecord(),'note')===initialNote);snapshots.reset=snapshot();
 await reset();

 get('split-actions-narrow').click();await wait('340px work-order frame applied',()=>get('split-actions-frame').getBoundingClientRect().width<=341);
 if(!checked(get('work-order-top-layer')))get('work-order-top-layer').click();await wait('narrow top layer enabled',()=>checked(get('work-order-top-layer')));await open();
 await focus(note(),'narrow native work-order note');
 await wait('narrow menu note is visible and hit-testable',()=>{
  const node=note(),box=node?.getBoundingClientRect();
  if(!box||box.width<=0||box.height<=0||box.left<0||box.top<0||box.right>innerWidth+1||box.bottom>innerHeight+1)return false;
  const hit=document.elementFromPoint(box.left+box.width/2,box.top+box.height/2);return hit===node||node.contains(hit);
 });
 const mainBox=primary().getBoundingClientRect(),triggerBox=trigger().getBoundingClientRect(),menuBox=menu().getBoundingClientRect();
 layout={frameWidth:get('split-actions-frame').getBoundingClientRect().width,mainWidth:mainBox.width,mainHeight:mainBox.height,triggerWidth:triggerBox.width,triggerHeight:triggerBox.height,joinedGap:triggerBox.left-mainBox.right,menuWidth:menuBox.width,menuHeight:menuBox.height,menuLeft:menuBox.left,menuRight:menuBox.right,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(Math.abs(layout.mainHeight-32)<=1&&Math.abs(layout.triggerHeight-32)<=1&&Math.abs(layout.joinedGap)<=1&&menuBox.left>=0&&menuBox.right<=innerWidth+1&&layout.pageScroll<=layout.pageClient+1,'joined Brand buttons and narrow menu/page containment remain usable');
 await setNote('Narrow service order');await choose('Service','Enter');assert(recordField(latestRecord(),'note')==='Narrow service order','narrow retained input feeds actual composed action');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,
  checks:['native primary pointer and keyboard-default permission with explicit click records','shared AD registered note opening and action Home/End/arrows/disabled skip with semantic Enter/Space once','native custom note keys and actual retained node/state across ordinary close','independent main/menu/all disable modes and configuration close','Both/Inside/Outside/None preserve real inside and outside action delivery','opening/closing veto preserves actual menu state while allowing content actions','native top-layer split actions append real records','Restore replaces owned retained content and clears local options/data/actions','340px joined Brand buttons with hit-testable top-layer native note and composed order'],
  input:'synthetic input/click/key events and explicit native focus; native-button Enter/Space and Tab defaults are not simulated',
  trustedInput:'separate Chromium native primary Enter/Space and custom-note Tab/typing; legacy-item callback/cancellation covered by Rust tests; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,documentFocusedNow:document.hasFocus()}}
"#;
