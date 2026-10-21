//! Excluded actual-gallery inert Button/IconButton form and bounded native driver.
use dioxus::prelude::*;
use dioxus_ix::*;

use crate::gallery_actions;

#[component]
pub fn InertActionsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {include_str!("../../examples/gallery/styles/main.css")} }
        style { {gallery_actions::STYLES} }
        IxTheme { class: "review-app",
            main { class: "review-content", gallery_actions::InertActionsExample {} }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_INERT_ACTIONS_CHECK);
                        if let Err(error) = query.send("inertactions") {
                            eprintln!("Inert actions native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native inert actions: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native inert actions failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;
                        eprintln!("Inert actions native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

// Excluded synthetic DOM events and native element focus requests exercise only
// the actual shared Rust example. Browser Tab/mouse defaults are separate evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_INERT_ACTIONS_CHECK: &str = r#"
let snapshots={},checks=[],focusAttempts=[];
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,root=()=>get('inert-actions'),form=()=>get('inert-actions-form'),note=()=>get('inert-note'),suspend=()=>get('inert-suspend'),result=()=>get('inert-result'),run=()=>get('inert-run'),review=()=>get('inert-link'),refresh=()=>get('inert-icon'),before=()=>get('inert-before'),after=()=>get('inert-after'),reset=()=>get('inert-reset'),actions=()=>[run(),review(),refresh()];
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<180;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; result='+result()?.outerHTML+'; focus='+active()?.id)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)},click=async(node,label)=>{await focus(node,label);node.click()};
 const counts=()=>[Number(result().dataset.run),Number(result().dataset.review),Number(result().dataset.refresh),Number(result().dataset.submissions)],sameCounts=values=>counts().every((value,index)=>value===values[index]),submitted=()=>result().dataset.submittedNote;
 const paint=node=>{const s=getComputedStyle(node);return{class:node.className,background:s.backgroundColor,color:s.color,opacity:s.opacity,disabled:node.hasAttribute('disabled'),ariaDisabled:node.getAttribute('aria-disabled'),busy:node.getAttribute('aria-busy')}};
 const edit=async value=>{await focus(note(),'current editable note');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(note(),value);note().dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual native note value',()=>note().value===value)};
 const snapshot=()=>({counts:counts(),submitted:submitted(),suspended:result().dataset.suspended,note:note().value,focus:active()?.id,actions:actions().map(node=>({id:node.id,inert:node.inert,hasInert:node.hasAttribute('inert'),paint:paint(node)})),hash:location.hash});
 await wait('actual shared inspection form mounted',()=>root()&&form()&&note()&&suspend()&&result()&&actions().every(Boolean)&&before()&&after()&&reset());await wait('native document ready for focus',()=>document.hasFocus());
 const originalActions=actions(),originalNote=note(),originalForm=form(),initialHash=location.hash;
 assert(run().tagName==='BUTTON'&&run().type==='submit'&&run().form===form()&&review().tagName==='A'&&review().getAttribute('href')==='#inert-actions'&&refresh().tagName==='BUTTON'&&refresh().type==='button','actual submit Button, anchor Button and native IconButton retain their roles');
 assert(note().name==='note'&&note().form===form()&&note().value==='Check cooling system'&&sameCounts([0,0,0,0])&&submitted()===''&&!suspend().checked&&actions().every(node=>!node.inert&&!node.hasAttribute('inert')),'initial native form data and default absent inert');
 await edit('Inspect Motor 001 coolant');await click(run(),'enabled submit action');await wait('enabled Run consumes actual FormEvent note',()=>sameCounts([1,0,0,1])&&submitted()==='Inspect Motor 001 coolant');await click(review(),'enabled review link');await wait('enabled review records once without navigation',()=>sameCounts([1,1,0,1])&&location.hash===initialHash);await click(refresh(),'enabled refresh icon');await wait('enabled icon records once',()=>sameCounts([1,1,1,1]));snapshots.enabled=snapshot();checks.push('actual enabled submit/anchor/icon callbacks and native FormEvent publish the current named note exactly once');
 await focus(before(),'ordinary before action');const baselinePaint=actions().map(paint);await click(suspend(),'actual suspend checkbox');await wait('native inert presence follows controlled suspension',()=>suspend().checked&&result().dataset.suspended==='true'&&actions().every(node=>node.inert&&node.hasAttribute('inert')));
 assert(actions().every((node,index)=>node===originalActions[index]&&node.inert&&node.hasAttribute('inert')&&JSON.stringify(paint(node))===JSON.stringify(baselinePaint[index])),'inert retains current native nodes, enabled classes/paint and disabled/loading attributes');
 await focus(before(),'independent focus destination');for(const node of actions()){node.focus();await wait('inert native control cannot take focus '+node.id,()=>active()===before());focusAttempts.push({target:node.id,active:active().id});}snapshots.suspended=snapshot();checks.push('native inert presence preserves enabled presentation and rejects direct focus on each current action');
 const guardedCounts=counts();for(const node of [review(),refresh()])node.click();await edit('Inspection note edited while actions are suspended');assert(note()===originalNote&&form()===originalForm&&new FormData(form()).get('note')===note().value,'suspended actions leave the same editable successful native note');await click(suspend(),'resume actual checkbox');await wait('ordinary resume restores current native actions',()=>!suspend().checked&&result().dataset.suspended==='false'&&actions().every(node=>!node.inert&&!node.hasAttribute('inert')));assert(sameCounts(guardedCounts)&&submitted()==='Inspect Motor 001 coolant'&&location.hash===initialHash,'synthetic non-submit inert action guards preserve callbacks, prior submission and location');assert(actions().every((node,index)=>node===originalActions[index])&&note()===originalNote&&note().value==='Inspection note edited while actions are suspended','resume retains actual controls and editable note');snapshots.guardedResumed=snapshot();checks.push('synthetic non-submit guards preserve action records while suspended note editing and ordinary resume retain actual content');
 await click(run(),'restored submit action');await wait('restored Run submits current edited note',()=>sameCounts([2,1,1,2])&&submitted()==='Inspection note edited while actions are suspended');await click(review(),'restored review action');await wait('restored current anchor activates',()=>sameCounts([2,2,1,2])&&location.hash===initialHash);await click(refresh(),'restored current icon');await wait('restored current icon activates',()=>sameCounts([2,2,2,2]));snapshots.restored=snapshot();checks.push('removing inert restores the same native actions and current-note FormEvent without losing edits');
 await click(reset(),'actual Reset counts action');await wait('actual reset clears records and resumes actions',()=>sameCounts([0,0,0,0])&&submitted()===''&&!suspend().checked&&result().dataset.suspended==='false'&&actions().every(node=>!node.inert&&!node.hasAttribute('inert')));assert(note()===originalNote&&note().value==='Inspection note edited while actions are suspended'&&actions().every((node,index)=>node===originalActions[index]),'reset preserves editable note and current native action ownership');snapshots.reset=snapshot();checks.push('actual Reset clears action/submission records and suspension while retaining the editable note');
 return{ok:true,checks,snapshots,focusAttempts,input:'Excluded native replay imports the actual gallery InertActionsExample. Synthetic DOM click/input events and explicit native element focus requests verify current browser inert presence/focus and non-submit Rust callback guards. Programmatic inert submit activation is deliberately omitted; direct default/callback guards are separate actual-VDOM evidence, not a desktop IPC chronology requirement. The driver never synthesizes Tab or pretends an untrusted key executes browser activation defaults. Checkbox DOM clicks use its native checked/input/change behavior. Actual form defaults supply the named FormEvent note; no suspend guard is added to the gallery form. Trusted Tab/mouse/Enter/Space evidence is separate. No physical OS/AT or exact event chronology claim; no copied model, replacement state, app/controller/callback/CSS injection or production bridge.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focusAttempts,focus:document.activeElement?.id,result:document.getElementById('inert-result')?.outerHTML}}
"#;
