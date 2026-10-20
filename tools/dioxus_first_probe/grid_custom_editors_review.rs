//! Excluded actual-gallery Dioxus-composed Grid editors and Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/grid_custom_editors.rs"]
mod gallery_custom;

#[component]
pub fn GridCustomEditorsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_custom::STYLES} }
        IxTheme { class: "grid-custom-editors-review",
            h1 { "Composed assignment editors" }
            Button { id: "custom-editors-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "custom-editors-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:1100px;max-width:100%;min-width:0" },
                gallery_custom::GridCustomEditorsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_GRID_CUSTOM_EDITORS_CHECK);
                        if let Err(error) = query.send("gridcustomeditors") {
                            eprintln!("Custom Grid editor native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native custom grid editors: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native custom grid editors failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Custom Grid editor native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.grid-custom-editors-review{box-sizing:border-box;min-height:100vh;padding:24px}.grid-custom-editors-review h1{font-size:28px;margin:0 0 24px}.grid-custom-editors-review h2{font-size:22px;margin:24px 0 16px}.grid-custom-editors-review h3{font-size:18px}.grid-custom-editors-review p{max-width:72ch;line-height:1.5}#custom-editors-frame{margin-top:24px}@media(max-width:600px){.grid-custom-editors-review{padding:16px}}";

// Excluded driver JS sends synthetic events and explicitly requests native focus.
// Actual Dioxus components own draft, validation, editor lifetime and navigation.
// Untrusted Tab/Enter/Space cannot execute native browser defaults: internal Tab
// permission and button Enter permission are checked, then explicit focus/click
// exercises those controls. Trusted Chromium separately checks their defaults.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_CUSTOM_EDITORS_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('grid-custom-editors'),result=()=>get('custom-grid-result');
 const grid=()=>get('custom-grid')?.querySelector('[role="grid"][aria-label="Maintenance assignments"]');
 const rows=()=>[...grid()?.querySelectorAll('.ix-data-grid-body [role="row"][data-row-id]')??[]];
 const row=id=>rows().find(node=>node.dataset.rowId===id);
 const cell=(id,column)=>row(id)?.querySelector('[role="gridcell"][data-column-id="'+column+'"]');
 const owner=()=>host()?.querySelector('#custom-owner-input:not([disabled])');
 const ownerAt=id=>cell(id,'owner')?.querySelector('#custom-owner-input:not([disabled])');
 const customControl=id=>host()?.querySelector('#'+id+':not([disabled])');
 const builtin=()=>get('custom-grid')?.querySelector(':is(input,select).ix-data-grid-editor[data-retiring="false"]');
 const builtinAt=(id,column)=>cell(id,column)?.querySelector(':is(input,select).ix-data-grid-editor[data-retiring="false"]');
 const error=()=>get('custom-grid')?.querySelector('.ix-data-grid-edit-error[role="alert"]');
 const requests=()=>Number(result()?.dataset.requests);
 const storedRow=id=>get('custom-grid-values')?.querySelector('[data-row-id="'+id+'"]');
 const stored=(id,column)=>storedRow(id)?.querySelector('[data-column-id="'+column+'"]')?.textContent.trim();
 const checked=node=>node instanceof HTMLInputElement?node.checked:node.getAttribute('aria-checked')==='true';
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; result='+result()?.textContent+'; focus='+(active()?.getAttribute('aria-label')||active()?.dataset?.columnId||active()?.id||active()?.tagName));
 };
 const key=(node,key,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key,code:key===' '?'Space':key,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setDraft=async value=>{
  const node=owner();assert(node,'actual active composed owner input exists');
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait('composed draft '+value+' applied',()=>owner()?.value===value&&node.isConnected);
 };
 const begin=async(id,method='F2')=>{
  await focus(cell(id,'owner'),id+' owner cell');
  if(method==='dblclick')cell(id,'owner').dispatchEvent(new MouseEvent('dblclick',{bubbles:true,cancelable:true}));else key(active(),method);
  await wait(id+' composed primary mounted control focused',()=>ownerAt(id)&&active()===ownerAt(id));
  assert(owner().type==='text'&&owner().getAttribute('aria-label')?.includes(id)&&!owner().hasAttribute('data-select-all'),'composed primary uses explicit native mount and accessible label without built-in selection adapter');
  return owner();
 };
 const finish=async(id,kind='cancel')=>{
  if(kind==='Escape')key(owner(),'Escape');else customControl('custom-owner-'+kind).click();
  await wait(kind+' closes composed editor and returns its stable cell',()=>!owner()&&!builtin()&&active()===cell(id,'owner'));
 };
 const reset=async()=>{
  get('custom-grid-reset').focus();get('custom-grid-reset').click();
  await wait('Restore assignments clears controlled data/modes/count/editor and restores rows',()=>requests()===0&&rows().length===3&&rows()[0]?.dataset.rowId==='task-1'&&stored('task-1','owner')==='Operations'&&stored('task-2','owner')==='Service team'&&stored('task-3','owner')==='Operations'&&stored('task-1','done')==='false'&&!checked(get('custom-grid-hold'))&&!checked(get('custom-grid-readonly'))&&!owner()&&!builtin()&&!error());
 };
 const snapshot=()=>({requests:requests(),order:rows().map(node=>node.dataset.rowId),owners:['task-1','task-2','task-3'].map(id=>({id,owner:stored(id,'owner'),done:stored(id,'done')})),held:checked(get('custom-grid-hold')),readonly:checked(get('custom-grid-readonly')),latest:result()?.textContent.trim()});

 await wait('actual custom-editor gallery and independent stored-value table mounted',()=>host()&&grid()&&result()&&rows().length===3&&stored('task-1','owner')==='Operations'&&stored('task-1','done')==='false');
 await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(requests()===0&&cell('task-3','owner').getAttribute('aria-readonly')==='true','initial counters and per-row readonly owner contract');
 await begin('task-1');assert(owner().value==='Operations'&&owner().getAttribute('list')&&get(owner().getAttribute('list'))?.querySelectorAll('option').length===3,'actual composed input exposes owner draft and native team suggestions');
 await setDraft('Inspection team');assert(stored('task-1','owner')==='Operations'&&requests()===0,'custom draft remains separate from stored host values');
 customControl('custom-owner-quick').click();await wait('quick assignment updates draft only',()=>owner()?.value==='Service team'&&stored('task-1','owner')==='Operations'&&requests()===0);
 await finish('task-1','save');await wait('Save applies one controlled owner request',()=>requests()===1&&stored('task-1','owner')==='Service team');
 assert(row('task-1').getAttribute('aria-selected')==='false','custom editor controls do not select their containing Grid row');snapshots.composedSave=snapshot();

 await begin('task-1','dblclick');await setDraft('Discard button draft');await finish('task-1','cancel');assert(requests()===1&&stored('task-1','owner')==='Service team','Cancel discards owner draft without request');
 await begin('task-1');await setDraft('Discard Escape draft');await finish('task-1','Escape');assert(requests()===1&&stored('task-1','owner')==='Service team','Escape discards owner draft without request');
 await begin('task-1');await setDraft('Operations');key(owner(),'Enter');
 await wait('unconsumed Enter saves composed draft and returns its cell',()=>!owner()&&!builtin()&&requests()===2&&stored('task-1','owner')==='Operations'&&active()===cell('task-1','owner'));
 await begin('task-1');await setDraft('');customControl('custom-owner-save').click();
 await wait('shared required validation retains custom editor with accessible error',()=>owner()&&owner().getAttribute('aria-invalid')==='true'&&error()?.textContent.trim().length>0);
 assert(requests()===2&&stored('task-1','owner')==='Operations','invalid custom draft does not emit a request or overwrite host');
 await finish('task-1','Escape');assert(!error(),'cancel clears shared validation error');snapshots.cancelValidation=snapshot();

 await begin('task-1');
 const internalTab=key(owner(),'Tab');keyPermissions.inputTab=!internalTab.defaultPrevented;
 assert(!internalTab.defaultPrevented&&owner()&&!builtin(),'internal custom Tab permits native focus traversal instead of Grid advance');
 await focus(customControl('custom-owner-quick'),'quick assignment native button');
 const enter=key(active(),'Enter');keyPermissions.quickEnter=!enter.defaultPrevented;
 assert(!enter.defaultPrevented&&owner()&&requests()===2,'custom button consumes propagation while preserving native Enter activation');
 active().click();await wait('explicit native button click changes composed draft',()=>owner()?.value==='Service team'&&requests()===2);
 const quickTab=key(customControl('custom-owner-quick'),'Tab');assert(!quickTab.defaultPrevented&&owner(),'quick button keeps internal native Tab permission');
 await focus(customControl('custom-owner-save'),'custom Save native button');const saveTab=key(active(),'Tab');assert(!saveTab.defaultPrevented&&owner(),'Save keeps internal native Tab permission');
 await focus(customControl('custom-owner-cancel'),'custom Cancel native button');key(active(),'Tab');
 await wait('last custom-control Tab commits and enters built-in checkbox fallback',()=>!owner()&&builtinAt('task-1','done')&&active()===builtinAt('task-1','done')&&requests()===3&&stored('task-1','owner')==='Service team');
 assert(builtin().type==='checkbox','Grid boundary advance retains actual native checkbox when custom renderer returns None');
 builtin().click();await wait('built-in checkbox draft toggled by explicit click',()=>builtin()?.checked===true);key(builtin(),'Tab');
 await wait('built-in checkbox Tab saves and enters next row custom owner',()=>ownerAt('task-2')&&active()===ownerAt('task-2')&&requests()===4&&stored('task-1','done')==='true');
 key(owner(),'Tab',true);await wait('first input ShiftTab returns to previous row built-in checkbox',()=>builtinAt('task-1','done')&&active()===builtinAt('task-1','done')&&!owner());
 key(builtin(),'Escape');await wait('built-in Escape restores containing done cell',()=>!builtin()&&active()===cell('task-1','done'));
 await focus(cell('task-2','done'),'second row done cell');key(active(),'F2');await wait('second checkbox fallback focused',()=>builtinAt('task-2','done')&&active()===builtinAt('task-2','done'));
 key(builtin(),'Tab');await wait('advance skips third readonly custom owner',()=>builtinAt('task-3','done')&&active()===builtinAt('task-3','done')&&!owner());
 key(builtin(),'Escape');await wait('readonly-skipping traversal cancellation complete',()=>!builtin()&&active()===cell('task-3','done'));assert(requests()===4,'unchanged cross-kind traversal does not invent requests');snapshots.crossKind=snapshot();

 await reset();get('custom-grid-hold').click();await wait('host hold mode enabled',()=>checked(get('custom-grid-hold')));
 await begin('task-1');await setDraft('Inspection team');await finish('task-1','save');
 await wait('held request is recorded without applying controlled data',()=>requests()===1&&stored('task-1','owner')==='Operations');
 await begin('task-1');assert(owner().value==='Operations','reopened held editor starts from current stored owner');await finish('task-1','Escape');snapshots.held=snapshot();
 await reset();await begin('task-1');await setDraft('Service team');get('custom-grid-conflict').click();
 await wait('external host change is visible independently of pending draft',()=>stored('task-1','owner')==='Inspection team'&&owner()?.value==='Service team');
 customControl('custom-owner-save').click();
 await wait('conflicting original owner retains custom draft with shared error',()=>owner()&&error()?.textContent.trim().length>0&&owner().getAttribute('aria-invalid')==='true');
 assert(requests()===0&&stored('task-1','owner')==='Inspection team','conflict never overwrites external owner or emits an accepted request');snapshots.conflict=snapshot();await finish('task-1','Escape');

 await reset();await focus(cell('task-3','owner'),'per-row readonly owner');key(active(),'F2');
 await wait('per-row readonly owner does not start custom editor',()=>!owner()&&!builtin()&&active()===cell('task-3','owner'));
 get('custom-grid-readonly').click();await wait('whole Grid readonly reaches editable owner/done cells',()=>checked(get('custom-grid-readonly'))&&cell('task-1','owner').getAttribute('aria-readonly')==='true'&&cell('task-1','done').getAttribute('aria-readonly')==='true');
 await focus(cell('task-1','owner'),'whole Grid readonly owner');key(active(),'F2');await wait('whole Grid readonly blocks custom opening',()=>!owner()&&!builtin()&&active()===cell('task-1','owner'));
 key(active(),' ');await wait('readonly Grid preserves ordinary row selection',()=>row('task-1').getAttribute('aria-selected')==='true');
 assert(requests()===0,'readonly attempts do not emit requests');snapshots.readonly=snapshot();

 await reset();get('custom-grid-reverse').click();await wait('host reverse preserves stable row values',()=>rows().map(node=>node.dataset.rowId).join(',')==='task-3,task-2,task-1');
 await begin('task-1');await setDraft('Inspection team');await finish('task-1','save');
 await wait('custom request applies stable row ID after reversal',()=>requests()===1&&stored('task-1','owner')==='Inspection team'&&stored('task-2','owner')==='Service team');snapshots.reverse=snapshot();
 await reset();await begin('task-1');await setDraft('Removed pending draft');get('custom-grid-remove').click();
 await wait('removing active row clears custom editor and leaves surviving data',()=>!row('task-1')&&!storedRow('task-1')&&!owner()&&!builtin()&&rows().length===2&&requests()===0&&stored('task-2','owner')==='Service team');
 await begin('task-2');await setDraft('Inspection team');await finish('task-2','save');await wait('new session after removal remains usable',()=>requests()===1&&stored('task-2','owner')==='Inspection team');snapshots.removal=snapshot();

 await begin('task-2');await setDraft('');customControl('custom-owner-save').click();await wait('pending validation error ready for reset cleanup',()=>owner()&&error());
 get('custom-grid-hold').click();get('custom-grid-reset').focus();await reset();
 assert(active()===get('custom-grid-reset'),'toolbar Restore assignments retains native focus');snapshots.reset=snapshot();

 get('custom-editors-narrow').click();await wait('340px custom Grid keeps horizontal overflow internal',()=>get('custom-editors-frame').getBoundingClientRect().width<=341&&grid().scrollWidth>grid().clientWidth);
 await begin('task-1');
 await wait('mounted composed primary input is visible and hit-testable in narrow scrolling Grid',()=>{
  const node=owner(),box=node?.getBoundingClientRect(),viewport=grid()?.getBoundingClientRect();
  if(!box||box.width<=0||box.height<=0||box.left<viewport.left-1||box.right>viewport.left+grid().clientWidth+1||box.top<0||box.bottom>innerHeight+1)return false;
  const hit=document.elementFromPoint(box.left+box.width/2,box.top+box.height/2);return hit===node||node.contains(hit);
 });
 const box=grid().closest('.ix-data-grid').getBoundingClientRect(),inputBox=owner().getBoundingClientRect();
 layout={width:box.width,height:box.height,rowHeight:row('task-1').getBoundingClientRect().height,clientWidth:grid().clientWidth,scrollWidth:grid().scrollWidth,scrollLeft:grid().scrollLeft,inputWidth:inputBox.width,inputHeight:inputBox.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(layout.pageScroll<=layout.pageClient+1,'custom editor and stored values preserve narrow page containment');
 await setDraft('Inspection team');key(owner(),'Enter');await wait('narrow composed input Enter saves controlled request',()=>!owner()&&!builtin()&&requests()===1&&stored('task-1','owner')==='Inspection team');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,
  checks:['mounted native primary focus, controlled draft and quick assignment Save','Cancel/Escape/input Enter and shared required validation','internal native Tab permission plus boundary cross-kind advance/readonly skip','held requests preserve stored values and conflicts never overwrite external owner','per-row and whole Grid readonly preserve ordinary selection','reversed rows retain stable-ID custom edits','removed pending editor cleanup and usable surviving-row session','Restore assignments clears pending custom error/modes/data/count','340px mounted primary input hit testing and saved composed edit'],
  input:'synthetic click/input/key events and explicit native focus; custom internal Tab and button Enter defaults checked as permission then explicit focus/click',
  trustedInput:'separate Chromium native internal Tab traversal/button activation; stale context token guards covered by Rust tests; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,documentFocusedNow:document.hasFocus()}}
"#;
