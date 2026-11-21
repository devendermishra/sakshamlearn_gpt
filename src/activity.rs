use crate::{
    content::{ArithmeticHelp, Kind, Paper, Question},
    math_help::{ArithmeticHelper, NumberCheck},
    progress::Input,
    state::{HelperDraft, Ink},
    writing::InkPad,
};


// Optional, pane-owned answer memory. Standalone lessons do not provide this context.
#[derive(Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ActivityDraft {
    pub selected: Option<usize>,
    pub missed: bool,
    pub choices_shown: bool,
    pub passage_ready: bool,
    pub answer_mode: bool,
    pub starter: Option<String>,
    pub write_mode: bool,
}

#[derive(Clone, Copy)]
pub struct ActivityMemory(pub Signal<ActivityDraft>);

#[component]
pub fn AnswerFeedback(correct: bool, text: String) -> Element {
    rsx! {
        div { class: if correct { "answer-feedback correct" } else { "answer-feedback incorrect" }, role: "status",
            span { class: "answer-symbol", aria_hidden: "true", if correct { "✓" } else { "✕" } }
            span { "{text}" }
        }
    }
}

#[component]
fn CorrectAnswer(language: Language, on_advance: EventHandler<()>) -> Element {
    rsx! {
        AnswerFeedback { correct: true, text: language.text("सही!", "Correct!") }
        button { class: "primary", onclick: move |_| on_advance.call(()),
            {language.text("आगे", "Next")}
        }
    }
}

