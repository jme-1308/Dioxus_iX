//! Excluded actual-gallery Grid cell composition and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/grid_cell_renderers.rs"]
mod gallery_cells;

#[component]
pub fn GridCellRenderersPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_cells::STYLES} }
        IxTheme { class: "grid-cell-renderers-review",
            h1 { "Dioxus components inside Grid cells" }
            Button { id: "grid-renderers-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "grid-renderers-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_cells::GridCellRenderersExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_GRID_CELL_RENDERERS_CHECK);
                        if let Err(error) = query.send("gridrenderers") {
                            eprintln!("Grid cell composition native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native Grid cell composition: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native Grid cell composition failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Grid cell composition native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.grid-cell-renderers-review{box-sizing:border-box;min-height:100vh;padding:24px}.grid-cell-renderers-review h1{font-size:28px;margin:0 0 24px}.grid-cell-renderers-review h2{font-size:22px;margin:24px 0 16px}.grid-cell-renderers-review h3{font-size:18px}.grid-cell-renderers-review p{max-width:72ch;line-height:1.5}#grid-renderers-frame{margin-top:24px}@media(max-width:600px){.grid-cell-renderers-review{padding:16px}}";

// Driver-only JS synthesizes input/click/keyboard events and requests native
// focus; actual Dioxus components own state, menus, editing and focus return.
// Untrusted Tab/Home/Arrow/Space events do not execute native keyboard defaults:
// this checks their permission, then explicitly focuses/updates the native input.
// Chromium separately verifies actual browser Tab/caret/typing/default behavior.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_GRID_CELL_RENDERERS_CHECK: &str = r#"
let snapshots={},layout={},inputFocusFacts={},keyPermissions={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('grid-cell-renderers'),result=()=>get('composed-result');
 const grid=()=>host()?.querySelector('[role="grid"][aria-label="Composed maintenance cells"]');
 const rows=()=>[...grid()?.querySelectorAll('.ix-data-grid-body [data-row-id]')??[]];
 const row=id=>rows().find(node=>node.dataset.rowId===id);
 const cell=(id,column)=>row(id)?.querySelector('[role="gridcell"][data-column-id="'+column+'"]');
 const note=id=>get('composed-note-'+id),pill=id=>cell(id,'status')?.querySelector('.ix-pill');
 const trigger=id=>cell(id,'actions')?.querySelector('[role="button"][aria-haspopup="menu"][aria-label="Actions for '+id+'"]');
 const menu=id=>get(trigger(id)?.getAttribute('aria-controls'));
 const opened=id=>trigger(id)?.getAttribute('aria-expanded')==='true'&&menu(id)&&!menu(id).hidden;
 const item=(id,label)=>menu(id)?.querySelector('[role="menuitem"][data-ix-dropdown-label="'+label+'"]');
 const disabled=node=>node?.getAttribute('aria-disabled')==='true';
 const editor=()=>host()?.querySelector('.ix-data-grid-editor[data-retiring="false"]');
 const data=name=>result()?.dataset[name],number=name=>Number(data(name));
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
 const key=(node,value)=>{
  const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;
 };
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setInput=async(node,value,label)=>{
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait(label+' actual input applied',()=>node.isConnected&&node.value===value);
 };
 const openMenu=async id=>{
  await focus(trigger(id),id+' action trigger');key(trigger(id),'ArrowDown');
  await wait(id+' actual top-layer Dropdown opened',()=>opened(id)&&menu(id).matches(':popover-open')&&item(id,'Inspect'));
  assert(menu(id).hasAttribute('popover')&&menu(id).getAttribute('role')==='menu','actual Dioxus menu uses native top layer above Grid clipping');
 };
 const choose=async(id,label)=>{
  const node=item(id,label);assert(node&&!disabled(node),label+' is available');await focus(node,label+' action');key(node,'Enter');
  await wait(label+' closes menu and restores its own trigger focus',()=>!opened(id)&&active()===trigger(id));
 };
 const escapeMenu=async id=>{
  await focus(item(id,'Inspect'),id+' menu Escape target');key(active(),'Escape');
  await wait(id+' Dropdown Escape returns its own trigger',()=>!opened(id)&&active()===trigger(id));
 };
 const begin=async(id,column)=>{
  await focus(cell(id,column),id+' '+column);key(active(),'F2');
  await wait(id+' '+column+' built-in editor focused',()=>editor()&&cell(id,column).contains(editor())&&active()===editor());
 };
 const snapshot=()=>({inspections:number('inspections'),last:data('last'),note1:data('note1'),status1:data('status1'),selected:number('selected'),visible:number('visible'),edits:number('edits'),
  readonly:checked(get('composed-readonly')),filtered:checked(get('composed-service-filter')),reversed:checked(get('composed-sort')),order:rows().map(node=>node.dataset.rowId)});

 await wait('actual composed-cell gallery mounted',()=>host()&&grid()&&result()&&rows().length===3&&note('asset-1')&&note('asset-3')&&trigger('asset-1')&&pill('asset-1'));
 await wait('WebView document ready for focus input',()=>document.hasFocus());
 assert(number('inspections')===0&&data('last')==='none'&&number('selected')===0&&number('visible')===3&&number('edits')===0&&data('note1')==='Inspect seal'&&data('status1')==='ready','initial composed cells reflect controlled host data');
 assert(cell('asset-1','asset').querySelector('.ix-data-grid-cell-text')?.textContent==='Motor 001'&&pill('asset-1').textContent.includes('Ready'),'None renderer preserves default asset text while display renderer shows status Pill');
 assert(note('asset-3').readOnly&&!note('asset-1').readOnly,'renderer context preserves per-row note editability');
 await focus(cell('asset-1','note'),'containing note cell');const tab=key(active(),'Tab');
 assert(!tab.defaultPrevented&&!editor(),'interactive cell permits native Tab instead of starting next Grid editor');
 await focus(note('asset-1'),'actual note input');
 for(const value of ['Home','End','ArrowLeft','ArrowDown',' ','F2']){
  const event=key(note('asset-1'),value);keyPermissions[value]=!event.defaultPrevented;
  assert(!event.defaultPrevented,'native input '+value+' is not hijacked by Grid');
 }
 note('asset-1').dispatchEvent(new MouseEvent('dblclick',{bubbles:true,cancelable:true}));
 await setInput(note('asset-1'),'Inspect seal soon','host-owned note');
 await wait('input typing stays native without Grid selection or built-in edit request',()=>data('note1')==='Inspect seal soon'&&active()===note('asset-1')&&number('selected')===0&&number('edits')===0&&!editor());
 inputFocusFacts={documentFocused:document.hasFocus(),active:active()?.id};
 key(note('asset-1'),'Escape');await wait('unhandled note Escape returns to containing Grid cell',()=>active()===cell('asset-1','note')&&!editor());
 snapshots.nativeInput=snapshot();

 await openMenu('asset-1');assert(!disabled(item('asset-1','Inspect'))&&disabled(item('asset-1','Archive')),'normal Inspect and disabled Archive semantics preserved');
 await choose('asset-1','Inspect');
 await wait('Inspect callback records stable row identity',()=>number('inspections')===1&&data('last')==='asset-1');
 assert(number('selected')===0&&number('edits')===0,'action activation does not select row or emit Grid edit');
 await openMenu('asset-1');await escapeMenu('asset-1');
 assert(active()===trigger('asset-1')&&active()!==cell('asset-1','actions'),'child-handled Dropdown Escape wins over containing-cell Escape return');
 await openMenu('asset-1');await choose('asset-1','Mark for service');
 await wait('host action updates actual status Pill through fresh renderer context',()=>data('status1')==='service'&&pill('asset-1')?.textContent.includes('Needs service'));
 assert(number('inspections')===1&&data('note1')==='Inspect seal soon','service action preserves independent notes and inspection count');
 row('asset-1').querySelector('.ix-data-grid-selection [role="checkbox"]').click();
 await wait('selected renderer context displays ordinary selection indicator',()=>number('selected')===1&&cell('asset-1','status').querySelector('.composed-status-selected')?.textContent==='Selected');
 assert(cell('asset-1','status').querySelector('.composed-status-cell').dataset.rowIndex==='0'&&pill('asset-1').textContent.includes('Needs service'),'selected display receives current stable row and zero-based position');
 snapshots.selectedContext=snapshot();
 row('asset-1').querySelector('.ix-data-grid-selection [role="checkbox"]').click();
 await wait('deselection updates renderer context',()=>number('selected')===0&&!cell('asset-1','status').querySelector('.composed-status-selected'));
 snapshots.actions=snapshot();

 await begin('asset-1','asset');assert(editor().value==='Motor 001','default text cell still uses established editor');
 await setInput(editor(),'Motor 001 serviced','default asset edit');key(editor(),'Enter');
 await wait('default asset edit is applied as ordinary controlled request',()=>!editor()&&number('edits')===1&&cell('asset-1','asset').textContent.includes('Motor 001 serviced')&&active()===cell('asset-1','asset'));
 await begin('asset-1','note');
 assert(!note('asset-1')&&editor().value==='Inspect seal soon','built-in active editor replaces interactive renderer content');
 await setInput(editor(),'Discard built-in note draft','built-in note draft');key(editor(),'Escape');
 await wait('Escape restores custom input from host-owned data',()=>!editor()&&note('asset-1')?.value==='Inspect seal soon'&&number('edits')===1&&active()===cell('asset-1','note'));
 get('composed-sort').click();
 await wait('host reverse sort updates displayed identity and preserves notes',()=>checked(get('composed-sort'))&&rows()[0]?.dataset.rowId==='asset-3'&&note('asset-1')?.value==='Inspect seal soon'&&cell('asset-1','asset').textContent.includes('Motor 001 serviced')&&cell('asset-3','status').querySelector('.composed-status-cell').dataset.rowIndex==='0'&&cell('asset-1','status').querySelector('.composed-status-cell').dataset.rowIndex==='2');
 get('composed-service-filter').click();
 await wait('host service filter retains both matching stable rows',()=>checked(get('composed-service-filter'))&&number('visible')===2&&!row('asset-3')&&row('asset-1')&&row('asset-2')&&note('asset-1')?.value==='Inspect seal soon'&&cell('asset-1','status').querySelector('.composed-status-cell').dataset.rowIndex==='1');
 get('composed-service-filter').click();
 await wait('cleared filter restores readonly note row without losing state',()=>number('visible')===3&&note('asset-3')?.readOnly&&data('note1')==='Inspect seal soon');
 snapshots.sortedFiltered=snapshot();

 get('composed-readonly').click();
 await wait('Grid readonly context reaches custom native input and default cell',()=>checked(get('composed-readonly'))&&note('asset-1')?.readOnly&&cell('asset-1','asset').getAttribute('aria-readonly')==='true');
 await focus(cell('asset-1','asset'),'readonly default cell');key(active(),'F2');await wait('readonly blocks built-in edit',()=>!editor()&&active()===cell('asset-1','asset'));
 await openMenu('asset-1');
 assert(!disabled(item('asset-1','Inspect'))&&disabled(item('asset-1','Mark for service'))&&disabled(item('asset-1','Clear note')),'readonly preserves Inspect and disables mutating composed actions');
 item('asset-1','Clear note').click();
 await wait('disabled mutation leaves nonempty host note intact',()=>data('note1')==='Inspect seal soon'&&number('edits')===1);
 if(!opened('asset-1'))await openMenu('asset-1');await choose('asset-1','Inspect');
 await wait('readonly Inspect remains usable',()=>number('inspections')===2&&data('last')==='asset-1'&&data('note1')==='Inspect seal soon');
 get('composed-readonly').click();await wait('readonly mode releases custom input',()=>!checked(get('composed-readonly'))&&!note('asset-1').readOnly);
 await openMenu('asset-1');await choose('asset-1','Clear note');
 await wait('enabled Clear note updates host data without inventing Grid edit callback',()=>data('note1')===''&&note('asset-1').value===''&&number('edits')===1);
 snapshots.readonly=snapshot();
 await openMenu('asset-1');
 get('composed-reset').focus();get('composed-reset').click();
 await wait('Restore assets resets host state and owned custom components',()=>number('inspections')===0&&data('last')==='none'&&number('selected')===0&&number('visible')===3&&number('edits')===0&&data('note1')==='Inspect seal'&&data('status1')==='ready'&&!checked(get('composed-readonly'))&&!checked(get('composed-service-filter'))&&!checked(get('composed-sort'))&&note('asset-1')?.value==='Inspect seal'&&cell('asset-1','asset').textContent.includes('Motor 001')&&!editor()&&active()===get('composed-reset'));
 assert(rows()[0].dataset.rowId==='asset-1'&&pill('asset-1').textContent.includes('Ready')&&!host().querySelector('[role="menu"]:not([hidden])'),'reset restores original displayed order/Pill and clears open child menus');snapshots.reset=snapshot();

 get('grid-renderers-narrow').click();
 await wait('narrow actual Grid keeps internal horizontal overflow',()=>get('grid-renderers-frame').getBoundingClientRect().width<=341&&grid().scrollWidth>grid().clientWidth);
 await openMenu('asset-1');
 await wait('narrow top-layer action is visible and receives hit testing above Grid clip',()=>{
  const node=item('asset-1','Inspect'),box=node?.getBoundingClientRect();
  if(!box||box.width<=0||box.height<=0||box.left<0||box.top<0||box.right>innerWidth+1||box.bottom>innerHeight+1)return false;
  const hit=document.elementFromPoint(box.left+box.width/2,box.top+box.height/2);return hit===node||node.contains(hit);
 });
 const menuBox=menu('asset-1').getBoundingClientRect(),gridBox=grid().closest('.ix-data-grid').getBoundingClientRect();
 layout={width:gridBox.width,height:gridBox.height,clientWidth:grid().clientWidth,scrollWidth:grid().scrollWidth,scrollLeft:grid().scrollLeft,
  rowHeight:row('asset-1').getBoundingClientRect().height,menuWidth:menuBox.width,menuHeight:menuBox.height,
  pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(Math.abs(layout.height-240)<=1&&Math.abs(layout.rowHeight-48)<=1&&layout.pageScroll<=layout.pageClient+1,'composed Grid preserves geometry and narrow page containment');
 await choose('asset-1','Inspect');await wait('narrow action callback usable after reset',()=>number('inspections')===1&&data('last')==='asset-1');
 return {ok:true,snapshots,layout,inputFocusFacts,keyPermissions,
  checks:['display Pill and default text fallback','native Tab permission and input key/default containment','host-owned note editing without Grid selection/edit','unhandled Escape returns containing cell','actual top-layer action Dropdown Inspect/dismiss/focus return','child-owned Dropdown Escape respected','host action and selected/index changes refresh renderer context','built-in default edit and custom-input replacement/cancel','stable host notes across sort/filter','readonly native notes/default edit/mutating action guards and usable Inspect','Restore assets resets host/component state','usable top-layer actions above narrow horizontal Grid clip'],
  input:'synthetic input/click/key events and explicit native focus; untrusted Tab/caret defaults are not simulated',
  trustedInput:'separate Chromium actual Tab traversal, input typing/caret/Space and keyboard defaults; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,inputFocusFacts,keyPermissions,documentFocusedNow:document.hasFocus()}}
"#;
