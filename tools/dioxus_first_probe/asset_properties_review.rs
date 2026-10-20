//! Excluded actual-gallery asset properties preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/asset_properties.rs"]
mod gallery_properties;

#[component]
pub fn AssetPropertiesPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_properties::STYLES} }
        IxTheme { class: "asset-properties-review",
            h1 { "Asset properties" }
            Button { id: "asset-properties-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "asset-properties-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_properties::AssetPropertiesExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_ASSET_PROPERTIES_CHECK);
                        if let Err(error) = query.send("assetproperties") {
                            eprintln!("Asset properties native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native asset properties: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native asset properties failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Asset properties native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.asset-properties-review{box-sizing:border-box;min-height:100vh;padding:24px}.asset-properties-review>h1{font-size:28px;margin:0 0 24px}.asset-properties-review h2{font-size:22px;margin:24px 0 16px}.asset-properties-review p{max-width:72ch;line-height:1.5}#asset-properties-frame{margin-top:24px}@media(max-width:600px){.asset-properties-review{padding:16px}}";

// Excluded actual-gallery replay: synthetic form events and explicit native focus;
// browser typing/Tab/default activation are separate trusted evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_ASSET_PROPERTIES_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('properties-result');
 const asset=()=>result()?.dataset.asset,list=id=>get('properties-list-'+id);
 const input=(name,id=asset())=>get('properties-'+name+'-'+id),row=(name,id=asset())=>get('properties-row-'+id+'-'+name);
 const apply=(id=asset())=>get('properties-apply-'+id),cancel=(id=asset())=>get('properties-cancel-'+id);
 const record=id=>get('properties-record-'+id),stored=(id,name)=>record(id)?.querySelector('[data-field="'+name+'"]')?.textContent.trim();
 const count=name=>Number(result()?.dataset[name]),order=(id=asset())=>[...list(id).querySelectorAll('.ix-key-value')].map(node=>node.id);
 const custom=(name,id=asset())=>row(name,id)?.querySelector('.ix-key-value-custom'),label=(name,id=asset())=>row(name,id)?.querySelector('dt label');
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; active='+(active()?.id||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const setSelect=async(id,value)=>{const node=get(id);node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' native selection applied',()=>node.value===value)};
 const setChecked=async(id,value)=>{const node=get(id);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' native form setup applied',()=>node.checked===value)};
 const edit=async(name,value,id=asset())=>{const node=input(name,id);assert(!node.disabled&&!node.readOnly&&node.getClientRects().length>0,'actual editable visible '+name+' field');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual '+id+' '+name+' draft updated',()=>input(name,id)===node&&node.value===value)};
 const hitTestable=(node,insideFrame=true)=>{
  const box=node?.getBoundingClientRect(),frame=get('asset-properties-frame')?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  failureGeometry={node:node?.id||node?.tagName,active:active()?.id,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,scrollTop:document.documentElement.scrollTop,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className}:null};
  return !!box&&box.width>0&&box.height>0&&(!insideFrame||(box.left>=frame.left-1&&box.right<=frame.right+1))&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };
 const snapshot=()=>({asset:asset(),mode:result()?.dataset.mode,applies:count('applies'),cancels:count('cancels'),order:order(),records:['motor','pump'].map(id=>({id,name:stored(id,'name'),location:stored(id,'location'),note:stored(id,'note')})),drafts:['motor','pump'].map(id=>({id,name:input('name',id)?.value,location:input('location',id)?.value,note:input('note',id)?.value}))});

 await wait('actual asset editors/lists/native fields and records mounted',()=>result()&&list('motor')&&list('pump')&&['motor','pump'].every(id=>['name','location','note'].every(name=>input(name,id)&&row(name,id)&&stored(id,name)!==undefined)));
 await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(asset()==='motor'&&count('applies')===0&&count('cancels')===0&&stored('motor','name')==='Motor 001'&&stored('motor','location')==='Plant A / Line 1'&&stored('motor','note')==='Check bearing housing'&&stored('pump','name')==='Pump 002'&&stored('pump','location')==='Plant A / Line 2'&&stored('pump','note')==='Check inlet pressure','real initial saved asset records');
 assert(list('motor').getAttribute('role')==='group'&&!!list('motor').getAttribute('aria-label')&&['name','location','note'].every(name=>row(name).querySelector('dl>dt')&&row(name).querySelector('dl>dd')&&label(name)?.htmlFor===input(name).id&&!custom(name).hidden),'named native group and explicit semantic label linkage');
 for(const name of ['name','location','note']){await focus(input(name),'native '+name+' before label activation');await wait('visible '+name+' label receives pointer hit',()=>hitTestable(label(name)));hitFacts['label '+name]=failureGeometry.hit;await focus(apply(),'independent native Apply');label(name).click();await wait('actual label click focuses explicit '+name+' input',()=>active()===input(name))}
 for(const value of ['Home','ArrowLeft',' ','Tab']){const event=key(input('note'),value);keyPermissions[value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native input '+value+' default remains available')}
 assert(row('name').querySelector('svg[role="img"][aria-label="Equipment property"]'),'composed equipment icon has an accessible label');snapshots.semantics=snapshot();

 await edit('name','Motor 001 serviced');await edit('location','Plant B / Bay 4');await edit('note','Stage replacement bearing');const beforeApply=count('applies');await focus(apply(),'native Apply properties');apply().click();
 await wait('Apply updates actual Motor record once',()=>count('applies')===beforeApply+1&&stored('motor','name')==='Motor 001 serviced'&&stored('motor','location')==='Plant B / Bay 4'&&stored('motor','note')==='Stage replacement bearing'&&stored('pump','name')==='Pump 002');
 await edit('name','Unapplied name');await edit('location','Unapplied location');await edit('note','Unapplied note');const beforeCancel=count('cancels');cancel().click();await wait('Cancel restores latest actual saved Motor fields',()=>count('cancels')===beforeCancel+1&&input('name').value==='Motor 001 serviced'&&input('location').value==='Plant B / Bay 4'&&input('note').value==='Stage replacement bearing'&&count('applies')===beforeApply+1);snapshots.applyCancel=snapshot();

 const motorNodes=['name','location','note'].map(name=>input(name,'motor'));await edit('note','Motor draft retained');await setSelect('properties-asset','pump');await wait('actual Pump editor displayed',()=>asset()==='pump'&&input('name').getClientRects().length>0&&input('name').value==='Pump 002'&&input('note','motor').getClientRects().length===0);
 const pumpNode=input('note');await edit('name','Pump 002 inspected');await edit('note','Pump draft retained');await setSelect('properties-asset','motor');await wait('Motor draft survives asset switch',()=>asset()==='motor'&&input('note').value==='Motor draft retained'&&motorNodes.every((node,index)=>node===input(['name','location','note'][index],'motor')));await setSelect('properties-asset','pump');await wait('Pump draft remains independent and same native field',()=>asset()==='pump'&&input('note')===pumpNode&&input('note').value==='Pump draft retained'&&input('name').value==='Pump 002 inspected');
 const pumpApply=count('applies');apply().click();await wait('Apply saves only current actual Pump record',()=>count('applies')===pumpApply+1&&stored('pump','name')==='Pump 002 inspected'&&stored('pump','note')==='Pump draft retained'&&stored('motor','note')==='Stage replacement bearing');snapshots.perAsset=snapshot();

 const options=get('properties-options');options.querySelector('summary').click();await wait('native property options expanded',()=>options.open);
 await edit('note','Hidden Pump draft');await focus(get('properties-value-mode'),'visible mode selector');await setSelect('properties-value-mode','text');await wait('actual supplied saved text takes precedence over custom inputs',()=>result().dataset.mode==='text'&&apply().disabled&&cancel().disabled&&['name','location','note'].every(name=>custom(name).hidden&&custom(name).hasAttribute('inert')&&input(name).getClientRects().length===0&&row(name).querySelector('.ix-key-value-text')?.textContent===stored('pump',name)&&!label(name)));
 input('note').focus();assert(active()!==input('note')&&input('note')===pumpNode&&input('note').value==='Hidden Pump draft','hidden retained custom value cannot receive native focus');
 await setSelect('properties-value-mode','blank');await wait('Some empty text still overrides custom composition',()=>result().dataset.mode==='blank'&&apply().disabled&&cancel().disabled&&['name','location','note'].every(name=>row(name).querySelector('.ix-key-value-text')?.textContent===''&&custom(name).hidden&&input(name).getClientRects().length===0&&!label(name)));
 await setSelect('properties-value-mode','custom');await wait('custom native fields reappear without losing local draft/scope',()=>result().dataset.mode==='custom'&&input('note')===pumpNode&&input('note').value==='Hidden Pump draft'&&['name','location','note'].every(name=>!custom(name).hidden&&!custom(name).hasAttribute('inert')&&label(name)?.htmlFor===input(name).id));snapshots.precedence=snapshot();

 await setChecked('properties-disabled',true);await wait('disabled fields and model actions guarded',()=>['name','location','note'].every(name=>input(name).disabled)&&apply().disabled&&cancel().disabled);const guarded={applies:count('applies'),cancels:count('cancels'),note:stored('pump','note')};apply().click();cancel().click();await wait('disabled native actions cannot change actual records/counts',()=>count('applies')===guarded.applies&&count('cancels')===guarded.cancels&&stored('pump','note')===guarded.note);await setChecked('properties-disabled',false);
 await setChecked('properties-readonly',true);await wait('read-only native fields preserve selectable content and guarded actions',()=>['name','location','note'].every(name=>input(name).readOnly&&!input(name).disabled)&&apply().disabled&&cancel().disabled);await focus(input('note'),'read-only native note');assert(input('note').value==='Hidden Pump draft','read-only preserves actual local draft');apply().click();cancel().click();await wait('read-only actions cannot change model/counts',()=>count('applies')===guarded.applies&&count('cancels')===guarded.cancels&&stored('pump','note')===guarded.note);await setChecked('properties-readonly',false);await wait('editable state restored',()=>!input('note').readOnly&&!apply().disabled);snapshots.guards=snapshot();

 const preservedOrder=order(),preservedRows=['name','location','note'].map(name=>row(name));await setSelect('properties-label-position','left');await setChecked('properties-horizontal',true);await setChecked('properties-striped',true);await setChecked('properties-long-label',true);
 await wait('live list and label presentation options applied',()=>result().dataset.longLabel==='true'&&list('pump').dataset.horizontal==='true'&&list('pump').dataset.striped==='true'&&['name','location','note'].every(name=>row(name).dataset.horizontal==='true')&&input('note')===pumpNode&&input('note').value==='Hidden Pump draft');
 get('properties-reverse').click();await wait('stable-key reverse changes actual property order and preserves native scopes',()=>order().join('|')===[...preservedOrder].reverse().join('|')&&preservedRows.every((node,index)=>node===row(['name','location','note'][index]))&&input('note')===pumpNode&&input('note').value==='Hidden Pump draft');
 get('properties-reverse').click();await wait('ordinary property order restored',()=>order().join('|')===preservedOrder.join('|'));await setSelect('properties-label-position','top');await setChecked('properties-horizontal',false);await wait('top vertical presentation restored without draft loss',()=>list('pump').dataset.horizontal==='false'&&row('name').dataset.horizontal==='false'&&input('note').value==='Hidden Pump draft');snapshots.presentation=snapshot();

 const reset=get('properties-reset'),oldNote=input('note','motor'),assetSelect=get('properties-asset'),reverse=get('properties-reverse');await focus(reset,'Restore asset properties');reset.click();
 await wait('Restore actual initial records/options/counts and fresh owned drafts',()=>asset()==='motor'&&count('applies')===0&&count('cancels')===0&&result().dataset.mode==='custom'&&stored('motor','name')==='Motor 001'&&stored('motor','location')==='Plant A / Line 1'&&stored('motor','note')==='Check bearing housing'&&stored('pump','name')==='Pump 002'&&stored('pump','note')==='Check inlet pressure'&&input('note','motor')?.value==='Check bearing housing'&&input('note','pump')?.value==='Check inlet pressure'&&['properties-disabled','properties-readonly','properties-horizontal','properties-striped','properties-long-label'].every(id=>!get(id).checked));
 assert(!oldNote.isConnected&&input('note','motor')!==oldNote&&get('properties-reset')===reset&&get('properties-asset')===assetSelect&&get('properties-reverse')===reverse&&get('properties-options')===options&&active()===reset,'Restore replaces owned draft scopes and preserves toolbar/options/native focus');snapshots.reset=snapshot();

 get('asset-properties-narrow').click();await wait('actual 340px frame without page overflow',()=>get('asset-properties-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);await setChecked('properties-long-label',true);await setChecked('properties-striped',true);await setSelect('properties-label-position','left');await setChecked('properties-horizontal',true);
 for(const name of ['name','location','note']){await focus(input(name),'narrow actual '+name);await wait('narrow '+name+' field fits and receives pointer hit',()=>hitTestable(input(name)));hitFacts['narrow '+name]=failureGeometry.hit;assert(label(name)?.htmlFor===input(name).id,'narrow long label retains native linkage')}
 await edit('name','Motor 001 long inspection preparation');await edit('location','Plant A / North service area / Maintenance bay 14');await edit('note','Prepare replacement bearing and tools before planned inspection');await focus(apply(),'narrow Apply properties');await wait('narrow native Apply fits and receives pointer hit',()=>hitTestable(apply()));hitFacts.narrowApply=failureGeometry.hit;apply().click();await wait('narrow workflow updates actual Motor record',()=>count('applies')===1&&stored('motor','note')==='Prepare replacement bearing and tools before planned inspection'&&stored('motor','location')==='Plant A / North service area / Maintenance bay 14');
 await setSelect('properties-value-mode','text');await wait('narrow long saved values display with custom fields hidden',()=>input('name').getClientRects().length===0&&row('location').querySelector('.ix-key-value-text')?.textContent==='Plant A / North service area / Maintenance bay 14'&&list('motor').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);await setSelect('properties-value-mode','custom');
 const frame=get('asset-properties-frame').getBoundingClientRect(),box=input('note').getBoundingClientRect();layout={frameWidth:frame.width,listWidth:list('motor').getBoundingClientRect().width,inputWidth:box.width,inputHeight:box.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(layout.listWidth<=341&&layout.inputHeight>=31&&layout.pageScroll<=layout.pageClient+1&&input('note').value===stored('motor','note')&&stored('pump','note')==='Check inlet pressure','narrow long-content presentation preserves native editable fields and distinct actual records');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,hitFacts,
 checks:['actual saved asset records/named semantic list/native label click-focus and input key permission','Apply saves actual properties and Cancel restores latest saved record','per-asset native draft ownership survives selection and saves independently','saved text and empty text override retained custom scopes without losing drafts','disabled/read-only fields and actions preserve actual records','top/left/horizontal/striped/long-label and keyed reorder retain native fields','Restore resets owned drafts/records/options while preserving toolbar focus','340px long-label/content input/action hits and actual saved record workflow'],
 input:'synthetic form/click/key events, explicit native focus and normal page scroll setup; native typing/Enter/Space/Tab defaults are not simulated',trustedInput:'separate Chromium trusted label click/typing/Tab/default activation and visual evidence; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
