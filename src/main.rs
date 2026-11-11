mod activity;
mod authoring;
mod content;
mod parent;
mod platform;
mod progress;
mod state;
mod writing;

use activity::Activity;
use content::{CustomCatalog, Kind, Topic};
use dioxus::prelude::*;
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
    Subject,
    Group,
    Session,
    Break,
    Done(bool),
    Parent,
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
            entry.insert(Session {
                subject,
                topic: topic.clone(),
                step: 0,
                review,
                missed: false,
            });
            commit.call(next);
        }
        active.set(Some((topic, review)));
        page.set(Page::Session);
    });

    let snapshot = data.read();
    let selected = subject_id();
    let subject = snapshot
        .catalog
        .subjects()
        .find(|item| item.id == selected)
        .unwrap_or(&content::BUNDLED[0]);
    let subject_name = subject.name.clone();
    let stars = snapshot.finished.len();
    let topics: Vec<_> = snapshot.catalog.topics(subject).collect();
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
        let topic = topics.iter().find(|topic| topic.progress_id() == id)?;
        let session = snapshot
            .sessions
            .get(&state::session_key(&subject.id, &id, review))?;
        Some((*topic, session))
    });

    let body = match page() {
        Page::Subjects => rsx! {
            h1 { "अपना विषय चुनो" }
            div { class: "cards",
                for item in snapshot.catalog.subjects() {
                    {
                        let id = item.id.clone();
                        let default_grade = snapshot.catalog.topics(item).map(|topic| topic.grade).max();
                        rsx! {
                            button { key: "{id}", disabled: default_grade.is_none(),
                                onclick: move |_| {
                                    subject_id.set(id.clone());
                                    if let Some(level) = default_grade { grade.set(level); }
                                    page.set(Page::Subject);
                                },
                                "{item.name}"
                                if default_grade.is_none() { small { "जल्द आएगा" } }
                            }
                        }
                    }
                }
            }
        },
        Page::Subject => rsx! {
            h1 { "{subject.name}" }
            div { class: "cards", aria_label: "कक्षा चुनो",
                for level in grades {
                    button { key: "{level}", class: if grade() == level { "primary" } else { "" },
                        aria_pressed: grade() == level, onclick: move |_| grade.set(level), "कक्षा {level}" }
                }
            }
            if !due.is_empty() {
                h2 { "फिर याद करो" }
                div { class: "cards",
                    for (_, topic) in due {
                        { topic_button(&snapshot, &subject.id, topic, true, start) }
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
                                { topic_button(&snapshot, &subject.id, topic, false, start) }
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
                        { topic_button(&snapshot, &subject.id, topic, false, start) }
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
                let activity_key = format!("{}-{review}-{index}", step.key());
                let total = topic.exercises.len();
                let title = topic.title.clone();
                let scale = PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale);
                rsx! {
                  section { key: "{activity_key}",
                    h1 { "{title}" }
                    if recall { h2 { "फिर याद करो" } } else { p { class: "step", "अभ्यास {index + 1} / {total}" } }
                    Activity { kind, initial: ink, scale, recall,
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
                    button { class: "quiet", onclick: move |_| page.set(Page::Break), "आराम" }
                  }
                }
            } else {
                rsx! { button { onclick: move |_| page.set(Page::Subject), "विषय पर लौटो" } }
            }
        }
        Page::Break => rsx! {
            h1 { "थोड़ा आराम" }
            button { class: "primary", onclick: move |_| page.set(Page::Session), "अभ्यास जारी रखो" }
        },
        Page::Done(awarded) => rsx! {
            h1 { "अभ्यास पूरा" }
            if awarded { p { class: "reward", "★ एक सितारा" } }
            button { class: "primary", onclick: move |_| page.set(Page::Subject), "{subject_name} पर लौटो" }
        },
        Page::Parent => rsx! { parent::Parent { data, commit, publish, save_calibration } },
    };
    drop(snapshot);

    rsx! {
        document::Title { "Saksham" }
        style { {include_str!("style.css")} }
        div { class: "app", lang: "hi",
            header {
                if page() == Page::Subject {
                    button { class: "quiet", onclick: move |_| page.set(Page::Subjects), "‹ विषय" }
                } else if page() != Page::Subjects {
                    button { class: "quiet", onclick: move |_| page.set(Page::Subject), "⌂ {subject_name}" }
                }
                strong { "सक्षम" }
                span { class: "stars", "★ {stars}" }
                if page() != Page::Parent {
                    button { class: "quiet", onclick: move |_| page.set(Page::Parent), "बड़ों के लिए" }
                }
            }
            main {
                if save_error() {
                    p { role: "alert", "अभ्यास सहेजा नहीं जा सका। · Progress could not be saved." }
                    button { onclick: move |_| { let next = data.peek().clone(); commit.call(next); }, "फिर सहेजें" }
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
) -> Element {
    let id = topic.progress_id();
    let paused = state
        .sessions
        .contains_key(&state::session_key(subject, &id, review));
    let finished = !review && state.finished.contains(&format!("{subject}/{id}"));
    rsx! {
        button { key: "{id}", onclick: move |_| start.call((id.clone(), review)),
            "{topic.title}"
            if paused { small { "जारी रखो" } } else if finished { small { "फिर अभ्यास करो" } }
        }
    }
}

#[cfg(test)]
mod regression;
