use crate::{
    content::Paper,
    keypad::{Keys, NumericField},
    language::Language,
    math_help::parse_numbers,
    state::{self, Ink},
    writing::{InkPad, PaperScale},
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf, sync::LazyLock};

#[derive(Deserialize)]
struct Sheet {
    id: String,
    subject: String,
    grade: u8,
    pack: String,
    title: String,
    #[serde(default)]
    adult: String,
    tasks: Vec<Task>,
}

#[derive(Deserialize)]
struct Task {
    prompt: String,
    #[serde(default)]
    picture: Option<String>,
    #[serde(default)]
    context: String,
    #[serde(default)]
    hint: String,
    #[serde(default)]
    key: String,
    #[serde(flatten)]
    action: Action,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Action {
    Choice { options: Vec<String>, correct: usize },
    Order { options: Vec<String>, order: Vec<usize> },
    Number { answer: String },
    Trace { model: String },
    Write,
    Draw,
    Adult,
}

static SHEETS: LazyLock<Vec<Sheet>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("content/worksheets.json"))
        .expect("bundled worksheets are valid")
});

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
struct Progress {
    position: usize,
    drafts: BTreeMap<usize, Draft>,
}

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
struct Draft {
    ink: Ink,
    taps: Vec<usize>,
    input: String,
    checked: bool,
}

fn text_language(text: &str, fallback: Language) -> Language {
    if text.chars().any(|ch| ('\u{0900}'..='\u{097f}').contains(&ch)) {
        Language::Hindi
    } else if text.chars().any(|ch| ch.is_ascii_alphabetic()) {
        Language::English
    } else {
        fallback
    }
}

#[component]
pub fn Worksheets(
    directory: PathBuf,
    subject: String,
    grade: u8,
    language: Language,
    scale: PaperScale,
) -> Element {
    let mut active = use_signal(|| None::<usize>);
    let title = language.text("कार्यपत्रक", "Worksheets");
    if let Some(index) = active() {
        let path = directory.join(format!("worksheet-{}.json", SHEETS[index].id));
        return rsx! {
            Practice { key: "{index}", index, path, language, scale,
                on_back: move |_| active.set(None) }
        };
    }
    let mut packs = Vec::new();
    for sheet in SHEETS.iter().filter(|sheet| sheet.subject == subject && sheet.grade == grade) {
        if !packs.contains(&sheet.pack.as_str()) {
            packs.push(sheet.pack.as_str());
        }
    }
    rsx! {
        section { class: "learner-page",
            h1 { "{title}" }
            p { "{language.text(\"कक्षा\", \"Class\")} {grade}" }
            for pack in packs {
                h2 { "{pack}" }
                div { class: "subject-cards worksheet-list",
                    for (index, sheet) in SHEETS.iter().enumerate().filter(|(_, sheet)|
                        sheet.subject == subject && sheet.grade == grade && sheet.pack == pack) {
                        button { key: "{sheet.id}", class: "subject-card",
                            onclick: move |_| active.set(Some(index)), "{sheet.title}" }
                    }
                }
            }
        }
    }
}

