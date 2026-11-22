//! Offline, subject-only curriculum. Source IDs, rather than catalog positions, identify work.
use crate::{
    activity::{Activity, ActivityDraft, ActivityMemory},
    basic,
    content::{self, Kind, Paper, Topic},
    curriculum,
    language::Language,
    progress::{Input, Outcome, Step},
    state::{self, Ink, Session, State, Stroke},
    worksheets,
    writing::PaperScale,
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::LazyLock,
};

const VERSION: u32 = 1;
const SUBJECTS: [&str; 4] = ["en", "hi", "math", "basic"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Source {
    Lesson {
        topic: String,
        index: usize,
        recall: bool,
    },
    Worksheet {
        sheet: String,
        index: usize,
    },
    Basic {
        index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Task {
    id: String,
    subject: String,
    grade: u8,
    goal: String,
    source_label: String,
    source: Source,
}

impl Task {
    fn number(&self) -> usize {
        match self.source {
            Source::Lesson { index, .. }
            | Source::Worksheet { index, .. }
            | Source::Basic { index } => index + 1,
        }
    }
}

#[derive(Serialize)]
struct Day {
    number: usize,
    tasks: Vec<Task>,
}

#[derive(Serialize)]
struct Manifest {
    version: u32,
    days: Vec<Day>,
}

fn topic(subject: &str, id: &str, language: Language) -> &'static Topic {
    let topic = content::BUNDLED
        .iter()
        .find(|s| s.id == subject)
        .unwrap()
        .topics
        .iter()
        .find(|t| t.id == id)
        .unwrap();
    if subject == "math" && language == Language::English {
        content::english_math_topic(topic)
    } else {
        topic
    }
}

fn lesson_units(subject: &str, grade: u8) -> Vec<Vec<Task>> {
    let bundled = content::BUNDLED.iter().find(|s| s.id == subject).unwrap();
    let mut topics: Vec<_> = bundled.topics.iter().filter(|t| t.grade == grade).collect();
    topics.sort_by_key(|t| curriculum::order(subject, t));
    let mut units: Vec<Vec<Task>> = Vec::new();
    for t in topics {
        for (index, exercise) in t.exercises.iter().enumerate() {
            let task = Task {
                id: format!("lesson/{subject}/{}/{}/{}", t.id, t.revision, exercise.id),
                subject: subject.into(),
                grade,
                goal: t.title.clone(),
                source_label: format!(
                    "Lesson · {} · {}",
                    curriculum::chapter(subject, t).1,
                    t.title
                ),
                source: Source::Lesson {
                    topic: t.id.clone(),
                    index,
                    recall: false,
                },
            };
            // A revision needs the composition before it. Never split this chain across days.
            if matches!(exercise.kind, Kind::Revise(..)) {
                units
                    .last_mut()
                    .expect("revision follows writing")
                    .push(task);
            } else {
                units.push(vec![task]);
            }
        }
        for index in 0..t.recall.len() {
            units.push(vec![Task {
                id: format!("recall/{subject}/{}/{}/{index}", t.id, t.revision),
                subject: subject.into(),
                grade,
                goal: t.title.clone(),
                source_label: format!("Recall · {}", t.title),
                source: Source::Lesson {
                    topic: t.id.clone(),
                    index,
                    recall: true,
                },
            }]);
        }
    }
    for sheet in worksheets::SHEETS
        .iter()
        .filter(|s| s.subject == subject && s.grade == grade)
    {
        for index in 0..sheet.tasks.len() {
            units.push(vec![Task {
                id: format!("worksheet/{}/{index}", sheet.id),
                subject: subject.into(),
                grade,
                goal: sheet.title.clone(),
                source_label: format!("{} · {}", sheet.pack, sheet.title),
                source: Source::Worksheet {
                    sheet: sheet.id.clone(),
                    index,
                },
            }]);
        }
    }
    units
}

fn build_manifest() -> Manifest {
    let mut days: Vec<_> = (1..=60)
        .map(|number| Day {
            number,
            tasks: vec![],
        })
        .collect();
    for grade in [1, 2] {
        for subject in ["en", "hi", "math"] {
            let units = lesson_units(subject, grade);
            assert!(units.len() >= 30, "each class needs 30 practice units");
            let mut remaining: usize = units.iter().map(Vec::len).sum();
            let mut position = 0;
            for day in 0..30 {
                let days_left = 30 - day;
                let target = remaining.div_ceil(days_left);
                let mut count = 0;
                while position < units.len()
                    && (count == 0 || count < target)
                    && units.len() - position >= days_left
                {
                    let unit = &units[position];
                    days[(grade as usize - 1) * 30 + day]
                        .tasks
                        .extend(unit.iter().cloned());
                    count += unit.len();
                    position += 1;
                }
                remaining -= count;
            }
            assert_eq!(position, units.len());
        }
    }
    for (index, sheet) in basic::SHEETS.iter().enumerate() {
        let day = index * 60 / basic::SHEETS.len();
        days[day].tasks.push(Task {
            id: format!("basic/{}/{}", sheet.page, sheet.row),
            subject: "basic".into(),
            grade: if day < 30 { 1 } else { 2 },
            goal: "Pencil control · पेंसिल नियंत्रण".into(),
            source_label: format!("Basic · Page {} · Band {}", sheet.page, sheet.row),
            source: Source::Basic { index },
        });
    }
    Manifest {
        version: VERSION,
        days,
    }
}

static PLAN: LazyLock<Manifest> = LazyLock::new(build_manifest);

#[derive(Clone, Serialize, Deserialize)]
struct Progress {
    #[serde(skip)]
    render_revision: u64,
    version: u32,
    day: usize,
    active: Option<String>,
    paused: bool,
    #[serde(default)]
    attempt: u32,
    completed: BTreeSet<String>,
    #[serde(default)]
    retrying: BTreeSet<String>,
    worksheets: BTreeMap<String, worksheets::Draft>,
    pencil: BTreeMap<String, Vec<Stroke>>,
    lessons: State,
    #[serde(default)]
    answers: BTreeMap<String, ActivityDraft>,
    #[serde(default)]
    helpers: BTreeMap<String, state::HelperDraft>,
}

// Every edit increments this transient revision, including edits to nested lesson State.
impl PartialEq for Progress {
    fn eq(&self, other: &Self) -> bool {
        self.render_revision == other.render_revision
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            render_revision: 0,
            version: VERSION,
            day: 1,
            active: None,
            paused: false,
            attempt: 0,
            completed: BTreeSet::new(),
            retrying: BTreeSet::new(),
            worksheets: BTreeMap::new(),
            pencil: BTreeMap::new(),
            lessons: State::default(),
            answers: BTreeMap::new(),
            helpers: BTreeMap::new(),
        }
    }
}

impl Progress {
    fn day_complete(&self, day: &Day) -> bool {
        day.tasks.iter().all(|t| self.completed.contains(&t.id))
    }
    fn next_day(&self) -> usize {
        PLAN.days
            .iter()
            .find(|d| !self.day_complete(d))
            .map_or(60, |d| d.number)
    }
    fn advance(&mut self, task: &Task) {
        self.completed.insert(task.id.clone());
        self.retrying.remove(&task.id);
        let tasks = &PLAN.days[self.day - 1].tasks;
        let start = tasks.iter().position(|t| t.id == task.id).unwrap() + 1;
        // Stay within the selected subject; the day overview is always the return point.
        self.active = tasks[start..]
            .iter()
            .find(|t| t.subject == task.subject && !self.completed.contains(&t.id))
            .map(|t| t.id.clone());
        self.paused = false;
    }

    fn retry(&mut self, task: &Task) {
        self.retrying.insert(task.id.clone());
        self.answers.remove(&task.id);
        self.helpers.remove(&task.id);
        self.worksheets.remove(&task.id);
        self.pencil.remove(&task.id);
        if let Source::Lesson {
            topic: id,
            index,
            recall: false,
        } = &task.source
        {
            let topic = topic(&task.subject, id, Language::Hindi);
            let step = Step {
                subject: &task.subject,
                topic,
                index: *index,
                review: false,
            };
            self.lessons.sessions.remove(&step.session_key());
            self.lessons
                .samples
                .remove(&format!("{}/{}", step.key(), topic.exercises[*index].id));
        }
        self.attempt += 1;
        self.paused = false;
    }

    fn lesson_ink(&self, task: &Task, step: &Step<'_>) -> Ink {
        let draft = step.ink(&self.lessons);
        if draft.iter().any(|cell| !cell.is_empty())
            || !self.completed.contains(&task.id)
            || self.retrying.contains(&task.id)
        {
            return draft;
        }
        self.lessons
            .history
            .get(&format!(
                "{}/{}",
                step.key(),
                step.topic.exercises[step.index].id
            ))
            .and_then(|samples| samples.last())
            .map_or(draft, |sample| sample.ink.clone())
    }

    fn lesson_input(&mut self, task: &Task, input: Input, language: Language) -> bool {
        let Source::Lesson {
            topic: id,
            index,
            recall: false,
        } = &task.source
        else {
            return false;
        };
        let topic = topic(&task.subject, id, language);
        let step = Step {
            subject: &task.subject,
            topic,
            index: *index,
            review: false,
        };
        let draft = input.is_draft();
        if let Input::Helper(ref helper) = input {
            self.helpers.insert(task.id.clone(), helper.clone());
        }
        let key = step.session_key();
        let session = self
            .lessons
            .sessions
            .entry(key.clone())
            .or_insert_with(|| Session::new(task.subject.clone(), topic.progress_id(), false));
        session.step = *index;
        session.helper = self.helpers.get(&task.id).cloned();
        if let Some(outcome) = step.apply(
            &mut self.lessons,
            input,
            state::day(),
            &state::sample_date(),
        ) {
            if !draft
                && self
                    .lessons
                    .sessions
                    .get(&key)
                    .is_none_or(|s| s.step > *index)
            {
                self.advance(task);
            }
            if outcome == Outcome::Break && self.active.is_some() {
                self.paused = true;
            }
            true
        } else {
            false
        }
    }
}

fn load(path: &Path) -> Result<Progress, String> {
    let progress = state::load_local::<Progress>(path).map_err(|e| e.to_string())?;
    if progress.version != VERSION {
        return Err(
            "This saved plan uses a different version. Its work has been preserved.".into(),
        );
    }
    if !(1..=60).contains(&progress.day) {
        return Err("Saved day is invalid. Its work has been preserved.".into());
    }
    let ids: BTreeSet<_> = PLAN
        .days
        .iter()
        .flat_map(|d| d.tasks.iter().map(|t| t.id.as_str()))
        .collect();
    if progress
        .completed
        .iter()
        .any(|id| !ids.contains(id.as_str()))
        || progress.active.as_ref().is_some_and(|id| {
            !PLAN.days[progress.day - 1]
                .tasks
                .iter()
                .any(|t| &t.id == id)
        })
    {
        return Err(
            "Saved task references do not match this plan. Its work has been preserved.".into(),
        );
    }
    Ok(progress)
}

fn subject_name(subject: &str) -> &'static str {
    match subject {
        "en" => "English",
        "hi" => "हिंदी",
        "math" => "Mathematics · गणित",
        _ => "Basic · बुनियादी अभ्यास",
    }
}

