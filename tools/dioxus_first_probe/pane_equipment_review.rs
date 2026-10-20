//! Excluded actual-gallery equipment Pane preview and bounded native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/pane_equipment.rs"]
mod gallery_equipment;

#[component]
pub fn PaneEquipmentPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "pane-equipment-review",
            h1 { "Equipment Pane workflow" }
            Button { id: "pane-equipment-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "pane-equipment-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_equipment::PaneEquipmentExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_PANE_EQUIPMENT_CHECK);
                        if let Err(error) = query.send("paneequipment") {
                            eprintln!("Pane equipment native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native Pane equipment: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native Pane equipment failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Pane equipment native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.pane-equipment-review{box-sizing:border-box;min-height:100vh;padding:24px}.pane-equipment-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}#pane-equipment-frame{margin-top:24px}@media(max-width:600px){.pane-equipment-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_PANE_EQUIPMENT_CHECK: &str = r#"
let snapshots={},geometry={},keys={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById('pane-equipment-'+id),active=()=>document.activeElement;
 const actualPane=()=>document.querySelector('#pane-equipment .ix-pane'),status=()=>get('status'),record=()=>get('record');
 const header=()=>actualPane()?.querySelector('.ix-pane-header > button'),opened=()=>status()?.dataset.open==='true'&&actualPane()?.dataset.expanded==='true';
 const decision=()=>status()?.dataset.decision==='discard';
 const saved=()=>({name:record()?.dataset.name,location:record()?.dataset.location,note:record()?.dataset.note});
 const draft=()=>({name:get('name')?.value,location:get('location')?.value,note:get('note')?.value});
 const saves=()=>Number(status()?.dataset.saves),assert=(condition,label)=>{if(!condition)throw Error(label)};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<180;n++){if(predicate())return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.tagName)+'; status='+status()?.outerHTML)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' native focus',()=>active()===node)};
 const input=async(id,value)=>{const node=get(id);assert(node,id+' field exists');node.value=value;node.dispatchEvent(new Event('input',{bubbles:true}));await wait(id+' controlled value',()=>get(id)?.value===value)};
 const click=id=>{assert(get(id),id+' control exists');get(id).click()};
 const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,code:value==='Tab'?'Tab':value,bubbles:true,cancelable:true,shiftKey});node.dispatchEvent(event);return event};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const visible=node=>{if(!node||node.closest('[hidden]'))return false;const r=node.getBoundingClientRect(),s=getComputedStyle(node);return r.width>0&&r.height>0&&s.visibility!=='hidden'&&s.display!=='none'};
 const hit=node=>{if(!visible(node))return false;const r=node.getBoundingClientRect();if(r.left< -1||r.right>innerWidth+1||r.top< -1||r.bottom>innerHeight+1)return false;const target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const capture=()=>({saved:saved(),draft:draft(),saves:saves(),open:opened(),dirty:status()?.dataset.dirty,decision:status()?.dataset.decision,focus:active()?.id||active()?.tagName,preparation:get('preparation')?.value});
 const open=async label=>{await focus(get('edit'),label+' registered opener');click('edit');await wait(label+' actual visible Pane and registered mounted focus',()=>opened()&&visible(get('form'))&&actualPane().contains(active()))};
 const closed=async(label,returned=true)=>wait(label,()=>!opened()&&actualPane()?.dataset.expanded==='false'&&(!returned||active()===get('edit')));
 const same=(a,b)=>a.name===b.name&&a.location===b.location&&a.note===b.note;
 const initial={name:'Motor 001',location:'Line 1 / Bay 2',note:'Inspect coupling and guards.'};
 await wait('actual gallery initial record and controls',()=>status()&&record()&&get('edit')&&same(saved(),initial));
 await wait('native document focus',()=>document.hasFocus());
 assert(!opened()&&saves()===0,'initial editor is closed with actual initial saved equipment');
 for(const [field,value] of Object.entries(initial))assert(document.getElementById('pane-equipment-saved-'+field)?.textContent.trim()===value,'visible saved '+field+' matches actual record');
 await input('preparation','Bring the local inspection checklist.');snapshots.initial=capture();

 await open('validation');await focus(get('name'),'equipment name');await input('name','');
 assert(get('name').required&&!get('form').checkValidity(),'actual native required name invalidates form');
 await wait('changed draft enables actual Save',()=>status().dataset.dirty==='true'&&!get('save').disabled);
 let invalid=0;get('form').addEventListener('invalid',()=>invalid++,true);
 await focus(get('save'),'invalid draft Save');click('save');
 await wait('native Save performs required validation',()=>invalid>0);
 assert(opened()&&saves()===0&&same(saved(),initial),'invalid native submit preserves saved equipment and open editor');
 assert(!get('form').noValidate&&get('payload').textContent.trim()==='{}','native required validation blocks owner submit before any payload');
 snapshots.validation={...capture(),invalidEvents:invalid,nativeRequired:get('name').required,formNoValidate:get('form').noValidate};
 const updated={name:'Motor 001 / inspected',location:'Line 2 / Bay 4',note:'Check alignment before release.'};
 for(const [field,value] of Object.entries(updated))await input(field,value);
 const formData=Object.fromEntries(new FormData(get('form')).entries());
 assert(formData.name===updated.name&&formData.location===updated.location&&formData.note===updated.note,'native successful controls contain exact equipment values');
 await focus(get('save'),'actual Save');click('save');
 await wait('native Save updates real record once',()=>saves()===1&&same(saved(),updated));
 await closed('Save closes and returns registered opener');const payload=JSON.parse(get('payload').textContent);assert(same(payload,updated),'actual FormEvent payload matches native submitted fields');snapshots.save={...capture(),nativeFormData:formData,payload};

 await open('dirty header close');await input('name','Uncommitted motor draft');await wait('dirty draft registered before close request',()=>status().dataset.dirty==='true');await focus(header(),'Pane header close');header().click();
 await wait('dirty header request is vetoed and offers real Keep/Discard controls',()=>opened()&&decision()&&visible(get('keep'))&&visible(get('discard')));
 await focus(get('keep'),'Keep editing');click('keep');
 await wait('Keep retains dirty draft and restores usable inside focus',()=>opened()&&!decision()&&get('name').value==='Uncommitted motor draft'&&actualPane().contains(active())&&visible(active()));
 assert(same(saved(),updated)&&saves()===1,'Keep does not save draft');snapshots.keep=capture();
 await focus(get('note'),'dirty Escape field');const escape=key(active(),'Escape');
 await wait('synthetic Escape requests dirty-close confirmation',()=>opened()&&decision());keys.escape={synthetic:true,defaultPrevented:escape.defaultPrevented};
 await focus(get('discard'),'Discard changes');click('discard');await closed('Discard closes and returns opener');
 assert(same(saved(),updated)&&saves()===1,'Discard preserves saved record');
 await open('restored saved draft');assert(same(draft(),updated),'fresh editor restores saved values after discard');snapshots.discard=capture();

 const tabs=()=>[...actualPane().querySelectorAll('button,input,textarea,select,[tabindex]')].filter(node=>visible(node)&&!node.disabled&&node.tabIndex>=0);
 const first=tabs()[0],last=tabs().at(-1);assert(first&&last&&first!==last,'Pane contains multiple actual native focus controls');
 await focus(last,'last registered Pane control');await wait('last focus observed by Pane',()=>actualPane().dataset.paneFocusCurrent===last.id);
 const forward=key(last,'Tab');await wait('synthetic forward boundary Tab wraps through existing Pane handler',()=>active()===first);
 await wait('first focus observed by Pane',()=>actualPane().dataset.paneFocusCurrent===first.id);
 const reverse=key(first,'Tab',true);await wait('synthetic reverse boundary Tab wraps through existing Pane handler',()=>active()===last);
 keys.boundaryTab={synthetic:true,forwardDefaultPrevented:forward.defaultPrevented,reverseDefaultPrevented:reverse.defaultPrevented,first:first.id,last:last.id};
 await input('location','Unsaved outside-close location');await wait('outside-close draft registered',()=>status().dataset.dirty==='true');await focus(get('preparation'),'independent outside preparation');const outsideFocusBefore=active().id;get('preparation').click();
 await wait('actual outside click veto preserves dirty editor',()=>opened()&&decision());
 assert(active()===get('preparation'),'outside close veto preserves independently focused preparation');
 await focus(get('keep'),'Keep after outside request');click('keep');await wait('outside Keep retains draft',()=>opened()&&!decision()&&get('location').value==='Unsaved outside-close location');
 await focus(get('cancel'),'explicit Cancel');click('cancel');await closed('explicit Cancel discards and returns opener');
 assert(same(saved(),updated)&&get('preparation').value==='Bring the local inspection checklist.','Cancel retains saved record and independent preparation');snapshots.outsideAndCancel={...capture(),outsideFocusBefore};

 click('floating');await wait('inline presentation is actual Pane state',()=>actualPane()?.dataset.floating==='false');
 await focus(get('edit'),'inline opener');click('edit');await wait('inline editor opens with saved values',()=>opened()&&same(draft(),updated));
 await focus(get('cancel'),'inline Cancel');click('cancel');await closed('inline Cancel closes',false);
 click('reset');await wait('scoped Reset restores actual initial saved equipment',()=>saves()===0&&same(saved(),initial)&&!opened());
 assert(!get('floating').checked&&get('preparation').value==='Bring the local inspection checklist.','scoped Reset retains option and preparation state');
 snapshots.reset=capture();click('floating');await wait('floating restored for bounded narrow preview',()=>actualPane()?.dataset.floating==='true');
 document.getElementById('pane-equipment-narrow').click();await wait('actual preview frame narrows',()=>document.getElementById('pane-equipment-frame').getBoundingClientRect().width<=341);
 await open('narrow frame');await input('note','Narrow local draft');await wait('narrow draft enables actual Save',()=>!get('save').disabled);
 const measurements={};
 geometry.narrow={viewport:{width:innerWidth,height:innerHeight},frame:rect(document.getElementById('pane-equipment-frame')),pane:rect(actualPane()),content:rect(actualPane().querySelector('.ix-pane-content')),mobile:actualPane().dataset.mobile,floating:actualPane().dataset.floating,controls:measurements};
 for(const id of ['name','location','note','save','cancel']){
  const node=get(id);node.scrollIntoView({block:'nearest',inline:'nearest'});measurements[id]={rect:rect(node),hit:hit(node)};await wait('narrow '+id+' visible center hit',()=>hit(node));
  const r=node.getBoundingClientRect(),content=actualPane().querySelector('.ix-pane-content').getBoundingClientRect();
  assert(r.left>=content.left-1&&r.right<=content.right+1,'narrow '+id+' stays within actual Pane content');measurements[id]={rect:rect(node),hit:hit(node)};
 }
 geometry.narrow.pane=rect(actualPane());geometry.narrow.content=rect(actualPane().querySelector('.ix-pane-content'));
 await focus(get('cancel'),'narrow Cancel');click('cancel');await closed('narrow Cancel closes and returns opener');
 snapshots.final=capture();
 return {ok:true,snapshots,geometry,keys,checks:['native required validation protects saved record; FormData and exact FormEvent Save update visible equipment values','dirty header/Escape/outside requests are vetoed; Keep retains and Discard/Cancel restore saved values','existing mounted Pane focus return and synthetic boundary Tab; dirty outside veto preserves independent focus','scoped Reset retains preparation/options; actual narrow-frame fields/actions fit and receive native center hits'],input:'Excluded native DOM clicks, focus and input events; Escape/Tab are untrusted synthetic keydowns exercising Dioxus handlers, without fabricating trusted interior Tab or Enter defaults.',viewportQualification:'340px is an actual gallery frame inside the unchanged desktop WebKitGTK viewport; data-mobile and actual Pane bounds are reported. Lead Chromium covers a true narrow viewport and trusted keys.',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,geometry,keys,focus:document.activeElement?.id||document.activeElement?.tagName,viewport:{width:innerWidth,height:innerHeight},status:document.getElementById('pane-equipment-status')?.outerHTML}}

"#;
