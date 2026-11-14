mod aba;
mod activity;
mod authoring;
mod content;
mod language;
mod math_help;
mod parent;
mod platform;
mod progress;
mod state;
mod writing;

use activity::Activity;
use content::{CustomCatalog, Kind, PracticeSkill, Topic};
use dioxus::prelude::*;
use language::Language;
use progress::{Outcome, Step};
use state::{Session, State};
use std::path::PathBuf;
use writing::PaperScale;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Subjects,
    Aba,
    Subject,
    Group,
    Generated,
    Session,
    Break,
    Done(bool),
    Parent(bool),
}

#[component]
fn App() -> Element {
    let storage = use_hook(|| {
        platform::state_path()
            .and_then(|path| Ok((state::load(&path)?, path)))
            .map_err(|error| error.to_string())
    });
    let Ok((initial, path)) = storage else {
        return rsx! { main { h1 { "Saksham" } p { "Saved progress could not be opened. Restart the app to try again." } } };
    };
    rsx! { LearningApp { path, initial: StateInput(initial) } }
}

#[derive(Clone)]
struct StateInput(State);
impl PartialEq for StateInput {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[component]
fn LearningApp(path: PathBuf, initial: StateInput) -> Element {
    let mut data = use_signal(|| initial.0);
    let mut page = use_signal(|| Page::Subjects);
    let mut subject_id = use_signal(|| content::BUNDLED[0].id.clone());
    let mut grade = use_signal(|| 2u8);
    let mut group_id = use_signal(String::new);
    let mut active = use_signal(|| None::<(String, bool)>);
    let mut save_error = use_signal(|| false);
    use_effect(platform::landscape);

    let learner_path = path.clone();
    let commit = use_callback(move |next: State| {
        save_error.set(state::save(&learner_path, &next).is_err());
        data.set(next);
    });
    let calibration_path = path.clone();
    let save_calibration = use_callback(move |scale: PaperScale| {
        let mut next = data.peek().clone();
        next.scale = Some(scale.horizontal);
        next.vertical_scale = Some(scale.vertical);
        state::save(&calibration_path, &next).map_err(|error| error.to_string())?;
        data.set(next);
        save_error.set(false);
        Ok(())
    });
    let publish = use_callback(move |catalog: CustomCatalog| {
        let result =
            state::save_catalog(&path, &data.peek(), catalog).map_err(|error| error.to_string());
        match result {
            Ok(next) => {
                data.set(next);
                save_error.set(false);
                Ok(())
            }
            Err(error) => Err(error),
        }
    });
    let start = use_callback(move |(topic, review): (String, bool)| {
        let subject = subject_id.peek().clone();
        let mut next = data.peek().clone();
        let key = state::session_key(&subject, &topic, review);
        if let std::collections::btree_map::Entry::Vacant(entry) = next.sessions.entry(key) {
            entry.insert(Session::new(subject, topic.clone(), review));
            commit.call(next);
        }
        active.set(Some((topic, review)));
        page.set(Page::Session);
    });
    let start_generated = use_callback(move |skill: PracticeSkill| {
        let mut next = data.peek().clone();
        if let Some(id) = next.start_generated(grade(), skill, rand::random()) {
            commit.call(next);
            active.set(Some((id, false)));
            page.set(Page::Session);
        }
    });

    let snapshot = data.read();
    let selected = subject_id();
    let subject = snapshot
        .catalog
        .subjects()
        .find(|item| item.id == selected)
        .unwrap_or(&content::BUNDLED[0]);
    let math_selected = subject.id == "math";
    let aba_selected = page() == Page::Aba;
    let aba_language = snapshot.aba_language.unwrap_or(Language::English);
    let aba_name = aba_language.text("एबीए सीखें", "ABA Learning");
    let language = if matches!(page(), Page::Aba | Page::Parent(true)) {
        aba_language
    } else if math_selected && page() != Page::Subjects {
        snapshot.math_language
    } else if subject.id == "en" && page() != Page::Subjects {
        Language::English
    } else {
        Language::Hindi
    };
    let subject_name = if math_selected {
        language.text("गणित", "Mathematics").to_owned()
    } else {
        subject.name.clone()
    };
    let other_language = match language {
        Language::Hindi => Language::English,
        Language::English => Language::Hindi,
    };
    let language_label = other_language.text("हिंदी", "English");
    let grade_label = language.text("कक्षा", "Class");
    let recall_label = language.text("फिर याद करो", "Recall");
    let practice_label = language.text("अभ्यास", "Practice");
    let new_practice_label = language.text("नया अभ्यास", "New practice");
    let back_to_subject = match language {
        Language::Hindi => format!("{subject_name} पर लौटो"),
        Language::English => format!("Back to {subject_name}"),
    };
    let stars = snapshot.finished.len();
    let topics: Vec<_> = snapshot
        .catalog
        .topics(subject)
        .map(|topic| {
            if math_selected && language == Language::English {
                content::english_math_topic(topic)
            } else {
                topic
            }
        })
        .collect();
    let grades: std::collections::BTreeSet<_> = topics.iter().map(|topic| topic.grade).collect();
    let mut groups = Vec::new();
    for topic in topics.iter().filter(|topic| topic.grade == grade()) {
        if !groups
            .iter()
            .any(|group: &&content::Group| group.id == topic.group.id)
        {
            groups.push(&topic.group);
        }
    }
    let mut due: Vec<_> = topics
        .iter()
        .filter_map(|topic| {
            let review =
                snapshot
                    .reviews
                    .get(&format!("{}/{}", subject.id, topic.progress_id()))?;
            (topic.grade == grade() && review.due <= state::day()).then_some((review.due, *topic))
        })
        .collect();
    due.sort_by_key(|(day, _)| *day);
    due.truncate(2);
    let group = groups.iter().find(|group| group.id == group_id());
    let active_step = active().and_then(|(id, review)| {
        let session = snapshot
            .sessions
            .get(&state::session_key(&subject.id, &id, review))?;
        let topic = if let Some(set) = &session.generated {
            set.topic(language)
        } else {
            *topics.iter().find(|topic| topic.progress_id() == id)?
        };
        Some((topic, session))
    });
    let generated_round =
        math_selected && active().is_some_and(|(id, _)| id.starts_with("new-practice-"));

    let body = match page() {
        Page::Subjects => rsx! {
            h1 { "अपना विषय चुनो" }
            div { class: "cards",
                for item in snapshot.catalog.subjects() {
                    {
                        let id = item.id.clone();
                        let default_grade = snapshot.catalog.topics(item).map(|topic| topic.grade).max();
                        let (name, lang) = if id == "math" {
                            (snapshot.math_language.text("गणित", "Mathematics"), snapshot.math_language.code())
                        } else if id == "en" {
                            (item.name.as_str(), Language::English.code())
                        } else {
                            (item.name.as_str(), "hi")
                        };
                        rsx! {
                            button { key: "{id}", lang, disabled: default_grade.is_none(),
                                onclick: move |_| {
                                    subject_id.set(id.clone());
                                    if let Some(level) = default_grade { grade.set(level); }
                                    page.set(Page::Subject);
                                },
                                "{name}"
                                if default_grade.is_none() { small { "जल्द आएगा" } }
                            }
                        }
                    }
                }
                button { lang: aba_language.code(), onclick: move |_| page.set(Page::Aba), "{aba_name}" }
            }
        },
        Page::Aba => rsx! {
            aba::AbaLearning { language, name: aba_name, data, commit }
        },
        Page::Subject => rsx! {
            h1 { "{subject_name}" }
            div { class: "cards", aria_label: language.text("कक्षा चुनो", "Choose a class"),
                for level in grades {
                    button { key: "{level}", class: if grade() == level { "primary" } else { "" },
                        aria_pressed: grade() == level, onclick: move |_| grade.set(level), "{grade_label} {level}" }
                }
            }
            if math_selected {
                button { class: "primary", onclick: move |_| page.set(Page::Generated), "{new_practice_label}" }
            }
            if !due.is_empty() {
                h2 { "{recall_label}" }
                div { class: "cards",
                    for (_, topic) in due {
                        { topic_button(&snapshot, &subject.id, topic, true, start, language) }
                    }
                }
            }
            for group in groups.iter().copied() {
                if group.separate_page {
                    {
                        let id = group.id.clone();
                        rsx! { button { key: "{id}", class: "primary", onclick: move |_| {
                            group_id.set(id.clone()); page.set(Page::Group);
                        }, "{group.title}" } }
                    }
                } else {
                    section { key: "{group.id}",
                        h2 { "{group.title}" }
                        if !group.guidance.is_empty() { p { "{group.guidance}" } }
                        div { class: "cards",
                            for topic in topics.iter().filter(|topic| topic.grade == grade() && topic.group.id == group.id) {
                                { topic_button(&snapshot, &subject.id, topic, false, start, language) }
                            }
                        }
                    }
                }
            }
        },
        Page::Group => rsx! {
            if let Some(group) = group {
                h1 { "{group.title}" }
                p { "{group.guidance}" }
                div { class: "cards",
                    for topic in topics.iter().filter(|topic| topic.grade == grade() && topic.group.id == group.id) {
                        { topic_button(&snapshot, &subject.id, topic, false, start, language) }
                    }
                }
            }
        },
        Page::Generated => rsx! {
            h1 { "{new_practice_label}" }
            div { class: "cards",
                for &skill in PracticeSkill::for_grade(grade()) {
                    {
                        let id = skill.id(grade());
                        let resume = snapshot.sessions.contains_key(&state::session_key("math", &id, false));
                        rsx! {
                            button { key: "{id}", onclick: move |_| start_generated.call(skill),
                                {skill.label(language)}
                                if resume { small { {language.text("जारी रखो", "Resume")} } }
                            }
                        }
                    }
                }
            }
        },
        Page::Session => {
            if let Some((topic, session)) = active_step {
                let subject = subject.id.clone();
                let topic = topic.clone();
                let index = session.step;
                let review = session.review;
                let step = Step {
                    subject: &subject,
                    topic: &topic,
                    review,
                    index,
                };
                let exercise = step.exercise();
                let recall = exercise.is_none();
                let kind = exercise.cloned().unwrap_or_else(|| {
                    let questions = step.recall(&snapshot);
                    Kind::Choice(
                        questions[if review {
                            index.min(questions.len() - 1)
                        } else {
                            0
                        }]
                        .clone(),
                    )
                });
                let ink = step.ink(&snapshot);
                let revision_source = step
                    .revision_source(&snapshot)
                    .map(|(ink, lines)| (ink.clone(), lines));
                let helper_draft = session.helper.clone().unwrap_or_default();
                let activity_key = format!("{}-{review}-{index}", step.key());
                let total = topic.exercises.len();
                let title = topic.title.clone();
                let scale = PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale);
                rsx! {
                  section { key: "{activity_key}",
                    h1 { "{title}" }
                    if recall { h2 { "{recall_label}" } } else { p { class: "step", "{practice_label} {index + 1} / {total}" } }
                    Activity { kind, initial: ink, helper_draft, scale, recall, language,
                        ruled_paper: content::Paper::for_subject(&subject), revision_source,
                        on_input: move |input| {
                            let mut next = data.peek().clone();
                            let step = Step { subject: &subject, topic: &topic, review, index };
                            if let Some(outcome) = step.apply(&mut next, input, state::day(), &state::sample_date()) {
                                commit.call(next);
                                match outcome {
                                    Outcome::Continue => {}
                                    Outcome::Break => page.set(Page::Break),
                                    Outcome::Done(awarded) => page.set(Page::Done(awarded)),
                                }
                            }
                        }
                    }
                    button { class: "quiet", onclick: move |_| page.set(Page::Break), {language.text("आराम", "Take a break")} }
                  }
                }
            } else {
                rsx! { button { onclick: move |_| page.set(Page::Subject), "{back_to_subject}" } }
            }
        }
        Page::Break => rsx! {
            h1 { {language.text("थोड़ा आराम", "Take a short break")} }
            button { class: "primary", onclick: move |_| page.set(Page::Session), {language.text("अभ्यास जारी रखो", "Continue practice")} }
        },
        Page::Done(awarded) => rsx! {
            h1 { {language.text("अभ्यास पूरा", "Practice complete")} }
            if awarded { p { class: "reward", {language.text("★ एक सितारा", "★ One star")} } }
            if generated_round {
                button { class: "primary", onclick: move |_| page.set(Page::Generated), "{new_practice_label}" }
            } else {
                button { class: "primary", onclick: move |_| page.set(Page::Subject), "{back_to_subject}" }
            }
        },
        Page::Parent(_) => rsx! { parent::Parent { data, commit, publish, save_calibration } },
    };
    drop(snapshot);

