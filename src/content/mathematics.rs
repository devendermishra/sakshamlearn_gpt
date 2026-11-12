use super::{Exercise, Group, Kind, Paper, Question, Topic};
mod generated;
mod writing_practice;
use crate::language::Language;
pub use generated::{generate_practice, GeneratedPractice, PracticeSkill};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

fn starter_topics(language: Language) -> Vec<Topic> {
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
                    language.text(
                        "गिनो और सही अंक चुनो।",
                        "Count the objects and choose the correct number.",
                    ),
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
                        guidance: language
                            .text("एक खाने में एक अंक लिखो।", "Write one digit in each square.")
                            .into(),
                        paper: Paper::Digit,
                    },
                });
                let number_names = match language {
                    Language::Hindi => ["एक", "दो", "तीन", "चार", "पाँच", "छह", "सात", "आठ", "नौ"],
                    Language::English => [
                        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
                    ],
                };
                let name = number_names[usize::from(number - 1)];
                let prompt = match language {
                    Language::Hindi => format!("‘{name}’ के लिए सही अंक चुनो।"),
                    Language::English => format!("Choose the number for ‘{name}’."),
                };
                recall.push(Question::pair(
                    &prompt,
                    options[0],
                    options[1],
                    usize::from(number % 2 == 0),
                ));
            }
            Topic {
                id: format!("numbers-{start}-{}", start + 2),
                title: format!(
                    "{} · {start}–{}",
                    language.text("गिनो और लिखो", "Count and write"),
                    start + 2
                ),
                grade: 1,
                group: Group::new("numbers", language.text("संख्याएँ", "Numbers")),
                exercises,
                recall,
                break_every: None,
                revision: 0,
            }
        })
        .collect()
}

pub fn topics(language: Language) -> Vec<Topic> {
    let mut topics = starter_topics(language);
    topics.extend(class_one(language));
    topics.extend(class_two(language));
    topics.extend([1, 2, 5, 10, 3, 4, 6, 7, 8, 9].map(|size| table(language, size)));
    topics.push(topic(
        2,
        "tables-mixed",
        language.text("पहाड़े मिलाकर", "Mixed multiplication tables"),
        tables(language),
        vec![
            fact(2, 7),
            fact(5, 9),
            fact(3, 8),
            fact(6, 7),
            fact(9, 9),
            fact(10, 6),
        ]
        .into_iter()
        .map(Kind::Choice)
        .collect(),
        [fact(4, 8), fact(7, 6), fact(8, 9), fact(10, 10)],
    ));
    topics.extend(division_topics(language));
    writing_practice::topics(language, &topics)
        .into_iter()
        .chain(topics)
        .collect()
}

fn numbers(language: Language) -> Group {
    Group::new("numbers", language.text("संख्याएँ", "Numbers"))
}

fn operations(language: Language) -> Group {
    Group::new(
        "add-subtract",
        language.text("जोड़ और घटाव", "Addition and subtraction"),
    )
}

fn tables(language: Language) -> Group {
    let mut group = Group::new(
        "tables",
        language.text("पहाड़े 1 से 10", "Multiplication tables 1 to 10"),
    );
    group.separate_page = true;
    group
}

fn topic(
    grade: u8,
    id: &str,
    title: &str,
    group: Group,
    items: Vec<Kind>,
    recall: impl Into<Vec<Question>>,
) -> Topic {
    Topic {
        id: id.into(),
        title: title.into(),
        grade,
        group,
        exercises: items
            .into_iter()
            .enumerate()
            .map(|(index, kind)| Exercise {
                id: format!("item-{}", index + 1),
                kind,
            })
            .collect(),
        recall: recall.into(),
        break_every: Some(3),
        revision: 0,
    }
}

fn pick(prompt: &str, answer: u8, other: u8) -> Question {
    numeric_choice(
        prompt,
        answer,
        other,
        (u16::from(answer) + u16::from(other)) % 4 < 2,
    )
}

fn numeric_choice(prompt: &str, answer: u8, other: u8, answer_first: bool) -> Question {
    let correct = answer.to_string();
    let incorrect = other.to_string();
    if answer_first {
        Question::pair(prompt, &correct, &incorrect, 0)
    } else {
        Question::pair(prompt, &incorrect, &correct, 1)
    }
}

