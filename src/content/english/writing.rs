use super::{q, topic, Group, Kind, Question, Topic};
use crate::content::Paper;

pub(super) fn trace(model: &str, guidance: &str) -> Kind {
    Kind::Trace {
        model: model.into(),
        guidance: guidance.into(),
        paper: Paper::English,
    }
}

pub(super) fn write(prompt: &str, lines: usize) -> Kind {
    Kind::Compose(prompt.into(), lines, None)
}

pub(super) fn dictation(prompt: &str, script: &str, lines: usize) -> Kind {
    Kind::Compose(prompt.into(), lines, Some(script.into()))
}

pub(super) fn topics(grade: u8) -> Vec<Topic> {
    let mut group = Group::new("writing-plan", "Writing practice plan");
    group.separate_page = true;
    group.guidance = if grade == 1 {
        "Try one card a day for about five minutes. Say it, trace it, then write it on the blank row. Choose your clearest attempt. Check just one thing each day."
    } else {
        "Try one card a day for five to seven minutes. Read it aloud, trace or copy it, then write from memory. Reread your work and fix one thing. For joined letters, use the cursive workbook with an adult."
    }.into();
    if grade == 1 {
        let mut topics = class_one(group);
        topics.extend(alphabet());
        topics.extend(lowercase());
        topics
    } else {
        class_two(group)
    }
}

fn lowercase() -> Vec<Topic> {
    let mut group = Group::new("lowercase", "Write small letters");
    group.separate_page = true;
    group.guidance = "Choose two small letters to practise. Say each letter, trace it, and write it on the blank row.".into();
    [
        ("a", "b", "Keep a between the blue lines. Follow the dots around the curved part.", "Start at the top red line. Draw down, then make a round loop to the right."),
        ("c", "d", "Curve left around and leave the right side open.", "Make the round part first, then draw the tall stroke on its right."),
        ("e", "f", "Start in the middle, go right, then curve back around.", "Curve at the top and come down to the lower blue line. Add the short crossbar."),
        ("g", "h", "Follow the dotted model. Keep the top of g between the blue lines and finish below them.", "Draw the tall stroke, then make one hump between the blue lines."),
        ("i", "j", "Make a short stroke between the blue lines. Add a dot.", "Take the stroke below the blue line and curve left. Add a dot."),
        ("k", "l", "Make a tall stroke. Bring a short slant in, then take a slant out.", "Pull straight down from the top red line to the lower blue line."),
        ("m", "n", "Draw a short downstroke and two humps between the blue lines.", "Draw a short downstroke and one hump between the blue lines."),
        ("o", "p", "Make a closed round shape between the blue lines.", "Draw the stem below the lower blue line and put the round part on its right."),
        ("q", "r", "Make a round shape, then take the stem down on the right.", "Make a short downstroke, then curve right at the top."),
        ("s", "t", "Make a small curve one way, then curve back the other way.", "Draw down, then add a short crossbar."),
        ("u", "v", "Curve down and back up, then add a short downstroke on the right.", "Slant down to the lower blue line, then slant up."),
        ("w", "x", "Make two shallow v shapes without lifting your pen.", "Draw one slant, then cross it with a second slant."),
        ("y", "z", "Draw the short slant, then take the long slant below the lower blue line.", "Go right, slant down to the left, then go right again."),
    ]
    .into_iter()
    .map(|(first, second, first_hint, second_hint)| {
        let first_capital = first.to_ascii_uppercase();
        let second_capital = second.to_ascii_uppercase();
        topic(
            &format!("en1-small-{first}-{second}"),
            &format!("Write {first} and {second}"),
            1,
            group.clone(),
            [
                ("first", trace(first, first_hint)),
                ("second", trace(second, second_hint)),
                ("write", write(&format!("Write {first} on the first row and {second} on the second row."), 2)),
            ],
            [
                q!(&format!("Which is the small letter for {first_capital}?"), second, first, 1),
                q!(&format!("Which is the small letter for {second_capital}?"), second, first, 0),
                q!(&format!("Which is lowercase, {first_capital} or {first}?"), &first_capital, first, 1),
                q!(&format!("Which is lowercase, {second} or {second_capital}?"), second, &second_capital, 0),
            ],
        )
    })
    .collect()
}

