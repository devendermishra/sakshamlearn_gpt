mod common_words;
mod english;
mod hindi;
mod mathematics;
pub use mathematics::{generate_practice, GeneratedPractice, PracticeSkill};

use crate::language::Language;
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
            if matches!(exercise.kind, Kind::Revise(..))
                && !index.checked_sub(1).is_some_and(|previous| {
                    matches!(
                        self.exercises[previous].kind,
                        Kind::Compose(..) | Kind::Revise(..)
                    )
                })
            {
                return Err(format!(
                    "Put a writing exercise before revision exercise {}.",
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
    Notebook,
    NotebookCarry,
    NotebookBorrow,
    English,
}

impl Paper {
    pub fn for_subject(subject: &str) -> Self {
        if subject == "en" {
            Self::English
        } else {
            Self::Hindi
        }
    }

    pub fn regrouping_cells(self) -> usize {
        match self {
            Self::NotebookCarry => 1,
            Self::NotebookBorrow => 3,
            _ => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Choice(Question),
    Picture(Picture, Question),
    Read(String, Question),
    Answer(String, Question),
    Trace {
        model: String,
        guidance: String,
        paper: Paper,
    },
    Compose(String, usize, Option<String>),
    Revise(String, usize),
    Solve {
        prompt: String,
        solution: String,
        lines: usize,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        answer: Vec<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        helper: Option<ArithmeticHelp>,
        #[serde(default)]
        notebook: bool,
    },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Picture {
    Animals,
    Food,
    Play,
}

impl Picture {
    pub fn illustration(self) -> (&'static str, &'static str) {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        use std::sync::OnceLock;

        static ANIMALS: OnceLock<String> = OnceLock::new();
        static FOOD: OnceLock<String> = OnceLock::new();
        static PLAY: OnceLock<String> = OnceLock::new();
        // Mridang, Class 1 pp. 47 and 84 and Class 2 p. 8, NCERT.
        let (cache, jpeg, alt): (_, &[u8], _) = match self {
            Self::Animals => (&ANIMALS, include_bytes!("content/english/pictures/animals.jpg"),
                "Animals around a tree, including a monkey on the trunk and a lion."),
            Self::Food => (&FOOD, include_bytes!("content/english/pictures/food.jpg"),
                "Fruit and vegetables, including a watermelon slice, bananas and carrots."),
            Self::Play => (&PLAY, include_bytes!("content/english/pictures/play.jpg"),
                "Children playing outside. A child covers his eyes while another peeks from behind a tree."),
        };
        (
            cache.get_or_init(|| format!("data:image/jpeg;base64,{}", STANDARD.encode(jpeg))),
            alt,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Add,
    Subtract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArithmeticHelp {
    pub left: u8,
    pub right: u8,
    pub operation: Operation,
    pub place_value: bool,
}

impl ArithmeticHelp {
    pub fn notebook_paper(self) -> Paper {
        let (left, right) = (self.left % 10, self.right % 10);
        match self.operation {
            Operation::Add if left + right >= 10 => Paper::NotebookCarry,
            Operation::Subtract if left < right => Paper::NotebookBorrow,
            _ => Paper::Notebook,
        }
    }

    pub fn answer(self) -> u8 {
        match self.operation {
            Operation::Add => self.left + self.right,
            Operation::Subtract => self.left - self.right,
        }
    }
}

pub struct Writing<'a> {
    pub label: &'a str,
    pub model: &'a str,
    pub lines: usize,
    pub paper: Paper,
    pub solution: Option<&'a str>,
}

impl Kind {
    pub fn writing(&self, ruled_paper: Paper) -> Option<Writing<'_>> {
        let (label, model, lines, paper, solution) = match self {
            Self::Trace { model, paper, .. } => (model.as_str(), model.as_str(), 1, *paper, None),
            Self::Compose(prompt, lines, _) | Self::Revise(prompt, lines) => {
                (prompt.as_str(), "", *lines, ruled_paper, None)
            }
            Self::Talk { prompt, .. } => (prompt.as_str(), "", 1, ruled_paper, None),
            Self::Answer(_, question) => (question.prompt.as_str(), "", 1, ruled_paper, None),
            Self::Solve {
                prompt,
                solution,
                lines,
                notebook,
                helper,
                ..
            } => (
                prompt.as_str(),
                "",
                *lines,
                if *notebook {
                    helper.map_or(Paper::Notebook, ArithmeticHelp::notebook_paper)
                } else {
                    Paper::Digit
                },
                Some(solution.as_str()),
            ),
            Self::Choice(_)
            | Self::Picture(..)
            | Self::Read(..)
            | Self::Listen(..)
            | Self::Count { .. } => return None,
        };
        Some(Writing {
            label,
            model,
            lines,
            paper,
            solution,
        })
    }

    fn valid(&self) -> bool {
        let filled = |text: &str| !text.trim().is_empty();
        match self {
            Self::Choice(question) | Self::Picture(_, question) => question.valid(),
            Self::Read(text, question)
            | Self::Listen(text, question)
            | Self::Answer(text, question) => filled(text) && question.valid(),
            Self::Trace { model, .. } => filled(model),
            Self::Compose(prompt, lines, _) | Self::Revise(prompt, lines) => {
                filled(prompt) && (1..=5).contains(lines)
            }
            Self::Solve {
                prompt,
                solution,
                lines,
                helper,
                answer,
                notebook,
            } => {
                filled(prompt)
                    && filled(solution)
                    && (1..=5).contains(lines)
                    && (!*notebook || (*lines == 1 && helper.is_some()))
                    && helper.is_none_or(|helper| {
                        helper.left <= 99
                            && helper.right <= 99
                            && (helper.place_value || (helper.left <= 19 && helper.right <= 19))
                            && (helper.operation == Operation::Add || helper.left >= helper.right)
                            && answer.as_slice() == [helper.answer()]
                            && (!*notebook || helper.answer() < 100)
                    })
            }
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
            topics: common_words::hindi()
                .into_iter()
                .chain(hindi::topics())
                .collect(),
        },
        Subject {
            id: "en".into(),
            name: "English".into(),
            topics: english::topics()
                .into_iter()
                .chain(common_words::english())
                .collect(),
        },
        Subject {
            id: "math".into(),
            name: "गणित".into(),
            topics: mathematics::topics(Language::Hindi),
        },
    ]
});

pub fn english_math_topic(topic: &Topic) -> &Topic {
    static ENGLISH: LazyLock<Vec<Topic>> = LazyLock::new(|| mathematics::topics(Language::English));
    ENGLISH
        .iter()
        .find(|translation| translation.id == topic.id)
        .unwrap_or(topic)
}

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