#[component]
fn SolveActivity(
    prompt: String,
    solution: String,
    answer: Vec<u8>,
    helper: Option<ArithmeticHelp>,
    notebook: bool,
    #[props(default)] table_size: Option<u8>,
    lines: usize,
    initial: Ink,
    helper_draft: HelperDraft,
    scale: PaperScale,
    language: Language,
    on_ink: EventHandler<Ink>,
    on_input: EventHandler<Input>,
) -> Element {
    let mut checked = use_signal(|| None::<Ink>);
    let mut correct = use_signal(|| None::<bool>);
    let mut tried = use_signal(|| false);
    let equation = if notebook { helper } else { None };
    let paper = equation.map_or(Paper::Digit, ArithmeticHelp::notebook_paper);
    let legacy = answer.is_empty();
    let label = if legacy {
        language.text("हल देखो", "Check solution")
    } else {
        language.text("जवाब दर्ज करो", "Enter answer")
    };
    let answer_label = if answer.len() > 1 {
        language.text(
            "अपनी लिखी संख्याएँ क्रम से दर्ज करो।",
            "Enter the numbers you wrote, in order.",
        )
    } else {
        language.text("अपना लिखा जवाब दर्ज करो।", "Enter the answer you wrote.")
    };
    rsx! {
        h2 { class: if notebook { "visually-hidden" } else { "" }, "{prompt}" }
        if let Some(help) = helper {
            ArithmeticHelper { help, language, scale, initial: helper_draft,
                on_draft: move |draft| on_input.call(Input::Helper(draft)) }
        }
        if let Some(work) = checked() {
            InkPad { key: "{language.code()}", id: "checked-math-ink", model: "", initial: work.clone(),
                scale, lines, paper, notebook: equation, table_size, language, readonly: true }
            if legacy || correct().is_some() {
                h3 { {language.text("सही हल", "Worked solution")} }
                p { class: "passage", "{solution}" }
                if correct() == Some(true) {
                    CorrectAnswer { language, on_advance: move |_| {
                        on_input.call(Input::SubmitSolution { ink: work.clone(), correct: true });
                    } }
                } else {
                    if correct() == Some(false) {
                        AnswerFeedback { correct: false, text: language.text("फिर कोशिश करो।", "Try again next time.") }
                    }
                    button { class: "primary", onclick: move |_| {
                        let input = if legacy { Input::Submit(work.clone()) }
                            else { Input::SubmitSolution { ink: work.clone(), correct: false } };
                        on_input.call(input);
                    }, {language.text("आगे", "Next")} }
                }
            } else {
                NumberCheck { label: answer_label, expected: answer, language,
                    on_correct: move |_| correct.set(Some(true)), on_wrong: move |_| tried.set(true) }
                button { class: "quiet", onclick: move |_| { checked.set(None); tried.set(false); },
                    {language.text("लिखा हुआ बदलो", "Edit writing")} }
                if tried() {
                    button { class: "quiet", onclick: move |_| correct.set(Some(false)),
                        {language.text("हल दिखाओ", "Show worked solution")} }
                }
            }
        } else {
            if notebook {
                p {
                    {match paper {
                        Paper::NotebookCarry => language.text("आगे जाने वाली दहाई ऊपर लिखो। जवाब रेखा के नीचे लिखो।", "Write the carry above the tens. Write the answer below the line."),
                        Paper::NotebookBorrow => language.text("नई दहाई और इकाइयाँ ऊपर लिखो। जवाब रेखा के नीचे लिखो।", "Write the new tens and ones above. Write the answer below the line."),
                        _ => language.text("रेखा के नीचे जवाब लिखो।", "Write the answer under the line."),
                    }}
                }
            } else if table_size.is_some() {
                p { {language.text("हर पूरी संख्या एक खाने में लिखो, जैसे 10 या 100।", "Write each whole number in one box, such as 10 or 100.")} }
            } else {
                p { {language.text("खानों में अपना हल लिखो। हर खाने में एक अंक या चिह्न लिखो।", "Write your working in the grid. Use one square for each digit or symbol.")} }
            }
            {rsx! {
                InkPad { key: "{language.code()}", id: "solve-math-ink", model: "", initial, scale,
                    lines, paper, notebook: equation, table_size, language, done_label: label, on_ink,
                    on_done: move |ink| checked.set(Some(ink)) }
            }}
        }
    }
}
use crate::{language::Language, writing::PaperScale};
use dioxus::prelude::*;
#[component]
fn Choice(
    passage: String,
    question: Question,
    #[props(default)] recall: bool,
    language: Language,
    on_done: EventHandler<bool>,
) -> Element {
    let memory = try_use_context::<ActivityMemory>();
    let saved = memory.map(|m| m.0.peek().clone()).unwrap_or_default();
    let mut show = use_signal(|| !recall || saved.choices_shown);
    let mut missed = use_signal(|| saved.missed);
    let mut wrong = use_signal(|| saved.selected.is_some_and(|index| index != question.answer));
    let mut correct = use_signal(|| saved.selected == Some(question.answer));
    let mut selected = use_signal(|| saved.selected);
    use_effect(move || {
        let (shown, missed, selected) = (show(), missed(), selected());
        if let Some(mut memory) = memory {
            let mut next = memory.0.peek().clone();
            next.choices_shown = shown; next.missed = missed; next.selected = selected;
            if *memory.0.peek() != next { memory.0.set(next); }
        }
    });
    let q = question;
    let answer_label = language.text("सही जवाब है:", "The correct answer is");
    rsx! {
        if !passage.is_empty() { p { class: "passage", "{passage}" } }
        h2 { class: "exercise-prompt", "{q.prompt}" }
        if !show() {
            p { {language.text("पहले अपना जवाब सोचो।", "Think of your answer first.")} }
            button { class: "primary", onclick: move |_| show.set(true), {language.text("विकल्प देखो", "Show choices")} }
            button { class: "quiet", onclick: move |_| { missed.set(true); show.set(true); }, {language.text("संकेत चाहिए", "I need a hint")} }
        } else {
        div { class: "cards answers",
            for (index, option) in q.options.iter().enumerate() {
                button { key: "{index}", disabled: correct(), onclick: move |_| {
                    if !correct() {
                        selected.set(Some(index));
                        if index == q.answer { correct.set(true); }
                        else { missed.set(true); wrong.set(true); }
                    }
                }, "{option}" }
            }
        }
        if correct() {
            CorrectAnswer { language, on_advance: move |_| on_done.call(missed()) }
        } else if missed() {
            if wrong() {
                AnswerFeedback { correct: false, text: language.text(
                    "फिर कोशिश करो। सवाल और विकल्प फिर देखो।",
                    "Try again. Look at the question and choices once more.",
                ) }
            } else {
                p { role: "status", "{answer_label} {q.options[q.answer]}" }
            }
            button { class: "quiet", onclick: move |_| on_done.call(true), {language.text("आगे", "Next")} }
        }
        }
    }
}

