const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const net = require("node:net");
const { spawn } = require("node:child_process");
const { chromium } = require(path.join(process.env.USERPROFILE, ".cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright"));
const preview = path.join(__dirname, "target", "debug", "saksham.exe");

async function port() {
    const server = net.createServer();
    await new Promise((resolve, reject) => server.listen(0, "127.0.0.1", resolve).once("error", reject));
    const result = server.address().port;
    await new Promise(resolve => server.close(resolve));
    return result;
}

async function run(profile, exercise) {
    const endpoint = "http://127.0.0.1:" + await port();
    const browserProfile = await fs.mkdtemp(path.join(profile, "webview-"));
    const child = spawn(preview, [], {
        cwd: profile,
        env: {
            ...process.env,
            LOCALAPPDATA: profile,
            WEBVIEW2_USER_DATA_FOLDER: browserProfile,
            WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: "--remote-debugging-port=" + new URL(endpoint).port,
        },
        windowsHide: true,
        stdio: "ignore",
    });
    const closed = new Promise(resolve => child.once("exit", resolve));
    let browser;
    try {
        for (let index = 0; index < 80; index++) {
            if (child.exitCode !== null) throw Error("Preview exited before its window was ready");
            const ready = await fetch(endpoint + "/json/version").then(response => response.ok, () => false);
            if (ready) break;
            await new Promise(resolve => setTimeout(resolve, 250));
        }
        browser = await chromium.connectOverCDP(endpoint, {timeout: 10000});
        const context = browser.contexts()[0];
        const page = context.pages()[0] ?? await context.waitForEvent("page", {timeout: 10000});
        page.setDefaultTimeout(7000);
        await exercise(page);
    } finally {
        await browser?.close().catch(() => {});
        if (child.exitCode === null) child.kill();
        await closed;
    }
}

function button(page, name) {
    return page.getByRole("button", {name, exact: true});
}

async function fillQuestion(scope, prompt, first, second) {
    await scope.getByLabel("Question", {exact: true}).fill(prompt);
    await scope.getByLabel("Choice 1", {exact: true}).fill(first);
    await scope.getByLabel("Choice 2", {exact: true}).fill(second);
}


async function drawIn(page, cell, fraction) {
    const box = await cell.boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height * fraction);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * .6, box.y + box.height * fraction, {steps: 5});
    await page.mouse.up();
}

