use super::{topic, Group, Kind, Language, Topic};
use crate::content::{ArithmeticHelp, Operation, Paper};

pub fn topics(language: Language, lessons: &[Topic]) -> Vec<Topic> {
    let mut group = Group::new(
        "math-writing",
        language.text("लिखकर हल करो", "Solve in writing"),
    );
    group.separate_page = true;
    let practice = |grade, id, title, problems, review_from| {
        let review = lessons
            .iter()
            .find(|lesson| lesson.id == review_from)
            .expect("writing practice must have a corresponding lesson");
        topic(
            grade,
            id,
            title,
            group.clone(),
            problems,
            review.recall.clone(),
        )
    };
    let sequences = language.text("गिनती में खाली जगह भरो", "Fill the gaps in counting");
    let ordering = language.text("संख्याएँ क्रम में लिखो", "Write numbers in order");
    let addition = language.text("जोड़ के सवाल", "Addition problems");
    let subtraction = language.text("घटाव के सवाल", "Subtraction problems");
    let notebook_add = language.text("कॉपी जैसा जोड़", "Notebook addition");
    let notebook_subtract = language.text("कॉपी जैसा घटाव", "Notebook subtraction");

    vec![
        practice(1, "notebook-add-1", notebook_add,
            notebook_problems([(3, 4), (2, 5), (5, 3)], Operation::Add, false, language), "add-to-twenty"),
        practice(1, "notebook-subtract-1", notebook_subtract,
            notebook_problems([(7, 4), (6, 2), (9, 3)], Operation::Subtract, false, language), "subtract-to-twenty"),
        practice(2, "notebook-add-2", notebook_add,
            notebook_problems([(5, 4), (23, 14), (31, 27)], Operation::Add, true, language), "add-within-99"),
        practice(2, "notebook-subtract-2", notebook_subtract,
            notebook_problems([(8, 3), (56, 23), (48, 16)], Operation::Subtract, true, language), "subtract-within-99"),
        practice(2, "notebook-carry-2", language.text("कॉपी में जोड़ · दहाई आगे ले जाओ", "Notebook addition with carrying"),
            notebook_problems([(27, 38), (46, 27), (58, 24)], Operation::Add, true, language), "add-exchange-ten"),
        practice(2, "notebook-borrow-2", language.text("कॉपी में घटाव · एक दहाई खोलो", "Notebook subtraction with borrowing"),
            notebook_problems([(52, 18), (60, 23), (61, 24)], Operation::Subtract, true, language), "subtract-exchange-ten"),
        practice(1, "write-sequences-1", sequences, vec![
            solve("8, 9, __, 11", "8, 9, 10, 11", 1, &[10]),
            solve("13, 12, __, 10", "13, 12, 11, 10", 1, &[11]),
            solve(language.text("17 से ठीक पहले की संख्या लिखो।", "Write the number just before 17."), "16", 1, &[16]),
            solve(language.text("19 के ठीक बाद की संख्या लिखो।", "Write the number just after 19."), "20", 1, &[20]),
            solve(language.text("8 और 10 के बीच की संख्या लिखो।", "Write the number between 8 and 10."), "9", 1, &[9]),
        ], "neighbors-to-twenty"),
        practice(1, "write-order-1", ordering, vec![
            order(language, "6, 2, 9", &[2, 6, 9], false),
            order(language, "11, 19, 14", &[19, 14, 11], true),
            order(language, "34, 19, 28", &[19, 28, 34], false),
        ], "number-order-1"),
        practice(1, "write-add-1", addition, vec![
            arithmetic(4, '+', 3, 7, 2),
            arithmetic(8, '+', 7, 15, 2),
            with_help(solve(language.text("रीना के पास 9 आम हैं। उसे 6 और मिले। अब उसके पास कितने आम हैं?", "Reena has 9 mangoes. She gets 6 more. How many mangoes does she have now?"), "9 + 6 = 15", 2, &[15]), 9, 6, Operation::Add),
        ], "add-to-twenty"),
        practice(1, "write-subtract-1", subtraction, vec![
            arithmetic(8, '−', 3, 5, 2),
            arithmetic(16, '−', 7, 9, 2),
            with_help(solve(language.text("डिब्बे में 12 कंचे थे। 4 निकाल लिए। कितने कंचे बचे?", "There were 12 marbles in a box. You took out 4. How many marbles are left?"), "12 − 4 = 8", 2, &[8]), 12, 4, Operation::Subtract),
        ], "subtract-to-twenty"),
        practice(1, "write-groups-1", language.text("बराबर समूह", "Equal groups"), vec![
            solve(language.text("3 प्लेटों में 2-2 बिस्कुट हैं। जोड़ लिखकर कुल बिस्कुट निकालो।", "There are 2 biscuits on each of 3 plates. Write an addition sentence to find the total."), "2 + 2 + 2 = 6", 2, &[6]),
            solve(language.text("2 थैलियों में 4-4 कंचे हैं। जोड़ लिखकर कुल कंचे निकालो।", "There are 4 marbles in each of 2 bags. Write an addition sentence to find the total."), "4 + 4 = 8", 2, &[8]),
            solve(language.text("4 डिब्बों में 3-3 गेंदें हैं। जोड़ लिखकर कुल गेंदें निकालो।", "There are 3 balls in each of 4 boxes. Write an addition sentence to find the total."), "3 + 3 + 3 + 3 = 12", 2, &[12]),
        ], "equal-groups-1"),
        practice(2, "write-sequences-2", sequences, vec![
            solve("58, 59, __, 61", "58, 59, 60, 61", 1, &[60]),
            solve("41, 40, __, 38", "41, 40, 39, 38", 1, &[39]),
            solve(language.text("100 से ठीक पहले की संख्या लिखो।", "Write the number just before 100."), "99", 1, &[99]),
            solve(language.text("69 के ठीक बाद की संख्या लिखो।", "Write the number just after 69."), "70", 1, &[70]),
            solve(language.text("5-5 करके गिनो। खाली जगह भरो: 35, 40, __, 50।", "Count in fives. Fill the gap: 35, 40, __, 50."), "35, 40, 45, 50", 1, &[45]),
        ], "count-to-100"),
        practice(2, "write-order-2", ordering, vec![
            order(language, "43, 38, 40, 49", &[38, 40, 43, 49], false),
            order(language, "91, 99, 100, 90", &[100, 99, 91, 90], true),
            order(language, "27, 70, 72, 7", &[7, 27, 70, 72], false),
        ], "number-order-2"),
        practice(2, "write-add-2", addition, vec![
            arithmetic(23, '+', 14, 37, 3),
            with_help(solve("27 + 38 = __", language.text(
                "7 + 8 = 15 इकाइयाँ। 10 इकाइयों को 1 दहाई बनाओ। 2 + 3 + 1 = 6 दहाइयाँ। 27 + 38 = 65।",
                "7 + 8 = 15 ones. Trade 10 ones for 1 ten. 2 + 3 + 1 = 6 tens. 27 + 38 = 65.",
            ), 3, &[65]), 27, 38, Operation::Add),
            with_help(solve(language.text("एक टोकरी में 36 सेब और दूसरी में 27 सेब हैं। कुल कितने सेब हैं?", "One basket has 36 apples and another has 27. How many apples are there altogether?"), "36 + 27 = 63", 3, &[63]), 36, 27, Operation::Add),
        ], "add-exchange-ten"),
        practice(2, "write-subtract-2", subtraction, vec![
            arithmetic(56, '−', 23, 33, 3),
            with_help(solve("52 − 18 = __", language.text(
                "1 दहाई खोलो। 52 में अब 4 दहाइयाँ और 12 इकाइयाँ हैं। 12 − 8 = 4 और 4 − 1 = 3। 52 − 18 = 34।",
                "Trade 1 ten for 10 ones. Now 52 is 4 tens and 12 ones. 12 − 8 = 4 and 4 − 1 = 3. 52 − 18 = 34.",
            ), 3, &[34]), 52, 18, Operation::Subtract),
            with_help(solve(language.text("किताब में 61 पन्ने हैं। सीमा ने 24 पढ़े। कितने पन्ने बाकी हैं?", "A book has 61 pages. Seema read 24. How many pages are left?"), "61 − 24 = 37", 3, &[37]), 61, 24, Operation::Subtract),
        ], "subtract-exchange-ten"),
        practice(2, "write-tables-1-to-5", language.text("पहाड़े 1 से 5", "Tables 1 to 5"), table_problems([(1, 9), (2, 6), (3, 5), (4, 8), (5, 10)]), "table-3"),
        practice(2, "write-tables-6-to-10", language.text("पहाड़े 6 से 10", "Tables 6 to 10"), table_problems([(6, 4), (7, 8), (8, 6), (9, 3), (10, 10)]), "table-8"),
        practice(2, "write-division-2", language.text("घटाकर भाग दो", "Divide by repeated subtraction"), vec![
            solve(language.text("12 ÷ 3 = __। हर बार 3 घटाओ और गिनो।", "12 ÷ 3 = __. Subtract 3 each time and count the steps."), "12 → 9 → 6 → 3 → 0. 12 ÷ 3 = 4", 3, &[4]),
            solve(language.text("20 ÷ 5 = __। हर बार 5 घटाओ और गिनो।", "20 ÷ 5 = __. Subtract 5 each time and count the steps."), "20 → 15 → 10 → 5 → 0. 20 ÷ 5 = 4", 3, &[4]),
            solve(language.text("18 पेंसिलों के 6-6 के समूह बनाओ। बार-बार घटाकर समूहों की संख्या लिखो।", "Put 18 pencils into groups of 6. Use repeated subtraction to find the number of groups."), "18 → 12 → 6 → 0. 18 ÷ 6 = 3", 3, &[3]),
        ], "division-take-away"),
    ]
}

