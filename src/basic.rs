use crate::{
    language::Language,
    state::{self, Stroke},
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::LazyLock,
};

#[derive(Deserialize)]
struct Sheet {
    page: usize,
    row: usize,
    file: String,
}
static SHEETS: LazyLock<Vec<Sheet>> =
    LazyLock::new(|| serde_json::from_str(include_str!("../assets/practice/pencil.json")).unwrap());
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Progress {
    position: usize,
    paused: bool,
    strokes: BTreeMap<usize, Vec<Stroke>>,
    completed: BTreeSet<usize>,
}

fn artwork(file: &str) -> String {
    crate::pictures::practice(file)
}

#[component]
pub fn Basic(directory: PathBuf, language: Language) -> Element {
    let path = directory.join("basic-pencil.json");
    let loaded = use_hook(|| state::load_local::<Progress>(&path).map_err(|e| e.to_string()));
    let mut progress = use_signal(|| loaded.clone().unwrap_or_default());
    let mut failed = use_signal(|| false);
    let mut page_list = use_signal(|| false);
    let persist = use_callback(move |next: Progress| {
        failed.set(state::save_local(&path, &next).is_err());
        progress.set(next);
    });
    let position = progress.read().position.min(SHEETS.len() - 1);
    let sheet = &SHEETS[position];
    rsx! {
        section { class: "basic-module",
            h1 { {language.text("बुनियादी अभ्यास", "Basic")} }
            if loaded.is_err() {
                p { role: "alert", {language.text("सहेजा अभ्यास नहीं खुला। वापस जाकर फिर कोशिश करें।", "Saved practice could not be opened. Go back and try again.")} }
            } else {
                p { {language.text("पेंसिल नियंत्रण", "Pencil control")} " · {position+1} / {SHEETS.len()} · {progress.read().completed.len()} " {language.text("पूरे", "complete")} }
                if failed() {
                    p { role: "alert", {language.text("अभ्यास सहेजा नहीं गया।", "Practice could not be saved.")} }
                    button { onclick: move |_| persist.call(progress()), {language.text("फिर सहेजें", "Retry save")} }
                }
                nav { class: "actions",
                    button { onclick: move |_| page_list.toggle(), {language.text("सभी पन्ने", "All pages")} }
                    button { onclick: move |_| { let mut n=progress(); n.paused=!n.paused; persist.call(n); }, {language.text("आराम / जारी रखें", "Break / Continue")} }
                }
                if page_list() {
                    div { class: "practice-page-grid",
                        for page in 1..=14 {
                            button { onclick: move |_| {
                                let mut n=progress(); n.position=SHEETS.iter().position(|s| s.page==page).unwrap(); n.paused=false; persist.call(n); page_list.set(false);
                            }, "{language.text(\"पन्ना\", \"Page\")} {page}" }
                        }
                    }
                } else if progress.read().paused {
                    h2 { {language.text("थोड़ा आराम करें", "Take a short break")} }
                    button { class: "primary", onclick: move |_| { let mut n=progress(); n.paused=false; persist.call(n); }, {language.text("जारी रखें", "Continue")} }
                } else {
                  div { key: "pencil-band-{position}",
                    div { class: "visual-instruction",
                        crate::pictures::Image { id: "pencil".to_string(), language }
                        p { {language.text("शुरू के निशान से पथ पर धीरे चलें। उँगली से बटन छुएँ, पेन से लिखें।", "Start at the marker and follow the path slowly. Tap controls with a finger; draw with the pen.")} }
                    }
                    h2 { "{language.text(\"पन्ना\", \"Page\")} {sheet.page}" if sheet.row>0 { " · {language.text(\"अभ्यास पंक्ति\", \"Practice band\")} {sheet.row}" } }
                    PencilCanvas { src: artwork(&sheet.file),
                        initial: progress.read().strokes.get(&position).cloned().unwrap_or_default(), language,
                        on_edit: move |strokes| { let mut n=progress(); n.strokes.insert(position,strokes); persist.call(n); } }
                    details { summary { {language.text("पूरा पन्ना देखें", "Full-page reference")} }
                        img { class: "pencil-reference", src: artwork(&format!("pencil-{}.jpg",sheet.page)), alt: language.text("पूरा पेंसिल अभ्यास पन्ना", "Full pencil-control worksheet") }
                    }
                    nav { class: "actions",
                        button { disabled: position==0, onclick: move |_| { let mut n=progress(); n.position=position-1; persist.call(n); }, {language.text("पिछला", "Previous")} }
                        button { class: "primary", aria_pressed: progress.read().completed.contains(&position), onclick: move |_| {
                            let mut n=progress(); if !n.completed.insert(position) { n.completed.remove(&position); } persist.call(n);
                        }, {if progress.read().completed.contains(&position) { language.text("✓ पूरा", "✓ Complete") } else { language.text("पूरा चिह्नित करें", "Mark complete") }} }
                        button { disabled: position+1==SHEETS.len(), onclick: move |_| { let mut n=progress(); n.position=position+1; persist.call(n); }, {language.text("अगला", "Next")} }
                    }
                  }
                }
            }
        }
    }
}

