const {id, model, initial, readonly, square, plain, englishPaper, digitWidth = 0, equationWidth = 0, lineTrace = false, fullSheet = false, blankColumns = 1, noteColumns = 0, lines, allowMouse, english, sentenceWidth, tableTrace = null} = config;
const container = document.getElementById(id);
if (!container) return;
if (model && !square && !plain) {
    try {
        await document.fonts.load(englishPaper ? '144px "Patrick Hand"'
            : model.includes('ब') ? '91px "Noto Sans Devanagari"' : '91px "Kalam"', model);
    } catch (error) {
        console.warn("Could not load the handwriting font", error);
    }
    if (!container.isConnected) return;
}

const traceColumns = englishPaper ? 3 : 6;
const rowHeight = 216;
const englishRowGap = englishPaper ? rowHeight * 1.6 / 16 : 0;
const rowPitch = rowHeight + englishRowGap;
const wideModelWidth = englishPaper ? 1958 : 1305;
const glyphPadding = 28;
const englishFont = '144px "Patrick Hand", Arial, sans-serif';
// A bundled handwriting model preserves the small loops in झ and द; device
// text fonts can fill those knots completely before centre-line extraction.
// Use a standard upright ब and let one font shape its whole practice text,
// including matras and conjuncts. Keep Kalam for the other handwriting models.
const hindiFont = model.includes("ब")
    ? '"Noto Sans Devanagari", Mangal, sans-serif'
    : '"Kalam", "Noto Sans Devanagari", Mangal, sans-serif';
const text = (hindi, translation) => english ? translation : hindi;
// Existing ink in a practice grid uses coordinates local to its original cells.
const hasGridInk = initial.length === 2 * traceColumns && initial.some(cell => cell.length);
const sentence = model && !square && !plain && !lineTrace && !hasGridInk ? sentenceRows() : [];
if (sentence.length) {
    container.classList.add("ink-sentence");
    container.style.setProperty("--sentence-width", sentenceWidth + "px");
    container.style.setProperty("--sentence-lines", sentence.length);
    container.querySelector(".instructions").textContent = text(
        "ऊपर बिंदियों पर लिखो। नीचे खाली पंक्तियों में खुद लिखो।",
        "Trace the dots above. Then write on your own on the blank lines below."
    );
}
const symbols = Array.from(model);
const symbolWidth = equationWidth || digitWidth;
const columns = tableTrace ? 2 * equationWidth : model ? (equationWidth || (digitWidth ? 2 * digitWidth : lineTrace || sentence.length ? 1 : traceColumns)) : blankColumns;
const count = tableTrace ? columns * 10 : fullSheet ? 1 : noteColumns + columns * (model ? 2 : lines);
const modelCell = model && !symbolWidth && !lineTrace && !sentence.length;
const grid = container.querySelector(".ink-grid");
const notes = container.querySelector(".regroup-grid");
// Persist legacy paper coordinates so existing drafts and dated samples retain
// their alignment, without rewriting every storage format that embeds Ink.
const paperRows = englishPaper ? (fullSheet ? lines : sentence.length || 1) : 1;
const oldPitch = fullSheet ? 20 : 16;
const newPitch = 17.6;
const oldExtent = 16 * paperRows + (fullSheet ? 4 * (paperRows - 1) : 0);
const newExtent = 16 * paperRows + 1.6 * (paperRows - 1);
function paperY(y, fromPitch, toPitch, fromExtent, toExtent) {
    if (!englishPaper || paperRows < 2) return y;
    const mm = y * fromExtent;
    const row = Math.max(0, Math.min(paperRows - 1, Math.floor((mm - 2) / fromPitch)));
    const local = mm - (2 + row * fromPitch);
    // Preserve the 12 mm between the four rules; scale only the blank gap.
    const mapped = local <= 12 || row === paperRows - 1 ? local
        : 12 + (local - 12) * (toPitch - 12) / (fromPitch - 12);
    return (2 + row * toPitch + mapped) / toExtent;
}
function mapInk(value, reverse = false) {
    if (!englishPaper || paperRows < 2) return value;
    return value.map(cell => cell.map(stroke => stroke.map(([x,y]) => [x,
        reverse ? paperY(y, newPitch, oldPitch, newExtent, oldExtent)
            : paperY(y, oldPitch, newPitch, oldExtent, newExtent)])));
}
const ink = mapInk(Array.from({length: count}, (_, i) => initial[i] ?? []));
const savedInk = () => mapInk(ink, true);
const cells = [];
const message = container.querySelector("[role=status]");
let active = null;
let tool = 'pen';

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

