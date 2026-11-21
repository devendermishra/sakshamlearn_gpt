// Reduce the filled glyph to one connected centre line. Sampling the outline or
// stippling the fill produces two competing routes around Hindi loops.
function traceGuide(mask, width, height) {
    const pixels = new Uint8Array(width * height);
    for (let i = 0; i < pixels.length; i++) pixels[i] = mask[i * 4 + 3] > 100 ? 1 : 0;
    let changed = true;
    while (changed) {
        changed = false;
        for (let pass = 0; pass < 2; pass++) {
            const remove = [];
            for (let y = 1; y < height - 1; y++) {
                for (let x = 1; x < width - 1; x++) {
                    const i = y * width + x;
                    if (!pixels[i]) continue;
                    const p = [pixels[i-width], pixels[i-width+1], pixels[i+1], pixels[i+width+1],
                        pixels[i+width], pixels[i+width-1], pixels[i-1], pixels[i-width-1]];
                    const count = p.reduce((sum, value) => sum + value, 0);
                    const turns = p.reduce((sum, value, j) => sum + (!value && p[(j+1)%8] ? 1 : 0), 0);
                    if (count < 2 || count > 6 || turns !== 1) continue;
                    if (pass === 0 ? p[0]*p[2]*p[4] || p[2]*p[4]*p[6]
                        : p[0]*p[2]*p[6] || p[0]*p[4]*p[6]) continue;
                    remove.push(i);
                }
            }
            for (const i of remove) pixels[i] = 0;
            changed ||= remove.length > 0;
        }
    }
    const neighbors = i => {
        const x = i % width, y = Math.floor(i / width), result = [];
        for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) {
            if ((!dx && !dy) || x+dx < 0 || x+dx >= width || y+dy < 0 || y+dy >= height) continue;
            const next = i + dy * width + dx;
            // A diagonal must not shortcut a corner and introduce tiny false loops.
            if (pixels[next] && !(dx && dy && (pixels[i+dx] || pixels[i+dy*width]))) result.push(next);
        }
        return result;
    };
    const nodes = new Map();
    for (let i = 0; i < pixels.length; i++) if (pixels[i]) nodes.set(i, neighbors(i));
    const used = new Set(), paths = [];
    const edge = (a, b) => a < b ? a + ':' + b : b + ':' + a;
    const follow = (start, next) => {
        const path = [start];
        let previous = start;
        while (!used.has(edge(previous, next))) {
            used.add(edge(previous, next));
            path.push(next);
            const adjacent = nodes.get(next);
            if (adjacent.length !== 2) break;
            const after = adjacent.find(i => i !== previous);
            previous = next;
            next = after;
        }
        paths.push(path.map(i => [i % width, Math.floor(i / width)]));
    };
    for (const [i, adjacent] of nodes) {
        if (!adjacent.length) paths.push([[i % width, Math.floor(i / width)]]);
        if (adjacent.length !== 2) for (const next of adjacent) if (!used.has(edge(i, next))) follow(i, next);
    }
    for (const [i, adjacent] of nodes) for (const next of adjacent) if (!used.has(edge(i, next))) follow(i, next);

    // These are suggested tracing directions, not a font-derived stroke order.
    // Open paths run downwards (horizontal paths left to right); loops clockwise.
    for (const path of paths) {
        const first = path[0], last = path.at(-1);
        const closed = first[0] === last[0] && first[1] === last[1];
        const area = closed ? path.slice(1).reduce((sum, p, i) => sum + path[i][0]*p[1] - p[0]*path[i][1], 0) : 0;
        if (closed ? area < 0 : Math.abs(last[1]-first[1]) > Math.abs(last[0]-first[0])
            ? last[1] < first[1] : last[0] < first[0]) path.reverse();
    }
    return paths;
}

function paintTraceGuide(ctx, paths) {
    // Scale dots to the displayed paper, keeping the same visual size across
    // Hindi, English and digit canvases with different backing resolutions.
    const displayWidth = ctx.canvas.getBoundingClientRect().width || ctx.canvas.width;
    const unit = ctx.canvas.width / displayWidth;
    const spacing = 4.5 * unit, radius = 0.85 * unit;
    const arrows = [];
    ctx.save();
    ctx.fillStyle = '#43534e';
    const placed = new Set();
    function dot(x, y, dotRadius) {
        const key = Math.round(x * 2) + ':' + Math.round(y * 2);
        if (placed.has(key)) return;
        placed.add(key);
        ctx.beginPath(); ctx.arc(x, y, dotRadius, 0, Math.PI * 2); ctx.fill();
    }
    for (const path of paths) {
        const lengths = [0];
        for (let i = 1; i < path.length; i++) lengths.push(lengths.at(-1) + Math.hypot(path[i][0]-path[i-1][0], path[i][1]-path[i-1][1]));
        const total = lengths.at(-1);
        function point(distance) {
            let i = 1;
            while (i < lengths.length - 1 && lengths[i] < distance) i++;
            if (path.length === 1) return path[0];
            const fraction = (distance-lengths[i-1]) / (lengths[i]-lengths[i-1]);
            return path[i].map((value, axis) => path[i-1][axis] + fraction*(value-path[i-1][axis]));
        }
        const first = path[0], last = path.at(-1);
        const closed = path.length > 2 && first[0] === last[0] && first[1] === last[1];
        // A small loop must still read as a loop, rather than two or three
        // large dots. Keep at least eight samples and shrink dots to leave gaps.
        const steps = Math.max(closed ? 8 : 1, Math.round(total / spacing));
        const dotRadius = closed ? Math.min(radius, total / steps * .28) : radius;
        for (let i = 0; i < (closed ? steps : steps + 1); i++) dot(...point(total*i/steps), dotRadius);
        if (total < 20 * unit) continue;
        const middle = total * .48;
        const from = point(Math.max(0, middle - 3*unit)), to = point(Math.min(total, middle + 3*unit));
        if (Math.hypot(to[0]-from[0], to[1]-from[1]) < 3*unit) continue;
        // Keep arrows apart at joins and on small loops.
        if (arrows.some(p => Math.hypot(p[0]-to[0], p[1]-to[1]) < 18*unit)) continue;
        arrows.push(to);
        ctx.save();
        ctx.translate(...to); ctx.rotate(Math.atan2(to[1]-from[1], to[0]-from[0]));
        ctx.strokeStyle = '#1570a6'; ctx.lineWidth = 1.2*unit;
        ctx.lineCap = ctx.lineJoin = 'round';
        ctx.beginPath(); ctx.moveTo(-6*unit, 0); ctx.lineTo(0, 0);
        ctx.moveTo(-3*unit, -2*unit); ctx.lineTo(0, 0); ctx.lineTo(-3*unit, 2*unit); ctx.stroke();
        ctx.restore();
    }
    ctx.restore();
}

if (typeof module !== 'undefined') module.exports = {traceGuide, paintTraceGuide};
