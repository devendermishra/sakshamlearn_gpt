# Saksham

Saksham is an offline learning app for an Android tablet in landscape. Choose Class 1 or Class 2 in the learner menu. Mathematics covers forward and backward counting, before/after/between, ascending and descending order, addition and subtraction, equal groups, tables 1–10, and division by repeated subtraction. Use the language button in Mathematics to switch between Hindi and English. Both languages share the same progress, reviews, and handwriting. Open Solve in writing in either Mathematics class to work out problems in calibrated 11 mm squares and compare the handwritten work with a solution. Hindi includes reading, listening and conversation, and writing. Class 1 includes the full workbook barakhadi and separate ऋ-matra practice. Listening and dictation include scripts for an adult to read aloud. Each topic has immediate recall and follow-up reviews. Submitted writing samples are dated in the parent area. Open Manage activities after entering the parent PIN to create subjects, draft or copy activities, preview them, and publish them on the device. The source PDFs remain in `books` and `practice_books`. See `src/content.rs` for subject, topic, exercise, and refresher definitions.

## English practice

Open **English** and select **Class 1** or **Class 2**. The reading cards use original practice passages and questions based on the units in the two *Mridang* textbooks in `books/English`. Class 1 covers letter sounds, rhymes, familiar words and short sentences. Class 2 adds longer passages, event order, reasons and short written answers. In **Listen and talk**, an adult reads the listening script and takes turns with the learner. Each activity has immediate recall and follow-up reviews. A short break is offered after two exercises when more exercises remain.

Open **Writing practice plan** and do one card a day. Class 1 moves from letter formation to labels, word spacing and a sentence. **Trace the alphabet** lets you open any of the 26 capital or small letters directly. Class 2 moves from letter height and spacing to dictation, questions and two-sentence writing. On-screen trace models use the bundled Patrick Hand font on four-line paper. The Class 2 plan points to pages in `practice_books/English-Cursive-Workbook.pdf` for joined handwriting with an adult.

The three picture activities use illustrations from NCERT’s *Mridang*, Class 1 pages 47 and 84 and Class 2 page 8. The app includes these pictures for offline use.

Use **Sound out words** for short-a word families such as `-an` and `-ap`. Use **Read and write common words** to read words in short sentences and practise them by tracing, copying or writing from dictation. The calibrated English guides are 4 mm apart, with a 5.6 mm gap between successive four-line writing groups throughout English practice. Saved handwriting keeps its guide alignment.

In Class 1, **Write small letters** covers lowercase a–z in pairs. In both classes, **Write practice words** gives the `-an` and `-ap` families their own tracing and handwriting cards.

Open **Daily recall** in Hindi, English, or Mathematics after completing a topic. Each round contains up to three questions per completed topic, in chapter order, and can be resumed during the same India calendar day. In the Hindi chapters, choose a letter or matra to practise one form and return to the chapter.

## Everyday vocabulary