#[component]
fn PencilCanvas(
    src: String,
    initial: Vec<Stroke>,
    language: Language,
    on_edit: EventHandler<Vec<Stroke>>,
) -> Element {
    let mut zoom = use_signal(|| 100);
    let mut pan = use_signal(|| false);
    use_effect(move || {
        let config = serde_json::json!({"initial":initial,"allowMouse":!cfg!(target_os="android")});
        let mut eval = document::eval(&format!(
            "const config={config};\n{}",
            include_str!("pencil.js")
        ));
        spawn(async move {
            while let Ok(strokes) = eval.recv::<Vec<Stroke>>().await {
                on_edit.call(strokes);
            }
        });
    });
    rsx! {
        div { id: "pencil-workspace",
            nav { class: "actions",
                button { "data-pencil-action": "undo", {language.text("पिछला निशान हटाएँ", "Undo stroke")} }
                button { "data-pencil-action": "clear", {language.text("इस अभ्यास को साफ करें", "Clear exercise")} }
                button { aria_pressed: pan(), onclick: move |_| pan.toggle(), {if pan() { language.text("पेन से लिखें", "Draw with pen") } else { language.text("पन्ना खिसकाएँ", "Move page") }} }
                button { disabled: zoom()<=100, onclick: move |_| zoom.set(zoom()-50), "−" }
                span { "{zoom()}%" }
                button { disabled: zoom()>=250, onclick: move |_| zoom.set(zoom()+50), "+" }
            }
            div { class: "pencil-scroll",
                div { class: "pencil-surface", style: "width: {zoom()}%;",
                    img { src, draggable: "false", alt: language.text("पेंसिल से पथ पर चलें", "Trace the pencil-control path") }
                    canvas { style: if pan() { "pointer-events: none;" } else { "touch-action: none;" }, aria_label: language.text("पेंसिल अभ्यास कैनवास", "Pencil practice canvas") }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_source_pages_and_bands_exist() {
        assert_eq!(SHEETS.len(), 68);
        for page in 1..=14 {
            assert_eq!(
                SHEETS.iter().filter(|s| s.page == page).count(),
                if page <= 2 {
                    7
                } else if page <= 8 {
                    8
                } else {
                    1
                }
            );
        }
        for s in SHEETS.iter() {
            assert!(!artwork(&s.file).is_empty());
        }
    }
    #[test]
    fn saves_strokes_completion_and_pause() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("basic.json");
        let mut p = Progress::default();
        p.position = 4;
        p.paused = true;
        p.completed.insert(2);
        p.strokes.insert(4, vec![vec![[0.2, 0.3], [0.5, 0.6]]]);
        state::save_local(&path, &p).unwrap();
        let r: Progress = state::load_local(&path).unwrap();
        assert!(r.paused);
        assert_eq!(r.position, 4);
        assert_eq!(r.strokes[&4], p.strokes[&4]);
        assert!(r.completed.contains(&2));
    }
}
