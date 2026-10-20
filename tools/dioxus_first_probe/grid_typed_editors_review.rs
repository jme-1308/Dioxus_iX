//! Excluded actual-gallery typed Grid editors and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/grid_typed_editors.rs"]
mod gallery_editors;

#[component]
pub fn GridTypedEditorsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_editors::STYLES} }
        IxTheme { class: "grid-typed-editors-review",
            h1 { "Typed spare-parts editors" }
            Button { id: "grid-editors-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "grid-editors-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_editors::GridTypedEditorsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_GRID_TYPED_EDITORS_CHECK);
                        if let Err(error) = query.send("grideditors") {
                            eprintln!("Typed Grid native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native typed Grid editors: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native typed Grid editors failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Typed Grid native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.grid-typed-editors-review{box-sizing:border-box;min-height:100vh;padding:24px}.grid-typed-editors-review h1{font-size:28px;margin:0 0 24px}.grid-typed-editors-review h2{font-size:22px;margin:24px 0 16px}.grid-typed-editors-review h3{font-size:18px}.grid-typed-editors-review p{max-width:72ch;line-height:1.5}#grid-editors-frame{margin-top:24px}@media(max-width:600px){.grid-typed-editors-review{padding:16px}}";

// Driver-only JS sends synthetic form/click/key events and requests native focus.
// The actual Rust/Dioxus Grid owns editing, validation, traversal and rows. An
// untrusted Space event has no native checkbox default: a separate real .click()
// is used here; Chromium separately tests actual keyboard defaults/activation.
// No production bridge, physical-device claim or event-chronology oracle.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_TYPED_EDITORS_CHECK: &str = r#"
let snapshots={},layout={},inputFocusFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('grid-typed-editors'),result=()=>get('grid-typed-result');
 const grid=()=>host()?.querySelector('[role="grid"][aria-label="Spare-parts editor"]');
 const row=id=>grid()?.querySelector('[data-row-id="'+id+'"]');
 const cell=(id,column)=>row(id)?.querySelector('[role="gridcell"][data-column-id="'+column+'"]');
 const editor=()=>host()?.querySelector('.ix-data-grid-editor[data-retiring="false"]');
 const editorAt=(id,column)=>cell(id,column)?.querySelector('.ix-data-grid-editor[data-retiring="false"]');
 const alert=()=>host()?.querySelector('.ix-data-grid-edit-error[role="alert"]');
 const data=name=>result()?.dataset[name],count=()=>Number(data('count'));
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
 const key=(node,value,shiftKey=false)=>{
  const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;
 };
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const begin=async(id,column)=>{
  await focus(cell(id,column),id+' '+column);key(active(),'F2');
  await wait(id+' '+column+' actual editor focused',()=>editorAt(id,column)&&active()===editorAt(id,column));
  assert(editorAt(id,column).getAttribute('aria-label')?.endsWith(' for '+id),'editor names stable row ID');return editorAt(id,column);
 };
 const setNumber=async value=>{
  const node=editor();assert(node?.type==='number','actual number editor present');
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait('number draft '+value+' applied',()=>editor()?.value===value);
 };
 const setChoice=async value=>{
  const node=editor();assert(node?.tagName==='SELECT','actual native select editor present');
  node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('choice draft '+value+' applied',()=>editor()?.value===value);
 };
 const snapshot=()=>({count:count(),last:data('last'),quantity:data('part1Quantity'),status:data('part1Status'),approved:data('part1Approved'),
  part2Quantity:data('part2Quantity'),held:checked(get('grid-typed-hold')),readonly:checked(get('grid-typed-readonly'))});
 const cancel=async(id,column)=>{key(editor(),'Escape');await wait('Escape cancels '+id+' '+column,()=>!editor()&&active()===cell(id,column))};
 const transfer=async(id,column,keyValue='Tab',shift=false)=>{
  key(editor(),keyValue,shift);await wait('typed transfer to '+id+' '+column,()=>editorAt(id,column)&&active()===editorAt(id,column));return editorAt(id,column);
 };

 await wait('actual three-row typed editor gallery mounted',()=>host()&&grid()&&result()&&row('part-1')&&row('part-2')&&row('part-3')&&get('grid-typed-hold')&&get('grid-typed-readonly')&&get('grid-typed-reset'));
 await wait('WebView document ready for focus input',()=>document.hasFocus());
 assert(count()===0&&data('last')==='none'&&data('part1Quantity')==='2.5'&&data('part1Status')==='ready'&&data('part1Approved')==='false'&&data('part2Quantity')==='10','fresh controlled parts data and counters match source');
 assert(!checked(get('grid-typed-hold'))&&!checked(get('grid-typed-readonly')),'normal save/edit modes start enabled');
 assert(cell('part-3','quantity').getAttribute('aria-readonly')==='true','third-row quantity retains its per-cell readonly flag');
 const number=await begin('part-1','quantity');
 assert(number.tagName==='INPUT'&&number.type==='number'&&Number(number.min)===0&&Number(number.max)===100&&Number(number.step)===0.5&&number.value==='2.5','number editor exposes actual min/max/step and controlled value');
 assert(!number.hasAttribute('data-select-all'),'number editor does not request text selection adapter');
 inputFocusFacts={documentFocused:document.hasFocus(),activeLabel:active()?.getAttribute('aria-label'),numberType:number.type,min:number.min,max:number.max,step:number.step};
 await setNumber('3.5');key(editor(),'Enter');
 await wait('Enter emits stable number request and renders applied3.5',()=>!editor()&&count()===1&&data('last')==='part-1:quantity:2.5->3.5'&&data('part1Quantity')==='3.5'&&cell('part-1','quantity').textContent.includes('3.5')&&active()===cell('part-1','quantity'));
 snapshots.numberSaved=snapshot();
 await begin('part-1','quantity');await setNumber('0.25');
 assert(editor().validity.stepMismatch,'native number enforces half-step precision');key(editor(),'Enter');
 await wait('quarter-step rejection retains editor and accessible error',()=>editor()?.getAttribute('aria-invalid')==='true'&&alert()?.textContent.trim().length>0);
 assert(count()===1&&data('part1Quantity')==='3.5','step rejection leaves controlled rows and request count unchanged');
 await cancel('part-1','quantity');assert(!alert(),'Escape removes number validation message');
 await begin('part-1','quantity');await setNumber('100.5');
 assert(editor().validity.rangeOverflow,'native numeric maximum is active');key(editor(),'Enter');
 await wait('over-maximum rejection retains editor and error',()=>editor()?.getAttribute('aria-invalid')==='true'&&alert()?.textContent.trim().length>0);
 assert(count()===1&&data('part1Quantity')==='3.5','bounds rejection remains atomic');await cancel('part-1','quantity');

 await begin('part-1','quantity');await setNumber('4.5');
 const select=await transfer('part-1','status');
 assert(count()===2&&data('part1Quantity')==='4.5'&&data('last')==='part-1:quantity:3.5->4.5','Tab commits number request before opening choice editor');
 assert(select.tagName==='SELECT'&&select.value==='ready'&&select.querySelector('option[value="retired"]')?.disabled,'actual native choice preserves current value and disabled retired option');
 assert(!select.hasAttribute('data-select-all'),'select editor does not request text selection');
 const arrow=key(select,'ArrowDown');assert(!arrow.defaultPrevented,'native choice arrows are not hijacked by Grid navigation');
 await setChoice('service');key(editor(),'Enter');
 await wait('Enter saves configured native choice',()=>!editor()&&count()===3&&data('part1Status')==='service'&&data('last')==='part-1:status:ready->service'&&cell('part-1','status').textContent.toLowerCase().includes('service')&&active()===cell('part-1','status'));
 const checkbox=await begin('part-1','approved');
 assert(checkbox.tagName==='INPUT'&&checkbox.type==='checkbox'&&!checkbox.checked&&!checkbox.hasAttribute('data-select-all'),'actual unchecked boolean editor uses native checkbox without text selection');
 const space=key(checkbox,' ');assert(!space.defaultPrevented,'checkbox Space retains native default permission');
 // Untrusted Space does not activate the native default. One explicit click
 // invokes the real checkbox input/change path without pretending it did.
 checkbox.click();await wait('native checkbox click updates draft',()=>editor()?.checked===true);
 assert(data('part1Approved')==='false'&&count()===3&&row('part-1').getAttribute('aria-selected')==='false','checkbox draft/Space do not mutate consumer data or select Grid row');
 key(editor(),'Enter');
 await wait('Enter saves boolean true request',()=>!editor()&&count()===4&&data('part1Approved')==='true'&&data('last')==='part-1:approved:false->true'&&active()===cell('part-1','approved'));
 await begin('part-1','approved');editor().click();await wait('unchecked draft available to cancel',()=>editor()?.checked===false);
 await cancel('part-1','approved');assert(count()===4&&data('part1Approved')==='true','Escape discards boolean draft without callback');
 snapshots.typedSaved=snapshot();

 await begin('part-1','approved');const text=await transfer('part-2','asset');
 assert(text.tagName==='INPUT'&&text.type==='text'&&text.value==='Valve kit'&&text.hasAttribute('data-select-all'),'Tab crosses row boundary into established text editor');
 await wait('text edit selects its current value after mounted focus',()=>active()===text&&text.selectionStart===0&&text.selectionEnd===text.value.length);
 const secondNumber=await transfer('part-2','quantity');assert(secondNumber.type==='number'&&secondNumber.value==='10','Tab moves from text to native number editor');
 await cancel('part-2','quantity');await begin('part-2','approved');
 await transfer('part-3','asset');const skipped=await transfer('part-3','status');
 assert(skipped.tagName==='SELECT'&&!editorAt('part-3','quantity')&&cell('part-3','quantity').getAttribute('aria-readonly')==='true','Tab skips readonly number cell across row boundary');
 const backwards=await transfer('part-3','asset','Tab',true);
 assert(backwards.type==='text'&&backwards.value==='Sensor kit','Shift+Tab skips readonly number cell in reverse');
 await cancel('part-3','asset');assert(count()===4,'unchanged cross-kind traversal does not emit edit requests');
 await focus(cell('part-3','asset'),'type-to-edit text cell');key(active(),'P');
 await wait('type-to-edit starts focused text with its caret at the end',()=>editorAt('part-3','asset')&&active()===editorAt('part-3','asset')&&editor().value==='P'&&editor().selectionStart===1&&editor().selectionEnd===1);
 const typedText=editor();
 // An untrusted key cannot type through the native default. Explicitly apply
 // its follow-up draft through the real input path; trusted Chromium appends.
 Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(typedText,'PQ');typedText.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
 await wait('follow-up text draft stays with the current editor',()=>editor()===typedText&&active()===typedText&&typedText.value==='PQ');
 await cancel('part-3','asset');assert(count()===4,'type-to-edit cancellation leaves stored data and request count unchanged');
 snapshots.traversed=snapshot();

 get('grid-typed-hold').click();await wait('host hold mode enabled',()=>checked(get('grid-typed-hold')));
 await begin('part-2','quantity');await setNumber('11');key(editor(),'Enter');
 await wait('valid held request is recorded while source rows remain unchanged',()=>!editor()&&count()===5&&data('last')==='part-2:quantity:10->11'&&data('part2Quantity')==='10'&&cell('part-2','quantity').textContent.includes('10'));
 snapshots.held=snapshot();get('grid-typed-readonly').click();
 await wait('whole Grid readonly state reaches typed cells',()=>checked(get('grid-typed-readonly'))&&cell('part-1','quantity')?.getAttribute('aria-readonly')==='true'&&cell('part-1','status')?.getAttribute('aria-readonly')==='true');
 await focus(cell('part-1','status'),'readonly choice cell');key(active(),'F2');
 await wait('readonly state blocks typed editing',()=>!editor()&&active()===cell('part-1','status'));
 key(active(),' ');
 await wait('readonly Grid preserves ordinary row selection',()=>row('part-1')?.getAttribute('aria-selected')==='true');
 assert(count()===5,'readonly attempts do not emit edit requests');
 get('grid-typed-readonly').click();await wait('Grid editable mode restored',()=>!checked(get('grid-typed-readonly'))&&cell('part-1','status')?.getAttribute('aria-readonly')==='false');
 await begin('part-1','status');await setChoice('ready');
 get('grid-typed-reset').focus();get('grid-typed-reset').click();
 await wait('external Restore parts clears pending typed editor/data/modes/counters',()=>count()===0&&data('last')==='none'&&data('part1Quantity')==='2.5'&&data('part1Status')==='ready'&&data('part1Approved')==='false'&&data('part2Quantity')==='10'&&!checked(get('grid-typed-hold'))&&!checked(get('grid-typed-readonly'))&&!host().querySelector('.ix-data-grid-editor')&&!alert()&&active()===get('grid-typed-reset'));
 assert(row('part-1').getAttribute('aria-selected')==='false','Restore parts also resets Grid selection state');snapshots.reset=snapshot();

 get('grid-editors-narrow').click();
 await wait('actual narrow Grid has internal horizontal scrolling',()=>get('grid-editors-frame').getBoundingClientRect().width<=341&&grid().scrollWidth>grid().clientWidth);
 await begin('part-1','approved');
 await wait('mounted typed editor focus reveals rightmost checkbox inside Grid viewport',()=>{
  const box=editor()?.getBoundingClientRect(),viewport=grid().getBoundingClientRect();
  return box&&grid().scrollLeft>0&&box.left>=viewport.left-1&&box.right<=viewport.left+grid().clientWidth+1;
 });
 const box=grid().closest('.ix-data-grid').getBoundingClientRect(),editBox=editor().getBoundingClientRect();
 layout={height:box.height,width:box.width,clientWidth:grid().clientWidth,scrollWidth:grid().scrollWidth,scrollLeft:grid().scrollLeft,
  editorWidth:editBox.width,editorHeight:editBox.height,rowHeight:row('part-1').getBoundingClientRect().height,
  pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(Math.abs(layout.height-240)<=1&&Math.abs(layout.rowHeight-42)<=1&&layout.pageScroll<=layout.pageClient+1,'typed editors preserve Grid height/row geometry and narrow page containment');
 await cancel('part-1','approved');assert(count()===0,'narrow boolean cancellation leaves restored source unchanged');
 return {ok:true,snapshots,layout,inputFocusFacts,
  checks:['actual native number/select/checkbox editors','number min/max/half-step accepted and rejected edits','Escape preserves controlled data on invalid/canceled drafts','Tab number-to-select and cross-kind/cross-row transfer with actual text selection','disabled native choice and saved string payload','checkbox native Space permission/click/save/cancel without row selection','Tab/ShiftTab readonly number skip plus type-to-edit end caret and canceled draft','valid held request recorded without applying data','whole Grid readonly blocks editing and preserves selection','Restore parts clears pending choice/modes/counters/data','mounted focus reveals rightmost typed editor in340px scrolling Grid'],
  input:'synthetic input/change/key events and explicit native focus; checkbox toggled by one real DOM click per draft',
  trustedInput:'separate Chromium actual native-select keyboard navigation and checkbox Space defaults; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
