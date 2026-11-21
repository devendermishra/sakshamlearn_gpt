const assert = require('node:assert/strict');
const test = require('node:test');
const {traceGuide, paintTraceGuide} = require('../src/trace_guide.js');

function guide(paint, width = 100, height = 100) {
    const mask = new Uint8Array(width * height * 4);
    for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
        if (paint(x, y)) mask[(y * width + x)*4 + 3] = 255;
    }
    return traceGuide(mask, width, height);
}

test('a thick stroke becomes one centre route, directed downwards', () => {
    const paths = guide((x, y) => x >= 40 && x <= 50 && y >= 10 && y <= 90);
    assert.equal(paths.length, 1);
    assert.ok(paths[0].length > 60);
    assert.ok(paths[0].every(([x]) => Math.abs(x - 45) <= 1));
    assert.ok(paths[0][0][1] < paths[0].at(-1)[1]);
});

test('a loop remains one closed route with an open centre', () => {
    const paths = guide((x, y) => {
        const r = Math.hypot(x - 50, y - 50);
        return r >= 24 && r <= 32;
    });
    assert.equal(paths.length, 1);
    assert.deepEqual(paths[0][0], paths[0].at(-1));
    assert.ok(paths[0].every(([x, y]) => Math.hypot(x - 50, y - 50) >= 24));
});

test('joined loop and headline retain all branches without bridging the hole', () => {
    const paths = guide((x, y) => {
        const r = Math.hypot(x - 50, y - 50);
        return r >= 24 && r <= 32 || x >= 10 && x <= 90 && y >= 17 && y <= 23;
    });
    assert.ok(paths.length >= 3);
    assert.ok(paths.flat().some(([x]) => x < 15));
    assert.ok(paths.flat().some(([x]) => x > 85));
    assert.ok(paths.flat().every(([x, y]) => Math.hypot(x - 50, y - 50) >= 23));
});

test('blank masks and isolated dots render safely; arrows only appear on long paths', () => {
    assert.deepEqual(guide(() => false), []);
    const paths = guide((x, y) => x === 50 && y === 50);
    let dots = 0, strokes = 0;
    const ctx = {canvas: {width: 100, getBoundingClientRect: () => ({width: 100})},
        save() {}, restore() {}, beginPath() {}, arc() { dots++; }, fill() {},
        translate() {}, rotate() {}, moveTo() {}, lineTo() {}, stroke() { strokes++; }};
    paintTraceGuide(ctx, paths);
    assert.equal(dots, 1);
    assert.equal(strokes, 0);
    paintTraceGuide(ctx, [[[10, 50], [90, 50]]]);
    assert.equal(strokes, 1);
});

test('tiny closed loops have enough separate round dots to remain readable', () => {
    const dots = [];
    const ctx = {canvas: {width: 240, getBoundingClientRect: () => ({width: 114})},
        save() {}, restore() {}, beginPath() {}, arc(x, y, r) { dots.push({x,y,r}); }, fill() {}};
    paintTraceGuide(ctx, [[[10,10],[14,10],[14,14],[10,14],[10,10]]]);
    assert.equal(dots.length, 8);
    for (let i = 0; i < dots.length; i++) {
        const a = dots[i], b = dots[(i+1)%dots.length];
        assert.ok(Math.hypot(a.x-b.x, a.y-b.y) > a.r+b.r, 'dots must leave open gaps');
    }
});
