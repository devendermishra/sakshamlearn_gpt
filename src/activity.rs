use crate::{
    content::{ArithmeticHelp, Kind, Paper, Question},
    math_help::{ArithmeticHelper, NumberCheck},
    progress::Input,
    state::{HelperDraft, Ink},
    writing::InkPad,
};

#[component]
fn SolveActivity(
    prompt: String,
    solution: String,
    answer: Vec<u8>,
    helper: Option<ArithmeticHelp>,
    notebook: bool,
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
                scale, lines, paper, notebook: equation, language, readonly: true }
            if legacy || correct().is_some() {
                if correct() == Some(true) {
                    p { role: "status", {language.text("सही!", "Correct!")} }
                }
                h3 { {language.text("सही हल", "Worked solution")} }
                p { class: "passage", "{solution}" }
                button { class: "primary", onclick: move |_| {
                    let input = match correct() {
                        Some(correct) => Input::SubmitSolution { ink: work.clone(), correct },
                        None => Input::Submit(work.clone()),
                    };
                    on_input.call(input);
                }, {language.text("आगे", "Next")} }
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
            } else {
                p { {language.text("खानों में अपना हल लिखो। हर खाने में एक अंक या चिह्न लिखो।", "Write your working in the grid. Use one square for each digit or symbol.")} }
            }
            {rsx! {
                InkPad { key: "{language.code()}", id: "solve-math-ink", model: "", initial, scale,
                    lines, paper, notebook: equation, language, done_label: label, on_ink,
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
    let mut show = use_signal(|| !recall);
    let mut missed = use_signal(|| false);
    let mut correct = use_signal(|| false);
    let q = question;
    let answer_label = language.text("सही जवाब है:", "The correct answer is");
    rsx! {
        if !passage.is_empty() { p { class: "passage", "{passage}" } }
        h2 { "{q.prompt}" }
        if !show() {
            p { {language.text("पहले अपना जवाब सोचो।", "Think of your answer first.")} }
            button { class: "primary", onclick: move |_| show.set(true), {language.text("विकल्प देखो", "Show choices")} }
            button { class: "quiet", onclick: move |_| { missed.set(true); show.set(true); }, {language.text("संकेत चाहिए", "I need a hint")} }
        } else {
        div { class: "cards answers",
            for (index, option) in q.options.iter().enumerate() {
                button { key: "{index}", disabled: correct(), onclick: move |_| {
                    if index == q.answer { correct.set(true); } else { missed.set(true); }
                }, "{option}" }
            }
        }
        if correct() {
            p { role: "status", {language.text("सही!", "Correct!")} }
            button { class: "primary", onclick: move |_| on_done.call(missed()), {language.text("आगे", "Next")} }
        } else if missed() {
            p { role: "status", "{answer_label} {q.options[q.answer]}" }
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
    let mut ready = use_signal(|| false);
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
    let mut picked = use_signal(|| None::<String>);
    let mut write = use_signal(|| false);
    rsx! {
        p { class: "picture-cue", "{picture}" }
        h2 { "{prompt}" }
        details { summary { {language.text("बड़ों के लिए बातचीत का संकेत", "Conversation cue for adults")} } p { "{adult}" } }
        if write() {
            InkPad { key: "{language.code()}", language, id: "talk-ink", model: "", initial, scale, paper, on_ink, on_done: on_written }
        } else {
            div { class: "cards",
                for starter in starters {
                    button { key: "{starter}", aria_pressed: picked().as_deref() == Some(starter.as_str()),
                        onclick: move |_| picked.set(Some(starter.clone())), "{starter}" }
                }
            }
            if let Some(response) = picked() {
                p { role: "status", "{response}" }
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
    let mut tap = use_signal(|| false);
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
pub fn Activity(
    kind: Kind,
    initial: Ink,
    scale: PaperScale,
    #[props(default)] recall: bool,
    #[props(default)] helper_draft: HelperDraft,
    #[props(default)] language: Language,
    #[props(default)] ruled_paper: Paper,
    #[props(default)] revision_source: Option<(Ink, usize)>,
    on_input: EventHandler<Input>,
) -> Element {
    let on_done = move |missed| on_input.call(Input::Answer(missed));
    let on_ink = move |ink| on_input.call(Input::Draft(ink));
    let on_written = move |ink| on_input.call(Input::Submit(ink));
    match kind {
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
                for index in 0..quantity { span { key: "{index}", "{visual}" } }
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
            if !guidance.is_empty() { details { summary { {language.text("लिखने का तरीका", "How to write")} } p { "{guidance}" } } }
            {rsx! {
                InkPad { key: "{language.code()}", language, id: "trace-ink", model, initial, scale, paper, on_ink, on_done: on_written }
            }}
        },
        Kind::Compose(prompt, lines, dictation) => rsx! {
            h2 { "{prompt}" }
            if let Some(words) = dictation {
                details { summary { {language.text("बड़े बोलकर सुनाएँ", "An adult can read this aloud")} } p { "{words}" } }
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
    }
}