fn ask(prompt: &str, answer: u8, other: u8) -> Kind {
    Kind::Choice(pick(prompt, answer, other))
}

fn show(example: &str, prompt: &str, answer: u8, other: u8) -> Kind {
    Kind::Read(example.into(), pick(prompt, answer, other))
}

fn write(language: Language, model: u8) -> Kind {
    Kind::Trace {
        model: model.to_string(),
        guidance: language.text("हर खाने में एक अंक लिखो। पहले बिंदियों पर लिखो, फिर खाली खानों में खुद लिखो।", "Write one digit in each square. Trace the dotted digits first, then write in the blank squares.").into(),
        paper: Paper::Digit,
    }
}

fn before(language: Language, number: u8) -> Question {
    let prompt = match language {
        Language::Hindi => format!("{number} से ठीक पहले क्या आता है?"),
        Language::English => format!("What comes just before {number}?"),
    };
    pick(&prompt, number - 1, number)
}

fn after(language: Language, number: u8) -> Question {
    let prompt = match language {
        Language::Hindi => format!("{number} के ठीक बाद क्या आता है?"),
        Language::English => format!("What comes just after {number}?"),
    };
    pick(&prompt, number + 1, number)
}

fn between(language: Language, first: u8, last: u8) -> Question {
    let prompt = match language {
        Language::Hindi => format!("{first}, __, {last}। बीच में क्या आता है?"),
        Language::English => format!("{first}, __, {last}. What comes in between?"),
    };
    pick(&prompt, first.min(last) + 1, first)
}

fn order_question(language: Language, values: &[u8], descending: bool, answer: usize) -> Question {
    let list = |values: &[u8], separator| {
        values
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(separator)
    };
    let mut ordered = values.to_vec();
    ordered.sort_unstable();
    if descending {
        ordered.reverse();
    }
    let correct = list(&ordered, " → ");
    ordered.reverse();
    let reverse = list(&ordered, " → ");
    ordered.reverse();
    ordered.swap(0, 1);
    let almost = list(&ordered, " → ");
    let mut options = vec![correct, reverse, almost];
    options.rotate_right(answer);
    let direction = if descending {
        language.text("बड़े से छोटे", "largest to smallest")
    } else {
        language.text("छोटे से बड़े", "smallest to largest")
    };
    Question {
        prompt: match language {
            Language::Hindi => format!("{direction} का क्रम चुनो। संख्याएँ: {}", list(values, ", ")),
            Language::English => format!(
                "Choose the order from {direction}. Numbers: {}",
                list(values, ", ")
            ),
        },
        options,
        answer,
    }
}

fn number_order<const N: usize>(
    language: Language,
    grade: u8,
    examples: [&str; 2],
    practice: [[u8; N]; 6],
    review: [[u8; N]; 4],
) -> Topic {
    let items = practice
        .iter()
        .enumerate()
        .map(|(index, values)| {
            let question = order_question(language, values, index % 2 == 1, index % 3);
            match examples.get(index) {
                Some(example) => Kind::Read((*example).into(), question),
                None => Kind::Choice(question),
            }
        })
        .collect();
    let [first, second, third, fourth] = review;
    topic(
        grade,
        &format!("number-order-{grade}"),
        language.text("बढ़ता और घटता क्रम", "Ascending and descending order"),
        numbers(language),
        items,
        [
            order_question(language, &first, false, 2),
            order_question(language, &second, true, 0),
            order_question(language, &third, false, 1),
            order_question(language, &fourth, true, 2),
        ],
    )
}

