//! Evidence, not completion, is the source of mastery estimates.
use crate::{
    content::{self, Kind, Topic},
    state::State,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Assistance {
    #[default]
    Unreported,
    Independent,
    VisualPrompt,
    VerbalPrompt,
    Guided,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Writing,
    Tracing,
    Selection,
    NumericEntry,
    Speech,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Context {
    pub submission_id: String,
    pub assistance: Assistance,
    pub prior_errors: bool,
    pub hint_used: bool,
    pub helper_used: bool,
    pub solution_exposed: bool,
    pub model_visible: bool,
    pub duration_ms: Option<u64>,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            submission_id: content::new_id(),
            assistance: Assistance::Unreported,
            prior_errors: false,
            hint_used: false,
            helper_used: false,
            solution_exposed: false,
            model_visible: false,
            duration_ms: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attempt {
    pub submission_id: String,
    pub source: String,
    pub exercise_id: String,
    pub competency_id: String,
    pub timestamp: i64,
    pub day: u64,
    pub correct: Option<bool>,
    pub accuracy: Option<f64>,
    pub mode: Mode,
    pub context: Context,
    pub review: bool,
}
impl Attempt {
    /// Independence is observable within the app; external assistance may be unreported.
    pub fn independent(&self) -> bool {
        matches!(
            self.context.assistance,
            Assistance::Unreported | Assistance::Independent
        ) && !self.context.model_visible
            && !self.context.hint_used
            && !self.context.helper_used
            && !self.context.solution_exposed
    }
    pub fn success(&self) -> bool {
        self.independent() && self.correct == Some(true) && !self.context.prior_errors
    }
}
#[derive(Clone, Debug)]
pub struct Mapping {
    pub competency: String,
    pub subject: String,
    pub topic: String,
    pub index: usize,
    pub recall: bool,
    pub grade: u8,
    pub label: String,
    pub minutes: u16,
    pub prerequisites: Vec<String>,
}
pub fn bundled(subject: &str, topic: &Topic) -> bool {
    content::BUNDLED
        .iter()
        .find(|s| s.id == subject)
        .is_some_and(|s| {
            s.topics.iter().any(|t| {
                t == topic || (subject == "math" && content::english_math_topic(t) == topic)
            })
        })
}
fn family(subject: &str, topic: &Topic) -> String {
    // These IDs come from source identifiers, never translated prompts or labels.
    if subject == "math" {
        let id = topic.id.as_str();
        let name = if id.contains("borrow") {
            "subtraction-borrowing"
        } else if id.contains("exchange") || id.contains("carry") {
            "addition-carrying"
        } else if id.contains("subtract") {
            "subtraction"
        } else if id.contains("add") {
            "addition"
        } else if id.contains("div") || id.contains("share") {
            "division"
        } else if id.contains("table") || id.contains("multipl") {
            "multiplication"
        } else {
            id
        };
        format!("math/{name}")
    } else {
        format!("{subject}/{}", topic.id)
    }
}
pub fn mapping(subject: &str, topic: &Topic, index: usize, recall: bool) -> Option<Mapping> {
    if !bundled(subject, topic) {
        return None;
    }
    let base = family(subject, topic);
    let (competency, minutes, suffix) = if recall {
        topic.recall.get(index)?;
        // Recall demonstrates recognition/comprehension, never letter formation.
        (format!("{base}/recognition/{index}"), 1, "Recognition")
    } else {
        let exercise = topic.exercises.get(index)?;
        match &exercise.kind {
            Kind::Trace { .. } => (
                format!("{base}/formation/{}", exercise.id),
                2,
                "Letter or number formation",
            ),
            Kind::Compose(..) | Kind::Revise(..) => {
                (format!("{base}/writing/{}", exercise.id), 3, "Writing")
            }
            Kind::Talk { .. } => (
                format!("{base}/conversation/{}", exercise.id),
                2,
                "Conversation",
            ),
            Kind::Solve { .. } | Kind::Table { .. } => {
                (format!("{base}/arithmetic"), 2, "Arithmetic")
            }
            _ => (
                format!("{base}/recognition/{}", exercise.id),
                1,
                "Recognition",
            ),
        }
    };
    let prerequisite = match base.as_str() {
        "math/addition-carrying" => Some("math/addition/arithmetic"),
        "math/subtraction-borrowing" => Some("math/subtraction/arithmetic"),
        "math/division" => Some("math/multiplication/arithmetic"),
        _ => None,
    };
    Some(Mapping {
        competency,
        subject: subject.into(),
        topic: topic.id.clone(),
        index,
        recall,
        grade: topic.grade,
        label: if !recall {
            match &topic.exercises[index].kind {
                Kind::Trace { model, .. } => format!("{} · {suffix}: {model}", topic.title),
                _ => format!("{} · {suffix}", topic.title),
            }
        } else {
            format!("{} · {suffix}", topic.title)
        },
        minutes,
        prerequisites: prerequisite.into_iter().map(str::to_owned).collect(),
    })
}
pub fn catalog() -> Vec<Mapping> {
    static CATALOG: std::sync::LazyLock<Vec<Mapping>> = std::sync::LazyLock::new(|| {
        content::BUNDLED
            .iter()
            .filter(|s| matches!(s.id.as_str(), "hi" | "en" | "math"))
            .flat_map(|s| {
                s.topics.iter().flat_map(move |t| {
                    (0..t.exercises.len())
                        .filter_map(move |i| mapping(&s.id, t, i, false))
                        .chain((0..t.recall.len()).filter_map(move |i| mapping(&s.id, t, i, true)))
                })
            })
            .collect()
    });
    debug_assert!(validate_catalog(&CATALOG));
    CATALOG.clone()
}
pub fn validate_catalog(items: &[Mapping]) -> bool {
    let ids: BTreeSet<_> = items.iter().map(|m| m.competency.as_str()).collect();
    let graph: BTreeMap<_, _> = items
        .iter()
        .map(|m| (m.competency.as_str(), &m.prerequisites))
        .collect();
    fn visit<'a>(
        id: &'a str,
        graph: &BTreeMap<&'a str, &'a Vec<String>>,
        path: &mut BTreeSet<&'a str>,
    ) -> bool {
        if !path.insert(id) {
            return false;
        }
        if let Some(edges) = graph.get(id) {
            for edge in *edges {
                if !graph.contains_key(edge.as_str()) || !visit(edge, graph, path) {
                    return false;
                }
            }
        }
        path.remove(id);
        true
    }
    ids.iter().all(|id| visit(id, &graph, &mut BTreeSet::new()))
}
pub fn attempt(
    subject: &str,
    topic: &Topic,
    index: usize,
    recall: bool,
    input: &crate::progress::Input,
    mut context: Context,
    source: &str,
    review: bool,
    day: u64,
) -> Option<Attempt> {
    use crate::progress::Input;
    let m = mapping(subject, topic, index, recall)?;
    let kind = (!recall).then(|| &topic.exercises[index].kind);
    let (mut correct, mode) = match input {
        Input::Answer(missed) => (Some(!*missed), Mode::Selection),
        Input::SubmitSolution { correct, .. } => (Some(*correct), Mode::NumericEntry),
        Input::Submit(_) => (
            None,
            if matches!(kind, Some(Kind::Trace { .. })) {
                Mode::Tracing
            } else {
                Mode::Writing
            },
        ),
        _ => return None,
    };
    if matches!(kind, Some(Kind::Talk { .. })) {
        correct = None;
    }
    let mode = if matches!(kind, Some(Kind::Talk { .. })) && matches!(input, Input::Answer(_)) {
        Mode::Speech
    } else {
        mode
    };
    if matches!(
        kind,
        Some(Kind::Trace { .. } | Kind::Compose(_, _, Some(_)))
    ) {
        context.model_visible = true;
    }
    if context.prior_errors {
        correct = Some(false).filter(|_| correct.is_some());
    }
    Some(Attempt {
        submission_id: context.submission_id.clone(),
        source: source.into(),
        exercise_id: format!(
            "{subject}/{}/{}",
            topic.progress_id(),
            if recall {
                format!("recall/{index}")
            } else {
                topic.exercises[index].id.clone()
            }
        ),
        competency_id: m.competency,
        timestamp: (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64,
        day,
        correct,
        accuracy: correct.map(|v| if v { 1.0 } else { 0.0 }),
        mode,
        context,
        review,
    })
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Evidence {
    pub version: u32,
    pub attempts: BTreeMap<String, Attempt>,
    pub reviews: BTreeMap<String, Check>,
    pub migrated_topics: BTreeSet<String>,
}
impl Default for Evidence {
    fn default() -> Self {
        Self {
            version: 1,
            attempts: Default::default(),
            reviews: Default::default(),
            migrated_topics: Default::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Check {
    pub due: u64,
    pub interval_days: u64,
    pub rotation: usize,
}
impl Evidence {
    pub fn record(&mut self, attempt: Attempt) -> bool {
        if self.attempts.contains_key(&attempt.submission_id) {
            return false;
        }
        let check = self
            .reviews
            .entry(attempt.competency_id.clone())
            .or_insert(Check {
                due: attempt.day.saturating_add(1),
                interval_days: 1,
                rotation: 0,
            });
        if attempt.review {
            if attempt.success() {
                check.interval_days = check.interval_days.saturating_mul(2).clamp(1, 60);
                check.due = attempt.day.saturating_add(check.interval_days);
            } else {
                if attempt.independent() && attempt.correct == Some(false) {
                    check.interval_days = (check.interval_days / 2).max(1);
                }
                check.due = attempt.day.saturating_add(1);
            }
            check.rotation = check.rotation.saturating_add(1);
        }
        self.version = 1;
        self.attempts.insert(attempt.submission_id.clone(), attempt);
        true
    }
}
pub fn migrate(state: &mut State, today: u64) {
    for s in content::BUNDLED
        .iter()
        .filter(|s| matches!(s.id.as_str(), "en" | "hi" | "math"))
    {
        for t in &s.topics {
            let key = format!("{}/{}", s.id, t.progress_id());
            if !state.finished.contains(&key) || !state.evidence.migrated_topics.insert(key.clone())
            {
                continue;
            }
            let due = state.reviews.get(&key).map_or(today, |r| r.due);
            for m in (0..t.recall.len()).filter_map(|i| mapping(&s.id, t, i, true)) {
                state.evidence.reviews.entry(m.competency).or_insert(Check {
                    due,
                    interval_days: 1,
                    rotation: 0,
                });
            }
        }
    }
    state.evidence.version = 1;
}

impl Assistance {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unreported => "Adult help unreported",
            Self::Independent => "Reported independent",
            Self::VisualPrompt => "Visual prompt",
            Self::VerbalPrompt => "Verbal prompt",
            Self::Guided => "Guided",
        }
    }
}
impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Writing => "Writing",
            Self::Tracing => "Tracing",
            Self::Selection => "Selection",
            Self::NumericEntry => "Entered arithmetic answer",
            Self::Speech => "Conversation",
        }
    }
}
