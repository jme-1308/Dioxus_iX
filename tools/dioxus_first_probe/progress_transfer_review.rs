//! Excluded actual-gallery transfer progress preview and bounded native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/progress_transfer.rs"]
mod gallery_transfer;

#[component]
pub fn ProgressTransferPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "progress-transfer-review",
            h1 { "Transfer progress workflow" }
            Button { id: "progress-transfer-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "progress-transfer-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_transfer::ProgressTransferExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_PROGRESS_TRANSFER_CHECK);
                        if let Err(error) = query.send("progresstransfer") {
                            eprintln!("Progress transfer native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native progress transfer: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native progress transfer failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Progress transfer native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.progress-transfer-review{box-sizing:border-box;min-height:100vh;padding:24px}.progress-transfer-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}#progress-transfer-frame{margin-top:24px}@media(max-width:600px){.progress-transfer-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_PROGRESS_TRANSFER_CHECK: &str = r#"
let snapshots={},geometry={},tooltipFacts={},keys={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById('transfer-'+id),stage=()=>document.getElementById('progress-transfer'),active=()=>document.activeElement;
 const progress=()=>get('progress'),bar=()=>progress()?.querySelector('[role="progressbar"]');
 const status=()=>get('status'),rows=()=>[...get('records')?.querySelectorAll('[id^="transfer-record-"]')??[]];
 const records=()=>rows().map(node=>({id:node.id,units:Number(node.dataset.units),text:node.textContent.trim()}));
 const helper=()=>progress()?.getAttribute('aria-describedby')?.split(/\s+/).map(id=>document.getElementById(id)).find(Boolean);
 const tooltip=()=>helper()?.getAttribute('role')==='tooltip'?helper():null;
 const tooltipOpen=()=>tooltip()?.dataset.open==='true'&&tooltip()?.matches(':popover-open')===true;
 const processed=()=>Number(status()?.dataset.processed),completed=()=>Number(status()?.dataset.completed);
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.tagName)+'; status='+status()?.outerHTML)};
 const click=id=>{assert(get(id),id+' control exists');get(id).click()};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' actual native focus',()=>active()===node)};
 const input=async(id,value)=>{assert(get(id),id+' field exists');get(id).value=value;get(id).dispatchEvent(new Event('input',{bubbles:true}));await wait(id+' controlled value',()=>get(id)?.value===value)};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const visible=node=>{if(!node||node.closest('[hidden]'))return false;const r=node.getBoundingClientRect(),s=getComputedStyle(node);return r.width>0&&r.height>0&&s.display!=='none'&&s.visibility!=='hidden'};
 const hit=node=>{if(!visible(node))return false;const r=node.getBoundingClientRect();if(r.left< -1||r.right>innerWidth+1||r.top< -1||r.bottom>innerHeight+1)return false;const target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const near=(actual,expected,tolerance=.1)=>Number.isFinite(actual)&&Math.abs(actual-expected)<=tolerance;
 const paint=()=>{const track=bar(),fill=progress()?.querySelector('.ix-progress-fill');return {aria:Number(track?.getAttribute('aria-valuenow')),linearPercent:track?.clientWidth?100*fill.getBoundingClientRect().width/track.getBoundingClientRect().width:NaN,track:rect(track),fillColor:fill?getComputedStyle(fill).backgroundColor:null}};
 const percent=async(units,label)=>wait(label+' actual normalized ARIA and rendered progress paint',()=>{const p=paint(),expected=100*units/12;return near(p.aria,expected)&&near(p.linearPercent,expected,1)});
 const capture=()=>({processed:processed(),total:Number(status()?.dataset.total),completed:completed(),paused:status()?.dataset.paused,failed:status()?.dataset.failed,mode:status()?.dataset.mode,records:records(),note:get('note')?.value,preparation:get('preparation')?.value,paint:paint(),helperId:helper()?.id,helperRole:helper()?.getAttribute('role'),focus:active()?.id||active()?.tagName});
 const openOptions=()=>{const details=get('tooltip-mode')?.closest('details');if(details&&!details.open)details.querySelector('summary').click()};
 const mode=async enabled=>{openOptions();if(get('tooltip-mode').checked!==enabled)click('tooltip-mode');await wait('actual '+(enabled?'tooltip':'inline')+' helper mode',()=>status()?.dataset.mode===(enabled?'tooltip':'inline'))};
 await wait('actual transfer gallery and native controls mounted',()=>stage()&&progress()&&bar()&&status()&&get('advance')&&get('note')&&get('preparation'));
 await wait('native document focus',()=>document.hasFocus());
 assert(processed()===0&&completed()===0&&rows().length===0,'initial actual batch records are empty');
 assert(stage().querySelectorAll('[id^="transfer-task-"]').length===3,'three actual queued batches are displayed');
 assert(helper()?.classList.contains('ix-progress-helper-content')&&helper().contains(get('note'))&&helper().contains(get('advance')),'actual composed helper owns inline input and action');
 assert(progress().tabIndex===-1&&!progress().hasAttribute('title'),'inline progress retains source-default programmatic focus without native title');
 assert(bar().getAttribute('aria-valuemin')==='0'&&bar().getAttribute('aria-valuemax')==='100','semantic normalized bounds remain0–100');
 await percent(0,'initial');geometry.initial=paint();assert(near(geometry.initial.track.height,8),'actual default MD bar preserves existing progress.css8px height');snapshots.initial=capture();
 await input('preparation','Check the local transfer destination.');await input('note','Manifest checked for equipment batch.');
 click('advance');await wait('first actual equipment batch committed',()=>processed()===4&&completed()===1&&!!document.getElementById('transfer-record-equipment'));
 assert(records()[0].units===4&&records()[0].text.includes('Manifest checked for equipment batch.'),'actual first saved record contains units and helper note');
 await percent(4,'first batch');snapshots.firstBatch=capture();

 click('pause');await wait('actual pause disables processing',()=>status().dataset.paused==='true'&&get('advance').disabled);
 click('advance');assert(rows().length===1&&processed()===4,'disabled native Advance cannot append a paused batch');
 assert(get('pause').textContent.includes('Resume'),'actual helper offers Resume');click('pause');await wait('helper Resume restores processing',()=>status().dataset.paused==='false'&&!get('advance').disabled);
 click('fail');await wait('local failure shows actual Retry',()=>status().dataset.failed==='true'&&visible(get('retry'))&&!get('retry').disabled&&get('advance').disabled);
 assert(progress().classList.contains('ix-progress-status-error'),'failed progress uses semantic error paint');click('retry');
 await wait('helper Retry retains completed records and resumes',()=>status().dataset.failed==='false'&&!get('advance').disabled&&processed()===4&&rows().length===1);
 assert(get('note').value==='Manifest checked for equipment batch.','helper note survives pause/resume/failure/retry');snapshots.recovery=capture();

 await mode(true);assert(progress().tabIndex===0,'tooltip mode opts actual progress anchor into ordinary Tab navigation');
 await wait('shared Tooltip replaces inline helper once',()=>!!tooltip()&&stage().querySelectorAll('#transfer-note').length===1);
 assert(!tooltip().querySelector('button,input,textarea,select,a[href]')&&!tooltip().contains(get('note')),'Tooltip content is noninteractive; actual operational field remains outside');
 assert(bar().getAttribute('aria-description')?.trim(),'semantic progress keeps plain helper description');
 progress().scrollIntoView({block:'center',inline:'nearest'});await focus(progress(),'actual progress Tooltip anchor');
 await wait('focus opens actual shared styled Tooltip',()=>tooltipOpen()&&visible(tooltip()));
 const tooltipRect=tooltip().getBoundingClientRect();assert(tooltipRect.left>=-1&&tooltipRect.right<=innerWidth+1&&tooltipRect.top>=-1&&tooltipRect.bottom<=innerHeight+1,'shared Tooltip fits actual native viewport');
 tooltipFacts.open={id:tooltip().id,text:tooltip().textContent.trim(),description:progress().getAttribute('aria-describedby'),rect:rect(tooltip()),anchor:tooltip().dataset.anchor};
 click('pause');await wait('focused Tooltip and semantic status update live',()=>status().dataset.paused==='true'&&tooltipOpen()&&tooltip().textContent.toLowerCase().includes('paused'));
 assert(active()===progress(),'excluded DOM action click leaves Tooltip anchor focus intact');
 tooltipFacts.live={id:tooltip().id,text:tooltip().textContent.trim(),plain:bar().getAttribute('aria-description')};
 const escape=new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,cancelable:true});progress().dispatchEvent(escape);
 await wait('synthetic Escape hides shared Tooltip without moving anchor focus',()=>!tooltipOpen()&&active()===progress());keys.escape={synthetic:true,defaultPrevented:escape.defaultPrevented};
 const oldTooltip=tooltip();await mode(false);
 await wait('mode change removes actual Tooltip and restores inline description',()=>!oldTooltip.isConnected&&!tooltip()&&helper()?.classList.contains('ix-progress-helper-content')&&helper().contains(get('note')));
 assert(get('note').value==='Manifest checked for equipment batch.','parent-owned helper note survives presentation change');
 tooltipFacts.cleanup={retiredTooltipConnected:oldTooltip.isConnected,helperId:helper().id,role:helper().getAttribute('role'),description:progress().getAttribute('aria-describedby')};
 click('pause');await wait('Resume after Tooltip workflow',()=>status().dataset.paused==='false'&&!get('advance').disabled);snapshots.tooltip=capture();

 await input('note','Measurements checked locally.');click('advance');await wait('actual measurement batch committed',()=>processed()===9&&completed()===2&&!!document.getElementById('transfer-record-measurements'));
 assert(records()[1].units===5&&records()[1].text.includes('Measurements checked locally.'),'second actual record saves five units and current helper note');await percent(9,'second batch');
 await input('note','Service notes reviewed locally.');click('advance');await wait('actual service notes complete transfer',()=>processed()===12&&completed()===3&&rows().length===3);
 assert(records()[2].units===3&&records()[2].text.includes('Service notes reviewed locally.'),'third actual record saves three units and current helper note');
 assert(records().reduce((sum,row)=>sum+row.units,0)===12&&progress().classList.contains('ix-progress-status-success'),'completion is derived from real recorded units');
 await percent(12,'completed');assert(get('advance').disabled&&get('pause').disabled,'completed transfer blocks further process/pause');snapshots.completed=capture();
 await mode(true);click('reset');await wait('owned Reset clears records/progress/helper note',()=>processed()===0&&completed()===0&&rows().length===0&&status().dataset.failed==='false'&&status().dataset.paused==='false'&&get('note').value==='');
 assert(status().dataset.mode==='tooltip'&&get('preparation').value==='Check the local transfer destination.','Reset retains display preference and independent preparation');await percent(0,'reset');snapshots.reset=capture();

 document.getElementById('progress-transfer-narrow').click();await wait('actual preview frame340px',()=>document.getElementById('progress-transfer-frame').getBoundingClientRect().width<=341);
 await mode(false);const frame=document.getElementById('progress-transfer-frame');geometry.narrow={viewport:{width:innerWidth,height:innerHeight},frame:rect(frame),controls:{}};
 for(const id of ['note','advance','pause','fail','reset','preparation']){const node=get(id);assert(node,id+' remains present at narrow frame');node.scrollIntoView({block:'nearest',inline:'nearest'});geometry.narrow.controls[id]={rect:rect(node),hit:hit(node)};await wait('narrow '+id+' native center hit',()=>hit(node));const r=node.getBoundingClientRect(),f=frame.getBoundingClientRect();assert(r.left>=f.left-1&&r.right<=f.right+1,'narrow '+id+' stays in actual frame');geometry.narrow.controls[id]={rect:rect(node),hit:hit(node)}}
 geometry.narrow.paint=paint();assert(near(geometry.initial.track.height,geometry.narrow.paint.track.height),'ordinary bar height survives helper and frame changes');
 assert(geometry.narrow.paint.track.width>0,'actual progress bar remains visible at narrow frame');
 snapshots.final=capture();
 return {ok:true,snapshots,geometry,tooltipFacts,keys,checks:['actual batches4+5+3 units and saved helper notes derive ARIA/paint/completion','inline helper actions preserve note through pause/resume/failure/retry and enforce guards','shared styled Tooltip opens on native focus, updates live, Escape hides and mode unmount cleans description','owned Reset preserves independent preparation/display preference; actual340px frame fields/actions hit and bar geometry remains coherent'],input:'Excluded native DOM focus/input/clicks; Escape is an untrusted synthetic keydown delivered to the actual anchor. Tooltip visibility uses bounded predicates without millisecond timing assertions or fabricated hover callbacks.',viewportQualification:'340px narrows the actual gallery frame inside the unchanged desktop WebKitGTK viewport; lead Chromium verifies a true narrow viewport and trusted pointer/Tab/Escape. Hover-only opening is covered by the shared Tooltip synthetic-event regression.',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,geometry,tooltipFacts,keys,focus:document.activeElement?.id||document.activeElement?.tagName,status:document.getElementById('transfer-status')?.outerHTML,viewport:{width:innerWidth,height:innerHeight}}}

"#;