#[component]
fn PassageActivity(
    script: String,
    question: Question,
    #[props(default)] listen: bool,
    language: Language,
    on_done: EventHandler<bool>,
) -> Element {
    let memory = try_use_context::<ActivityMemory>();
    let saved = memory.map(|m| m.0.peek().clone()).unwrap_or_default();
    let mut ready = use_signal(|| saved.passage_ready);
    use_effect(move || {
        let ready=ready();
        if let Some(mut memory)=memory {
            let mut next=memory.0.peek().clone(); next.passage_ready=ready;
            if *memory.0.peek()!=next { memory.0.set(next); }
        }
    });
    let repeat = if listen {
        language.text("फिर सुनो", "Listen again")
    } else {
        language.text("फिर पढ़ो", "Read again")
    };
    let next = if listen {
        language.text("सुन लिया", "I listened")
    } else {
        language.text("पढ़ लिया", "I’ve read it")
    };
    rsx! {
        if ready() {
            Choice { passage: String::new(), question, language, on_done }
            button { class: "quiet", onclick: move |_| ready.set(false), "{repeat}" }
        } else {
            if listen {
                h2 { {language.text("ध्यान से सुनो", "Listen carefully")} }
                details { open: true,
                    summary { {language.text("बड़े पढ़कर सुनाएँ", "An adult can read this aloud")} }
                    p { class: "passage", "{script}" }
                }
            } else {
                p { class: "passage", "{script}" }
            }
            button { class: "primary", onclick: move |_| ready.set(true), "{next}" }
        }
    }
}

#[component]
fn TalkActivity(
    picture: String,
    prompt: String,
    starters: Vec<String>,
    adult: String,
    initial: Ink,
    scale: PaperScale,
    on_ink: EventHandler<Ink>,
    on_written: EventHandler<Ink>,
    language: Language,
    paper: Paper,
    on_done: EventHandler<()>,
) -> Element {
    let memory=try_use_context::<ActivityMemory>();
    let saved=memory.map(|m|m.0.peek().clone()).unwrap_or_default();
    let mut picked=use_signal(||saved.starter);
    let mut write=use_signal(||saved.write_mode);
    use_effect(move || {
        let (starter,write_mode)=(picked(),write());
        if let Some(mut memory)=memory {
            let mut next=memory.0.peek().clone(); next.starter=starter; next.write_mode=write_mode;
            if *memory.0.peek()!=next { memory.0.set(next); }
        }
    });
    rsx! {
        p { class: "picture-cue", "{picture}" }
        h2 { "{prompt}" }
        details { summary { {language.text("बड़ों के लिए बातचीत का संकेत", "Conversation cue for adults")} } p { class: "learning-text", "{adult}" } }
        if write() {
            InkPad { key: "{language.code()}", language, id: "talk-ink", model: "", initial, scale, paper, on_ink, on_done: on_written }
        } else {
            div { class: "cards",
                for starter in starters {
                    button { key: "{starter}", aria_pressed: picked().as_deref() == Some(starter.as_str()),
                        class: "learning-text", onclick: move |_| picked.set(Some(starter.clone())), "{starter}" }
                }
            }
            if let Some(response) = picked() {
                p { class: "learning-text", role: "status", "{response}" }
            }
            button { class: "quiet", onclick: move |_| write.set(true), {language.text("लिखकर बताओ", "Write your answer")} }
            button { class: "primary", onclick: move |_| on_done.call(()), {language.text("बात हो गई", "Done talking")} }
        }
    }
}

#[component]
fn WrittenAnswer(
    passage: String,
    question: Question,
    initial: Ink,
    scale: PaperScale,
    on_ink: EventHandler<Ink>,
    on_written: EventHandler<Ink>,
    language: Language,
    paper: Paper,
    on_done: EventHandler<bool>,
) -> Element {
    let memory=try_use_context::<ActivityMemory>();
    let saved=memory.map(|m|m.0.peek().clone()).unwrap_or_default();
    let mut tap=use_signal(||saved.answer_mode);
    use_effect(move || {
        let tap=tap();
        if let Some(mut memory)=memory {
            let mut next=memory.0.peek().clone(); next.answer_mode=tap;
            if *memory.0.peek()!=next { memory.0.set(next); }
        }
    });
    rsx! {
        if tap() {
            Choice { passage, question, language, on_done }
        } else {
            p { class: "passage", "{passage}" }
            h2 { "{question.prompt}" }
            {rsx! {
                InkPad { key: "{language.code()}", language, id: "answer-ink", model: "", initial, scale, paper, on_ink,
                    on_done: on_written }
            }}
            button { class: "quiet", onclick: move |_| tap.set(true), {language.text("जवाब चुनो", "Choose an answer")} }
        }
    }
}