Open **Everyday words** in English or **रोज़ के शब्द** in Hindi for either class. Class 1 covers home and family, school supplies, food, everyday actions, colours and sizes, and polite words. Class 2 covers questions, position, time and sequence, pronouns, and connecting ideas. Each card introduces four words in context and includes tracing, adult-led dictation, and recall. The sets use the themes in the *Mridang* and *Sarangi* textbooks and the vocabulary goals in NCERT’s [National Curriculum Framework for Foundational Stage, competency C-9.7](https://www.ncert.nic.in/pdf/NCF_for_Foundational_Stage_20_October_2022.pdf).

## Build and install

Build from this `saksham-stage` checkout, which includes both Colors & backgrounds and the dotted tracing update. Colors & backgrounds in the header offers Forest, Sky, Marigold, and Plum palettes plus Storybook, Playground, Animal friends, and Plain backgrounds. Choices are saved with learner progress. Run `node tests/appearance-ui.cjs` after a desktop build to check theme switching and restart persistence.

Install Rust, the Dioxus 0.7.10 CLI, and Android Studio with a JDK, Android SDK, build tools, platform tools, and a side-by-side NDK. Configure `ANDROID_HOME` and `ANDROID_NDK_HOME` for the installed versions, then run:

```powershell
rustup target add aarch64-linux-android
cargo test --locked
node --test tests/trace.cjs tests/pencil.cjs
./build-android.ps1
adb devices
adb install -r target/Saksham-Android.apk
```

The script builds an optimized ARM64 release APK by default (`-DebugBuild` selects a development APK), adds the Saksham launcher icon, and packages the result with Gradle's debug signing key. Keep the same signing key for in-place updates. You can also copy the APK to the tablet and open it there. Run `cargo run --locked` for a Windows desktop preview with WebView2.

### Offline operation and battery checks

Android retains Internet permission solely because Dioxus 0.7.10 uses a device-local `ws://127.0.0.1:<port>` connection to render UI changes. This uses no mobile or Wi-Fi destination; Android may still attribute loopback bytes to the app. Bundled content and files stay on the device. Android request/navigation guards and a document content security policy block external resources and connections, including links routed through Dioxus IPC. Missing local assets fail locally. WebView Safe Browsing and AppCompat's downloadable emoji font/profile startup components are disabled. The build stops if upstream Wry templates change or the final APK introduces unexpected permissions or background components.

WebView timers pause in the background. The ABA calendar schedules one India-midnight update while visible and refreshes on resume. Pencil drawing batches work by animation frame, paints new segments, and saves completed/interrupted strokes plus dirty long-stroke checkpoints at most once per second. Handwriting flushes before pause and navigation; no background service, alarm, or wake lock is added.

```powershell
node --test tests/pencil.cjs tests/day-clock.cjs tests/trace.cjs
./tests/patch-android.ps1
node tests/offline-ui.cjs
node tests/handwriting-lifecycle-ui.cjs
```

The browser checks require Edge and the bundled Playwright package. The offline test explicitly grants desktop Chromium's localhost permission to its synthetic app origin, then verifies the shipped CSP permits local UI traffic and blocks external requests before they reach the network.

On the tablet, install as an update and verify progress remains intact. Complete reading, writing, Basic, ABA, worksheets, and Focus Mode checks in airplane mode and with connectivity enabled. Background during a pen stroke, resume, then restart and confirm the saved stroke. Test India-midnight rollover and returning after multiple days. Compare the old and new APK with the same brightness, content, and device conditions: ten minutes idle in foreground, ten minutes backgrounded/screen-off, and five minutes drawing. Collect per-UID network counters, CPU time, wake locks, and a system trace with Android Studio/ADB. Require no external app connections, no periodic app polling/drawing in the background, and no app-held wake locks. Device measurements are required before claiming a battery reduction; localhost counters alone do not establish Internet data usage.

On the tablet, open **बड़ों के लिए**, set a parent PIN, and measure both edges of the 80 mm reference square with a physical ruler. Enter both measurements, adjust the guides, and check the Hindi and Mathematics samples with the ruler before saving. Repeat if the device display scaling changes. The default size before calibration is an estimate. Writing accepts active stylus input. Tap buttons with a finger or stylus. Progress and saved writing live in the app's local storage and will be erased if its app data is cleared.

Trace models use round dots along a single centre path to keep Hindi loops clear, with blue arrows suggesting a tracing direction. Arrows describe local path direction, rather than a prescribed handwriting stroke sequence. The original filled glyph remains the coverage target for feedback. Run `node --test tests/trace.cjs tests/trace-guide.cjs` for path and feedback checks, and `node tests/trace-guide-ui.cjs` for headless Edge rendering checks and a preview in `target/trace-review/dotted-hindi.png`.

Hindi tracing uses the bundled Kalam handwriting font (Indian Type Foundry, SIL OFL; see `assets/Kalam-OFL.txt`) so small handwritten loops do not depend on the tablet's system font. Practice text containing ब uses the bundled, unmodified Noto Sans Devanagari Regular font for its standard upright form (SIL OFL; see `assets/NotoSansDevanagari-OFL.txt`). The whole model uses the same font so matras and conjuncts shape together. Source: https://github.com/notofonts/noto-fonts/tree/main/hinted/ttf/NotoSansDevanagari. Paths are extracted at three times the canvas resolution, then scaled to the calibrated paper. Tiny loops use smaller, closer dots. `node tests/hindi-loop-ui.cjs` reviews 49 Hindi forms and checks loops in 13 base forms and 84 forms with matras; `node tests/trace-guide-ui.cjs` also checks the integrated renderer for झ, द, दा, दि, and related letters.

## Rough sheets and worksheets

Open Hindi, Mathematics, or English and choose **Rough Sheet** for pen writing or **Worksheets** for guided practice in the selected class. Rough sheets use the calibrated Hindi rules, English four-line guides, or Mathematics grid. **Tracing Practice** offers common words and short sentences in Hindi and English, and numbers, addition facts, and times tables in Mathematics. Each subject has one rough sheet for both classes. The app saves completed strokes, answers, and the current worksheet step on the device. **Take a break** returns to the same worksheet task when the learner chooses **Continue**.

Worksheets are bundled for offline use. The six Word packs can be re-imported with:

```powershell
python tools/import_worksheets.py <folder-with-the-six-docx-files> src/content/worksheets.json
```

## Eraser and exercise lock

Choose **Eraser** on any editable writing surface and touch or drag across learner ink to remove whole strokes. **Undo** restores everything removed by the last eraser gesture. Printed guides and worksheet artwork remain intact. Choose **Pen** to write again. Tablet input uses the stylus; desktop previews also accept the mouse.

Choose **Lock exercise** to keep the current practice item open. Navigation, item selection, resets, and actions that complete or advance an item are disabled until **Unlock exercise** is tapped. Writing, erasing, undo, zoom, and checks that stay on the item remain available. Breaks retain the lock and resume the same item. The lock needs no PIN and starts off after restarting the app. It is separate from parent-controlled Focus Mode.

Run `node tests/eraser-ui.cjs` and `node tests/exercise-lock-ui.cjs` after a desktop build. On Android, verify stylus erasing, palm rejection, accidental navigation taps, break/resume, and persistence after restarting.

## Parent-controlled Focus Mode

Set a parent PIN once through **For parents** or the Focus Mode shortcut. Tap the **Focus Mode** shortcut to start. To leave Focus Mode from Saksham, tap the shortcut again and enter the parent PIN. The parent settings button opens the same PIN prompt.

**Shared tablet.** In Android Settings, enable **Screen pinning** or **App pinning**. Enable the device unlock requirement for unpinning if the tablet offers it. Android may ask for confirmation when the learner starts Focus Mode. Saksham displays **Pinned** after Android confirms. Android's own unpin gesture remains available.

**Dedicated tablet.** Add Saksham's package ID to the administrator's lock-task allowlist with the tablet's device policy controller (DPC). Configure the desired restrictions for Home, Recents, and notifications. Start Focus Mode and confirm Saksham displays **Locked**. Verify the parent PIN exit and test administrator recovery by removing the app from the allowlist. See the [Android lock task mode guide](https://developer.android.com/work/dpc/dedicated-devices/lock-task-mode).

## Device acceptance

- On each tablet, check that the app opens in landscape, Hindi shaping renders correctly, a pen stroke follows the tip, resting a palm does not add marks, and the two Hindi writing rules are 8 mm apart and Mathematics spans 110 mm across ten 11 mm squares, each 11 mm high.
- In airplane mode, complete an English reading card and write on the four-line English paper. Pause, force-stop and reopen the app, resume the same item, and open the writing sample from the parent area.
- Complete a topic and repeat it to verify that the star total does not increase a second time. Check due reviews and next-day retry behavior.
- Trace a model, an incomplete model, and a scribble. Verify that the hint responds and that an attempt can advance regardless of its trace result.
- Exercise shared pinning and managed lock separately. On a provisioned tablet verify Home, Recents, notification shade, and app switching are blocked. Check parent PIN exit and administrator recovery.


## Basic pencil control and ABA picture practice

Open **Basic** from the learner menu, independently of class selection. Its 68 activities cover all 14 pencil-control source pages: one practice band at a time on pages 1–8, then six full-page paths. Draw with the stylus; use **Undo stroke**, **Clear exercise**, and **Mark complete** as needed. **Move page** disables drawing while panning, and +/− changes zoom. **All pages** jumps directly to a worksheet. Strokes, completion, position, and breaks are saved locally. An adult decides when to mark an exercise complete; tracing accuracy never blocks advancement.

In **ABA Learning**, choose **Flashcards** or **Exercises**, or use **Open practice** beside any of the 30 checklist activities. Picture practice includes identification, matching, transport/bird sorting, alternating patterns, yes/no, and adult-led speech and practical tasks. Rounds contain three tasks and offer repeat, continue, or a break. Adults record **Independent**, **With help**, or **Try later**. These records do not tick the daily checklist. Oral-motor activities retain the existing therapist-directed instructions.

The offline image catalog contains 42 realistic pictures generated with the built-in OpenAI imagegen tool, plus 24 original cards from Autism Parenting Magazine's *Speech Practice Flashcards*. Source-page numbers appear in card notes. Original CV/VC labels are preserved as source prompts and are not pronunciation scoring rules. Generated prompts are recorded in `assets/pictures/generation-prompts.json`. Shared pictures support concrete English/Hindi vocabulary in both classes, position words, counting, equal groups, and selected mathematics worksheets. Parent-authored exercises can optionally select a bundled picture; older saved exercises remain compatible.

Re-import the two source PDFs with Python, Pillow, and pypdfium2 installed:

```powershell
python tools/import_practice_pdfs.py D:/Garage/sakshamlearn_gpt/practice_books
```

Basic work is stored in `learner-work/basic-pencil.json`; ABA rounds are in `learner-work/aba-exercises.json`, separate from daily checklist progress. If saved work cannot be read, the module displays an error instead of overwriting it. A failed write offers retry.

Additional desktop verification:

```powershell
cargo test --locked
node --test tests/trace.cjs
cargo build --locked
node tests/practice-ui.cjs
```

The UI check uses the bundled Playwright package beside the Node runtime and a fresh profile under `target`; it verifies pencil controls, restart recovery, checklist links, ABA tasks, and offline cards. Screenshots are saved under `target/practice-review`. On the Android tablet, additionally verify active-stylus input, palm rejection, touch scrolling in Move page mode, landscape layout, and airplane-mode restart recovery.

## Daily exercise plans

Open **Daily exercise plans · रोज़ के अभ्यास** from the learner home. The combined plan contains English, Hindi, Mathematics and Basic every day. Its days 1–30 cover Class 1 and days 31–60 cover Class 2. Basic covers all 68 pencil-control bands and paths in source order. The plan includes every bundled academic lesson exercise, recall question and worksheet task; ABA and therapy activities are excluded. Academic listening and conversation remain included.

Choose **Combined · 60 days**, **Class 1 · 30 days**, or **Class 2 · 60 days**. The Class 1 plan reuses combined days 1–30; the Class 2 plan spreads its academic exercises across 60 days. Each class retains its existing Basic exercises; some Class 2 days have no Basic task. Completion, answers and handwriting are shared across daily plans by exercise ID, while each plan remembers its own day, open exercise and break. Completing an exercise in one plan counts wherever it appears. Ordinary subject practice remains independent. Existing daily saves are upgraded with their combined-plan work and location intact.

All days are open. Choose **Continue next unfinished day**, select a subject, and complete its tasks in the pane. Daily counts reflect the full content, so workloads vary. Work, answers, handwriting, breaks and resume location are saved offline in `learner-work/daily-exercises.json`, independently of normal lesson and worksheet progress. Completed tasks can be reviewed or retried without increasing completion counts. A day is complete when all its scheduled tasks are complete; no Basic work is required on days without a Basic task.

The versioned manifest and coverage inventory are built deterministically from bundled source IDs in `src/daily_exercises.rs`. **Coverage inventory** in the pane lists the day, subject, goal, source and task count for every scheduled group. Dependent composition/revision steps stay together. Version 1 daily saves migrate automatically to the shared-plan format; other incompatible versions require an explicit migration; incompatible or unreadable saves are reported and left untouched.

Run `cargo test --locked`, `cargo build --locked`, `node tests/daily-exercises-ui.cjs`, `node --test tests/english-spacing.cjs`, and `node tests/english-spacing-ui.cjs` to verify coverage, persistence, calibrated paper spacing and the desktop pane. On Android, additionally verify active stylus input, palm rejection, airplane-mode restart and landscape layout using the existing device acceptance checks.

## Optimized Android builds

`./build-android.ps1` builds the optimized ARM64 release Rust library and packages a non-debuggable, minified APK. It uses the existing Gradle debug signing key for tablet update compatibility; use a dedicated release key for public distribution. `./build-android.ps1 -DebugBuild` retains the development build. Add `-Offline` once the toolchain dependencies have been cached. Release Rust uses size optimization, thin LTO, one codegen unit and stripped symbols.

Drawing runs locally in the WebView canvas; Rust owns learning logic and ordered background persistence. Progress retains its existing JSON format. Pending saves coalesce by file, use buffered atomic writes, and finish before a file is reopened. Pencil input renders new segments once per animation frame and checkpoints long strokes every two seconds, then saves immediately at stroke completion, blur and visibility changes. Parent handwriting history is preserved and paginated.


## Adaptive Learning V1

**Today's learning** offers a separate, resumable session for the selected Class 1 or 2 across English, Hindi, and mathematics. The existing 60-day plans and individual lessons remain available. Parents can set an estimated 5–30 minute budget (15 by default) behind the parent PIN. Activity durations are planning estimates, not measured ability or a promised session length.

The planner selects overdue competency checks, prerequisite practice, then at most two new units. It reserves up to 20% of the budget for new work that fits, orders reviews by oldest due date, rotates new-work subject priority by India learning date, and keeps composition/revision chains together. Recommendations never lock lessons. Unscored skills can be introduced without claiming mastery. An unfinished queue stays available after restarting; a new queue starts only when requested.

`assessment` maps bundled exercises and recall questions to stable source-based competency IDs. Formation, writing, conversation, recognition, and arithmetic are distinct skills. Bilingual math uses the same IDs; topic revision and exercise IDs remain in attempt provenance. The explicit arithmetic prerequisite graph covers addition before carrying, subtraction before borrowing, and multiplication before division; other skills have no inferred prerequisite. Custom authored activities, worksheets, pencil practice, ABA, and generated math rounds are outside V1 estimates.

The main learner state contains a versioned, deduplicated attempt journal: submission ID, source, exercise and competency, UTC timestamp in milliseconds and India learning day, optional correctness/accuracy, modality, assistance, visible models, errors, hints, helper/solution use, and optional duration. Duration is currently unmeasured. Answers entered after writing assess the entered arithmetic response, not handwriting. Writing, tracing coverage, and conversation are unscored. First-response errors remain errors after correction; app help and optionally reported adult help exclude independent success. “Unreported” external help is distinct from a parent's explicit “Independent” report. Answer and assistance drafts survive restarts.

The parent **Learning evidence** dashboard displays rule-based estimates rather than validated psychometric scores:

- **No evidence:** no recorded attempt for the skill.
- **Practising:** evidence exists but independent-success criteria are unmet, including a most recent independent failure.
- **Independent success:** at least three independent scored attempts across two learning dates, at least 80% success among the latest ten, and the latest eligible attempt successful.
- **Retained:** independent success plus successful independent reviews on two later dates, one at least seven days after first qualifying success. A scored independent failure resets the retention observation period.

Independence is observed within the app; outside help can remain unreported. The dashboard shows this limitation, modality/assistance counts, unscored work, review history, and recommended targets. No handwriting recognition or clinical assessment is implied.

Checks begin after one day. Independent review success doubles the interval to at most 60 days; independent failure halves it (minimum one day) and schedules tomorrow. Assisted or unscored checks cannot increase the interval and also schedule tomorrow. Reviews continue indefinitely and their outcomes remain in the journal. Existing topic review navigation remains compatible and now continues instead of ending after three rounds.

Older saves gain empty evidence fields. Completion and stars are preserved and never converted into correct answers. Completed topics receive baseline recall checks using legacy due dates where available, otherwise due now; the planner spreads those checks across its budget. New daily-recall rounds store competency provenance; already saved rounds without provenance can finish without guessed evidence. The 60-day plan keeps its existing work files and a submission journal, committing canonical evidence before acknowledging separate task completion. Replays use the same durable submission IDs. Failed canonical saves leave the activity available for retry; existing save-error handling covers separate work-file failures.

Run `cargo test --locked`, `cargo build --locked`, and `node tests/adaptive-learning-ui.cjs`. Adaptive tests cover migration, identity across translations, conservative mastery, ongoing schedules, duplicate replay, failed saves, budgeted backlog simulations, deterministic planning, and session persistence. Android acceptance still requires stylus/palm rejection, focus locking, airplane-mode restart, layout, and storage-failure checks on a tablet. Encrypted backup and richer tracing validation remain future work.
