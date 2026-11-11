function tracePasses(mask, thinInk, wideInk, width, height) {
    let glyphPixels = 0, covered = 0, inkPixels = 0, onPath = 0;
    const at = (data, x, y) => data[(y * width + x) * 4 + 3] > 100;
    const offsets = [-8, -4, 0, 4, 8];

    for (let y = 0; y < height; y += 2) {
        for (let x = 0; x < width; x += 2) {
            if (at(mask, x, y)) {
                glyphPixels++;
                if (at(wideInk, x, y)) covered++;
            }
            if (at(thinInk, x, y)) {
                inkPixels++;
                if (offsets.some(dx => offsets.some(dy =>
                    x + dx >= 0 && x + dx < width && y + dy >= 0 && y + dy < height &&
                    at(mask, x + dx, y + dy)))) onPath++;
            }
        }
    }
    return glyphPixels > 0 && inkPixels > 0 && covered >= glyphPixels * 0.65 && onPath >= inkPixels * 0.70;
}

if (typeof module !== "undefined") module.exports = tracePasses;