    rsx! {
        document::Title { "Saksham" }
        style { {include_str!("style.css")} }
        div { class: "app", lang: language.code(),
            header {
                if matches!(page(), Page::Subject | Page::Aba) {
                    button { class: "quiet", onclick: move |_| page.set(Page::Subjects), {language.text("‹ विषय", "‹ Subjects")} }
                } else if page() == Page::Parent(true) {
                    button { class: "quiet", onclick: move |_| page.set(Page::Aba), "‹ {aba_name}" }
                } else if page() != Page::Subjects {
                    button { class: "quiet", onclick: move |_| page.set(Page::Subject), "⌂ {subject_name}" }
                }
                strong { {language.text("सक्षम", "Saksham")} }
                span { class: "stars", "★ {stars}" }
                if aba_selected || (math_selected && !matches!(page(), Page::Subjects | Page::Parent(_))) {
                    button { lang: other_language.code(), onclick: move |_| {
                        let mut next = data.peek().clone();
                        if aba_selected {
                            next.aba_language = Some(other_language);
                        } else {
                            next.math_language = other_language;
                        }
                        commit.call(next);
                    }, "{language_label}" }
                }
                if !matches!(page(), Page::Parent(_)) {
                    button { class: "quiet", onclick: move |_| page.set(Page::Parent(page() == Page::Aba)), {language.text("बड़ों के लिए", "For parents")} }
                }
            }
            main {
                if save_error() {
                    p { role: "alert", "अभ्यास सहेजा नहीं जा सका। · Progress could not be saved." }
                    button { onclick: move |_| { let next = data.peek().clone(); commit.call(next); }, {language.text("फिर सहेजें", "Try saving again")} }
                }
                {body}
            }
        }
    }
}

fn topic_button(
    state: &State,
    subject: &str,
    topic: &Topic,
    review: bool,
    start: Callback<(String, bool)>,
    language: Language,
) -> Element {
    let id = topic.progress_id();
    let paused = state
        .sessions
        .contains_key(&state::session_key(subject, &id, review));
    let finished = !review && state.finished.contains(&format!("{subject}/{id}"));
    rsx! {
        button { key: "{id}", onclick: move |_| start.call((id.clone(), review)),
            "{topic.title}"
            if paused { small { {language.text("जारी रखो", "Resume")} } }
            else if finished { small { {language.text("फिर अभ्यास करो", "Practise again")} } }
        }
    }
}

#[cfg(test)]
mod regression;