fn class_one(language: Language) -> Vec<Topic> {
    vec![
        topic(
            1,
            "zero-and-ten",
            language.text("शून्य और दस बनाना", "Zero and making ten"),
            numbers(language),
            vec![
                show(language.text("कटोरी खाली है। उसमें 0 फल हैं।", "The bowl is empty. It has 0 pieces of fruit."), language.text("खाली डिब्बे में कितनी गेंदें हैं?", "How many balls are in the empty box?"), 0, 1),
                write(language, 0),
                show(
                    language.text("● ● ● ● ● | ● ● ● ● ●। पाँच और पाँच मिलकर 10 होते हैं।", "● ● ● ● ● | ● ● ● ● ●. Five and five make 10."),
                    language.text("6 में कितना जोड़ने से 10 बनेगा?", "What do you add to 6 to make 10?"),
                    4,
                    5,
                ),
                ask("10 = 8 + __", 2, 3),
                write(language, 10),
            ],
            [
                pick(language.text("7 में कितना जोड़ने से 10 बनेगा?", "What do you add to 7 to make 10?"), 3, 4),
                pick(language.text("टोकरी में कोई सेब नहीं है। कितने सेब हैं?", "There are no apples in the basket. How many apples are there?"), 0, 1),
                pick("9 + __ = 10", 1, 2),
                pick("10 = 4 + __", 6, 5),
            ],
        ),
        topic(
            1,
            "count-to-twenty",
            language.text("आगे और पीछे गिनो · 0–20", "Count forward and backward · 0–20"),
            numbers(language),
            vec![
                show(
                    language.text("8 — 9 — 10 — 11 — 12। आगे गिनने पर हर बार 1 बढ़ता है।", "8 — 9 — 10 — 11 — 12. Add 1 each time you count forward."),
                    "12, 13, __",
                    14,
                    15,
                ),
                show(
                    language.text("7 — 8 — 9 — 10 — 11। 11 से बाईं ओर चलो: 11, 10, 9, 8।", "7 — 8 — 9 — 10 — 11. Start at 11 and move left: 11, 10, 9, 8."),
                    "9, 8, __, 6",
                    7,
                    5,
                ),
                ask("3, 2, 1, __", 0, 1),
                ask("20, 19, __, 17", 18, 16),
                write(language, 19),
                Kind::Count {
                    quantity: 12,
                    visual: "●".into(),
                    question: pick(language.text("बिंदु गिनो। कितने हैं?", "Count the dots. How many are there?"), 12, 11),
                },
            ],
            [
                pick("13, 12, __, 10", 11, 9),
                pick("17, 18, 19, __", 20, 19),
                pick("2, 1, __", 0, 2),
                pick("11, 10, __, 8", 9, 10),
            ],
        ),
        topic(
            1,
            "neighbors-to-twenty",
            language.text("पहले, बाद में और बीच में · 0–20", "Before, after and between · 0–20"),
            numbers(language),
            vec![
                show(
                    language.text("7 — 8 — 9 — 10 — 11। 9 से पहले 8 है और 9 के बाद 10 है।", "7 — 8 — 9 — 10 — 11. 8 comes before 9, and 10 comes after 9."),
                    language.text("10 से ठीक पहले क्या आता है?", "What comes just before 10?"),
                    9,
                    11,
                ),
                Kind::Choice(after(language, 11)),
                Kind::Choice(between(language, 8, 10)),
                Kind::Choice(before(language, 1)),
                Kind::Choice(after(language, 19)),
                Kind::Choice(between(language, 15, 17)),
            ],
            [between(language, 10, 12), before(language, 7), after(language, 0), between(language, 18, 20)],
        ),
        topic(
            1,
            "tens-to-99",
            language.text("दहाई और इकाई · 21–99", "Tens and ones · 21–99"),
            numbers(language),
            vec![
                show(
                    language.text("हर बंडल में 10 तीलियाँ हैं। 2 बंडल और 3 खुली तीलियाँ मिलकर 23 हैं।", "Each bundle has 10 sticks. 2 bundles and 3 loose sticks make 23."),
                    language.text("3 दहाई और 5 इकाई से कौन-सी संख्या बनती है?", "What number has 3 tens and 5 ones?"),
                    35,
                    53,
                ),
                ask(language.text("कौन-सी संख्या बड़ी है: 48 या 58?", "Which number is greater: 48 or 58?"), 58, 48),
                ask("36, 37, __, 39", 38, 40),
                write(language, 48),
                ask(language.text("कौन-सी संख्या छोटी है: 89 या 99?", "Which number is smaller: 89 or 99?"), 89, 99),
            ],
            [
                pick(language.text("7 दहाई और 2 इकाई से कौन-सी संख्या बनती है?", "What number has 7 tens and 2 ones?"), 72, 27),
                pick(language.text("कौन-सी संख्या बड़ी है: 63 या 36?", "Which number is greater: 63 or 36?"), 63, 36),
                pick("96, 97, 98, __", 99, 95),
                pick(language.text("52 में कितनी दहाइयाँ हैं?", "How many tens are in 52?"), 5, 2),
            ],
        ),
        number_order(language,
            1,
            [
                language.text("बढ़ता क्रम यानी छोटे से बड़े। 5, 1 और 8 को ऐसे लगाओ: 1 → 5 → 8।", "Ascending order means smallest to largest. Put 5, 1 and 8 in order: 1 → 5 → 8."),
                language.text("घटता क्रम यानी बड़े से छोटे। 5, 1 और 8 को ऐसे लगाओ: 8 → 5 → 1।", "Descending order means largest to smallest. Put 5, 1 and 8 in order: 8 → 5 → 1."),
            ],
            [
                [7, 0, 3],
                [6, 10, 2],
                [12, 8, 20],
                [19, 11, 15],
                [41, 19, 32],
                [99, 79, 89],
            ],
            [[4, 9, 2], [9, 5, 13], [20, 18, 0], [39, 57, 45]],
        ),
        topic(
            1,
            "add-to-twenty",
            language.text("जोड़ो · 20 तक", "Add within 20"),
            operations(language),
            vec![
                show(
                    language.text("● ● ● और ● ●। दोनों समूह मिलाकर 5 हैं। 3 + 2 = 5।", "● ● ● and ● ●. Put the groups together to make 5. 3 + 2 = 5."),
                    language.text("2 गेंदें थीं। 4 और मिलीं। अब कितनी हैं?", "You had 2 balls and got 4 more. How many do you have now?"),
                    6,
                    5,
                ),
                show(
                    language.text("8 से 4 कदम आगे: 9, 10, 11, 12। इसलिए 8 + 4 = 12।", "Take 4 steps forward from 8: 9, 10, 11, 12. So 8 + 4 = 12."),
                    language.text("9 से 4 कदम आगे जाओ। कहाँ पहुँचे?", "Take 4 steps forward from 9. Where do you land?"),
                    13,
                    12,
                ),
                ask("11 + 5 = __", 16, 15),
                ask(language.text("टोकरी में 12 आम थे। 8 और रखे। अब कितने आम हैं?", "There were 12 mangoes in a basket. You put in 8 more. How many are there now?"), 20, 19),
                write(language, 16),
            ],
            [
                pick("7 + 6 = __", 13, 12),
                pick("9 + 8 = __", 17, 18),
                pick("12 + 3 = __", 15, 14),
                pick("14 + 6 = __", 20, 19),
            ],
        ),
        topic(
            1,
            "subtract-to-twenty",
            language.text("घटाओ · 20 तक", "Subtract within 20"),
            operations(language),
            vec![
                show(
                    language.text("● ● ● ● ● में से 2 हटाओ। ● ● ● बचे। 5 − 2 = 3।", "Take 2 away from ● ● ● ● ●. There are ● ● ● left. 5 − 2 = 3."),
                    language.text("7 केले थे। 2 खा लिए। कितने बचे?", "There were 7 bananas. You ate 2. How many are left?"),
                    5,
                    4,
                ),
                show(
                    language.text("14 से 3 कदम पीछे: 13, 12, 11। इसलिए 14 − 3 = 11।", "Take 3 steps back from 14: 13, 12, 11. So 14 − 3 = 11."),
                    language.text("13 से 3 कदम पीछे जाओ। कहाँ पहुँचे?", "Take 3 steps back from 13. Where do you land?"),
                    10,
                    11,
                ),
                ask("20 − 6 = __", 14, 16),
                ask(language.text("डिब्बे में 9 कंचे थे। सभी निकाल लिए। कितने बचे?", "A box held 9 marbles. You took them all out. How many are left?"), 0, 1),
                write(language, 14),
            ],
            [
                pick("15 − 6 = __", 9, 10),
                pick("11 − 2 = __", 9, 8),
                pick("18 − 0 = __", 18, 17),
                pick("16 − 4 = __", 12, 13),
            ],
        ),
        topic(
            1,
            "equal-groups-1",
            language.text("बराबर समूह बनाओ", "Make equal groups"),
            Group::new("groups", language.text("समूह और गुणा", "Groups and multiplication")),
            vec![
                show(
                    language.text("● ● | ● ● | ● ●। 3 बराबर समूह हैं। हर समूह में 2 बिंदु हैं। 2 + 2 + 2 = 6।", "● ● | ● ● | ● ●. There are 3 equal groups with 2 dots in each group. 2 + 2 + 2 = 6."),
                    language.text("2, 2 और 2 मिलाकर कितने बिंदु हैं?", "How many dots are there in 2, 2 and 2 together?"),
                    6,
                    5,
                ),
                show(
                    language.text("● ● ● | ● ● ●। 2 समूह, हर समूह में 3। 3 + 3 = 6। इसे 3 × 2 = 6 भी लिखते हैं।", "● ● ● | ● ● ●. There are 2 groups with 3 dots each. 3 + 3 = 6. We can also write 3 × 2 = 6."),
                    language.text("4 + 4 को 4 × कितने की तरह लिख सकते हैं?", "4 + 4 is the same as 4 × what?"),
                    2,
                    3,
                ),
                ask(language.text("4 प्लेटों में 2-2 बिस्कुट हैं। कुल कितने बिस्कुट हैं?", "There are 2 biscuits on each of 4 plates. How many biscuits are there altogether?"), 8, 6),
                ask(language.text("3 थैलियों में 3-3 कंचे हैं। कुल कितने कंचे हैं?", "There are 3 marbles in each of 3 bags. How many marbles are there altogether?"), 9, 6),
            ],
            [
                pick("2 + 2 + 2 + 2 = __", 8, 6),
                pick("3 + 3 + 3 = __", 9, 12),
                pick(language.text("2 डिब्बों में 5-5 गेंदें हैं। कुल कितनी गेंदें हैं?", "There are 5 balls in each of 2 boxes. How many balls are there altogether?"), 10, 7),
                pick(language.text("तीन बार 4 जोड़ो। कुल कितना है?", "Add 4 three times. What is the total?"), 12, 7),
            ],
        ),
    ]
}

