use crate::{
    content::{ArithmeticHelp, Operation, Paper},
    language::Language,
    keypad::{Keys, NumericField},
    state::{HelperDraft, Ink},
    writing::{InkPad, PaperScale},
};
use dioxus::prelude::*;

pub fn parse_numbers(text: &str) -> Option<Vec<u8>> {
    let normalize = |ch| match ch {
        '०'..='९' => char::from_u32('0' as u32 + ch as u32 - '०' as u32).unwrap(),
        _ => ch,
    };
    let numbers = text
        .split(|ch: char| ch.is_whitespace() || ch == ',')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let digits: String = part.chars().map(normalize).collect();
            if digits.bytes().all(|byte| byte.is_ascii_digit()) {
                digits.parse().ok()
            } else {
                None
            }
        })
        .collect::<Option<Vec<_>>>()?;
    (!numbers.is_empty()).then_some(numbers)
}

#[derive(Clone, Copy, PartialEq)]
enum Check {
    Waiting,
    Invalid,
    Wrong,
    Correct,
}

#[component]
pub fn NumberCheck(
    label: String,
    expected: Vec<u8>,
    language: Language,
    #[props(default)] confirmed: bool,
    #[props(default)] on_correct: Option<EventHandler<()>>,
    #[props(default)] on_wrong: Option<EventHandler<()>>,
) -> Element {
    let mut input = use_signal(|| {
        if confirmed {
            expected
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        }
    });
    let mut result = use_signal(|| {
        if confirmed {
            Check::Correct
        } else {
            Check::Waiting
        }
    });
    let count = expected.len();
    let feedback = match result() {
        Check::Waiting => "",
        Check::Invalid if count == 1 => language.text("एक संख्या लिखो।", "Enter one number."),
        Check::Invalid => language.text(
            "सभी संख्याएँ क्रम से लिखो। बीच में अल्पविराम या खाली जगह रखो।",
            "Enter all the numbers in order, separated by commas or spaces.",
        ),
        Check::Wrong => language.text(
            "फिर गिनो और दोबारा जाँचो।",
            "Check your working and try again.",
        ),
        Check::Correct => language.text("सही!", "Correct!"),
    };
    rsx! {
        form { class: "number-check", onsubmit: move |event| {
            event.prevent_default();
            if result() == Check::Correct { return; }
            match parse_numbers(&input()) {
                Some(numbers) if numbers == expected => {
                    result.set(Check::Correct);
                    if let Some(done) = on_correct { done.call(()); }
                }
                Some(numbers) if numbers.len() == count => {
                    result.set(Check::Wrong);
                    if let Some(wrong) = on_wrong { wrong.call(()); }
                }
                _ => result.set(Check::Invalid),
            }
        },
            NumericField { label, value: input(), language, limit: (count * 5).max(32),
                keys: if count == 1 { Keys::Whole } else { Keys::List },
                disabled: result() == Check::Correct,
                on_change: move |value| { input.set(value); result.set(Check::Waiting); }
            }
            button { r#type: "submit", disabled: result() == Check::Correct,
                {language.text("जाँचो", "Check")} }
            if matches!(result(), Check::Wrong | Check::Correct) {
                crate::activity::AnswerFeedback { correct: result() == Check::Correct, text: feedback }
            } else {
                p { role: "status", "{feedback}" }
            }
        }
    }
}

fn count_tallies(ink: &Ink) -> (usize, bool) {
    let mut count = 0;
    let mut invalid = false;
    for cell in ink.iter().filter(|cell| !cell.is_empty()) {
        let [stroke] = cell.as_slice() else {
            invalid = true;
            continue;
        };
        let (Some(first), Some(last)) = (stroke.first(), stroke.last()) else {
            invalid = true;
            continue;
        };
        let height = (last[1] - first[1]).abs();
        let width = stroke
            .iter()
            .map(|point| (point[0] - first[0]).abs())
            .fold(0.0, f32::max);
        let travel: f32 = stroke
            .windows(2)
            .map(|points| (points[1][1] - points[0][1]).abs())
            .sum();
        if height >= 0.25 && height >= 1.5 * width && travel <= 1.6 * height {
            count += 1;
        } else {
            invalid = true;
        }
    }
    (count, invalid)
}

