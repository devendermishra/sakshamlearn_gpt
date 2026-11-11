use std::{io, path::PathBuf};

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Off,
    Pinned,
    Locked,
}

impl Focus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Pinned => "Pinned",
            Self::Locked => "Locked",
        }
    }
}

#[cfg(not(target_os = "android"))]
pub fn state_path() -> io::Result<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .ok_or_else(|| io::Error::other("No application data directory"))?;
    Ok(base.join("Saksham").join("progress.json"))
}

#[cfg(not(target_os = "android"))]
pub fn focus() -> Result<Focus, String> {
    Ok(Focus::Off)
}

#[cfg(not(target_os = "android"))]
pub fn set_focus(_: bool) -> Result<(), String> {
    Err("Focus Mode is available in the Android app.".into())
}

#[cfg(not(target_os = "android"))]
pub fn landscape() {}

#[cfg(target_os = "android")]
pub use android::{focus, landscape, set_focus, state_path};

#[cfg(target_os = "android")]
mod android {
    use super::*;
    use jni::{
        objects::{JObject, JString},
        JNIEnv, JavaVM,
    };

    fn activity<T>(
        action: impl FnOnce(&mut JNIEnv<'_>, &JObject<'_>) -> jni::errors::Result<T>,
    ) -> Result<T, String> {
        let context = ndk_context::android_context();
        // The Dioxus Android host retains the JVM and Activity for the event loop lifetime.
        let vm = unsafe { JavaVM::from_raw(context.vm().cast()) }.map_err(|err| err.to_string())?;
        let mut env = vm.attach_current_thread().map_err(|err| err.to_string())?;
        let activity = unsafe { JObject::from_raw(context.context().cast()) };
        let result = env.with_local_frame(16, |env| action(env, &activity));
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
            return Err(
                "Android rejected the request. Check screen pinning or device policy.".into(),
            );
        }
        result.map_err(|err| err.to_string())
    }

    pub fn state_path() -> io::Result<PathBuf> {
        activity(|env, activity| {
            let dir = env
                .call_method(activity, "getFilesDir", "()Ljava/io/File;", &[])?
                .l()?;
            let path = env
                .call_method(dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
                .l()?;
            let path = JString::from(path);
            let path: String = env.get_string(&path)?.into();
            Ok(PathBuf::from(path).join("progress.json"))
        })
        .map_err(io::Error::other)
    }

    pub fn landscape() {
        // ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
        let _ = activity(|env, activity| {
            env.call_method(activity, "setRequestedOrientation", "(I)V", &[6.into()])?;
            Ok(())
        });
    }

    pub fn focus() -> Result<Focus, String> {
        activity(|env, activity| {
            let name = env.new_string("activity")?;
            let manager = env
                .call_method(
                    activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[(&name).into()],
                )?
                .l()?;
            let state = env
                .call_method(manager, "getLockTaskModeState", "()I", &[])?
                .i()?;
            Ok(match state {
                1 => Focus::Locked,
                2 => Focus::Pinned,
                _ => Focus::Off,
            })
        })
    }

    pub fn set_focus(enabled: bool) -> Result<(), String> {
        activity(|env, activity| {
            let method = if enabled {
                "startLockTask"
            } else {
                "stopLockTask"
            };
            env.call_method(activity, method, "()V", &[])?;
            Ok(())
        })
    }
}
