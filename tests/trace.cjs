const assert = require("node:assert/strict");
const test = require("node:test");
const tracePasses = require("../src/trace.js");

const width = 80;
const height = 80;
function raster(paint) {
    const pixels = new Uint8Array(width * height * 4);
    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            if (paint(x, y)) pixels[(y * width + x) * 4 + 3] = 255;
        }
    }
    return pixels;
}
const model = raster((x, y) => x >= 18 && x <= 22 && y >= 8 && y <= 70);
const partial = raster((x, y) => x >= 18 && x <= 22 && y >= 8 && y <= 20);
const scribble = raster((x, y) => (x >= 18 && x <= 22 || x >= 44) && y >= 8 && y <= 70);
const blank = raster(() => false);

test("a complete trace passes", () => {
    assert.equal(tracePasses(model, model, model, width, height), true);
});
test("partial coverage needs more practice", () => {
    assert.equal(tracePasses(model, partial, partial, width, height), false);
});
test("excess ink outside the model needs more practice", () => {
    assert.equal(tracePasses(model, scribble, scribble, width, height), false);
});
test("a missing model or blank attempt cannot pass", () => {
    assert.equal(tracePasses(model, blank, blank, width, height), false);
    assert.equal(tracePasses(blank, model, model, width, height), false);
});