fn class_two(language: Language) -> Vec<Topic> {
    vec![
        topic(2, "count-to-100", language.text("आगे और पीछे गिनो · 0–100", "Count forward and backward · 0–100"), numbers(language), vec![
            show(language.text("दहाई पार करते हुए आगे गिनो: 38, 39, 40, 41।", "Count forward across a ten: 38, 39, 40, 41."), "58, 59, __, 61", 60, 62),
            show(language.text("दहाई पार करते हुए पीछे गिनो: 72, 71, 70, 69।", "Count backward across a ten: 72, 71, 70, 69."), "51, 50, __, 48", 49, 48),
            ask("97, 98, 99, __", 100, 98),
            ask("100, 99, __, 97", 98, 96),
            ask("3, 2, 1, __", 0, 1),
            write(language, 100),
        ], [pick("61, 60, __, 58", 59, 61), pick("88, 89, __, 91", 90, 92),
            pick("21, 20, __, 18", 19, 18), pick("2, 1, __", 0, 1)]),
        topic(2, "neighbors-to-100", language.text("पहले, बाद में और बीच में · 0–100", "Before, after and between · 0–100"), numbers(language), vec![
            show(language.text("28 — 29 — 30 — 31। 30 से पहले 29 है और 30 के बाद 31 है।", "28 — 29 — 30 — 31. 29 comes before 30, and 31 comes after 30."), language.text("80 से ठीक पहले क्या आता है?", "What comes just before 80?"), 79, 81),
            Kind::Choice(after(language, 59)),
            Kind::Choice(between(language, 40, 38)),
            Kind::Choice(between(language, 98, 100)),
            Kind::Choice(before(language, 100)),
            Kind::Choice(after(language, 0)),
        ], [between(language, 69, 71), before(language, 1), after(language, 99), between(language, 90, 88)]),
        topic(2, "skip-count", language.text("2, 5 और 10 की छलाँग", "Skip-count by 2, 5 and 10"), numbers(language), vec![
            show(language.text("जूते जोड़ी में गिनो: 2, 4, 6, 8। हर बार 2 बढ़े।", "Count shoes in pairs: 2, 4, 6, 8. Add 2 each time."), "10, 12, 14, __", 16, 15),
            show(language.text("हर हाथ में 5 उँगलियाँ। पाँच-पाँच करके गिनो: 5, 10, 15, 20।", "There are 5 fingers on each hand. Count in fives: 5, 10, 15, 20."), "25, 30, 35, __", 40, 36),
            show(language.text("हर बंडल में 10 तीलियाँ। दस-दस करके गिनो: 10, 20, 30।", "There are 10 sticks in each bundle. Count in tens: 10, 20, 30."), "70, 80, 90, __", 100, 91),
            ask(language.text("पीछे गिनो: 50, 45, 40, __", "Count backward: 50, 45, 40, __"), 35, 30),
            ask(language.text("पीछे गिनो: 60, 50, __, 30", "Count backward: 60, 50, __, 30"), 40, 45),
        ], [pick("16, 18, __, 22", 20, 19), pick("40, 45, __, 55", 50, 46),
            pick("10, 20, 30, __", 40, 31), pick("18, 16, 14, __", 12, 13)]),
        topic(2, "place-value-100", language.text("दहाई, इकाई और सौ", "Tens, ones and one hundred"), numbers(language), vec![
            show(language.text("57 = 50 + 7। 5 दहाई और 7 इकाई।", "57 = 50 + 7. That's 5 tens and 7 ones."), language.text("6 दहाई और 4 इकाई से कौन-सी संख्या बनेगी?", "What number has 6 tens and 4 ones?"), 64, 46),
            ask(language.text("83 में कितनी इकाइयाँ हैं?", "What is the ones digit in 83?"), 3, 8),
            show(language.text("10 इकाई = 1 दहाई। 10 दहाई = 1 सौ।", "10 ones make 1 ten. 10 tens make 1 hundred."), language.text("10 दहाइयाँ मिलकर कितनी इकाइयाँ बनाती हैं?", "How many ones are in 10 tens?"), 100, 10),
            ask(language.text("72 से 10 अधिक कितना है?", "What is 10 more than 72?"), 82, 73),
            write(language, 64),
        ], [pick("90 + 6 = __", 96, 69), pick(language.text("46 में कितनी दहाइयाँ हैं?", "How many tens are in 46?"), 4, 6),
            pick(language.text("100 में कितने सौ हैं?", "How many hundreds are in 100?"), 1, 10), pick(language.text("1 सौ में कितनी इकाइयाँ हैं?", "How many ones are in 1 hundred?"), 100, 10)]),
        number_order(language,
            2,
            [
                language.text("बढ़ता क्रम यानी छोटे से बड़े। दो अंकों की संख्याओं में पहले दहाई देखो। दहाई बराबर हो तो इकाई देखो। 38 → 42 → 47 → 53।", "Ascending order means smallest to largest. For two-digit numbers, compare the tens first. If the tens are the same, compare the ones. 38 → 42 → 47 → 53."),
                language.text("घटता क्रम यानी बड़े से छोटे। सबसे बड़ी संख्या पहले रखो। 53 → 47 → 42 → 38।", "Descending order means largest to smallest. Put the largest number first. 53 → 47 → 42 → 38."),
            ],
            [[50, 39, 49, 41], [62, 19, 60, 27], [54, 45, 5, 50], [91, 99, 90, 100], [20, 12, 2, 0], [77, 70, 87, 80]],
            [[69, 71, 59, 70], [98, 0, 100, 89], [38, 28, 83, 30], [11, 91, 19, 9]],
        ),
        topic(2, "add-within-99", language.text("दहाई और इकाई जोड़ो", "Add tens and ones"), operations(language), vec![
            show(language.text("23 + 14। दहाइयाँ: 20 + 10 = 30। इकाइयाँ: 3 + 4 = 7। कुल 37।", "23 + 14. Tens: 20 + 10 = 30. Ones: 3 + 4 = 7. The total is 37."), "32 + 15 = __", 47, 57),
            ask("46 + 22 = __", 68, 66),
            ask(language.text("एक डिब्बे में 41 चॉक और दूसरे में 28 हैं। कुल कितनी हैं?", "One box has 41 sticks of chalk and another has 28. How many are there altogether?"), 69, 59),
            write(language, 68),
        ], [pick("25 + 13 = __", 38, 37), pick("40 + 29 = __", 69, 59),
            pick("11 + 88 = __", 99, 98), pick("72 + 6 = __", 78, 76)]),
        topic(2, "subtract-within-99", language.text("दहाई और इकाई घटाओ", "Subtract tens and ones"), operations(language), vec![
            show(language.text("57 − 24। दहाइयाँ: 50 − 20 = 30। इकाइयाँ: 7 − 4 = 3। बाकी 33।", "57 − 24. Tens: 50 − 20 = 30. Ones: 7 − 4 = 3. That leaves 33."), "48 − 23 = __", 25, 35),
            ask("67 − 40 = __", 27, 37),
            ask(language.text("75 पन्नों में से 24 पढ़ लिए। कितने बाकी हैं?", "You have read 24 of 75 pages. How many pages are left?"), 51, 41),
            write(language, 51),
        ], [pick("86 − 45 = __", 41, 51), pick("96 − 52 = __", 44, 54),
            pick("78 − 8 = __", 70, 68), pick("45 − 45 = __", 0, 5)]),
        topic(2, "add-exchange-ten", language.text("जोड़ में नई दहाई बनाओ", "Make a new ten when adding"), operations(language), vec![
            show(language.text("28 + 17। 8 + 7 = 15 इकाई। 10 इकाई को 1 दहाई बनाओ। अब 4 दहाई और 5 इकाई हैं। कुल 45।", "28 + 17. 8 + 7 = 15 ones. Trade 10 ones for 1 ten. Now you have 4 tens and 5 ones. The total is 45."), "27 + 16 = __", 43, 33),
            ask("38 + 22 = __", 60, 50),
            ask(language.text("मीरा के पास 46 मनके और रवि के पास 27 हैं। दोनों के पास कुल कितने हैं?", "Meera has 46 beads and Ravi has 27. How many do they have altogether?"), 73, 63),
            write(language, 73),
        ], [pick("19 + 24 = __", 43, 33), pick("36 + 27 = __", 63, 53),
            pick("48 + 32 = __", 80, 70), pick("56 + 38 = __", 94, 84)]),
        topic(2, "subtract-exchange-ten", language.text("घटाने में एक दहाई खोलो", "Trade a ten when subtracting"), operations(language), vec![
            show(language.text("42 − 17। 1 दहाई खोलो: 42 = 3 दहाई और 12 इकाई। 12 − 7 = 5 और 3 − 1 = 2। बाकी 25।", "42 − 17. Trade 1 ten for 10 ones: 42 is 3 tens and 12 ones. 12 − 7 = 5 and 3 − 1 = 2. That leaves 25."), "52 − 18 = __", 34, 44),
            ask("60 − 23 = __", 37, 47),
            ask(language.text("54 कागज़ थे। 28 इस्तेमाल किए। कितने बचे?", "You had 54 sheets of paper and used 28. How many are left?"), 26, 36),
            write(language, 26),
        ], [pick("51 − 26 = __", 25, 35), pick("80 − 35 = __", 45, 55),
            pick("73 − 19 = __", 54, 64), pick("91 − 8 = __", 83, 93)]),
    ]
}