const editable = [];
function eraserPoint(event, canvas) {
 const rect=canvas.getBoundingClientRect();
 return [(event.clientX-rect.left)/rect.width,(event.clientY-rect.top)/rect.height];
}
function eraseAt(index, to) {
 const source=cells[index].getBoundingClientRect();
 for(const target of editable) {
  const rect=cells[target].getBoundingClientRect();
  const local=p=>[(source.left+p[0]*source.width-rect.left)/rect.width,(source.top+p[1]*source.height-rect.top)/rect.height];
  const from=local(active.last),end=local(to);
  let changed=false;
  for(let i=ink[target].length-1;i>=0;i--) {
   if(strokeHit(ink[target][i],from,end,rect)) {
    active.removed.push({index:target,position:i,stroke:ink[target].splice(i,1)[0]});changed=true;
   }
  }
  if(changed)draw(target);
 }
 active.last=to;
}

let latest = -1;
let submitted = false;
let frame = 0;
let paused = false;
let checkpoint = 0;
let finish;
const listeners = new AbortController();
const lifetime = readonly ? null : new Promise(resolve => { finish = resolve; });

function scheduleDraw(index) {
    if (frame || paused || document.hidden) return;
    frame = requestAnimationFrame(() => {
        frame = 0;
        drawActive(index);
    });
}
const history = [];

function useHindiFont(ctx) {
    ctx.font = '91px ' + hindiFont;
    const ascent = ctx.measureText("क").actualBoundingBoxAscent;
    if (ascent > 0) ctx.font = (91 * 71 / ascent) + 'px ' + hindiFont;
}

function sentenceRows() {
    const measure = document.createElement("canvas").getContext("2d");
    if (englishPaper) measure.font = englishFont;
    else useHindiFont(measure);
    const paragraphs = model.trim().split(/\r?\n/).map(line => line.trim()).filter(Boolean);
    const cellWidth = englishPaper ? 540 : 240;
    if (!/[\r\n]/.test(model.trim()) && (!/\s/.test(model.trim()) ||
        measure.measureText(model).width <= cellWidth - glyphPadding)) return [];

    const rows = [];
    for (const paragraph of paragraphs) {
        let row = "";
        for (const word of paragraph.split(/\s+/)) {
            const next = row ? row + " " + word : word;
            if (row && measure.measureText(next).width > wideModelWidth - glyphPadding) {
                rows.push(row);
                row = word;
            } else {
                row = next;
            }
        }
        rows.push(row);
    }
    return rows;
}

function paper(index) {
    const canvas = document.createElement("canvas");
    canvas.width = plain ? 1200 : englishPaper ? (model ? 540 : 1620) : square ? 216 : model ? 240 : 720;
    canvas.height = plain ? 600 : rowHeight;
    if (lineTrace || sentence.length) canvas.width = wideModelWidth;
    if (sentence.length) canvas.height = rowHeight * sentence.length + englishRowGap * (sentence.length - 1);
    if (fullSheet) {
        const {width, height} = grid.getBoundingClientRect();
        canvas.width = Math.max(1, Math.min(1800, Math.round(width * 2)));
        canvas.height = Math.max(1, Math.min(1800, Math.round(height * 2)));
    }
    canvas.dataset.cell = index;
    if (index < noteColumns) {
        const label = noteColumns === 1
            ? text("आगे जाने वाली दहाई", "Carried ten")
            : index === 0 ? text("नई दहाई", "New tens") : text("नई इकाइयाँ", "New ones");
        canvas.setAttribute("aria-label", label);
        notes.append(canvas);
    } else {
        if (tableTrace && index % columns === equationWidth) canvas.classList.add("table-copy-start");
        if (tableTrace && index % columns < equationWidth) canvas.dataset.model = characterAt(index);
        canvas.setAttribute("aria-label", index === 0 && modelCell
            ? text("नमूना", "Example")
            : text("लिखने की जगह", "Writing space"));
        grid.append(canvas);
    }
    return canvas;
}

