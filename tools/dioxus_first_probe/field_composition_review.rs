//! Excluded actual-gallery custom field composition preview and bounded native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/custom_field_workflow.rs"]
mod gallery_field;

#[component]
pub fn FieldCompositionPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "field-composition-review",
            h1 { "Composed service entry fields" }
            Button { id: "field-composition-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "field-composition-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_field::CustomFieldWorkflowExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_FIELD_COMPOSITION_CHECK);
                        if let Err(error) = query.send("fieldcomposition") {
                            eprintln!("Field composition native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native field composition: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native field composition failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Field composition native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.field-composition-review{box-sizing:border-box;min-height:100vh;padding:24px}.field-composition-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}#field-composition-frame{margin-top:24px}@media(max-width:600px){.field-composition-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_FIELD_COMPOSITION_CHECK: &str = r#"
let snapshots={},geometry={};
try {
 await dioxus.recv();
 const byId=id=>document.getElementById(id),get=id=>byId('field-composition-'+id),root=()=>get('workflow'),active=()=>document.activeElement;
 const status=()=>get('status'),count=()=>Number(status()?.dataset.count),submissions=()=>Number(status()?.dataset.submissions);
 const record=code=>byId('field-composition-record-'+code);
 const records=()=>[...get('records')?.querySelectorAll('[data-code][data-note]')??[]].map(node=>({code:node.dataset.code,note:node.dataset.note,text:node.textContent.trim()}));
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.tagName)+'; status='+status()?.outerHTML)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' actual native focus',()=>active()===node)};
 const click=id=>{assert(get(id),id+' control exists');get(id).click()};
 const input=async(id,value)=>{const node=get(id);assert(node,id+' actual field exists');node.value=value;node.dispatchEvent(new Event('input',{bubbles:true}));await wait(id+' Rust-owned native input acknowledgement',()=>get(id)?.value===value&&(id==='code'||id==='note'?status()?.dataset[id]===value:true))};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const visible=node=>{if(!node||node.closest('[hidden]'))return false;const r=node.getBoundingClientRect(),s=getComputedStyle(node);return r.width>0&&r.height>0&&s.display!=='none'&&s.visibility!=='hidden'};
 const hit=node=>{if(!visible(node))return false;const r=node.getBoundingClientRect();if(r.left< -1||r.right>innerWidth+1||r.top< -1||r.bottom>innerHeight+1)return false;const target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const descriptions=()=>({ids:(get('code').getAttribute('aria-describedby')||'').split(/\s+/).filter(Boolean),feedback:get('feedback')?.textContent.trim(),invalid:get('code').getAttribute('aria-invalid'),state:status()?.dataset.state});
 const capture=()=>({code:get('code')?.value,note:get('note')?.value,count:count(),submissions:submissions(),records:records(),rich:status()?.dataset.rich,locked:status()?.dataset.locked,preparation:get('preparation')?.value,feedback:descriptions(),focus:active()?.id||active()?.tagName});
 const checkDescription=()=>{const d=descriptions();assert(d.ids.includes('field-composition-feedback')&&d.ids.every(id=>byId(id)),'actual input describes explicit existing feedback ID');assert(document.querySelectorAll('#field-composition-feedback').length===1,'feedback target ID remains unique');return d};
 const mode=async rich=>{if(get('rich').checked!==rich)click('rich');await wait('actual '+(rich?'rich':'plain')+' label composition',()=>status()?.dataset.rich===String(rich));checkDescription()};
 const save=async(code,note,expectedCount)=>{await input('code',code);await input('note',note);assert(get('code').validity.valid,'native service code valid before Save');const payload=Object.fromEntries(new FormData(get('form')).entries());assert(payload['service-code']===code&&payload['service-note']===note,'actual native FormData contains exact code and multiline note');const before=submissions();await focus(get('save'),'actual Save entry');click('save');await wait('actual stored service entry',()=>record(code)?.dataset.note===note&&count()===expectedCount&&submissions()===before+1);assert(record(code).textContent.includes(code)&&record(code).textContent.includes(note),'visible stored entry contains submitted identifier and exact note');const submitted=JSON.parse(get('payload').textContent);assert(submitted['service-code']===code&&submitted['service-note']===note,'actual Rust FormEvent payload agrees with native successful controls');checkDescription();return payload};
 await wait('actual field composition workflow mounted',()=>root()&&status()&&get('form')&&get('code')&&get('note')&&get('save')&&get('clear')&&get('reset')&&get('rich')&&get('preparation')&&get('feedback'));
 await wait('native document focus',()=>document.hasFocus());
 snapshots.initialNativeControl={outerHTML:get('code').outerHTML,tag:get('code').tagName,type:get('code').type,required:get('code').required,requiredAttribute:get('code').getAttribute('required'),pattern:get('code').pattern,patternAttribute:get('code').getAttribute('pattern'),value:get('code').value,validity:{valid:get('code').validity?.valid,valueMissing:get('code').validity?.valueMissing,patternMismatch:get('code').validity?.patternMismatch},count:count(),submissions:submissions()};
 assert(count()===0&&submissions()===0&&get('code').required&&get('code').pattern==='[A-Z]{2}-[0-9]{3}','actual empty native form starts with required code pattern and no records');
 checkDescription();await mode(true);
 const label=root().querySelector('label[for="field-composition-code"]');assert(label&&label.querySelector('strong,em,span'),'rich composition renders native label content');label.click();await wait('actual composed label targets native code input',()=>active()===get('code'));
 let before=submissions();click('save');await hold(80);assert(submissions()===before&&count()===0&&!get('code').validity.valid&&get('code').validity.valueMissing,'native required field prevents actual form submit');
 await input('code','bad');await input('note','Draft retained when clearing only the service code.');await wait('actual invalid code feedback',()=>status().dataset.state==='invalid');
 checkDescription();assert(get('code').validity.patternMismatch,'native code pattern mismatch is exposed');before=submissions();click('save');await hold(80);assert(submissions()===before&&count()===0,'native pattern mismatch prevents actual form submit');
 assert(get('clear').type==='button'&&!get('clear').closest('label'),'bottom-right Clear uses independent native button semantics');await focus(get('clear'),'actual footer Clear');click('clear');
 await wait('footer Clear clears private code and returns real focus',()=>get('code').value===''&&status().dataset.code===''&&active()===get('code'));
 assert(get('note').value==='Draft retained when clearing only the service code.'&&submissions()===before&&count()===0,'Clear preserves private note and never submits');snapshots.constraintsAndClear=capture();

 const note='Inspect isolation switch.\nRecord local handover: valve A & valve B.';
 const initialPayload=await save('AB-123',note,1);assert(status().dataset.state==='valid','saved valid service code shows valid state');snapshots.saved={...capture(),payload:initialPayload};
 await input('code','CD-456');await input('note','Private draft\nretained across label composition.');const draft={code:get('code').value,note:get('note').value};
 await mode(false);assert(get('code').value===draft.code&&get('note').value===draft.note&&record('AB-123')?.dataset.note===note,'rich to plain preserves actual private fields and stored records');
 const plainLabel=root().querySelector('label[for="field-composition-code"]');plainLabel.click();await wait('plain native label targets unchanged input',()=>active()===get('code'));
 await mode(true);assert(get('code').value===draft.code&&get('note').value===draft.note,'plain to rich preserves private fields');
 const secondPayload=await save('CD-456',draft.note,2);const updatedNote='Inspect isolation switch.\nConfirmed locally; keep both valves closed.';
 const upsertPayload=await save('AB-123',updatedNote,2);assert(records().filter(row=>row.code==='AB-123').length===1&&record('CD-456')?.dataset.note===draft.note,'same service code upserts one actual row without replacing another record');
 click('lock');await wait('actual locked fields and actions',()=>status().dataset.locked==='true'&&get('code').disabled&&get('note').disabled&&get('clear').disabled&&get('save').disabled);before=submissions();click('clear');click('save');assert(submissions()===before&&get('code').value==='AB-123'&&get('note').value===updatedNote&&count()===2,'disabled native actions preserve actual fields/records without submitting');
 click('lock');await wait('actual unlocked fields and actions',()=>status().dataset.locked==='false'&&!get('code').disabled&&!get('note').disabled&&!get('clear').disabled&&!get('save').disabled);snapshots.compositionAndUpsert={...capture(),secondPayload,upsertPayload};

 await input('preparation','Keep independent shutdown preparation.');await mode(false);await focus(get('reset'),'actual scoped Reset');click('reset');
 await wait('Reset clears owned form drafts and records',()=>count()===0&&submissions()===0&&records().length===0&&get('code').value===''&&get('note').value==='');
 assert(get('preparation').value==='Keep independent shutdown preparation.'&&status().dataset.rich==='false','owned Reset preserves independent preparation and label preference');checkDescription();snapshots.reset=capture();

 get('narrow').click();await wait('actual native field preview frame340px',()=>get('frame').getBoundingClientRect().width<=341);await mode(true);
 const narrowNote='Narrow-frame handover.\nLeave local service records intact.';await input('code','EF-789');await input('note',narrowNote);
 geometry.narrow={viewport:{width:innerWidth,height:innerHeight},frame:rect(get('frame')),controls:{}};
 for(const id of ['preparation','code','note','clear','save','reset','rich','lock']){const node=get(id);node.scrollIntoView({block:'nearest',inline:'nearest'});await wait('narrow '+id+' actual center hit',()=>hit(node));const r=node.getBoundingClientRect(),f=get('frame').getBoundingClientRect();assert(r.left>=f.left-1&&r.right<=f.right+1,'narrow '+id+' remains contained in actual frame');geometry.narrow.controls[id]={rect:rect(node),hit:hit(node)}}
 const narrowPayload=await save('EF-789',narrowNote,1);snapshots.narrow={...capture(),payload:narrowPayload};
 return {ok:true,snapshots,geometry,checks:['composed native label focuses code; required/pattern validation prevents form handlers; independent footer Clear preserves note without submitting','actual native FormData and Rust FormEvent store exact identifiers/multiline notes and upsert one record per code','rich/plain composition preserves private field state, explicit feedback IDs and existing saved entries','scoped Reset clears owned form and records while preserving independent preparation and label preference','actual340px frame has contained hit-testable fields/actions and working native save'],input:'Excluded native DOM label clicks, focus, input and button clicks exercise actual gallery handlers, native constraint validation and successful form controls. No fabricated application callbacks, trusted keyboard defaults or millisecond timing equivalence are claimed.',viewportQualification:'340px narrows the actual gallery frame inside unchanged desktop WebKitGTK viewport. Lead Chromium verifies true narrow viewport and trusted input.',renderer:navigator.userAgent};
}catch(error){return {ok:false,error:String(error),snapshots,geometry,focus:document.activeElement?.id||document.activeElement?.tagName,status:document.getElementById('field-composition-status')?.outerHTML,viewport:{width:innerWidth,height:innerHeight}}}
"#;
