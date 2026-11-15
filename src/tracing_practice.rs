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
                        if let Some(model) = models.get(index) {
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
fn TraceModel(
    model: String,
    paper: Paper,
    scale: PaperScale,
    language: Language,
    path: PathBuf,
) -> Element {
    let initial = use_hook(|| state::load_local::<Ink>(&path).map_err(|error| error.to_string()));
    let mut draft = use_signal(|| initial.clone().unwrap_or_default());
    let mut failed = use_signal(|| false);
    let save = use_callback(move |ink: Ink| {
        failed.set(state::save_local(&path, &ink).is_err());
        draft.set(ink);
    });
    rsx! {
        section { class: "trace-model",
            h2 { "{model}" }
            if let Ok(initial) = initial {
                InkPad { id: "practice-trace-ink", model, initial, paper, scale, language, single_trace: true,
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
