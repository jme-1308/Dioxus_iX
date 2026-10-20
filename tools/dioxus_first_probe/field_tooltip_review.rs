//! Excluded actual-gallery field feedback tooltip preview and bounded native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

use crate::custom_field_workflow as gallery_field;

#[component]
pub fn FieldTooltipPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "field-tooltip-review",
            h1 { "Native field feedback tooltips" }
            Button { id: "field-tooltip-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "field-tooltip-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_field::CustomFieldWorkflowExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_FIELD_TOOLTIP_CHECK);
                        if let Err(error) = query.send("fieldtooltip") {
                            eprintln!("Field tooltip native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native field tooltip: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native field tooltip failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Field tooltip native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.field-tooltip-review{box-sizing:border-box;min-height:100vh;padding:24px}.field-tooltip-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}#field-tooltip-frame{margin-top:24px}@media(max-width:600px){.field-tooltip-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_FIELD_TOOLTIP_CHECK: &str = r#"
let snapshots={},geometry={};
try {
 await dioxus.recv();
 const byId=id=>document.getElementById(id),get=id=>byId('field-composition-'+id),root=()=>get('workflow'),active=()=>document.activeElement;
 const status=()=>get('status'),count=()=>Number(status()?.dataset.count),submissions=()=>Number(status()?.dataset.submissions);
 const record=code=>byId('field-composition-record-'+code),records=()=>[...get('records')?.querySelectorAll('[data-code][data-note]')??[]].map(node=>({code:node.dataset.code,note:node.dataset.note,text:node.textContent.trim()}));
 const panel=()=>get('feedback')?.closest('[role="tooltip"]'),anchor=()=>get('code')?.closest('.ix-field');
 const open=()=>Boolean(panel()?.matches(':popover-open'));
 const ready=()=>Boolean(open()&&panel().dataset.positioned==='true'&&getComputedStyle(panel()).visibility==='visible'&&!panel().inert);
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.tagName)+'; status='+status()?.outerHTML+'; panel='+panel()?.outerHTML)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' actual native focus',()=>active()===node)};
 const click=id=>{assert(get(id),id+' control exists');get(id).click()};
 const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
 const hover=node=>node.dispatchEvent(new MouseEvent('mouseenter',{bubbles:false})),leave=node=>node.dispatchEvent(new MouseEvent('mouseleave',{bubbles:false}));
 const input=async(id,value)=>{const node=get(id);assert(node,id+' actual field exists');node.value=value;node.dispatchEvent(new Event('input',{bubbles:true}));await wait(id+' Rust-owned native input acknowledgement',()=>get(id)?.value===value&&(id==='code'||id==='note'?status()?.dataset[id]===value:true))};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const visible=node=>{if(!node||node.closest('[hidden]'))return false;const r=node.getBoundingClientRect(),s=getComputedStyle(node);return r.width>0&&r.height>0&&s.display!=='none'&&s.visibility!=='hidden'};
 const hit=node=>{if(!visible(node))return false;const r=node.getBoundingClientRect();if(r.left< -1||r.right>innerWidth+1||r.top< -1||r.bottom>innerHeight+1)return false;const target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const feedback=()=>({text:get('feedback')?.textContent.trim(),ids:(get('code').getAttribute('aria-describedby')||'').split(/\s+/).filter(Boolean),panel:panel()?.id,open:open(),ready:ready(),state:status()?.dataset.state});
 const checkDescription=()=>{const f=feedback();assert(f.ids.includes('field-composition-feedback')&&f.ids.every(id=>byId(id)),'actual native input describes explicit existing feedback');assert(document.querySelectorAll('#field-composition-feedback').length===1,'helper ID remains unique across feedback presentation/state');if(panel()){assert(panel().id!=='field-composition-feedback'&&document.querySelectorAll('[id="'+panel().id+'"]').length===1,'private tooltip panel ID is distinct and unique');assert(panel().classList.contains('ix-field-feedback-tooltip'),'current feedback uses shared styled tooltip surface')}return f};
 const capture=()=>({code:get('code')?.value,note:get('note')?.value,count:count(),submissions:submissions(),records:records(),rich:status()?.dataset.rich,locked:status()?.dataset.locked,tooltip:status()?.dataset.tooltip,preparation:get('preparation')?.value,feedback:feedback(),focus:active()?.id||active()?.tagName});
 const preference=async(id,value)=>{if(get(id).checked!==value)click(id);await wait('actual '+id+' preference',()=>status()?.dataset[id]===String(value));checkDescription()};
 const show=async()=>{await focus(get('preparation'),'outside preparation');await focus(get('code'),'actual custom native code input');await wait('current field feedback enters positioned native top layer',ready);assert(panel().dataset.anchor===anchor().id,'actual shared tooltip is anchored to Dioxus-owned field root');checkDescription()};
 const save=async(code,note,expectedCount)=>{await input('code',code);await input('note',note);assert(get('code').validity.valid,'native service code valid before Save');const payload=Object.fromEntries(new FormData(get('form')).entries());assert(payload['service-code']===code&&payload['service-note']===note,'actual native FormData contains exact code and multiline note');const before=submissions();await focus(get('save'),'actual Save service note');click('save');await wait('actual saved service record',()=>record(code)?.dataset.note===note&&count()===expectedCount&&submissions()===before+1);const submitted=JSON.parse(get('payload').textContent);assert(submitted['service-code']===code&&submitted['service-note']===note,'actual Rust FormEvent payload matches native successful controls');assert(record(code).textContent.includes(code)&&record(code).textContent.includes(note),'visible saved record contains identifier and exact multiline note');checkDescription();return payload};
 await wait('actual custom field tooltip workflow mounted',()=>root()&&status()&&get('form')&&get('code')&&get('note')&&get('tooltip')&&get('feedback')&&get('save')&&get('clear')&&get('reset')&&get('preparation'));
 await wait('native document focus',()=>document.hasFocus());
 snapshots.initialNativeControl={outerHTML:get('code').outerHTML,required:get('code').required,pattern:get('code').pattern,validity:{valid:get('code').validity?.valid,valueMissing:get('code').validity?.valueMissing},count:count(),submissions:submissions()};
 assert(status().dataset.tooltip==='false'&&!panel()&&count()===0&&submissions()===0,'actual gallery starts with inline feedback and no saved records');
 assert(get('code').required&&get('code').pattern==='[A-Z]{2}-[0-9]{3}','actual native input keeps required pattern constraints');checkDescription();
 await input('code','bad');await input('note','Private service draft.\nPreserve during feedback changes.');
 const draft={code:get('code').value,note:get('note').value};await preference('tooltip',true);await wait('tooltip feedback moves into one shared panel',()=>panel()&&get('feedback').textContent.includes('Enter an identifier such as AB-123.'));
 assert(get('code').value===draft.code&&get('note').value===draft.note,'enabling tooltip preserves actual private input/textarea drafts');
 const privatePanelId=panel().id;await show();snapshots.inlineToTooltip=capture();

 assert(feedback().text==='Enter an identifier such as AB-123.'&&status().dataset.state==='invalid','visible native tooltip contains chosen invalid feedback');
 await input('code','AB-123');await wait('visible native tooltip updates to valid feedback',()=>ready()&&feedback().text==='Identifier format is valid.'&&status().dataset.state==='valid');assert(panel().id===privatePanelId,'state update preserves private tooltip identity');
 key(get('code'),'Escape');await wait('Escape dismisses actual native feedback panel',()=>!open());
 await show();await focus(get('preparation'),'actual outside blur');await wait('leaving field dismisses actual native feedback',()=>!open());
 hover(anchor());await wait('synthetic native mouseenter shows current feedback',ready);assert(feedback().text==='Identifier format is valid.','hover chooses current valid feedback');leave(anchor());await wait('synthetic mouseleave dismisses feedback',()=>!open());
 await preference('tooltip',false);await wait('inline mode removes native tooltip panel',()=>!panel());assert(get('code').value==='AB-123'&&get('note').value===draft.note,'returning inline preserves private fields and feedback');
 await preference('tooltip',true);await show();snapshots.focusHoverAndPresentation=capture();

 let before=submissions();assert(get('clear').type==='button'&&!get('clear').closest('label'),'footer Clear remains independent native non-submit button');await focus(get('clear'),'actual footer Clear');click('clear');await wait('footer Clear clears only code and focuses actual native input',()=>get('code').value===''&&status().dataset.code===''&&active()===get('code'));assert(get('note').value===draft.note&&count()===0&&submissions()===before,'footer Clear preserves note and never submits in tooltip mode');
 click('save');await hold(80);assert(get('code').validity.valueMissing&&submissions()===before&&count()===0,'native required constraint blocks Save in tooltip mode');await input('code','bad');click('save');await hold(80);assert(get('code').validity.patternMismatch&&submissions()===before&&count()===0,'native pattern mismatch blocks Save in tooltip mode');
 const note='Inspect service valve.\nRecord local handover: valve A & valve B.';const firstPayload=await save('AB-123',note,1);await preference('rich',false);assert(get('code').value==='AB-123'&&get('note').value===note&&record('AB-123')?.dataset.note===note,'rich/plain label change retains private drafts and saved records in tooltip mode');
 click('lock');await wait('actual locked preference',()=>status().dataset.locked==='true');
 assert(get('code').disabled&&get('note').disabled&&get('clear').disabled&&get('save').disabled,'locked fields/actions use actual native disabled semantics');before=submissions();click('clear');click('save');assert(submissions()===before&&record('AB-123')?.dataset.note===note,'disabled native actions preserve tooltip-mode record');click('lock');await wait('actual unlocked preference',()=>status().dataset.locked==='false'&&!get('save').disabled);
 const updatedNote='Inspect service valve.\nHandover confirmed locally.';const upsertPayload=await save('AB-123',updatedNote,1);const secondPayload=await save('CD-456','Second local service note.',2);assert(records().filter(row=>row.code==='AB-123').length===1&&record('AB-123')?.dataset.note===updatedNote,'same code upserts one row while second entry is retained');snapshots.formAndRecords={...capture(),firstPayload,upsertPayload,secondPayload};

 await input('preparation','Keep independent shutdown preparation.');await focus(get('reset'),'actual scoped Reset');click('reset');await wait('Reset clears owned form/records in tooltip mode',()=>count()===0&&submissions()===0&&records().length===0&&get('code').value===''&&get('note').value==='');assert(status().dataset.tooltip==='true'&&status().dataset.rich==='false'&&status().dataset.locked==='false'&&get('preparation').value==='Keep independent shutdown preparation.','Reset retains independent preparation and presentation preferences');checkDescription();await show();assert(feedback().text==='Use two uppercase letters, a hyphen and three digits.','fresh field default helper works after scoped reset');await focus(get('preparation'),'outside after Reset');await wait('reset field feedback dismisses on blur',()=>!open());snapshots.reset=capture();

 byId('field-tooltip-narrow').click();await wait('actual native gallery frame340px',()=>byId('field-tooltip-frame').getBoundingClientRect().width<=341);await input('code','bad');await input('note','Narrow-frame service handover.\nKeep local work visible.');geometry.narrow={viewport:{width:innerWidth,height:innerHeight},frame:rect(byId('field-tooltip-frame')),controls:{}};
 for(const id of ['preparation','code','note','clear','save','reset','rich','lock','tooltip']){const node=get(id);node.scrollIntoView({block:'nearest',inline:'nearest'});await wait('narrow '+id+' actual center hit',()=>hit(node));const r=node.getBoundingClientRect(),f=byId('field-tooltip-frame').getBoundingClientRect();assert(r.left>=f.left-1&&r.right<=f.right+1,'narrow '+id+' stays in actual frame');geometry.narrow.controls[id]={rect:rect(node),hit:hit(node)}}
 get('code').scrollIntoView({block:'center',inline:'nearest'});await show();geometry.narrow.panel={rect:rect(panel()),anchor:rect(anchor()),feedback:feedback()};const p=panel().getBoundingClientRect();assert(p.left>=-1&&p.right<=innerWidth+1&&p.top>=-1&&p.bottom<=innerHeight+1,'narrow field tooltip is contained in actual native viewport');assert(p.width>0&&get('feedback').textContent.includes('Enter an identifier'),'narrow native top-layer panel shows real invalid feedback');key(get('code'),'Escape');await wait('narrow Escape dismisses current feedback',()=>!open());const narrowPayload=await save('EF-789','Narrow-frame service handover.\nKeep local work visible.',1);snapshots.narrow={...capture(),payload:narrowPayload};
 return {ok:true,snapshots,geometry,checks:['inline/tooltip modes preserve private drafts and one explicit helper description','native focus and synthetic hover show current invalid/valid feedback in shared native top layer; Escape and blur dismiss','independent footer Clear and native constraints remain effective; exact FormData saves/upserts records through rich/locked changes','scoped Reset clears owned work while preserving preparation/preferences and renewed default feedback','actual340px frame keeps controls hit-testable/contained and tooltip visible inside viewport with working Save'],input:'Excluded native DOM focus/input/click, synthetic mouseenter/mouseleave and Escape exercise actual gallery handlers, native popover state, native constraint validation and successful form controls. No fabricated application callbacks, trusted keyboard default actions or millisecond chronology equivalence are claimed.',viewportQualification:'340px narrows actual gallery frame inside unchanged desktop WebKitGTK viewport; popup containment is against real native viewport. Lead Chromium verifies true narrow viewport and trusted focus/keys.',renderer:navigator.userAgent};
}catch(error){const code=document.getElementById('field-composition-code'),feedback=document.getElementById('field-composition-feedback'),panel=feedback?.closest('[role="tooltip"]');return {ok:false,error:String(error),snapshots,geometry,focus:document.activeElement?.id||document.activeElement?.tagName,status:document.getElementById('field-composition-status')?.outerHTML,code:code?.outerHTML,panel:panel?.outerHTML,viewport:{width:innerWidth,height:innerHeight}}}
"#;
