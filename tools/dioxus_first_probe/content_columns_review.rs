//! Excluded actual-gallery content-sized equipment layout preview and bounded native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/equipment_search_layout.rs"]
mod gallery_equipment;

#[component]
pub fn ContentColumnsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "content-columns-review",
            h1 { "Content-sized equipment layout" }
            Button { id: "content-columns-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "content-columns-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_equipment::EquipmentSearchLayoutExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_CONTENT_COLUMNS_CHECK);
                        if let Err(error) = query.send("contentcolumns") {
                            eprintln!("Content columns native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native content columns: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native content columns failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Content columns native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.content-columns-review{box-sizing:border-box;min-height:100vh;padding:24px}.content-columns-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}#content-columns-frame{margin-top:24px}@media(max-width:600px){.content-columns-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CONTENT_COLUMNS_CHECK: &str = r#"
let snapshots={},geometry={};
try {
 await dioxus.recv();
 const byId=id=>document.getElementById(id),get=id=>byId('equipment-search-'+id),root=()=>byId('equipment-search-layout'),active=()=>document.activeElement;
 const status=()=>get('status'),selected=()=>status()?.dataset.selected,assigned=()=>Number(status()?.dataset.assigned);
 const select=id=>byId('equipment-select-'+id),record=id=>byId('equipment-search-assignment-'+id);
 const rows=()=>[...get('results')?.querySelectorAll('[id^="equipment-search-row-"]')??[]].filter(node=>visible(node)).map(node=>node.dataset.equipment);
 const records=()=>[...get('assignments')?.querySelectorAll('[id^="equipment-search-assignment-"]')??[]].map(node=>({id:node.id,equipment:node.dataset.equipment,note:node.dataset.note,text:node.textContent.trim()}));
 const column=node=>node?.closest('.ix-layout-col');
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.tagName)+'; status='+status()?.outerHTML)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' actual native focus',()=>active()===node)};
 const click=id=>{assert(get(id),id+' control exists');get(id).click()};
 const input=async(node,value)=>{assert(node,'actual field exists');node.value=value;node.dispatchEvent(new Event('input',{bubbles:true}));await wait('controlled native field value',()=>node.value===value)};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const visible=node=>{if(!node||node.closest('[hidden]'))return false;const r=node.getBoundingClientRect(),s=getComputedStyle(node);return r.width>0&&r.height>0&&s.display!=='none'&&s.visibility!=='hidden'};
 const hit=node=>{if(!visible(node))return false;const r=node.getBoundingClientRect();if(r.left< -1||r.right>innerWidth+1||r.top< -1||r.bottom>innerHeight+1)return false;const target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const near=(actual,expected,tolerance=2)=>Number.isFinite(actual)&&Math.abs(actual-expected)<=tolerance;
 const colFacts=node=>{const col=column(node),style=col?getComputedStyle(col):null;return {rect:rect(col),basis:style?.flexBasis,grow:style?.flexGrow,row:rect(col?.parentElement),variables:col?Object.fromEntries(['--ix-col-basis','--ix-col-sm','--ix-col-md','--ix-col-lg','--ix-col-grow','--ix-col-grow-sm','--ix-col-grow-md','--ix-col-grow-lg'].map(key=>[key,col.style.getPropertyValue(key)])):null}};
 const columns=()=>({query:colFacts(get('query')),line:colFacts(get('line')),filterActions:colFacts(get('apply')),note:colFacts(get('note')),assign:colFacts(get('assign'))});
 const capture=()=>({query:get('query')?.value,line:get('line')?.value,note:get('note')?.value,appliedQuery:status()?.dataset.query,appliedLine:status()?.dataset.line,selected:selected(),visible:rows(),assigned:assigned(),records:records(),auto:status()?.dataset.auto,preparation:get('preparation')?.value,focus:active()?.id||active()?.tagName});
 const names={motor:'Motor 001',pump:'Pump 002',compressor:'Compressor 003'};
 const pick=async id=>{await focus(select(id),id+' actual selection');select(id).click();await wait(id+' real selected equipment and action label',()=>selected()===id&&!get('assign').disabled&&get('assign').textContent.includes(names[id]))};
 const filter=async(value,lineValue,expected)=>{await input(get('query'),value);if(get('line').value!==lineValue){get('line').value=lineValue;get('line').dispatchEvent(new Event('change',{bubbles:true}))}const payload=Object.fromEntries(new FormData(get('filter-form')).entries());assert(payload.query===value&&payload.line===lineValue,'native filter controls contain exact query/line');await focus(get('apply'),'actual Apply filter');click('apply');await wait('actual equipment list matches applied filter',()=>rows().join(',')===expected.join(',')&&status().dataset.query===value);const submitted=JSON.parse(get('filter-payload').textContent);assert(submitted.query===value&&submitted.line===lineValue,'actual FormEvent filter payload matches successful controls');return payload};
 const assign=async(id,value,expectedCount)=>{await input(get('note'),value);const payload=Object.fromEntries(new FormData(get('assignment-form')).entries());assert(payload['assignment-note']===value,'actual native assignment note participates in form');await focus(get('assign'),'actual Assign equipment');click('assign');await wait(id+' actual stored assignment',()=>record(id)?.dataset.note===value&&assigned()===expectedCount);assert(record(id).textContent.includes(names[id])&&record(id).textContent.includes(value),'visible assignment record contains equipment and submitted note');const submitted=JSON.parse(get('assignment-payload').textContent);assert(submitted['assignment-note']===value,'actual assignment FormEvent payload contains exact note');return payload};
 const mode=async auto=>{if(get('auto').checked!==auto)click('auto');await wait('actual '+(auto?'Auto':'fixed')+' width mode',()=>status().dataset.auto===String(auto))};
 await wait('actual equipment search layout mounted',()=>root()&&status()&&get('query')&&get('line')&&get('note')&&get('apply')&&get('assign')&&get('auto')&&get('preparation')&&rows().length===4);
 await wait('native document focus',()=>document.hasFocus());
 const initialLine=get('line').value;
 assert(status().dataset.auto==='true'&&assigned()===0&&get('assign').disabled,'initial actual equipment list uses Auto with no assignment selection');
 assert(rows().join(',')==='motor,pump,compressor,archive'&&select('archive').disabled,'all real equipment records are visible with archived selection disabled');
 geometry.initial=columns();assert(geometry.initial.query.grow==='1'&&Number.parseFloat(geometry.initial.query.basis)===0,'actual search column remains Flexible');
 assert(geometry.initial.line.basis==='auto'&&geometry.initial.line.grow==='0'&&geometry.initial.filterActions.basis==='auto'&&geometry.initial.filterActions.grow==='0','actual line/action columns size naturally with Auto');snapshots.initial=capture();
 await input(get('preparation'),'Prepare local equipment-assignment handoff.');
 const filterPayload=await filter('pump','Line 2',['pump']);await pick('pump');const pumpPayload=await assign('pump','Pump assignment reviewed locally.',1);snapshots.pumpAssignment={...capture(),filterPayload,payload:pumpPayload};

 await focus(get('clear'),'actual Clear filters');click('clear');await wait('Clear restores real equipment list and private filter fields',()=>rows().length===4&&get('query').value===''&&get('line').value===initialLine&&status().dataset.query==='');
 assert(get('note').value==='Pump assignment reviewed locally.'&&record('pump')?.dataset.note==='Pump assignment reviewed locally.','clearing filters preserves private assignment note and stored record');
 await pick('motor');await input(get('note'),'Unsubmitted assignment draft kept across widths.');await input(get('query'),'unsubmitted filter draft');geometry.motorAuto=columns();
 await pick('compressor');await wait('Auto action column follows longer real equipment label',()=>colFacts(get('assign')).rect.width>geometry.motorAuto.assign.rect.width+4);geometry.compressorAuto=columns();
 assert(geometry.compressorAuto.assign.basis==='auto'&&geometry.compressorAuto.assign.grow==='0'&&geometry.compressorAuto.note.grow==='1','Auto action and Flexible private note remain distinct actual column modes');
 assert(get('note').value==='Unsubmitted assignment draft kept across widths.'&&get('query').value==='unsubmitted filter draft'&&status().dataset.query==='','actual private drafts survive selection and content-driven width changes without applying query');
 await mode(false);await wait('fixed action column uses actual quarter-row span',()=>{const f=colFacts(get('assign'));return f.grow==='0'&&f.basis!=='auto'&&near(f.rect.width,f.row.width/4)});geometry.fixed=columns();
 assert(get('note').value==='Unsubmitted assignment draft kept across widths.'&&get('query').value==='unsubmitted filter draft','switching Auto to fixed preserves private field drafts');
 await mode(true);await wait('Auto width restores natural content sizing',()=>colFacts(get('assign')).basis==='auto'&&near(colFacts(get('assign')).rect.width,geometry.compressorAuto.assign.rect.width));
 assert(get('note').value==='Unsubmitted assignment draft kept across widths.'&&get('query').value==='unsubmitted filter draft','switching fixed back to Auto keeps drafts');
 select('archive').click();assert(selected()==='compressor'&&assigned()===1,'disabled native archived button cannot change selected equipment or assignments');
 const compressorPayload=await assign('compressor','Compressor handoff reviewed locally.',2);snapshots.widthChanges={...capture(),payload:compressorPayload};

 await mode(false);await focus(get('reset'),'scoped Reset');click('reset');
 await wait('Reset clears owned private fields/filter/selection/records',()=>assigned()===0&&records().length===0&&rows().length===4&&get('query').value===''&&get('line').value===initialLine&&get('note').value===''&&get('assign').disabled);
 assert(status().dataset.auto==='false'&&get('preparation').value==='Prepare local equipment-assignment handoff.','owned Reset preserves width preference and independent preparation');snapshots.reset=capture();
 document.getElementById('content-columns-narrow').click();await wait('actual native gallery frame340px',()=>document.getElementById('content-columns-frame').getBoundingClientRect().width<=341);
 await pick('motor');const narrowFilterPayload=await filter('motor','Line 1',['motor']);await input(get('note'),'Motor assignment from narrow frame.');
 const frame=document.getElementById('content-columns-frame');geometry.narrow={viewport:{width:innerWidth,height:innerHeight},frame:rect(frame),columns:columns(),controls:{}};
 const firstRow=column(get('query')).parentElement,secondRow=column(get('note')).parentElement;
 for(const [label,container] of [['filter',firstRow],['assignment',secondRow]]){const cols=[...container.children].filter(node=>node.classList.contains('ix-layout-col'));let bottom=-Infinity;for(const node of cols){const r=node.getBoundingClientRect();assert(near(r.width,container.getBoundingClientRect().width)&&r.top>=bottom-1,'small Span12 '+label+' columns actually stack at full row width');bottom=r.bottom}}
 for(const id of ['query','line','apply','clear','note','assign','auto','reset','preparation']){const node=get(id);node.scrollIntoView({block:'nearest',inline:'nearest'});geometry.narrow.controls[id]={rect:rect(node),hit:hit(node)};await wait('narrow '+id+' actual center hit',()=>hit(node));const r=node.getBoundingClientRect(),f=frame.getBoundingClientRect();assert(r.left>=f.left-1&&r.right<=f.right+1,'narrow '+id+' remains contained in real frame');geometry.narrow.controls[id]={rect:rect(node),hit:hit(node)}}
 const narrowPayload=await assign('motor','Motor assignment from narrow frame.',1);snapshots.narrow={...capture(),filterPayload:narrowFilterPayload,payload:narrowPayload};
 return {ok:true,snapshots,geometry,checks:['actual native filter FormData updates equipment list and selection saves real equipment/note records','Auto action width follows actual selected equipment label while Flexible search/note use remaining row space','private filter/assignment drafts survive Auto/fixed transitions; disabled archived equipment remains guarded','scope Reset clears owned fields/records while retaining preparation/width preference','actual340px frame uses full-row small Span12 stacking with hit-testable contained fields/actions and working filter/assignment'],input:'Excluded native DOM focus/input/change/clicks exercise actual gallery handlers and native form successful controls. No fabricated application callbacks, trusted keyboard defaults or millisecond timing are claimed.',viewportQualification:'340px narrows the actual gallery frame inside the unchanged desktop WebKitGTK viewport; this tests existing container rules, not upstream viewport threshold equivalence. Lead Chromium verifies true narrow layout and trusted input.',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,geometry,focus:document.activeElement?.id||document.activeElement?.tagName,status:document.getElementById('equipment-search-status')?.outerHTML,viewport:{width:innerWidth,height:innerHeight}}}

"#;