#[component]
fn TallyRow(
    id: &'static str,
    label: &'static str,
    expected: u8,
    scale: PaperScale,
    language: Language,
    initial: Option<Ink>,
    on_change: EventHandler<Ink>,
) -> Element {
    let expected = usize::from(expected);
    let (count, invalid) = initial.as_ref().map_or((0, false), count_tallies);
    let feedback = if initial.is_none() {
        String::new()
    } else if invalid {
        language
            .text(
                "हर खाने में एक सीधी खड़ी रेखा बनाओ।",
                "Draw one straight upright mark in each square.",
            )
            .into()
    } else {
        match count.cmp(&expected) {
            std::cmp::Ordering::Less => match language {
                Language::Hindi => format!("{count} बनीं। {} रेखाएँ और बनाओ।", expected - count),
                Language::English => format!("{count} drawn. Add {} more.", expected - count),
            },
            std::cmp::Ordering::Greater => match language {
                Language::Hindi => format!("{count} बनीं। {} रेखाएँ हटाओ।", count - expected),
                Language::English => format!("{count} drawn. Remove {}.", count - expected),
            },
            std::cmp::Ordering::Equal => {
                language.text("सही गिनती!", "Right number of marks!").into()
            }
        }
    };
    let reference = if expected == 0 {
        "0".into()
    } else {
        "│ ".repeat(expected)
    };
    rsx! {
        div { class: "tally-group",
            p { class: "tally-label", "{label} {expected}" }
            p { class: "tally-reference", aria_hidden: "true", "{reference}" }
            {rsx! {
                InkPad { key: "{language.code()}", id, model: "", initial: initial.unwrap_or_default(), scale,
                    work_columns: expected + 2,
                    language, paper: Paper::Digit, on_ink: on_change
                }
            }}
            p { role: "status", "{feedback}" }
        }
    }
}

enum HelperEdit {
    First(Ink),
    Second(Ink),
    BorrowedTens,
    BorrowedOnes,
    Carried,
    OnesChecked,
    TensChecked,
}

#[derive(Debug, PartialEq, Eq)]
struct Places {
    left_tens: u8,
    left_ones: u8,
    right_tens: u8,
    right_ones: u8,
    borrow: bool,
    carry: u8,
    tens: u8,
    ones: u8,
}

fn places(help: ArithmeticHelp) -> Places {
    let (left_tens, left_ones) = (help.left / 10, help.left % 10);
    let (right_tens, right_ones) = (help.right / 10, help.right % 10);
    let borrow = help.operation == Operation::Subtract && left_ones < right_ones;
    let left_tens = left_tens - u8::from(borrow);
    let left_ones = left_ones + 10 * u8::from(borrow);
    let carry = if help.operation == Operation::Add {
        (left_ones + right_ones) / 10
    } else {
        0
    };
    let (tens, ones) = match help.operation {
        Operation::Add => (
            left_tens + right_tens + carry,
            (left_ones + right_ones) % 10,
        ),
        Operation::Subtract => (left_tens - right_tens, left_ones - right_ones),
    };
    Places {
        left_tens,
        left_ones,
        right_tens,
        right_ones,
        borrow,
        carry,
        tens,
        ones,
    }
}

