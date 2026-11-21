use super::{
    division_example, numeric_choice, topic, writing_practice, Group, Kind, Language, Question,
    Topic,
};
use crate::content::{ArithmeticHelp, Operation};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PracticeSkill {
    Counting,
    Addition,
    Subtraction,
    Carrying,
    Borrowing,
    Tables,
    Division,
}

impl PracticeSkill {
    pub fn for_grade(grade: u8) -> &'static [Self] {
        use PracticeSkill::*;
        match grade {
            1 => &[Counting, Addition, Subtraction],
            2 => &[
                Counting,
                Addition,
                Subtraction,
                Carrying,
                Borrowing,
                Tables,
                Division,
            ],
            _ => &[],
        }
    }

    pub fn label(self, language: Language) -> &'static str {
        match self {
            Self::Counting => language.text("गिनती", "Counting"),
            Self::Addition => language.text("जोड़", "Addition"),
            Self::Subtraction => language.text("घटाव", "Subtraction"),
            Self::Carrying => language.text("दहाई आगे ले जाकर जोड़", "Addition with carrying"),
            Self::Borrowing => language.text("दहाई खोलकर घटाव", "Subtraction with borrowing"),
            Self::Tables => language.text("पहाड़े 1 से 10", "Tables 1 to 10"),
            Self::Division => {
                language.text("बार-बार घटाकर भाग", "Division by repeated subtraction")
            }
        }
    }

    pub fn id(self, grade: u8) -> String {
        let name = match self {
            Self::Counting => "counting",
            Self::Addition => "addition",
            Self::Subtraction => "subtraction",
            Self::Carrying => "carrying",
            Self::Borrowing => "borrowing",
            Self::Tables => "tables",
            Self::Division => "division",
        };
        format!("new-practice-{grade}-{name}")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GeneratedPractice {
    hindi: Topic,
    english: Topic,
}

impl GeneratedPractice {
    pub fn topic(&self, language: Language) -> &Topic {
        match language {
            Language::Hindi => &self.hindi,
            Language::English => &self.english,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Problem {
    Objects(u8),
    Sequence { first: u8, step: i8 },
    Arithmetic(ArithmeticHelp),
    Product { size: u8, groups: u8 },
    Quotient { size: u8, groups: u8 },
}

impl Problem {
    fn prompt(self, language: Language) -> String {
        match self {
            Self::Objects(_) => language
                .text("बिंदु गिनो। कितने हैं?", "Count the dots. How many are there?")
                .into(),
            Self::Sequence { first, step } => {
                let next = first.checked_add_signed(step).unwrap();
                let last = next.checked_add_signed(2 * step).unwrap();
                format!("{first}, {next}, __, {last}")
            }
            Self::Arithmetic(help) => {
                let sign = if help.operation == Operation::Add {
                    '+'
                } else {
                    '−'
                };
                format!("{} {sign} {} = __", help.left, help.right)
            }
            Self::Product { size, groups } => format!("{size} × {groups} = __"),
            Self::Quotient { size, groups } => format!("{} ÷ {size} = __", size * groups),
        }
    }

    fn answer(self) -> u8 {
        match self {
            Self::Objects(amount) => amount,
            Self::Sequence { first, step } => first.checked_add_signed(2 * step).unwrap(),
            Self::Arithmetic(help) => help.answer(),
            Self::Product { size, groups } => size * groups,
            Self::Quotient { groups, .. } => groups,
        }
    }

    fn question(
        self,
        language: Language,
        answer_first: bool,
        prefer_higher: bool,
        recall: bool,
    ) -> Question {
        let answer = self.answer();
        let distance = match self {
            Self::Product { size, .. } => size,
            _ => 1,
        };
        let lower = answer.saturating_sub(distance);
        let higher = (answer + distance).min(100);
        let other = if (prefer_higher && higher != answer) || lower == answer {
            higher
        } else {
            lower
        };
        let mut prompt = self.prompt(language);
        if let Self::Objects(amount) = self {
            if recall {
                prompt.push(' ');
                prompt.push_str("● ".repeat(usize::from(amount)).trim_end());
            }
        }
        numeric_choice(&prompt, answer, other, answer_first)
    }

    fn exercise(self, language: Language, answer_first: bool, prefer_higher: bool) -> Kind {
        match self {
            Self::Objects(quantity) => Kind::Count {
                quantity,
                visual: "●".into(),
                question: self.question(language, answer_first, prefer_higher, false),
            },
            Self::Arithmetic(help) if help.place_value => {
                writing_practice::notebook_problem(help, language)
            }
            Self::Arithmetic(help) => {
                let sign = if help.operation == Operation::Add {
                    '+'
                } else {
                    '−'
                };
                writing_practice::arithmetic(help.left, sign, help.right, help.answer(), 2)
            }
            Self::Quotient { size, groups } => {
                let equation = self.prompt(language);
                let instruction =
                    language.text("बार-बार घटाकर भाग दो।", "Divide by repeated subtraction.");
                writing_practice::solve(
                    &format!("{instruction} {equation}"),
                    &division_example(language, size, groups),
                    3,
                    &[groups],
                )
            }
            Self::Product { size, .. } => Kind::Table { size, writing: false },
            _ => Kind::Choice(self.question(language, answer_first, prefer_higher, false)),
        }
    }
}

fn candidates(grade: u8, skill: PracticeSkill) -> Vec<Problem> {
    match skill {
        PracticeSkill::Counting => {
            let mut problems: Vec<_> = (1..=if grade == 1 { 12 } else { 20 })
                .map(Problem::Objects)
                .collect();
            let max = if grade == 1 { 20 } else { 100 };
            let strides: &[i8] = if grade == 1 {
                &[1, -1]
            } else {
                &[1, -1, 2, -2, 5, -5, 10, -10]
            };
            for &step in strides {
                for first in 0u8..=max {
                    if first
                        .checked_add_signed(3 * step)
                        .is_some_and(|last| last <= max)
                    {
                        problems.push(Problem::Sequence { first, step });
                    }
                }
            }
            problems
        }
        PracticeSkill::Tables => (1..=10)
            .map(|size| Problem::Product { size, groups: 1 })
            .collect(),
        PracticeSkill::Division => (2..=10)
            .flat_map(|size| (2..=10).map(move |groups| Problem::Quotient { size, groups }))
            .collect(),
        skill => {
            let addition = matches!(skill, PracticeSkill::Addition | PracticeSkill::Carrying);
            let regroup = matches!(skill, PracticeSkill::Carrying | PracticeSkill::Borrowing);
            let max_operand = if grade == 1 { 19 } else { 99 };
            let max_answer = if grade == 1 { 20 } else { 99 };
            let operation = if addition {
                Operation::Add
            } else {
                Operation::Subtract
            };
            let mut problems = Vec::new();
            for left in 1..=max_operand {
                for right in 1..=max_operand {
                    if (addition && left + right > max_answer) || (!addition && right > left) {
                        continue;
                    }
                    let helper = ArithmeticHelp {
                        left,
                        right,
                        operation,
                        place_value: grade == 2,
                    };
                    if grade == 1 || (helper.notebook_paper().regrouping_cells() > 0) == regroup {
                        problems.push(Problem::Arithmetic(helper));
                    }
                }
            }
            problems
        }
    }
}

pub fn generate_practice(grade: u8, skill: PracticeSkill, seed: u64) -> Option<GeneratedPractice> {
    if !PracticeSkill::for_grade(grade).contains(&skill) {
        return None;
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let candidates = candidates(grade, skill);
    let mut problems = Vec::new();
    if skill == PracticeSkill::Counting {
        // Always include a chance to count objects as well as number-sequence practice.
        let objects: Vec<_> = candidates
            .iter()
            .copied()
            .filter(|problem| matches!(problem, Problem::Objects(_)))
            .collect();
        let sequences: Vec<_> = candidates
            .iter()
            .copied()
            .filter(|problem| matches!(problem, Problem::Sequence { .. }))
            .collect();
        problems.extend(objects.choose_multiple(&mut rng, 2).copied());
        problems.extend(sequences.choose_multiple(&mut rng, 4).copied());
    } else {
        problems.extend(candidates.choose_multiple(&mut rng, 6).copied());
    }
    problems.shuffle(&mut rng);
    let recall = problems.pop()?;
    let mut positions = [false, true, false, true, false, true];
    let mut higher = positions;
    positions.shuffle(&mut rng);
    higher.shuffle(&mut rng);
    let choices: Vec<_> = problems
        .into_iter()
        .enumerate()
        .map(|(index, problem)| (problem, positions[index], higher[index]))
        .collect();
    let recall_position = positions[5];
    let recall_higher = higher[5];
    let localize = |language: Language| {
        let group = Group::new("new-practice", language.text("नया अभ्यास", "New practice"));
        let items = choices
            .iter()
            .map(|&(problem, position, higher)| problem.exercise(language, position, higher))
            .collect();
        let mut set = topic(
            grade,
            &skill.id(grade),
            skill.label(language),
            group,
            items,
            vec![recall.question(language, recall_position, recall_higher, true)],
        );
        for exercise in &mut set.exercises {
            exercise.id = format!("set-{seed:016x}-{}", exercise.id);
        }
        set
    };
    Some(GeneratedPractice {
        hindi: localize(Language::Hindi),
        english: localize(Language::English),
    })
}
