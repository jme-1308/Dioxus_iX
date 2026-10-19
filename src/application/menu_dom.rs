const MENU_CATEGORY_FACTS: &str = r#"
const categoryBelongs=(root,category)=>category instanceof HTMLElement&&category.isConnected&&category.hasAttribute('data-menu-category-root')&&category.dataset.menuOwner===root.id;
const categoryNodes=(root,category)=>Array.from(category.querySelectorAll('[data-menu-category-child]')).filter(node=>node.dataset.menuCategoryOwner===root.id&&node.dataset.menuCategory===category.dataset.menuCategory);
const categoryFacts=(root,category)=>categoryNodes(root,category).map(node=>{const host=node.closest('[data-ix-item-kind="menu"]'),source=host&&category.contains(host)?host:node;return{id:Number(node.dataset.menuCategoryChild),disabled:source===node?node.dataset.menuCategoryDisabled==='true':source.hasAttribute('disabled'),hidden:source.hasAttribute('hidden'),tabindex:source.getAttribute('tabindex')};});
"#;
// Pinned dioxus-html =0.8.0-alpha.2 MountedData (events/mounted.rs) can
// focus and measure a registered control, but cannot enumerate current keyed
// or wrapped descendants/order. Keep these bounded read-only membership and
// eligibility facts; Dioxus handlers own keyboard, click and focus behavior.
// Geometry/top-layer, hover/outside/removal and shared document discovery below
// remain legacy native plumbing for later R02 migration.
const MENU_DOM_FACTS: &str = r#"
const menuNodes = root => {
  const direct = zone => zone ? Array.from(zone.children).flatMap(node => node.matches('[data-menu-entry]') ? [node] : node.tagName === 'A' ? Array.from(node.children).filter(child => child.matches('[data-menu-entry]')) : []) : [];
  return [...direct(root.querySelector('[data-menu-light="primary"]')), ...direct(root.querySelector('[data-menu-light="bottom"]')), ...direct(root.querySelector('[data-menu-utilities]'))].filter(node => node.dataset.menuOwner === root.id && node.closest('[data-menu-root]') === root);
};
const menuFacts = root => menuNodes(root).map(node => {
  const parent=node.parentElement;
  // Observe the ancestry of the source host, before this Rust port's layout wrappers.
  const grandparent=parent?.matches('[data-menu-utilities]') ? root.querySelector('[role="menubar"]') : parent?.tagName==='A' ? root : root.parentElement;
  return {id:Number(node.dataset.menuEntry),disabled:node.hasAttribute('disabled'),hidden:node.hasAttribute('hidden'),display:node.style.display,grandparent_display:grandparent?.style.display ?? ''};
});
"#;
fn menu_roving_bridge(owner: &str) -> String {
    let id = serde_json::to_string(owner).expect("menu identity serialization");
    format!("{MENU_DOM_FACTS}\n{MENU_CATEGORY_FACTS}\n{}\n{}\n{}",crate::menu_scope_dom::MENU_SCOPE_DIRECTORY,crate::menu_hover_dom::MENU_HOVER_NATIVE_FACTS,r#"
const directory=menuScopeDirectory(),scopeLease=directory.acquire();
// One discovery pass at this actual Menu mount also covers a consumer that
// attached a new open tree after the document journal was initialized.
directory.discoverSubtree(document);
const root=directory.elementById(__MENU_ROOT__),transport=root?.querySelector('input[data-menu-observer]');
if (!root || !transport) {directory.release(scopeLease);return;}
const hoverTransport=root.querySelector('input[data-menu-hover-observer]');
document[Symbol.for('dioxus-ix-document-input')]?.get(hoverTransport?.dataset.menuDocumentOwner)?.sync();
const hover=installMenuCategoryHover(root,message=>{if(hoverTransport?.isConnected){hoverTransport.value=JSON.stringify(message);hoverTransport.dispatchEvent(new Event('input',{bubbles:true,cancelable:false}));}},directory);
let previous='',sequence=0;const categoryPrevious=new Map(),categoryBindings=new Map(),categoryConnections=new Map();
const categoryConnection=category=>{
  const ancestors=new Set();
  for(let node=category;node;node=node.parentNode??(node instanceof ShadowRoot?node.host:null))ancestors.add(node);
  return {category,id:Number(category.dataset.menuCategoryRoot),owner:category.dataset.menuOwner,ancestors};
};
const currentRoot=()=>directory.sameNodeById(root,__MENU_ROOT__)&&root.matches('[data-menu-root]');
const send=(kind,extra={})=>{
  if(!currentRoot()||!transport.isConnected)return;
  const report={owner:root.id,kind,...extra};
  if(kind==='category-outside'||kind==='category-disconnect')report.active_categories=menuCategoryActiveFacts();
  if(kind==='ready'||kind==='order'){
    const items=menuFacts(root);previous=JSON.stringify(items);report.items=items;
  }
  transport.value=JSON.stringify(report);
  transport.dispatchEvent(new Event('input',{bubbles:true,cancelable:false}));
};
const sync=()=>{
  hover.sync();
  if(!currentRoot())return;
  const facts=JSON.stringify(menuFacts(root));if(facts!==previous)send('order');
  const currentCategories=new Set();
  const currentObjects=new Set(directory.categories(root.id));
  for(const category of currentObjects) {
    // Retain the old physical ancestry until the shared mutation journal has
    // delivered removals, even if another observer synchronizes first.
    if(!categoryConnections.has(category))categoryConnections.set(category,categoryConnection(category));
    if(!categoryBindings.has(category)) {
      const listeners=new AbortController(),changes=new MutationObserver(sync);
      categoryBindings.set(category,{listeners,changes});
      changes.observe(category,{childList:true,subtree:true,attributes:true,attributeFilter:['disabled','hidden','style','tabindex','data-menu-category-disabled','data-menu-category-mode','data-menu-category-popup-open','data-menu-category-body-surface','data-menu-category-popup','data-open','popover']});
      // Focusout is retained only as an outside-dismissal observation.
      // It neither publishes root focus nor transports keyboard decisions.
      category.addEventListener('focusout',event=>queueCategoryOutside(event,category),{signal:listeners.signal});
    }
    const panel=category.querySelector(':scope > [data-menu-category-popup]');
    // Mirror Rust's retained collapsed surface into the native top layer.
    // Inert/no-source opening must not autofocus or install an invoker return.
    for(const surface of category.querySelectorAll(':scope > [data-menu-category-panel], :scope > [data-menu-category-empty-popup]'))if(typeof surface.showPopover==='function') {
      const shown=surface.matches(':popover-open');
      const visible=surface===panel&&category.dataset.menuCategoryPopupOpen==='true'&&!surface.hidden&&surface.getAttribute('popover')==='manual';
      if(visible&&!shown) {
        const inert=surface.inert;surface.inert=true;
        try{surface.showPopover();}finally{surface.inert=inert;}
      } else if(!visible&&shown)surface.hidePopover();
    }
    const id=Number(category.dataset.menuCategoryRoot),items=categoryFacts(root,category),serialized=JSON.stringify(items);currentCategories.add(id);
    if(categoryPrevious.get(id)!==serialized){categoryPrevious.set(id,serialized);send('category-order',{category:id,category_items:items});}
    category.dataset.categoryNative='true';category.dataset.categoryHoverNative='true';
  }
  for(const [category,binding]of categoryBindings)if(!currentObjects.has(category)){binding.listeners.abort();binding.changes.disconnect();categoryBindings.delete(category);}
  for(const id of categoryPrevious.keys())if(!currentCategories.has(id))categoryPrevious.delete(id);
};
function queueCategoryOutside(event,category) {
  // Upstream listens on its internal Dropdown, not on the parent control.
  // Keep that receiver boundary when the source shadow tree is flattened.
  const surface=category.querySelector(':scope > [data-menu-category-popup]');
  if(!surface||!event.composedPath().includes(surface))return;
  const identity=category.dataset.menuCategoryRoot;
  requestAnimationFrame(()=>{if(currentRoot()&&categoryBelongs(root,category)&&directory.categories(root.id,identity)[0]===category)send('category-outside',{category:Number(identity),inside:category.contains(document.activeElement),outside_cause:'focusout',sequence:++sequence,category_items:categoryFacts(root,category)});});
}
const observer=new MutationObserver(sync);observer.observe(root,{childList:true,subtree:true,attributes:true,attributeFilter:['disabled','hidden','style','tabindex','data-menu-category-disabled','data-menu-category-mode','data-menu-category-popup-open','data-menu-category-body-surface','data-menu-category-popup','data-open','popover']});
directory.subscribe(scopeLease,changes=>{
  const removals=[];
  for(const change of changes)if(change.type==='childList')for(const removed of change.removedNodes){
    for(const connection of categoryConnections.values())if(connection.ancestors.has(removed)||removed.contains(connection.category))removals.push(connection);
  }
  // Report actual object removal. Rust decides whether its retained controller
  // is present and authorizes dismissal/focus; DOM movement never unregisters it.
  for(const {category,id,owner}of removals)if(owner===root.id&&category.dataset.menuOwner===owner&&Number(category.dataset.menuCategoryRoot)===id){
    send('category-disconnect',{category:id,connected:category.isConnected,sequence:++sequence});
  }
  sync();
  const currentObjects=new Set(directory.categories(root.id));
  for(const category of categoryConnections.keys())if(!currentObjects.has(category))categoryConnections.delete(category);
  for(const category of currentObjects)categoryConnections.set(category,categoryConnection(category));
});
sync();send('ready');root.dataset.menuNative='true';
try{await dioxus.recv();}finally{hover.dispose();observer.disconnect();directory.release(scopeLease);delete root.dataset.menuNative;for(const [category,binding]of categoryBindings){binding.listeners.abort();binding.changes.disconnect();for(const panel of category.querySelectorAll(':scope > [data-menu-category-panel], :scope > [data-menu-category-empty-popup]'))if(panel.matches(':popover-open'))panel.hidePopover();delete category.dataset.categoryNative;delete category.dataset.categoryHoverNative;}categoryBindings.clear();categoryConnections.clear();}
"#.replace("__MENU_ROOT__",&id))
}