fn class_one(group: Group) -> Vec<Topic> {
    macro_rules! day {
        ($id:literal, $title:literal, $first:expr, $second:expr, $third:expr; $($recall:expr),+ $(,)?) => {
            topic($id, $title, 1, group.clone(), [
                ("model", $first), ("practise", $second), ("apply", $third),
            ], [$($recall),+])
        };
    }
    vec![
        day!("en1-write-1", "Day 1 · Tall and small letters",
            trace("l", "Start at the top red line. Pull straight down to the lower blue line."),
            trace("i", "Start at the upper blue line. Pull down to the lower blue line. Add the dot."),
            write("Write l and i from memory. Check which one is taller.", 1);
            q!("Which letter is taller?", "l", "i", 0),
            q!("Which letter needs a dot?", "l", "i", 1),
            q!("Which guide should the bottom of l touch?", "The lower blue line", "The top red line", 0),
            q!("Which has a short stroke and a dot?", "l", "i", 1),
        ),
        day!("en1-write-2", "Day 2 · Round letters and tails",
            trace("o", "Curve around to make a closed round shape between the blue lines."),
            trace("p", "Take the stem below the lower blue line. Make the round part between the blue lines."),
            write("Write o and p. Check that only p has a stem below the lower blue line.", 1);
            q!("Which letter has a stem that goes down below the blue line?", "o", "p", 1),
            q!("Which letter is a closed round shape?", "o", "p", 0),
            q!("Which letter begins the word pot?", "o", "p", 1),
            q!("Where does the round part of p go?", "Between the blue lines", "Below both blue lines", 0),
        ),
        day!("en1-write-3", "Day 3 · Label an object",
            trace("cap", "Say the sounds in cap. Keep the letters close together inside the word."),
            write("Look at a hat or cap. Say ‘cap’. Copy the label cap.", 1),
            dictation("Listen. Write one word. Say each sound before you write.", "Say ‘cap’. Use it in ‘I wear a cap.’ Say ‘cap’ once more.", 1);
            q!("Which label names something you wear on your head?", "cap", "cup", 0),
            q!("Which letter comes last in cap?", "c", "p", 1),
            q!("Which word rhymes with cap?", "tap", "top", 0),
            q!("Which letter comes first in cap?", "a", "c", 1),
        ),
        day!("en1-write-4", "Day 4 · Leave a word space",
            trace("red", "Read red aloud. Keep its three letters together."),
            write("Copy red cap. Leave one finger space between the two words.", 1),
            dictation("Listen and write two words. Check the space.", "Say ‘red cap’. Pause. Say ‘red cap’ again.", 1);
            q!("Which has a space between its two words?", "redcap", "red cap", 1),
            q!("How many words are in ‘red cap’?", "Two", "One", 0),
            q!("Where does the space go in ‘blue bag’?", "Between b and l", "Between blue and bag", 1),
            q!("Which label has a clear space?", "big bag", "bigbag", 0),
        ),
        day!("en1-write-5", "Day 5 · Write a sentence",
            write("Read, then copy ‘I see a cap.’ Start with a capital I.", 1),
            write("Say a new sentence using ‘I see a …’. Write it about something you can see.", 1),
            Kind::Revise("Read your sentence. Write it again with clear spaces and a full stop.".into(), 1);
            q!("Which sentence begins with a capital and ends with a full stop?", "I see a cap.", "i see a cap", 0),
            q!("What goes at the end of ‘I see a bag’?", "A comma", "A full stop", 1),
            q!("Which begins with a capital letter?", "I see a bird.", "i see a bird.", 0),
            q!("Which sentence has clear word spaces?", "Isee acat.", "I see a cat.", 1),
        ),
    ]
}

fn alphabet() -> Vec<Topic> {
    let mut group = Group::new("alphabet", "Trace the alphabet");
    group.separate_page = true;
    group.guidance = "Say each letter name. Trace the capital and small letters, then try each on the blank row. Take a short break after two writing cards.".into();
    ('a'..='z')
        .step_by(2)
        .map(|first| {
            let second = char::from(first as u8 + 1);
            let capital_first = first.to_ascii_uppercase().to_string();
            let capital_second = second.to_ascii_uppercase().to_string();
            let first = first.to_string();
            let second = second.to_string();
            topic(
                &format!("en1-alphabet-{first}-{second}"),
                &format!("{capital_first} {first} and {capital_second} {second}"),
                1,
                group.clone(),
                [
                    (
                        "first-capital",
                        trace(&capital_first, "Start near the top red line."),
                    ),
                    (
                        "first-small",
                        trace(&first, "Watch where the letter meets the blue lines."),
                    ),
                    (
                        "second-capital",
                        trace(&capital_second, "Start near the top red line."),
                    ),
                    (
                        "second-small",
                        trace(&second, "Watch where the letter meets the blue lines."),
                    ),
                ],
                [
                    q!(
                        &format!("Which is the small letter for {capital_first}?"),
                        &first,
                        &second,
                        0
                    ),
                    q!(
                        &format!("Which is the capital letter for {second}?"),
                        &capital_first,
                        &capital_second,
                        1
                    ),
                    q!(
                        &format!("Which is the capital letter for {first}?"),
                        &capital_first,
                        &capital_second,
                        0
                    ),
                    q!(
                        &format!("Which is the small letter for {capital_second}?"),
                        &first,
                        &second,
                        1
                    ),
                ],
            )
        })
        .collect()
}