#[component]
fn Practice(
    index: usize,
    path: PathBuf,
    language: Language,
    scale: PaperScale,
    on_back: EventHandler<()>,
) -> Element {
    let sheet = &SHEETS[index];
    let saved = use_hook(|| state::load_local::<Progress>(&path).map_err(|error| error.to_string()));
    let mut progress = use_signal(|| saved.clone().unwrap_or_default());
    let mut failed = use_signal(|| false);
    let mut paused = use_signal(|| false);
    let mut pending_answer = use_signal(|| None::<String>);
    let persist = use_callback(move |next: Progress| {
        failed.set(state::save_local(&path, &next).is_err());
        progress.set(next);
    });
    let position = progress.read().position;
    let advance = move |_| {
        let current = progress.peek();
        let mut next = current.clone();
        drop(current);
        next.position += 1;
        pending_answer.set(None);
        persist.call(next);
    };
    let edit = move |draft: Draft| {
        let mut next = progress.peek().clone();
        next.drafts.insert(position, draft);
        persist.call(next);
    };
    rsx! {
        section { class: "learner-page worksheet-practice",
            nav { class: "actions",
                button { class: "quiet", onclick: move |_| on_back.call(()),
                    {language.text("‹ सभी कार्यपत्रक", "‹ All worksheets")}
                }
                if !paused() && position < sheet.tasks.len() && saved.is_ok() {
                    button { class: "quiet", onclick: move |_| paused.set(true),
                        {language.text("थोड़ा आराम", "Take a break")}
                    }
                }
            }
            h1 { "{sheet.title}" }
            if failed() {
                p { role: "alert", {language.text("काम सहेजा नहीं गया।", "Your work could not be saved.")} }
                button { onclick: move |_| persist.call(progress()), {language.text("फिर सहेजें", "Try saving again")} }
            }
            if saved.is_err() {
                p { role: "alert", {language.text("सहेजा हुआ काम नहीं खुला।", "Your saved work could not be opened.")} }
            } else if paused() {
                h2 { {language.text("थोड़ा आराम करो", "Take a short break")} }
                button { class: "primary", onclick: move |_| paused.set(false),
                    {language.text("जारी रखो", "Continue")}
                }
            } else if let Some(task) = sheet.tasks.get(position) {
                div { key: "{sheet.id}-{position}",
                    p { class: "worksheet-position",
                        {match language {
                            Language::Hindi => format!("{} में से {}", sheet.tasks.len(), position + 1),
                            Language::English => format!("{} of {}", position + 1, sheet.tasks.len()),
                        }}
                    }
                    Exercise { sheet: index, position, language, scale,
                        draft: progress.read().drafts.get(&position).cloned().unwrap_or_default(),
                        on_edit: edit, pending_answer,
                        on_next: advance,
                    }
                    if !sheet.adult.is_empty() || !task.key.is_empty() {
                        details { class: "learner-help",
                            summary { {language.text("बड़ों के लिए", "For grown-ups")} }
                            if !sheet.adult.is_empty() { p { "{sheet.adult}" } }
                            if !task.key.is_empty() { p { "{task.key}" } }
                        }
                    }
                }
            } else {
                h2 { {language.text("कार्यपत्रक पूरा", "Worksheet complete")} }
                button { class: "primary", onclick: move |_| {
                    let mut next = progress.peek().clone();
                    next.position = 0;
                    next.drafts.clear();
                    persist.call(next);
                }, {language.text("फिर से अभ्यास करो", "Practise again")} }
            }
        }
    }
}

