//! Excluded actual-gallery native action attributes and bounded DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

use crate::gallery_actions;

#[component]
pub fn ActionAttributesPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {include_str!("../../examples/gallery/styles/main.css")} }
        style { {gallery_actions::STYLES} }
        IxTheme { class: "review-app",
            main { class: "review-content", gallery_actions::ActionExamples {} }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_ACTION_ATTRIBUTES_CHECK);
                        if let Err(error) = query.send("actionattributes") {
                            eprintln!("Action attributes native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native action attributes: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native action attributes failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;
                        eprintln!("Action attributes native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

// Excluded synthetic clicks/input and explicit native focus exercise the actual
// shared example. Trusted key defaults, Tab and physical/AT behavior are separate.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_ACTION_ATTRIBUTES_CHECK: &str = r#"
let checks=[],snapshots={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),start=()=>get('actions-start'),reset=()=>get('actions-reset-loading'),status=()=>get('actions-status'),toggle=()=>get('actions-toggle'),favorite=()=>get('actions-favorite'),toggleStatus=()=>get('toggle-status'),upstream=()=>get('actions-upstream'),note=()=>get('inert-note'),icon=size=>get('actions-icon-'+size),sizes=[12,16,20,24];
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<180;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; action='+status()?.textContent+'; toggle='+toggleStatus()?.textContent+'; focus='+document.activeElement?.id)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>document.activeElement===node)},click=async(node,label)=>{await focus(node,label);node.click()};
 const relationship=(node,controls,description)=>node?.getAttribute('aria-controls')===controls&&(!description||node.getAttribute('aria-describedby')===description)&&get(controls)?.getAttribute('role')==='status'&&(!description||!!get(description));
 const metadata=()=>({start:{controls:start().getAttribute('aria-controls'),mode:start().getAttribute('data-inspection-mode'),busy:start().getAttribute('aria-busy'),disabled:start().disabled},icons:sizes.map(size=>({id:icon(size).id,name:icon(size).getAttribute('aria-label'),controls:icon(size).getAttribute('aria-controls'),description:icon(size).getAttribute('aria-describedby')})),toggle:{controls:toggle().getAttribute('aria-controls'),description:toggle().getAttribute('aria-describedby'),pressed:toggle().getAttribute('aria-pressed')},favorite:{controls:favorite().getAttribute('aria-controls'),description:favorite().getAttribute('aria-describedby'),pressed:favorite().getAttribute('aria-pressed')},link:{id:upstream().id,href:upstream().getAttribute('href'),target:upstream().getAttribute('target'),description:upstream().getAttribute('aria-describedby')}});
 const snapshot=()=>({metadata:metadata(),action:status().textContent,toggle:toggleStatus().textContent,note:note().value,focus:document.activeElement?.id});
 await wait('actual shared action examples mounted',()=>start()&&reset()&&status()&&toggle()&&favorite()&&toggleStatus()&&upstream()&&note()&&sizes.every(size=>icon(size)));
 await wait('native document ready for focus',()=>document.hasFocus());
 assert(start().tagName==='BUTTON'&&reset().tagName==='BUTTON'&&relationship(start(),'actions-status')&&relationship(reset(),'actions-status'),'Button metadata belongs to actual native controls and existing status');
 assert(sizes.every(size=>icon(size).tagName==='BUTTON'&&icon(size).getAttribute('aria-label')==='Add, '+size+' pixel icon'&&relationship(icon(size),'actions-status','actions-help')),'IconButton names and native relationships retain all existing sizes');
 assert(toggle().tagName==='BUTTON'&&favorite().tagName==='BUTTON'&&favorite().getAttribute('aria-label')==='Favorite'&&relationship(toggle(),'toggle-status','toggle-actions-help')&&relationship(favorite(),'toggle-status','toggle-actions-help'),'ToggleButton and IconToggleButton metadata points to the actual helper and status');
 assert(upstream().tagName==='A'&&upstream().getAttribute('href')==='https://github.com/siemens/ix'&&upstream().getAttribute('target')==='_blank'&&upstream().getAttribute('aria-describedby')==='actions-upstream-help'&&get('actions-upstream-help').textContent.includes('new tab'),'LinkButton forwards native identity and factual description without external activation');
 assert(start().getAttribute('data-inspection-mode')==='false'&&toggle().getAttribute('aria-pressed')==='false'&&favorite().getAttribute('aria-pressed')==='true','initial native metadata reflects current shared state');snapshots.initial=snapshot();checks.push('actual native Button/IconButton/ToggleButton/IconToggleButton/LinkButton retain declared identity, names and helper/status relationships');
 const originalNote=note(),noteText='Keep inspection notes while changing action options';await focus(note(),'existing inspection note');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(note(),noteText);note().dispatchEvent(new Event('input',{bubbles:true,composed:true}));
 await click(toggle(),'current ToggleButton');await wait('pressed state and forwarded data attribute update together',()=>toggle().getAttribute('aria-pressed')==='true'&&start().getAttribute('data-inspection-mode')==='true'&&toggleStatus().textContent.includes('Toggle pressed: true. Favorite: true.'));
 await click(favorite(),'current IconToggleButton');await wait('favorite state updates independently through the actual callback',()=>favorite().getAttribute('aria-pressed')==='false'&&toggle().getAttribute('aria-pressed')==='true'&&start().getAttribute('data-inspection-mode')==='true'&&toggleStatus().textContent.includes('Toggle pressed: true. Favorite: false.'));
 assert(note()===originalNote&&note().value===noteText&&relationship(toggle(),'toggle-status','toggle-actions-help')&&relationship(favorite(),'toggle-status','toggle-actions-help'),'parent state changes preserve actual inspection note and native relationships');snapshots.live=snapshot();checks.push('actual controlled toggles update pressed/status/data metadata independently while retaining the nested inspection note');
 for(const size of sizes){await click(icon(size),'actual '+size+' pixel IconButton');await wait('current icon callback updates described status '+size,()=>status().textContent===size+' pixel button clicked'&&relationship(icon(size),'actions-status','actions-help'));assert(note()===originalNote&&note().value===noteText,'icon callback rerender retains the existing inspection note');}snapshots.icons=snapshot();checks.push('existing icon-size callbacks update their declared actual status without replacing the editable nested note');
 await click(start(),'actual Start action');await wait('loading action remains natively disabled with current metadata',()=>start().disabled&&start().getAttribute('aria-busy')==='true'&&start().getAttribute('aria-disabled')==='true'&&relationship(start(),'actions-status')&&start().getAttribute('data-inspection-mode')==='true');
 const guards=[start(),get('icon-buttons').querySelector('button[aria-label="Disabled action"]'),get('icon-buttons').querySelector('button[aria-label="Loading action"]')];assert(guards.every(node=>node?.disabled&&node.getAttribute('aria-disabled')==='true'),'existing disabled/loading controls retain their native guards');const beforeGuard=status().textContent;for(const node of guards)node.click();assert(status().textContent===beforeGuard,'native disabled DOM clicks leave the current status unchanged');
 await click(reset(),'actual Reset loading');await wait('reset callback restores usable Start with unchanged metadata',()=>!start().disabled&&start().getAttribute('aria-busy')==='false'&&start().getAttribute('aria-disabled')==='false'&&status().textContent==='Action reset'&&relationship(start(),'actions-status')&&start().getAttribute('data-inspection-mode')==='true');
 assert(beforeGuard==='24 pixel button clicked'&&note()===originalNote&&note().value===noteText&&new FormData(note().form).get('note')===noteText,'loading/reset parent changes preserve actual successful note');
 await click(toggle(),'restore current ToggleButton');await click(favorite(),'restore current IconToggleButton');await wait('normal current toggles restore initial state and live metadata',()=>toggle().getAttribute('aria-pressed')==='false'&&favorite().getAttribute('aria-pressed')==='true'&&start().getAttribute('data-inspection-mode')==='false'&&toggleStatus().textContent.includes('Toggle pressed: false. Favorite: true.'));assert(note()===originalNote&&note().value===noteText,'restoring parent flags preserves the edited note');snapshots.restored=snapshot();checks.push('existing loading/disabled native guards and Reset callback preserve metadata/note; ordinary toggles restore current state');
 return{ok:true,checks,snapshots,input:'Excluded native preview imports actual shared ActionExamples. Synthetic DOM click/input events and explicit native element focus exercise Rust callbacks and observe native metadata, status, pressed/loading guards and retained note. The external LinkButton is observed only and never activated. No untrusted key is represented as native Enter/Space/Tab default execution; trusted browser evidence is separate. No physical OS/AT or exact chronology claim, copied state/model, app/controller/callback/CSS injection or production bridge. The separately closed inertactions scope is unchanged.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:document.activeElement?.id,action:document.getElementById('actions-status')?.textContent,toggle:document.getElementById('toggle-status')?.textContent}}
"#;
