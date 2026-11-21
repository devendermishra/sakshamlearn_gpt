mod aba;
mod daily_exercises;
mod appearance;
mod basic;
mod pictures;
mod activity;
mod authoring;
mod content;
mod curriculum;
mod focus;
mod language;
mod keypad;
mod math_help;
mod parent;
mod platform;
mod progress;
mod persistence;
mod state;
mod writing;
mod worksheets;
mod rough_sheet;
mod tracing_practice;

use activity::Activity;
use content::{CustomCatalog, Kind, PracticeSkill, Topic};
use dioxus::prelude::*;
use language::Language;
use progress::{Outcome, Step};
use state::{Session, State};
use std::path::PathBuf;
use writing::PaperScale;

fn handwriting_font() -> &'static str {
    static FONT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let bytes = include_bytes!("../assets/PatrickHand-Regular.ttf");
        let hindi = include_bytes!("../assets/Kalam-Regular.ttf");
        format!("/* {} */\n@font-face {{ font-family: 'Patrick Hand'; src: url(data:font/ttf;base64,{}) format('truetype'); font-display: swap; }}\n@font-face {{ font-family: 'Kalam'; src: url(data:font/ttf;base64,{}) format('truetype'); font-display: swap; }}", include_str!("../assets/Kalam-OFL.txt"), STANDARD.encode(bytes), STANDARD.encode(hindi))
    });
    &FONT
}

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Subjects,
    DailyExercises,
    Aba,
    Basic,
    Subject,
    Group,
    Trace,
    DailyRecall,
    RoughSheet,
    TracingPractice,
    Worksheets,
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
    let mut jump = use_signal(|| None::<(String, usize)>);
    let mut active = use_signal(|| None::<(String, bool)>);
    let mut save_error = use_signal(|| false);
    use_effect(platform::landscape);
    let focus_controls = focus::Controls::use_controls();
    let pin_path = path.clone();
    let save_pin = use_callback(move |pin: state::Pin| {
        let mut next = data.peek().clone();
        if next.pin.is_none() {
            next.pin = Some(pin);
            state::save(&pin_path, &next).map_err(|error| error.to_string())?;
            data.set(next);
        }
        Ok(())
    });
    let local_folder = path.with_file_name("learner-work");

    let save_progress = persistence::use_saver(path.clone(), save_error);
    let commit = use_callback(move |next: State| {
        save_progress.call(next.clone());
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
    let chapters = matches!(subject.id.as_str(), "hi" | "en");
    let handwriting = subject.id == "en";
    let aba_selected = page() == Page::Aba;
    let aba_language = snapshot.aba_language.unwrap_or(Language::English);
    let aba_name = aba_language.text("एबीए सीखें", "ABA Learning");
    let language = if matches!(page(), Page::Aba | Page::Basic | Page::Parent(true)) {
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
    let mut topics: Vec<_> = snapshot
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
    topics.sort_by_key(|topic| curriculum::order(&subject.id, topic));
    let grades: std::collections::BTreeSet<_> = topics.iter().map(|topic| topic.grade).collect();
    let mut groups = Vec::new();
    for topic in topics.iter().filter(|topic| topic.grade == grade()) {
        let (id, title) = curriculum::chapter(&subject.id, topic);
        if !groups.iter().any(|(existing, _, _, _)| *existing == id) {
            groups.push((id, title, topic.group.guidance.as_str(), topic.group.separate_page));
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
    let group = groups.iter().find(|(id, _, _, _)| *id == group_id());
    let daily_key = format!("{}/{}", subject.id, grade());
    let today = state::india_date();
    let today_label = today.to_string();
    let daily = snapshot.daily_recall.get(&daily_key).filter(|round| round.date == today_label);
    let has_completed_topic = topics.iter().any(|topic| {
        topic.grade == grade() && snapshot.finished.contains(&format!("{}/{}", subject.id, topic.progress_id()))
    });
    let daily_label = language.text("रोज़ की दोहराई", "Daily recall");
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
        Page::DailyExercises => rsx! {
            daily_exercises::DailyExercises {
                directory: local_folder.clone(), math_language: snapshot.math_language,
                scale: PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale)
            }
        },
        Page::Subjects => rsx! {
            section { class: "home-cover",
                p { "कक्षा 1 और 2" }
                h1 { "अपना विषय चुनो" }
            }
            button { class: "primary daily-home", "data-subject": "daily-exercises",
                onclick: move |_| page.set(Page::DailyExercises), "60-day exercises · 60 दिन के अभ्यास" }
            div { class: "subject-cards home-subjects",
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
                            button { key: "{id}", class: "subject-card", "data-subject": "{id}", lang, disabled: default_grade.is_none(),
                                onclick: move |_| {
                                    subject_id.set(id.clone());
                                    if let Some(level) = default_grade { grade.set(level); }
                                    page.set(Page::Subject);
                                },
                                span { class: "subject-icon", aria_hidden: "true", {subject_icon(&id)} }
                                span { "{name}" }
                                if default_grade.is_none() { small { "जल्द आएगा" } }
                            }
                        }
                    }
                }
                button { class: "subject-card", "data-subject": "basic", lang: aba_language.code(), onclick: move |_| page.set(Page::Basic),
                    span { class: "subject-icon", aria_hidden: "true", "✎" }
                    span { {aba_language.text("बुनियादी अभ्यास", "Basic")} }
                }
                button { class: "subject-card", "data-subject": "aba", lang: aba_language.code(), onclick: move |_| page.set(Page::Aba),
                    span { class: "subject-icon", aria_hidden: "true", "◇" }
                    span { "{aba_name}" }
                }
            }
        },
        Page::Aba => rsx! {
            aba::AbaLearning { language, name: aba_name, data, commit, directory: local_folder.clone() }
        },
        Page::Basic => rsx! { basic::Basic { directory: local_folder.clone(), language } },
        Page::Subject => rsx! {
            h1 { class: "subject-heading",
                span { class: "subject-icon", aria_hidden: "true", {subject_icon(&selected)} }
                "{subject_name}"
            }
            if matches!(selected.as_str(), "hi" | "math" | "en") {
                div { class: "subject-cards subject-tools",
                    button { class: "subject-card", onclick: move |_| page.set(Page::RoughSheet),
                        span { class: "subject-icon", aria_hidden: "true", "✎" }
                        {language.text("रफ़ कॉपी", "Rough Sheet")}
                    }
                    button { class: "subject-card", onclick: move |_| page.set(Page::TracingPractice),
                        span { class: "subject-icon", aria_hidden: "true", "⋯" }
                        {language.text("बिंदियों पर लिखो", "Tracing Practice")}
                    }
                    button { class: "subject-card", onclick: move |_| page.set(Page::Worksheets),
                        span { class: "subject-icon", aria_hidden: "true", "▤" }
                        {language.text("कार्यपत्रक", "Worksheets")}
                    }
                }
            }
            div { class: "cards", aria_label: language.text("कक्षा चुनो", "Choose a class"),
                for level in grades {
                    button { key: "{level}", class: if grade() == level { "primary" } else { "" },
                        aria_pressed: grade() == level, onclick: move |_| grade.set(level), "{grade_label} {level}" }
                }
            }
            if math_selected {
                button { class: "primary", onclick: move |_| page.set(Page::Generated), "{new_practice_label}" }
            }
            if chapters || math_selected {
                {
                    let daily_key = daily_key.clone();
                    let id = subject.id.clone();
                    let selected_grade = grade();
                    rsx! { button { class: "primary", onclick: move |_| {
                        let mut next = data.peek().clone();
                        let date = state::india_date();
                        if has_completed_topic && !next.daily_recall.get(&daily_key).is_some_and(|round| round.date == date.to_string()) {
                            let subject = next.catalog.subjects().find(|subject| subject.id == id);
                            if let Some(subject) = subject {
                                let mut topics = next.catalog.topics(subject).filter(|topic| topic.grade == selected_grade).collect::<Vec<_>>();
                                topics.sort_by_key(|topic| curriculum::order(&id, topic));
                                let round = state::DailyRecall::new(&next, &id, &topics, date);
                                next.daily_recall.insert(daily_key.clone(), round);
                                commit.call(next);
                            }
                        }
                        page.set(Page::DailyRecall);
                    }, "{daily_label}"
                        if let Some(round) = daily {
                            small {
                                if round.position == round.questions.len() { {language.text("आज पूरा हुआ", "Done today")} }
                                else { {language.text("जारी रखो", "Resume")} }
                            }
                        }
                    } }
                }
            }
            if !due.is_empty() {
                h2 { "{recall_label}" }
                div { class: "cards",
                    for (_, topic) in due {
                        { topic_button(&snapshot, &subject.id, topic, true, start, language) }
                    }
                }
            }
            if chapters {
                h2 { {language.text("अध्याय", "Chapters")} }
                if grade() == 2 {
                    button { class: "quiet", onclick: move |_| {
                        grade.set(1);
                        group_id.set(if handwriting { "alphabet" } else { "vowels" }.into());
                        page.set(Page::Group);
                    }, {language.text("अक्षर और मात्राएँ · कक्षा 1", "Letter practice · Class 1")} }
                }
            }
            for (id, title, guidance, separate) in groups.iter().copied() {
                if chapters || separate {
                    {
                        let id = id.to_owned();
                        rsx! { button { key: "{id}", class: "primary", onclick: move |_| {
                            group_id.set(id.clone()); page.set(Page::Group);
                        }, "{title}" } }
                    }
                } else {
                    section { key: "{id}",
                        h2 { "{title}" }
                        if !guidance.is_empty() { p { "{guidance}" } }
                        div { class: "cards",
                            for topic in topics.iter().filter(|topic| topic.grade == grade() && curriculum::chapter(&subject.id, topic).0 == id) {
                                { topic_button(&snapshot, &subject.id, topic, false, start, language) }
                            }
                        }
                    }
                }
            }
        },
        Page::Group => rsx! {
            if let Some(&(id, title, guidance, _)) = group {
                h1 { "{title}" }
                if !chapters && !guidance.is_empty() { p { "{guidance}" } }
                if subject.id == "hi" && id == "matras" {
                    if topics.iter().any(|topic| topic.id == "barakhadi-1") {
                        div { class: "letter-picker", aria_label: "मात्रा चुनो",
                            for (index, mark) in ["", "◌ा", "◌ि", "◌ी", "◌ु", "◌ू", "◌े", "◌ै", "◌ो", "◌ौ", "◌ं", "◌ः"].into_iter().enumerate().skip(1) {
                                button { key: "{mark}", onclick: move |_| {
                                    jump.set(Some(("barakhadi-1".into(), index)));
                                    page.set(Page::Trace);
                                }, "{mark}" }
                            }
                            button { onclick: move |_| {
                                jump.set(Some(("ri-one".into(), 0)));
                                page.set(Page::Trace);
                            }, "◌ृ" }
                        }
                    }
                }
                for topic in topics.iter().filter(|topic| topic.grade == grade() && curriculum::chapter(&subject.id, topic).0 == id) {
                    section { key: "{topic.id}", class: "chapter-topic",
                        { topic_button(&snapshot, &subject.id, topic, false, start, language) }
                        div { class: "letter-picker",
                            for index in 0..topic.exercises.len() {
                                if let Some(model) = curriculum::letter_model(&subject.id, topic, index) {
                                    {
                                        let topic_id = topic.progress_id();
                                        rsx! { button { key: "{topic.exercises[index].id}",
                                            class: if handwriting { "handwriting-letter" } else { "" },
                                            onclick: move |_| { jump.set(Some((topic_id.clone(), index))); page.set(Page::Trace); },
                                            "{model}"
                                        } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Page::Trace => {
            if let Some(((id, index), topic)) = jump().and_then(|target| {
                topics.iter().find(|topic| topic.progress_id() == target.0).map(|topic| (target, (*topic).clone()))
            }) {
                let subject = subject.id.clone();
                let step = Step { subject: &subject, topic: &topic, review: false, index };
                let title = topic.title.clone();
                let ink = step.ink(&snapshot);
                let picture = topic.exercises.get(index).and_then(|e| e.picture.clone()).or_else(|| step.exercise().and_then(pictures::for_kind).map(str::to_string));
                let kind = step.exercise().cloned();
                let scale = PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale);
                rsx! {
                    h1 { "{title}" }
                    if let Some(kind) = kind {
                        Activity { key: "jump-{id}-{index}", picture, kind, initial: ink, scale, language, handwriting,
                            on_input: move |input| {
                                let mut next = data.peek().clone();
                                let step = Step { subject: &subject, topic: &topic, review: false, index };
                                if let Some(done) = step.trace_only(&mut next, input, &state::sample_date()) {
                                    commit.call(next);
                                    if done { page.set(Page::Group); }
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! { p { {language.text("अक्षर चुनो।", "Choose a letter.")} } }
            }
        }
        Page::DailyRecall => rsx! {
            h1 { "{daily_label}" }
            if let Some(round) = daily {
                if let Some(item) = round.questions.get(round.position) {
                    {
                        let position = round.position;
                        let date = round.date.clone();
                        let (title, question) = if math_selected && language == Language::English {
                            item.english.as_ref().map(|(title, question)| (title, question))
                                .unwrap_or((&item.title, &item.question))
                        } else { (&item.title, &item.question) };
                        let current = position + 1;
                        let total = round.questions.len();
                        rsx! { section { key: "daily-{date}-{position}",
                            p { class: "step", "{title} · {current} / {total}" }
                            Activity { kind: Kind::Choice(question.clone()),
                                initial: Vec::new(), scale: PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale),
                                recall: true, language, handwriting, on_input: move |input| {
                                    if !matches!(input, progress::Input::Answer(_)) { return; }
                                    let mut next = data.peek().clone();
                                    if let Some(round) = next.daily_recall.get_mut(&daily_key) {
                                        if round.date == date && round.position == position {
                                            round.position += 1;
                                            commit.call(next);
                                        }
                                    }
                                }
                            }
                        } }
                    }
                } else {
                    p { {language.text("आज की दोहराई पूरी हुई।", "Daily recall complete for today.")} }
                }
            } else {
                p { {language.text("रोज़ की दोहराई शुरू करने के लिए पहले इस कक्षा का एक विषय पूरा करो।", "Finish a topic in this class to unlock daily recall.")} }
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
                let picture = if recall { None } else { topic.exercises.get(index).and_then(|e| e.picture.clone()).or_else(|| pictures::for_kind(&kind).map(str::to_string)) };
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
                    Activity { picture, kind, initial: ink, helper_draft, scale, recall, language, handwriting,
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
        Page::RoughSheet => rsx! {
            rough_sheet::RoughSheet { key: "{selected}", path: local_folder.join(format!("rough-{selected}.json")),
                paper: if math_selected { content::Paper::Digit } else { content::Paper::for_subject(&selected) },
                language, scale: PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale) }
        },
        Page::TracingPractice => rsx! {
            tracing_practice::TracingPractice { key: "{selected}-{grade}", subject: selected.clone(),
                grade: grade(), directory: local_folder.clone(), language,
                scale: PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale) }
        },
        Page::Worksheets => rsx! {
            worksheets::Worksheets { key: "{selected}-{grade}", directory: local_folder.clone(),
                subject: selected.clone(), grade: grade(), language,
                scale: PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale) }
        },
        Page::Parent(_) => rsx! { parent::Parent { data, publish, save_calibration, save_pin, focus_controls } },
    };
    let pin = snapshot.pin.clone();
    let appearance = snapshot.appearance;
    let accent = if page() == Page::Subjects { "" } else { selected.as_str() };
    drop(snapshot);

    rsx! {
        document::Title { "Saksham" }
        style { {include_str!("style.css")} }
        style { {handwriting_font()} }
        style { {appearance::background_styles()} }
        div { class: "app", "data-subject": "{accent}", "data-palette": appearance.palette.id(),
            "data-cover": appearance.background.id(), lang: language.code(),
            header {
                if matches!(page(), Page::Subject | Page::Aba | Page::Basic | Page::DailyExercises) {
                    button { class: "quiet", onclick: move |_| page.set(Page::Subjects), {language.text("‹ विषय", "‹ Subjects")} }
                } else if page() == Page::Parent(true) {
                    button { class: "quiet", onclick: move |_| page.set(Page::Aba), "‹ {aba_name}" }
                } else if page() == Page::Trace {
                    button { class: "quiet", onclick: move |_| page.set(Page::Group), {language.text("‹ अध्याय", "‹ Chapter")} }
                } else if page() != Page::Subjects {
                    button { class: "quiet", onclick: move |_| page.set(Page::Subject), "⌂ {subject_name}" }
                }
                strong { {language.text("सक्षम", "Saksham")} }
                span { class: "stars", "★ {stars}" }
                appearance::AppearanceButton { language }
                if aba_selected || page() == Page::Basic || (math_selected && !matches!(page(), Page::Subjects | Page::Parent(_))) {
                    button { lang: other_language.code(), onclick: move |_| {
                        let mut next = data.peek().clone();
                        if aba_selected || page() == Page::Basic {
                            next.aba_language = Some(other_language);
                        } else {
                            next.math_language = other_language;
                        }
                        commit.call(next);
                    }, "{language_label}" }
                }
                if !matches!(page(), Page::Parent(_)) {
                    focus::FocusButton { controls: focus_controls, has_pin: pin.is_some(), language }
                    button { class: "quiet", onclick: move |_| page.set(Page::Parent(page() == Page::Aba)), {language.text("बड़ों के लिए", "For parents")} }
                }
            }
            appearance::AppearanceOptions { appearance, language,
                on_change: move |change| {
                    let mut next = data.peek().clone();
                    next.appearance.apply(change);
                    commit.call(next);
                }
            }
            main { class: if page() == Page::RoughSheet { "rough-notebook-main" } else { "" },
                if save_error() {
                    p { role: "alert", "अभ्यास सहेजा नहीं जा सका। · Progress could not be saved." }
                    button { onclick: move |_| { let next = data.peek().clone(); commit.call(next); }, {language.text("फिर सहेजें", "Try saving again")} }
                }
                focus::FocusPrompt { controls: focus_controls, pin, save_pin, language }
                {body}
            }
        }
    }
}

fn subject_icon(subject: &str) -> &'static str {
    match subject {
        "hi" => "क्ष",
        "math" => "1 2 3",
        "en" => "ABC",
        _ => "◇",
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