function glyph(ctx, text, kind, width = ctx.canvas.width) {
    ctx.save();
    ctx.textBaseline = "alphabetic";
    ctx.textAlign = "center";
    ctx.fillStyle = kind === "model" ? "#223c37" : "#444";
    if (square) {
        ctx.font = "144px system-ui, sans-serif";
        ctx.textBaseline = "middle";
        if (text.includes("4")) drawHandwrittenFour(ctx, text);
        else ctx.fillText(text, 108, 112, 180);
        ctx.restore();
        return;
    }
    if (englishPaper) {
        drawEnglishModel(ctx, text);
        ctx.restore();
        return;
    }
    useHindiFont(ctx);
    ctx.textAlign = sentence.length ? "left" : "center";
    ctx.fillText(text, sentence.length ? glyphPadding / 2 : width / 2, 143, width - glyphPadding);
    ctx.restore();
}

// Keep the open-top school handwriting form of 4 in both the visible
// model and the raster used to generate dots and assess tracing.
function drawHandwrittenFour(ctx, text) {
    const characters = Array.from(text);
    const widths = characters.map(character => ctx.measureText(character).width);
    const total = widths.reduce((sum, width) => sum + width, 0);
    ctx.translate(108, 0);
    ctx.scale(Math.min(1, 180 / total), 1);
    let left = -total / 2;
    for (const [index, character] of characters.entries()) {
        const width = widths[index];
        if (character === "4") {
            ctx.save();
            ctx.translate(left + width * 0.08, 56);
            ctx.scale(width * 0.84, 108);
            ctx.beginPath();
            ctx.moveTo(0.58, 0);
            ctx.lineTo(0.08, 0.66);
            ctx.lineTo(0.92, 0.66);
            ctx.moveTo(0.68, 0.28);
            ctx.lineTo(0.68, 1);
            ctx.restore();
            ctx.strokeStyle = ctx.fillStyle;
            ctx.lineWidth = 9;
            ctx.lineCap = ctx.lineJoin = "round";
            ctx.stroke();
        } else {
            ctx.fillText(character, left + width / 2, 112);
        }
        left += width;
    }
}

function drawEnglishModel(ctx, text) {
    const width = ctx.canvas.width;
    const height = rowHeight;
    const source = document.createElement("canvas");
    source.width = width;
    source.height = height;
    const modelContext = source.getContext("2d");
    modelContext.font = englishFont;
    modelContext.textAlign = sentence.length ? "left" : "center";
    modelContext.textBaseline = "alphabetic";
    modelContext.fillStyle = ctx.fillStyle;
    const baseline = 145;
    const cap = Math.round(modelContext.measureText("H").actualBoundingBoxAscent);
    const xHeight = Math.round(modelContext.measureText("x").actualBoundingBoxAscent);
    modelContext.fillText(text, sentence.length ? glyphPadding / 2 : width / 2, baseline, width - glyphPadding);

    // Align the cap line, x-height and baseline with the printed workbook guides.
    const input = [0, baseline - cap, baseline - xHeight, baseline, height];
    const guides = [0, height / 8, 3 * height / 8, 5 * height / 8, height];
    for (let index = 0; index + 1 < guides.length; index++) {
        ctx.drawImage(source, 0, input[index], width, input[index + 1] - input[index],
            0, guides[index], width, guides[index + 1] - guides[index]);
    }
}

