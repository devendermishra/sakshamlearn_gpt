# Saksham

Saksham is an offline learning app for an Android tablet in landscape. Choose Class 1 or Class 2 in the learner menu. Mathematics covers forward and backward counting, before/after/between, ascending and descending order, addition and subtraction, equal groups, tables 1–10, and division by repeated subtraction. Use the language button in Mathematics to switch between Hindi and English. Both languages share the same progress, reviews, and handwriting. Open Solve in writing in either Mathematics class to work out problems in calibrated 11 mm squares and compare the handwritten work with a solution. Hindi includes reading, listening and conversation, and writing. Class 1 includes the full workbook barakhadi and separate ऋ-matra practice. Listening and dictation include scripts for an adult to read aloud. Each topic has immediate recall and follow-up reviews. Submitted writing samples are dated in the parent area. Open Manage activities after entering the parent PIN to create subjects, draft or copy activities, preview them, and publish them on the device. The source PDFs remain in `books` and `practice_books`. See `src/content.rs` for subject, topic, exercise, and refresher definitions.

## Build and install

Install Rust, the Dioxus 0.7.10 CLI, and Android Studio with a JDK, Android SDK, build tools, platform tools, and a side-by-side NDK. Configure `ANDROID_HOME` and `ANDROID_NDK_HOME` for the installed versions, then run:

```powershell
rustup target add aarch64-linux-android
cargo test --locked
node --test tests/trace.cjs
dx bundle --platform android --target aarch64-linux-android --package-types apk --no-default-features --features mobile --locked
adb devices
adb install -r <apk-path>
```

Use the APK path reported by Dioxus. If the Android build produces an unsigned APK, sign it with the Android SDK build tools and a development key before installing. Enable USB debugging only for development. Run `cargo run --locked` for a Windows desktop preview with WebView2.

On the tablet, open **बड़ों के लिए**, set a parent PIN, and measure both edges of the 80 mm reference square with a physical ruler. Enter both measurements, adjust the guides, and check the Hindi and Mathematics samples with the ruler before saving. Repeat if the device display scaling changes. The default size before calibration is an estimate. Writing accepts active stylus input. Tap buttons with a finger or stylus. Progress and saved writing live in the app's local storage and will be erased if its app data is cleared.

## Parent-controlled Focus Mode

**Shared tablet.** In Android Settings, search for **Screen pinning** or **App pinning** and enable it. Enable the device unlock requirement for unpinning if the tablet offers it. In Saksham, enter the parent PIN and select **Start Focus Mode**. Confirm Android's pinning prompt. The app shows **Pinned** once Android reports it. Android's own unpin gesture is still available.

**Dedicated tablet.** Use an Android Enterprise device policy controller (DPC) provisioned by the tablet administrator on a device prepared for dedicated use. Provisioning may require a factory reset. Add the Saksham APK package ID to the DPC's lock-task allowlist. Confirm the installed package ID using the SDK's `apkanalyzer manifest application-id <apk-path>` if needed. Configure lock-task features to omit Home, Recents, notifications, and other app entry points. From Saksham's parent area start Focus Mode and confirm the status reads **Locked**.

Before handing over a managed tablet, test parent PIN exit and DPC recovery. If the parent PIN is unavailable or the app cannot open its parent screen, the administrator can remove Saksham from the DPC's lock-task allowlist to release the device, then service the app from Android settings. Keep access to the DPC and its admin credentials outside the tablet. See [Android's Lock Task Mode guide](https://developer.android.com/work/dpc/dedicated-devices/lock-task-mode).

## Device acceptance

- On each tablet, check that the app opens in landscape, Hindi shaping renders correctly, a pen stroke follows the tip, resting a palm does not add marks, and the two Hindi writing rules are 8 mm apart and Mathematics spans 110 mm across ten 11 mm squares, each 11 mm high.
- In airplane mode, complete a reading card and writing item. Pause, force-stop and reopen the app, resume the same item, and open the writing sample from the parent area.
- Complete a topic and repeat it to verify that the star total does not increase a second time. Check due reviews and next-day retry behavior.
- Trace a model, an incomplete model, and a scribble. Verify that the hint responds and that an attempt can advance regardless of its trace result.
- Exercise shared pinning and managed lock separately. On a provisioned tablet verify Home, Recents, notification shade, and app switching are blocked. Check parent PIN exit and administrator recovery.
