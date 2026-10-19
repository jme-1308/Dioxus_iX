// Pinned dioxus-html =0.8.0-alpha.2 MountedData (events/mounted.rs) can
// focus and measure a registered control, but cannot enumerate current keyed
// or wrapped descendant order/eligibility. Observe only this Rust-owned Menu;
// Dioxus handlers own pointer, keyboard, focus and component cleanup. Native
// manual popover activation also has no MountedData equivalent. Foreign/open
// shadow scopes or reparented Category trees are not discovered here.
const MENU_DOM_FACTS: &str = r#"
const menuNodes=root=>{
  const direct=zone=>zone?Array.from(zone.children).flatMap(node=>node.matches('[data-menu-entry]')?[node]:node.tagName==='A'?Array.from(node.children).filter(child=>child.matches('[data-menu-entry]')):[]):[];
  return [...direct(root.querySelector('[data-menu-light="primary"]')),...direct(root.querySelector('[data-menu-light="bottom"]')),...direct(root.querySelector('[data-menu-utilities]'))].filter(node=>node.dataset.menuOwner===root.id&&node.closest('[data-menu-root]')===root);
};
const menuFacts=root=>menuNodes(root).map(node=>{
  const parent=node.parentElement;
  const grandparent=parent?.matches('[data-menu-utilities]')?root.querySelector('[role="menubar"]'):parent?.tagName==='A'?root:root.parentElement;
  return {id:Number(node.dataset.menuEntry),disabled:node.hasAttribute('disabled'),hidden:node.hasAttribute('hidden'),display:node.style.display,grandparent_display:grandparent?.style.display??''};
});
const categories=root=>Array.from(root.querySelectorAll('[data-menu-category-root]')).filter(node=>node.dataset.menuOwner===root.id&&node.closest('[data-menu-root]')===root);
const categoryFacts=(root,category)=>Array.from(category.querySelectorAll('[data-menu-category-child]')).filter(node=>node.dataset.menuCategoryOwner===root.id&&node.dataset.menuCategory===category.dataset.menuCategory).map(node=>{
  const host=node.closest('[data-ix-item-kind="menu"]'),source=host&&category.contains(host)?host:node;
  return {id:Number(node.dataset.menuCategoryChild),disabled:source===node?node.dataset.menuCategoryDisabled==='true':source.hasAttribute('disabled'),hidden:source.hasAttribute('hidden'),tabindex:source.getAttribute('tabindex')};
});
"#;
fn menu_roving_bridge(owner: &str) -> String {
    let id = serde_json::to_string(owner).expect("menu identity serialization");
    format!("{MENU_DOM_FACTS}\n{}",r#"
const root=document.getElementById(__MENU_ROOT__);
const transport=root?.querySelector('input[data-menu-observer]');
if(!root||!transport)return;
let previous='',previousCategories='';
const current=()=>root.isConnected&&document.getElementById(__MENU_ROOT__)===root;
const send=report=>{
  if(!current()||!transport.isConnected)return;
  transport.value=JSON.stringify({owner:root.id,...report});
  transport.dispatchEvent(new Event('input',{bubbles:true,cancelable:false}));
};
function sync(){
  if(!current())return;
  const items=menuFacts(root),serialized=JSON.stringify(items);
  if(serialized!==previous){previous=serialized;send({kind:'order',items});}
  const currentCategories=categories(root),facts=[];
  for(const category of currentCategories){
    const panel=category.querySelector(':scope > [data-menu-category-popup]');
    for(const surface of category.querySelectorAll(':scope > [data-menu-category-panel], :scope > [data-menu-category-empty-popup]'))if(typeof surface.showPopover==='function'){
      const shown=surface.matches(':popover-open');
      const visible=surface===panel&&category.dataset.menuCategoryPopupOpen==='true'&&!surface.hidden&&surface.getAttribute('popover')==='manual';
      if(visible&&!shown){
        const inert=surface.inert;surface.inert=true;
        try{surface.showPopover();}finally{surface.inert=inert;}
      }else if(!visible&&shown)surface.hidePopover();
    }
    facts.push({kind:'category-order',category:Number(category.dataset.menuCategoryRoot),category_items:categoryFacts(root,category)});
    category.dataset.categoryNative='true';
  }
  const serializedCategories=JSON.stringify(facts);
  if(serializedCategories!==previousCategories){previousCategories=serializedCategories;for(const report of facts)send(report);}
}
const observer=new MutationObserver(sync);
observer.observe(root,{childList:true,subtree:true,attributes:true,attributeFilter:['disabled','hidden','style','tabindex','data-menu-category-disabled','data-menu-category-mode','data-menu-category-popup-open','data-menu-category-body-surface','data-menu-category-popup','data-open','popover']});
try{
  sync();send({kind:'ready',items:menuFacts(root)});root.dataset.menuNative='true';
  await dioxus.recv();
}finally{
  observer.disconnect();delete root.dataset.menuNative;
  for(const category of categories(root)){
    for(const panel of category.querySelectorAll(':scope > [data-menu-category-panel], :scope > [data-menu-category-empty-popup]'))if(typeof panel.hidePopover==='function'&&panel.matches(':popover-open'))panel.hidePopover();
    delete category.dataset.categoryNative;
  }
}
"#.replace("__MENU_ROOT__",&id))
}
