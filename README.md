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

Install Rust, the Dioxus 0.7.10 CLI, and Android Studio with a JDK, Android SDK, build tools, platform tools, and a side-by-side NDK. Configure `ANDROID_HOME` and `ANDROID_NDK_HOME` for the installed versions, then run:

```powershell
rustup target add aarch64-linux-android
cargo test --locked
node --test tests/trace.cjs
./build-android.ps1
adb devices
adb install -r target/Saksham-Android.apk
```

The script builds an ARM64 debug APK, adds the Saksham launcher icon, and packages the result with Gradle's debug signing key. Keep the same signing key for in-place updates. You can also copy the APK to the tablet and open it there. Run `cargo run --locked` for a Windows desktop preview with WebView2.

On the tablet, open **बड़ों के लिए**, set a parent PIN, and measure both edges of the 80 mm reference square with a physical ruler. Enter both measurements, adjust the guides, and check the Hindi and Mathematics samples with the ruler before saving. Repeat if the device display scaling changes. The default size before calibration is an estimate. Writing accepts active stylus input. Tap buttons with a finger or stylus. Progress and saved writing live in the app's local storage and will be erased if its app data is cleared.

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
