const root = document.getElementById('pencil-workspace');
if (!root) return;
const canvas = root.querySelector('canvas');
const image = root.querySelector('.pencil-surface img');
const ctx = canvas.getContext('2d');
let strokes = config.initial.map(s => s.map(p => [...p]));
let active = null;
let previous = 0;
const listeners = new AbortController();
function save() { dioxus.send(strokes); }
function draw() {
 const rect = canvas.getBoundingClientRect();
 const ratio = window.devicePixelRatio || 1;
 canvas.width = Math.max(1,Math.round(rect.width*ratio)); canvas.height = Math.max(1,Math.round(rect.height*ratio));
 ctx.strokeStyle='#2256c7'; ctx.fillStyle='#2256c7'; ctx.lineWidth=3*ratio; ctx.lineCap='round';ctx.lineJoin='round';
 for (const stroke of strokes) {
  if(!stroke.length) continue;
  ctx.beginPath();ctx.moveTo(stroke[0][0]*canvas.width,stroke[0][1]*canvas.height);
  for(const [x,y] of stroke) ctx.lineTo(x*canvas.width,y*canvas.height);
  if(stroke.length===1) {ctx.arc(stroke[0][0]*canvas.width,stroke[0][1]*canvas.height,1.5*ratio,0,Math.PI*2);ctx.fill();} else ctx.stroke();
 }
}
function point(e) { const r=canvas.getBoundingClientRect(); return [Math.max(0,Math.min(1,(e.clientX-r.left)/r.width)),Math.max(0,Math.min(1,(e.clientY-r.top)/r.height))]; }
function finish() { if(active===null)return;active=null;save(); }
canvas.addEventListener('pointerdown',e=>{
 if(active!==null || (e.pointerType!=='pen' && !(config.allowMouse && e.pointerType==='mouse' && e.button===0))) return;
 e.preventDefault();active=e.pointerId;canvas.setPointerCapture(active);strokes.push([point(e)]);draw();save();
},{signal:listeners.signal});
canvas.addEventListener('pointermove',e=>{
 if(e.pointerId!==active)return;e.preventDefault();
 for(const sample of (e.getCoalescedEvents?.() || [e]))strokes[strokes.length-1].push(point(sample));
 draw();if(performance.now()-previous>150){previous=performance.now();save();}
},{signal:listeners.signal});
for(const event of ['pointerup','pointercancel','lostpointercapture'])canvas.addEventListener(event,e=>{if(e.pointerId===active)finish();},{signal:listeners.signal});
root.querySelector('[data-pencil-action=undo]').addEventListener('click',()=>{finish();strokes.pop();draw();save();},{signal:listeners.signal});
root.querySelector('[data-pencil-action=clear]').addEventListener('click',()=>{finish();strokes=[];draw();save();},{signal:listeners.signal});
window.addEventListener('blur',finish,{signal:listeners.signal});
document.addEventListener('visibilitychange',finish,{signal:listeners.signal});
const resize=new ResizeObserver(draw);resize.observe(canvas);
image.addEventListener('load',draw,{signal:listeners.signal});draw();
let endLifetime;
const lifetime=new Promise(resolve=>{endLifetime=resolve;});
const observer=new MutationObserver(()=>{if(!root.isConnected)endLifetime();});
observer.observe(document.body,{childList:true,subtree:true});
if(!root.isConnected)endLifetime();
// Keep Dioxus's message channel open until this workspace is unmounted.
await lifetime;
listeners.abort();resize.disconnect();observer.disconnect();