#[component]
fn Exercise(
    sheet: usize,
    position: usize,
    language: Language,
    scale: PaperScale,
    draft: Draft,
    pending_answer: Signal<Option<String>>,
    on_edit: EventHandler<Draft>,
    on_next: EventHandler<()>,
) -> Element {
    let sheet = &SHEETS[sheet];
    let task = &sheet.tasks[position];
    let prompt_language = text_language(&task.prompt, language);
    let (verb, complete, missed) = match &task.action {
        Action::Choice { correct, .. } => (
            language.text("छुओ", "Tap"),
            draft.taps.last() == Some(correct),
            !draft.taps.is_empty() && draft.taps.last() != Some(correct),
        ),
        Action::Order { order, .. } => (
            language.text("क्रम से छुओ", "Tap in order"),
            draft.taps.len() == order.len() && !draft.checked,
            draft.checked,
        ),
        Action::Number { answer } => (
            language.text("उत्तर लिखो", "Enter the answer"),
            draft.checked && same_number(&draft.input, answer),
            draft.checked && !same_number(&draft.input, answer),
        ),
        Action::Trace { .. } => (language.text("रेखा पर लिखो", "Trace"), false, false),
        Action::Write => (language.text("लिखो", "Write"), false, false),
        Action::Draw => (language.text("बनाओ", "Draw"), false, false),
        Action::Adult => (language.text("साथ में करो", "Do it together"), false, false),
    };
    let can_continue = complete || matches!(&task.action, Action::Adult)
        || (matches!(&task.action, Action::Trace { .. } | Action::Write | Action::Draw)
            && draft.ink.iter().any(|cell| !cell.is_empty()));
    let hint = if task.hint.is_empty() {
        match &task.action {
            Action::Order { .. } => language.text("अगला शब्द या अंक सोचो।", "Think about what comes next."),
            Action::Number { .. } => language.text("अंकों को फिर देखो।", "Look at the numbers again."),
            Action::Choice { .. } => language.text("संकेत और विकल्प फिर देखो।", "Look at the question and the choices again."),
            _ => language.text("ज़रूरत हो तो बड़ों की मदद लो।", "Ask a grown-up to show you an example."),
        }
    } else {
        task.hint.as_str()
    };
    rsx! {
        div { class: "worksheet-task",
            p { class: "worksheet-action", "{verb}" }
            if !task.context.is_empty() { p { class: "worksheet-context", lang: text_language(&task.context, language).code(), "{task.context}" } }
            h2 { lang: prompt_language.code(), "{task.prompt}" }
            if let Some(id) = &task.picture { crate::pictures::Image { id: id.clone(), language } }
            details { class: "learner-help",
                summary { {language.text("मदद", "Help")} }
                p { "{hint}" }
            }
            match &task.action {
                Action::Choice { options, .. } => rsx! {
                    div { class: "cards worksheet-choices",
                        for (index, option) in options.iter().enumerate() {
                            {
                                let mut next = draft.clone();
                                next.taps = vec![index];
                                rsx! { button { key: "{index}", aria_pressed: draft.taps.last() == Some(&index),
                                    disabled: complete, onclick: move |_| on_edit.call(next.clone()), "{option}" } }
                            }
                        }
                    }
                },
                Action::Order { options, order } => rsx! {
                    if !draft.taps.is_empty() {
                        p { class: "worksheet-context",
                            {draft.taps.iter().filter_map(|&index| options.get(index).map(String::as_str)).collect::<Vec<_>>().join(" → ")}
                        }
                    }
                    div { class: "cards worksheet-choices",
                        for (index, option) in options.iter().enumerate() {
                            {
                                let mut next = draft.clone();
                                next.checked = order.get(next.taps.len()) != Some(&index);
                                if !next.checked { next.taps.push(index); }
                                rsx! { button { key: "{index}", disabled: draft.taps.contains(&index) || complete,
                                    onclick: move |_| on_edit.call(next.clone()), "{option}" } }
                            }
                        }
                    }
                },
                Action::Number { answer } => {
                    let mut next = draft.clone();
                    let answer = answer.clone();
                    let number = pending_answer().unwrap_or_else(|| draft.input.clone());
                    let empty = number.trim().is_empty();
                    let submitted = number.clone();
                    let mut pending = pending_answer;
                    rsx! {
                        form { onsubmit: move |event| {
                            event.prevent_default();
                            if !empty && !complete {
                                next.input.clone_from(&submitted);
                                next.checked = true;
                                on_edit.call(next.clone());
                            }
                        },
                            NumericField { label: language.text("उत्तर", "Answer").to_owned(), value: number,
                                language, limit: 28, keys: if answer.contains(',') { Keys::List } else { Keys::Whole },
                                disabled: complete, on_change: move |value| pending.set(Some(value)) }
                            if !complete {
                                button { r#type: "submit", disabled: empty, {language.text("जाँचो", "Check")} }
                            }
                        }
                    }
                },
                Action::Trace { model } => {
                    let paper = if prompt_language == Language::Hindi { Paper::Hindi } else if model.bytes().all(|byte| byte.is_ascii_digit()) { Paper::Digit } else { Paper::English };
                    let mut next = draft.clone();
                    rsx! {
                        InkPad { id: "worksheet-ink", model: model.clone(), initial: draft.ink.clone(), scale,
                            paper, language, on_ink: move |ink| { next.ink = ink; on_edit.call(next.clone()); } }
                    }
                },
                Action::Write | Action::Draw => {
                    let mut next = draft.clone();
                    rsx! {
                        InkPad { id: "worksheet-ink", model: String::new(), initial: draft.ink.clone(), scale,
                            paper: Paper::Blank, language,
                            on_ink: move |ink| { next.ink = ink; on_edit.call(next.clone()); } }
                    }
                },
                Action::Adult => rsx! {},
            }
            if complete {
                p { class: "worksheet-feedback", role: "status",
                    {language.text("✓ सही।", "✓ That’s right.")}
                }
            } else if missed {
                p { class: "worksheet-feedback", role: "status",
                    {language.text("फिर कोशिश करो।", "Try again.")} " {hint}"
                }
            }
            button { class: "primary", disabled: !can_continue, onclick: move |_| on_next.call(()),
                {language.text("जारी रखो", "Continue")}
            }
        }
    }
}

fn same_number(input: &str, answer: &str) -> bool {
    parse_numbers(input).is_some_and(|value| Some(value) == parse_numbers(answer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn hindi_worksheet_directions_are_in_hindi() {
        let translation_prompts = [
            "🐘 Elephant", "🐐 Goat", "🐦 Bird", "🐟 Fish", "🦁 Lion",
            "↑ North", "← West", "→ East", "↓ South",
        ];
        for sheet in SHEETS.iter().filter(|sheet| sheet.subject == "hi") {
            let directions = [&*sheet.title, &*sheet.adult].into_iter().chain(
                sheet.tasks.iter().flat_map(|task| {
                    [&*task.prompt, &*task.context, &*task.hint]
                }).filter(|text| !translation_prompts.contains(text)),
            );
            for text in directions {
                assert!(!text.chars().any(|ch| ch.is_ascii_alphabetic()), "{}: {text}", sheet.id);
            }
        }
    }

    #[test]
    fn all_supplied_packs_have_unique_sheets_and_valid_tasks() {
        let mut ids = HashSet::new();
        let mut packs = BTreeSet::new();
        for sheet in SHEETS.iter() {
            assert!(ids.insert(&sheet.id), "duplicate sheet {}", sheet.id);
            assert!((1..=2).contains(&sheet.grade));
            assert!(matches!(sheet.subject.as_str(), "hi" | "en" | "math"));
            assert!(!sheet.title.is_empty() && !sheet.tasks.is_empty());
            packs.insert((&sheet.subject, &sheet.pack));
            for task in &sheet.tasks {
                assert!(!task.prompt.is_empty(), "empty task in {}", sheet.id);
                if let Some(id)=&task.picture { assert!(crate::pictures::card(id).is_some()); }
                match &task.action {
                    Action::Choice { options, correct } => {
                        assert!(options.len() >= 2 && *correct < options.len());
                        assert_eq!(options.len(), options.iter().collect::<HashSet<_>>().len());
                    }
                    Action::Order { options, order } => {
                        assert_eq!(order.iter().copied().collect::<BTreeSet<_>>(), (0..options.len()).collect());
                    }
                    Action::Number { answer } => assert!(same_number(answer, answer)),
                    Action::Trace { model } => assert!(!model.trim().is_empty()),
                    _ => {}
                }
            }
        }
        assert_eq!(packs.len(), 6);
        assert_eq!(ids.len(), 150);
        for subject in ["hi", "math", "en"] {
            for grade in [1, 2] {
                assert_eq!(SHEETS.iter().filter(|sheet| sheet.subject == subject && sheet.grade == grade)
                    .map(|sheet| &sheet.pack).collect::<HashSet<_>>().len(), 2);
            }
        }
    }
}