fn fact(size: u8, groups: u8) -> Question {
    let mut rng = StdRng::seed_from_u64(u64::from(size));
    let mut positions = [
        false, true, false, true, false, true, false, true, false, true,
    ];
    let mut higher = positions;
    positions.shuffle(&mut rng);
    higher.shuffle(&mut rng);
    let index = usize::from(groups - 1);
    let answer = size * groups;
    let other = if higher[index] && answer < 100 {
        (answer + size).min(100)
    } else {
        answer - size
    };
    numeric_choice(
        &format!("{size} × {groups} = __"),
        answer,
        other,
        positions[index],
    )
}

fn table(language: Language, size: u8) -> Topic {
    let mut items: Vec<_> = (1..=10)
        .map(|groups| Kind::Choice(fact(size, groups)))
        .collect();
    let example = match language {
        Language::Hindi => format!(
            "हर समूह में {size} हैं। 1 समूह में {size}, 2 समूहों में {}। हर नए समूह पर {size} जोड़ो।",
            size * 2
        ),
        Language::English => format!(
            "Each group has {size}. One group has {size}, and two groups have {}. Add {size} for each new group.",
            size * 2
        ),
    };
    items[0] = Kind::Read(example, fact(size, 1));
    let title = match language {
        Language::Hindi => format!("{size} का पहाड़ा"),
        Language::English => format!("{size} times table"),
    };
    topic(
        2,
        &format!("table-{size}"),
        &title,
        tables(language),
        items,
        [fact(size, 3), fact(size, 5), fact(size, 7), fact(size, 9)],
    )
}