pub(super) fn solve(prompt: &str, solution: &str, lines: usize, answer: &[u8]) -> Kind {
    Kind::Solve {
        prompt: prompt.into(),
        solution: solution.into(),
        lines,
        answer: answer.into(),
        helper: None,
        notebook: false,
    }
}

fn notebook_problems(
    operands: [(u8, u8); 3],
    operation: Operation,
    place_value: bool,
    language: Language,
) -> Vec<Kind> {
    operands
        .into_iter()
        .map(|(left, right)| {
            notebook_problem(
                ArithmeticHelp {
                    left,
                    right,
                    operation,
                    place_value,
                },
                language,
            )
        })
        .collect()
}

pub(super) fn notebook_problem(helper: ArithmeticHelp, language: Language) -> Kind {
    let ArithmeticHelp {
        left,
        right,
        operation,
        ..
    } = helper;
    let sign = match operation {
        Operation::Add => '+',
        Operation::Subtract => '−',
    };
    let answer = helper.answer();
    Kind::Solve {
        prompt: format!("{left} {sign} {right} = __"),
        solution: notebook_solution(helper, sign, answer, language),
        lines: 1,
        answer: vec![answer],
        helper: Some(helper),
        notebook: true,
    }
}

fn notebook_solution(
    problem: ArithmeticHelp,
    sign: char,
    answer: u8,
    language: Language,
) -> String {
    let ArithmeticHelp { left, right, .. } = problem;
    let (left_ones, right_ones) = (left % 10, right % 10);
    let equation = format!("{left} {sign} {right} = {answer}");
    match (problem.notebook_paper(), language) {
        (Paper::NotebookCarry, Language::Hindi) => format!(
            "{left_ones} + {right_ones} = {}। इकाई में {} लिखो। 1 दहाई ऊपर ले जाओ। {equation}।",
            left_ones + right_ones, answer % 10
        ),
        (Paper::NotebookCarry, Language::English) => format!(
            "{left_ones} + {right_ones} = {}. Write {} in the ones place and carry 1 ten. {equation}.",
            left_ones + right_ones, answer % 10
        ),
        (Paper::NotebookBorrow, Language::Hindi) => format!(
            "1 दहाई को 10 इकाइयों में बदलो। अब {} दहाइयाँ और {} इकाइयाँ हैं। {equation}।",
            left / 10 - 1, left_ones + 10
        ),
        (Paper::NotebookBorrow, Language::English) => format!(
            "Trade 1 ten for 10 ones. Now there are {} tens and {} ones. {equation}.",
            left / 10 - 1, left_ones + 10
        ),
        _ => equation,
    }
}

