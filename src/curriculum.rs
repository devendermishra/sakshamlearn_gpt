use crate::content::{Kind, Topic};

fn chapter_id(subject: &str, topic: &Topic) -> &'static str {
    match (subject, topic.group.id.as_str()) {
        ("hi", "writing") if topic.grade == 1 => match topic.id.as_str() {
            id if id.starts_with("vowels-") => "vowels",
            "consonants" | "nukta" => "consonants",
            "matra-forms" | "ri-one" | "ri-two" | "ri-three" => "matras",
            _ => "writing",
        },
        ("hi", "barakhadi") => "consonants",
        ("hi", "writing") => "writing",
        ("hi", "lesson") => "reading",
        ("en", "alphabet") => "alphabet",
        ("en", "lowercase") => "lowercase",
        ("en", "word-sounds" | "common-words" | "word-writing") => "words",
        ("en", "reading") => "reading",
        ("en", "writing-plan") => "writing",
        ("hi" | "en", "conversation") => "conversation",
        ("hi" | "en", "everyday-words") => "words",
        _ => "",
    }
}

pub fn chapter<'a>(subject: &str, topic: &'a Topic) -> (&'a str, &'a str) {
    match (subject, chapter_id(subject, topic)) {
        ("hi", "vowels") => ("vowels", "स्वर"),
        ("hi", "consonants") => ("consonants", "व्यंजन और बारहखड़ी"),
        ("hi", "matras") => ("matras", "मात्राएँ"),
        ("hi", "words" | "writing") if topic.grade == 1 => ("writing", "शब्द और लिखाई"),
        ("hi", "words") => ("words", "रोज़ के शब्द"),
        ("hi", "writing") => ("writing", "लिखने का अभ्यास"),
        ("hi", "reading") => ("reading", "पढ़ो"),
        ("hi", "conversation") => ("conversation", "सुनो और बात करो"),
        ("en", "alphabet") => ("alphabet", "Trace the alphabet"),
        ("en", "lowercase") => ("lowercase", "Write small letters"),
        ("en", "words") => ("words", "Sounds and words"),
        ("en", "reading") => ("reading", "Read and understand"),
        ("en", "conversation") => ("conversation", "Listen and talk"),
        ("en", "writing") => ("writing", "Writing practice plan"),
        _ => (&topic.group.id, &topic.group.title),
    }
}

pub fn order(subject: &str, topic: &Topic) -> usize {
    let sequence: &[&str] = match subject {
        "hi" => &["vowels", "consonants", "matras", "words", "writing", "reading", "conversation"],
        "en" => &["alphabet", "lowercase", "words", "reading", "conversation", "writing"],
        "math" => &["numbers", "add-subtract", "groups", "tables", "division", "math-writing"],
        _ => &[],
    };
    sequence.iter().position(|id| *id == chapter(subject, topic).0).unwrap_or(sequence.len())
}

pub fn letter_model<'a>(subject: &str, topic: &'a Topic, index: usize) -> Option<&'a str> {
    let chapter = chapter_id(subject, topic);
    if topic.grade != 1 || !matches!((subject, chapter),
        ("hi", "vowels" | "consonants" | "matras") | ("en", "alphabet" | "lowercase")) {
        return None;
    }
    match &topic.exercises.get(index)?.kind {
        Kind::Trace { model, .. } => Some(model),
        _ => None,
    }
}
