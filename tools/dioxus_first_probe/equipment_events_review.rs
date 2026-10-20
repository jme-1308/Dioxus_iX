//! Excluded actual-gallery equipment events and bounded Linux native DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/equipment_events.rs"]
mod gallery_events;

#[component]
pub fn EquipmentEventsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_events::STYLES} }
        IxTheme { class: "equipment-events-review",
            h1 { "Independent equipment event controls" }
            Button { id: "equipment-events-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "equipment-events-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_events::EquipmentEventsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_EQUIPMENT_EVENTS_CHECK);
                        if let Err(error) = query.send("equipmentevents") {
                            eprintln!("Equipment event native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native equipment events: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native equipment events failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Equipment event native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.equipment-events-review{box-sizing:border-box;min-height:100vh;padding:24px}.equipment-events-review h1{font-size:28px;margin:0 0 24px}.equipment-events-review h2{font-size:22px;margin:24px 0 16px}.equipment-events-review h3{font-size:18px}.equipment-events-review p{max-width:72ch;line-height:1.5}#equipment-events-frame{margin-top:24px}@media(max-width:600px){.equipment-events-review{padding:16px}}";

// Excluded driver JS observes real rows/details/controls and sends synthetic
// events. Native Enter/Space/Tab defaults cannot be activated by untrusted keys:
// permission is checked, then one explicit DOM click exercises the action.
// Inputs are scrolled into view before programmatic focus as native test setup;
// WebKit focus alone can expose only the caret rather than the entire field.
// Actual Dioxus owns all records, local notes, filtering, context and lifecycle.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_EQUIPMENT_EVENTS_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},paint={},animation={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('equipment-events'),result=()=>get('events-result'),list=()=>host()?.querySelector('.ix-event-list[role="list"]');
 const row=id=>get('events-row-'+id),primary=id=>row(id)?.querySelector('button.ix-event-list-item');
 const body=id=>row(id)?.querySelector('.ix-event-row-body'),note=id=>get('events-note-'+id),ack=id=>get('events-ack-'+id);
 const indicator=id=>body(id)?.querySelector('.ix-event-indicator'),chevron=id=>body(id)?.querySelector('.ix-event-chevron');
 const allRows=()=>[...list()?.querySelectorAll('.ix-event-row[role="listitem"]')??[]];
 const visibleRows=()=>allRows().filter(node=>node.getClientRects().length>0);
 const ids=()=>visibleRows().map(node=>node.id.replace('events-row-',''));
 const count=name=>Number(result()?.dataset[name]),detail=()=>get('events-detail');
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
  throw Error(label+'; result='+result()?.textContent+'; visible='+ids()+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const key=(node,key)=>{const event=new KeyboardEvent('keydown',{key,code:key===' '?'Space':key,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setInput=async(node,text,label)=>{
  assert(node,label+' exists');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait(label+' actual input applied',()=>node.isConnected&&node.value===text);
 };
 const choice=async(id,value)=>{
  const node=get(id);assert(node&&[...node.options].some(option=>option.value===value),id+' configured option exists');
  node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' actual choice '+value+' applied',()=>node.isConnected&&node.value===value);
 };
 const select=async id=>{
  const before=count('selections');primary(id).click();
  await wait('one native row selection updates actual selected detail '+id,()=>count('selections')===before+1&&primary(id).getAttribute('aria-pressed')==='true'&&detail().textContent.includes('EVT-'+id));
 };
 const acknowledge=async(id,expected)=>{
  const before=count('acknowledgements'),selectedBefore=count('selections');ack(id).click();
  await wait('native acknowledgement changes actual '+id+' record and action name',()=>count('acknowledgements')===before+1&&ack(id).getAttribute('aria-label')===(expected?'Reopen EVT-':'Acknowledge EVT-')+id);
  assert(count('selections')===selectedBefore,'independent acknowledgement does not select its row');
 };
 const snapshot=()=>({visible:count('visible'),total:count('total'),selections:count('selections'),acknowledgements:count('acknowledgements'),order:ids(),note101:note('101')?.value,note105:note('105')?.value,action101:ack('101')?.getAttribute('aria-label'),selected:detail()?.textContent.trim(),search:get('events-search')?.value,height:get('events-height')?.value,chevrons:get('events-chevrons')?.value,compact:checked(get('events-compact')),filled:checked(get('events-filled')),animated:checked(get('events-animated')),color:get('events-color')?.value});
 const title=id=>body(id)?.querySelector('.equipment-event-title');
 const backgroundPoint=id=>{const box=primary(id).getBoundingClientRect();return {x:box.left+14,y:box.top+box.height/2}};
 const hitTestable=(node,sampleNode=node,point=null)=>{
  const box=node?.getBoundingClientRect(),frame=get('equipment-events-frame')?.getBoundingClientRect(),sample=sampleNode?.getBoundingClientRect();
  const at=point??(sample?{x:sample.left+sample.width/2,y:sample.top+sample.height/2}:null);
  const hit=at?document.elementFromPoint(at.x,at.y):null;
  failureGeometry={node:node?.id,active:active()?.id,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,documentScrollTop:document.documentElement.scrollTop,samplePoint:at,sampleClass:sampleNode?.className,centerHit:hit?{id:hit.id,tag:hit.tagName,className:hit.className,label:hit.getAttribute('aria-label')}:null,pointerEvents:node?getComputedStyle(node).pointerEvents:null};
  return !!box&&box.width>0&&box.height>0&&box.left>=frame.left-1&&box.right<=frame.right+1&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };

 await wait('actual equipment queue and retained native fields mounted',()=>host()&&list()&&result()&&detail()&&['101','102','103','104'].every(id=>row(id)&&primary(id)&&body(id)&&note(id)&&ack(id)));
 await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(list().getAttribute('role')==='list'&&allRows().length===4&&ids().join(',')==='101,102,103,104'&&count('visible')===4&&count('total')===4&&count('selections')===0&&count('acknowledgements')===0,'initial actual list and event counters match four records');
 for(const id of ['101','102','103','104']){
  assert(row(id).getAttribute('role')==='listitem'&&primary(id).getAttribute('aria-label')==='Select EVT-'+id&&body(id).contains(note(id))&&body(id).contains(ack(id))&&!primary(id).contains(note(id))&&!primary(id).contains(ack(id))&&!primary(id).querySelector('button,input,select,a'),'native note/action controls are visible body siblings outside selection button '+id);
  const owned=primary(id).getAttribute('aria-labelledby');if(owned)assert(get(owned)&&body(id).contains(get(owned)),'authored content labeling resolves to actual row body');
 }
 assert(detail().getAttribute('role')==='region'&&detail().getAttribute('aria-label')==='Selected event'&&detail().textContent.includes('EVT-101')&&detail().textContent.includes('Motor 001')&&detail().textContent.includes('vibration high')&&detail().textContent.includes('Alarm')&&primary('101').getAttribute('aria-pressed')==='true','selected detail shows the actual initial Motor event');
 assert(primary('104').disabled&&note('104').disabled&&ack('104').disabled,'archived row and composed controls explicitly disabled');
 await focus(get('events-search'),'native filter input');note('104').focus();primary('104').click();ack('104').click();
 await wait('disabled row/actions cannot select or acknowledge',()=>count('selections')===0&&count('acknowledgements')===0&&primary('101').getAttribute('aria-pressed')==='true');
 assert(active()===get('events-search')&&note('104').value==='Archived record','disabled native note cannot take focus or mutate archived content');snapshots.semantics=snapshot();

 await focus(primary('102'),'ordinary event background and title');
 await wait('ordinary visible row text hit testing reaches actual primary selection button',()=>hitTestable(primary('102'),title('102')));hitFacts.wideText=failureGeometry.centerHit;
 await wait('ordinary row background hit testing reaches actual primary selection button',()=>hitTestable(primary('102'),primary('102'),backgroundPoint('102')));hitFacts.wideBackground=failureGeometry.centerHit;
 for(const [id,value] of [['102','Enter'],['101',' ']]){
  await focus(primary(id),'native event '+id+' selection');const before=count('selections'),event=key(primary(id),value);keyPermissions['selection'+value]=!event.defaultPrevented;
  assert(!event.defaultPrevented,'selection '+value+' native default is not hijacked');await wait('untrusted native key does not fabricate selection',()=>count('selections')===before);
  await select(id);
 }
 assert(primary('101').getAttribute('aria-pressed')==='true'&&primary('102').getAttribute('aria-pressed')==='false'&&count('selections')===2,'each explicit selection invokes once and preserves controlled selected identity');snapshots.selection=snapshot();

 const retainedNote=note('101'),retainedOther=note('102');await focus(note('101'),'actual operator note');await wait('operator note native hit testing bypasses selection surface',()=>hitTestable(note('101')));hitFacts.note=failureGeometry.centerHit;
 for(const value of ['Home','End','ArrowLeft',' ']){const event=key(note('101'),value);keyPermissions['note'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'operator note '+value+' retains native key defaults')}
 await setInput(note('101'),'Check bearing seal','operator note');assert(count('selections')===2&&count('acknowledgements')===0,'local note edit does not select or acknowledge event');
 await focus(ack('101'),'native acknowledgement action');await wait('acknowledgement native hit testing bypasses selection surface',()=>hitTestable(ack('101')));hitFacts.acknowledgement=failureGeometry.centerHit;const ackKey=key(ack('101'),'Enter');keyPermissions.ackEnter=!ackKey.defaultPrevented;assert(!ackKey.defaultPrevented,'native acknowledgement Enter retains default');
 await acknowledge('101',true);assert(/\backnowledged\b/i.test(detail().textContent)&&!detail().textContent.toLowerCase().includes('unacknowledged'),'selected detail reflects actual acknowledged record');
 await acknowledge('101',false);assert(detail().textContent.includes('Awaiting acknowledgement')&&note('101').value==='Check bearing seal','Reopen restores actual awaiting-acknowledgement record without losing local note');snapshots.independentControls=snapshot();

 await setInput(get('events-search'),'Pump','filter queue');await wait('filter displays only matching retained Pump event',()=>count('visible')===1&&ids().join(',')==='102');
 assert(note('101')===retainedNote&&retainedNote.isConnected&&note('101').value==='Check bearing seal'&&note('101').getClientRects().length===0&&note('102')===retainedOther,'filtered local note scopes stay mounted and hidden');
 await setInput(get('events-search'),'No matching equipment','empty event filter');await wait('empty actual queue presents EmptyState and Clear filter',()=>count('visible')===0&&ids().length===0&&host().querySelector('.ix-empty-state')&&get('events-clear'));
 get('events-clear').click();await wait('Clear filter restores actual records and retained note',()=>get('events-search').value===''&&count('visible')===4&&ids().join(',')==='101,102,103,104'&&note('101')===retainedNote&&note('101').value==='Check bearing seal');
 get('events-reverse').click();await wait('keyed event reverse retains local note and changes actual DOM order',()=>ids().join(',')==='104,103,102,101'&&note('101')===retainedNote&&note('101').value==='Check bearing seal');
 get('events-reverse').click();await wait('second reverse restores original display order',()=>ids().join(',')==='101,102,103,104');snapshots.filterReverse=snapshot();

 const options=get('events-options');if(!options.open)options.querySelector('summary').click();await wait('native queue appearance details opens',()=>options.open);
 await choice('events-chevrons','none');await wait('shared chevron override hides actual row chevrons',()=>allRows().every(node=>!node.querySelector('.ix-event-chevron')));
 await choice('events-chevrons','per-row');await wait('per-row chevrons restore authored mixed settings',()=>!!chevron('101')&&!chevron('102')&&!!chevron('103')&&!chevron('104'));
 await choice('events-chevrons','all');await wait('shared all chevrons renders actual decorative icons',()=>allRows().every(node=>node.querySelector('.ix-event-chevron[aria-hidden="true"] svg')));
 get('events-compact').click();await wait('compact list option applies actual zero-gap layout',()=>checked(get('events-compact'))&&list().getAttribute('data-compact')==='true'&&parseFloat(getComputedStyle(list()).rowGap)===0);
 await choice('events-height','small');await wait('small height keeps actual selection/action usable and hides retained notes',()=>Math.abs(row('101').getBoundingClientRect().height-40)<=1&&note('101')===retainedNote&&note('101').getClientRects().length===0&&ack('101').getClientRects().length>0&&!ack('101').disabled);
 await choice('events-height','custom');await wait('custom event height and retained note restored',()=>Math.abs(row('101').getBoundingClientRect().height-128)<=1&&note('101')===retainedNote&&note('101').getClientRects().length>0&&note('101').value==='Check bearing seal');
 await choice('events-height','large');await wait('large event height restored',()=>Math.abs(row('101').getBoundingClientRect().height-104)<=1);
 get('events-filled').click();await wait('filled rows reach actual native primary surfaces',()=>checked(get('events-filled'))&&allRows().every(node=>node.querySelector('button.ix-event-list-item').getAttribute('data-filled')==='true'));
 assert(note('101')===retainedNote&&note('102')===retainedOther,'shared appearance options never remount local note scopes');snapshots.sharedOptions=snapshot();

 paint.severity=getComputedStyle(indicator('102')).backgroundColor;
 await choice('events-color','primary');await wait('custom primary indicator changes actual painted bar',()=>getComputedStyle(indicator('102')).backgroundColor!==paint.severity);paint.primary=getComputedStyle(indicator('102')).backgroundColor;
 await choice('events-color','info');await wait('custom info indicator changes actual painted bar',()=>getComputedStyle(indicator('102')).backgroundColor!==paint.primary);paint.info=getComputedStyle(indicator('102')).backgroundColor;
 assert(Object.values(paint).every(color=>color!=='transparent'&&color!=='rgba(0, 0, 0, 0)')&&Math.abs(indicator('102').getBoundingClientRect().width-8)<=1,'semantic and custom status bars have visible actual paint and retained8px indicator width');snapshots.indicatorPaint=snapshot();

 get('events-animated').click();await wait('optional list entrance animation enabled',()=>checked(get('events-animated')));
 const addButton=get('events-add');await focus(addButton,'native add/remove event toolbar button');addButton.click();
 await wait('actual extra event row and local note inserted without replacing toolbar',()=>count('total')===5&&count('visible')===5&&row('105')&&note('105')?.value==='Review cooling fan'&&get('events-add')===addButton&&addButton.textContent.includes('Remove'));
 const reduced=matchMedia('(prefers-reduced-motion:reduce)').matches;animation={reducedMotion:reduced,name:getComputedStyle(row('105')).animationName,animated:row('105').getAttribute('data-animated')};
 assert(animation.animated==='true'&&(reduced?animation.name==='none':animation.name!=='none'),'actual entering row CSS animation respects current platform reduced-motion preference');
 const transient=note('105');await focus(transient,'temporary event note');await setInput(transient,'Temporary cooling note','temporary event note');await acknowledge('105',true);
 await focus(addButton,'same native remove event button');addButton.click();await wait('removing extra event drops its owned note scope',()=>count('total')===4&&count('visible')===4&&!row('105')&&!transient.isConnected&&get('events-add')===addButton&&addButton.textContent.includes('Add'));
 addButton.click();await wait('readding event creates fresh actual default note/state',()=>count('total')===5&&note('105')&&note('105')!==transient&&note('105').value==='Review cooling fan'&&ack('105').getAttribute('aria-label')==='Acknowledge EVT-105');snapshots.dynamicAnimation=snapshot();

 const resetButton=get('events-reset'),oldNote=note('101'),oldExtra=note('105'),retainedSearch=get('events-search');await focus(resetButton,'native restore queue toolbar button');resetButton.click();
 await wait('Restore queue resets actual records, selected detail, options, counters and local notes',()=>count('total')===4&&count('visible')===4&&count('selections')===0&&count('acknowledgements')===0&&ids().join(',')==='101,102,103,104'&&note('101')?.value==='Check bearing housing'&&note('102')?.value==='Schedule service'&&note('103')?.value==='Calibration verified'&&note('104')?.value==='Archived record'&&!row('105')&&primary('101').getAttribute('aria-pressed')==='true'&&ack('101').getAttribute('aria-label')==='Acknowledge EVT-101'&&detail().textContent.includes('EVT-101')&&get('events-search').value===''&&get('events-height').value==='large'&&get('events-chevrons').value==='all'&&get('events-color').value==='severity'&&!checked(get('events-compact'))&&!checked(get('events-filled'))&&!checked(get('events-animated')));
 assert(!oldNote.isConnected&&!oldExtra.isConnected&&note('101')!==oldNote&&get('events-reset')===resetButton&&get('events-search')===retainedSearch&&get('events-add')===addButton&&active()===resetButton,'Restore replaces only owned list/row scopes and preserves toolbar native focus');snapshots.reset=snapshot();

 get('equipment-events-narrow').click();await wait('340px actual queue frame without horizontal page overflow',()=>get('equipment-events-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);
 await focus(note('101'),'narrow native operator note');await wait('narrow actual note fits and receives center hit testing',()=>hitTestable(note('101')));await setInput(note('101'),'Narrow bearing note','narrow operator note');
 await focus(ack('101'),'narrow native acknowledgement action');await wait('narrow acknowledgement fits and receives actual hit testing',()=>hitTestable(ack('101')));await acknowledge('101',true);
 await focus(primary('102'),'narrow native selection primary');await wait('narrow ordinary title hits actual primary selection surface',()=>hitTestable(primary('102'),title('102')));hitFacts.narrowText=failureGeometry.centerHit;await wait('narrow ordinary background hits actual primary selection surface',()=>hitTestable(primary('102'),primary('102'),backgroundPoint('102')));hitFacts.narrowBackground=failureGeometry.centerHit;await select('102');
 const frame=get('equipment-events-frame').getBoundingClientRect(),input=note('101').getBoundingClientRect(),action=ack('101').getBoundingClientRect();
 layout={frameWidth:frame.width,listWidth:list().getBoundingClientRect().width,rowHeight:row('101').getBoundingClientRect().height,inputWidth:input.width,inputHeight:input.height,actionWidth:action.width,actionHeight:action.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(note('101').value==='Narrow bearing note'&&ack('101').getAttribute('aria-label')==='Reopen EVT-101'&&detail().textContent.includes('EVT-102')&&layout.pageScroll<=layout.pageClient+1,'narrow independent native controls update actual data and keep page contained');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,paint,animation,hitFacts,
  checks:['actual list/listitem/native-button semantics with sibling controls and disabled guards','native selection Enter/Space permission and explicit single-click selected details','local operator note and acknowledgement/Reopen independent of row selection','retained native fields across filter/empty/clear and keyed reversal','shared chevrons, compact40/104/128 heights and filled states without note remount','actual semantic/custom indicator paint and8px width','optional CSS entrance animation/current reduced-motion and add/remove fresh local state','Restore resets records/owned scopes/options/counts while preserving toolbar focus','340px native note/action/selection hit testing and contained layout'],
  input:'synthetic form/click/key events, explicit native focus and native page scroll setup; native Enter/Space/Tab defaults are not simulated',
  trustedInput:'separate Chromium native Enter/Space/Tab/typing and pointer interactions; nested list context boundaries covered by Rust tests; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,paint,animation,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
