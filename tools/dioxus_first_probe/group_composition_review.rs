//! Excluded actual-gallery group composition preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/group_composition.rs"]
mod gallery_composition;

#[component]
pub fn GroupCompositionPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_composition::STYLES} }
        IxTheme { class: "group-composition-review",
            h1 { "Group composition" }
            Button { id: "group-composition-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "group-composition-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_composition::GroupCompositionExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_GROUP_COMPOSITION_CHECK);
                        if let Err(error) = query.send("groupcomposition") {
                            eprintln!("Group composition native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native group composition: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native group composition failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Group composition native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.group-composition-review{box-sizing:border-box;min-height:100vh;padding:24px}.group-composition-review>h1{font-size:28px;margin:0 0 24px}.group-composition-review h2{font-size:22px;margin:24px 0 16px}.group-composition-review p{max-width:72ch;line-height:1.5}#group-composition-frame{margin-top:24px}@media(max-width:600px){.group-composition-review{padding:16px}}";

// Excluded actual-gallery replay. Untrusted Enter/Space only check native-default
// permission; explicit DOM clicks activate actual controls. Trusted keys are lead evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GROUP_COMPOSITION_CHECK: &str = r#"
let snapshots={},keyPermissions={},hitFacts={},layout={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('group-compose-result'),group=()=>get('group-compose-group'),row=asset=>get('group-compose-'+asset),note=()=>get('group-compose-note'),preparation=()=>get('group-compose-preparation'),save=()=>get('group-compose-save'),selected=()=>result()?.dataset.selected,saves=()=>Number(result()?.dataset.saves),records=()=>[...get('group-compose-records')?.querySelectorAll('tbody tr[data-asset]')??[]],field=(record,name)=>record?.querySelector('[data-field="'+name+'"]')?.textContent;
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const input=async(node,value,label)=>{await focus(node,label);Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(label+' current native value',()=>node.value===value)};
 const option=async(name,value)=>{const node=get('group-compose-'+name);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(name+' actual option state',()=>result().getAttribute('data-'+name)===String(value))};
 const select=async asset=>{row(asset).click();await wait('actual '+asset+' assignment selected',()=>selected()===asset&&row(asset).getAttribute('aria-pressed')==='true'&&row(asset).dataset.selected==='true')};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const hit=async(node,label)=>{await focus(node,label);await wait(label+' native hit and containment',()=>{const r=node.getBoundingClientRect(),frame=get('group-composition-frame').getBoundingClientRect(),point=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);failureGeometry={node:node.id||node.className,rect:rect(node),frame:rect(get('group-composition-frame')),hit:point?.id||point?.className||point?.tagName,viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};return r.width>0&&r.height>0&&r.left>=frame.left-1&&r.right<=frame.right+1&&r.top>=0&&r.bottom<=innerHeight+1&&(point===node||node.contains(point))});hitFacts[label]={...failureGeometry}};
 const snapshot=()=>({state:{...result().dataset},rows:['motor','pump','archive'].map(asset=>({asset,tag:row(asset).tagName,label:row(asset).getAttribute('aria-label'),pressed:row(asset).getAttribute('aria-pressed'),disabled:row(asset).disabled})),note:note().value,preparation:preparation().value,records:records().map(record=>({asset:record.dataset.asset,label:field(record,'asset'),note:field(record,'note')})),focus:active()?.id});

 await wait('actual composed GroupItem gallery mounted',()=>group()&&row('motor')&&row('pump')&&row('archive')&&note()&&preparation()&&save()&&result());await wait('native document focused',()=>document.hasFocus());
 const originalGroup=group(),originalNote=note(),originalPreparation=preparation(),editor=get('group-compose-editor');
 assert(selected()===''&&saves()===0&&records().length===0&&note().value==='Inspect coupling before assignment'&&preparation().value==='Prepare local assignments','actual initial empty assignment and retained local defaults');
 assert(['motor','pump','archive'].every(asset=>row(asset).tagName==='BUTTON'&&row(asset).type==='button'&&!row(asset).querySelector('button,input,textarea,select,a[href],[tabindex]'))&&row('motor').getAttribute('aria-label')==='Assign Motor 001'&&row('pump').getAttribute('aria-label')==='Assign Pump 002'&&row('archive').getAttribute('aria-label')==='Archived 004 unavailable'&&row('archive').disabled,'structured label content stays within one native button with no nested interactive controls');
 assert(editor.tagName==='DIV'&&editor.classList.contains('ix-group-custom-item')&&editor.contains(note())&&editor.contains(save())&&!editor.closest('button')&&!row('motor').contains(editor),'legacy custom row keeps native note/Save independently owned');save().click();await wait('missing-selection Save reports useful error without an assignment',()=>result().dataset.error==='true'&&saves()===0&&records().length===0&&selected()==='');snapshots.semantics=snapshot();

 await input(note(),'Inspect retained coupling note','independent assignment note');await input(preparation(),'Prepare retained assignments','independent page preparation');
 for(const [value,asset] of [['Enter','motor'],[' ','pump']]){await focus(row(asset),'native composed '+asset+' row');const before=selected(),event=key(row(asset),value);keyPermissions[asset+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native composed '+value+' default remains available');await wait('untrusted key does not manufacture native activation',()=>selected()===before);await select(asset)}
 assert(row('pump').dataset.selected==='true'&&row('motor').getAttribute('aria-pressed')==='false'&&note()===originalNote&&note().value==='Inspect retained coupling note'&&preparation()===originalPreparation&&preparation().value==='Prepare retained assignments'&&saves()===0,'actual controlled row selection preserves independent native input scopes and has no accidental Save');snapshots.selection=snapshot();

 row('archive').click();await wait('disabled archived row cannot change selected asset',()=>selected()==='pump'&&saves()===0);await option('paused',true);row('motor').click();row('pump').click();await wait('suppressed rows preserve owner selection without pressed cues',()=>selected()==='pump'&&!row('motor').hasAttribute('aria-pressed')&&!row('pump').hasAttribute('aria-pressed')&&row('pump').dataset.selected==='false');assert(!row('motor').disabled&&row('motor').tabIndex===0&&note()===originalNote&&note().value==='Inspect retained coupling note','suppression keeps native focusability and independent editor state');await option('paused',false);await wait('resuming selection restores current Pump cue',()=>row('pump').getAttribute('aria-pressed')==='true');await option('motor-unavailable',true);assert(row('motor').disabled,'live unavailable option disables actual Motor button');row('motor').click();await wait('disabled Motor cannot replace actual Pump',()=>selected()==='pump');await option('motor-unavailable',false);await select('motor');await option('motor-unavailable',true);save().click();await wait('selected unavailable Motor Save guard leaves actual records empty',()=>selected()==='motor'&&row('motor').disabled&&saves()===0&&records().length===0&&result().dataset.error==='true');await option('motor-unavailable',false);snapshots.guards=snapshot();

 await select('pump');await focus(save(),'independent legacy row Save');save().click();await wait('Save appends exact actual Pump assignment and retained note',()=>saves()===1&&records().length===1&&records()[0].dataset.asset==='pump'&&field(records()[0],'asset')==='Pump 002'&&field(records()[0],'note')==='Inspect retained coupling note');assert(selected()==='pump'&&note()===originalNote&&note().value==='Inspect retained coupling note','independent Save does not select its custom row or clear the note');await option('paused',true);save().click();await wait('paused selection still permits saving existing valid assignment',()=>saves()===2&&records().length===2&&records()[1].dataset.asset==='pump'&&field(records()[1],'note')==='Inspect retained coupling note');const reset=get('group-compose-reset'),pausedOption=get('group-compose-paused');await focus(reset,'actual assignment Reset');reset.click();await wait('Reset clears local model/options and remounts owned note/preparation scopes',()=>selected()===''&&saves()===0&&records().length===0&&result().dataset.paused==='false'&&result().getAttribute('data-motor-unavailable')==='false'&&result().dataset.error==='false'&&note()!==originalNote&&note().value==='Inspect coupling before assignment'&&preparation()!==originalPreparation&&preparation().value==='Prepare local assignments');assert(!originalGroup.isConnected&&!originalNote.isConnected&&!originalPreparation.isConnected&&get('group-compose-reset')===reset&&get('group-compose-paused')===pausedOption&&active()===reset,'scoped reset removes former owned controls while preserving actual toolbar/options focus');snapshots.savedAndReset=snapshot();

 get('group-composition-narrow').click();await wait('340px actual composed group frame',()=>get('group-composition-frame').getBoundingClientRect().width<=341);await hit(row('pump'),'narrow composed Pump native button');const narrowLabel=row('pump').querySelector('.group-compose-label'),nameLine=narrowLabel?.querySelector('.group-compose-name-line'),description=narrowLabel?.lastElementChild;assert(narrowLabel&&nameLine&&description&&getComputedStyle(narrowLabel).whiteSpace==='normal'&&rect(description).top>=rect(nameLine).bottom-1&&rect(narrowLabel).height>=rect(nameLine).height+rect(description).height&&rect(narrowLabel).right<=rect(row('pump')).right+1&&rect(narrowLabel).bottom<=rect(row('pump')).bottom+1,'actual structured label wraps as visible stacked content within narrow native row');await select('pump');await hit(note(),'narrow independent native note');await input(note(),'Narrow Pump assignment','narrow assignment note');await hit(save(),'narrow independent Save action');save().click();await wait('narrow workflow appends actual Pump assignment',()=>saves()===1&&records().length===1&&records()[0].dataset.asset==='pump'&&field(records()[0],'note')==='Narrow Pump assignment');await hit(preparation(),'narrow independent preparation');layout={frame:rect(get('group-composition-frame')),group:rect(group()),motor:rect(row('motor')),pump:rect(row('pump')),label:rect(narrowLabel),nameLine:rect(nameLine),description:rect(description),editor:rect(get('group-compose-editor')),viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};assert(layout.group.width<=341&&row('pump').scrollWidth<=row('pump').clientWidth+1&&row('motor').scrollWidth<=row('motor').clientWidth+1&&layout.pageWidth<=innerWidth+1,'structured labels and independent controls fit340px with no clipped horizontal content');snapshots.narrow=snapshot();
 return {ok:true,snapshots,keyPermissions,hitFacts,layout,checks:['actual composed native buttons/accessibility and independent legacy editor without interactive nesting','actual controlled click selection retains local note/preparation with native key default permission','disabled/paused selection and unavailable selected-asset Save guard','exact real assignment records, paused Save and scoped Reset ownership/focus','340px composed row/note/Save/preparation hit-testing and actual saved record'],input:'excluded synthetic input and key events plus explicit native focus/normal page positioning and actual DOM clicks; untrusted Enter/Space only establish default permission and do not prove native activation. No Rust callbacks are invoked.',trustedInput:'lead Chromium supplies trusted Tab/Enter/Space/pointer/typing and paired visual evidence',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,keyPermissions,hitFacts,layout,failureGeometry}}
"#;
