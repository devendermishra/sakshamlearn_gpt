//! Deterministic, budgeted recommendations. Nothing here locks the curriculum.
use crate::{
    assessment::{self, Mapping},
    content::{self, Kind},
    mastery::{self, Level},
    state::{self, Ink, State},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    Review,
    Remediation,
    New,
}
impl Reason {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Review => "Overdue review",
            Self::Remediation => "Prerequisite practice",
            Self::New => "New learning",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub subject: String,
    pub topic: String,
    #[serde(default)]
    pub revision: u32,
    pub indices: Vec<usize>,
    pub recall: bool,
    pub competency: String,
    pub label: String,
    pub minutes: u16,
    pub reason: Reason,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    pub id: String,
    pub day: u64,
    pub grade: u8,
    pub tasks: Vec<Task>,
    pub position: usize,
    pub part: usize,
    pub drafts: BTreeMap<String, crate::activity::ActivityDraft>,
    pub ink: BTreeMap<String, Ink>,
    pub helpers: BTreeMap<String, state::HelperDraft>,
}
impl Session {
    pub fn complete(&self) -> bool {
        self.position >= self.tasks.len()
    }
    pub fn key(&self) -> String {
        format!("{}/{}", self.position, self.part)
    }
    pub fn advance(&mut self) {
        if let Some(task) = self.tasks.get(self.position) {
            self.part += 1;
            if self.part >= task.indices.len() {
                self.part = 0;
                self.position += 1;
            }
        }
    }
    pub fn minutes(&self) -> u16 {
        self.tasks.iter().map(|t| t.minutes).sum()
    }
}
pub fn topic(task: &Task, language: crate::language::Language) -> &'static content::Topic {
    let t = content::BUNDLED
        .iter()
        .find(|s| s.id == task.subject)
        .unwrap()
        .topics
        .iter()
        .find(|t| t.id == task.topic)
        .unwrap();
    if task.subject == "math" && language == crate::language::Language::English {
        content::english_math_topic(t)
    } else {
        t
    }
}
fn task(m: &Mapping, reason: Reason) -> Option<Task> {
    let t = content::BUNDLED
        .iter()
        .find(|s| s.id == m.subject)?
        .topics
        .iter()
        .find(|t| t.id == m.topic)?;
    if !m.recall && matches!(t.exercises[m.index].kind, Kind::Revise(..)) {
        return None;
    }
    let mut indices = vec![m.index];
    let mut minutes = m.minutes;
    if !m.recall && matches!(t.exercises[m.index].kind, Kind::Compose(..)) {
        for i in m.index + 1..t.exercises.len() {
            if !matches!(t.exercises[i].kind, Kind::Revise(..)) {
                break;
            }
            indices.push(i);
            minutes += 3;
        }
    }
    Some(Task {
        id: format!("{}/{}/{}/{}", m.subject, m.topic, m.recall, m.index),
        subject: m.subject.clone(),
        topic: m.topic.clone(),
        revision: t.revision,
        indices,
        recall: m.recall,
        competency: m.competency.clone(),
        label: m.label.clone(),
        minutes,
        reason,
    })
}
pub fn build(state: &State, grade: u8, day: u64) -> Session {
    let budget = if state.daily_minutes == 0 {
        15
    } else {
        state.daily_minutes.clamp(5, 30)
    };
    let catalog: Vec<_> = assessment::catalog()
        .into_iter()
        .filter(|m| m.grade == grade)
        .collect();
    let mut reviews = Vec::new();
    let mut remediation = Vec::new();
    let mut fresh = Vec::new();
    let mut grouped: BTreeMap<&str, Vec<&Mapping>> = BTreeMap::new();
    for m in &catalog {
        grouped.entry(&m.competency).or_default().push(m);
    }
    for (id, maps) in &grouped {
        if let Some(check) = state.evidence.reviews.get(*id).filter(|c| c.due <= day) {
            let candidates: Vec<_> = maps
                .iter()
                .filter_map(|m| task(m, Reason::Review))
                .collect();
            if !candidates.is_empty() {
                reviews.push((
                    check.due,
                    candidates[check.rotation % candidates.len()].clone(),
                ));
            }
        }
    }
    for m in &catalog {
        let active_skill = state.evidence.reviews.contains_key(&m.competency)
            || state
                .evidence
                .attempts
                .values()
                .any(|a| a.competency_id == m.competency);
        for prereq in m.prerequisites.iter().filter(|_| active_skill) {
            if !matches!(
                mastery::estimate(&state.evidence, prereq).level,
                Level::IndependentSuccess | Level::Retained
            ) {
                if let Some(targets) = grouped.get(prereq.as_str()) {
                    if let Some(t) = targets.iter().find_map(|p| task(p, Reason::Remediation)) {
                        remediation.push(t);
                    }
                }
            }
        }
        let exposed = state.evidence.attempts.values().any(|a| {
            a.exercise_id
                == format!(
                    "{}/{}/{}",
                    m.subject,
                    m.topic,
                    if m.recall {
                        format!("recall/{}", m.index)
                    } else {
                        let t = content::BUNDLED
                            .iter()
                            .find(|s| s.id == m.subject)
                            .unwrap()
                            .topics
                            .iter()
                            .find(|t| t.id == m.topic)
                            .unwrap();
                        t.exercises[m.index].id.clone()
                    }
                )
        });
        if !exposed
            && !state
                .finished
                .contains(&format!("{}/{}", m.subject, m.topic))
        {
            if let Some(t) = task(m, Reason::New) {
                fresh.push(t);
            }
        }
    }
    reviews.sort_by_key(|(due, t)| (*due, t.id.clone()));
    remediation.sort_by_key(|t| t.id.clone());
    // Curriculum order supplies prerequisite-friendly ordering; IDs break ties.
    fresh.sort_by_key(|t| {
        let subject = content::BUNDLED.iter().find(|s| s.id == t.subject).unwrap();
        let topic = subject.topics.iter().find(|p| p.id == t.topic).unwrap();
        let gaps = catalog
            .iter()
            .find(|m| m.competency == t.competency)
            .map_or(0, |m| {
                m.prerequisites
                    .iter()
                    .filter(|p| {
                        !matches!(
                            mastery::estimate(&state.evidence, p).level,
                            Level::IndependentSuccess | Level::Retained
                        )
                    })
                    .count()
            });
        (
            gaps,
            (["en", "hi", "math"]
                .iter()
                .position(|s| *s == t.subject)
                .unwrap()
                + 3
                - (day % 3) as usize)
                % 3,
            crate::curriculum::order(&t.subject, topic),
            t.subject.clone(),
            subject.topics.iter().position(|p| p.id == t.topic).unwrap(),
            t.indices[0],
            t.recall,
            t.id.clone(),
        )
    });
    let due_ids: BTreeSet<_> = reviews.iter().map(|(_, t)| t.id.clone()).collect();
    fresh.retain(|t| !due_ids.contains(&t.id));
    let mut tasks = Vec::new();
    let mut used = BTreeSet::new();
    let mut total = 0;
    let mut new_tasks = Vec::new();
    let mut reserved = 0;
    for t in &fresh {
        if new_tasks.len() < 2 && reserved + t.minutes <= budget / 5 && used.insert(t.id.clone()) {
            reserved += t.minutes;
            new_tasks.push(t.clone());
        }
    }
    for t in reviews.into_iter().map(|(_, t)| t).chain(remediation) {
        if total + reserved + t.minutes <= budget && used.insert(t.id.clone()) {
            total += t.minutes;
            tasks.push(t);
        }
    }
    for t in fresh {
        if new_tasks.len() < 2
            && total + reserved + t.minutes <= budget
            && used.insert(t.id.clone())
        {
            reserved += t.minutes;
            new_tasks.push(t);
        }
    }
    tasks.extend(new_tasks);
    Session {
        id: content::new_id(),
        day,
        grade,
        tasks,
        ..Session::default()
    }
}

