# Saksham

Saksham is an offline learning app for an Android tablet in landscape. Choose Class 1 or Class 2 in the learner menu. Mathematics covers forward and backward counting, before/after/between, ascending and descending order, addition and subtraction, equal groups, tables 1–10, and division by repeated subtraction. Use the language button in Mathematics to switch between Hindi and English. Both languages share the same progress, reviews, and handwriting. Open Solve in writing in either Mathematics class to work out problems in calibrated 11 mm squares and compare the handwritten work with a solution. Hindi includes reading, listening and conversation, and writing. Class 1 includes the full workbook barakhadi and separate ऋ-matra practice. Listening and dictation include scripts for an adult to read aloud. Each topic has immediate recall and follow-up reviews. Submitted writing samples are dated in the parent area. Open Manage activities after entering the parent PIN to create subjects, draft or copy activities, preview them, and publish them on the device. The source PDFs remain in `books` and `practice_books`. See `src/content.rs` for subject, topic, exercise, and refresher definitions.

## English practice

Open **English** and select **Class 1** or **Class 2**. The reading cards use original practice passages and questions based on the units in the two *Mridang* textbooks in `books/English`. Class 1 covers letter sounds, rhymes, familiar words and short sentences. Class 2 adds longer passages, event order, reasons and short written answers. In **Listen and talk**, an adult reads the listening script and takes turns with the learner. Each activity has immediate recall and follow-up reviews. A short break is offered after two exercises when more exercises remain.

Open **Writing practice plan** and do one card a day. Class 1 moves from letter formation to labels, word spacing and a sentence. **Trace the alphabet** lets you open any of the 26 capital or small letters directly. Class 2 moves from letter height and spacing to dictation, questions and two-sentence writing. On-screen trace models use the bundled Patrick Hand font on four-line paper. The Class 2 plan points to pages in `practice_books/English-Cursive-Workbook.pdf` for joined handwriting with an adult.

The three picture activities use illustrations from NCERT’s *Mridang*, Class 1 pages 47 and 84 and Class 2 page 8. The app includes these pictures for offline use.

Use **Sound out words** for short-a word families such as `-an` and `-ap`. Use **Read and write common words** to read words in short sentences and practise them by tracing, copying or writing from dictation. The calibrated English guides are 4 mm apart, as in the workbook.

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

The script builds an ARM64 debug APK, adds the Saksham launcher icon, and packages the result with Gradle's debug signing key. Keep the same signing key for in-place updates. You can also copy the APK to the tablet and open it there. Run `cargo run --locked` for a Windows desktop preview with WebView2.

On the tablet, open **बड़ों के लिए**, set a parent PIN, and measure both edges of the 80 mm reference square with a physical ruler. Enter both measurements, adjust the guides, and check the Hindi and Mathematics samples with the ruler before saving. Repeat if the device display scaling changes. The default size before calibration is an estimate. Writing accepts active stylus input. Tap buttons with a finger or stylus. Progress and saved writing live in the app's local storage and will be erased if its app data is cleared.

Trace models use round dots along a single centre path to keep Hindi loops clear, with blue arrows suggesting a tracing direction. Arrows describe local path direction, rather than a prescribed handwriting stroke sequence. The original filled glyph remains the coverage target for feedback. Run `node --test tests/trace.cjs tests/trace-guide.cjs` for path and feedback checks, and `node tests/trace-guide-ui.cjs` for headless Edge rendering checks and a preview in `target/trace-review/dotted-hindi.png`.

Hindi tracing uses the bundled Kalam handwriting font (Indian Type Foundry, SIL OFL; see `assets/Kalam-OFL.txt`) so small handwritten loops do not depend on the tablet's system font. Paths are extracted at three times the canvas resolution, then scaled to the calibrated paper. Tiny loops use smaller, closer dots. `node tests/hindi-loop-ui.cjs` reviews 49 Hindi forms and checks loops in 13 base forms and 84 forms with matras; `node tests/trace-guide-ui.cjs` also checks the integrated renderer for झ, द, दा, दि, and related letters.

## Rough sheets and worksheets

Open Hindi, Mathematics, or English and choose **Rough Sheet** for pen writing or **Worksheets** for guided practice in the selected class. Rough sheets use the calibrated Hindi rules, English four-line guides, or Mathematics grid. **Tracing Practice** offers common words and short sentences in Hindi and English, and numbers, addition facts, and times tables in Mathematics. Each subject has one rough sheet for both classes. The app saves completed strokes, answers, and the current worksheet step on the device. **Take a break** returns to the same worksheet task when the learner chooses **Continue**.

Worksheets are bundled for offline use. The six Word packs can be re-imported with:

```powershell
python tools/import_worksheets.py <folder-with-the-six-docx-files> src/content/worksheets.json
```

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