#[component]
pub fn ArithmeticHelper(
    help: ArithmeticHelp,
    language: Language,
    scale: PaperScale,
    initial: HelperDraft,
    on_draft: EventHandler<HelperDraft>,
) -> Element {
    let place = help.place_value.then(|| places(help));
    let (left, right) = place
        .as_ref()
        .map_or((help.left, help.right), |p| (p.left_ones, p.right_ones));
    let mut draft = use_signal(|| initial);
    let update = use_callback(move |edit: HelperEdit| {
        let mut next = draft.peek().clone();
        match edit {
            HelperEdit::First(ink) => next.first = Some(ink),
            HelperEdit::Second(ink) => next.second = Some(ink),
            HelperEdit::BorrowedTens => next.borrowed_tens = true,
            HelperEdit::BorrowedOnes => next.borrowed_ones = true,
            HelperEdit::Carried => next.carried = true,
            HelperEdit::OnesChecked => next.ones_checked = true,
            HelperEdit::TensChecked => next.tens_checked = true,
        }
        draft.set(next.clone());
        on_draft.call(next);
    });
    let draft = draft.read();
    let tally_matches = |ink: &Option<Ink>, target: u8| {
        ink.as_ref().map_or(target == 0, |ink| {
            count_tallies(ink) == (usize::from(target), false)
        })
    };
    let tallies_ready = tally_matches(&draft.first, left) && tally_matches(&draft.second, right);
    let borrowing = place.as_ref().is_some_and(|p| p.borrow);
    let ready = !borrowing || draft.borrowed_ones;
    let addition = help.operation == Operation::Add;
    let first_label = if help.place_value {
        language.text("पहली संख्या की इकाइयाँ", "Ones in the first number")
    } else {
        language.text("पहला समूह", "First group")
    };
    let second_label = match (addition, help.place_value) {
        (true, true) => language.text("दूसरी संख्या की इकाइयाँ", "Ones in the second number"),
        (true, false) => language.text("दूसरा समूह", "Second group"),
        (false, true) => language.text("इतनी इकाइयाँ हटाओ", "Ones to take away"),
        (false, false) => language.text("इतनी रेखाएँ हटाओ", "Marks to take away"),
    };
    let tens_word = language.text("दहाई", "tens");
    let ones_word = language.text("इकाई", "ones");
    let sign = if addition { "+" } else { "−" };
    rsx! {
        details { class: "math-helper", open: *draft != HelperDraft::default(),
            summary { onclick:move |_|crate::activity::note_helper(), {language.text("गिनने में मदद", "Counting helper")} }
            if let Some(p) = &place {
                p { "{help.left}: {help.left / 10} {tens_word}, {help.left % 10} {ones_word} · {help.right}: {p.right_tens} {tens_word}, {p.right_ones} {ones_word}" }
                if p.borrow {
                    p { {language.text("ऊपर की इकाइयाँ कम हैं। 1 दहाई को 10 इकाइयों में बदलो।", "There are too few ones on top. Trade 1 ten for 10 ones.")} }
                    NumberCheck { label: language.text("बदलने के बाद कितनी दहाइयाँ बचीं?", "How many tens are left after borrowing?"), expected: vec![p.left_tens], language,
                        confirmed: draft.borrowed_tens, on_correct: move |_| update.call(HelperEdit::BorrowedTens) }
                    if draft.borrowed_tens {
                        NumberCheck { label: language.text("अब ऊपर कितनी इकाइयाँ हैं?", "How many ones are on top now?"), expected: vec![p.left_ones], language,
                            confirmed: draft.borrowed_ones, on_correct: move |_| update.call(HelperEdit::BorrowedOnes) }
                    }
                }
            }
            if ready {
                p { {language.text("नमूने जितनी खड़ी रेखाएँ बनाओ। हर खाने में एक रेखा।", "Draw as many upright marks as shown. Put one mark in each square.")} }
                div { class: "tally-line",
                    TallyRow { id: "tally-first", label: first_label, expected: left, scale, language, initial: draft.first.clone(),
                        on_change: move |ink| update.call(HelperEdit::First(ink)) }
                    TallyRow { id: "tally-second", label: second_label, expected: right, scale, language, initial: draft.second.clone(),
                        on_change: move |ink| update.call(HelperEdit::Second(ink)) }
                }
            }
            if ready && tallies_ready {
                if addition {
                    p { {language.text("दोनों समूहों की रेखाएँ साथ गिनो।", "Count the marks in both groups together.")} }
                } else {
                    p { {language.text("कटी हुई रेखाएँ हटा दी गई हैं। बची रेखाएँ गिनो।", "The crossed-out marks were taken away. Count the marks left.")} }
                    p { class: "tally-reference",
                        span { class: "crossed-tallies", {"│ ".repeat(usize::from(right))} }
                        span { {"│ ".repeat(usize::from(left - right))} }
                    }
                }
                if let Some(p) = &place {
                    if p.carry > 0 {
                        p { {language.text("10 इकाइयों का एक समूह बनाकर उसे दहाई में ले जाओ।", "Group 10 ones into a ten and carry it to the tens column.")} }
                        NumberCheck { label: language.text("कितनी दहाइयाँ आगे जाएँगी?", "How many tens do you carry?"), expected: vec![p.carry], language,
                            confirmed: draft.carried, on_correct: move |_| update.call(HelperEdit::Carried) }
                    }
                    if p.carry == 0 || draft.carried {
                        NumberCheck { label: language.text("उत्तर की इकाई का अंक क्या है?", "What is the ones digit of the answer?"), expected: vec![p.ones], language,
                            confirmed: draft.ones_checked, on_correct: move |_| update.call(HelperEdit::OnesChecked) }
                    }
                    if draft.ones_checked {
                        if p.carry > 0 {
                            p { "{p.left_tens} + {p.right_tens} + {p.carry} {tens_word}" }
                        } else {
                            p { "{p.left_tens} {sign} {p.right_tens} {tens_word}" }
                        }
                        NumberCheck { label: language.text("उत्तर में कितनी दहाइयाँ हैं?", "How many tens are in the answer?"), expected: vec![p.tens], language,
                            confirmed: draft.tens_checked, on_correct: move |_| update.call(HelperEdit::TensChecked) }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_answers_and_ordered_lists_in_both_scripts() {
        assert_eq!(parse_numbers(" ३७ "), Some(vec![37]));
        assert_eq!(parse_numbers("०, 39 १००"), Some(vec![0, 39, 100]));
        for input in ["", "1 + 2", "37.0", "256", "-2", "one"] {
            assert_eq!(parse_numbers(input), None, "{input}");
        }
    }

    #[test]
    fn counts_upright_marks_and_rejects_dots_and_multiple_marks_in_a_cell() {
        let line = vec![[0.4, 0.15], [0.43, 0.8]];
        assert_eq!(
            count_tallies(&vec![vec![line.clone()], vec![line.clone()]]),
            (2, false)
        );
        assert_eq!(
            count_tallies(&vec![vec![line.clone()], vec![vec![[0.5, 0.5]]]]),
            (1, true)
        );
        assert_eq!(count_tallies(&vec![vec![line.clone(), line]]), (0, true));
    }

    #[test]
    fn place_value_helper_carries_and_borrows_at_the_tens_boundary() {
        let add = places(ArithmeticHelp {
            left: 27,
            right: 38,
            operation: Operation::Add,
            place_value: true,
        });
        assert_eq!((add.carry, add.tens, add.ones), (1, 6, 5));
        let subtract = places(ArithmeticHelp {
            left: 60,
            right: 23,
            operation: Operation::Subtract,
            place_value: true,
        });
        assert!(subtract.borrow);
        assert_eq!(
            (
                subtract.left_tens,
                subtract.left_ones,
                subtract.tens,
                subtract.ones
            ),
            (5, 10, 3, 7)
        );
        let ordinary = places(ArithmeticHelp {
            left: 56,
            right: 23,
            operation: Operation::Subtract,
            place_value: true,
        });
        assert!(!ordinary.borrow);
        assert_eq!((ordinary.tens, ordinary.ones), (3, 3));
    }
}
