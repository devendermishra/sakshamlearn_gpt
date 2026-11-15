const {id, model, initial, readonly, square, plain, englishPaper, digitWidth = 0, equationWidth = 0, lineTrace = false, fullSheet = false, blankColumns = 1, noteColumns = 0, lines, allowMouse, english, sentenceWidth} = config;
const container = document.getElementById(id);
if (!container) return;
if (englishPaper && model) {
    try {
        await document.fonts.load('144px "Patrick Hand"', model);
    } catch (error) {
        console.warn("Could not load the handwriting font", error);
    }
    if (!container.isConnected) return;
}

const traceColumns = englishPaper ? 3 : 6;
const rowHeight = 216;
const wideModelWidth = englishPaper ? 1958 : 1305;
const glyphPadding = 28;
const englishFont = '144px "Patrick Hand", Arial, sans-serif';
const hindiFont = '"Noto Sans Devanagari", Mangal, sans-serif';
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
const columns = model ? (equationWidth || (digitWidth ? 2 * digitWidth : lineTrace || sentence.length ? 1 : traceColumns)) : blankColumns;
const count = fullSheet ? 1 : noteColumns + columns * (model ? 2 : lines);
const modelCell = model && !symbolWidth && !lineTrace && !sentence.length;
const grid = container.querySelector(".ink-grid");
const notes = container.querySelector(".regroup-grid");
const ink = Array.from({length: count}, (_, i) => initial[i] ?? []);
const cells = [];
const message = container.querySelector("[role=status]");
let active = null;
let latest = -1;
let submitted = false;
let frame = 0;
let finish;
const listeners = new AbortController();
const lifetime = readonly ? null : new Promise(resolve => { finish = resolve; });

function scheduleDraw(index) {
    if (frame) return;
    frame = requestAnimationFrame(() => {
        frame = 0;
        draw(index);
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
    if (sentence.length) canvas.height = rowHeight * sentence.length;
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
        canvas.setAttribute("aria-label", index === 0 && modelCell
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
    ctx.fillStyle = kind === "model" ? "#223c37" : "#444";
    if (square) {
        ctx.font = "144px system-ui, sans-serif";
        ctx.textBaseline = "middle";
        ctx.fillText(text, 108, 112, 180);
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
    ctx.fillText(text, sentence.length ? glyphPadding / 2 : ctx.canvas.width / 2, 143, ctx.canvas.width - glyphPadding);
    ctx.restore();
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
    if (modelCell && index === 0) glyph(ctx, model, "model");
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
    return !!model && index >= (modelCell ? 1 : 0) && index < columns;
}

const references = new Map();
function template(canvas, index) {
    const character = symbolWidth ? symbols[index % symbolWidth] : model;
    if (!references.has(character)) {
        references.set(character, raster(canvas, ctx => {
            if (!sentence.length) {
                glyph(ctx, character, "model");
                return;
            }
            for (const [index, row] of sentence.entries()) {
                ctx.save();
                ctx.translate(0, index * rowHeight);
                glyph(ctx, row, "model");
                ctx.restore();
            }
        }));
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
    if (readonly || (modelCell && (index === 0 || index === columns))) return;
    canvas.addEventListener("pointerdown", event => {
        const accepted = event.pointerType === "pen" || (allowMouse && event.pointerType === "mouse");
        if (submitted || active || !accepted || event.button !== 0) return;
        event.preventDefault();
        canvas.setPointerCapture(event.pointerId);
        ink[index].push([point(event, canvas)]);
        active = {id: event.pointerId, index};
        scheduleDraw(index);
    }, {signal: listeners.signal});
    canvas.addEventListener("pointermove", event => {
        if (active?.id !== event.pointerId) return;
        const stroke = ink[index].at(-1);
        const events = event.getCoalescedEvents?.() ?? [];
        for (const sample of events.length ? events : [event]) {
            if (stroke.length < 4096) stroke.push(point(sample, canvas));
        }
        scheduleDraw(index);
    }, {signal: listeners.signal});
    function end(event) {
        if (active?.id !== event.pointerId) return;
        if (event.type === "pointerup" && ink[index].at(-1).length < 4096) {
            ink[index].at(-1).push(point(event, canvas));
        }
        active = null;
        cancelAnimationFrame(frame);
        frame = 0;
        latest = index;
        history.push(index);
        draw(index);
        update();
    }
    for (const type of ["pointerup", "pointercancel", "lostpointercapture"]) canvas.addEventListener(type, end, {signal: listeners.signal});
}

for (let i = 0; i < count; i++) {
    cells.push(paper(i));
    for (const _ of ink[i]) history.push(i);
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
        dioxus.send({ink, done: true});
        finish();
    };
}


if (lifetime) {
    // Dioxus keeps the message channel open while this promise is pending.
    // Observe removal instead of polling the DOM on every open writing pad.
    const observer = new MutationObserver(() => {
        if (!container.isConnected) finish();
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