fn division(divisor: u8, quotient: u8) -> Question {
    let mut rng = StdRng::seed_from_u64(u64::from(divisor) * 100 + u64::from(quotient));
    let other = if rng.gen() {
        quotient + 1
    } else {
        quotient - 1
    };
    numeric_choice(
        &format!("{} ÷ {divisor} = __", divisor * quotient),
        quotient,
        other,
        rng.gen(),
    )
}

fn division_example(language: Language, divisor: u8, quotient: u8) -> String {
    let total = divisor * quotient;
    let sequence = (0..=quotient)
        .rev()
        .map(|groups| (groups * divisor).to_string())
        .collect::<Vec<_>>()
        .join(" → ");
    match language {
        Language::Hindi => format!("{total} पेंसिलों में से हर बार {divisor} हटाओ। {sequence}। {quotient} बार घटाया, इसलिए {total} ÷ {divisor} = {quotient}।"),
        Language::English => format!("Start with {total} pencils and take away {divisor} each time. {sequence}. You subtracted {quotient} times, so {total} ÷ {divisor} = {quotient}."),
    }
}

fn division_topics(language: Language) -> Vec<Topic> {
    let group = Group::new(
        "division",
        language.text("बार-बार घटाकर भाग", "Division by repeated subtraction"),
    );
    vec![
        topic(
            2,
            "division-take-away",
            language.text("घटाओ और समूह गिनो", "Subtract and count the groups"),
            group.clone(),
            vec![
                show(
                    &division_example(language, 3, 4),
                    language.text(
                        "12 से पहली बार 3 हटाए। कितनी पेंसिलें बचीं?",
                        "Take 3 pencils away from 12. How many are left?",
                    ),
                    9,
                    8,
                ),
                ask(
                    language.text(
                        "12 → 9 → __ → 3 → 0। हर बार 3 घटाओ।",
                        "12 → 9 → __ → 3 → 0. Subtract 3 each time.",
                    ),
                    6,
                    5,
                ),
                ask(
                    language.text(
                        "12 → 9 → 6 → 3 → 0। कितनी बार 3 घटाया?",
                        "12 → 9 → 6 → 3 → 0. How many times did you subtract 3?",
                    ),
                    4,
                    5,
                ),
                Kind::Choice(division(2, 4)),
                ask(
                    language.text(
                        "15 बिस्कुटों के 5-5 के समूह बनाओ। कितने समूह बनेंगे?",
                        "Put 15 biscuits into groups of 5. How many groups can you make?",
                    ),
                    3,
                    5,
                ),
            ],
            [
                division(3, 3),
                division(2, 5),
                division(5, 4),
                division(4, 2),
            ],
        ),
        topic(
            2,
            "division-practice",
            language.text("भाग का अभ्यास", "Practise division"),
            group,
            vec![
                show(
                    &division_example(language, 4, 4),
                    language.text(
                        "16 → 12 → 8 → 4 → 0। हर बार कितना घटाया?",
                        "16 → 12 → 8 → 4 → 0. How much did you subtract each time?",
                    ),
                    4,
                    8,
                ),
                ask(
                    language.text(
                        "18 → 12 → __ → 0। हर बार 6 घटाओ।",
                        "18 → 12 → __ → 0. Subtract 6 each time.",
                    ),
                    6,
                    8,
                ),
                ask(
                    language.text(
                        "24 → 16 → 8 → 0। कितने समूह बने?",
                        "24 → 16 → 8 → 0. How many groups did you make?",
                    ),
                    3,
                    4,
                ),
                ask(
                    language.text(
                        "30 मनकों को 5-5 के समूह में रखो। कितने समूह बनेंगे?",
                        "Put 30 beads into groups of 5. How many groups can you make?",
                    ),
                    6,
                    5,
                ),
                Kind::Choice(division(10, 4)),
            ],
            [
                division(4, 5),
                division(7, 3),
                division(6, 4),
                division(8, 4),
            ],
        ),
    ]
}
