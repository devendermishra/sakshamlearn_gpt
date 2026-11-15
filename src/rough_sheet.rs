use crate::{
    content::Paper,
    language::Language,
    state::{self, Ink},
    writing::{InkPad, PaperScale},
};
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn RoughSheet(path: PathBuf, paper: Paper, language: Language, scale: PaperScale) -> Element {
    let lines = match paper {
        Paper::Digit => 18,
        Paper::English => 12,
        _ => 24,
    };
    let saved = use_hook(|| state::load_local::<Ink>(&path).map_err(|error| error.to_string()));
    let mut draft = use_signal(|| saved.clone().unwrap_or_default());
    let mut failed = use_signal(|| false);
    let save = use_callback(move |ink: Ink| {
        failed.set(state::save_local(&path, &ink).is_err());
        draft.set(ink);
    });
    let title = language.text("रफ़ कॉपी", "Rough Sheet");
    rsx! {
        section { class: "rough-notebook",
            h1 { "{title}" }
            details { class: "learner-help",
                summary { {language.text("मदद", "Help")} }
                p { {language.text("पेन से कुछ भी लिखो या बनाओ।", "Use your pen to write or draw anywhere on the sheet.")} }
            }
            if saved.is_ok() {
                InkPad { id: "rough-ink", model: String::new(), initial: saved.unwrap_or_default(),
                    scale, paper, lines, full_sheet: true, language, on_ink: move |ink| save.call(ink) }
                if failed() {
                    p { role: "alert", {language.text("कॉपी सहेजी नहीं गई।", "The sheet could not be saved.")} }
                    button { onclick: move |_| save.call(draft()), {language.text("फिर सहेजें", "Try saving again")} }
                }
            } else {
                p { role: "alert", {language.text("कॉपी नहीं खुली।", "The saved sheet could not be opened.")} }
            }
        }
    }
}