#[component]
pub(crate) fn TableConcept(size: u8, language: Language) -> Element {
    rsx! {
        details { class: "table-concept",
            summary { {language.text("पहाड़े का मतलब समझो", "Understand the table")} }
            p { {match language {
                Language::Hindi => format!("{size} × समूहों की संख्या: हर समूह में {size} वस्तुएँ हैं।"),
                Language::English => format!("{size} × number of groups: each group contains {size} objects."),
            }} }
            div { class: "table-concept-examples",
                for groups in 0u8..=10 {
                    figure { key: "{groups}",
                        figcaption { class: "table-concept-meaning",
                            p { {match language {
                                Language::Hindi => format!("{size} के {groups} समूह"),
                                Language::English if groups == 1 => format!("1 group of {size}"),
                                Language::English => format!("{groups} groups of {size}"),
                            }} }
                        strong { "{size} × {groups} = {size * groups}" }
                        }
                        div { class: "table-concept-pattern",
                        div { class: "table-group-diagram", aria_hidden: "true",
                            if groups == 0 {
                                span { class: "table-zero", "∅" }
                            }
                            for group in 0..groups {
                                if group > 0 { span { "+" } }
                                span { class: if group + 1 == groups { "table-object-group table-new-group" } else { "table-object-group" },
                                    for dot in 0..size { span { key: "{dot}", class: "table-dot" } }
                                }
                            }
                        }
                        p { class: "table-total", {language.text("कुल", "Total")} " = {size * groups}" }
                        if groups == 0 {
                            p { {language.text("कोई समूह नहीं, इसलिए कोई वस्तु नहीं।", "No groups means no objects.")} }
                        }
                        if groups > 0 { p { class: "table-addition",
                            {(0..groups).map(|_| size.to_string()).collect::<Vec<_>>().join(" + ")}
                            " = {size * groups}"
                        } }
                        if groups > 0 {
                            p { class: "table-from-previous",
                                {language.text("पिछला कुल", "Previous total")}
                                " {size * (groups - 1)} + {size} = {size * groups}"
                            }
                            p { class: "table-count-on",
                                {language.text("पिछले कुल से आगे गिनो: ", "Count on from the previous total: ")}
                                {(size * (groups - 1)..=size * groups).map(|number| number.to_string()).collect::<Vec<_>>().join(" → ")}
                            }
                        }
                        }
                    }
                }
            }
            p { {match language {
                Language::Hindi => format!("हर नया समूह जोड़ने पर कुल में {size} और जुड़ते हैं।"),
                Language::English => format!("Each new group adds {size} more to the total."),
            }} }
        }
    }
}

#[component]
fn TableActivity(
    size: u8,
    writing: bool,
    initial: Ink,
    scale: PaperScale,
    language: Language,
    on_input: EventHandler<Input>,
) -> Element {
    let mut completed = use_signal(|| [false; 10]);
    let mut missed = use_signal(|| false);
    let prompt = match language {
        Language::Hindi => format!("{size} का पूरा पहाड़ा: ×1 से ×10"),
        Language::English => format!("The complete {size} times table: ×1 to ×10"),
    };
    rsx! {
        TableConcept { size, language }
        section { class: "table-sheet",
        if writing {
            SolveActivity { prompt,
                solution: (1..=10).map(|groups| format!("{size} × {groups} = {}", size * groups)).collect::<Vec<_>>().join("\n"),
                answer: (1..=10).map(|groups| size * groups).collect::<Vec<_>>(),
                helper: None, notebook: false, table_size: Some(size), lines: 10, initial, helper_draft: HelperDraft::default(), scale, language,
                on_ink: move |ink| on_input.call(Input::Draft(ink)), on_input }
        } else {
            h2 { "{prompt}" }
            p { {language.text("सभी दस जवाब भरो और जाँचो।", "Fill in and check all ten answers.")} }
            div { class: "table-practice",
                for groups in 1u8..=10 {
                    NumberCheck { key: "{groups}", label: format!("{size} × {groups} ="),
                        expected: vec![size * groups], language,
                        on_correct: move |_| completed.write()[usize::from(groups - 1)] = true,
                        on_wrong: move |_| missed.set(true) }
                }
            }
            if completed().iter().all(|done| *done) {
                CorrectAnswer { language, on_advance: move |_| on_input.call(Input::Answer(missed())) }
            }
        }
        }
    }
}

