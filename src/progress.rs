use crate::{
    content::{self, Kind, Paper, Topic},
    state::{self, HelperDraft, Ink, Sample, State},
};

pub enum Input {
    Answer(bool),
    Draft(Ink),
    Helper(HelperDraft),
    Submit(Ink),
    SubmitSolution { ink: Ink, correct: bool },
}

impl Input {
    pub fn is_draft(&self) -> bool {
        matches!(self, Self::Draft(_) | Self::Helper(_))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Break,
    Done(bool),
}

pub struct Step<'a> {
    pub subject: &'a str,
    pub topic: &'a Topic,
    pub review: bool,
    pub index: usize,
}

impl Step<'_> {
    pub fn key(&self) -> String {
        format!("{}/{}", self.subject, self.topic.progress_id())
    }
    pub fn session_key(&self) -> String {
        state::session_key(self.subject, &self.topic.progress_id(), self.review)
    }
    pub fn exercise(&self) -> Option<&Kind> {
        if self.review {
            None
        } else {
            self.topic
                .exercises
                .get(self.index)
                .map(|exercise| &exercise.kind)
        }
    }
    fn sample_key(&self) -> Option<String> {
        self.exercise()?;
        Some(format!(
            "{}/{}",
            self.key(),
            self.topic.exercises[self.index].id
        ))
    }
    pub fn ink(&self, state: &State) -> Ink {
        self.sample_key()
            .and_then(|id| state.samples.get(&id))
            .map(|sample| sample.ink.clone())
            .unwrap_or_default()
    }
    pub fn revision_source<'a>(&self, state: &'a State) -> Option<(&'a Ink, usize)> {
        if !matches!(self.exercise(), Some(Kind::Revise(..))) {
            return None;
        }
        let previous = self.topic.exercises.get(self.index.checked_sub(1)?)?;
        let sample = state
            .history
            .get(&format!("{}/{}", self.key(), previous.id))?
            .last()?;
        Some((&sample.ink, sample.lines))
    }
    pub fn recall(&self, state: &State) -> &[content::Question] {
        let interval = if self.review {
            state.reviews.get(&self.key()).map(|review| review.interval)
        } else {
            None
        };
        content::review_questions(self.topic, interval)
    }
    pub fn apply(
        &self,
        state: &mut State,
        input: Input,
        today: u64,
        date: &str,
    ) -> Option<Outcome> {
        let id = self.session_key();
        let current = state.sessions.get_mut(&id)?;
        if current.step != self.index {
            return None;
        }
        if let Input::Helper(helper) = input {
            if matches!(
                self.exercise(),
                Some(Kind::Solve {
                    helper: Some(_),
                    ..
                })
            ) {
                current.helper = Some(helper);
                return Some(Outcome::Continue);
            }
            return None;
        }
        let generated = current.generated.is_some();
        let missed = current.missed
            || matches!(
                input,
                Input::Answer(true) | Input::SubmitSolution { correct: false, .. }
            );
        let writing_input = match input {
            Input::Answer(_) | Input::Helper(_) => None,
            Input::Draft(ink) => Some((ink, false, None)),
            Input::Submit(ink) => Some((ink, true, None)),
            Input::SubmitSolution { ink, correct } => Some((ink, true, Some(correct))),
        };
        if let Some((ink, submitted, correct)) = writing_input {
            let writing = self.exercise()?.writing(Paper::for_subject(self.subject))?;
            let sample_id = self.sample_key()?;
            if submitted
                && writing.paper == Paper::English
                && writing.lines > 1
                && (ink.len() != writing.lines || ink.iter().any(Vec::is_empty))
            {
                return None;
            }
            if ink.iter().any(|cell| !cell.is_empty()) {
                let sample = Sample {
                    label: format!("{} · {}", self.topic.title, writing.label),
                    model: writing.model.into(),
                    solution: writing.solution.map(str::to_owned),
                    correct,
                    lines: writing.lines,
                    paper: writing.paper,
                    ink,
                    saved_on: submitted.then(|| date.to_owned()),
                };
                if submitted {
                    state.samples.remove(&sample_id);
                    state.history.entry(sample_id).or_default().push(sample);
                } else {
                    state.samples.insert(sample_id, sample);
                }
            } else if submitted {
                return None;
            } else {
                state.samples.remove(&sample_id);
            }
            if !submitted {
                return Some(Outcome::Continue);
            }
        }
        let more = if self.review {
            self.index + 1 < self.recall(state).len()
        } else {
            self.exercise().is_some()
        };
        if more {
            let current = state.sessions.get_mut(&id)?;
            current.step += 1;
            current.helper = None;
            current.missed = missed;
            let pause = !self.review
                && current.step < self.topic.exercises.len()
                && self
                    .topic
                    .break_every
                    .is_some_and(|interval| interval > 0 && current.step % interval == 0);
            return Some(if pause {
                Outcome::Break
            } else {
                Outcome::Continue
            });
        }
        let key = self.key();
        let awarded = !generated && !self.review && !state.finished.contains(&key);
        if self.review {
            state.review(&key, today, missed);
        } else if !generated {
            state.finish(&key, today);
        }
        state.sessions.remove(&id);
        Some(Outcome::Done(awarded))
    }
}
