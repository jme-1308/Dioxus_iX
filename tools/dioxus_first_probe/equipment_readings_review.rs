//! Excluded actual-gallery equipment readings preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/equipment_readings.rs"]
mod gallery_readings;

#[component]
pub fn EquipmentReadingsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_readings::STYLES} }
        IxTheme { class: "equipment-readings-review",
            h1 { "Equipment readings" }
            Button { id: "equipment-readings-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "equipment-readings-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_readings::EquipmentReadingsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_EQUIPMENT_READINGS_CHECK);
                        if let Err(error) = query.send("equipmentreadings") {
                            eprintln!("Equipment readings native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native equipment readings: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native equipment readings failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Equipment readings native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.equipment-readings-review{box-sizing:border-box;min-height:100vh;padding:24px}.equipment-readings-review>h1{font-size:28px;margin:0 0 24px}.equipment-readings-review h2{font-size:22px;margin:24px 0 16px}.equipment-readings-review p{max-width:72ch;line-height:1.5}#equipment-readings-frame{margin-top:24px}@media(max-width:600px){.equipment-readings-review{padding:16px}}";

// Excluded actual-gallery replay; explicit native focus and synthetic events.
// Trusted typing/Tab/default activation are verified separately by the lead.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_EQUIPMENT_READINGS_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('readings-result');
 const empty=()=>get('readings-empty'),action=()=>empty()?.querySelector('button'),note=()=>get('readings-note');
 const ids=['motor-speed','pump-temperature','fan-vibration'],kpi=id=>get('readings-kpi-'+id),row=id=>get('readings-record-'+id);
 const count=name=>Number(result()?.dataset[name]),pending=()=>result()?.dataset.pending==='true';
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; active='+(active()?.id||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const setSelect=async(id,value)=>{const node=get(id);node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' native selection applied',()=>node.value===value)};
 const setChecked=async(id,value)=>{const node=get(id);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' explicit form setup applied',()=>node.checked===value)};
 const setNote=async text=>{const node=note();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual local preparation draft updated',()=>note()===node&&node.value===text)};
 const hitTestable=(node,insideFrame=true)=>{
  const box=node?.getBoundingClientRect(),frame=get('equipment-readings-frame')?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  failureGeometry={node:node?.id||node?.tagName,active:active()?.id,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,scrollTop:document.documentElement.scrollTop,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className}:null};
  return !!box&&box.width>0&&box.height>0&&(!insideFrame||(box.left>=frame.left-1&&box.right<=frame.right+1))&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };
 const snapshot=()=>({pending:pending(),records:count('recordCount'),visible:count('visibleCount'),requests:count('loadRequests'),completions:count('completions'),cancellations:count('cancellations'),samples:count('samples'),filterClears:count('filterClears'),filter:result()?.dataset.filter,note:note()?.value,readings:ids.filter(id=>row(id)).map(id=>({id,value:row(id).dataset.value,state:row(id).dataset.state,unit:row(id).dataset.unit,title:kpi(id)?.title}))});
 const start=async()=>{const before=count('loadRequests');action().click();await wait('accepted recovery enters explicit pending state once',()=>pending()&&count('loadRequests')===before+1&&action()?.disabled&&action().getAttribute('aria-busy')==='true'&&!get('readings-complete').disabled&&!get('readings-cancel').disabled)};
 const complete=async()=>{const before=count('completions');get('readings-complete').click();await wait('Complete creates actual three local readings',()=>!pending()&&count('completions')===before+1&&count('recordCount')===3&&count('visibleCount')===3&&ids.every(id=>row(id)&&kpi(id)))};

 await wait('actual empty recovery and retained preparation mounted',()=>result()&&empty()&&action()&&note()&&get('readings-options'));
 await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(count('recordCount')===0&&count('visibleCount')===0&&!pending()&&['loadRequests','completions','cancellations','samples','filterClears'].every(name=>count(name)===0)&&action().getAttribute('aria-label')==='Load illustrative equipment readings'&&note().value==='Prepare line inspection','real initial empty model, recovery label and local preparation');
 assert(empty().querySelector('svg[role="img"][aria-label="Equipment readings"]')&&empty().querySelector('.ix-empty-state-labels strong')?.textContent==='No equipment readings'&&empty().querySelector('.ix-empty-state-labels p')?.textContent.includes('local example records'),'actual empty icon and explanation remain accessible');
 const options=get('readings-options');options.querySelector('summary').click();await wait('native reading options opened',()=>options.open);
 await setChecked('readings-disabled',true);await wait('disabled recovery reflects native disabled action',()=>action().disabled);action().click();await wait('disabled recovery rejects request without seeding data',()=>count('loadRequests')===0&&count('recordCount')===0&&!pending());await setChecked('readings-disabled',false);await wait('recovery restored',()=>!action().disabled);
 await focus(action(),'native recovery action');await wait('actual recovery action receives pointer hit',()=>hitTestable(action()));hitFacts.recovery=failureGeometry.hit;
 for(const value of ['Enter',' ']){const event=key(action(),value);keyPermissions['recovery'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native recovery '+value+' default remains available')}
 assert(count('loadRequests')===0,'untrusted native keys do not fabricate default activation');snapshots.emptyDisabled=snapshot();

 await start();const retainedNote=note();await focus(note(),'local preparation during pending load');await setNote('Prepare bearing inspection');const duplicate=count('loadRequests');action().click();await wait('busy recovery prevents duplicate requests without data',()=>pending()&&count('loadRequests')===duplicate&&count('recordCount')===0);
 get('readings-cancel').click();await wait('explicit Cancel ends pending without seeding readings',()=>!pending()&&count('cancellations')===1&&count('recordCount')===0&&!action().disabled&&get('readings-complete').disabled&&get('readings-cancel').disabled);
 await start();await complete();assert(note()===retainedNote&&note().value==='Prepare bearing inspection'&&count('loadRequests')===2&&count('cancellations')===1&&count('completions')===1,'explicit completion and cancellation retain unrelated preparation');snapshots.recovery=snapshot();

 assert(row('motor-speed').dataset.value==='122.6'&&row('motor-speed').dataset.state==='neutral'&&row('motor-speed').dataset.unit==='rpm'&&row('pump-temperature').dataset.value==='78.2'&&row('pump-temperature').dataset.state==='warning'&&row('pump-temperature').dataset.unit==='°C'&&row('fan-vibration').dataset.value==='1.4'&&row('fan-vibration').dataset.unit==='mm/s','actual seeded values, units and severity states');
 for(const id of ids){const node=kpi(id);assert(node.getAttribute('role')==='group'&&node.tabIndex===0&&!!node.getAttribute('aria-label')&&!!node.title&&node.title.includes(row(id).dataset.value),'native readable KPI group/title in ordinary Tab order');await focus(node,'actual native reading '+id);await wait('actual KPI receives pointer hit',()=>hitTestable(node));hitFacts[id]=failureGeometry.hit}
 assert(kpi('pump-temperature').querySelector('svg[role="img"][aria-label="Warning"]'),'warning severity icon has localized accessible default label');snapshots.nativeReadings=snapshot();

 await setSelect('readings-filter','alarm');await wait('actual alarm filter produces recoverable no-results state',()=>result().dataset.filter==='alarm'&&count('recordCount')===3&&count('visibleCount')===0&&empty()&&action()?.getAttribute('aria-label')==='Clear reading filter');const beforeFilter={requests:count('loadRequests'),completions:count('completions')};await focus(action(),'native Clear reading filter');await wait('real clear-filter action receives pointer hit',()=>hitTestable(action()));action().click();await wait('Clear filter restores actual records without new recovery',()=>result().dataset.filter==='all'&&count('visibleCount')===3&&count('recordCount')===3&&count('loadRequests')===beforeFilter.requests&&count('completions')===beforeFilter.completions&&count('filterClears')===1);assert(note()===retainedNote&&note().value==='Prepare bearing inspection','filter empty/recovery retains local preparation');snapshots.filterRecovery=snapshot();

 await focus(get('readings-sample'),'native Sample readings');get('readings-sample').click();await wait('Sample mutates actual values and cycles real severity',()=>count('samples')===1&&row('motor-speed').dataset.value==='123.6'&&row('pump-temperature').dataset.value==='79.2'&&row('fan-vibration').dataset.value==='1.6'&&row('motor-speed').dataset.state==='warning'&&row('pump-temperature').dataset.state==='alarm'&&row('fan-vibration').dataset.state==='warning');
 await setChecked('readings-german',true);await wait('localized warning/alarm icons update on actual readings',()=>kpi('motor-speed').querySelector('svg[role="img"][aria-label="Warnmeldung"]')&&kpi('pump-temperature').querySelector('svg[role="img"][aria-label="Alarmmeldung"]'));
 await setChecked('readings-vertical',true);await setChecked('readings-long-label',true);await wait('actual layout and long labels apply without recreating preparation',()=>result().dataset.vertical==='true'&&ids.every(id=>kpi(id).dataset.vertical==='true')&&result().dataset.longLabel==='true'&&note()===retainedNote&&note().value==='Prepare bearing inspection');
 assert(ids.every(id=>kpi(id).title.includes(row(id).dataset.value))&&kpi('motor-speed').title.includes('Motor'),'long labels retain full native reading titles');snapshots.sampleLocalization=snapshot();

 get('readings-clear').click();await wait('Clear readings restores actual empty state without dropping preparation',()=>count('recordCount')===0&&count('visibleCount')===0&&empty()&&action()?.getAttribute('aria-label')==='Load illustrative equipment readings'&&note()===retainedNote&&note().value==='Prepare bearing inspection');
 await setSelect('readings-layout','compact');await wait('actual compact empty presentation applied',()=>result().dataset.emptyLayout==='compact'&&empty().classList.contains('ix-empty-state-compact'));await setSelect('readings-layout','compact-break');await wait('actual compact-break empty presentation applied',()=>result().dataset.emptyLayout==='compact-break'&&empty().classList.contains('ix-empty-state-compact-break'));await focus(action(),'compact-break recovery');await wait('compact-break recovery remains usable',()=>hitTestable(action()));hitFacts.compactRecovery=failureGeometry.hit;
 for(const value of ['Home','ArrowLeft',' ','Tab']){await focus(note(),'native retained preparation');const event=key(note(),value);keyPermissions['note'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native preparation '+value+' default remains available')}
 assert(note()===retainedNote&&note().value==='Prepare bearing inspection'&&count('samples')===1,'clear and layout changes preserve independent local draft and historical model action counts');snapshots.emptyLayoutsRetention=snapshot();

 const reset=get('readings-reset'),oldNote=note(),filter=get('readings-filter');await focus(reset,'Restore equipment readings');reset.click();await wait('Restore resets actual records/options/counts and owned preparation',()=>count('recordCount')===0&&count('visibleCount')===0&&!pending()&&['loadRequests','completions','cancellations','samples','filterClears'].every(name=>count(name)===0)&&result().dataset.filter==='all'&&result().dataset.emptyLayout==='large'&&note()?.value==='Prepare line inspection'&&['readings-disabled','readings-vertical','readings-long-label','readings-german'].every(id=>!get(id).checked));
 assert(!oldNote.isConnected&&note()!==oldNote&&get('readings-reset')===reset&&get('readings-filter')===filter&&get('readings-options')===options&&active()===reset,'Restore replaces owned preparation while preserving toolbar/options/focus');snapshots.reset=snapshot();

 get('equipment-readings-narrow').click();await wait('actual 340px preview and page fit',()=>get('equipment-readings-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);await setSelect('readings-layout','compact-break');await setChecked('readings-long-label',true);
 await focus(action(),'narrow native recovery');await wait('narrow actual recovery fits and receives pointer hit',()=>hitTestable(action()));hitFacts.narrowRecovery=failureGeometry.hit;await start();await focus(get('readings-complete'),'narrow explicit Complete');await wait('narrow native Complete fits and receives hit',()=>hitTestable(get('readings-complete')));await complete();await setChecked('readings-vertical',true);
 await focus(note(),'narrow native preparation');await wait('narrow actual preparation fits and receives hit',()=>hitTestable(note()));hitFacts.narrowNote=failureGeometry.hit;await setNote('Narrow equipment inspection');
 for(const id of ids){await focus(kpi(id),'narrow actual reading '+id);await wait('narrow '+id+' KPI fits and is hit-testable',()=>hitTestable(kpi(id)))}
 await focus(get('readings-sample'),'narrow Sample readings');await wait('narrow actual Sample fits and receives hit',()=>hitTestable(get('readings-sample')));get('readings-sample').click();await wait('narrow actual sample changes saved reading values',()=>count('samples')===1&&row('motor-speed').dataset.value==='123.6'&&row('pump-temperature').dataset.state==='alarm'&&row('fan-vibration').dataset.value==='1.6');
 const frame=get('equipment-readings-frame').getBoundingClientRect(),input=note().getBoundingClientRect();layout={frameWidth:frame.width,kpis:ids.map(id=>({id,width:kpi(id).getBoundingClientRect().width,height:kpi(id).getBoundingClientRect().height,title:kpi(id).title,tabindex:kpi(id).tabIndex})),inputWidth:input.width,inputHeight:input.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(layout.kpis.every(item=>item.width<=341&&item.width>0)&&layout.inputHeight>=31&&layout.pageScroll<=layout.pageClient+1&&note().value==='Narrow equipment inspection'&&count('loadRequests')===1&&count('completions')===1&&count('cancellations')===0,'340px recovery/measurement/preparation workflow remains contained and changes actual records');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,hitFacts,
 checks:['actual initial empty recovery/native action key permission and disabled request guard','explicit pending/duplicate guard/Cancel/Complete seed actual readings without timing assumptions','native readable KPI groups/title/tabindex0/actual seed values and warning icon label','real empty filter clears filter without seeding or losing preparation','actual sample values/severity/localized icon labels and retained long-label layout','Clear readings and compact/compact-break recovery retain local preparation','Restore resets records/options/counts and owned preparation while preserving toolbar focus','340px actual recovery/Complete/KPI/preparation/Sample pointer hits and real record changes'],
 input:'synthetic form/click/key events, explicit native focus and normal page scroll setup; native typing/Enter/Space/Tab defaults are not simulated',trustedInput:'separate Chromium trusted typing/Tab/native activation and visual evidence; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
