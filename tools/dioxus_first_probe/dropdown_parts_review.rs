//! Excluded actual-gallery native-button quick actions and passive dropdown parts.
use super::dropdown_button_review::gallery_composition;
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/feedback_scenarios.rs"]
mod gallery_feedback;

#[component]
pub fn DropdownPartsPreview() -> Element {
    debug_assert!(gallery_feedback::SCENARIOS.contains(&"dropdown"));
    debug_assert!(gallery_feedback::SCENARIOS.contains(&"dropdown-quick-actions"));
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_composition::STYLES} }
        style { {gallery_feedback::STYLES} }
        IxTheme { class:"dropdown-parts-review",
            gallery_composition::CompositionExamples {}
            section { id:"dropdown-parts-source-quick",
                h2 { "Existing quick-action source preview" }
                gallery_feedback::FeedbackScenario {name:"dropdown-quick-actions"}
            }
            section { id:"dropdown-parts-source-header",
                h2 { "Existing header source preview" }
                gallery_feedback::FeedbackScenario {name:"dropdown"}
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_DROPDOWN_PARTS_CHECK);let _=query.send("dropdownparts");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native Dropdown parts: {value}");window.close();}
                        value=>{eprintln!("Dioxus native Dropdown parts failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(60)).await;eprintln!("Dropdown parts native replay timed out");std::process::exit(1);});}
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.dropdown-parts-review{box-sizing:border-box;min-height:100vh;padding:24px}.dropdown-parts-review .demo-row{display:flex;flex-wrap:wrap;align-items:center;gap:8px;margin-block:12px}.dropdown-parts-review h2{font-size:24px;line-height:32px}.dropdown-parts-review>section{margin-bottom:24px}@media(max-width:600px){.dropdown-parts-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_DROPDOWN_PARTS_CHECK: &str = r#"
let snapshots={},keys=[],geometry={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),assert=(v,label)=>{if(!v)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<250;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; focus='+document.activeElement?.outerHTML)};
 const scope=id=>get(id),trigger=id=>scope(id)?.querySelector('[aria-haspopup="menu"][aria-controls]'),menu=id=>get(trigger(id)?.getAttribute('aria-controls')),opened=id=>trigger(id)?.getAttribute('aria-expanded')==='true'&&menu(id)&&!menu(id).hidden,feedback=id=>scope(id)?.querySelector('[role=status]')?.textContent.trim(),quick=id=>menu(id)?.querySelector('[role=group][aria-label="Quick actions"]'),action=(id,label)=>quick(id)?.querySelector('button[aria-label="'+label+'"]'),item=(id,label)=>[...menu(id)?.querySelectorAll('[role=menuitem]')??[]].find(n=>n.textContent.trim()===label);
 const focus=async node=>{assert(node,'actual focus target exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait('native focus reaches current control',()=>document.activeElement===node)};
 const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const open=async id=>{await focus(trigger(id));trigger(id).click();await wait('actual current popup opens '+id,()=>opened(id))};
 const closed=async id=>wait('normal Both inside close '+id,()=>!opened(id));
 const rect=n=>{const r=n.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom}};
 const hit=async(n,label)=>{await focus(n);await wait(label+' actual visible native center hit',()=>{const r=n.getBoundingClientRect(),point=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return r.width>0&&r.height>0&&r.x>=0&&r.right<=innerWidth+1&&r.y>=0&&r.bottom<=innerHeight+1&&(point===n||n.contains(point))});return{rect:rect(n),label:n.getAttribute('aria-label'),viewport:{width:innerWidth,height:innerHeight}}};
 const validate=id=>{const group=quick(id),controls=[...group.querySelectorAll('button')];assert(group.getAttribute('role')==='group'&&controls.length===3,'exact three ordinary native buttons in named group');for(const label of ['Cut','Light','Delete']){const n=action(id,label);assert(n&&n.type==='button'&&!n.disabled&&n.tabIndex>=0&&n.getAttribute('role')!=='menuitem'&&!n.hasAttribute('data-ix-dropdown-item'),'native '+label+' is composed button, not typed menuitem');}const divider=menu(id).querySelector('[role=separator]');assert(divider&&divider.getAttribute('aria-orientation')==='horizontal'&&divider.tabIndex<0&&!divider.querySelector('button,[tabindex]'),'Divider remains passive horizontal separator');assert(menu(id).querySelectorAll('[role=menuitem]').length===3,'regular menu items remain separate');};
 const main='dropdown-quick-actions',source='dropdown-parts-source-quick',header='dropdown-parts-source-header';await wait('actual main/source examples ready',()=>trigger(main)&&trigger(source)&&trigger(header)&&menu(main)&&menu(source)&&menu(header));
 for(const id of [main,source]){
  const records=[];
  for(const [label,activation] of [['Cut','Enter'],['Light',' '],['Delete','Tab']]){
   await open(id);validate(id);const n=action(id,label);geometry[id+'-'+label]=await hit(n,id+' '+label);const event=key(n,activation);assert(!event.defaultPrevented,'ordinary quick button admits '+activation+' default');keys.push({scope:id,label,key:activation,synthetic:true,defaultPrevented:event.defaultPrevented});if(activation==='Tab'){const reverse=key(n,'Tab',true);assert(!reverse.defaultPrevented,'ordinary quick button admits Shift+Tab default');keys.push({scope:id,label,key:'Shift+Tab',synthetic:true,defaultPrevented:reverse.defaultPrevented});}
   // Untrusted keydown does not perform native button activation. The standard
   // DOM click independently invokes the real existing onclick callback.
   n.click();await wait('actual action feedback '+label,()=>feedback(id)===label);await closed(id);records.push({action:label,feedback:feedback(id),closed:!opened(id),focus:document.activeElement.id});
  }
  snapshots[id]={records};
 }
 await open(main);await focus(trigger(main));key(trigger(main),'Home');await wait('first typed menu item active',()=>trigger(main).getAttribute('aria-activedescendant')===item(main,'Item 1').id||item(main,'Item 1').dataset.active==='true');key(trigger(main),'ArrowDown');await wait('normal menu arrow reaches next actual item',()=>document.activeElement===item(main,'Item 2')||trigger(main).getAttribute('aria-activedescendant')===item(main,'Item 2').id||item(main,'Item 2').dataset.active==='true');key(document.activeElement,'Enter');await wait('ordinary item Enter invokes Item2',()=>feedback(main)==='Item 2');await closed(main);await open(main);await focus(trigger(main));key(trigger(main),'Escape');await closed(main);snapshots.regular={feedback:feedback(main),closed:!opened(main)};
 await open(header);const heading=menu(header).querySelector('.ix-dropdown-header'),divider=menu(header).querySelector('[role=separator]');assert(heading?.textContent.trim()==='Category'&&heading.getAttribute('role')==='presentation'&&heading.tabIndex<0&&!heading.querySelector('button,input,select,textarea,[tabindex]'),'DropdownHeader category is noninteractive presentation');assert(divider&&divider.tabIndex<0&&divider.getAttribute('aria-orientation')==='horizontal'&&!divider.querySelector('button,[tabindex]'),'source Divider has no interactive descendants/extra Tab stop');assert(menu(header).querySelectorAll('[role=menuitem]').length===4,'source Header is not registered as a fifth menu item');await focus(trigger(header));key(trigger(header),'Home');await wait('first source typed item active',()=>trigger(header).getAttribute('aria-activedescendant')===item(header,'Item 2').id||item(header,'Item 2').dataset.active==='true');key(trigger(header),'ArrowDown');await wait('header/divider skipped by ordinary item arrow',()=>document.activeElement===item(header,'Item 3')||trigger(header).getAttribute('aria-activedescendant')===item(header,'Item 3').id||item(header,'Item 3').dataset.active==='true');key(document.activeElement,'Enter');await wait('source ordinary item activation updates existing model',()=>feedback(header)==='Item 3');await closed(header);snapshots.header={text:heading.textContent,role:heading.getAttribute('role'),dividerRole:divider.getAttribute('role'),feedback:feedback(header)};
 return{ok:true,snapshots,keys,geometry,checks:['actual main named Quick actions group composes three native IconButtons; original Cut/Light/Delete callbacks close Both and retain recorded physical focus','actual source Quick actions composition preserves the same three native actions, focus/hits and ordinary close/reopen behavior','regular dropdown item Arrow/Enter remains independent of quick buttons; Escape closes/reopens without changing saved action','actual source Header is noninteractive presentation and Divider horizontal separator with no extra Tab stops; ordinary item navigation skips them'],input:'Excluded synthetic DOM keydown, standard DOM click and explicit current native focus invoke actual unchanged gallery models/compositions. Enter/Space/Tab/ShiftTab event metadata is recorded for ordinary buttons, but untrusted keydown does not perform or certify native activation/Tab defaults; independent DOM click invokes the actual callback. Existing typed menu-item Arrow/Enter/Escape Rust handlers are exercised. Trusted automated Chromium separately verifies real pointer/native Enter/Space/Tab. No app/model/controller/callback/CSS injection, copied business state, new JS runtime or physical OS/AT claim.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),snapshots,keys,geometry}}
"#;
