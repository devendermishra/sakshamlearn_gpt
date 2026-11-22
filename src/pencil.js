const root = document.getElementById('pencil-workspace');
if (!root) return;
const canvas = root.querySelector('canvas');
const image = root.querySelector('.pencil-surface img');
const ctx = canvas.getContext('2d');
let strokes = config.initial.map(s => s.map(p => [...p]));
let active = null;
let tool = 'pen';
let gesture = null;
const history = strokes.map((stroke,index)=>({type:'draw',stroke,index}));

// Distance between an eraser sweep and each segment of a normalized stroke.
function strokeHit(stroke, from, to, rect) {
 const pixel = p => [p[0]*rect.width,p[1]*rect.height];
 const a=pixel(from), b=pixel(to);
 function distance(p,a,b) {
  const dx=b[0]-a[0],dy=b[1]-a[1],length=dx*dx+dy*dy;
  const t=length?Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/length)):0;
  return Math.hypot(p[0]-a[0]-t*dx,p[1]-a[1]-t*dy);
 }
 const cross=(p,q,r)=>(q[0]-p[0])*(r[1]-p[1])-(q[1]-p[1])*(r[0]-p[0]);
 for(let i=0;i<stroke.length;i++) {
  const c=pixel(stroke[i]),d=pixel(stroke[Math.max(0,i-1)]);
  const crosses=cross(a,b,c)*cross(a,b,d)<0 && cross(c,d,a)*cross(c,d,b)<0;
  if(crosses || Math.min(distance(a,c,d),distance(b,c,d),distance(c,a,b),distance(d,a,b))<=8)return true;
 }
 return false;
}

function eraseAt(to) {
 const rect=canvas.getBoundingClientRect();
 for(let i=strokes.length-1;i>=0;i--) {
  if(strokeHit(strokes[i],gesture.last,to,rect)) {
   const stroke=strokes.splice(i,1)[0];
   gesture.removed.push({stroke,index:i});dirty=true;
  }
 }
 gesture.last=to;draw();
}
let previous = performance.now();
let dirty = false;
let frame = 0;
let rendered = 0;
let scale = 1;
let paused = false;
const listeners = new AbortController();
for(const mode of ['pen','eraser']) {
 const button=root.querySelector('[data-pencil-action='+mode+']');
 button?.addEventListener('click',()=>{
  finish();tool=mode;
  for(const value of ['pen','eraser'])root.querySelector('[data-pencil-action='+value+']')?.setAttribute('aria-pressed',String(value===tool));
 },{signal:listeners.signal});
}