function line(ctx, points, color, width) {
    if (!points.length) return;
    const {width: w, height: h} = ctx.canvas;
    ctx.beginPath();
    ctx.moveTo(points[0][0] * w, points[0][1] * h);
    for (const [x, y] of points.slice(1)) ctx.lineTo(x * w, y * h);
    if (points.length === 1) ctx.lineTo(points[0][0] * w + 0.01, points[0][1] * h);
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.lineCap = ctx.lineJoin = "round";
    ctx.stroke();
}

function raster(canvas, paint, resolution = 1) {
    const scratch = document.createElement("canvas");
    scratch.width = canvas.width * resolution;
    scratch.height = canvas.height * resolution;
    const ctx = scratch.getContext("2d", {willReadFrequently: true});
    ctx.scale(resolution, resolution);
    paint(ctx);
    const pixels = ctx.getImageData(0, 0, scratch.width, scratch.height).data;
    // Free the temporary canvas backing store; pixels own a separate buffer.
    scratch.width = scratch.height = 1;
    return pixels;
}

function draw(index) {
    const canvas = cells[index];
    const ctx = canvas.getContext("2d");
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (modelCell && index === 0) glyph(ctx, model, "model");
    if (isTraceCell(index)) {
        paintTraceGuide(ctx, guide(canvas, index));
    }
    for (const stroke of ink[index]) line(ctx, stroke, "#223c37", 5);
    if (active?.index === index && !active.removed) active.rendered = ink[index].at(-1).length;
}

function drawActive(index) {
    if (active?.index !== index || active.removed) return;
    const stroke = ink[index].at(-1);
    if (stroke.length <= active.rendered) return;
    line(cells[index].getContext("2d"), stroke.slice(Math.max(0, active.rendered - 1)), "#223c37", 5);
    active.rendered = stroke.length;
}

function isTraceCell(index) {
    if (tableTrace) return index % columns < equationWidth && !!characterAt(index);
    return !!model && index >= (modelCell ? 1 : 0) && index < columns;
}

function characterAt(index) {
    if (tableTrace) {
        const groups = Math.floor(index / columns) + 1;
        return [String(tableTrace), "×", String(groups), "=", String(tableTrace * groups)][index % equationWidth] || "";
    }
    return symbolWidth ? symbols[index % symbolWidth] : model;
}

const references = new Map();
const guides = new Map();
function guide(canvas, index) {
    const character = characterAt(index);
    if (!guides.has(character)) {
        // Preserve subpixel counters in handwritten Hindi, including when matras
        // select alternate glyphs. Reduce coordinates only after extracting paths.
        const resolution = !square && !englishPaper ? 3 : 1;
        const paths = traceGuide(template(canvas, index, resolution), canvas.width * resolution, canvas.height * resolution);
        guides.set(character, paths.map(path => path.map(([x,y]) => [x/resolution,y/resolution])));
        // Feedback uses the 1x mask. Never retain the much larger extraction mask.
        if (resolution > 1) references.delete(character + '/' + resolution);
    }
    return guides.get(character);
}
function template(canvas, index, resolution = 1) {
    const character = characterAt(index);
    const key = character + '/' + resolution;
    if (!references.has(key)) {
        references.set(key, raster(canvas, ctx => {
            if (!sentence.length) {
                glyph(ctx, character, "model", canvas.width);
                return;
            }
            for (const [index, row] of sentence.entries()) {
                ctx.save();
                ctx.translate(0, index * rowPitch);
                glyph(ctx, row, "model", canvas.width);
                ctx.restore();
            }
        }, resolution));
    }
    return references.get(key);
}