(async () => {
    const profile = await fs.mkdtemp(path.join(__dirname, "target", "notebook-ui-"));
    const save = path.join(profile, "Saksham", "progress.json");
    await run(profile, async page => {
        await button(page, "गणित").click();
        await button(page, "कक्षा 1").click();
        await button(page, "गिनो और लिखो · 1–3").click();
        await button(page, "1").click();
        await page.locator(".answer-feedback.correct").waitFor();
        assert.equal(await page.locator(".ink-square canvas").count(), 0);
        await button(page, "आगे").click();
        const mathGrid = page.locator(".ink-square .ink-grid");
        const mathCells = mathGrid.locator("canvas:visible");
        await mathCells.nth(9).waitFor();
        assert.equal(await mathCells.count(), 10);
        const first = await mathCells.first().boundingBox();
        const sixth = await mathCells.nth(5).boundingBox();
        assert.ok(Math.abs(first.width - 11 * 96 / 25.4) < .1);
        assert.ok(Math.abs(first.width - first.height) < .1);
        assert.ok(Math.abs(first.y - sixth.y) < .1);
        assert.ok(Math.abs(sixth.x - first.x - 5 * first.width) < .1);
        await mathGrid.screenshot({path: path.join(__dirname, "target", "notebook-math.png")});
        await drawIn(page, mathCells.first(), .5);
        await drawIn(page, mathCells.nth(5), .5);
        await button(page, "हो गया").click();
        await page.getByText("अभ्यास 3 / 6").waitFor();
        await button(page, "⌂ गणित").click();
        await button(page, "‹ विषय").click();
        await button(page, "हिंदी").click();
        await button(page, "कक्षा 1").click();
        await button(page, "स्वर").click();
        await button(page, "स्वर · अ आ इ").click();
        const hindi = page.locator(".ink-hindi-trace .ink-grid");
        await hindi.locator("canvas").nth(11).waitFor();
        const grid = await hindi.boundingBox();
        const trace = hindi.locator('[data-cell="1"]');
        const blank = hindi.locator('[data-cell="7"]');
        const traceBox = await trace.boundingBox();
        const blankBox = await blank.boundingBox();
        const band = 8 * 96 / 25.4;
        assert.ok(Math.abs(grid.height - 6 * band) < .1);
        assert.ok(Math.abs(traceBox.width - 30 * 96 / 25.4) < .1);
        assert.ok(Math.abs(blankBox.y - traceBox.y - 3 * band) < .1);
        assert.ok(Math.abs(traceBox.height - 3 * band) < .1);
        await hindi.screenshot({path: path.join(__dirname, "target", "notebook-hindi.png")});
        await drawIn(page, trace, .55);

        await drawIn(page, blank, .55);
        await button(page, "हो गया").click();
        await page.getByText("अभ्यास 2 / 3").waitFor();
        const saved = JSON.parse(await fs.readFile(save, "utf8"));
        for (const key of ["math/numbers-1-3/write-1", "hi/vowels-one/a"]) {
            const sample = saved.history[key][0];
            assert.equal(sample.ink.length, 12);
            assert.equal(sample.ink[1].length, 1);
            assert.equal(sample.ink[7].length, 1);
        }
        await button(page, "⌂ हिंदी").click();
        await button(page, "मात्राएँ").click();
        await button(page, "◌ी").click();
        await page.locator("#trace-ink canvas").nth(11).waitFor();
        await drawIn(page, page.locator('#trace-ink [data-cell="1"]'), .55);
        await drawIn(page, page.locator('#trace-ink [data-cell="7"]'), .55);
        await button(page, "हो गया").click();
        await page.getByRole("heading", {name: "मात्राएँ", exact: true}).waitFor();
        const afterJump = JSON.parse(await fs.readFile(save, "utf8"));
        assert.equal(afterJump.history["hi/barakhadi-1/ii"][0].model, "की");
        assert.deepEqual(afterJump.sessions, saved.sessions);
        console.log("Verified continuous 11 mm Mathematics squares, two separate Hindi writing rows, and writing submissions in both layouts.");
    });
    await run(profile, async page => {
        await button(page, "गणित").click();
        await button(page, "English").click();
        await button(page, "Class 1").click();
        await button(page, "Solve in writing").click();
        const keypad = page.getByRole("group", {name: "Numeric keypad"});
        const check = button(page, "Check");
        await button(page, "Fill the gaps in counting").click();
        await drawIn(page, page.locator("#solve-math-ink canvas").first(), .5);
        await button(page, "Enter answer").click();
        const answer = page.getByLabel("Enter the answer you wrote.");
        await answer.focus();
        await button(keypad, "9").click();
        await check.click();
        await page.locator(".answer-feedback.incorrect").waitFor();
        await button(keypad, "Clear").click();
        await button(keypad, "1").click();
        await button(keypad, "0").click();
        await check.click();
        await page.locator(".answer-feedback.correct").waitFor();
        await button(page, "Take a break").click();
        await page.getByRole("heading", {name: "Take a short break"}).waitFor();
        await button(page, "Continue practice").click();
        await page.getByText("Practice 1 / 5", {exact: true}).waitFor();
        await button(page, "⌂ Mathematics").click();
        await button(page, "Solve in writing").click();
        await button(page, "Write numbers in order").click();
        await drawIn(page, page.locator("#solve-math-ink canvas").first(), .5);
        await button(page, "Enter answer").click();
        const list = page.getByLabel("Enter the numbers you wrote, in order.");
        await list.focus();
        for (const [key, value] of [["2", "2"], ["Comma", "2,"], ["6", "2,6"], ["Comma", "2,6,"], ["9", "2,6,9"]]) {
            await button(keypad, key).click();
            await page.waitForFunction(expected => document.querySelector(".number-check input")?.value === expected, value);
        }
        await check.click();
        await button(page, "Next").click();
        await page.getByText("Practice 2 / 3", {exact: true}).waitFor();
        console.log("Verified single and multiple number entry, answer feedback, and cancellation on break.");

        await button(page, "⌂ Mathematics").click();
        await button(page, "‹ Subjects").click();
        await button(page, "English").click();
        await button(page, "Class 1").click();
        await button(page, "Trace the alphabet").click();
        await button(page, "b").click();
        await page.locator("#trace-ink canvas").nth(5).waitFor();
        assert.equal(await page.evaluate(() => [...document.fonts].some(
            font => font.family.includes("Patrick Hand") && font.status === "loaded")), true);
        const modelHasInk = await page.locator('#trace-ink [data-cell="0"]').evaluate(canvas =>
            canvas.getContext("2d").getImageData(0, 0, canvas.width, canvas.height).data.some(
                (value, index) => index % 4 === 3 && value > 0));
        assert.equal(modelHasInk, true);
        await button(page, "‹ Chapter").click();
        await button(page, "⌂ English").click();
        await button(page, "Read and understand").click();
        await button(page, "Two Little Hands").click();
        await button(page, "I’ve read it").click();
        await button(page, "My hands").click();
        await button(page, "Next").click();
        await button(page, "Cap").click();
        await button(page, "Next").click();
        await page.getByRole("heading", {name: "Recall", exact: true}).waitFor();
        await button(page, "Show choices").click();
        await button(page, "My feet").click();
        await button(page, "Next").click();
        await button(page, "Back to English").click();
        await button(page, "Daily recall").click();
        await page.getByText("Two Little Hands · 1 / 3", {exact: true}).waitFor();
        console.log("Verified offline English handwriting and a three-question recall round from a completed lesson.");
    });
    await run(profile, async page => {
        await page.getByRole("heading", {name: "अपना विषय चुनो"}).waitFor();
        await page.screenshot({path: path.join(__dirname, "target", "learner-home.png")});
        const work = path.join(profile, "Saksham", "learner-work");
        await fs.mkdir(work, {recursive: true});
        const legacyStroke = [[.1, .1], [.2, .2]];
        await fs.writeFile(path.join(work, "rough-en.json"), JSON.stringify([[legacyStroke]]));
        for (const [subject, rough, undo, back] of [
            ["hi", "रफ़ कॉपी", "एक रेखा हटाओ", "‹ विषय"],
            ["math", "Rough Sheet", "Undo last stroke", "‹ Subjects"],
            ["en", "Rough Sheet", "Undo last stroke", "‹ Subjects"],
        ]) {
            await page.locator('button[data-subject="' + subject + '"]').click();
            await button(page, rough).click();
            const canvas = page.locator("#rough-ink canvas");
            await canvas.waitFor();
            const paperClass = {hi: "ink-hindi", math: "ink-square", en: "ink-english"}[subject];
            assert.equal(await page.locator("#rough-ink").evaluate((node, name) => node.classList.contains(name), paperClass), true);
            assert.equal(await canvas.count(), 1);
            const {width, available, height, ruleRepeat, rowHeight, gap, ruling} = await page.locator("#rough-ink .ink-grid").evaluate(grid => {
                const style = getComputedStyle(grid);
                return {
                    width: grid.getBoundingClientRect().width,
                    available: grid.parentElement.clientWidth,
                    height: grid.getBoundingClientRect().height,
                    ruleRepeat: Number.parseFloat(style.backgroundSize.split(" ")[1]),
                    rowHeight: Number.parseFloat(style.getPropertyValue("--height")),
                    gap: Number.parseFloat(style.getPropertyValue("--english-gap")),
                    ruling: style.backgroundImage,
                };
            });
            assert.ok(width > 300 && height > 300);
            assert.ok(Math.abs(available - width) < (subject === "math" ? rowHeight + 1 : 1));
            if (subject === "en") {
                assert.ok(Math.abs(gap - rowHeight / 4) < .1);
                assert.ok(Math.abs(ruleRepeat - (rowHeight + gap)) < .1);
                assert.ok(Math.abs(height - (12 * rowHeight + 11 * gap)) < .1);
            } else {
                assert.equal(gap, 0);
                const rows = subject === "hi" ? 24 : 18;
                assert.ok(Math.abs(height - rows * rowHeight) < .1);
                if (subject === "hi") {
                    assert.ok(Math.abs(rowHeight - 8 * 96 / 25.4) < .1);
                    assert.ok(Math.abs(ruleRepeat - rowHeight) < .1);
                    assert.equal((ruling.match(/rgb\(/g) ?? []).length, 1);
                    await page.locator("#rough-ink .ink-grid").screenshot({path: path.join(__dirname, "target", "rough-hindi.png")});
                }
            }
            const box = await canvas.boundingBox();
            await page.mouse.move(box.x + box.width * .08, box.y + box.height * .18);
            await page.mouse.down();
            await page.mouse.move(box.x + box.width * .74, box.y + box.height * .45, {steps: 14});
            await page.mouse.up();
            await drawIn(page, canvas, .24);
            await button(page, undo).click();
            await page.getByRole("banner").getByRole("button").first().click();
            await button(page, back).click();
        }
        for (const subject of ["hi", "math", "en"]) {
            const notebook = JSON.parse(await fs.readFile(path.join(work, `rough-${subject}.json`), "utf8"));
            assert.equal(notebook[0].length, subject === "en" ? 2 : 1);
            if (subject === "en") {
                const oldStroke = notebook[0][0];
                assert.ok(Math.abs(oldStroke[0][0] - legacyStroke[0][0]) < 1e-6);
                assert.ok(Math.abs(oldStroke[1][1] - legacyStroke[1][1]) < 1e-6);
            }
            assert.ok(notebook[0].at(-1).at(-1)[0] > .7, "the pen can cross rules and squares");

        }
        await button(page, "English").click();
        await button(page, "Class 1").click();
        await button(page, "Worksheets").click();
        await button(page, "Capital & Lowercase Matching").click();
        await page.getByText("1 of 22", {exact: true}).waitFor();
        await button(page, "b").click();
        await page.getByText(/Try again/).waitFor();
        await button(page, "a").click();
        await button(page, "Take a break").click();
        await page.getByRole("heading", {name: "Take a short break"}).waitFor();
        await button(page, "Continue").click();
        await page.getByText("✓ That’s right.").waitFor();
        await button(page, "Continue").click();
        await page.getByText("2 of 22", {exact: true}).waitFor();
        await page.screenshot({path: path.join(__dirname, "target", "learner-worksheet.png")});
    });
    await run(profile, async page => {
        await button(page, "English").click();
        await button(page, "Class 2").click();
        await button(page, "Rough Sheet").click();
        const canvas = page.locator("#rough-ink canvas");
        await canvas.waitFor();
        const hasInk = await canvas.evaluate(node => {
            const context = node.getContext("2d");
            return context.getImageData(0, 0, node.width, node.height).data.some(
                (value, index) => index % 4 === 3 && value > 0);
        });
        assert.equal(hasInk, true);
        await button(page, "Clear writing").click();
        await button(page, "⌂ English").click();
        await button(page, "Class 1").click();
        await button(page, "Worksheets").click();
        await button(page, "Capital & Lowercase Matching").click();
        await page.getByText("2 of 22", {exact: true}).waitFor();
        await button(page, "⌂ English").click();
        await button(page, "‹ Subjects").click();
        await button(page, "Mathematics").click();
        await button(page, "Class 1").click();
        await button(page, "Worksheets").click();
        await button(page, "Place Value: Numbers 21 to 99").click();
        const number = page.getByLabel("Answer", {exact: true});
        await number.fill("3");
        await button(page, "Take a break").click();
        await button(page, "Continue").click();
        assert.equal(await number.inputValue(), "3");
        await button(page, "Check").click();
        await page.getByText("✓ That’s right.").waitFor();
        await button(page, "⌂ Mathematics").click();
        await button(page, "Class 2").click();
        await button(page, "Tracing Practice").click();
        await button(page, "Numbers").click();
        await page.getByLabel("Choose a model").selectOption("100");
        await page.getByRole("heading", {name: "100", exact: true}).waitFor();
        await page.locator("#practice-trace-ink canvas").nth(5).waitFor();
        assert.equal(await page.locator("#practice-trace-ink canvas").count(), 6);
        await button(page, "‹ All tracing practice").click();
        await button(page, "10 times table").click();
        assert.equal(await page.getByLabel("Choose a model").count(), 0);
        const fact = page.locator("#table-trace-ink canvas");
        await fact.nth(99).waitFor();
        assert.equal(await fact.count(), 100);
        const firstRow = fact.nth(9 * 10 + 3);
        const hasModel = canvas => canvas.evaluate(node =>
            node.getContext("2d").getImageData(0, 0, node.width, node.height).data
                .some((value, index) => index % 4 === 3 && value > 0));
        assert.equal(await hasModel(firstRow), true, "equals sign is traceable");
        assert.equal(await hasModel(fact.last()), false, "the copy row starts empty");
        await button(page, "‹ All tracing practice").click();
        await button(page, "Addition").click();
        await page.getByRole("heading", {name: "12+5=17", exact: true}).waitFor();
        await page.locator("#practice-trace-ink canvas").nth(13).waitFor();
        await button(page, "⌂ Mathematics").click();
        await button(page, "‹ Subjects").click();
        for (const [subject, tracing, words, sentences, paper, back] of [
            ["English", "Tracing Practice", "Common words", "Short sentences", "ink-english", "‹ All tracing practice"],
            ["हिंदी", "बिंदियों पर लिखो", "रोज़ के शब्द", "छोटे वाक्य", "ink-hindi", "‹ सभी लिखाई अभ्यास"],
        ]) {
            await button(page, subject).click();
            await button(page, tracing).click();
            await button(page, words).click();
            const lines = page.locator("#practice-trace-ink canvas");
            await lines.nth(1).waitFor();
            assert.equal(await lines.count(), 2);
            assert.equal(await hasModel(lines.first()), true);
            assert.equal(await hasModel(lines.last()), false);
            await button(page, back).click();
            await button(page, sentences).click();
            await lines.nth(1).waitFor();
            assert.equal(await page.locator("#practice-trace-ink").evaluate((node, name) => node.classList.contains(name), paper), true);
            assert.equal(await page.locator("#practice-trace-ink").evaluate(node =>
                getComputedStyle(node).getPropertyValue("--english-gap").trim()), "0px");
            assert.equal(await hasModel(lines.first()), true);
            await button(page, "⌂ " + subject).click();
            await button(page, subject === "English" ? "‹ Subjects" : "‹ विषय").click();
        }
        console.log("Verified ruled rough sheets, legacy ink, word and sentence traces, and complete math models.");
    });

})().catch(error => { console.error(error); process.exitCode = 1; });

