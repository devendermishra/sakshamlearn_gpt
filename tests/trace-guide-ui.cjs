const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const {chromium} = require(path.join(process.env.USERPROFILE, '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright'));

(async () => {
    const root = path.resolve(__dirname, '..');
    const scripts = await Promise.all(['trace.js', 'trace_guide.js', 'ink.js'].map(file => fs.readFile(path.join(root, 'src', file), 'utf8')));
    const browser = await chromium.launch({channel: 'msedge', headless: true});
    try {
        const page = await browser.newPage({viewport: {width: 1100, height: 1000}, deviceScaleFactor: 2});
        const font = await fs.readFile(path.join(root, 'assets', 'PatrickHand-Regular.ttf'), 'base64');
        const hindiFont = await fs.readFile(path.join(root, 'assets', 'Kalam-Regular.ttf'), 'base64');
        const hindiBaFont = await fs.readFile(path.join(root, 'assets', 'NotoSansDevanagari-Regular.ttf'), 'base64');
        await page.setContent(`<style>
            @font-face { font-family: 'Patrick Hand'; src: url(data:font/ttf;base64,${font}); }
            @font-face { font-family: 'Kalam'; src: url(data:font/ttf;base64,${hindiFont}); }
            @font-face { font-family: 'Noto Sans Devanagari'; src: url(data:font/ttf;base64,${hindiBaFont}); }
            body { font: 20px Arial; color: #223c37; padding: 24px; }
            section { margin-bottom: 24px; } h2 { margin-bottom: 8px; }
            .ink-grid { display: grid; grid-template-columns: repeat(6, 114px); width: fit-content; }
            canvas { width: 114px; height: 103px; border: 1px solid #c5d3d0;
                background: linear-gradient(to bottom, transparent 32.7%, #9ac6df 33%, transparent 33.8%, transparent 65.8%, #9ac6df 66.2%, transparent 67%); }
            .spacer { visibility: hidden; }
            .ink-sentence .ink-grid { grid-template-columns: 684px; }
            .ink-sentence canvas { width: 684px; }
        </style><h1>Dotted tracing · direction arrows</h1>
        <p>Follow the dots in the direction of the blue arrows.</p>`);
        for (const [index, model] of ['ब', 'बा', 'बि', 'बी', 'बु', 'बू', 'बृ', 'बे', 'बै', 'बो', 'बौ', 'बं', 'बच्चा', 'बस्ता', 'किताब', 'झ', 'द', 'ढ', 'ध', 'भ', 'म', 'श', 'दा', 'दि', 'झु', 'क', 'फ', 'अ', 'क्ष', '8', 'g', 'पानी', 'We play at home.'].entries()) {
            const id = 'sample-' + index;
            await page.evaluate(({id, model}) => {
                const section = document.createElement('section');
                section.id = id;
                section.innerHTML = '<h2></h2><p class="instructions"></p><div class="ink-grid"></div><p role="status"></p>';
                section.querySelector('h2').textContent = model;
                document.body.append(section);
            }, {id, model});
            const englishPaper = /[a-z]/i.test(model);
            const config = {id, model, initial: [], readonly: true, square: model === '8', plain: false,
                englishPaper, lines: 1, english: true, sentenceWidth: 684};
            const hasLoop = await page.evaluate(new Function('return (async () => { const config = ' + JSON.stringify(config) + ';\n' + scripts.join('\n') + '\nreturn guide(cells[1], 1).some(p => p.length > 2 && p[0][0] === p.at(-1)[0] && p[0][1] === p.at(-1)[1]); })()'));
            if (['झ','द','ढ','ध','भ','म','श','दा','दि','झु'].includes(model)) assert.ok(hasLoop, model + ' has a complete tracing loop in the app renderer');
            const sentence = model.includes(' ');
            const colors = await page.locator('#' + id + ' canvas').nth(sentence ? 0 : 1).evaluate(canvas => {
                const rgba = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
                let dots = 0, arrows = 0;
                for (let i = 0; i < rgba.length; i += 4) {
                    if (rgba[i+3] > 100 && rgba[i] === 67 && rgba[i+1] === 83) dots++;
                    if (rgba[i+3] > 100 && rgba[i] === 21 && rgba[i+1] === 112) arrows++;
                }
                return {dots, arrows};
            });
            assert.ok(colors.dots > 50, model + ' has visible dots');
            assert.ok(colors.arrows > 5, model + ' has direction arrows');
        }
        const output = path.join(root, 'target', 'trace-review');
        await fs.mkdir(output, {recursive: true});
        await page.screenshot({path: path.join(output, 'dotted-hindi.png'), fullPage: true});
        await page.addStyleTag({content: 'body{display:grid;grid-template-columns:repeat(3,240px);gap:20px}body>h1,body>p{grid-column:1/-1}section:nth-of-type(n+11){display:none}.ink-grid{grid-template-columns:repeat(2,114px)}canvas:nth-child(n+3){display:none}'});
        await page.screenshot({path:path.join(output,'hindi-loops-corrected-renderer.png'),fullPage:true});
        console.log('Hindi loops, conjuncts, words, English and square digits render with dots and arrows. Screenshot: ' + output);
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
