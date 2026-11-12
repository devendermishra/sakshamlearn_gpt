const {id, model, initial, readonly, square, digitWidth = 0, blankColumns = 1, noteColumns = 0, lines, allowMouse, english} = config;
const container = document.getElementById(id);
if (!container) return;

const columns = model ? (digitWidth ? 2 * digitWidth : 6) : blankColumns;
const count = noteColumns + columns * (model ? 2 : lines);
const grid = container.querySelector(".ink-grid");
const notes = container.querySelector(".regroup-grid");
const ink = Array.from({length: count}, (_, i) => initial[i] ?? []);
const cells = [];
const message = container.querySelector("[role=status]");
let active = null;
let latest = -1;
let submitted = false;
const history = [];
const text = (hindi, translation) => english ? translation : hindi;

function paper(index) {
    const canvas = document.createElement("canvas");
    canvas.width = square ? 216 : model ? 240 : 720;
    canvas.height = 216;
    canvas.dataset.cell = index;
    if (index < noteColumns) {
        const label = noteColumns === 1
            ? text("आगे जाने वाली दहाई", "Carried ten")
            : index === 0 ? text("नई दहाई", "New tens") : text("नई इकाइयाँ", "New ones");
        canvas.setAttribute("aria-label", label);
        notes.append(canvas);
    } else {
        canvas.setAttribute("aria-label", index === 0 && model && !digitWidth
            ? text("नमूना", "Example")
            : text("लिखने की जगह", "Writing space"));
        grid.append(canvas);
    }
    return canvas;
}

function glyph(ctx, text, kind) {
    ctx.save();
    ctx.textBaseline = "alphabetic";
    ctx.textAlign = "center";
    if (square) {
        ctx.font = "144px system-ui, sans-serif";
        ctx.fillStyle = kind === "model" ? "#223c37" : "#444";
        ctx.textBaseline = "middle";
        ctx.fillText(text, 108, 112, 180);
        ctx.restore();
        return;
    }
    ctx.font = '91px "Noto Sans Devanagari", Mangal, sans-serif';
    const ascent = ctx.measureText("क").actualBoundingBoxAscent;
    if (ascent > 0) ctx.font = (91 * 71 / ascent) + 'px "Noto Sans Devanagari", Mangal, sans-serif';
    ctx.fillStyle = kind === "model" ? "#223c37" : "#444";
    ctx.fillText(text, ctx.canvas.width / 2, 143, ctx.canvas.width - 28);
    ctx.restore();
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

function raster(canvas, paint) {
    const scratch = document.createElement("canvas");
    scratch.width = canvas.width;
    scratch.height = canvas.height;
    const ctx = scratch.getContext("2d", {willReadFrequently: true});
    paint(ctx);
    return ctx.getImageData(0, 0, scratch.width, scratch.height).data;
}

function draw(index) {
    const canvas = cells[index];
    const ctx = canvas.getContext("2d");
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (model && !digitWidth && index === 0) glyph(ctx, model, "model");
    if (isTraceCell(index)) {
        const mask = template(canvas, index);
        ctx.fillStyle = "#84918b";
        for (let y = 0; y < canvas.height; y += 5) {
            for (let x = 0; x < canvas.width; x += 5) {
                if (mask[(y * canvas.width + x) * 4 + 3] > 100) ctx.fillRect(x, y, 2.2, 2.2);
            }
        }
    }
    for (const stroke of ink[index]) line(ctx, stroke, "#223c37", 5);
}

function isTraceCell(index) {
    return !!model && index >= (digitWidth ? 0 : 1) && index < columns;
}

const references = new Map();
function template(canvas, index) {
    const character = digitWidth ? model[index % digitWidth] : model;
    if (!references.has(character)) {
        references.set(character, raster(canvas, ctx => glyph(ctx, character, "model")));
    }
    return references.get(character);
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
    dioxus.send({ink, done: false});
    message.textContent = "";
    feedback();
}

function point(event, canvas) {
    const rect = canvas.getBoundingClientRect();
    const clamp = value => Math.min(1, Math.max(0, value));
    return [clamp((event.clientX - rect.left) / rect.width), clamp((event.clientY - rect.top) / rect.height)];
}

function bind(canvas, index) {
    if (readonly || (model && !digitWidth && (index === 0 || index === columns))) return;
    canvas.addEventListener("pointerdown", event => {
        const accepted = event.pointerType === "pen" || (allowMouse && event.pointerType === "mouse");
        if (submitted || active || !accepted || event.button !== 0) return;
        event.preventDefault();
        canvas.setPointerCapture(event.pointerId);
        ink[index].push([point(event, canvas)]);
        active = {id: event.pointerId, index};
        draw(index);
    });
    canvas.addEventListener("pointermove", event => {
        if (active?.id !== event.pointerId) return;
        const stroke = ink[index].at(-1);
        const events = event.getCoalescedEvents?.() ?? [];
        for (const sample of events.length ? events : [event]) {
            if (stroke.length < 4096) stroke.push(point(sample, canvas));
        }
        draw(index);
    });
    function end(event) {
        if (active?.id !== event.pointerId) return;
        if (event.type === "pointerup" && ink[index].at(-1).length < 4096) {
            ink[index].at(-1).push(point(event, canvas));
        }
        active = null;
        latest = index;
        history.push(index);
        draw(index);
        update();
    }
    for (const type of ["pointerup", "pointercancel", "lostpointercapture"]) canvas.addEventListener(type, end);
}

for (let i = 0; i < count; i++) {
    cells.push(paper(i));
    for (const _ of ink[i]) history.push(i);
    if (model && !digitWidth && i === columns) cells[i].classList.add("spacer");
    if (square && model && !digitWidth && i === 0) cells[i].hidden = true;
    draw(i);
    bind(cells[i], i);
}

function hasCompleteCopy(rowStart) {
    for (let start = rowStart; start < rowStart + columns; start += digitWidth) {
        if (ink.slice(start, start + digitWidth).every(cell => cell.length)) return true;
    }
    return false;
}

if (!readonly) {
    container.querySelector("[data-undo]").onclick = () => {
        if (active || submitted || !history.length) return;
        latest = history.pop();
        ink[latest].pop();
        draw(latest);
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
        if (active || submitted) return;
        if (noteColumns && !ink.slice(noteColumns).some(cell => cell.length)) {
            message.textContent = text("पहले रेखा के नीचे जवाब लिखो।", "Write the answer below the line first.");
            return;
        }
        if (!ink.some(cell => cell.length)) {
            message.textContent = text("पहले पेन से लिखो।", "Write with the pen first.");
            return;
        }
        if (digitWidth && (!hasCompleteCopy(0) || !hasCompleteCopy(columns))) {
            message.textContent = text("ऊपर और नीचे कम से कम एक बार पूरी संख्या लिखो।", "Write the whole number at least once on each row.");
            return;
        }
        if (model && !digitWidth && (!ink.slice(1, columns).some(cell => cell.length) ||
            !ink.slice(columns + 1).some(cell => cell.length))) {
            message.textContent = text("बिंदियों पर और खाली जगह में लिखो।", "Write on the dots and in the blank space.");
            return;
        }
        submitted = true;
        dioxus.send({ink, done: true});
    };
}