#[component]
pub fn Activity(
    kind: Kind,
    #[props(default)] picture: Option<String>,
    initial: Ink,
    scale: PaperScale,
    #[props(default)] recall: bool,
    #[props(default)] helper_draft: HelperDraft,
    #[props(default)] language: Language,
    #[props(default)] ruled_paper: Paper,
    #[props(default)] handwriting: bool,
    #[props(default)] revision_source: Option<(Ink, usize)>,
    on_input: EventHandler<Input>,
) -> Element {
    let on_done = move |missed| on_input.call(Input::Answer(missed));
    let on_ink = move |ink| on_input.call(Input::Draft(ink));
    let on_written = move |ink| on_input.call(Input::Submit(ink));
    let picture_groups = if recall { None } else { crate::pictures::groups(&kind) };
    let content = match kind {
        Kind::Table { size, writing } => rsx! {
            TableActivity { size, writing, initial, scale, language, on_input }
        },
        Kind::Choice(question) => {
            rsx! { Choice { passage: "", question, recall, language, on_done } }
        }
        Kind::Picture(picture, question) => {
            let (src, alt) = picture.illustration();
            rsx! {
                img { class: "lesson-picture", src, alt }
                Choice { passage: "", question, language, on_done }
            }
        }
        Kind::Read(script, question) => {
            rsx! { PassageActivity { script, question, language, on_done } }
        }
        Kind::Listen(script, question) => {
            rsx! { PassageActivity { script, question, listen: true, language, on_done } }
        }
        Kind::Count {
            quantity,
            visual,
            question,
        } => rsx! {
            div { class: "count-objects", aria_label: language.text("गिनने के लिए चित्र", "Objects to count"),
                for index in 0..quantity { span { key: "{index}",
                    if visual == "🍎" { crate::pictures::Image { id: "apple".to_string(), language, compact: true } }
                    else if visual == "🔵" { crate::pictures::Image { id: "ball".to_string(), language, compact: true } }
                    else { "{visual}" }
                } }
            }
            Choice { passage: "", question, language, on_done }
        },
        Kind::Answer(passage, question) => rsx! {
            WrittenAnswer { passage, question, initial, scale, language, paper: ruled_paper, on_ink, on_written, on_done }
        },
        Kind::Talk {
            picture,
            prompt,
            starters,
            adult,
        } => rsx! {
            TalkActivity { picture, prompt, starters, adult, initial, scale, language, paper: ruled_paper, on_ink, on_written,
                on_done: move |_| on_done(false) }
        },
        Kind::Trace {
            model,
            guidance,
            paper,
        } => rsx! {
            h2 { {language.text("बिंदियों पर लिखो, फिर खुद लिखो", "Trace the dots, then write on your own")} }
            if !guidance.is_empty() { details { summary { {language.text("लिखने का तरीका", "How to write")} } p { class: "learning-text", "{guidance}" } } }
            {rsx! {
                InkPad { key: "{language.code()}", language, id: "trace-ink", model, initial, scale, paper, on_ink, on_done: on_written }
            }}
        },
        Kind::Compose(prompt, lines, dictation) => rsx! {
            h2 { class: "exercise-prompt", "{prompt}" }
            if let Some(words) = dictation {
                details { summary { {language.text("बड़े बोलकर सुनाएँ", "An adult can read this aloud")} } p { class: "learning-text", "{words}" } }
            }
            {rsx! {
                InkPad { key: "{language.code()}", language, id: "compose-ink", model: "", initial, scale, paper: ruled_paper, lines, on_ink, on_done: on_written }
            }}
        },
        Kind::Revise(prompt, lines) => rsx! {
            h2 { "{prompt}" }
            if let Some((ink, previous_lines)) = revision_source {
                p { {language.text("तुम्हारी पिछली लिखाई", "Your previous writing")} }
                InkPad { id: "previous-writing", model: "", initial: ink, scale,
                    lines: previous_lines, language, paper: ruled_paper, readonly: true }
            } else {
                p { {language.text("याद करके वाक्य लिखो।", "Write the sentence again from memory.")} }
            }
            InkPad { id: "revision-ink", model: "", initial, scale, lines, language,
                paper: ruled_paper, on_ink, on_done: on_written }
        },
        Kind::Solve {
            prompt,
            solution,
            answer,
            helper,
            lines,
            notebook,
        } => rsx! {
            SolveActivity { prompt, solution, answer, helper, notebook, lines, initial, helper_draft, scale, language, on_ink, on_input }
        },
    };
    rsx! { div { class: if handwriting { "handwriting-content" } else { "" },
        if let Some(id) = picture { crate::pictures::Image { id, language } }
        if let Some((groups,size)) = picture_groups {
            div { class: "picture-groups", aria_label: language.text("बराबर समूह", "Equal groups"),
                for group in 0..groups { div { key: "{group}",
                    for index in 0..size { crate::pictures::Image { key: "{index}", id: "ball".to_string(), language, compact: true } }
                } }
            }
        }
        {content} } }
}