function save() {
 if (!dirty) return;
 dioxus.send(strokes);
 dirty = false;
 previous = performance.now();
}
function paint(stroke, start = 0) {
 if (!stroke.length) return;
 ctx.beginPath();ctx.moveTo(stroke[start][0]*canvas.width,stroke[start][1]*canvas.height);
 for (let i=start+1;i<stroke.length;i++)ctx.lineTo(stroke[i][0]*canvas.width,stroke[i][1]*canvas.height);
 if(stroke.length===1){ctx.arc(stroke[0][0]*canvas.width,stroke[0][1]*canvas.height,1.5*scale,0,Math.PI*2);ctx.fill();}else ctx.stroke();
}
function draw() {
 ctx.clearRect(0,0,canvas.width,canvas.height);
 for (const stroke of strokes) paint(stroke);
 rendered = active === null || gesture ? 0 : strokes.at(-1).length;
}
function drawActive() {
 if(active===null || gesture || paused || document.hidden)return;
 const stroke=strokes.at(-1);
 if(stroke.length<=rendered)return;
 paint(stroke,Math.max(0,rendered-1));
 rendered=stroke.length;
}
function scheduleDraw() {
 if(frame || paused || document.hidden)return;
 frame=requestAnimationFrame(()=>{frame=0;drawActive();});
}
function resizeCanvas() {
 const rect=canvas.getBoundingClientRect();
 const ratio=window.devicePixelRatio||1;
 // Bound GPU backing storage to 4M pixels (16 MiB), including on zoomed pages.
 scale=Math.min(ratio,Math.sqrt(4*1024*1024/Math.max(1,rect.width*rect.height)));
 const width=Math.max(1,Math.floor(rect.width*scale)),height=Math.max(1,Math.floor(rect.height*scale));
 if(canvas.width===width&&canvas.height===height)return;
 canvas.width=width;canvas.height=height;
 ctx.strokeStyle='#2256c7';ctx.fillStyle='#2256c7';ctx.lineWidth=3*scale;ctx.lineCap='round';ctx.lineJoin='round';
 draw();
}
// Show the full pattern while keeping the original canvas coordinate system.
function fitPage() {
 if (!config.fitPage || !image.naturalWidth || !image.naturalHeight) return;
 const section = root.closest('.basic-pattern-mode');
 const main = root.closest('main');
 const scroll = root.querySelector('.pencil-scroll');
 const frame = root.querySelector('.pencil-frame');
 const surface = root.querySelector('.pencil-surface');
 const [x,y,w,h] = config.crop;
 const padding = parseFloat(getComputedStyle(main).paddingBottom) || 0;
 const top = section.getBoundingClientRect().top + window.scrollY;
 section.style.height = Math.max(320, Math.floor(window.innerHeight - top - padding)) + 'px';
 const aspect = image.naturalWidth * w / (image.naturalHeight * h);
 const width = Math.floor(Math.min(scroll.clientWidth - 16, (scroll.clientHeight - 16) * aspect));
 frame.style.setProperty('--pencil-fit-width', Math.max(1,width) + 'px');
 frame.style.aspectRatio = String(aspect);
 surface.style.width = (100 / w) + '%';
 surface.style.left = (-100 * x / w) + '%';
 surface.style.top = (-100 * y / h) + '%';
}
function point(e) { const r=canvas.getBoundingClientRect(); return [Math.max(0,Math.min(1,(e.clientX-r.left)/r.width)),Math.max(0,Math.min(1,(e.clientY-r.top)/r.height))]; }
function finish() {
 if(active===null)return;
 cancelAnimationFrame(frame);frame=0;drawActive();
 if(gesture) {
  if(gesture.removed.length)history.push({type:'erase',removed:gesture.removed});
  gesture=null;
 } else history.push({type:'draw',stroke:strokes.at(-1),index:strokes.length-1});
 active=null;save();
}
canvas.addEventListener('pointerdown',e=>{
 if(paused || document.hidden || active!==null || (e.pointerType!=='pen' && !(config.allowMouse && e.pointerType==='mouse' && e.button===0))) return;
 e.preventDefault();active=e.pointerId;canvas.setPointerCapture(active);
 if(tool==='eraser'){gesture={last:point(e),removed:[]};eraseAt(point(e));return;}
 strokes.push([point(e)]);rendered=0;dirty=true;drawActive();
},{signal:listeners.signal});
canvas.addEventListener('pointermove',e=>{
 if(e.pointerId!==active)return;e.preventDefault();
 const samples=e.getCoalescedEvents?.()??[];
 if(gesture){
  for(const sample of samples.length?samples:[e])eraseAt(point(sample));
  if(performance.now()-previous>=2000)save();
  return;
 }
 for(const sample of samples.length?samples:[e])strokes.at(-1).push(point(sample));
 dirty=true;scheduleDraw();
 // Checkpoint long strokes, but avoid cloning/syncing the entire page per move.
 if(performance.now()-previous>=2000)save();
},{signal:listeners.signal});
for(const event of ['pointerup','pointercancel','lostpointercapture'])canvas.addEventListener(event,e=>{if(e.pointerId===active){if(gesture && event==='pointerup')eraseAt(point(e));finish();}},{signal:listeners.signal});
root.querySelector('[data-pencil-action=undo]').addEventListener('click',()=>{finish();const action=history.pop();if(!action)return;
 if(action.type==='erase')for(const item of [...action.removed].reverse())strokes.splice(item.index,0,item.stroke);
 else {const index=strokes.indexOf(action.stroke);if(index>=0)strokes.splice(index,1);}
 dirty=true;draw();save();},{signal:listeners.signal});
root.querySelector('[data-pencil-action=clear]').addEventListener('click',()=>{finish();strokes=[];history.length=0;dirty=true;draw();save();},{signal:listeners.signal});
window.addEventListener('blur',finish,{signal:listeners.signal});
window.addEventListener('pagehide',finish,{signal:listeners.signal});
// Flush before navigation disposes the Rust receiver.
document.addEventListener('click',finish,{capture:true,signal:listeners.signal});
window.addEventListener('saksham:pause',()=>{finish();paused=true;cancelAnimationFrame(frame);frame=0;},{signal:listeners.signal});
window.addEventListener('saksham:resume',()=>{paused=false;layout();draw();},{signal:listeners.signal});
document.addEventListener('visibilitychange',()=>{if(document.hidden)finish();else {layout();draw();}},{signal:listeners.signal});
function layout() { if(paused || document.hidden || !root.isConnected)return; fitPage(); resizeCanvas(); }
const resize=new ResizeObserver(layout);resize.observe(canvas);
if(config.fitPage) {
 resize.observe(root.closest('.basic-pattern-mode'));
 resize.observe(root.querySelector('.pencil-scroll'));
 window.addEventListener('resize',layout,{signal:listeners.signal});
}
image.addEventListener('load',layout,{signal:listeners.signal});layout();
let endLifetime;
const lifetime=new Promise(resolve=>{endLifetime=resolve;});
const observer=new MutationObserver(()=>{if(!root.isConnected){finish();endLifetime();}});
observer.observe(document.body,{childList:true,subtree:true});
if(!root.isConnected)endLifetime();
// Keep Dioxus's message channel open until this workspace is unmounted.
await lifetime;
finish();cancelAnimationFrame(frame);
listeners.abort();resize.disconnect();observer.disconnect();