function feedback() {
    if (!isTraceCell(latest) || !ink[latest].length) return;
    const canvas = cells[latest];
    const mask = template(canvas, latest);
    const thinInk = raster(canvas, ctx => ink[latest].forEach(stroke => line(ctx, stroke, "#000", 5)));
    const wideInk = raster(canvas, ctx => ink[latest].forEach(stroke => line(ctx, stroke, "#000", 24)));
    const {width, height} = canvas;
    message.textContent = tracePasses(mask, thinInk, wideInk, width, height)
        ? text("बढ़िया, रेखा नमूने के पास है।", "Good tracing. Your strokes follow the example.")
        : text("बिंदियों को देखकर एक बार और लिख सकते हो।", "Look at the dots and try tracing again.");
}

function update() {
    dioxus.send({ink: savedInk(), done: false});
    message.textContent = "";
    feedback();
}

function point(event, canvas) {
    const rect = canvas.getBoundingClientRect();
    const clamp = value => Math.min(1, Math.max(0, value));
    return [clamp((event.clientX - rect.left) / rect.width), clamp((event.clientY - rect.top) / rect.height)];
}

function bind(canvas, index) {
    if (readonly || (modelCell && (index === 0 || index === columns))) return;
    editable.push(index);
    canvas.addEventListener("pointerdown", event => {
        const accepted = event.pointerType === "pen" || (allowMouse && event.pointerType === "mouse");
        if (submitted || paused || document.hidden || active || !accepted || event.button !== 0) return;
        event.preventDefault();
        canvas.setPointerCapture(event.pointerId);
        if(tool==='eraser') {
            active={id:event.pointerId,index,last:eraserPoint(event,canvas),removed:[]};
            eraseAt(index,active.last);return;
        }
        ink[index].push([point(event, canvas)]);
        active = {id: event.pointerId, index, rendered: 0};
        checkpoint = performance.now();
        scheduleDraw(index);
    }, {signal: listeners.signal});
    canvas.addEventListener("pointermove", event => {
        if (active?.id !== event.pointerId) return;
        if(active.removed) {
            event.preventDefault();
            const samples=event.getCoalescedEvents?.()??[];
            for(const sample of samples.length?samples:[event])eraseAt(index,eraserPoint(sample,canvas));
            if(performance.now()-checkpoint>=1000){dioxus.send({ink: savedInk(),done:false});checkpoint=performance.now();}
            return;
        }
        const stroke = ink[index].at(-1);
        const before = stroke.length;
        const events = event.getCoalescedEvents?.() ?? [];
        for (const sample of events.length ? events : [event]) {
            if (stroke.length < 4096) stroke.push(point(sample, canvas));
        }
        if (stroke.length === before) return;
        scheduleDraw(index);
        if (performance.now() - checkpoint >= 1000) {
            dioxus.send({ink: savedInk(), done: false});
            checkpoint = performance.now();
        }
    }, {signal: listeners.signal});
    function end(event) {
        if (active?.id !== event.pointerId) return;
        if(active.removed) {
            if(event.type==='pointerup')eraseAt(index,eraserPoint(event,canvas));
            finishActive();return;
        }
        if (event.type === "pointerup" && ink[index].at(-1).length < 4096) {
            ink[index].at(-1).push(point(event, canvas));
        }
        finishActive();
    }
    for (const type of ["pointerup", "pointercancel", "lostpointercapture"]) canvas.addEventListener(type, end, {signal: listeners.signal});
}

function finishActive() {
    if (!active) return;
    const {index,removed} = active;
    if (!paused && !document.hidden && container.isConnected) drawActive(index);
    active = null;
    cancelAnimationFrame(frame);
    frame = 0;
    latest = index;
    if(removed){if(removed.length)history.push({index,removed});}
    else history.push({index,stroke:ink[index].at(-1)});
    update();
}

for (let i = 0; i < count; i++) {
    cells.push(paper(i));
    for (const stroke of ink[i]) history.push({index:i,stroke});
    if (modelCell && i === columns) cells[i].classList.add("spacer");
    if (square && modelCell && i === 0) cells[i].hidden = true;
    draw(i);
    bind(cells[i], i);
}

function hasCompleteCopy(rowStart) {
    for (let start = rowStart; start < rowStart + columns; start += symbolWidth) {
        if (ink.slice(start, start + symbolWidth).every(cell => cell.length)) return true;
    }
    return false;
}

