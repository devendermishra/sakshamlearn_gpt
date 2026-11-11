use crate::writing::PaperScale;
use crate::{
    content::{Kind, Question},
    progress::Input,
    state::Ink,
    writing::InkPad,
};
use dioxus::prelude::*;
#[component]
fn Choice(
    passage: String,
    question: Question,
    #[props(default)] recall: bool,
    on_done: EventHandler<bool>,
) -> Element {
    let mut show = use_signal(|| !recall);
    let mut missed = use_signal(|| false);
    let mut correct = use_signal(|| false);
    let q = question;
    rsx! {
        if !passage.is_empty() { p { class: "passage", "{passage}" } }
        h2 { "{q.prompt}" }
        if !show() {
            p { "पहले अपना जवाब सोचो।" }
            button { class: "primary", onclick: move |_| show.set(true), "विकल्प देखो" }
            button { class: "quiet", onclick: move |_| { missed.set(true); show.set(true); }, "संकेत चाहिए" }
        } else {
        div { class: "cards answers",
            for (index, option) in q.options.iter().enumerate() {
                button { key: "{index}", disabled: correct(), onclick: move |_| {
                    if index == q.answer { correct.set(true); } else { missed.set(true); }
                }, "{option}" }
            }
        }
        if correct() {
            p { role: "status", "सही!" }
            button { class: "primary", onclick: move |_| on_done.call(missed()), "आगे" }
        } else if missed() {
            p { role: "status", "सही जवाब है: {q.options[q.answer]}" }
            button { class: "quiet", onclick: move |_| on_done.call(true), "आगे" }
        }
        }
    }
}

#[component]
fn PassageActivity(
    script: String,
    question: Question,
    #[props(default)] listen: bool,
    on_done: EventHandler<bool>,
) -> Element {
    let mut ready = use_signal(|| false);
    let repeat = if listen {
        "फिर सुनो"
    } else {
        "फिर पढ़ो"
    };
    let next = if listen {
        "सुन लिया"
    } else {
        "पढ़ लिया"
    };
    rsx! {
        if ready() {
            Choice { passage: String::new(), question, on_done }
            button { class: "quiet", onclick: move |_| ready.set(false), "{repeat}" }
        } else {
            if listen {
                h2 { "ध्यान से सुनो" }
                details { open: true,
                    summary { "बड़े पढ़कर सुनाएँ" }
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
    on_done: EventHandler<()>,
) -> Element {
    let mut picked = use_signal(|| None::<String>);
    let mut write = use_signal(|| false);
    rsx! {
        p { class: "picture-cue", "{picture}" }
        h2 { "{prompt}" }
        details { summary { "बड़ों के लिए बातचीत का संकेत" } p { "{adult}" } }
        if write() {
            InkPad { id: "talk-ink", model: "", initial, scale, on_ink, on_done: on_written }
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
            button { class: "quiet", onclick: move |_| write.set(true), "लिखकर बताओ" }
            button { class: "primary", onclick: move |_| on_done.call(()), "बात हो गई" }
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
    on_done: EventHandler<bool>,
) -> Element {
    let mut tap = use_signal(|| false);
    rsx! {
        if tap() {
            Choice { passage, question, on_done }
        } else {
            p { class: "passage", "{passage}" }
            h2 { "{question.prompt}" }
            InkPad { id: "answer-ink", model: "", initial, scale, on_ink,
                on_done: on_written }
            button { class: "quiet", onclick: move |_| tap.set(true), "जवाब चुनो" }
        }
    }
}

#[component]
pub fn Activity(
    kind: Kind,
    initial: Ink,
    scale: PaperScale,
    #[props(default)] recall: bool,
    on_input: EventHandler<Input>,
) -> Element {
    let on_done = move |missed| on_input.call(Input::Answer(missed));
    let on_ink = move |ink| on_input.call(Input::Draft(ink));
    let on_written = move |ink| on_input.call(Input::Submit(ink));
    match kind {
        Kind::Choice(question) => rsx! { Choice { passage: "", question, recall, on_done } },
        Kind::Read(script, question) => rsx! { PassageActivity { script, question, on_done } },
        Kind::Listen(script, question) => {
            rsx! { PassageActivity { script, question, listen: true, on_done } }
        }
        Kind::Count {
            quantity,
            visual,
            question,
        } => rsx! {
            div { class: "count-objects", aria_label: "गिनने के लिए चित्र",
                for index in 0..quantity { span { key: "{index}", "{visual}" } }
            }
            Choice { passage: "", question, on_done }
        },
        Kind::Answer(passage, question) => rsx! {
            WrittenAnswer { passage, question, initial, scale, on_ink, on_written, on_done }
        },
        Kind::Talk {
            picture,
            prompt,
            starters,
            adult,
        } => rsx! {
            TalkActivity { picture, prompt, starters, adult, initial, scale, on_ink, on_written,
                on_done: move |_| on_done(false) }
        },
        Kind::Trace {
            model,
            guidance,
            paper,
        } => rsx! {
            h2 { "बिंदियों पर लिखो, फिर खुद लिखो" }
            if !guidance.is_empty() { details { summary { "लिखने का तरीका" } p { "{guidance}" } } }
            InkPad { id: "trace-ink", model, initial, scale, paper, on_ink, on_done: on_written }
        },
        Kind::Compose(prompt, lines, dictation) => rsx! {
            h2 { "{prompt}" }
            if let Some(words) = dictation {
                details { summary { "बड़े बोलकर सुनाएँ" } p { "{words}" } }
            }
            InkPad { id: "compose-ink", model: "", initial, scale, lines, on_ink, on_done: on_written }
        },
    }
}
