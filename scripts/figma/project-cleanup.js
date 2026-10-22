// Structural Phase 2.3 project-folder approval handoff. Original kit and earlier milestone views are retained.
const page=await figma.getNodeByIdAsync('155:87');await figma.setCurrentPageAsync(page);
const source=await figma.getNodeByIdAsync('165:64');
const texts=source.findAllWithCriteria({types:['TEXT']});
await Promise.all(texts.flatMap(t=>t.getStyledTextSegments(['fontName']).map(s=>figma.loadFontAsync(s.fontName))));
const styles=await figma.getLocalTextStylesAsync();
const theme=(await figma.variables.getLocalVariableCollectionsAsync()).find(c=>c.name==='Coqui / Appearance');
const vars=(await figma.variables.getLocalVariablesAsync()).filter(v=>v.variableCollectionId===theme.id);
const paint=name=>figma.variables.setBoundVariableForPaint({type:'SOLID',color:{r:0,g:0,b:0}},'color',vars.find(v=>v.name===name));
const buttonSource=await figma.getNodeByIdAsync('156:32');
const created=[];const frames=[];const actions=[];
function text(parent,copy,style='Body',width=1100){const n=figma.createText();n.characters=copy;n.textStyleId=styles.find(s=>s.name==='Coqui/'+style).id;n.fills=[paint(style==='Caption'?'secondary':'text')];parent.appendChild(n);n.textAutoResize='HEIGHT';n.resize(width,n.height);created.push(n.id);return n;}
const specs=[
 {name:'Project cleanup / Fresh folder inspection / Phase 2.3',title:'Review supported project output',rows:['Select a single output from Developer artifacts; no default selection.','Fresh metadata traversal: exact descendant paths, regular-file allocation and manifest identity.','Require a clean confirmed Git checkout, zero tracked output entries and a Git-ignored folder.','Reject links, nested projects, excluded/cloud entries and unsafe worktree relationships.','Supported: Rust target, Node build outputs, Dart/Flutter outputs and Python bytecode. Dependencies, virtual environments and unassociated Xcode data remain analysis only.'],action:'Review folder cleanup'},
 {name:'Project cleanup / Exact-folder approval / Phase 2.3',title:'Move the selected folder to Trash?',rows:['Illustrative exact path: /project/target · fresh Git and worktree evidence','Inspect the complete paginated descendant list. Allocation is not guaranteed recoverable space.','The whole folder moves. A ten-minute review binds its metadata, manifests and Git state.','Acknowledge stopped builds/watchers/editors and tools.','Acknowledge every ignored file contains no unique work and can be regenerated.','Keep folder makes no change. Approve performs fresh checks before the native Trash call.'],action:'Approve and move to Trash'},
 {name:'Project cleanup / Verification and recovery / Phase 2.3',title:'Inspect the actual folder result',rows:['Verified: original path absent, same directory identity and all reviewed descendant identities in Trash.','Changed or unavailable state: skip; native failure or incomplete verification: preserve explicit outcome.','Cancellation stops pending work; an in-flight native move still receives verification.','Action history retains reviewed scope, consequences, approval and per-item outcomes.','Reveal the returned Trash path. Finder may offer Put Back; recovery depends on the item remaining there.','No permanent deletion, automatic retry, build execution, empty Trash or claimed performance improvement.'],action:'Return to artifact review'}
];
for(let i=0;i<specs.length;i++){
 const spec=specs[i];let root=page.children.find(n=>n.name===spec.name);
 if(root){frames.push(root.id);continue;}
 root=source.clone();page.appendChild(root);root.name=spec.name;root.x=1800+i*1640;root.y=2600;created.push(root.id);
 const content=root.findOne(n=>n.type==='FRAME'&&n.name==='Content');for(const n of [...content.children])n.remove();content.itemSpacing=18;
 text(content,'Optimization','Title');text(content,'Cache files          Developer artifacts          Action history','Card');
 text(content,spec.title,'Section');text(content,'Illustrative structural handoff — sample paths, states and sizes are not device readings.','Caption');
 const panel=figma.createAutoLayout('VERTICAL');panel.name='Developer evidence';panel.fills=[paint('surface')];panel.cornerRadius=12;panel.paddingLeft=panel.paddingRight=20;panel.paddingTop=panel.paddingBottom=20;panel.itemSpacing=16;panel.counterAxisSizingMode='FIXED';panel.primaryAxisSizingMode='AUTO';panel.resize(1120,100);content.appendChild(panel);created.push(panel.id);
 for(const row of spec.rows)text(panel,row,'Body',1060);
 const action=buttonSource.createInstance();content.appendChild(action);action.resize(260,36);for(const t of action.findAllWithCriteria({types:['TEXT']}))t.characters=spec.action;created.push(action.id);actions.push(action.id);frames.push(root.id);
}
if(actions.length===3)for(let i=0;i<3;i++){const action=await figma.getNodeByIdAsync(actions[i]);await action.setReactionsAsync([{trigger:{type:'ON_CLICK'},actions:[{type:'NODE',destinationId:frames[(i+1)%3],navigation:'NAVIGATE',transition:null,preserveScrollPosition:false}]}]);}
page.flowStartingPoints=[...(page.flowStartingPoints||[]),{nodeId:frames[0],name:'Phase 2.3 / project cleanup'}].filter((v,i,a)=>a.findIndex(x=>x.nodeId===v.nodeId)===i);
for(const id of frames){const frame=await figma.getNodeByIdAsync(id);for(const node of frame.findAll())if(!created.includes(node.id))created.push(node.id);}
return {pageId:page.id,frames,actions,created,sourcePreserved:source.id,component:buttonSource.id};
