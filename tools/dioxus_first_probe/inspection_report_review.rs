//! Excluded actual-gallery inspection report filters and bounded native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

use crate::inspection_report as gallery_report;

#[component]
pub fn InspectionReportPreview() -> Element {
    let mut narrow = use_signal(|| false);
    let controller = use_date_dropdown();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let native_controller = controller.clone();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "inspection-report-review",
            h1 { "Local inspection report periods" }
            Button { id: "report-preview-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "report-preview-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_report::InspectionReportExample { controller: controller.clone() }
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    let controller = native_controller.clone();
                    spawn(async move {
                        let mut query = dioxus::document::eval(NATIVE_INSPECTION_REPORT_CHECK);
                        if let Err(error) = query.send("reportfilter") {
                            eprintln!("Inspection report native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        loop {
                            let message = match query.recv::<serde_json::Value>().await {
                                Ok(message) => message,
                                Err(error) => {
                                    eprintln!("Inspection report native driver channel failed: {error:?}");
                                    std::process::exit(1);
                                }
                            };
                            let reply = match message["kind"].as_str() {
                                Some("dateRange") => match controller.get_date_range() {
                                    Ok(range) => serde_json::json!({
                                        "kind": "dateRange",
                                        "id": range.id,
                                        "from": range.dates.from.map(|date| date.to_string()),
                                        "to": range.dates.to.map(|date| date.to_string()),
                                    }),
                                    Err(error) => {
                                        eprintln!("Inspection report range query failed: {error:?}");
                                        std::process::exit(1);
                                    }
                                },
                                Some("finished") => {
                                    let result = &message["result"];
                                    if result["ok"] != true {
                                        eprintln!("Dioxus native inspection report failed: {result}");
                                        std::process::exit(1);
                                    }
                                    if let Err(error) = query.send(serde_json::json!({ "kind": "finished" })) {
                                        eprintln!("Inspection report native driver acknowledgement failed: {error:?}");
                                        std::process::exit(1);
                                    }
                                    println!("Dioxus native inspection report: {result}");
                                    window.close();
                                    break;
                                },
                                _ => {
                                    eprintln!("Inspection report native driver sent an unknown request: {message}");
                                    std::process::exit(1);
                                }
                            };
                            if let Err(error) = query.send(reply) {
                                eprintln!("Inspection report native driver reply failed: {error:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Inspection report native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.inspection-report-review{box-sizing:border-box;min-height:100vh;padding:24px}.inspection-report-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}#report-preview-frame{margin-top:24px}@media(max-width:600px){.inspection-report-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_INSPECTION_REPORT_CHECK: &str = r#"
let snapshots={},geometry={},result;
try {
 await dioxus.recv();
 const byId=id=>document.getElementById(id),get=id=>byId('report-'+id),root=()=>get('workflow'),active=()=>document.activeElement;
 const status=()=>get('status'),saved=()=>Number(status()?.dataset.saved),form=()=>get('form'),submissions=()=>Number(form()?.dataset.submissions);
 const trigger=()=>get('period')?.matches('button')?get('period'):get('period')?.querySelector('button[aria-haspopup="dialog"]');
 const popup=()=>byId(trigger()?.getAttribute('aria-controls')),opened=()=>Boolean(trigger()?.getAttribute('aria-expanded')==='true'&&popup()&&!popup().hidden),ready=()=>Boolean(opened()&&popup().matches(':popover-open')&&getComputedStyle(popup()).visibility==='visible');
 const rows=()=>[...root()?.querySelectorAll('[id^="report-row-"]')??[]].map(node=>({id:node.id.slice('report-row-'.length),date:node.dataset.date,text:node.textContent.trim()}));
 const ids=()=>rows().map(row=>row.id),record=index=>get('record-'+index),recordFacts=index=>{const node=record(index);return node?{...node.dataset,text:node.textContent.trim()}:null};
 const preset=label=>[...popup()?.querySelectorAll('button.ix-date-preset')??[]].find(node=>node.textContent.trim()===label);
 const day=label=>popup()?.querySelector('button.ix-calendar-day[aria-label="'+label+'"]'),done=()=>[...popup()?.querySelectorAll('.ix-date-card-footer button')??[]].find(node=>node.textContent.trim()==='Done');
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.getAttribute('aria-label')||active()?.tagName)+'; status='+status()?.outerHTML+'; popup='+popup()?.outerHTML)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' actual native focus',()=>active()===node)};
 const click=id=>{assert(get(id),id+' control exists');get(id).click()};
 const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
 const input=async(id,value)=>{const node=get(id);assert(node,id+' actual native field exists');node.value=value;node.dispatchEvent(new Event('input',{bubbles:true}));await wait(id+' Rust-owned form input acknowledgement',()=>get(id)?.value===value&&(id==='name'?form()?.dataset.draftName===value:id==='note'?form()?.dataset.draftNote===value:true))};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const visible=node=>{if(!node||node.closest('[hidden]'))return false;const r=node.getBoundingClientRect(),s=getComputedStyle(node);return r.width>0&&r.height>0&&s.display!=='none'&&s.visibility!=='hidden'};
 const hit=node=>{if(!visible(node))return false;const r=node.getBoundingClientRect();if(r.left< -1||r.right>innerWidth+1||r.top< -1||r.bottom>innerHeight+1)return false;const target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const range=()=>({from:status()?.dataset.from,to:status()?.dataset.to,id:status()?.dataset.rangeId});
 const queryRange=async(expected,label)=>{const before={range:range(),ids:ids().join(','),open:opened(),focus:active(),saved:saved(),submissions:submissions()};dioxus.send({kind:'dateRange'});const snapshot=await dioxus.recv();assert(snapshot?.kind==='dateRange'&&snapshot.id===expected.id&&snapshot.from===expected.from&&snapshot.to===expected.to,label+' reads exact Rust draft');assert(JSON.stringify(range())===JSON.stringify(before.range)&&ids().join(',')===before.ids&&opened()===before.open&&active()===before.focus&&saved()===before.saved&&submissions()===before.submissions,label+' leaves published range, rows, popup, focus and reports unchanged');return snapshot};
 const capture=()=>({range:range(),rows:rows(),saved:saved(),submissions:submissions(),name:get('name')?.value,note:get('note')?.value,lock:status()?.dataset.lock,preparation:get('preparation')?.value,expanded:trigger()?.getAttribute('aria-expanded'),nativeTopLayer:ready(),records:Array.from({length:saved()},(_,index)=>recordFacts(index)),focus:active()?.id||active()?.getAttribute('aria-label')||active()?.tagName});
 const open=async()=>{await focus(trigger(),'actual DateDropdown trigger');if(!opened())trigger().click();await wait('actual DateDropdown dialog enters shared native top layer',()=>ready()&&popup()?.querySelector('.ix-calendar-grid'));assert(trigger().getAttribute('aria-haspopup')==='dialog'&&popup().getAttribute('role')==='dialog','actual trigger and surface expose matching dialog semantics')};
 const escape=async()=>{const node=popup()?.querySelector('button.ix-calendar-day[tabindex="0"]')||popup();await focus(node,'actual DateDropdown Escape target');key(node,'Escape');await wait('Escape closes shared surface and returns actual trigger focus',()=>!opened()&&active()===trigger())};
 const choose=async(label,from,to,expectedIds)=>{await open();const node=preset(label);assert(node&&!node.disabled,'actual '+label+' preset exists');await focus(node,'actual '+label+' preset');node.click();await wait(label+' immediately filters actual inspection records without closing',()=>ready()&&status().dataset.from===from&&status().dataset.to===to&&ids().join(',')===expectedIds.join(','));return {range:range(),ids:ids()}};
 const save=async(name,note,index,expectedRange,expectedIds)=>{await input('name',name);await input('note',note);assert(get('name').required&&get('name').validity.valid,'native report-name constraint satisfied');const payload=Object.fromEntries(new FormData(form()).entries());assert(payload['report-name']===name&&payload['report-note']===note,'actual native FormData contains exact report name and multiline note');const before=submissions();await focus(get('save'),'actual Save local report');click('save');await wait('actual immutable local report snapshot saved',()=>saved()===index+1&&submissions()===before+1&&record(index)?.dataset.name===name&&record(index)?.dataset.note===note);const result=recordFacts(index);assert(result.from===expectedRange.from&&result.to===expectedRange.to&&result.rangeId===expectedRange.id&&result.inspectionIds===expectedIds.join(','),'saved snapshot contains current committed range and exact matching inspection IDs');const submitted=JSON.parse(get('payload').textContent);assert(submitted['report-name']===name&&submitted['report-note']===note,'actual Rust FormEvent payload agrees with native successful controls');assert(record(index).textContent.includes(name)&&record(index).textContent.includes(note),'visible report snapshot contains actual submitted name and exact note');return {payload,snapshot:result}};
 await wait('actual inspection report workflow mounted',()=>root()&&status()&&form()&&trigger()&&get('toolbar')&&get('name')&&get('note')&&get('save')&&get('reset')&&get('lock')&&get('preparation'));
 await wait('native document focus',()=>document.hasFocus());
 assert(range().from==='2026-10-14'&&range().to==='2026-10-20'&&range().id==='recent'&&ids().join(',')==='sensor,compressor,valve'&&saved()===0,'actual recent period filters the dated inspection seed records');snapshots.initial=capture();snapshots.initialQuery=await queryRange(range(),'initial closed controller query');
 await open();
 // Native replay starts outside interactions after the existing per-owner document adapter subscribes.
 await wait('actual overlay owner subscribed to native document events',()=>{const owner=popup()?.closest('[data-ix-overlay-owner]')?.dataset.ixOverlayOwner;return owner&&document[Symbol.for('dioxus-ix.document-native-domain.v1')]?.events?.receivers.has(owner)});
 geometry.initial={toolbar:rect(get('toolbar')),popup:rect(popup()),trigger:rect(trigger()),viewport:{width:innerWidth,height:innerHeight},toolbarOverflow:getComputedStyle(get('toolbar')).overflow};assert(geometry.initial.popup.bottom>geometry.initial.toolbar.bottom+8,'native shared popup escapes actual clipped toolbar');assert(getComputedStyle(get('toolbar')).overflowX==='hidden'||getComputedStyle(get('toolbar')).overflowY==='hidden','actual toolbar is a clipping host');assert(!root().querySelector('.ix-temporal-backdrop'),'shared DateDropdown uses no private temporal backdrop');await escape();await open();await focus(get('preparation'),'outside report preparation');root().querySelector('h2').click();await wait('outside actual click dismisses shared dialog without stealing destination focus',()=>!opened()&&active()===get('preparation'));snapshots.sharedSurface=capture();

 await input('name','Unsubmitted inspection summary');await input('note','Keep this private report draft across period changes.');await choose('Today','2026-10-20','2026-10-20',['valve']);await choose('All inspections','2026-10-01','2026-10-20',['pump','motor','sensor','compressor','valve']);await choose('Recent inspections','2026-10-14','2026-10-20',['sensor','compressor','valve']);
 const day12=day('12 October 2026'),day13=day('13 October 2026');assert(day12&&day13&&day('18 October 2026'),'actual October calendar dates are present');await focus(day12,'actual October calendar keyboard day');key(day12,'ArrowRight');await wait('dialog does not consume calendar ArrowRight navigation',()=>active()===day13);day12.click();await wait('actual calendar starts custom range draft',()=>day('12 October 2026')?.getAttribute('aria-selected')==='true');day('18 October 2026').click();await wait('actual calendar completes selected range draft',()=>day('18 October 2026')?.getAttribute('aria-selected')==='true');assert(range().from==='2026-10-14'&&range().to==='2026-10-20'&&ids().join(',')==='sensor,compressor,valve','calendar selection remains unpublished until dismissal/Done');snapshots.unpublishedDraftQuery=await queryRange({id:'custom',from:'2026-10-12',to:'2026-10-18'},'unpublished custom controller query');assert(ready(),'query preserves actual open native dialog');assert(done(),'actual DateDropdown Done action exists');done().click();await wait('Done publishes actual custom range and filtered inspection IDs',()=>!opened()&&range().from==='2026-10-12'&&range().to==='2026-10-18'&&range().id==='custom'&&ids().join(',')==='motor,sensor,compressor'&&active()===trigger());assert(get('name').value==='Unsubmitted inspection summary'&&get('note').value==='Keep this private report draft across period changes.','real private report fields survive presets/calendar/shared surface changes');await open();await escape();assert(ids().join(',')==='motor,sensor,compressor','normal reopen/Escape preserves actual committed custom filter');snapshots.periodsAndCalendar=capture();snapshots.committedQuery=await queryRange(range(),'committed closed controller query');

 await input('name','');let before=submissions();click('save');await hold(80);assert(get('name').validity.valueMissing&&submissions()===before&&saved()===0,'native required report name prevents form handler and saved snapshot');const firstRange=range(),firstIds=ids();const first=await save('Local interval review','Inspect local handover.\nKeep valve A & valve B isolated.',0,firstRange,firstIds);const immutableFirst=JSON.stringify(recordFacts(0));await choose('Today','2026-10-20','2026-10-20',['valve']);await escape();const second=await save('Today inspection summary','Only the dated valve record belongs in this snapshot.',1,range(),ids());assert(JSON.stringify(recordFacts(0))===immutableFirst&&record(1)?.dataset.inspectionIds==='valve','changing filter and saving preserves prior immutable range/record snapshot');snapshots.nativeReports={...capture(),first,second};

 await input('preparation','Keep independent inspection-report preparation.');await input('name','Private locked-period report draft');await input('note','Preserve this note while the period is locked.');click('lock');await wait('actual period lock disables native trigger',()=>status().dataset.lock==='true'&&trigger().disabled);trigger().click();assert(!opened()&&get('name').value==='Private locked-period report draft'&&get('note').value==='Preserve this note while the period is locked.'&&!get('save').disabled,'locking period guards dropdown while preserving editable report fields');await focus(get('reset'),'actual scoped report Reset');click('reset');await wait('Reset clears owned report fields/snapshots and restores recent filter',()=>saved()===0&&submissions()===0&&get('name').value===''&&get('note').value===''&&range().id==='recent'&&range().from==='2026-10-14'&&range().to==='2026-10-20'&&ids().join(',')==='sensor,compressor,valve');assert(get('preparation').value==='Keep independent inspection-report preparation.'&&status().dataset.lock==='true'&&trigger().disabled,'scoped Reset preserves independent preparation and period lock');click('lock');await wait('normal period editing restored',()=>status().dataset.lock==='false'&&!trigger().disabled);snapshots.lockAndReset=capture();snapshots.resetQuery=await queryRange(range(),'reset closed controller query');

 byId('report-preview-narrow').click();await wait('actual340px report frame',()=>byId('report-preview-frame').getBoundingClientRect().width<=341);geometry.narrow={viewport:{width:innerWidth,height:innerHeight},frame:rect(byId('report-preview-frame')),controls:{}};
 for(const [label,node]of [['preparation',get('preparation')],['period',trigger()],['lock',get('lock')],['name',get('name')],['note',get('note')],['save',get('save')],['reset',get('reset')]]){node.scrollIntoView({block:'nearest',inline:'nearest'});await wait('narrow '+label+' actual center hit',()=>hit(node));const r=node.getBoundingClientRect(),f=byId('report-preview-frame').getBoundingClientRect();assert(r.left>=f.left-1&&r.right<=f.right+1,'narrow '+label+' control fits actual frame');geometry.narrow.controls[label]={rect:rect(node),hit:hit(node)}}
 trigger().scrollIntoView({block:'start',inline:'nearest'});await open();geometry.narrow.popup={rect:rect(popup()),toolbar:rect(get('toolbar')),trigger:rect(trigger())};const p=popup().getBoundingClientRect();assert(p.left>=-1&&p.right<=innerWidth+1&&p.top>=-1&&p.bottom<=innerHeight+1,'narrow native DateDropdown popup fits actual viewport');const all=preset('All inspections');await wait('narrow actual all-inspections preset receives native hit',()=>hit(all));all.click();await wait('narrow actual preset filters all five inspection records',()=>ready()&&ids().join(',')==='pump,motor,sensor,compressor,valve');const narrowDone=done();await wait('narrow actual Done receives native hit',()=>hit(narrowDone));narrowDone.click();await wait('narrow Done closes native shared dialog',()=>!opened()&&active()===trigger());const narrow=await save('Narrow local report','Actual narrow-frame report.\nRetain all five dated inspection IDs.',0,range(),ids());snapshots.narrow={...capture(),report:narrow};
 result={ok:true,snapshots,geometry,checks:['Rust controller reads initial, unpublished custom, committed and reset ranges without changing published rows, popup, focus or saved reports','actual DateDropdown dialog/trigger semantics and shared native top layer escape clipped toolbar; Escape/outside dismiss with useful focus','real presets filter dated records immediately; calendar keyboard/range draft and Done publish exact committed rows while private report drafts survive','native required name blocks Save; exact FormData creates immutable snapshots of current range/actual inspection IDs','period lock guards native trigger and scoped Reset restores owned range/forms/reports while preserving preparation/lock','actual340px frame has hit-testable contained controls, viewport-contained native popup and working all-inspections preset/Done/report Save'],input:'Excluded native DOM input/focus/click and synthetic calendar ArrowRight/Escape exercise actual gallery handlers, native constraint validation, successful FormData and shared native popovers. No physical pointer, trusted key default action, source callback-count equivalence or millisecond chronology is claimed.',viewportQualification:'340px narrows actual gallery frame inside unchanged desktop WebKitGTK viewport; native popup containment uses actual viewport. This native replay does not verify a true narrow viewport or trusted input.',renderer:navigator.userAgent};
}catch(error){const period=document.getElementById('report-period'),trigger=period?.matches('button')?period:period?.querySelector('button[aria-haspopup="dialog"]'),popup=document.getElementById(trigger?.getAttribute('aria-controls'));result={ok:false,error:String(error),snapshots,geometry,focus:document.activeElement?.id||document.activeElement?.getAttribute('aria-label')||document.activeElement?.tagName,status:document.getElementById('report-status')?.outerHTML,form:document.getElementById('report-form')?.outerHTML,trigger:trigger?.outerHTML,popup:popup?.outerHTML,viewport:{width:innerWidth,height:innerHeight}}}
dioxus.send({kind:'finished',result});
await dioxus.recv();
return result;
"#;