use dioxus::prelude::*;
#[component]
pub fn Today(
    data: Signal<State>,
    commit: Callback<State>,
    durable: Callback<State, Result<(), String>>,
    grade: u8,
) -> Element {
    let mut error = use_signal(String::new);
    let snapshot = data.read();
    let session = snapshot.adaptive.clone();
    let complete = session.as_ref().is_none_or(Session::complete);
    let display_grade = session
        .as_ref()
        .filter(|s| !s.complete())
        .map_or(grade, |s| s.grade);
    let preview = if complete {
        build(&snapshot, grade, state::day())
    } else {
        Session::default()
    };
    rsx! { section { class: "adaptive-pane",
        h1 { "Today's learning · आज की पढ़ाई" }
        p { "Class {display_grade} · Estimated budget: {snapshot.daily_minutes} minutes" }
        if complete {
            if session.is_some() { p { role: "status", "Session complete. Your work is saved." } }
            p { "Next session: about {preview.minutes()} minutes. You can also choose any lesson yourself." }
            for t in &preview.tasks { p { "{t.label} · {t.reason.label()} · about {t.minutes} min" } }
            button { class: "primary", disabled: preview.tasks.is_empty() || crate::exercise_lock::locked(),
                onclick: crate::exercise_lock::guard(move |_| {
                    let mut next = data.peek().clone(); next.adaptive = Some(build(&next,grade,state::day()));
                    match durable.call(next) { Ok(()) => error.set(String::new()), Err(e) => error.set(e) }
                }), "Start today's session" }
        } else if let Some(session) = session {
            if let Some(task) = session.tasks.get(session.position) {
                p { "Activity {session.position + 1} / {session.tasks.len()} · {task.reason.label()} · about {task.minutes} min" }
                {rsx! { section { key: "{session.id}-{session.position}-{session.part}",
                    AdaptiveActivity { data, commit, durable, session: AdaptiveInput(session), on_error: move |e| error.set(e) }
                } }}
            }
        }
        p { role: "alert", "{error}" }
    } }
}
#[derive(Clone)]
struct AdaptiveInput(Session);
impl PartialEq for AdaptiveInput {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
#[component]
fn AdaptiveActivity(
    data: Signal<State>,
    commit: Callback<State>,
    durable: Callback<State, Result<(), String>>,
    session: AdaptiveInput,
    on_error: EventHandler<String>,
) -> Element {
    let session = session.0;
    let key = session.key();
    let task = session.tasks[session.position].clone();
    let index = task.indices[session.part];
    let snapshot = data.read();
    let language = if task.subject == "en" {
        crate::language::Language::English
    } else if task.subject == "math" {
        snapshot.math_language
    } else {
        crate::language::Language::Hindi
    };
    let topic = topic(&task, language);
    let kind = if task.recall {
        Kind::Choice(topic.recall[index].clone())
    } else {
        topic.exercises[index].kind.clone()
    };
    let initial = session.ink.get(&key).cloned().unwrap_or_default();
    let helper_draft = session.helpers.get(&key).cloned().unwrap_or_default();
    let memory = use_signal(|| session.drafts.get(&key).cloned().unwrap_or_default());
    use_context_provider(|| crate::activity::ActivityMemory(memory));
    let memory_key = key.clone();
    use_effect(move || {
        let draft = memory();
        let mut next = data.peek().clone();
        if let Some(s) = next.adaptive.as_mut() {
            if s.key() == memory_key && s.drafts.get(&memory_key) != Some(&draft) {
                s.drafts.insert(memory_key.clone(), draft);
                commit.call(next);
            }
        }
    });
    let revision_source = session
        .part
        .checked_sub(1)
        .and_then(|part| session.ink.get(&format!("{}/{part}", session.position)))
        .map(|ink| {
            (
                ink.clone(),
                topic.exercises[index.saturating_sub(1)]
                    .kind
                    .writing(content::Paper::for_subject(&task.subject))
                    .map_or(1, |w| w.lines),
            )
        });
    let scale = crate::writing::PaperScale::from_saved(snapshot.scale, snapshot.vertical_scale);
    rsx! { crate::activity::Activity { kind, initial, helper_draft, scale, language, recall: task.recall,
        handwriting: task.subject=="en", ruled_paper: content::Paper::for_subject(&task.subject), revision_source,
        on_input: move |input: crate::progress::Input| {
            if !input.is_draft() && crate::exercise_lock::locked() { return; }
            let mut next = data.peek().clone();
            let (raw,context) = input.unpack();
            let Some(s) = next.adaptive.as_mut() else { return; };
            if s.key()!=key || s.id!=session.id { return; }
            match &raw {
                crate::progress::Input::Draft(ink) => { s.ink.insert(key.clone(),ink.clone()); commit.call(next); return; }
                crate::progress::Input::Helper(helper) => { s.helpers.insert(key.clone(),helper.clone()); commit.call(next); return; }
                crate::progress::Input::Submit(ink) | crate::progress::Input::SubmitSolution{ink,..} => {
                    if !ink.iter().any(|cell| !cell.is_empty()) { return; }
                    if let Some(writing) = kind_for(topic,index,task.recall).and_then(|k|k.writing(content::Paper::for_subject(&task.subject))) {
                        if writing.paper==content::Paper::English && writing.lines>1 && (ink.len()!=writing.lines || ink.iter().any(Vec::is_empty)) { return; }
                    }
                    s.ink.insert(key.clone(),ink.clone());
                }
                _ => {}
            }
            let ctx = context.unwrap_or_default();
            if record_submission(&mut next,&task,index,&raw,ctx,state::day()) {
                next.adaptive.as_mut().unwrap().advance();
                match durable.call(next) { Ok(()) => on_error.call(String::new()), Err(e) => on_error.call(format!("Work could not be saved. Try Next again. {e}")) }
            }
        }
    } }
}
fn kind_for(topic: &content::Topic, index: usize, recall: bool) -> Option<&Kind> {
    if recall {
        None
    } else {
        topic.exercises.get(index).map(|e| &e.kind)
    }
}

pub fn valid_session(session: &Session) -> bool {
    if !matches!(session.grade, 1 | 2) || session.position > session.tasks.len() {
        return false;
    }
    for task in &session.tasks {
        let Some(topic) = content::BUNDLED
            .iter()
            .find(|s| s.id == task.subject)
            .and_then(|s| s.topics.iter().find(|t| t.id == task.topic))
        else {
            return false;
        };
        if task.revision != topic.revision
            || task.indices.is_empty()
            || task.indices.iter().any(|i| {
                *i >= if task.recall {
                    topic.recall.len()
                } else {
                    topic.exercises.len()
                }
            })
        {
            return false;
        }
    }
    session
        .tasks
        .get(session.position)
        .map_or(session.part == 0, |task| session.part < task.indices.len())
}

pub fn record_submission(
    state: &mut State,
    task: &Task,
    index: usize,
    raw: &crate::progress::Input,
    context: assessment::Context,
    day: u64,
) -> bool {
    let language = if task.subject == "math" {
        state.math_language
    } else {
        crate::language::Language::Hindi
    };
    let topic = topic(task, language);
    if let crate::progress::Input::Submit(ink)
    | crate::progress::Input::SubmitSolution { ink, .. } = raw
    {
        if !ink.iter().any(|cell| !cell.is_empty()) {
            return false;
        }
        if let Some(w) = kind_for(topic, index, task.recall)
            .and_then(|k| k.writing(content::Paper::for_subject(&task.subject)))
        {
            if w.paper == content::Paper::English
                && w.lines > 1
                && (ink.len() != w.lines || ink.iter().any(Vec::is_empty))
            {
                return false;
            }
        }
    }
    let Some(a) = assessment::attempt(
        &task.subject,
        topic,
        index,
        task.recall,
        raw,
        context,
        "adaptive",
        task.reason == Reason::Review,
        day,
    ) else {
        return false;
    };
    if !state.evidence.record(a) {
        return false;
    }
    if !task.recall {
        if let (
            Some(w),
            crate::progress::Input::Submit(ink)
            | crate::progress::Input::SubmitSolution { ink, .. },
        ) = (
            topic.exercises[index]
                .kind
                .writing(content::Paper::for_subject(&task.subject)),
            raw,
        ) {
            let sample = state::Sample {
                paper: w.paper,
                label: format!("{} · {}", topic.title, w.label),
                model: w.model.into(),
                solution: w.solution.map(str::to_owned),
                correct: match raw {
                    crate::progress::Input::SubmitSolution { correct, .. } => Some(*correct),
                    _ => None,
                },
                ink: ink.clone(),
                saved_on: Some(state::sample_date()),
                lines: w.lines,
            };
            state
                .history
                .entry(format!(
                    "{}/{}/{}",
                    task.subject,
                    topic.progress_id(),
                    topic.exercises[index].id
                ))
                .or_default()
                .push(sample);
        }
    }
    true
}
