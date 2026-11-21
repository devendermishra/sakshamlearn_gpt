use crate::{
    content::{self, Kind, Paper},
    language::Language,
    state::{self, Ink},
    writing::{InkPad, PaperScale},
};
use dioxus::prelude::*;
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Clone, Copy, PartialEq)]
enum Set {
    Words,
    Sentences,
    Numbers,
    Addition,
    Table(u8),
}

impl Set {
    fn title(self, language: Language) -> String {
        match self {
            Self::Words => language.text("रोज़ के शब्द", "Common words").into(),
            Self::Sentences => language.text("छोटे वाक्य", "Short sentences").into(),
            Self::Numbers => language.text("संख्याएँ", "Numbers").into(),
            Self::Addition => language.text("जोड़", "Addition").into(),
            Self::Table(table) => match language {
                Language::Hindi => format!("{table} का पहाड़ा"),
                Language::English => format!("{table} times table"),
            },
        }
    }
}

struct Model {
    id: String,
    text: String,
}

fn models(subject: &str, grade: u8, set: Set) -> Vec<Model> {
    match set {
        Set::Words => {
            let mut seen = BTreeSet::new();
            content::BUNDLED
                .iter()
                .filter(|item| item.id == subject)
                .flat_map(|item| &item.topics)
                .filter(|topic| topic.grade == grade && topic.group.id == "everyday-words")
                .flat_map(|topic| {
                    topic.exercises.iter().filter_map(|exercise| {
                        if let Kind::Trace { model, .. } = &exercise.kind {
                            Some(Model {
                                id: format!("{}-{}", topic.id, exercise.id),
                                text: model.clone(),
                            })
                        } else {
                            None
                        }
                    })
                })
                .filter(|model| seen.insert(model.text.clone()))
                .collect()
        }
        Set::Sentences => {
            let sentences: &[&str] = match (subject, grade) {
                ("hi", 1) => &[
                    "यह मेरा घर है।",
                    "माँ रोटी बनाती हैं।",
                    "यह लाल सेब है।",
                    "मैं पानी पीता हूँ।",
                    "मेरे पास किताब है।",
                    "मैं स्कूल जाता हूँ।",
                ],
                ("hi", _) => &[
                    "मैं अपने दोस्त के साथ खेलती हूँ।",
                    "नीमा ने एक कहानी पढ़ी।",
                    "हमने पेड़ को पानी दिया।",
                    "मेरा बस्ता मेज़ के पास है।",
                    "पहले पढ़ो, फिर लिखो।",
                    "आज आकाश में बादल हैं।",
                ],
                ("en", 1) => &[
                    "This is my bag.",
                    "I drink water.",
                    "I see a red ball.",
                    "We play at home.",
                    "This is my book.",
                    "I like apples.",
                ],
                _ => &[
                    "My book is on the table.",
                    "I play with my friend.",
                    "Please give me a pencil.",
                    "We water the plants.",
                    "First read, then write.",
                    "The sky is full of clouds.",
                ],
            };
            sentences
                .iter()
                .enumerate()
                .map(|(index, text)| Model {
                    id: format!("sentence-{index:02}"),
                    text: (*text).into(),
                })
                .collect()
        }
        Set::Numbers => (0..=if grade == 1 { 20 } else { 100 })
            .map(|value| Model {
                id: format!("number-{value}"),
                text: value.to_string(),
            })
            .collect(),
        Set::Addition => {
            let pairs: &[(u16, u16)] = if grade == 1 {
                &[
                    (1, 2),
                    (2, 3),
                    (3, 4),
                    (4, 4),
                    (5, 4),
                    (6, 4),
                    (8, 4),
                    (5, 9),
                    (9, 7),
                    (10, 10),
                ]
            } else {
                &[
                    (12, 5),
                    (17, 8),
                    (23, 14),
                    (28, 7),
                    (36, 18),
                    (47, 16),
                    (59, 12),
                    (65, 24),
                    (48, 37),
                    (56, 44),
                ]
            };
            pairs
                .iter()
                .map(|&(left, right)| Model {
                    id: format!("add-{left}-{right}"),
                    text: format!("{left}+{right}={}", left + right),
                })
                .collect()
        }
        Set::Table(table) => (1u8..=10)
            .map(|right| Model {
                id: format!("table-{table}-{right}"),
                text: format!("{table}×{right}={}", u16::from(table) * u16::from(right)),
            })
            .collect(),
    }
}

