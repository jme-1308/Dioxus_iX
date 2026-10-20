//! Excluded actual-gallery composed asset workspace and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/asset_workspace.rs"]
mod gallery_workspace;

#[component]
pub fn AssetWorkspacePreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_workspace::STYLES} }
        IxTheme { class: "asset-workspace-review",
            h1 { "Composed asset workspace" }
            Button { id: "asset-workspace-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "asset-workspace-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_workspace::AssetWorkspaceExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_ASSET_WORKSPACE_CHECK);
                        if let Err(error) = query.send("assetworkspace") {
                            eprintln!("Asset workspace native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native asset workspace: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native asset workspace failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Asset workspace native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.asset-workspace-review{box-sizing:border-box;min-height:100vh;padding:24px}.asset-workspace-review>h1{font-size:28px;margin:0 0 24px}.asset-workspace-review h2{font-size:22px;margin:24px 0 16px}.asset-workspace-review p{max-width:72ch;line-height:1.5}#asset-workspace-frame{margin-top:24px}@media(max-width:600px){.asset-workspace-review{padding:16px}}";

// Excluded driver JS sends synthetic events and observes real asset/model/field
// state. Untrusted native Enter/Space/Tab do not activate browser defaults:
// permission is checked, then one explicit DOM click exercises the action.
// Normal page scroll positions actual test targets before programmatic focus;
// no artificial scrollport, production DOM bridge or chronology instrumentation.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_ASSET_WORKSPACE_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('asset-workspace'),result=()=>get('workspace-result');
 const view=()=>result()?.dataset.view,count=name=>Number(result()?.dataset[name]);
 const detail=id=>get('workspace-detail-'+id),header=id=>get('workspace-header-'+id),title=id=>header(id)?.querySelector('h1');
 const back=id=>header(id)?.querySelector('button[aria-label="Return to asset overview"]');
 const condition=id=>get('workspace-condition-'+id),noteTile=id=>get('workspace-note-tile-'+id);
 const note=id=>get('workspace-note-'+id),check=id=>get('workspace-check-'+id),status=id=>get('workspace-status-'+id);
 const temp=id=>get('workspace-temperature-'+id),temperature=id=>parseFloat(temp(id)?.textContent);
 const record=id=>get('workspace-asset-'+id),recordField=(id,field)=>record(id)?.querySelector('[data-field="'+field+'"]')?.textContent.trim();
 const checked=node=>node instanceof HTMLInputElement?node.checked:node.getAttribute('aria-checked')==='true';
 const near=(actual,expected)=>Math.abs(actual-expected)<0.11;
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; result='+result()?.textContent+'; view='+view()+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const key=(node,key)=>{const event=new KeyboardEvent('keydown',{key,code:key===' '?'Space':key,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' native focus established',()=>active()===node)};
 const open=async id=>{
  get('workspace-open-'+id).click();await wait('actual '+id+' record details opened',()=>view()===id&&!detail(id).hidden&&detail(id==='motor'?'pump':'motor').hidden&&title(id)?.textContent.includes(id==='motor'?'Motor 001':'Pump 002'));
 };
 const overview=async id=>{
  const before=count('back');back(id).click();await wait('localized native back returns real asset overview once',()=>view()==='overview'&&count('back')===before+1&&get('workspace-assets').getClientRects().length>0&&detail('motor').hidden&&detail('pump').hidden);
 };
 const setNote=async(id,value)=>{
  const node=note(id);assert(view()===id&&node,'actual current asset preparation field exists');
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait(id+' actual local preparation note applied',()=>note(id)===node&&note(id).value===value);
 };
 const setSize=async value=>{
  const node=get('workspace-size');node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));
  await wait('actual '+value+' size reaches both composed tiles',()=>node.value===value&&condition('motor').classList.contains('ix-tile-'+value)&&noteTile('motor').classList.contains('ix-tile-'+value));
 };
 const snapshot=()=>({view:view(),back:count('back'),runs:count('runs'),samples:count('samples'),inspections:count('inspections'),reviews:count('reviews'),secondary:checked(get('workspace-secondary')),ellipsis:checked(get('workspace-ellipsis')),longTitle:checked(get('workspace-long-title')),composed:checked(get('workspace-composed')),size:get('workspace-size')?.value,assets:['motor','pump'].map(id=>({id,running:record(id)?.dataset.running,samples:record(id)?.dataset.samples,temperature:temp(id)?.textContent.trim(),inspection:get('workspace-inspection-state-'+id)?.textContent.trim(),condition:get('workspace-condition-state-'+id)?.textContent.trim(),note:note(id)?.value,ready:check(id)?.checked}))});
 const hitTestable=node=>{
  const box=node?.getBoundingClientRect(),frame=get('asset-workspace-frame')?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  failureGeometry={node:node?.id,active:active()?.id,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,scrollTop:document.documentElement.scrollTop,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className,label:hit.getAttribute('aria-label')}:null};
  return !!box&&box.width>0&&box.height>0&&box.left>=frame.left-1&&box.right<=frame.right+1&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };
 const inspect=async(id,desired)=>{
  const before=count('inspections');get('workspace-inspect-'+id).click();
  await wait('independent '+id+' header inspection readiness updated',()=>count('inspections')===before+1&&get('workspace-inspect-'+id).getAttribute('aria-pressed')===String(desired)&&get('workspace-inspection-state-'+id).textContent===(desired?'Inspection ready':'Inspection needed'));
 };
 const review=async(id,desired)=>{
  const before=count('reviews');get('workspace-ack-'+id).click();
  await wait('independent '+id+' condition review updated',()=>count('reviews')===before+1&&get('workspace-condition-state-'+id).textContent===(desired?'Reviewed':'Needs review'));
 };

 await wait('actual overview plus both retained asset detail scopes mounted',()=>host()&&result()&&get('workspace-assets')&&record('motor')&&record('pump')&&header('motor')&&header('pump')&&note('motor')&&note('pump'));
 await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(view()==='overview'&&detail('motor').hidden&&detail('pump').hidden&&get('workspace-assets').querySelectorAll('tbody tr').length===2&&recordField('motor','asset').includes('Motor 001')&&recordField('pump','asset').includes('Pump 002'),'overview presents actual local asset records while mounted details are hidden');
 assert(record('motor').dataset.running==='true'&&record('pump').dataset.running==='false'&&record('motor').dataset.samples==='0'&&record('pump').dataset.samples==='0'&&near(parseFloat(recordField('motor','temperature')),42.6)&&near(parseFloat(recordField('pump','temperature')),36.2)&&['back','runs','samples','inspections','reviews'].every(name=>count(name)===0),'initial actual table/model values and action counters');
 assert(note('motor').value==='Prepare Motor 001'&&note('pump').value==='Prepare Pump 002'&&!check('motor').checked&&!check('pump').checked&&note('motor').getClientRects().length===0,'initial private preparation notes/checks are retained but hidden');
 await focus(get('workspace-reset'),'stable native workspace toolbar');note('motor').focus();assert(active()===get('workspace-reset'),'hidden retained asset note cannot take native focus');
 await focus(get('workspace-open-motor'),'native Motor overview action');await wait('overview navigation receives actual pointer hit',()=>hitTestable(get('workspace-open-motor')));hitFacts.open=failureGeometry.hit;
 const openKey=key(get('workspace-open-motor'),'Enter');keyPermissions.openEnter=!openKey.defaultPrevented;assert(!openKey.defaultPrevented&&view()==='overview','native overview Enter default remains available without synthetic activation');
 await open('motor');assert(title('motor').textContent==='Motor 001'&&header('motor').textContent.includes('Plant A / Line 1')&&status('motor').textContent==='Running'&&near(temperature('motor'),42.6)&&temp('motor').textContent.includes('°C'),'native overview action shows real Motor detail record');snapshots.overview=snapshot();

 const primaryFont=parseFloat(getComputedStyle(title('motor')).fontSize);await focus(get('workspace-run-motor'),'native Motor run action');await wait('header Run action receives actual pointer hit',()=>hitTestable(get('workspace-run-motor')));hitFacts.run=failureGeometry.hit;
 get('workspace-run-motor').click();await wait('Stop updates real Motor header and overview record independently',()=>count('runs')===1&&status('motor').textContent==='Stopped'&&record('motor').dataset.running==='false'&&recordField('motor','state')==='Stopped'&&view()==='motor');
 get('workspace-run-motor').click();await wait('Run restores real Motor running state',()=>count('runs')===2&&status('motor').textContent==='Running'&&record('motor').dataset.running==='true');
 get('workspace-sample-motor').click();await wait('Sample updates actual temperature and per-asset/global sample records',()=>count('samples')===1&&record('motor').dataset.samples==='1'&&near(temperature('motor'),43.6)&&near(parseFloat(recordField('motor','temperature')),43.6));
 assert(count('back')===0&&count('inspections')===0&&count('reviews')===0&&note('motor').value==='Prepare Motor 001','page actions do not navigate, inspect, review or replace preparation');
 await focus(back('motor'),'localized native Back action');const backKey=key(back('motor'),' ');keyPermissions.backSpace=!backKey.defaultPrevented;assert(!backKey.defaultPrevented,'native Back Space default remains available');await overview('motor');await open('motor');assert(near(temperature('motor'),43.6)&&status('motor').textContent==='Running','back/reopen retains current actual model state');snapshots.headerActions=snapshot();

 const actualC=temperature('motor');get('workspace-units-motor').click();await wait('composed native unit action renders actual Fahrenheit conversion',()=>temp('motor').textContent.includes('°F')&&near(temperature('motor'),actualC*1.8+32));
 assert(record('motor').dataset.samples==='1'&&near(parseFloat(recordField('motor','temperature')),actualC)&&count('samples')===1,'presentation units do not fabricate sensor samples');
 get('workspace-units-motor').click();await wait('Celsius presentation restored',()=>temp('motor').textContent.includes('°C')&&near(temperature('motor'),actualC));
 await inspect('motor',true);await review('motor',true);assert(get('workspace-inspect-motor').getAttribute('aria-pressed')==='true'&&count('runs')===2&&count('samples')===1&&status('motor').textContent==='Running','tile header/footer actions remain independent of actual run/sample state');await review('motor',false);snapshots.tileActions=snapshot();

 const retainedMotor=note('motor'),retainedMotorCheck=check('motor'),retainedPump=note('pump');await focus(note('motor'),'native Motor preparation note');await wait('actual preparation note receives native hit testing',()=>hitTestable(note('motor')));hitFacts.note=failureGeometry.hit;
 for(const value of ['Home','ArrowLeft',' ']){const event=key(note('motor'),value);keyPermissions['note'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native preparation '+value+' default remains available')}
 await setNote('motor','Prepare motor seals');check('motor').click();await wait('actual Motor toolkit checkbox updated',()=>check('motor').checked);
 get('workspace-sample-motor').click();await wait('model prop update retains modified local Motor preparation',()=>count('samples')===2&&record('motor').dataset.samples==='2'&&near(temperature('motor'),44.6)&&note('motor')===retainedMotor&&note('motor').value==='Prepare motor seals'&&check('motor').checked);
 await overview('motor');await open('pump');assert(note('pump')===retainedPump&&note('pump').value==='Prepare Pump 002'&&!check('pump').checked&&status('pump').textContent==='Stopped'&&near(temperature('pump'),36.2),'Pump details have their own untouched record and local preparation');
 await focus(note('pump'),'native Pump preparation note');await setNote('pump','Prepare pump gasket');check('pump').click();await wait('actual Pump toolkit checkbox updated independently',()=>check('pump').checked);
 get('workspace-units-pump').click();await wait('Pump local unit presentation switched',()=>temp('pump').textContent.includes('°F')&&near(temperature('pump'),36.2*1.8+32));
 get('workspace-run-pump').click();get('workspace-sample-pump').click();await wait('Pump run/sample update only its actual record',()=>count('runs')===3&&count('samples')===3&&record('pump').dataset.running==='true'&&record('pump').dataset.samples==='1'&&near(temperature('pump'),37.2*1.8+32));
 await overview('pump');await open('motor');assert(note('motor')===retainedMotor&&check('motor')===retainedMotorCheck&&note('motor').value==='Prepare motor seals'&&check('motor').checked&&temp('motor').textContent.includes('°C')&&near(temperature('motor'),44.6),'returning to Motor restores its own retained local note/check/units');
 await overview('motor');await open('pump');assert(note('pump')===retainedPump&&note('pump').value==='Prepare pump gasket'&&check('pump').checked&&temp('pump').textContent.includes('°F')&&near(temperature('pump'),37.2*1.8+32),'returning to Pump preserves its separate local note/check/unit scope');await overview('pump');await open('motor');snapshots.retainedAssets=snapshot();

 const options=get('workspace-options');if(!options.open)options.querySelector('summary').click();await wait('native Workspace options opened',()=>options.open);
 get('workspace-long-title').click();await wait('actual long asset title uses retained native title attribute',()=>title('motor').textContent.includes('production monitoring and maintenance preparation')&&title('motor').getAttribute('title')===title('motor').textContent);
 assert(header('motor').getAttribute('data-ellipsis')==='false'&&getComputedStyle(title('motor')).whiteSpace!=='nowrap','legacy default wrapping remains active for long title');
 get('workspace-ellipsis').click();await wait('explicit ellipsis controls actual native title overflow',()=>header('motor').getAttribute('data-ellipsis')==='true'&&getComputedStyle(title('motor')).textOverflow==='ellipsis'&&getComputedStyle(title('motor')).whiteSpace==='nowrap');
 get('workspace-secondary').click();await wait('secondary page header renders its smaller actual title treatment',()=>checked(get('workspace-secondary'))&&parseFloat(getComputedStyle(title('motor')).fontSize)<primaryFont);
 assert(note('motor')===retainedMotor&&note('motor').value==='Prepare motor seals'&&note('pump')===retainedPump&&check('motor').checked,'header appearance/long-title changes retain both private preparation scopes');
 get('workspace-ellipsis').click();get('workspace-long-title').click();await wait('normal wrapping/title restored independently of secondary mode',()=>header('motor').getAttribute('data-ellipsis')==='false'&&title('motor').textContent==='Motor 001'&&checked(get('workspace-secondary')));snapshots.headerOptions=snapshot();

 get('workspace-composed').click();await wait('text fallback replaces only explicit tile heading regions',()=>!checked(get('workspace-composed'))&&!get('workspace-sensor-motor')&&!get('workspace-units-motor')&&condition('motor').querySelector('.ix-tile-header')?.textContent.includes('Condition')&&condition('motor').querySelector('.ix-tile-subheader')?.textContent==='Temperature'&&get('workspace-inspect-motor'));
 await inspect('motor',false);assert(note('motor')===retainedMotor&&note('motor').value==='Prepare motor seals'&&check('motor').checked,'text header fallback keeps independent header actions and unchanged note body');
 get('workspace-composed').click();await wait('composed header/subheader actions restored without replacing tile body',()=>checked(get('workspace-composed'))&&get('workspace-sensor-motor')?.textContent==='Local sensor'&&get('workspace-units-motor')&&condition('motor').querySelector('.ix-tile-header svg')&&note('motor')===retainedMotor);
 for(const size of ['small','big','medium']){await setSize(size);assert(note('motor')===retainedMotor&&note('motor').value==='Prepare motor seals'&&check('motor').checked&&note('pump')===retainedPump,'size '+size+' preserves existing native note/check scopes')}
 assert(temp('motor').textContent.includes('°C')&&near(temperature('motor'),44.6)&&get('workspace-condition-state-motor').textContent==='Needs review','header/sizes preserve local unit/review presentation and model temperature');snapshots.tileOptions=snapshot();

 const resetButton=get('workspace-reset'),oldMotor=note('motor'),oldPump=note('pump'),sizeControl=get('workspace-size');await focus(resetButton,'native Restore asset workspace');resetButton.click();
 await wait('Restore resets actual asset model/options/counts and private owned fields',()=>view()==='overview'&&detail('motor').hidden&&detail('pump').hidden&&['back','runs','samples','inspections','reviews'].every(name=>count(name)===0)&&record('motor').dataset.running==='true'&&record('pump').dataset.running==='false'&&record('motor').dataset.samples==='0'&&record('pump').dataset.samples==='0'&&near(parseFloat(recordField('motor','temperature')),42.6)&&near(parseFloat(recordField('pump','temperature')),36.2)&&note('motor')?.value==='Prepare Motor 001'&&note('pump')?.value==='Prepare Pump 002'&&!check('motor').checked&&!check('pump').checked&&get('workspace-size').value==='medium'&&!checked(get('workspace-secondary'))&&!checked(get('workspace-ellipsis'))&&!checked(get('workspace-long-title'))&&checked(get('workspace-composed')));
 assert(!oldMotor.isConnected&&!oldPump.isConnected&&note('motor')!==oldMotor&&note('pump')!==oldPump&&get('workspace-reset')===resetButton&&get('workspace-size')===sizeControl&&get('workspace-options')===options&&active()===resetButton,'reset replaces only owned overview/detail preparation and preserves toolbar/options native focus');snapshots.reset=snapshot();

 get('asset-workspace-narrow').click();await wait('340px asset frame without horizontal page overflow',()=>get('asset-workspace-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);await open('motor');
 for(const id of ['workspace-run-motor','workspace-sample-motor','workspace-inspect-motor','workspace-units-motor','workspace-ack-motor']){
  await focus(get(id),'narrow independent '+id);await wait('narrow actual control fits and receives pointer hit '+id,()=>hitTestable(get(id)));hitFacts[id]=failureGeometry.hit;
 }
 get('workspace-run-motor').click();get('workspace-sample-motor').click();await wait('narrow header actions update real Motor model',()=>count('runs')===1&&count('samples')===1&&status('motor').textContent==='Stopped'&&record('motor').dataset.running==='false'&&near(temperature('motor'),43.6));
 get('workspace-units-motor').click();await wait('narrow composed units control remains functional',()=>temp('motor').textContent.includes('°F')&&near(temperature('motor'),43.6*1.8+32));await inspect('motor',true);await review('motor',true);
 await focus(note('motor'),'narrow native Motor preparation input');await wait('narrow note input fits and receives actual pointer hit',()=>hitTestable(note('motor')));await setNote('motor','Narrow motor preparation');check('motor').click();await wait('narrow native toolkit checkbox remains usable',()=>check('motor').checked);
 await focus(back('motor'),'narrow localized native Back');await wait('narrow page header Back fits and receives actual pointer hit',()=>hitTestable(back('motor')));await overview('motor');await focus(get('workspace-open-pump'),'narrow Pump overview action');await wait('narrow real Pump navigation remains hit-testable',()=>hitTestable(get('workspace-open-pump')));await open('pump');
 await focus(note('pump'),'narrow separate Pump preparation input');await wait('narrow separate asset note fits and receives actual pointer hit',()=>hitTestable(note('pump')));
 const frame=get('asset-workspace-frame').getBoundingClientRect(),input=note('pump').getBoundingClientRect();
 layout={frameWidth:frame.width,headerWidth:header('pump').getBoundingClientRect().width,conditionWidth:condition('pump').getBoundingClientRect().width,noteTileWidth:noteTile('pump').getBoundingClientRect().width,inputWidth:input.width,inputHeight:input.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(view()==='pump'&&count('back')===1&&count('runs')===1&&count('samples')===1&&count('inspections')===1&&count('reviews')===1&&note('motor').value==='Narrow motor preparation'&&check('motor').checked&&note('pump').value==='Prepare Pump 002'&&!check('pump').checked&&status('pump').textContent==='Stopped'&&near(temperature('pump'),36.2)&&layout.pageScroll<=layout.pageClient+1,'narrow composed page/tile controls and independent retained asset notes produce real contained workflow');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,hitFacts,
  checks:['actual overview/native-open records and hidden retained detail fields','independent page Run/Sample/Back update actual model/table/temperature','composed tile units/inspection/review update independent real states','private notes/checks/units retained per asset across navigation and model updates','secondary/long-title/wrapping/ellipsis preserve native header behavior and local fields','text/composed heading precedence plus three tile sizes retain body and independent actions','Restore resets owned model/detail scopes/options/counts while preserving toolbar focus','340px real header/tile/back/navigation/input pointer hits and contained workflow'],
  input:'synthetic form/click/key events with explicit native focus and normal page scroll setup; native Enter/Space/Tab defaults are not simulated',
  trustedInput:'separate Chromium native Enter/Space/Tab/typing/pointer interactions and visual checks; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
