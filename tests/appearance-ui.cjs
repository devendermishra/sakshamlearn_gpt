const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {chromium} = require(path.join(process.env.USERPROFILE, '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright'));
const root = path.resolve(__dirname, '..');

async function run(profile, check) {
    const server = net.createServer();
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    const port = server.address().port;
    await new Promise(resolve => server.close(resolve));
    const endpoint = 'http://127.0.0.1:' + port;
    const webview = await fs.mkdtemp(path.join(profile, 'webview-'));
    const child = spawn(path.join(root, 'target/debug/saksham.exe'), [], {
        cwd: profile, windowsHide: true, stdio: 'ignore',
        env: {...process.env, LOCALAPPDATA: profile, WEBVIEW2_USER_DATA_FOLDER: webview,
            WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=' + port},
    });
    const closed = new Promise(resolve => child.once('exit', resolve));
    let browser;
    try {
        for (let i = 0; i < 160; i++) {
            if (child.exitCode !== null) throw Error('Preview exited early');
            if (await fetch(endpoint + '/json/version').then(r => r.ok, () => false)) break;
            await new Promise(resolve => setTimeout(resolve, 250));
        }
        browser = await chromium.connectOverCDP(endpoint);
        const context = browser.contexts()[0];
        const page = context.pages()[0] || await context.waitForEvent('page');
        page.setDefaultTimeout(15000);
        await check(page);
    } finally {
        await browser?.close().catch(() => {});
        if (child.exitCode === null) child.kill();
        await closed;
    }
}

(async () => {
    const profile = await fs.mkdtemp(path.join(root, 'target/appearance-ui-'));
    const output = path.join(root, 'target/appearance-review');
    await fs.mkdir(output, {recursive: true});
    await run(profile, async page => {
        const app = page.locator('.app');
        await page.locator('.home-cover').waitFor();
        assert.equal(await app.getAttribute('data-palette'), 'forest');
        assert.equal(await page.getByRole('button', {name: 'Basic', exact: true}).count(), 1);
        assert.equal(await page.getByRole('button', {name: 'ABA Learning', exact: true}).count(), 1);
        await page.locator('.appearance-toggle').click();
        await page.locator('#appearance-options[open]').waitFor();
        const palettes = page.locator('input[name="appearance-palette"]');
        const backgrounds = page.locator('input[name="appearance-background"]');
        assert.equal(await palettes.count(), 4);
        assert.equal(await backgrounds.count(), 4);
        const colors = [];
        for (let i = 0; i < 4; i++) {
            await palettes.nth(i).check();
            colors.push(await app.evaluate(node => getComputedStyle(node).getPropertyValue('--accent').trim()));
            await backgrounds.nth(i).check();
        }
        assert.equal(new Set(colors).size, 4);
        await backgrounds.nth(2).check();
        await page.locator('#appearance-options form button').click();
        await page.waitForFunction(() => !document.querySelector('#appearance-options').open);
        assert.equal(await app.getAttribute('data-palette'), 'plum');
        assert.equal(await app.getAttribute('data-cover'), 'animals');
        await page.screenshot({path: path.join(output, 'themes-restored.png'), fullPage: true});
    });
    const save = JSON.parse(await fs.readFile(path.join(profile, 'Saksham/progress.json'), 'utf8'));
    assert.deepEqual(save.appearance, {palette: 'plum', background: 'animals'});
    await run(profile, async page => {
        await page.locator('.app[data-palette="plum"][data-cover="animals"]').waitFor();
        await page.locator('.appearance-toggle').click();
        assert.equal(await page.locator('input[name="appearance-palette"]').nth(3).isChecked(), true);
        assert.equal(await page.locator('input[name="appearance-background"]').nth(2).isChecked(), true);
    });
    console.log('All four themes/backgrounds work, Basic/ABA remain available, and choices survive restart. Preview: ' + output);
})().catch(error => {console.error(error); process.exitCode = 1;});
