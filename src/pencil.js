const root = document.getElementById('pencil-workspace');
if (!root) return;
const canvas = root.querySelector('canvas');
const image = root.querySelector('.pencil-surface img');
const ctx = canvas.getContext('2d');
let strokes = config.initial.map(s => s.map(p => [...p]));
let active = null;
let previous = performance.now();
let dirty = false;
let frame = 0;
let rendered = 0;
let scale = 1;
const listeners = new AbortController();
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
 rendered = active === null ? 0 : strokes.at(-1).length;
}
function drawActive() {
 if(active===null)return;
 const stroke=strokes.at(-1);
 if(stroke.length<=rendered)return;
 paint(stroke,Math.max(0,rendered-1));
 rendered=stroke.length;
}
function scheduleDraw() {
 if(frame)return;
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
 cancelAnimationFrame(frame);frame=0;drawActive();active=null;save();
}
canvas.addEventListener('pointerdown',e=>{
 if(active!==null || (e.pointerType!=='pen' && !(config.allowMouse && e.pointerType==='mouse' && e.button===0))) return;
 e.preventDefault();active=e.pointerId;canvas.setPointerCapture(active);strokes.push([point(e)]);rendered=0;dirty=true;drawActive();
},{signal:listeners.signal});
canvas.addEventListener('pointermove',e=>{
 if(e.pointerId!==active)return;e.preventDefault();
 const samples=e.getCoalescedEvents?.()??[];
 for(const sample of samples.length?samples:[e])strokes.at(-1).push(point(sample));
 dirty=true;scheduleDraw();
 // Checkpoint long strokes, but avoid cloning/syncing the entire page per move.
 if(performance.now()-previous>=2000)save();
},{signal:listeners.signal});
for(const event of ['pointerup','pointercancel','lostpointercapture'])canvas.addEventListener(event,e=>{if(e.pointerId===active)finish();},{signal:listeners.signal});
root.querySelector('[data-pencil-action=undo]').addEventListener('click',()=>{finish();strokes.pop();dirty=true;draw();save();},{signal:listeners.signal});
root.querySelector('[data-pencil-action=clear]').addEventListener('click',()=>{finish();strokes=[];dirty=true;draw();save();},{signal:listeners.signal});
window.addEventListener('blur',finish,{signal:listeners.signal});
document.addEventListener('visibilitychange',finish,{signal:listeners.signal});
function layout() { fitPage(); resizeCanvas(); }
const resize=new ResizeObserver(layout);resize.observe(canvas);
if(config.fitPage) {
 resize.observe(root.closest('.basic-pattern-mode'));
 resize.observe(root.querySelector('.pencil-scroll'));
 window.addEventListener('resize',layout,{signal:listeners.signal});
}
image.addEventListener('load',layout,{signal:listeners.signal});layout();
let endLifetime;
const lifetime=new Promise(resolve=>{endLifetime=resolve;});
const observer=new MutationObserver(()=>{if(!root.isConnected)endLifetime();});
observer.observe(document.body,{childList:true,subtree:true});
if(!root.isConnected)endLifetime();
// Keep Dioxus's message channel open until this workspace is unmounted.
await lifetime;
finish();cancelAnimationFrame(frame);
listeners.abort();resize.disconnect();observer.disconnect();