if (!readonly) {
    for(const mode of ['pen','eraser']) {
        container.querySelector('[data-ink-tool='+mode+']')?.addEventListener('click',()=>{
            finishActive();tool=mode;
            for(const value of ['pen','eraser'])container.querySelector('[data-ink-tool='+value+']')?.setAttribute('aria-pressed',String(value===tool));
        },{signal:listeners.signal});
    }
    window.addEventListener('blur', finishActive, {signal: listeners.signal});
    window.addEventListener('pagehide', finishActive, {signal: listeners.signal});
    document.addEventListener('click', finishActive, {capture: true, signal: listeners.signal});
    window.addEventListener('saksham:pause', () => {
        finishActive(); paused = true;
        cancelAnimationFrame(frame); frame = 0;
    }, {signal: listeners.signal});
    window.addEventListener('saksham:resume', () => { paused = false; }, {signal: listeners.signal});
    document.addEventListener('visibilitychange', () => {
        if (document.hidden) finishActive();
    }, {signal: listeners.signal});
    container.querySelector("[data-undo]").onclick = () => {
        if (active || submitted || !history.length) return;
        const action=history.pop();latest=action.index;
        if(action.removed)for(const item of [...action.removed].reverse())ink[item.index].splice(item.position,0,item.stroke);
        else {const position=ink[latest].indexOf(action.stroke);if(position>=0)ink[latest].splice(position,1);}
        if(action.removed)for(const index of new Set(action.removed.map(item=>item.index)))draw(index);
        else draw(latest);
        update();
    };
    container.querySelector("[data-clear]").onclick = () => {
        if (active || submitted) return;
        for (const cell of ink) cell.length = 0;
        history.length = 0;
        latest = -1;
        cells.forEach((_, i) => draw(i));
        update();
    };
    const done = container.querySelector("[data-done]");
    if (done) done.onclick = () => {
        if (active || submitted || done.disabled) return;
        if (noteColumns && !ink.slice(noteColumns).some(cell => cell.length)) {
            message.textContent = text("पहले रेखा के नीचे जवाब लिखो।", "Write the answer below the line first.");
            return;
        }
        if (!ink.some(cell => cell.length)) {
            message.textContent = text("पहले पेन से लिखो।", "Write with the pen first.");
            return;
        }
        if (englishPaper && !model && lines > 1 && ink.some(cell => !cell.length)) {
            message.textContent = text("हर पंक्ति पर लिखो।", "Write on each line before finishing.");
            return;
        }
        if (symbolWidth && (!hasCompleteCopy(0) || !hasCompleteCopy(columns))) {
            message.textContent = text("ऊपर और नीचे कम से कम एक बार पूरी संख्या लिखो।", "Write the whole number at least once on each row.");
            return;
        }
        if (((lineTrace || sentence.length) && (!ink[0].length || !ink[1].length)) ||
            (modelCell && (!ink.slice(1, columns).some(cell => cell.length) ||
                !ink.slice(columns + 1).some(cell => cell.length)))) {
            message.textContent = text("बिंदियों पर और खाली जगह में लिखो।", "Write on the dots and in the blank space.");
            return;
        }
        submitted = true;
        dioxus.send({ink: savedInk(), done: true});
        finish();
    };
}


if (lifetime) {
    // Dioxus keeps the message channel open while this promise is pending.
    // Observe removal instead of polling the DOM on every open writing pad.
    const observer = new MutationObserver(() => {
        if (!container.isConnected) { finishActive(); finish(); }
    });
    observer.observe(document.body, {childList: true, subtree: true});
    if (!container.isConnected) finish();
    await lifetime;
    observer.disconnect();
    listeners.abort();
    cancelAnimationFrame(frame);
    container.querySelector("[data-undo]").onclick = null;
    container.querySelector("[data-clear]").onclick = null;
    const done = container.querySelector("[data-done]");
    if (done) done.onclick = null;
}
