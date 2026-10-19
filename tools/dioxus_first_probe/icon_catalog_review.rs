//! Excluded actual-gallery icon preview and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/icon_catalog.rs"]
mod gallery_icons;

#[component]
pub fn IconCatalogPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_icons::STYLES} }
        IxTheme { class: "icon-catalog-review",
            h1 { "Icon catalog and scoped registration" }
            gallery_icons::IconCatalogExample {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_ICON_CATALOG_CHECK);
                        if let Err(error) = query.send("iconcatalog") {
                            eprintln!("Icon catalog native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native icon catalog: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native icon catalog failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Dioxus native icon catalog timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.icon-catalog-review{box-sizing:border-box;min-height:100vh;padding:24px}.icon-catalog-review h1{font-size:28px;margin:0 0 24px}.icon-catalog-review h2{font-size:22px;margin:24px 0 16px}.icon-catalog-review h3{font-size:18px}.icon-catalog-review p{max-width:72ch;line-height:1.5}.icon-catalog-review .demo-section{margin-bottom:32px;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.icon-catalog-review .demo-row{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:12px 0}.icon-catalog-review .review-icon-search{display:flex;flex-direction:column;gap:8px}@media(max-width:600px){.icon-catalog-review{padding:16px}}";

// Driver-only JavaScript dispatches synthetic form/click events, observes actual
// SVG geometry/style and explicitly requests native focus. It does not implement
// icon behavior or emulate trusted typing and keyboard button activation. Those
// ordinary input defaults are verified separately by the lead in Chromium.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_ICON_CATALOG_CHECK: &str = r#"
let geometry={},inputFocusFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const status=()=>get('icon-catalog-status'),previewStatus=()=>get('icon-preview-status');
 const choices=()=>[...get('icon-catalog-grid').querySelectorAll('button')];
 const svg=()=>get('icon-preview')?.querySelector('svg.ix-icon');
 const custom=()=>get('icon-registry-preview')?.querySelector('svg.ix-icon');
 const registryStatus=()=>get('icon-registry-status');
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; catalog='+status()?.textContent+'; preview='+previewStatus()?.textContent+'; registry='+registryStatus()?.textContent);
 };
 const setSearch=async(value,predicate)=>{
  const node=get('icon-catalog-search');
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);
  node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait('search '+value+' applied',()=>node.value===value&&status()?.dataset.page==='0'&&predicate());
 };
 const setSelect=async(id,value,data)=>{
  const node=get(id);node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));
  await wait(id+' selection applied',()=>previewStatus()?.dataset[data]===value);
 };
 const selectIcon=async name=>{
  const button=get('icon-choice-'+name);assert(button,'choice exists: '+name);
  button.focus();button.click();
  await wait('selected '+name,()=>previewStatus()?.dataset.name===name&&svg());
  assert(active()===button,'selection preserves explicitly established native button focus');
  assert(get('icon-preview-name').value===name&&get('icon-preview-name').readOnly,'selected canonical name is available in readonly native input');
 };
 const frameFacts=node=>{
  const box=node.getBoundingClientRect(),shape=node.querySelector('path,polygon,circle');
  const shapeBox=shape?.getBoundingClientRect(),inner=node.querySelector('svg');
  return {width:box.width,height:box.height,viewBox:node.getAttribute('viewBox'),innerViewBox:inner?.getAttribute('viewBox')??null,
   shape:shape?.localName,shapeWidth:shapeBox?.width??0,shapeHeight:shapeBox?.height??0,
   shapeInside:!!shapeBox&&shapeBox.left>=box.left-1&&shapeBox.top>=box.top-1&&shapeBox.right<=box.right+1&&shapeBox.bottom<=box.bottom+1,
   color:getComputedStyle(node).color,fill:shape?getComputedStyle(shape).fill:null};
 };
 const assertFrame=(facts,size,label)=>{
  assert(Math.abs(facts.width-size)<=1&&Math.abs(facts.height-size)<=1,label+' outer size');
  assert(facts.shapeWidth>0&&facts.shapeHeight>0&&facts.shapeInside,label+' visible geometry stays inside outer frame');
  assert(facts.fill===facts.color,label+' geometry inherits currentColor');
 };
 await wait('actual gallery icon catalog mounted',()=>status()&&previewStatus()&&svg()&&registryStatus()&&get('icon-catalog-next')&&get('icon-catalog-previous')&&get('icon-catalog-clear')&&choices().length===60);
 await wait('WebView document ready for focus input',()=>document.hasFocus());
 get('icon-catalog-search').focus();
 await wait('search input focus established',()=>active()===get('icon-catalog-search'));
 inputFocusFacts={documentFocused:document.hasFocus(),active:active()?.id};
 const initial={matches:Number(status().dataset.matches),pages:Number(status().dataset.pages),start:Number(status().dataset.start),end:Number(status().dataset.end)};
 assert(initial.matches===1473&&initial.pages===25&&initial.start===0&&initial.end===60,'complete allowed catalog starts with sixty of 1473 icons');
 assert(get('icon-catalog-previous').disabled&&!get('icon-catalog-next').disabled,'initial pagination bounds');
 const firstId=choices()[0].id;
 get('icon-catalog-next').click();
 await wait('second catalog page',()=>status().dataset.page==='1'&&status().dataset.start==='60'&&status().dataset.end==='120'&&choices().length===60);
 assert(choices()[0].id!==firstId&&!get('icon-catalog-previous').disabled,'next page changes actual choices');
 get('icon-catalog-previous').click();
 await wait('first catalog page restored',()=>status().dataset.page==='0'&&choices()[0]?.id===firstId);

 await setSearch('iconStarFilled',()=>get('icon-choice-star-filled'));
 await selectIcon('star-filled');
 await setSelect('icon-preview-size','48','size');
 await setSelect('icon-preview-color','primary','color');
 geometry.starFilled=frameFacts(svg());assertFrame(geometry.starFilled,48,'star-filled');
 assert(geometry.starFilled.viewBox==='0 0 24 24'&&geometry.starFilled.innerViewBox==='0 0 512 512','star-filled retains its actual nested 512 coordinate system');
 assert(svg().getAttribute('role')==='img'&&svg().getAttribute('aria-hidden')==='false'&&svg().getAttribute('aria-label')?.includes('star-filled'),'standalone preview has a meaningful accessible label');
 const decorative=document.querySelector('.icon-catalog-decorative svg.ix-icon');
 assert(decorative?.getAttribute('role')==='presentation'&&decorative.getAttribute('aria-hidden')==='true'&&!decorative.hasAttribute('aria-label'),'adjacent-text counterpart remains decorative');
 assert(getComputedStyle(svg()).color===getComputedStyle(get('icon-preview')).color,'primary color is inherited from preview container');

 await setSearch('about',()=>get('icon-choice-about'));
 await selectIcon('about');
 geometry.about=frameFacts(svg());assertFrame(geometry.about,48,'about');
 assert(geometry.about.innerViewBox==='0 0 512 512','about preserves nested 512 viewBox');
 await setSearch('aas-project-ax',()=>get('icon-choice-aas-project-ax'));
 await selectIcon('aas-project-ax');
 geometry.direct24=frameFacts(svg());assertFrame(geometry.direct24,48,'direct 24 icon');
 assert(geometry.direct24.viewBox==='0 0 24 24'&&geometry.direct24.innerViewBox===null,'direct geometry retains 24-unit outer framing');
 await setSearch('no-such-pictogram-xyz',()=>status().dataset.matches==='0'&&choices().length===0);
 assert(status().dataset.pages==='0'&&status().dataset.start==='0'&&status().dataset.end==='0'&&get('icon-catalog-previous').disabled&&get('icon-catalog-next').disabled,'empty search has honest pagination bounds');
 assert(previewStatus().dataset.name==='aas-project-ax','search preserves selected icon');
 get('icon-catalog-clear').click();
 await wait('clear restores complete catalog first page',()=>get('icon-catalog-search').value===''&&Number(status().dataset.matches)===initial.matches&&status().dataset.page==='0'&&choices().length===60);
 assert(get('icon-catalog-clear').disabled,'clear action disables when search is already empty');

 const isolated=()=>get('icon-registry-isolated').querySelector('svg');
 assert(registryStatus().dataset.count==='0'&&registryStatus().dataset.version==='none'&&!custom()&&!isolated(),'unknown custom names initially produce no SVG in either provider');
 get('icon-registry-register').click();
 await wait('custom circle registered',()=>registryStatus().dataset.count==='1'&&registryStatus().dataset.version==='registered'&&custom()?.querySelector('circle'));
 geometry.circle=frameFacts(custom());assertFrame(geometry.circle,48,'custom circle');
 assert(geometry.circle.viewBox==='0 0 40 20'&&custom().getAttribute('aria-label')==='Local status'&&!isolated(),'camelCase registration resolves with non-square framing and provider isolation');
 get('icon-registry-replace').click();
 await wait('alias replacement produces triangle',()=>registryStatus().dataset.count==='1'&&registryStatus().dataset.version==='replaced'&&custom()?.querySelector('path')&&!custom()?.querySelector('circle'));
 geometry.triangle=frameFacts(custom());assertFrame(geometry.triangle,48,'custom triangle');
 assert(geometry.triangle.viewBox==='0 0 40 20'&&geometry.triangle.shapeWidth>geometry.circle.shapeWidth&&!isolated(),'alias replacement changes geometry without adding a definition or leaking across providers');
 get('icon-registry-remove').click();
 await wait('alias removal clears actual preview',()=>registryStatus().dataset.count==='0'&&registryStatus().dataset.version==='removed'&&!custom()&&!isolated());
 assert(get('icon-registry-remove').disabled&&get('icon-registry-replace').disabled,'absent custom entry disables replacement/removal');
 return {ok:true,initial,geometry,inputFocusFacts,
  checks:['complete 1473-name catalog and sixty-entry next/back pagination','export-like alias search and empty/clear states','selection and readonly canonical name','48px and inherited primary color','standalone and decorative semantics','nested 512 and direct 24 geometry framing','scoped custom register/alias replacement/removal','non-square viewBox and independent provider isolation'],
  input:'synthetic DOM form/click events and explicit native focus, with actual SVG geometry/style',
  trustedInput:'separate Chromium typing, keyboard button activation and native select defaults',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),geometry,inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