fn class_two(group: Group) -> Vec<Topic> {
    macro_rules! day {
        ($id:literal, $title:literal, $first:expr, $second:expr, $third:expr; $($recall:expr),+ $(,)?) => {
            topic($id, $title, 2, group.clone(), [
                ("model", $first), ("practise", $second), ("apply", $third),
            ], [$($recall),+])
        };
    }
    vec![
        day!("en2-write-1", "Day 1 · Letter height",
            trace("h", "Start the tall stroke near the top red line. Keep the hump between the blue lines."),
            trace("y", "Keep the top between the blue lines. Let the tail go below the lower blue line."),
            write("Copy ‘my hat’. Check the height of h and the tail of y. For cursive, ask an adult to model h and y from workbook pages 5 and 14.", 1);
            q!("Which letter has a tail below the lower blue line?", "h", "y", 1),
            q!("Which letter reaches the top red line?", "h", "a", 0),
            q!("Which part of h stays between the blue lines?", "The top of its tall stroke", "The hump", 1),
            q!("Which letter has a tail in ‘my hat’?", "y", "t", 0),
        ),
        day!("en2-write-2", "Day 2 · Words and spacing",
            trace("bell", "Say bell. Keep the two l letters equally tall."),
            write("Copy ‘red bell’. Leave a finger space between the words. For cursive, trace one word on the My Bicycle workbook word page (page 15), then copy it here.", 1),
            dictation("Listen, then write two words. Check the letter sizes and word space.", "Say ‘red bell’. Pause and say ‘red bell’ again.", 1);
            q!("Which has a space between the two words?", "red bell", "redbell", 0),
            q!("Which word has two l letters?", "red", "bell", 1),
            q!("Where should you leave a finger space in ‘blue bike’?", "Between blue and bike", "Between b and l", 0),
            q!("Which version keeps the letters of one word together?", "b i k e", "bike", 1),
        ),
        day!("en2-write-3", "Day 3 · Copy and check a sentence",
            write("Read, then copy ‘The kite has a tail.’ Keep the letters on the lower blue line.", 1),
            write("Look away and say the sentence. Write it from memory.", 1),
            Kind::Revise("Reread your work and fix one space or mark. For cursive, choose a sentence on the Picture Reading workbook sentence page (page 18). Trace once, then copy once.".into(), 1);
            q!("Which sentence is ready to read?", "the kite has a tail", "The kite has a tail.", 1),
            q!("Where is the full stop in ‘The kite has a tail.’?", "After tail", "After kite", 0),
            q!("Which is ready to start a sentence?", "the", "The", 1),
            q!("Which is easiest to read?", "A red kite.", "Aredkite.", 0),
        ),
        day!("en2-write-4", "Day 4 · Write a question",
            write("Copy ‘Where is the school?’ Read it back. Look at the mark at the end.", 1),
            dictation("Listen to the question. Say it back, then write it.", "Ask ‘Where is the park?’ Repeat it once and let the learner choose the end mark.", 1),
            write("Write your own question about a place. Start with ‘Where is …’ and check the end mark.", 1);
            q!("Which mark goes at the end of ‘Where is the school’?", "?", ".", 0),
            q!("Which sentence asks for a place?", "The school is near.", "Where is the school?", 1),
            q!("Which question starts with a capital?", "Where is the park?", "where is the park?", 0),
            q!("Which mark finishes ‘How do I get to the park’?", ".", "?", 1),
        ),
        day!("en2-write-5", "Day 5 · Write two sentences",
            write("Say two sentences about your way to school. Try ‘First I pass …’ and ‘Then I see …’. Write one sentence on each line.", 2),
            Kind::Revise("Reread your two sentences. Check capitals, spaces and full stops. Rewrite one sentence as clearly as you can.".into(), 1),
            write("Write one more sentence about a place you pass. You can use ‘I see …’ or your own words.", 1);
            q!("Which word can start the second step?", "First", "Then", 1),
            q!("Which sentence has clear spaces and a full stop?", "I see a bridge.", "Isee abridge", 0),
            q!("‘First I pass the shop. Then I cross the road.’ How many sentences are there?", "One", "Two", 1),
            q!("What should you check at the start of each sentence?", "A capital letter", "A full stop", 0),
        ),
    ]
}