pub(super) fn arithmetic(left: u8, sign: char, right: u8, answer: u8, lines: usize) -> Kind {
    let problem = solve(
        &format!("{left} {sign} {right} = __"),
        &format!("{left} {sign} {right} = {answer}"),
        lines,
        &[answer],
    );
    match sign {
        '+' => with_help(problem, left, right, Operation::Add),
        '−' => with_help(problem, left, right, Operation::Subtract),
        _ => problem,
    }
}

fn with_help(mut problem: Kind, left: u8, right: u8, operation: Operation) -> Kind {
    if let Kind::Solve { helper, lines, .. } = &mut problem {
        *helper = Some(ArithmeticHelp {
            left,
            right,
            operation,
            place_value: *lines == 3,
        });
    }
    problem
}

fn table_problems(facts: [(u8, u8); 5]) -> Vec<Kind> {
    facts
        .into_iter()
        .map(|(left, right)| arithmetic(left, '×', right, left * right, 2))
        .collect()
}

fn order(language: Language, unsorted: &str, sorted: &[u8], descending: bool) -> Kind {
    let instruction = if descending {
        language.text("बड़े से छोटे क्रम में लिखो", "Write from largest to smallest")
    } else {
        language.text("छोटे से बड़े क्रम में लिखो", "Write from smallest to largest")
    };
    let solution = sorted
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    solve(&format!("{instruction}: {unsorted}"), &solution, 2, sorted)
}
