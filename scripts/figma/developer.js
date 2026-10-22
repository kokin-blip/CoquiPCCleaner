// Structural Phase 2.2 handoff; original kit/pages and Phase 2.1 views are retained.
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
 {name:'Optimization / Developer artifacts / Phase 2.2',title:'Developer artifacts',rows:['Rust · target — indexed allocation: illustrative 4 KB · Git clean · Review required','Node.js · dist — indexed allocation: illustrative 4 KB · Git dirty · Protected','Filters: path / technology / assessment · 25 results per page','Check artifacts performs bounded read-only Git validation; cancel preserves partial observations.','Tracked entries, nested projects and unknown state remain protected. Clean or ignored does not mean disposable.'],action:'Inspect artifact'},
 {name:'Developer artifact / Git worktree review / Phase 2.2',title:'Developer artifact review',rows:['Exact path, project marker, indexed sizes and observation times','Repository root / Git directory / shared common directory','Current worktree: staged, modified, untracked and conflict counts','Registered worktrees: path, branch or detached HEAD, locked/prunable state and presence','Tracked artifact index entries / gitlinks / indexed nested project markers','Technology consequences and incomplete-coverage explanations. No project removal is authorized.'],action:'Done'}
];
for(let i=0;i<specs.length;i++){
 const spec=specs[i];let root=page.children.find(n=>n.name===spec.name);
 if(root){frames.push(root.id);continue;}
 root=source.clone();page.appendChild(root);root.name=spec.name;root.x=1800+i*1640;root.y=1300;created.push(root.id);
 const content=root.findOne(n=>n.type==='FRAME'&&n.name==='Content');for(const n of [...content.children])n.remove();content.itemSpacing=18;
 text(content,'Optimization','Title');text(content,'Cache files          Developer artifacts          Action history','Card');
 text(content,spec.title,'Section');text(content,'Illustrative structural handoff — sample paths, states and sizes are not device readings.','Caption');
 const panel=figma.createAutoLayout('VERTICAL');panel.name='Developer evidence';panel.fills=[paint('surface')];panel.cornerRadius=12;panel.paddingLeft=panel.paddingRight=20;panel.paddingTop=panel.paddingBottom=20;panel.itemSpacing=16;panel.counterAxisSizingMode='FIXED';panel.primaryAxisSizingMode='AUTO';panel.resize(1120,100);content.appendChild(panel);created.push(panel.id);
 for(const row of spec.rows)text(panel,row,'Body',1060);
 const action=buttonSource.createInstance();content.appendChild(action);for(const t of action.findAllWithCriteria({types:['TEXT']}))t.characters=spec.action;created.push(action.id);actions.push(action.id);frames.push(root.id);
}
if(actions.length===2)for(let i=0;i<2;i++){const action=await figma.getNodeByIdAsync(actions[i]);await action.setReactionsAsync([{trigger:{type:'ON_CLICK'},actions:[{type:'NODE',destinationId:frames[1-i],navigation:'NAVIGATE',transition:null,preserveScrollPosition:false}]}]);}
page.flowStartingPoints=[...(page.flowStartingPoints||[]),{nodeId:frames[0],name:'Phase 2.2 / developer review'}].filter((v,i,a)=>a.findIndex(x=>x.nodeId===v.nodeId)===i);
return {pageId:page.id,frames,actions,created,sourcePreserved:source.id,component:buttonSource.id};
