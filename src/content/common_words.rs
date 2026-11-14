mod english;
mod hindi;

use super::{Exercise, Group, Kind, Paper, Question, Topic};

macro_rules! card {
    ($id:literal, $title:literal, $grade:literal, $words:expr, $text:literal;
        $($prompt:literal => $first:literal, $second:literal, $answer:literal);+ $(;)?) => {
        super::topic($id, $title, $grade, GROUP, $words, $text,
            [$(Question::pair($prompt, $first, $second, $answer)),+])
    };
}
use card;

pub fn english() -> Vec<Topic> {
    english::topics()
}

pub fn hindi() -> Vec<Topic> {
    hindi::topics()
}

fn topic(
    id: &str,
    title: &str,
    grade: u8,
    (group_title, paper): (&str, Paper),
    words: [&str; 4],
    passage: &str,
    [check, first, second, third, fourth]: [Question; 5],
) -> Topic {
    let mut group = Group::new("everyday-words", group_title);
    group.separate_page = true;
    let english = paper == Paper::English;
    group.guidance = if english {
        "Read the words in a sentence, trace them, and write them from dictation. Do one card at a time."
    } else {
        "वाक्य पढ़ो, शब्दों पर लिखो और फिर सुनकर लिखो। एक बार में एक अभ्यास करो।"
    }.into();
    let word_line = words.join(" · ");
    let mut exercises = vec![Exercise {
        id: "read".into(),
        kind: Kind::Read(format!("{word_line}\n{passage}"), check),
    }];
    for (index, word) in words.iter().enumerate() {
        exercises.push(Exercise {
            id: format!("word-{}", index + 1),
            kind: Kind::Trace {
                model: (*word).into(),
                guidance: if english {
                    "Read the word aloud, then trace it."
                } else {
                    "शब्द बोलो और बिंदियों पर लिखो।"
                }
                .into(),
                paper,
            },
        });
    }
    let prompt = if english {
        "Listen and write one word on each line."
    } else {
        "सुनो और हर पंक्ति पर एक शब्द लिखो।"
    };
    let script = if english {
        format!("Say ‘{}’. Pause. Then say ‘{}’.", words[1], words[3])
    } else {
        format!("पहले ‘{}’ बोलें। थोड़ा रुकें, फिर ‘{}’ बोलें।", words[1], words[3])
    };
    exercises.push(Exercise {
        id: "dictation".into(),
        kind: Kind::Compose(prompt.into(), 2, Some(script)),
    });
    Topic {
        id: id.into(),
        title: title.into(),
        grade,
        group,
        exercises,
        recall: vec![first, second, third, fourth],
        break_every: Some(2),
        revision: 0,
    }
}