#[component]
pub fn DailyExercises(directory: PathBuf, math_language: Language, scale: PaperScale) -> Element {
    let path = directory.join("daily-exercises.json");
    let (mut progress, loaded) = crate::persistence::use_loaded(|| load(&path));
    let error = loaded.as_ref().err().cloned();
    let writable = loaded.is_ok();
    let failed = use_signal(|| false);
    let save_progress = crate::persistence::use_saver(path.clone(), failed);
    let persist = use_callback(move |mut next: Progress| {
        // No writes after a load error, including from delayed canvas events.
        if writable {
            next.render_revision = progress.peek().render_revision + 1;
            save_progress.call(next.clone());
            progress.set(next);
        }
    });
    if let Some(error) = error {
        return rsx! { section { h1 { "60-day exercises · 60 दिन के अभ्यास" }
        p { role: "alert", "Saved practice could not be opened. सहेजा अभ्यास नहीं खुला। {error}" } } };
    }
    let current = progress.read();
    let day = &PLAN.days[current.day - 1];
    let grade = if day.number <= 30 { 1 } else { 2 };
    let active = current
        .active
        .as_ref()
        .and_then(|id| day.tasks.iter().find(|t| &t.id == id))
        .cloned();
    let finished_days = PLAN.days.iter().filter(|d| current.day_complete(d)).count();
    rsx! {
        section { class: "daily-pane", "data-plan-version": "{VERSION}",
            h1 { "60-day exercises · 60 दिन के अभ्यास" }
            p { "{finished_days} / 60 days complete · दिन पूरे" }
            if failed() {
                p { role: "alert", "Work could not be saved. काम सहेजा नहीं गया।" }
                button { onclick: move |_| persist.call(progress()), "Retry save · फिर सहेजें" }
            }
            if let Some(task) = active {
                nav { class: "actions",
                    button { disabled: crate::exercise_lock::locked(), onclick: crate::exercise_lock::guard(move |_| { let mut n=progress(); n.active=None; n.paused=false; persist.call(n); }), "Back to day · दिन पर लौटें" }
                    button { onclick: move |_| { let mut n=progress(); n.paused=true; persist.call(n); }, "Take a break · थोड़ा आराम" }
                }
                p { "Day {day.number} · दिन {day.number} · {subject_name(&task.subject)}" }
                h2 { "{task.goal}" }
                p { class: "daily-task-source", "{task.source_label}" }
                p { "Exercise {task.number()} · अभ्यास {task.number()}" }
                if current.completed.contains(&task.id) {
                    p { "✓ Previously completed · पहले पूरा किया" }
                    { let retry_task=task.clone(); rsx! {
                        button { disabled: crate::exercise_lock::locked(), onclick: crate::exercise_lock::guard(move |_| { let mut n=progress(); n.retry(&retry_task); persist.call(n); }), "Try again · फिर अभ्यास करें" }
                    } }
                }
                if current.paused {
                    h2 { "Take a short break · थोड़ा आराम करें" }
                    button { class: "primary", onclick: move |_| { let mut n=progress(); n.paused=false; persist.call(n); }, "Continue · जारी रखें" }
                } else {
                    TaskRunner { key: "{task.id}-{current.attempt}", task: task.clone(), progress: progress(), persist, math_language, scale }
                }
            } else {
                button { disabled: crate::exercise_lock::locked(), class: "primary", onclick: crate::exercise_lock::guard(move |_| {
                    let mut n=progress(); n.day=n.next_day(); n.active=None; persist.call(n);
                }), "Continue next unfinished day · अगला अधूरा दिन" }
                details {
                    summary { "Choose a day · दिन चुनें" }
                    nav { class: "daily-days", aria_label: "Choose a day",
                        for d in &PLAN.days {
                            button { disabled: crate::exercise_lock::locked(), "data-day": "{d.number}", aria_pressed: current.day==d.number,
                                onclick: crate::exercise_lock::guard(move |_| { let mut n=progress(); n.day=d.number; n.active=None; n.paused=false; persist.call(n); }),
                                "{d.number}" if current.day_complete(d) { " ✓" }
                            }
                        }
                    }
                }
                h2 { "Day {day.number} · दिन {day.number} · Class {grade}" }
                p { "{day.tasks.iter().filter(|t| current.completed.contains(&t.id)).count()} / {day.tasks.len()} tasks complete · अभ्यास पूरे" }
                if current.day_complete(day) { p { role: "status", "✓ Day complete · दिन पूरा" } }
                div { class: "daily-sections",
                    for subject in SUBJECTS {
                        section { class: "daily-subject", "data-daily-subject": subject,
                            h2 { "{subject_name(subject)}" }
                            p { "{day.tasks.iter().filter(|t| t.subject==subject && current.completed.contains(&t.id)).count()} / {day.tasks.iter().filter(|t| t.subject==subject).count()} tasks · अभ्यास" }
                            button { disabled: crate::exercise_lock::locked(), class: "primary", onclick: crate::exercise_lock::guard(move |_| {
                                let mut n=progress();
                                let tasks=&PLAN.days[n.day-1].tasks;
                                n.active=tasks.iter().find(|t| t.subject==subject && !n.completed.contains(&t.id))
                                    .or_else(|| tasks.iter().find(|t| t.subject==subject)).map(|t| t.id.clone());
                                persist.call(n);
                            }), "Start / Resume · शुरू / जारी" }
                            details { summary { "Goals and exercises · लक्ष्य और अभ्यास" }
                                div { class: "daily-goals",
                                    for task in day.tasks.iter().filter(|t| t.subject==subject) {
                                        { let id=task.id.clone(); rsx! {
                                            button { disabled: crate::exercise_lock::locked(), "data-task-id": "{task.id}", onclick: crate::exercise_lock::guard(move |_| { let mut n=progress(); n.active=Some(id.clone()); persist.call(n); }),
                                                if current.completed.contains(&task.id) { "✓ " }
                                                "{task.goal} · {task.source_label} · #{task.number()}"
                                            }
                                        } }
                                    }
                                }
                            }
                        }
                    }
                }
                details { class: "daily-coverage",
                    summary { "Coverage inventory · लक्ष्य सूची" }
                    p { "English, हिंदी, Mathematics and Basic · All learning goals and activities · सभी लक्ष्य और अभ्यास" }
                    for d in &PLAN.days {
                        details {
                            summary { "Day {d.number} · {d.tasks.len()} tasks" }
                            for subject in SUBJECTS {
                                h3 { "{subject_name(subject)}" }
                                for (label, count) in inventory(d, subject) { p { "{label} · {count} tasks" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn inventory(day: &Day, subject: &str) -> Vec<(String, usize)> {
    let mut groups: Vec<(String, usize)> = vec![];
    for task in day.tasks.iter().filter(|t| t.subject == subject) {
        if let Some((label, count)) = groups
            .last_mut()
            .filter(|(label, _)| label == &task.source_label)
        {
            let _ = label;
            *count += 1;
        } else {
            groups.push((task.source_label.clone(), 1));
        }
    }
    groups
}

#[component]
fn TaskRunner(
    task: Task,
    progress: Progress,
    persist: Callback<Progress>,
    math_language: Language,
    scale: PaperScale,
) -> Element {
    let memory = use_signal(|| progress.answers.get(&task.id).cloned().unwrap_or_default());
    use_context_provider(|| ActivityMemory(memory));
    let answer_id = task.id.clone();
    let answer_progress = progress.clone();
    let save_answer = use_callback(move |draft: ActivityDraft| {
        if answer_progress.answers.get(&answer_id) != Some(&draft)
            && draft != ActivityDraft::default()
        {
            let mut next = answer_progress.clone();
            next.answers.insert(answer_id.clone(), draft);
            persist.call(next);
        }
    });
    use_effect(move || save_answer.call(memory()));
    let pending_answer = use_signal(|| None::<String>);
    let language = match task.subject.as_str() {
        "hi" => Language::Hindi,
        "math" => math_language,
        _ => Language::English,
    };
    let next = progress.clone();
    let done_task = task.clone();
    let done = use_callback(crate::exercise_lock::guard(move |_| {
        let mut n = next.clone();
        n.advance(&done_task);
        persist.call(n);
    }));
    match &task.source {
        Source::Basic { index } => {
            let sheet = &basic::SHEETS[*index];
            let initial = progress.pencil.get(&task.id).cloned().unwrap_or_default();
            let id = task.id.clone();
            rsx! { div { class: "daily-task",
                crate::exercise_lock::LockControl { language }
                p { "Start at the marker and follow the path with your pen. निशान से पथ पर पेन चलाएँ।" }
                basic::PencilCanvas { src: basic::artwork(&sheet.file), fit_page: sheet.row==0,
                    crop: basic::pattern_crop(sheet.page), initial, language,
                    on_edit: move |strokes| { let mut n=progress.clone(); n.pencil.insert(id.clone(),strokes); persist.call(n); } }
                button { disabled: crate::exercise_lock::locked(), class: "primary", onclick: crate::exercise_lock::guard(move |_| done.call(())), "Mark complete · पूरा चिह्नित करें" }
            } }
        }
        Source::Worksheet { sheet, index } => {
            let sheet = worksheets::SHEETS
                .iter()
                .position(|s| &s.id == sheet)
                .unwrap();
            let source = &worksheets::SHEETS[sheet];
            let id = task.id.clone();
            rsx! { div { class: "daily-task",
                worksheets::Exercise { sheet, position: *index, language, scale,
                    draft: progress.worksheets.get(&task.id).cloned().unwrap_or_default(), pending_answer,
                    on_edit: move |draft| { let mut n=progress.clone(); n.worksheets.insert(id.clone(),draft); persist.call(n); },
                    on_next: move |_| done.call(()) }
                if !source.adult.is_empty() || !source.tasks[*index].key.is_empty() {
                    details { summary { "For grown-ups · बड़ों के लिए" }
                        p { "{source.adult}" } p { "{source.tasks[*index].key}" }
                    }
                }
            } }
        }
        Source::Lesson {
            topic: id,
            index,
            recall,
        } => {
            let topic = topic(&task.subject, id, language);
            let index = *index;
            if *recall {
                return rsx! { Activity { kind: Kind::Choice(topic.recall[index].clone()), initial: Ink::new(), scale, language, recall: true,
                on_input: move |input| { if matches!(input,Input::Answer(_)) { done.call(()); } } } };
            }
            let step = Step {
                subject: &task.subject,
                topic,
                index,
                review: false,
            };
            let initial = progress.lesson_ink(&task, &step);
            let revision_source = step
                .revision_source(&progress.lessons)
                .map(|(ink, lines)| (ink.clone(), lines));
            let helper_draft = progress.helpers.get(&task.id).cloned().unwrap_or_default();
            let exercise = &topic.exercises[index];
            rsx! { Activity { kind: exercise.kind.clone(), picture: exercise.picture.clone().or_else(||crate::pictures::for_kind(&exercise.kind).map(str::to_owned)),
                initial, scale, language, helper_draft, revision_source,
                handwriting: task.subject=="en", ruled_paper: Paper::for_subject(&task.subject),
                on_input: move |input: Input| {
                    if !input.is_draft() && crate::exercise_lock::locked() { return; }
                    let mut n=progress.clone();
                    if n.lesson_input(&task,input,language) { persist.call(n); }
                }
            } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_subject_only_coverage_and_stable_ids() {
        assert_eq!(PLAN.version, VERSION);
        assert_eq!(PLAN.days.len(), 60);
        let mut ids = BTreeSet::new();
        let mut lessons = BTreeSet::new();
        let mut sheets = BTreeSet::new();
        let mut pencil = Vec::new();
        let mut last_lesson: BTreeMap<(String, u8), (usize, usize)> = BTreeMap::new();
        for day in &PLAN.days {
            for subject in SUBJECTS {
                assert!(
                    day.tasks.iter().any(|t| t.subject == subject),
                    "day {} missing {subject}",
                    day.number
                );
            }
            let basic_count = day.tasks.iter().filter(|t| t.subject == "basic").count();
            assert!((1..=2).contains(&basic_count));
            for task in &day.tasks {
                assert!(ids.insert(task.id.clone()), "duplicate {}", task.id);
                assert!(SUBJECTS.contains(&task.subject.as_str()));
                assert_eq!(task.grade, if day.number <= 30 { 1 } else { 2 });
                match &task.source {
                    Source::Lesson {
                        topic: id,
                        index,
                        recall,
                    } => {
                        let topic = topic(&task.subject, id, Language::Hindi);
                        let list = content::BUNDLED
                            .iter()
                            .find(|s| s.id == task.subject)
                            .unwrap();
                        let mut topics: Vec<_> = list
                            .topics
                            .iter()
                            .filter(|t| t.grade == task.grade)
                            .collect();
                        topics.sort_by_key(|t| curriculum::order(&task.subject, t));
                        let order = topics.iter().position(|t| t.id == *id).unwrap();
                        let step = if *recall {
                            topic.exercises.len() + index
                        } else {
                            *index
                        };
                        let previous =
                            last_lesson.insert((task.subject.clone(), task.grade), (order, step));
                        if let Some(previous) = previous {
                            assert!(previous < (order, step));
                        }
                        if *recall {
                            assert!(topic.recall[*index].valid());
                        } else if matches!(topic.exercises[*index].kind, Kind::Revise(..)) {
                            assert!(day.tasks.iter().any(|t|matches!(&t.source,Source::Lesson{topic:other,index:before,recall:false} if other==id && *before+1==*index)));
                        }
                        assert!(lessons.insert((
                            task.subject.clone(),
                            id.clone(),
                            *recall,
                            *index
                        )));
                    }
                    Source::Worksheet { sheet, index } => {
                        let s = worksheets::SHEETS.iter().find(|s| &s.id == sheet).unwrap();
                        assert!(*index < s.tasks.len());
                        assert_eq!(s.subject, task.subject);
                        assert!(sheets.insert((sheet.clone(), *index)));
                    }
                    Source::Basic { index } => pencil.push(*index),
                }
            }
        }
        for s in content::BUNDLED.iter() {
            for t in &s.topics {
                for index in 0..t.exercises.len() {
                    assert!(lessons.remove(&(s.id.clone(), t.id.clone(), false, index)));
                }
                for index in 0..t.recall.len() {
                    assert!(lessons.remove(&(s.id.clone(), t.id.clone(), true, index)));
                }
            }
        }
        assert!(lessons.is_empty());
        for s in worksheets::SHEETS.iter() {
            for index in 0..s.tasks.len() {
                assert!(sheets.remove(&(s.id.clone(), index)));
            }
        }
        assert!(sheets.is_empty());
        assert_eq!(basic::SHEETS.len(), 68);
        assert_eq!(pencil, (0..68).collect::<Vec<_>>());
        assert_eq!(
            serde_json::to_string(&*PLAN).unwrap(),
            serde_json::to_string(&build_manifest()).unwrap()
        );
    }

    #[test]
    fn resume_completion_review_and_storage_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("daily-exercises.json");
        let mut p = load(&path).unwrap();
        assert_eq!(p.next_day(), 1);
        let task = &PLAN.days[0].tasks[0];
        p.active = Some(task.id.clone());
        p.paused = true;
        p.worksheets
            .insert(task.id.clone(), worksheets::Draft::default());
        state::save_local(&path, &p).unwrap();
        let mut p = load(&path).unwrap();
        assert!(p.paused);
        assert_eq!(p.active.as_ref(), Some(&task.id));
        p.advance(task);
        p.advance(task);
        assert_eq!(p.completed.len(), 1);
        for t in &PLAN.days[0].tasks {
            p.advance(t);
        }
        assert!(p.day_complete(&PLAN.days[0]));
        assert_eq!(p.next_day(), 2);
        assert!(State::default().finished.is_empty());
        p.version = VERSION + 1;
        state::save_local(&path, &p).unwrap();
        assert!(load(&path).is_err());
        std::fs::write(&path, "{broken").unwrap();
        assert!(load(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{broken");
        assert!(state::save_local(&path.join("not-a-folder"), &p).is_err());
    }

    #[test]
    fn daily_handwriting_and_revision_resume_without_touching_standalone_work() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("daily-exercises.json");
        let (day, task) = PLAN
            .days
            .iter()
            .find_map(|day| {
                day.tasks
                    .iter()
                    .find(|task| {
                        let Source::Lesson {
                            topic: id,
                            index,
                            recall: false,
                        } = &task.source
                        else {
                            return false;
                        };
                        if task.subject != "en" {
                            return false;
                        }
                        let topic = topic("en", id, Language::English);
                        matches!(topic.exercises[*index].kind, Kind::Compose(..))
                            && topic
                                .exercises
                                .get(index + 1)
                                .is_some_and(|e| matches!(e.kind, Kind::Revise(..)))
                    })
                    .map(|task| (day, task))
            })
            .unwrap();
        let Source::Lesson {
            topic: id, index, ..
        } = &task.source
        else {
            unreachable!()
        };
        let t = topic("en", id, Language::English);
        let lines = t.exercises[*index]
            .kind
            .writing(Paper::English)
            .unwrap()
            .lines;
        let ink = vec![vec![vec![[0.2, 0.3], [0.4, 0.5]]]; lines];
        let mut p = Progress {
            day: day.number,
            active: Some(task.id.clone()),
            ..Progress::default()
        };
        assert!(p.lesson_input(task, Input::Draft(ink.clone()), Language::English));
        assert!(!p.completed.contains(&task.id));
        state::save_local(&path, &p).unwrap();
        let mut p = load(&path).unwrap();
        let step = Step {
            subject: "en",
            topic: t,
            index: *index,
            review: false,
        };
        assert_eq!(step.ink(&p.lessons), ink);
        assert!(p.lesson_input(task, Input::Submit(ink.clone()), Language::English));
        assert!(p.completed.contains(&task.id));
        assert_eq!(p.lesson_ink(task, &step), ink);
        let revision = Step {
            index: index + 1,
            ..step
        };
        assert_eq!(revision.revision_source(&p.lessons), Some((&ink, lines)));
        let count = p.completed.len();
        p.retry(task);
        assert_eq!(p.completed.len(), count);
        assert!(step.ink(&p.lessons).is_empty());
        assert!(p.lesson_ink(task, &step).is_empty());
        assert!(!dir.path().join("progress.json").exists());
        assert!(!dir.path().join("basic-pencil.json").exists());
    }

    #[test]
    fn arithmetic_helper_drafts_stay_with_their_daily_exercise() {
        let tasks:Vec<_>=PLAN.days.iter().flat_map(|d|d.tasks.iter().map(move|t|(d,t)))
            .filter(|(_,t)|matches!(&t.source,Source::Lesson{topic:id,index,recall:false}
                if t.subject=="math" && matches!(topic("math",id,Language::Hindi).exercises[*index].kind,Kind::Solve{helper:Some(_),..})))
            .take(2).collect();
        assert_eq!(tasks.len(), 2);
        let mut p = Progress {
            day: tasks[0].0.number,
            ..Progress::default()
        };
        let helper = state::HelperDraft {
            ones_checked: true,
            ..state::HelperDraft::default()
        };
        assert!(p.lesson_input(tasks[0].1, Input::Helper(helper.clone()), Language::Hindi));
        assert!(!p.helpers.contains_key(&tasks[1].1.id));
        p.day = tasks[1].0.number;
        assert!(p.lesson_input(
            tasks[1].1,
            Input::Helper(state::HelperDraft::default()),
            Language::Hindi
        ));
        assert_eq!(p.helpers[&tasks[0].1.id], helper);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("daily-exercises.json");
        state::save_local(&path, &p).unwrap();
        assert_eq!(load(&path).unwrap().helpers[&tasks[0].1.id], helper);
    }
}
