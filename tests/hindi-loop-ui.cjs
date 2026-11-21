const fs = require('node:fs/promises');
const assert = require('node:assert/strict');
const path = require('node:path');
const {chromium} = require(path.join(process.env.USERPROFILE, '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright'));
(async () => {
    const root = path.resolve(__dirname, '..');
    const script = await fs.readFile(path.join(root, 'src/trace_guide.js'), 'utf8');
    const browser = await chromium.launch({channel: 'msedge', headless: true});
    try {
        const page = await browser.newPage({viewport: {width: 1200, height: 1000}, deviceScaleFactor: 2});
        await page.setContent('<style>body{font:18px sans-serif}main{display:grid;grid-template-columns:repeat(6,190px);gap:6px}canvas{width:180px;height:150px;border:1px solid #ddd}h2{margin:5px}</style><h1>Hindi loop review: solid / dotted</h1><main></main>');
        const kalam = !process.argv.includes('--system-font');
        if (kalam) {
            const font = await fs.readFile(path.join(root, 'assets/Kalam-Regular.ttf'), 'base64');
            await page.addStyleTag({content: `@font-face {font-family: Kalam;src:url(data:font/ttf;base64,${font})}`});
            await page.evaluate(() => document.fonts.load('91px Kalam', 'झद'));
        }
        await page.addScriptTag({content: script});
        const stats = await page.evaluate(kalam => {
            const result = [];
            for (const letter of ['अ','आ','इ','ई','उ','ऊ','ऋ','ए','ऐ','ओ','औ','क','ख','ग','घ','ङ','च','छ','ज','झ','ञ','ट','ठ','ड','ढ','ण','त','थ','द','ध','न','प','फ','ब','भ','म','य','र','ल','व','श','ष','स','ह','क्ष','त्र','ज्ञ','ड़','ढ़']) {
                const section = document.createElement('section');
                section.innerHTML = '<h2>' + letter + '</h2>';
                const source = document.createElement('canvas'); source.width = 240; source.height = 216;
                const ctx = source.getContext('2d');
                const font = kalam ? 'Kalam' : '"Noto Sans Devanagari", Mangal, sans-serif';
                ctx.font = '91px ' + font;
                ctx.font = (91 * 71 / ctx.measureText('क').actualBoundingBoxAscent) + 'px ' + font;
                ctx.textAlign = 'center'; ctx.fillText(letter, 120, 143, 212);
                const mask = ctx.getImageData(0,0,240,216).data;
                const paths = traceGuide(mask, 240, 216);
                // Keep source and trace at the same size for side-by-side inspection.
                const canvas = document.createElement('canvas'); canvas.width = 480; canvas.height = 216;
                canvas.style.width = '180px'; canvas.style.height = '81px';
                section.append(canvas); document.querySelector('main').append(section);
                const paint = canvas.getContext('2d'); paint.drawImage(source,0,0);
                paint.save(); paint.translate(240,0); paintTraceGuide(paint, paths); paint.restore();
                result.push({letter, paths: paths.length, closed: paths.filter(p => p.length > 2 && p[0][0] === p.at(-1)[0] && p[0][1] === p.at(-1)[1]).length});
            }
            return result;
        }, kalam);
        if (kalam) {
            for (const letter of ['झ','द','ढ','ध','भ','म','श','इ','ई','ठ','थ','न','क्ष']) {
                assert.ok(stats.find(item => item.letter === letter).closed > 0, letter + ' retains its closed loop');
            }
            // Matras must not switch back to device text glyphs or lose the loop.
            const variants = await page.evaluate(() => {
                const result = [];
                for (const base of ['झ','द','ढ','ध','भ','म','श']) for (const matra of ['', 'ा','ि','ी','ु','ू','ृ','े','ै','ो','ौ','ं']) {
                    const canvas = document.createElement('canvas'); canvas.width=720; canvas.height=648;
                    const ctx = canvas.getContext('2d'); ctx.scale(3,3); ctx.font='91px Kalam';
                    ctx.font=(91*71/ctx.measureText('क').actualBoundingBoxAscent)+'px Kalam';
                    ctx.textAlign='center'; ctx.fillText(base+matra,120,143,212);
                    const paths=traceGuide(ctx.getImageData(0,0,720,648).data,720,648);
                    const links = new Map(); let edges=0;
                    for (const p of paths) {
                        if (p.length<2) continue;
                        const a=p[0].join(','), b=p.at(-1).join(',');
                        if (!links.has(a)) links.set(a,[]); if (!links.has(b)) links.set(b,[]);
                        links.get(a).push(b); links.get(b).push(a); edges++;
                    }
                    let components=0; const seen=new Set();
                    for (const key of links.keys()) if (!seen.has(key)) {
                        components++; const todo=[key]; seen.add(key);
                        while(todo.length) for(const next of links.get(todo.pop())) if(!seen.has(next)){seen.add(next);todo.push(next);}
                    }
                    result.push({letter:base+matra, closed:edges-links.size+components});
                }
                return result;
            });
            for (const item of variants) assert.ok(item.closed > 0, item.letter + ' retains its loop with matra');
            console.log('Verified loops in 13 base forms and ' + variants.length + ' matra forms.');
        }
        const output = path.join(root, 'target/trace-review'); await fs.mkdir(output, {recursive:true});
        await page.screenshot({path: path.join(output, kalam ? 'hindi-loop-kalam.png' : 'hindi-loop-review.png'), fullPage:true});
        if (kalam) {
            await page.evaluate(() => {
                for (const section of document.querySelectorAll('main section')) {
                    if (!['झ','द','ढ','ध','भ','म','श','इ','ई','थ','न','क्ष'].includes(section.querySelector('h2').textContent)) section.remove();
                }
            });
            await page.screenshot({path:path.join(output,'hindi-loops-corrected.png'),fullPage:true});
        }
        console.log(JSON.stringify(stats));
    } finally { await browser.close(); }
})().catch(error => {console.error(error); process.exitCode = 1;});
