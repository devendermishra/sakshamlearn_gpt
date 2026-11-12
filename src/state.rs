use crate::content::{generate_practice, CustomCatalog, GeneratedPractice, Paper, PracticeSkill};
use crate::language::Language;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const INTERVALS: [u64; 3] = [1, 3, 7];
pub type Stroke = Vec<[f32; 2]>;
pub type Ink = Vec<Vec<Stroke>>;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub catalog: CustomCatalog,
    pub math_language: Language,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aba_language: Option<Language>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub aba_practice: BTreeMap<String, BTreeSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<Session>,
    pub sessions: BTreeMap<String, Session>,
    pub finished: BTreeSet<String>,
    pub reviews: BTreeMap<String, Review>,
    pub samples: BTreeMap<String, Sample>,
    pub history: BTreeMap<String, Vec<Sample>>,
    history_initialized: bool,
    pub scale: Option<f64>,
    pub vertical_scale: Option<f64>,
    pub pin: Option<Pin>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub subject: String,
    pub topic: String,
    pub step: usize,
    pub review: bool,
    pub missed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated: Option<GeneratedPractice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub helper: Option<HelperDraft>,
}

impl Session {
    pub fn new(subject: String, topic: String, review: bool) -> Self {
        Self {
            subject,
            topic,
            step: 0,
            review,
            missed: false,
            generated: None,
            helper: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HelperDraft {
    pub first: Option<Ink>,
    pub second: Option<Ink>,
    pub borrowed_tens: bool,
    pub borrowed_ones: bool,
    pub carried: bool,
    pub ones_checked: bool,
    pub tens_checked: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sample {
    #[serde(default)]
    pub paper: Paper,
    pub label: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct: Option<bool>,
    pub ink: Ink,
    #[serde(default)]
    pub saved_on: Option<String>,
    #[serde(default = "one_line")]
    pub lines: usize,
}

fn one_line() -> usize {
    1
}

pub fn india_date() -> time::Date {
    time::OffsetDateTime::now_utc()
        .to_offset(time::UtcOffset::from_hms(5, 30, 0).expect("valid India offset"))
        .date()
}

pub fn sample_date() -> String {
    india_date().to_string()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Review {
    pub started: u64,
    pub due: u64,
    pub interval: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Pin {
    salt: [u8; 16],
    digest: [u8; 32],
}

impl Pin {
    pub fn new(input: &str) -> Self {
        let salt = rand::random();
        Self {
            salt,
            digest: hash(input, salt),
        }
    }

    pub fn matches(&self, input: &str) -> bool {
        hash(input, self.salt)
            .iter()
            .zip(self.digest)
            .fold(0, |diff, (a, b)| diff | (*a ^ b))
            == 0
    }
}

fn hash(input: &str, salt: [u8; 16]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut digest: [u8; 32] = Sha256::new()
        .chain_update(salt)
        .chain_update(input)
        .finalize()
        .into();
    for _ in 0..30_000 {
        digest = Sha256::new()
            .chain_update(salt)
            .chain_update(digest)
            .finalize()
            .into();
    }
    digest
}

pub fn day() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400
}

impl State {
    pub fn mark_aba_practice(&mut self, date: time::Date, activity: &str, practiced: bool) {
        let date = date.to_string();
        if practiced {
            self.aba_practice
                .entry(date)
                .or_default()
                .insert(activity.to_owned());
        } else if let std::collections::btree_map::Entry::Occupied(mut entry) =
            self.aba_practice.entry(date)
        {
            entry.get_mut().remove(activity);
            if entry.get().is_empty() {
                entry.remove();
            }
        }
    }

    pub fn start_generated(
        &mut self,
        grade: u8,
        skill: PracticeSkill,
        seed: u64,
    ) -> Option<String> {
        let id = skill.id(grade);
        let key = session_key("math", &id, false);
        if let std::collections::btree_map::Entry::Vacant(entry) = self.sessions.entry(key) {
            let mut session = Session::new("math".into(), id.clone(), false);
            session.generated = Some(generate_practice(grade, skill, seed)?);
            entry.insert(session);
        }
        Some(id)
    }

    pub fn due_review_count(&self, today: u64) -> usize {
        self.catalog
            .subjects()
            .flat_map(|subject| {
                self.catalog
                    .topics(subject)
                    .map(move |topic| format!("{}/{}", subject.id, topic.progress_id()))
            })
            .filter(|key| {
                self.reviews
                    .get(key)
                    .is_some_and(|review| review.due <= today)
            })
            .count()
    }

    pub fn finish(&mut self, id: &str, today: u64) {
        if self.finished.insert(id.to_owned()) {
            self.reviews.insert(
                id.to_owned(),
                Review {
                    started: today,
                    due: today + INTERVALS[0],
                    interval: 0,
                },
            );
        }
    }

    pub fn review(&mut self, id: &str, today: u64, missed: bool) {
        let Some(review) = self.reviews.get_mut(id) else {
            return;
        };
        if missed {
            review.due = today + 1;
        } else if let Some(delay) = INTERVALS.get(review.interval + 1) {
            review.interval += 1;
            review.due = (review.started + delay).max(today + 1);
        } else {
            self.reviews.remove(id);
        }
    }
}

pub fn session_key(subject: &str, topic: &str, review: bool) -> String {
    let mode = if review { "review" } else { "practice" };
    format!("{subject}/{topic}/{mode}")
}

pub fn load(path: &Path) -> io::Result<State> {
    match fs::read(path) {
        Ok(bytes) => {
            let mut state: State = serde_json::from_slice(&bytes)?;
            if let Some(session) = state.session.take() {
                let id = session_key(&session.subject, &session.topic, session.review);
                state.sessions.entry(id).or_insert(session);
            }
            if !state.history_initialized {
                for (id, sample) in &state.samples {
                    state
                        .history
                        .entry(id.clone())
                        .or_insert_with(|| vec![sample.clone()]);
                }
                state.history_initialized = true;
            }
            Ok(state)
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(State {
            history_initialized: true,
            ..State::default()
        }),
        Err(err) => Err(err),
    }
}

pub fn save(path: &Path, state: &State) -> io::Result<()> {
    let parent = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(&mut file, state)?;
    file.flush()?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|err| err.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviews_retry_and_advance_without_duplicating_rewards() {
        let mut state = State::default();
        state.finish("hi/neema", 100);
        state.finish("hi/neema", 101);
        assert_eq!(state.finished.len(), 1);
        assert_eq!(state.reviews["hi/neema"].due, 101);
        state.review("hi/neema", 101, false);
        assert_eq!(state.reviews["hi/neema"].due, 103);
        state.review("hi/neema", 103, true);
        assert_eq!(state.reviews["hi/neema"].due, 104);
        state.review("hi/neema", 104, false);
        assert_eq!(state.reviews["hi/neema"].due, 107);
        state.review("hi/neema", 107, false);
        assert!(state.reviews.is_empty());
    }

    #[test]
    fn progress_and_ink_survive_reopening() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut state = State::default();
        state.finish("hi/neema", 100);
        save(&path, &state).unwrap();
        state.samples.insert(
            "sample".into(),
            Sample {
                solution: None,
                correct: None,
                label: "स्वर".into(),
                paper: Paper::Hindi,
                model: "अ".into(),
                lines: 1,
                saved_on: None,
                ink: vec![vec![vec![[0.1, 0.2]]]],
            },
        );
        save(&path, &state).unwrap();
        let reopened = load(&path).unwrap();
        assert_eq!(reopened.finished.len(), 1);
        assert_eq!(reopened.samples["sample"].ink, state.samples["sample"].ink);
        assert_eq!(reopened.history["sample"].len(), 1);
        let mut reopened = reopened;
        reopened.samples.remove("sample");
        save(&path, &reopened).unwrap();
        assert_eq!(load(&path).unwrap().history["sample"].len(), 1);
    }
}

pub fn save_catalog(path: &Path, current: &State, catalog: CustomCatalog) -> io::Result<State> {
    let mut next = current.clone();
    next.catalog = catalog;
    save(path, &next)?;
    Ok(next)
}
