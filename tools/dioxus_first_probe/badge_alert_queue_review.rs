//! Excluded actual-gallery equipment alert badges and bounded native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

use crate::badge_alert_queue as gallery_alerts;

#[component]
pub fn BadgeAlertQueuePreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "badge-alert-review",
            h1 { "Equipment alert badge descriptions" }
            Button { id: "badge-alert-preview-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "badge-alert-preview-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_alerts::BadgeAlertQueueExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_BADGE_ALERT_CHECK);
                        if let Err(error) = query.send("badgealerts") {
                            eprintln!("Badge alert native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native badge alerts: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native badge alerts failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Badge alert native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.badge-alert-review{box-sizing:border-box;min-height:100vh;padding:24px}.badge-alert-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}#badge-alert-preview-frame{margin-top:24px}@media(max-width:600px){.badge-alert-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_BADGE_ALERT_CHECK: &str = r#"
let snapshots={},geometry={};
try {
 await dioxus.recv();
 const byId=id=>document.getElementById(id),get=id=>byId('badge-alert-'+id),root=()=>get('workflow'),active=()=>document.activeElement;
 const status=()=>get('status'),pending=()=>Number(status()?.dataset.pending),reviewed=()=>Number(status()?.dataset.reviewed);
 const form=id=>get('form-'+id),note=id=>get('note-'+id),ack=id=>get('ack-'+id),review=id=>get('review-'+id),record=id=>get('record-'+id);
 const badge=()=>root()?.querySelector('.badge-alert-status-badge'),panel=()=>badge()?.querySelector('[role="tooltip"]'),attached=()=>get('open')?.closest('.ix-badge-attached');
 const open=()=>Boolean(panel()?.matches(':popover-open'));
 const ready=()=>Boolean(open()&&panel().dataset.positioned==='true'&&getComputedStyle(panel()).visibility==='visible'&&!panel().inert);
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.tagName)+'; status='+status()?.outerHTML+'; panel='+panel()?.outerHTML)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' actual native focus',()=>active()===node)};
 const click=id=>{assert(get(id),id+' control exists');get(id).click()};
 const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
 const hover=node=>node.dispatchEvent(new MouseEvent('mouseenter',{bubbles:false})),leave=node=>node.dispatchEvent(new MouseEvent('mouseleave',{bubbles:false}));
 const input=async(id,value)=>{const node=note(id);assert(node,id+' native alert note exists');node.value=value;node.dispatchEvent(new Event('input',{bubbles:true}));await wait(id+' Rust-owned note acknowledgement',()=>note(id)?.value===value&&form(id)?.dataset.draftNote===value)};
 const setAck=async(id,value)=>{if(ack(id).checked!==value)ack(id).click();await wait(id+' actual acknowledgement state',()=>ack(id).checked===value&&form(id)?.dataset.ack===String(value))};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const visible=node=>{if(!node||node.closest('[hidden]'))return false;const r=node.getBoundingClientRect(),s=getComputedStyle(node);return r.width>0&&r.height>0&&s.display!=='none'&&s.visibility!=='hidden'};
 const hit=node=>{if(!visible(node))return false;const r=node.getBoundingClientRect();if(r.left< -1||r.right>innerWidth+1||r.top< -1||r.bottom>innerHeight+1)return false;const target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const records=()=>[...root()?.querySelectorAll('[id^="badge-alert-record-"]')??[]].map(node=>({alert:node.dataset.alert,note:node.dataset.note,text:node.textContent.trim()}));
 const popupFacts=()=>({id:panel()?.id,text:panel()?.textContent.trim(),open:open(),ready:ready(),name:badge()?.getAttribute('aria-label'),role:badge()?.getAttribute('role'),describedby:badge()?.getAttribute('aria-describedby'),anchor:panel()?.dataset.anchor});
 const capture=()=>({pending:pending(),reviewed:reviewed(),open:status()?.dataset.open,detailed:status()?.dataset.detailed,preparation:get('preparation')?.value,records:records(),forms:Object.fromEntries(['pump','compressor','sensor'].map(id=>[id,{note:note(id)?.value,ack:ack(id)?.checked,reviewed:form(id)?.dataset.reviewed,submissions:form(id)?.dataset.submissions}])),popup:popupFacts(),focus:active()?.id||active()?.tagName});
 const descriptions=()=>{const tokens=(get('open').getAttribute('aria-describedby')||'').split(/\s+/).filter(Boolean);assert(tokens.includes('badge-alert-overview-help')&&tokens.includes('badge-alert-count-help')&&tokens.length===new Set(tokens).size&&tokens.every(id=>byId(id)),'actual Open alerts button keeps both unique existing/count descriptions');assert(document.querySelectorAll('#badge-alert-count-help').length===1,'attached count helper target is unique');assert(get('count-help').textContent.includes(String(pending())),'explicit attached count description reflects actual unread records');const indicator=attached()?.querySelector('.ix-badge-indicator');assert(attached()&&!attached().hasAttribute('tabindex')&&!attached().hasAttribute('role')&&!attached().hasAttribute('aria-label'),'attached root leaves native focus/role/name on real button');assert(indicator?.getAttribute('aria-hidden')==='true'&&!indicator.hasAttribute('role')&&!indicator.hasAttribute('aria-label'),'attached badge indicator is decorative');assert(badge()?.getAttribute('role')==='img'&&badge().tabIndex===0&&badge().getAttribute('aria-label'),'standalone badge has actual keyboard focus, explicit image role and useful name');assert(!badge().hasAttribute('title')&&!badge().querySelector('[title]'),'styled help has no duplicate native title');return {tokens,countHelp:get('count-help').textContent.trim(),buttonName:get('open').textContent.trim(),badge:popupFacts()}};
 const show=async()=>{await focus(get('preparation'),'outside preparation');await focus(badge(),'actual standalone status badge');await wait('standalone badge opens positioned native shared help',ready);assert(panel().dataset.anchor===badge().id,'actual native help uses Dioxus-owned badge root');const tokens=(badge().getAttribute('aria-describedby')||'').split(/\s+/);assert(tokens.includes(panel().id),'focused standalone badge describes current shared popup');return popupFacts()};
 const queue=async value=>{if(status().dataset.open!==String(value))click('open');await wait('actual '+(value?'open':'hidden')+' alert queue',()=>status().dataset.open===String(value)&&get('panel').hidden===!value)};
 const detail=async value=>{if(get('details').checked!==value)click('details');await wait('actual detailed-help preference',()=>status().dataset.detailed===String(value))};
 const save=async(id,value,expectedPending)=>{await input(id,value);await setAck(id,true);assert(ack(id).required&&ack(id).validity.valid,'actual acknowledgement constraint is satisfied before review');const payload=Object.fromEntries(new FormData(form(id)).entries());assert(payload['review-note']===value&&payload['review-ack']==='acknowledged','actual native FormData contains exact note and acknowledgement');await focus(review(id),'actual Review '+id);review(id).click();await wait('actual '+id+' saved review updates unread queue',()=>record(id)?.dataset.note===value&&pending()===expectedPending&&reviewed()===3-expectedPending&&form(id)?.dataset.reviewed==='true');const submitted=JSON.parse(get('payload').textContent)[id];assert(submitted?.['review-note']===value&&submitted?.['review-ack']==='acknowledged','actual Rust FormEvent stores exact native review values');assert(record(id).textContent.includes(value),'visible reviewed record contains exact multiline note');assert(note(id).disabled&&ack(id).disabled&&review(id).disabled,'saved review controls use native disabled guard');descriptions();return payload};
 await wait('actual alert queue workflow mounted',()=>root()&&status()&&get('open')&&get('panel')&&get('details')&&get('preparation')&&get('count-help')&&badge()&&form('pump')&&form('compressor')&&form('sensor'));
 await wait('native document focus',()=>document.hasFocus());
 assert(pending()===3&&reviewed()===0&&records().length===0&&status().dataset.detailed==='true','actual three unreviewed equipment alerts initialize detailed status help');snapshots.initial={...capture(),descriptions:descriptions()};
 await focus(get('open'),'actual Open alerts button');await queue(true);assert(visible(note('pump'))&&visible(ack('pump'))&&visible(review('pump')),'actual queue reveals native per-alert review controls');
 await input('pump','Pump draft remains private until reviewed.');await input('compressor','Compressor draft retained across badge count changes.');await setAck('compressor',true);await queue(false);assert(!visible(note('pump'))&&get('panel').hidden,'closed actual queue hides review controls');await queue(true);assert(note('pump').value==='Pump draft remains private until reviewed.'&&note('compressor').value==='Compressor draft retained across badge count changes.'&&ack('compressor').checked,'closing/reopening actual queue preserves private note/ack drafts');snapshots.openAndRetainedDrafts=capture();

 const firstHelp=await show();assert(firstHelp.text&&firstHelp.text!==firstHelp.name&&firstHelp.text.includes('3'),'custom detailed help describes actual initial unread count');geometry.badgeFocus={rect:rect(badge()),focusVisible:badge().matches(':focus-visible'),outlineStyle:getComputedStyle(badge()).outlineStyle,outlineWidth:getComputedStyle(badge()).outlineWidth,outlineColor:getComputedStyle(badge()).outlineColor};
 if(geometry.badgeFocus.focusVisible)assert(geometry.badgeFocus.outlineStyle!=='none'&&Number.parseFloat(geometry.badgeFocus.outlineWidth)>0,'native keyboard-visible badge focus has an actual outline');key(badge(),'Escape');await wait('Escape dismisses standalone badge native help',()=>!open());await show();await focus(get('preparation'),'outside native badge blur');await wait('badge blur dismisses actual help',()=>!open());hover(badge());await wait('synthetic hover opens actual shared badge help',ready);leave(badge());await wait('synthetic leave dismisses native badge help',()=>!open());snapshots.nativeHelp=capture();

 await setAck('pump',false);const beforeRecords=records().length,beforeSubmissions=form('pump').dataset.submissions;review('pump').click();await hold(80);assert(!ack('pump').validity.valid&&ack('pump').validity.valueMissing&&records().length===beforeRecords&&pending()===3&&form('pump').dataset.submissions===beforeSubmissions,'native required acknowledgement prevents review form handler and saved queue mutation');
 const pumpNote='Inspect pump isolation.\nLocal review: valve A & valve B checked.';const pumpPayload=await save('pump',pumpNote,2);assert(note('compressor').value==='Compressor draft retained across badge count changes.'&&ack('compressor').checked,'actual other-alert private draft survives count/style changes');const helpAfterPump=await show();assert(helpAfterPump.text.includes('2')&&helpAfterPump.text!==firstHelp.text,'shared custom help updates to actual remaining unread count');key(badge(),'Escape');await wait('updated count help dismisses',()=>!open());
 await queue(false);await detail(false);await queue(true);assert(note('compressor').value==='Compressor draft retained across badge count changes.'&&ack('compressor').checked&&record('pump')?.dataset.note===pumpNote,'queue visibility/detail changes preserve actual private draft and saved record');const fallbackHelp=await show();assert(fallbackHelp.text===badge().getAttribute('aria-label')&&fallbackHelp.text.includes('2'),'tooltip_from_label uses actual meaningful current status aria label');key(badge(),'Escape');await wait('label-derived help dismisses',()=>!open());
 const compressorPayload=await save('compressor','Compressor inspected locally.\nKeep inlet isolated.',1);const sensorPayload=await save('sensor','Sensor wiring reviewed locally.',0);assert(records().length===3&&pending()===0&&reviewed()===3,'actual three reviewed records derive zero pending alerts');const beforeSaved=JSON.stringify(records()),beforeSavedSubmissions=form('pump').dataset.submissions;review('pump').click();assert(JSON.stringify(records())===beforeSaved&&form('pump').dataset.submissions===beforeSavedSubmissions,'native disabled saved-review action cannot duplicate or mutate records');const zeroHelp=await show();assert(zeroHelp.text===badge().getAttribute('aria-label')&&zeroHelp.text==='All equipment alerts reviewed','zero count status uses meaningful label-derived shared help');snapshots.reviewedAndFallback={...capture(),pumpPayload,compressorPayload,sensorPayload};key(badge(),'Escape');await wait('zero count badge help dismisses',()=>!open());

 const preparation=get('preparation');preparation.value='Keep independent alert-review preparation.';preparation.dispatchEvent(new Event('input',{bubbles:true}));await focus(get('reset'),'actual alert queue Reset');click('reset');await wait('Reset restores actual owned unread alerts and private forms',()=>pending()===3&&reviewed()===0&&records().length===0&&['pump','compressor','sensor'].every(id=>note(id)?.value===''&&!ack(id)?.checked&&!review(id)?.disabled));assert(get('preparation').value==='Keep independent alert-review preparation.'&&status().dataset.detailed==='false','owned Reset preserves independent preparation and help preference');descriptions();snapshots.reset=capture();

 byId('badge-alert-preview-narrow').click();await wait('actual native alert preview340px frame',()=>byId('badge-alert-preview-frame').getBoundingClientRect().width<=341);await queue(true);await focus(get('preparation'),'outside narrow native help');geometry.narrow={viewport:{width:innerWidth,height:innerHeight},frame:rect(byId('badge-alert-preview-frame')),controls:{}};
 for(const [label,node]of [['preparation',get('preparation')],['open',get('open')],['details',get('details')],['reset',get('reset')],['statusBadge',badge()],['pumpNote',note('pump')],['pumpAck',ack('pump')],['pumpReview',review('pump')]]){node.scrollIntoView({block:'nearest',inline:'nearest'});await wait('narrow '+label+' actual center hit',()=>hit(node));const r=node.getBoundingClientRect(),f=byId('badge-alert-preview-frame').getBoundingClientRect();assert(r.left>=f.left-1&&r.right<=f.right+1,'narrow '+label+' control is contained in actual frame');geometry.narrow.controls[label]={rect:rect(node),hit:hit(node)}}
 badge().scrollIntoView({block:'center',inline:'nearest'});await show();geometry.narrow.popup={rect:rect(panel()),anchor:rect(badge()),facts:popupFacts()};const p=panel().getBoundingClientRect();assert(p.left>=-1&&p.right<=innerWidth+1&&p.top>=-1&&p.bottom<=innerHeight+1,'narrow badge native tooltip fits actual viewport');key(badge(),'Escape');await wait('narrow badge help dismisses',()=>!open());const narrowPayload=await save('pump','Narrow-frame pump review.\nSave exact local note.',2);snapshots.narrow={...capture(),payload:narrowPayload};
 return {ok:true,snapshots,geometry,checks:['attached real button owns name/focus and explicit existing/count descriptions; actual queue open/close retains private note/ack drafts','standalone labelled focusable badge uses shared native top-layer current help; synthetic hover, Escape and blur dismiss with native outline facts','required native acknowledgement prevents Save; exact FormData reviews derive count3to2to0 and native disabled saved guards prevent duplicates','help switches custom/aria-label fallback while drafts and records survive count/style/visibility changes; Reset restores work and retains preparation/preferences','actual340px frame keeps real controls hit-testable/contained, native popup fits viewport and actual review form saves'],input:'Excluded native DOM focus/input/click, synthetic mouseenter/mouseleave and Escape exercise actual gallery native controls, constraints, FormEvent handlers and shared native popovers. No physical pointer, trusted keyboard defaults, exhaustive icon/counter matrix or millisecond chronology equivalence is claimed.',viewportQualification:'340px narrows actual gallery frame inside unchanged WebKitGTK viewport; popup containment uses actual native viewport. Lead Chromium verifies true narrow viewport and trusted keyboard/focus ring.',renderer:navigator.userAgent};
}catch(error){const badge=document.querySelector('.badge-alert-status-badge'),panel=badge?.querySelector('[role="tooltip"]');return {ok:false,error:String(error),snapshots,geometry,focus:document.activeElement?.id||document.activeElement?.tagName,status:document.getElementById('badge-alert-status')?.outerHTML,badge:badge?.outerHTML,panel:panel?.outerHTML,pumpForm:document.getElementById('badge-alert-form-pump')?.outerHTML,viewport:{width:innerWidth,height:innerHeight}}}
"#;
