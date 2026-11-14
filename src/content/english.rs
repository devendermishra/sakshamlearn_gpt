mod reading;
mod words;
mod writing;

use super::{Exercise, Group, Kind, Question, Topic};

macro_rules! q {
    ($prompt:expr, $first:expr, $second:expr, $answer:expr) => {
        Question::pair($prompt, $first, $second, $answer)
    };
}
use q;

pub fn topics() -> Vec<Topic> {
    [1, 2]
        .into_iter()
        .flat_map(|grade| {
            words::topics(grade)
                .into_iter()
                .chain(reading::topics(grade))
                .chain(conversations(grade))
                .chain(writing::topics(grade))
        })
        .collect()
}

fn topic(
    id: &str,
    title: &str,
    grade: u8,
    group: Group,
    exercises: impl IntoIterator<Item = (&'static str, Kind)>,
    recall: [Question; 4],
) -> Topic {
    Topic {
        id: id.into(),
        title: title.into(),
        grade,
        group,
        exercises: exercises
            .into_iter()
            .map(|(id, kind)| Exercise {
                id: id.into(),
                kind,
            })
            .collect(),
        recall: recall.into(),
        break_every: Some(2),
        revision: 0,
    }
}

fn talk(picture: &str, prompt: &str, starters: &[&str], adult: &str) -> Kind {
    Kind::Talk {
        picture: picture.into(),
        prompt: prompt.into(),
        starters: starters.iter().map(|starter| (*starter).into()).collect(),
        adult: adult.into(),
    }
}

fn conversations(grade: u8) -> Vec<Topic> {
    let group = Group::new("conversation", "Listen and talk");
    if grade == 1 {
        vec![
            topic("en1-greet", "Say hello", grade, group.clone(), [
                ("listen", Kind::Listen("Good morning, Isha! I am Rani.".into(),
                    q!("What greeting did Rani use?", "Good night", "Good morning", 1))),
                ("greet", talk("👋", "Say hello. Ask your partner’s name.", &["Hello!", "What is your name?"],
                    "Say hello and give your name. Then ask the learner the same question. Pause for their reply.")),
                ("reply", talk("🙂", "What is your name? Say your answer aloud.", &["My name is …"],
                    "Say the starter together if needed. Then ask again so the learner can say the whole answer.")),
            ], [
                q!("Which question asks for a name?", "What is your name?", "Where is my bag?", 0),
                q!("You meet a friend before school. What can you say?", "Good night.", "Good morning.", 1),
                q!("How can you tell someone your name?", "My name is Ravi.", "I have a bag.", 0),
                q!("What can you say at bedtime?", "Good afternoon.", "Good night.", 1),
            ]),
            topic("en1-ask", "Ask for something", grade, group, [
                ("listen", Kind::Listen("Mina says, ‘May I have some water, please?’ Dev gives her a cup. Mina says, ‘Thank you.’".into(),
                    q!("What does Mina ask for?", "A book", "Water", 1))),
                ("request", talk("✏️", "Ask your partner for a pencil.", &["May I have a pencil, please?", "Thank you."],
                    "Hold a pencil. Wait for the request, hand it over, and pause for the learner to thank you. Swap roles.")),
                ("where", talk("🎒", "Ask where your bag is. Then answer your partner.", &["Where is my bag?", "It is on the …"],
                    "Put the bag on a chair or table. Use the real location in your answer. Swap roles.")),
            ], [
                q!("Which is a polite request?", "May I have a pencil, please?", "I see a pencil.", 0),
                q!("Someone gives you a cup. What can you say?", "Good night.", "Thank you.", 1),
                q!("What can you ask to find your book?", "Where is my book?", "What is your name?", 0),
                q!("‘Where is my cap?’ Which reply helps?", "My cap is red.", "It is on the chair.", 1),
            ]),
        ]
    } else {
        vec![
            topic("en2-directions", "Ask for directions", grade, group.clone(), [
                ("listen", Kind::Listen("Walk past the tree. Then turn left. The school is beside the park.".into(),
                    q!("What comes just after you pass the tree?", "Turn right.", "Turn left.", 1))),
                ("ask", talk("🏫", "Ask how to get to the school.", &["Excuse me. How do I get to the school?", "Is it beside the park?"],
                    "Use a quick sketch with a school, a park and a tree. Give two short directions. Ask the learner to repeat them in order.")),
                ("give", talk("🌳", "Give your partner two directions.", &["First, go past …", "Then, turn …"],
                    "Follow the learner’s instructions on the sketch. Ask a follow-up if a direction is missing. Swap roles.")),
            ], [
                q!("Which question asks for directions?", "How do I get to the park?", "Do you like the park?", 0),
                q!("‘First, go straight. Then turn right.’ What do you do first?", "Turn right.", "Go straight.", 1),
                q!("A friend says, ‘The shop is beside the park.’ Where is the shop?", "Next to the park", "Inside the park", 0),
                q!("Which word tells what to do next?", "First", "Then", 1),
            ]),
            topic("en2-interview", "Ask a follow-up question", grade, group, [
                ("listen", Kind::Listen("Asha says, ‘I like drawing. Yesterday I drew a bird.’ Omar asks, ‘What colour was it?’".into(),
                    q!("What does Omar ask about?", "The bird’s colour", "Asha’s age", 0))),
                ("interview", talk("🎨", "Ask what your partner likes doing. Ask one more question about their answer.", &["What do you like to do?", "When do you do it?", "Who does it with you?"],
                    "Answer with a familiar activity. Wait for a related question, then give one detail. Switch roles and ask the learner to answer in a full sentence.")),
                ("retell", talk("💬", "Tell your partner one thing you learned about them.", &["You like …", "You do it with …"],
                    "Prompt the learner to say one accurate detail from memory. Ask them to write the detail if they want more practice.")),
            ], [
                q!("‘I like to read.’ Which follow-up fits?", "What do you like to read?", "Where is my shoe?", 0),
                q!("‘I play on Sunday.’ Which question would get that answer?", "Who do you play with?", "When do you play?", 1),
                q!("‘My sister comes with me.’ Who comes along?", "My sister", "My brother", 0),
                q!("‘I like to draw flowers.’ Which reply stays on topic?", "Is that your bag?", "Which flowers do you draw?", 1),
            ]),
        ]
    }
}