#[component]
pub fn TracingPractice(
    subject: String,
    grade: u8,
    directory: PathBuf,
    language: Language,
    scale: PaperScale,
) -> Element {
    let mut set = use_signal(|| None::<Set>);
    let mut position = use_signal(|| 0usize);
    let math = subject == "math";
    let paper = if math {
        Paper::Digit
    } else {
        Paper::for_subject(&subject)
    };
    let groups: &[Set] = if math {
        &[Set::Numbers, Set::Addition]
    } else {
        &[Set::Words, Set::Sentences]
    };
    let choose_set = use_callback(move |group| {
        position.set(0);
        set.set(Some(group));
    });
    let title = language.text("बिंदियों पर लिखो", "Tracing Practice");
    rsx! {
        section { class: "learner-page tracing-practice",
            if let Some(group) = set() {
                {
                    let models = models(&subject, grade, group);
                    let index = position();
                    let total = models.len();
                    let heading = group.title(language);
                    let choose = language.text("नमूना चुनो", "Choose a model");
                    let current = index.to_string();
                    rsx! {
                        button { class: "quiet", onclick: move |_| set.set(None),
                            {language.text("‹ सभी लिखाई अभ्यास", "‹ All tracing practice")}
                        }
                        h1 { "{heading}" }
                        if let Set::Table(size) = group {
                            crate::activity::TableConcept { size, language }
                            TableTraceSheet { size, grade, scale, language, directory: directory.clone() }
                        } else if let Some(model) = models.get(index) {
                            label { "{choose}"
                                select { value: "{current}", onchange: move |event| {
                                    if let Ok(next) = event.value().parse::<usize>() {
                                        if next < total { position.set(next); }
                                    }
                                },
                                    for (option, model) in models.iter().enumerate() {
                                        option { key: "{model.id}", value: "{option}", "{model.text}" }
                                    }
                                }
                            }
                            p { class: "worksheet-position",
                                {match language {
                                    Language::Hindi => format!("{total} में से {}", index + 1),
                                    Language::English => format!("{} of {total}", index + 1),
                                }}
                            }
                            {rsx! {
                                TraceModel { key: "{model.id}", model: model.text.clone(), paper, scale, language,
                                    path: directory.join(format!("trace-{subject}-{grade}-{}.json", model.id)) }
                            }}
                            div { class: "actions",
                                button { disabled: index == 0, onclick: move |_| position.set(index.saturating_sub(1)),
                                    {language.text("पिछला", "Previous")}
                                }
                                button { class: "primary", disabled: index + 1 >= total, onclick: move |_| position.set(index + 1),
                                    {language.text("अगला", "Next")}
                                }
                            }
                        }
                    }
                }
            } else {
                h1 { "{title}" }
                div { class: "cards",
                    for group in groups.iter().copied() {
                        button { onclick: move |_| choose_set.call(group), {group.title(language)} }
                    }
                }
                if math {
                    h2 { {language.text("पहाड़े", "Times tables")} }
                    div { class: "cards",
                        for table in 1..=if grade == 1 { 5 } else { 10 } {
                            button { key: "{table}", onclick: move |_| choose_set.call(Set::Table(table)),
                                { Set::Table(table).title(language) }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TableTraceSheet(size: u8, grade: u8, scale: PaperScale, language: Language, directory: PathBuf) -> Element {
    let path = directory.join(format!("trace-math-{grade}-table-{size}-sheet.json"));
    let (mut draft, initial) = crate::persistence::use_loaded(|| load_table_sheet(&directory, &path, grade, size).map_err(|error| error.to_string()));
    let failed = use_signal(|| false);
    let save_ink = crate::persistence::use_saver(path.clone(), failed);
    let save = use_callback(move |ink: Ink| {
        save_ink.call(ink.clone());
        draft.set(ink);
    });
    rsx! {
        section { class: "table-sheet table-tracing-sheet",
            p { {language.text("बाईं ओर बिंदियों पर लिखो। दाईं ओर खुद लिखो। हर पूरी संख्या एक खाने में लिखो।", "Trace on the left. Write on your own on the right. Keep each whole number in one box.")} }
            if initial.is_ok() {
                InkPad { id: "table-trace-ink", model: "", initial: draft(), paper: Paper::Digit, scale, language,
                    lines: 10, table_trace: Some(size), on_ink: move |ink| save.call(ink) }
                if failed() {
                    p { role: "alert", {language.text("लिखाई सहेजी नहीं गई।", "Your writing could not be saved.")} }
                    button { onclick: move |_| save.call(draft()), {language.text("फिर सहेजें", "Try saving again")} }
                }
            } else {
                p { role: "alert", {language.text("सहेजी लिखाई नहीं खुली।", "Your saved writing could not be opened.")} }
            }
        }
    }
}

fn load_table_sheet(directory: &std::path::Path, path: &std::path::Path, grade: u8, size: u8) -> std::io::Result<Ink> {
    crate::persistence::flush()?;
    let old_sheet: Option<Ink> = if path.exists() { Some(state::load_local(path)?) } else { None };
    if let Some(sheet) = &old_sheet {
        if sheet.len() == 100 { return Ok(sheet.clone()); }
        if sheet.len() != 20 * format!("{size}×10={}", size * 10).chars().count() {
            return Err(std::io::Error::other("Unexpected table sheet size"));
        }
    }
    let mut ink = vec![Vec::new(); 100];
    for groups in 1..=10 {
        let tokens = [size.to_string(), "×".into(), groups.to_string(), "=".into(), (size * groups).to_string()];
        let (previous, old_width) = if let Some(sheet) = &old_sheet {
            let width = sheet.len() / 20;
            let start = (usize::from(groups) - 1) * 2 * width;
            (sheet[start..start + 2 * width].to_vec(), width)
        } else {
            (state::load_local::<Ink>(&directory.join(format!("trace-math-{grade}-table-{size}-{groups}.json")))?,
             tokens.iter().map(|token| token.chars().count()).sum())
        };
        for side in 0..2 {
            let mut column = 0;
            for (token_index, token) in tokens.iter().enumerate() {
                let digits = token.chars().count();
                let target = &mut ink[((usize::from(groups) - 1) * 2 + side) * 5 + token_index];
                for digit in 0..digits {
                    if let Some(cell) = previous.get(side * old_width + column + digit) {
                        target.extend(cell.iter().map(|stroke| stroke.iter().map(|point|
                            [(digit as f32 + point[0]) / digits as f32, point[1]]
                        ).collect()));
                    }
                }
                column += digits;
            }
        }
    }
    Ok(ink)
}

#[component]
fn TraceModel(
    #[props(default = "practice-trace-ink".to_string())] id: String,
    model: String,
    paper: Paper,
    scale: PaperScale,
    language: Language,
    path: PathBuf,
) -> Element {
    let (mut draft, initial) = crate::persistence::use_loaded(|| state::load_local::<Ink>(&path).map_err(|error| error.to_string()));
    let failed = use_signal(|| false);
    let save_ink = crate::persistence::use_saver(path.clone(), failed);
    let save = use_callback(move |ink: Ink| {
        save_ink.call(ink.clone());
        draft.set(ink);
    });
    rsx! {
        section { class: "trace-model",
            h2 { "{model}" }
            if initial.is_ok() {
                InkPad { id, model, initial: draft(), paper, scale, language, single_trace: true,
                    on_ink: move |ink| save.call(ink) }
                if failed() {
                    p { role: "alert", {language.text("लिखाई सहेजी नहीं गई।", "Your writing could not be saved.")} }
                    button { onclick: move |_| save.call(draft()), {language.text("फिर सहेजें", "Try saving again")} }
                }
            } else {
                p { role: "alert", {language.text("सहेजी लिखाई नहीं खुली।", "Your saved writing could not be opened.")} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_sheet_preserves_old_trace_and_copy_rows_and_reopens_saved_sheet() {
        let directory = tempfile::tempdir().unwrap();
        let stroke = vec![vec![[0.2, 0.3], [0.6, 0.7]]];
        let mut old: Ink = vec![Vec::new(); 10];
        old[0] = stroke.clone();
        old[9] = stroke.clone();
        state::save_local(&directory.path().join("trace-math-2-table-3-1.json"), &old).unwrap();
        let path = directory.path().join("trace-math-2-table-3-sheet.json");
        let sheet = load_table_sheet(directory.path(), &path, 2, 3).unwrap();
        assert_eq!(sheet.len(), 100);
        assert_eq!(sheet[0], stroke);
        assert_eq!(sheet[9], stroke);
        assert_eq!(sheet.iter().filter(|cell| !cell.is_empty()).count(), 2);
        state::save_local(&path, &sheet).unwrap();
        assert_eq!(load_table_sheet(directory.path(), &path, 2, 3).unwrap(), sheet);
    }

    #[test]
    fn table_sheet_merges_saved_digits_into_whole_number_boxes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("trace-math-2-table-10-sheet.json");
        let mut old: Ink = vec![Vec::new(); 180];
        for index in [162, 163, 168, 169, 170] {
            old[index] = vec![vec![[0.2, 0.3], [0.6, 0.7]]];
        }
        state::save_local(&path, &old).unwrap();
        let sheet = load_table_sheet(directory.path(), &path, 2, 10).unwrap();
        assert_eq!(sheet.len(), 100);
        assert_eq!(sheet[90].len(), 2);
        assert_eq!(sheet[90][0][0], [0.1, 0.3]);
        assert_eq!(sheet[90][1][0], [0.6, 0.3]);
        assert_eq!(sheet[94].len(), 3);
        assert!(sheet[94][0][0][0] < sheet[94][1][0][0]);
        assert!(sheet[94][1][0][0] < sheet[94][2][0][0]);
    }

    #[test]
    fn catalog_covers_both_classes_and_full_tables() {
        for subject in ["hi", "en"] {
            for grade in [1, 2] {
                assert!(models(subject, grade, Set::Words).len() >= 20);
                assert_eq!(models(subject, grade, Set::Sentences).len(), 6);
            }
        }
        assert_eq!(models("math", 1, Set::Numbers).len(), 21);
        assert_eq!(models("math", 2, Set::Numbers).last().unwrap().text, "100");
        for table in 1..=10 {
            let facts = models("math", 2, Set::Table(table));
            assert_eq!(facts.len(), 10);
            assert_eq!(facts[9].text, format!("{table}×10={}", table * 10));
        }
        assert_eq!(
            models("math", 2, Set::Addition).last().unwrap().text,
            "56+44=100"
        );
    }
}
