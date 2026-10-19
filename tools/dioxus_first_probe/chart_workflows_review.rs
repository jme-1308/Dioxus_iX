//! Excluded actual-gallery energy chart preview and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/chart_workflows.rs"]
mod gallery_workflows;

#[component]
pub fn ChartWorkflowsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_workflows::STYLES} }
        IxTheme { class: "chart-workflows-review",
            h1 { "Energy charts" }
            Button { id: "chart-workflows-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "chart-workflows-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_workflows::ChartWorkflows {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_CHART_WORKFLOWS_CHECK);
                        if let Err(error) = query.send("chartworkflows") {
                            eprintln!("Chart workflows native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native energy charts: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native energy charts failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Chart workflows native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.chart-workflows-review{box-sizing:border-box;min-height:100vh;padding:24px}.chart-workflows-review h1{font-size:28px;margin:0 0 24px}.chart-workflows-review h2{font-size:22px;margin:24px 0 16px}.chart-workflows-review h3{font-size:18px}.chart-workflows-review p{max-width:72ch;line-height:1.5}.chart-workflows-review .demo-section{margin-bottom:32px;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}#chart-workflows-frame{margin-top:24px}@media(max-width:600px){.chart-workflows-review{padding:16px}}";

// Driver-only JS observes real SVG/table/layout and decodes the returned PNG.
// Click/input/keyboard events are synthetic and native focus is explicit. Rust
// gallery/component code owns dataset changes, editor state, zoom and export.
// Trusted typing, keyboard activation and physical pointer use are separate
// Chromium evidence. A ready PNG/download request does not prove host file saving.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CHART_WORKFLOWS_CHECK: &str = r#"
let layout={},snapshots={},png={},inputFocusFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('energy-charts'),toolbox=()=>host()?.querySelector('.ix-chart-toolbox');
 const status=()=>get('energy-state'),dataset=()=>get('energy-dataset'),exportStatus=()=>get('energy-export');
 const data=name=>status()?.dataset[name],changes=()=>Number(data('changes'));
 const control=label=>toolbox()?.querySelector('.ix-chart-toolbox-controls button[aria-label="'+label+'"]');
 const figure=()=>toolbox()?.querySelector('figure[aria-label="Energy history"]');
 const comparison=()=>host()?.querySelector('figure[aria-label="Energy comparison"]');
 const canvas=()=>figure()?.querySelector('[role="listbox"]'),svg=()=>figure()?.querySelector('svg[role="group"]');
 const points=()=>[...figure()?.querySelectorAll('.ix-chart-mark[role="option"]')??[]];
 const tableRows=node=>[...node?.querySelectorAll('.ix-chart-data tbody tr')??[]];
 const values=node=>tableRows(node).map(row=>[...row.querySelectorAll('th,td')].map(cell=>cell.textContent.trim()));
 const dialog=()=>toolbox()?.querySelector('.ix-chart-data-editor[role="dialog"]');
 const textarea=()=>dialog()?.querySelector('textarea[aria-label="Chart data"]');
 const editorButton=label=>[...dialog()?.querySelectorAll('button')??[]].find(button=>button.textContent.trim()===label);
 const selectedPoint=()=>get(canvas()?.getAttribute('aria-activedescendant'));
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; state='+status()?.textContent+'; dataset='+dataset()?.textContent+'; export='+exportStatus()?.textContent);
 };
 const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true}));
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setText=async value=>{
  const node=textarea();Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(node,value);
  node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait('data-view draft applied',()=>textarea()?.value===value);
 };
 const openEditor=async()=>{
  await focus(control('Data view'),'data-view trigger');control('Data view').click();
  await wait('data-view dialog and Dioxus textarea focus',()=>dialog()&&textarea()&&active()===textarea());
  assert(toolbox().querySelector('.ix-chart-toolbox-content').hasAttribute('inert'),'data-view editor makes underlying chart controls inert');
 };
 const snapshot=()=>({windowStart:Number(data('windowStart')),windowEnd:Number(data('windowEnd')),
  categories:data('categories'),firstValue:data('firstValue'),kind:data('kind'),changes:changes(),rows:values(figure())});
 const shapeFacts=node=>{
  const frame=node.querySelector('.ix-chart-canvas'),plot=node.querySelector('svg[role="group"]');
  const box=plot.getBoundingClientRect(),frameBox=frame.getBoundingClientRect();
  return {width:box.width,height:box.height,frameWidth:frameBox.width,viewBoxWidth:plot.viewBox.baseVal.width,
   contained:box.width<=frameBox.width+1,tableRows:tableRows(node).length,points:node.querySelectorAll('.ix-chart-mark').length};
 };

 await wait('actual two-series energy charts mounted',()=>status()&&dataset()&&exportStatus()&&figure()&&comparison()&&points().length===14&&tableRows(figure()).length===7);
 await wait('WebView document ready for keyboard focus',()=>document.hasFocus());
 assert(dataset().dataset.week==='1'&&values(figure())[0].join('|')==='Mon|42|50','initial energy history has declared first category and series values');
 assert(values(comparison())[0].join('|')==='Mon|42|50'&&comparison().querySelectorAll('.ix-chart-mark').length===14,'companion chart uses the same original two-series dataset');
 assert(data('kind')==='line'&&Number(data('firstValue'))===42&&Number(data('categories'))===7&&Number(data('windowStart'))===0&&Number(data('windowEnd'))===7,'initial callback summary describes full line chart');
 const headers=[...figure().querySelectorAll('.ix-chart-data thead th')].map(node=>node.textContent.trim());
 assert(headers.join('|')==='Category|Output|Target','actual accessible table exposes two named series');
 const targetLegend=()=>[...comparison().querySelectorAll('.ix-chart-legend button')].find(button=>button.textContent.trim()==='Target');
 const companionChanges=changes();assert(targetLegend()?.getAttribute('aria-pressed')==='true','companion legend starts with Target visible');
 targetLegend().click();
 await wait('companion legend hides only its own Target marks',()=>targetLegend()?.getAttribute('aria-pressed')==='false'&&comparison().querySelectorAll('.ix-chart-mark').length===7);
 assert(points().length===14&&changes()===companionChanges,'companion legend does not mutate toolbox data or callbacks');
 const tableDetails=()=>comparison().querySelector('details.ix-chart-data');
 assert(!tableDetails().open,'companion data table starts collapsed');tableDetails().querySelector('summary').click();
 await wait('native summary opens companion accessible data table',()=>tableDetails().open&&comparison().querySelector('.ix-chart-data table').getBoundingClientRect().height>0);
 assert(values(comparison()).length===7&&values(comparison())[0].join('|')==='Mon|42|50','expanded companion table retains both series including hidden Target');
 tableDetails().querySelector('summary').click();await wait('native summary closes companion table',()=>!tableDetails().open);
 targetLegend().click();await wait('companion legend restores both series',()=>targetLegend()?.getAttribute('aria-pressed')==='true'&&comparison().querySelectorAll('.ix-chart-mark').length===14);
 const initialChanges=changes();control('Switch to bar chart').click();
 await wait('bar switch updates callback and actual marks',()=>data('kind')==='bar'&&changes()>initialChanges&&control('Switch to bar chart').getAttribute('aria-pressed')==='true'&&figure().querySelectorAll('.ix-chart-mark rect').length===14);
 const barChanges=changes();control('Switch to line chart').click();
 await wait('line switch updates callback and actual geometry',()=>data('kind')==='line'&&changes()>barChanges&&control('Switch to line chart').getAttribute('aria-pressed')==='true'&&figure().querySelectorAll('path.ix-chart-line').length===2&&figure().querySelectorAll('.ix-chart-mark rect').length===0);

 await focus(canvas(),'history chart');key(canvas(),'Home');
 await wait('keyboard inspection selects Monday output',()=>selectedPoint()?.getAttribute('aria-label')==='Mon, Output: 42');
 key(canvas(),'ArrowRight');await wait('keyboard next category',()=>selectedPoint()?.getAttribute('aria-label')==='Tue, Output: 48');
 key(canvas(),'ArrowDown');await wait('keyboard next series',()=>selectedPoint()?.getAttribute('aria-label')==='Tue, Target: 50');
 key(canvas(),'ArrowUp');await wait('keyboard returns to Output series',()=>selectedPoint()?.getAttribute('aria-label')==='Tue, Output: 48');
 inputFocusFacts={documentFocused:document.hasFocus(),activeLabel:active()?.getAttribute('aria-label'),selected:selectedPoint()?.getAttribute('aria-label')};
 control('Zoom').click();await wait('zoom brush enabled through actual toolbox',()=>control('Zoom').getAttribute('aria-pressed')==='true'&&svg()?.querySelector('.ix-chart-brush-hit'));
 await focus(canvas(),'keyboard brush chart');key(canvas(),'Home');
 await wait('brush starts from first output category',()=>selectedPoint()?.getAttribute('aria-label')==='Mon, Output: 42');
 key(canvas(),'ArrowRight');await wait('brush first category Tuesday',()=>selectedPoint()?.getAttribute('aria-label')==='Tue, Output: 48');
 key(canvas(),'Enter');await wait('keyboard brush anchor visible',()=>svg()?.querySelector('.ix-chart-brush-selection'));
 key(canvas(),'ArrowRight');await wait('brush extends through Wednesday',()=>selectedPoint()?.getAttribute('aria-label')==='Wed, Output: 55');
 key(canvas(),'ArrowRight');await wait('brush extends through Thursday',()=>selectedPoint()?.getAttribute('aria-label')==='Thu, Output: 51');
 const beforeBrush=changes();key(canvas(),'Enter');
 await wait('keyboard brush applies exclusive1..4 window and callback',()=>Number(data('windowStart'))===1&&Number(data('windowEnd'))===4&&Number(data('categories'))===3&&Number(data('firstValue'))===48&&changes()>beforeBrush&&tableRows(figure()).length===3&&points().length===6&&!control('Zoom back').disabled);
 assert(values(figure())[0].join('|')==='Tue|48|50'&&values(figure())[2][0]==='Thu','visible chart/table reflect brushed category range');
 snapshots.brushed=snapshot();control('Zoom back').click();
 await wait('zoom back restores full original range',()=>Number(data('windowStart'))===0&&Number(data('windowEnd'))===7&&tableRows(figure()).length===7&&control('Zoom back').disabled);
 control('Switch to bar chart').click();await wait('bar before Restore',()=>data('kind')==='bar');
 control('Restore').click();
 await wait('Restore resets kind/range/brush to original input',()=>data('kind')==='line'&&Number(data('windowStart'))===0&&Number(data('windowEnd'))===7&&control('Zoom').getAttribute('aria-pressed')==='false'&&values(figure())[0][1]==='42');

 await openEditor();
 const initialDraft=textarea().value,validLines=initialDraft.split('\n');
 assert(validLines[1]?.split('\t').join('|')==='Mon|42|50','data-view draft reflects original actual series');
 const firstCells=validLines[1].split('\t');firstCells[1]='45';validLines[1]=firstCells.join('\t');
 await setText(validLines.join('\n'));const beforeValid=changes();editorButton('Refresh').click();
 await wait('valid data edit applies atomically and restores trigger focus',()=>!dialog()&&active()===control('Data view')&&changes()>beforeValid&&Number(data('firstValue'))===45&&values(figure())[0][1]==='45');
 assert(points().some(point=>point.getAttribute('aria-label')==='Mon, Output: 45'),'edited value updates actual plot marks');
 assert(values(comparison())[0][1]==='42','toolbox-owned edits do not mutate independent companion source props');
 snapshots.validEdit=snapshot();
 await openEditor();const invalidLines=textarea().value.split('\n');
 const invalidFirst=invalidLines[1].split('\t');invalidFirst[1]='66';invalidLines[1]=invalidFirst.join('\t');
 const invalidSecond=invalidLines[2].split('\t');invalidSecond[1]='not-a-number';invalidLines[2]=invalidSecond.join('\t');
 const invalidDraft=invalidLines.join('\n'),beforeInvalid=changes();await setText(invalidDraft);editorButton('Refresh').click();
 await wait('invalid data edit retains dialog/draft with visible validation',()=>dialog()&&textarea()?.value===invalidDraft&&dialog().querySelector('.ix-chart-data-error')?.hidden===false&&dialog().querySelector('.ix-chart-data-error').textContent.trim().length>0);
 assert(changes()===beforeInvalid&&Number(data('firstValue'))===45&&values(figure())[0][1]==='45','invalid later row does not partially apply earlier valid66');
 await focus(textarea(),'invalid data draft');key(textarea(),'Escape');
 await wait('Escape discards draft and returns focus to Data view',()=>!dialog()&&active()===control('Data view'));
 assert(changes()===beforeInvalid&&values(figure())[0][1]==='45','Escape preserves last accepted data');

 await openEditor();const staleLines=textarea().value.split('\n');
 const staleFirst=staleLines[1].split('\t');staleFirst[1]='777';staleLines[1]=staleFirst.join('\t');await setText(staleLines.join('\n'));
 get('energy-next').click();
 await wait('external dataset replacement closes stale editor and renders week2',()=>dataset().dataset.week==='2'&&!dialog()&&values(figure())[0]?.join('|')==='Mon|38|50'&&values(comparison())[0]?.join('|')==='Mon|38|50'&&points().length===14);
 assert(!toolbox().querySelector('.ix-chart-toolbox-content').hasAttribute('inert'),'replacement releases editor inert state');
 await openEditor();
 assert(textarea().value.split('\n')[1]?.split('\t')[1]==='38'&&!textarea().value.includes('777'),'reopened data view is based on replacement props rather than discarded stale draft');
 key(textarea(),'Escape');await wait('replacement editor closes normally',()=>!dialog()&&active()===control('Data view'));
 snapshots.replaced=snapshot();get('energy-empty').click();
 await wait('empty dataset removes actual plot points and categories',()=>dataset().dataset.week==='empty'&&points().length===0&&tableRows(figure()).length===0&&tableRows(comparison()).length===0&&canvas()?.getAttribute('tabindex')==='-1');
 get('energy-reset').click();
 await wait('reset recovers initial dataset and usable chart',()=>dataset().dataset.week==='1'&&points().length===14&&values(figure())[0]?.join('|')==='Mon|42|50'&&values(comparison())[0]?.join('|')==='Mon|42|50'&&data('kind')==='line'&&canvas()?.getAttribute('tabindex')==='0');
 snapshots.recovered=snapshot();

 layout.wide={history:shapeFacts(figure()),comparison:shapeFacts(comparison()),frame:get('chart-workflows-frame').getBoundingClientRect().width};
 get('chart-workflows-narrow').click();
 await wait('Dioxus narrow preview resizes actual SVG coordinate systems',()=>get('chart-workflows-frame').getBoundingClientRect().width<=341&&[figure(),comparison()].every(node=>{
  const facts=shapeFacts(node);return facts.width>100&&facts.width<=341&&Math.abs(facts.width-facts.viewBoxWidth)<=1&&facts.contained;
 }));
 layout.narrow={history:shapeFacts(figure()),comparison:shapeFacts(comparison()),frame:get('chart-workflows-frame').getBoundingClientRect().width,
  pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(layout.narrow.pageScroll<=layout.narrow.pageClient+1,'narrow chart content does not overflow the page');

 // Exactly one export request. Decode and validate its bytes; never certify host saving.
 control('Save as image').click();
 await wait('one export delivers callback bytes and ready PNG link',()=>Number(exportStatus()?.dataset.bytes)>33&&Number(exportStatus()?.dataset.width)>100&&Number(exportStatus()?.dataset.height)>100&&toolbox().querySelector('a.ix-chart-toolbox-download')?.href.startsWith('data:image/png;base64,')&&!control('Save as image').disabled);
 const link=toolbox().querySelector('a.ix-chart-toolbox-download'),binary=atob(link.href.split(',')[1]);
 assert([...binary.slice(0,8)].map(value=>value.charCodeAt(0)).join(',')==='137,80,78,71,13,10,26,10','ready link contains real PNG signature');
 assert(binary.slice(12,16)==='IHDR','ready PNG has IHDR image dimensions');
 const read32=offset=>((binary.charCodeAt(offset)*16777216)+(binary.charCodeAt(offset+1)<<16)+(binary.charCodeAt(offset+2)<<8)+binary.charCodeAt(offset+3));
 png={width:Number(exportStatus().dataset.width),height:Number(exportStatus().dataset.height),bytes:Number(exportStatus().dataset.bytes),
  ihdrWidth:read32(16),ihdrHeight:read32(20),downloadName:link.getAttribute('download')};
 assert(binary.length===png.bytes&&png.width===png.ihdrWidth&&png.height===png.ihdrHeight,'callback byte count/dimensions match actual ready PNG');
 const image=new Image();image.src=link.href;
 await wait('ready PNG decodes in actual WebView image engine',()=>image.complete&&image.naturalWidth>0&&image.naturalHeight>0);
 png.decodedWidth=image.naturalWidth;png.decodedHeight=image.naturalHeight;
 assert(png.decodedWidth===png.width&&png.decodedHeight===png.height&&png.downloadName?.endsWith('.png'),'decoded PNG and download-ready name match callback');
 assert(toolbox().querySelector('.ix-chart-toolbox-export-status').textContent.includes('PNG ready'),'toolbox reports ready export status');
 return {ok:true,layout,snapshots,png,inputFocusFacts,
  checks:['actual two-series7-category plot and accessible table','independent companion legend toggle and native expandable table','line/bar callbacks and geometry','native focus keyboard inspection and brush/back/restore','data-view valid edit applies to actual plot','invalid data stays atomic and draft retained','Escape restores trigger focus','external replacement closes stale editor and drops draft','empty dataset and reset recovery','Dioxus resize and narrow containment','single PNG export callback/signature/IHDR/actual decoded ready link'],
  input:'synthetic native click/input/keyboard events and explicit focus with actual SVG/layout/image decoding',
  trustedInput:'separate Chromium ordinary typing, keyboard activation and pointer workflows',
  download:'one automatic host request and ready link; actual host file saving is not certified',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),layout,snapshots,png,inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
