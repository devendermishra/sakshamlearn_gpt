use super::{Exercise, Group, Kind, Paper, Question, Topic};

pub fn topics() -> Vec<Topic> {
    [(1, "🍎"), (4, "⭐"), (7, "🔵")]
        .into_iter()
        .map(|(start, visual)| {
            let mut exercises = Vec::new();
            let mut recall = Vec::new();
            for number in start..start + 3 {
                let digit = number.to_string();
                let other = (if number == 9 { 8 } else { number + 1 }).to_string();
                let options = if number % 2 == 0 {
                    [&other, &digit]
                } else {
                    [&digit, &other]
                };
                let question = Question::pair(
                    "गिनो और सही अंक चुनो।",
                    options[0],
                    options[1],
                    usize::from(number % 2 == 0),
                );
                exercises.push(Exercise {
                    id: format!("count-{number}"),
                    kind: Kind::Count {
                        quantity: number,
                        visual: visual.into(),
                        question,
                    },
                });
                exercises.push(Exercise {
                    id: format!("write-{number}"),
                    kind: Kind::Trace {
                        model: digit.clone(),
                        guidance: "एक खाने में एक अंक लिखो।".into(),
                        paper: Paper::Digit,
                    },
                });
                let number_names = ["एक", "दो", "तीन", "चार", "पाँच", "छह", "सात", "आठ", "नौ"];
                recall.push(Question::pair(
                    &format!(
                        "‘{}’ के लिए सही अंक चुनो।",
                        number_names[usize::from(number - 1)]
                    ),
                    options[0],
                    options[1],
                    usize::from(number % 2 == 0),
                ));
            }
            Topic {
                id: format!("numbers-{start}-{}", start + 2),
                title: format!("गिनो और लिखो · {start}–{}", start + 2),
                grade: 1,
                group: Group::new("numbers", "संख्याएँ"),
                exercises,
                recall,
                break_every: None,
                revision: 0,
            }
        })
        .collect()
}
