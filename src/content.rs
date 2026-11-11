mod hindi;
mod mathematics;

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    pub prompt: String,
    pub options: Vec<String>,
    pub answer: usize,
}

impl Question {
    pub fn pair(prompt: &str, first: &str, second: &str, answer: usize) -> Self {
        Self {
            prompt: prompt.into(),
            options: vec![first.into(), second.into()],
            answer,
        }
    }

    pub fn valid(&self) -> bool {
        !self.prompt.trim().is_empty()
            && (2..=8).contains(&self.options.len())
            && self.answer < self.options.len()
            && self.options.iter().enumerate().all(|(index, option)| {
                !option.trim().is_empty()
                    && !self.options[..index]
                        .iter()
                        .any(|previous| previous.trim() == option.trim())
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub grade: u8,
    pub group: Group,
    pub exercises: Vec<Exercise>,
    pub recall: Vec<Question>,
    #[serde(default)]
    pub break_every: Option<usize>,
    #[serde(default)]
    pub revision: u32,
}

impl Topic {
    pub fn progress_id(&self) -> String {
        if self.revision == 0 {
            self.id.clone()
        } else {
            format!("{}@{}", self.id, self.revision)
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Enter an activity title.".into());
        }
        if !matches!(self.grade, 1 | 2) {
            return Err("Choose Class 1 or 2.".into());
        }
        if self.exercises.is_empty() {
            return Err("Add at least one exercise.".into());
        }
        for (index, exercise) in self.exercises.iter().enumerate() {
            if !exercise.kind.valid() {
                return Err(format!(
                    "Complete all required fields in exercise {}.",
                    index + 1
                ));
            }
        }
        if self.recall.is_empty() || !self.recall.iter().all(Question::valid) {
            return Err(
                "Add at least one recall question with distinct answers and a correct choice."
                    .into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub title: String,
    pub guidance: String,
    pub separate_page: bool,
}

impl Group {
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            guidance: String::new(),
            separate_page: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Paper {
    #[default]
    Hindi,
    Digit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Choice(Question),
    Read(String, Question),
    Answer(String, Question),
    Trace {
        model: String,
        guidance: String,
        paper: Paper,
    },
    Compose(String, usize, Option<String>),
    Listen(String, Question),
    Talk {
        picture: String,
        prompt: String,
        starters: Vec<String>,
        adult: String,
    },
    Count {
        quantity: u8,
        visual: String,
        question: Question,
    },
}

pub struct Writing<'a> {
    pub label: &'a str,
    pub model: &'a str,
    pub lines: usize,
    pub paper: Paper,
}

impl Kind {
    pub fn writing(&self) -> Option<Writing<'_>> {
        let (label, model, lines, paper) = match self {
            Self::Trace { model, paper, .. } => (model.as_str(), model.as_str(), 1, *paper),
            Self::Compose(prompt, lines, _) => (prompt.as_str(), "", *lines, Paper::Hindi),
            Self::Talk { prompt, .. } => (prompt.as_str(), "", 1, Paper::Hindi),
            Self::Answer(_, question) => (question.prompt.as_str(), "", 1, Paper::Hindi),
            Self::Choice(_) | Self::Read(..) | Self::Listen(..) | Self::Count { .. } => {
                return None
            }
        };
        Some(Writing {
            label,
            model,
            lines,
            paper,
        })
    }

    fn valid(&self) -> bool {
        let filled = |text: &str| !text.trim().is_empty();
        match self {
            Self::Choice(question) => question.valid(),
            Self::Read(text, question)
            | Self::Listen(text, question)
            | Self::Answer(text, question) => filled(text) && question.valid(),
            Self::Trace { model, .. } => filled(model),
            Self::Compose(prompt, lines, _) => filled(prompt) && (1..=5).contains(lines),
            Self::Talk { prompt, .. } => filled(prompt),
            Self::Count {
                quantity,
                visual,
                question,
            } => (1..=20).contains(quantity) && filled(visual) && question.valid(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub id: String,
    pub name: String,
    pub topics: Vec<Topic>,
}

pub static BUNDLED: LazyLock<Vec<Subject>> = LazyLock::new(|| {
    vec![
        Subject {
            id: "hi".into(),
            name: "हिंदी".into(),
            topics: hindi::topics(),
        },
        Subject {
            id: "en".into(),
            name: "English".into(),
            topics: Vec::new(),
        },
        Subject {
            id: "math".into(),
            name: "गणित".into(),
            topics: mathematics::topics(),
        },
    ]
});

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomCatalog {
    pub subjects: Vec<Subject>,
    pub additions: std::collections::BTreeMap<String, Vec<Topic>>,
}

impl CustomCatalog {
    pub fn subjects(&self) -> impl Iterator<Item = &Subject> {
        BUNDLED.iter().chain(&self.subjects)
    }

    pub fn topics<'a>(&'a self, subject: &'a Subject) -> impl Iterator<Item = &'a Topic> {
        subject
            .topics
            .iter()
            .chain(self.additions.get(&subject.id).into_iter().flatten())
    }

    pub fn create_subject(&mut self, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Enter a subject name.".into());
        }
        self.subjects.push(Subject {
            id: new_id(),
            name: name.into(),
            topics: Vec::new(),
        });
        Ok(())
    }

    pub fn publish(&mut self, subject_id: &str, mut topic: Topic) -> Result<(), String> {
        topic.title = topic.title.trim().into();
        topic.validate()?;
        if !self.subjects().any(|subject| subject.id == subject_id) {
            return Err("Choose a subject.".into());
        }
        let topics = self.additions.entry(subject_id.into()).or_default();
        if let Some(previous) = topics.iter_mut().find(|previous| previous.id == topic.id) {
            let changed = previous.exercises != topic.exercises
                || previous.recall != topic.recall
                || previous.grade != topic.grade
                || previous.break_every != topic.break_every;
            topic.revision = previous
                .revision
                .checked_add(u32::from(changed))
                .ok_or("This activity has too many revisions.")?;
            *previous = topic;
        } else {
            topic.id = new_id();
            topic.revision = 0;
            topics.push(topic);
        }
        Ok(())
    }
}

pub fn new_id() -> String {
    format!("local-{:032x}", rand::random::<u128>())
}

pub fn review_questions(topic: &Topic, interval: Option<usize>) -> &[Question] {
    let Some(interval) = interval else {
        return &topic.recall[..1];
    };
    let remaining = topic.recall.len() - 1;
    let start = (remaining * interval.min(2)).div_ceil(3);
    let end = (remaining * (interval.min(2) + 1)).div_ceil(3);
    if start < end {
        &topic.recall[1 + start..1 + end]
    } else {
        let index = interval % topic.recall.len();
        &topic.recall[index..=index]
    }
}
