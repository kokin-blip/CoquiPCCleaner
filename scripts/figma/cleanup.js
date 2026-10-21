// Phase 2.1 structural handoff. Run against the existing Coqui optimization page.
const page=await figma.getNodeByIdAsync('155:87');await figma.setCurrentPageAsync(page);
await Promise.all(['Regular','Medium','Semi Bold','Bold'].map(style=>figma.loadFontAsync({family:'Inter',style})));
const source=await figma.getNodeByIdAsync('161:327');
const collections=await figma.variables.getLocalVariableCollectionsAsync();const theme=collections.find(c=>c.name==='Coqui / Appearance');
const vars=(await figma.variables.getLocalVariablesAsync()).filter(v=>v.variableCollectionId===theme.id);const styles=await figma.getLocalTextStylesAsync();
const paint=name=>figma.variables.setBoundVariableForPaint({type:'SOLID',color:{r:0,g:0,b:0}},'color',vars.find(v=>v.name===name));
const buttonSet=await figma.getNodeByIdAsync('156:62');const navSet=await figma.getNodeByIdAsync('156:87');const createdNodeIds=[];
function text(parent,copy,style='Body',width=1100){const t=figma.createText();t.characters=copy;t.textStyleId=styles.find(s=>s.name==='Coqui/'+style).id;t.fills=[paint(style==='Caption'?'secondary':'text')];parent.appendChild(t);t.textAutoResize='HEIGHT';t.resize(width,t.height);createdNodeIds.push(t.id);return t;}
function section(parent,name,width=1120){const n=figma.createAutoLayout('VERTICAL');n.name=name;n.fills=[paint('surface')];n.cornerRadius=12;n.paddingLeft=n.paddingRight=20;n.paddingTop=n.paddingBottom=20;n.itemSpacing=14;n.counterAxisSizingMode='FIXED';n.primaryAxisSizingMode='AUTO';n.resize(width,100);parent.appendChild(n);createdNodeIds.push(n.id);return n;}
function button(parent,label){const c=buttonSet.children.find(n=>n.type==='COMPONENT'&&n.name==='State=Default');const n=c.createInstance();parent.appendChild(n);for(const t of n.findAllWithCriteria({types:['TEXT']}))t.characters=label;createdNodeIds.push(n.id);return n;}
const specs=[
 {key:'selection',title:'Optimization / Cache files / Phase 2.1',body:'Choose what to change. Review each path before anything moves.',rows:['Review individual cache files','Only scanned regular files inside your Library/Caches. No automatic selection.','Illustrative path: ~/Library/Caches/Example/cache.dat · 4 KB allocated','Folders, links, cloud-only files, exclusions and project-managed locations are omitted.'],action:'Review selected files'},
 {key:'approval',title:'Optimization / Exact-path approval / Phase 2.1',body:'A review expires after ten minutes. Keep files leaves the selected data unchanged.',rows:['Move the selected file to Trash?','Exact path, allocated size and modification date appear in the review.','Close the related application. Cached data may need to download or regenerate.','Trash still occupies disk space. Coqui never empties it.','Acknowledge closed apps and reviewed consequences before approval.'],action:'Approve and move to Trash'},
 {key:'verification',title:'Optimization / Verification & recovery / Phase 2.1',body:'Illustrative result states. Actual app results come from the persisted engine journal.',rows:['Per-file outcomes and action history','Verified move: original path absent; matching file identity at the returned Trash location.','Changed targets are skipped. Errors and unverified moves remain visible.','Stop remaining moves preserves completed work. Interrupted actions are never automatically retried.','Show in Trash when a location is available. Finder may offer Put Back; recovery is not guaranteed.'],action:'Back to cache files'}
];
const frames=[];const links=[];
for(let i=0;i<specs.length;i++){
 const spec=specs[i];let root=page.children.find(n=>n.name===spec.title);
 if(root){frames.push({key:spec.key,id:root.id});continue;}
 root=source.clone();page.appendChild(root);root.name=spec.title;root.x=1800+i*1640;root.y=0;createdNodeIds.push(root.id);
 const content=root.findOne(n=>n.type==='FRAME'&&n.name==='Content');for(const n of [...content.children])n.remove();content.itemSpacing=18;
 const sidebar=root.findOne(n=>n.type==='FRAME'&&n.name==='Sidebar');for(const n of [...sidebar.children])if(n.type==='TEXT'&&n.characters==='Read-only by design')n.remove();
 const nav=navSet.children.find(n=>n.type==='COMPONENT'&&n.name==='State=Selected').createInstance();sidebar.appendChild(nav);for(const t of nav.findAllWithCriteria({types:['TEXT']}))t.characters='Optimization';createdNodeIds.push(nav.id);
 text(content,'Optimization','Title');text(content,spec.body);text(content,'Cache files                    Action history','Card');text(content,'Illustrative design handoff — paths and sizes below are examples, not device readings.','Caption');
 const panel=section(content,spec.key==='approval'?'Protected approval review':'Cleanup workflow');for(let j=0;j<spec.rows.length;j++)text(panel,spec.rows[j],j===0?'Card':'Body',1060);
 const action=button(content,spec.action);links.push({key:spec.key,id:action.id});if(spec.key==='approval'){const keep=button(content,'Keep files');links.push({key:'keep',id:keep.id});}
 frames.push({key:spec.key,id:root.id});
}
const ids=Object.fromEntries(frames.map(f=>[f.key,f.id]));
for(const link of links){const n=await figma.getNodeByIdAsync(link.id);const destination=link.key==='selection'?ids.approval:link.key==='approval'?ids.verification:ids.selection;await n.setReactionsAsync([{trigger:{type:'ON_CLICK'},actions:[{type:'NODE',destinationId:destination,navigation:'NAVIGATE',transition:null,preserveScrollPosition:false}]}]);}
page.flowStartingPoints=[...(page.flowStartingPoints||[]),{nodeId:ids.selection,name:'Phase 2.1 / cache review'}].filter((v,i,a)=>a.findIndex(x=>x.nodeId===v.nodeId)===i);
return{pageId:page.id,createdNodeIds,frames,links,originalGuidePreserved:source.id};
