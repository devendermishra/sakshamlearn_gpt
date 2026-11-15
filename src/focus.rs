use crate::{
    keypad::{Keys, NumericField},
    language::Language,
    platform::{self, Focus},
    state::Pin,
};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Prompt {
    Setup,
    Exit,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Controls {
    mode: Signal<Option<Focus>>,
    prompt: Signal<Option<Prompt>>,
    status: Signal<String>,
}

impl Controls {
    pub fn use_controls() -> Self {
        let controls = Self {
            mode: use_signal(|| platform::focus().ok()),
            prompt: use_signal(|| None),
            status: use_signal(String::new),
        };
        use_future(move || async move {
            if !cfg!(target_os = "android") {
                return;
            }
            let mut events = document::eval(
                "const refresh = () => dioxus.send(true);
                 window.addEventListener('focus', refresh);
                 window.addEventListener('saksham:android-focus', refresh);
                 document.addEventListener('visibilitychange', () => {
                     if (!document.hidden) refresh();
                 });
                 await new Promise(() => {});",
            );
            while events.recv::<bool>().await.is_ok() {
                controls.refresh();
            }
        });
        controls
    }

    fn refresh(mut self) -> Option<Focus> {
        let actual = platform::focus().ok();
        if *self.mode.peek() != actual {
            self.mode.set(actual);
        }
        actual
    }

    fn change(mut self, enable: bool) {
        match platform::set_focus(enable) {
            Ok(()) => {
                self.status.set(String::new());
                self.refresh();
                // The native transition can arrive after the call returns.
                spawn(async move {
                    let mut event = document::eval(
                        "await new Promise(resolve => setTimeout(resolve, 350)); dioxus.send(true);",
                    );
                    if event.recv::<bool>().await.is_ok() {
                        self.refresh();
                    }
                });
            }
            Err(error) => {
                self.status.set(error);
                self.refresh();
            }
        }
    }

    fn request(mut self, has_pin: bool) {
        self.status.set(String::new());
        match self.refresh() {
            Some(Focus::Off) if !has_pin => self.prompt.set(Some(Prompt::Setup)),
            Some(Focus::Off) => self.change(true),
            Some(_) => self.prompt.set(Some(Prompt::Exit)),
            None => self.status.set("Focus Mode status is unavailable.".into()),
        }
    }
}

#[component]
pub fn FocusButton(controls: Controls, has_pin: bool, language: Language) -> Element {
    if !cfg!(target_os = "android") {
        return rsx! {};
    }
    let mode = match (controls.mode)() {
        Some(Focus::Off) => language.text("बंद", "Off"),
        Some(Focus::Pinned) => language.text("पिन किया", "Pinned"),
        Some(Focus::Locked) => language.text("लॉक है", "Locked"),
        None => language.text("उपलब्ध नहीं", "Unavailable"),
    };
    let label = language.text("ध्यान मोड", "Focus Mode");
    rsx! {
        button { class: "focus-shortcut", onclick: move |_| controls.request(has_pin),
            span { aria_hidden: "true", "◎" }
            "{label} · {mode}"
        }
    }
}

#[component]
pub fn FocusPrompt(
    mut controls: Controls,
    pin: Option<Pin>,
    save_pin: Callback<Pin, Result<(), String>>,
    language: Language,
) -> Element {
    let Some(prompt) = (controls.prompt)() else {
        return rsx! {
            if !(controls.status)().is_empty() {
                p { class: "focus-message", role: "status", "{(controls.status)()}" }
            }
        };
    };
    let title = match prompt {
        Prompt::Setup => language.text("बड़ों का पिन बनाएँ", "Set a parent PIN"),
        Prompt::Exit => language.text("ध्यान मोड से बाहर निकलें", "Exit Focus Mode"),
    };
    let setup = prompt == Prompt::Setup;
    rsx! {
        PinDialog {
            title, setup, language,
            on_cancel: move |_| controls.prompt.set(None),
            on_submit: move |value: String| {
                if setup {
                    save_pin.call(Pin::new(&value))?;
                    controls.status.set(language.text(
                        "पिन सहेजा गया। शुरू करने के लिए ध्यान मोड दबाएँ।",
                        "PIN saved. Tap Focus Mode to start.",
                    ).into());
                } else if pin.as_ref().is_some_and(|pin| pin.matches(&value)) {
                    if controls.refresh().is_some_and(|mode| mode != Focus::Off) {
                        controls.change(false);
                    }
                } else {
                    return Err(language.text("पिन सही नहीं है।", "Incorrect PIN.").into());
                }
                controls.prompt.set(None);
                Ok(())
            },
        }
    }
}

#[component]
fn PinDialog(
    title: &'static str,
    setup: bool,
    language: Language,
    on_cancel: EventHandler<()>,
    on_submit: Callback<String, Result<(), String>>,
) -> Element {
    let mut input = use_signal(String::new);
    let mut error = use_signal(String::new);
    use_effect(|| {
        let _ = document::eval(
            "const dialog = document.getElementById('focus-pin');
             if (dialog && !dialog.open) dialog.showModal();",
        );
    });
    rsx! {
        dialog { id: "focus-pin", aria_labelledby: "focus-pin-title",
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    on_cancel.call(());
                }
            },
            h2 { id: "focus-pin-title", "{title}" }
            form { onsubmit: move |event| {
                event.prevent_default();
                let value = input();
                input.set(String::new());
                if value.len() < 4 || value.len() > 8 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    error.set(language.text("4 से 8 अंक लिखें।", "Use 4 to 8 digits.").into());
                } else {
                    error.set(on_submit.call(value).err().unwrap_or_default());
                }
            },
                NumericField { label: language.text("बड़ों का पिन", "Parent PIN").to_string(),
                    value: input(), language, limit: 8, keys: Keys::Pin, on_change: move |value| input.set(value) }
                p { role: "status", "{error}" }
                div { class: "actions",
                    button { r#type: "button", onclick: move |_| on_cancel.call(()), {language.text("रद्द करें", "Cancel")} }
                    button { class: "primary", r#type: "submit",
                        if setup { {language.text("पिन सहेजें", "Save PIN")} }
                        else { {language.text("बाहर निकलें", "Exit")} }
                    }
                }
            }
        }
    }
}

