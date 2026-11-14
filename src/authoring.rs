use crate::writing::PaperScale;
use crate::{
    activity::Activity,
    content::{self, CustomCatalog, Exercise, Group, Kind, Paper, Question, Topic},
    language::Language,
    progress::Input,
    state::Ink,
};
use dioxus::prelude::*;

fn blank_topic() -> Topic {
    Topic {
        id: String::new(),
        title: String::new(),
        grade: 1,
        group: Group::new("parent-practice", "अतिरिक्त अभ्यास"),
        exercises: Vec::new(),
        recall: Vec::new(),
        break_every: None,
        revision: 0,
    }
}

fn blank_question() -> Question {
    Question::pair("", "", "", 0)
}

fn form(kind: &Kind) -> &'static str {
    match kind {
        Kind::Choice(_) => "choice",
        Kind::Read(..) => "read",
        Kind::Listen(..) => "listen",
        Kind::Trace { .. } => "trace",
        Kind::Compose(_, _, None) => "write",
        Kind::Answer(..)
        | Kind::Picture(..)
        | Kind::Revise(..)
        | Kind::Talk { .. }
        | Kind::Compose(_, _, Some(_))
        | Kind::Solve { .. }
        | Kind::Count { .. } => "copied",
    }
}

fn retarget_trace_paper(topic: &mut Topic, previous: Paper, destination: Paper) {
    for exercise in &mut topic.exercises {
        if let Kind::Trace { paper, .. } = &mut exercise.kind {
            if *paper == previous {
                *paper = destination;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_follows_new_subject_and_keeps_explicit_math_paper() {
        let mut topic = blank_topic();
        for paper in [Paper::Hindi, Paper::Digit] {
            topic.exercises.push(Exercise {
                id: content::new_id(),
                kind: empty_form("trace", paper),
            });
        }
        retarget_trace_paper(&mut topic, Paper::Hindi, Paper::English);
        assert!(matches!(
            topic.exercises[0].kind,
            Kind::Trace {
                paper: Paper::English,
                ..
            }
        ));
        assert!(matches!(
            topic.exercises[1].kind,
            Kind::Trace {
                paper: Paper::Digit,
                ..
            }
        ));
        retarget_trace_paper(&mut topic, Paper::English, Paper::Hindi);
        assert!(matches!(
            topic.exercises[0].kind,
            Kind::Trace {
                paper: Paper::Hindi,
                ..
            }
        ));
    }
}

fn empty_form(form: &str, paper: Paper) -> Kind {
    match form {
        "read" => Kind::Read(String::new(), blank_question()),
        "listen" => Kind::Listen(String::new(), blank_question()),
        "trace" => Kind::Trace {
            model: String::new(),
            guidance: String::new(),
            paper,
        },
        "write" => Kind::Compose(String::new(), 1, None),
        _ => Kind::Choice(blank_question()),
    }
}

#[component]
pub fn Authoring(
    catalog: CustomCatalog,
    scale: PaperScale,
    publish: Callback<CustomCatalog, Result<(), String>>,
    on_back: EventHandler<()>,
) -> Element {
    let mut subject = use_signal(|| content::BUNDLED[0].id.clone());
    let mut name = use_signal(String::new);
    let mut draft = use_signal(|| None::<Topic>);
    let mut preview = use_signal(|| None::<usize>);
    let mut previous_writing = use_signal(|| None::<(Ink, usize)>);
    let mut error = use_signal(String::new);
    let mut notice = use_signal(String::new);
    let subject_id = subject();
    let paper = Paper::for_subject(&subject_id);
    let language = if paper == Paper::English {
        Language::English
    } else {
        Language::Hindi
    };
    let current = catalog.subjects().find(|item| item.id == subject_id);
    let custom_subject = catalog.subjects.iter().any(|item| item.id == subject_id);
    let authored = catalog
        .additions
        .get(&subject_id)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let publish_topic = catalog.clone();
    let publish_subject = catalog.clone();
    let rename_subject = catalog.clone();

    rsx! {
        section { class: "parent authoring", lang: "en",
            h1 { "Manage activities" }
            button { class: "quiet", onclick: move |_| on_back.call(()), "Parent settings" }
            if !error().is_empty() { p { role: "alert", "{error}" } }
            if !notice().is_empty() { p { role: "status", "{notice}" } }

            if let Some(topic) = draft() {
                if let Some(index) = preview() {
                    h2 { "Learner preview" }
                    button { class: "quiet", onclick: move |_| preview.set(None), "Back to draft" }
                    if let Some(exercise) = topic.exercises.get(index) {
                        {
                            let output_lines = exercise.kind.writing(paper).map(|writing| writing.lines);
                            rsx! {
                                Activity { key: "exercise-{index}", kind: exercise.kind.clone(), initial: Vec::new(), scale,
                                    language, ruled_paper: paper, revision_source: previous_writing(),
                                    on_input: move |input: Input| if !input.is_draft() {
                                        previous_writing.set(match input {
                                            Input::Submit(ink) => output_lines.map(|lines| (ink, lines)),
                                            _ => None,
                                        });
                                        preview.set(Some(index + 1));
                                    }
                                }
                            }
                        }
                    } else if let Some(question) = topic.recall.get(index - topic.exercises.len()) {
                        Activity { key: "recall-{index}", kind: Kind::Choice(question.clone()), initial: Vec::new(), scale, recall: true,
                            language,
                            on_input: move |_| preview.set(Some(index + 1)) }
                    } else {
                        p { "Preview complete." }
                    }
                } else {
                    h2 { "Activity draft" }
                    if topic.id.is_empty() {
                        label { "Publish to subject"
                            select { value: "{subject_id}", onchange: move |event| {
                                let destination = event.value();
                                if let Some(topic) = draft.write().as_mut() {
                                    retarget_trace_paper(topic, paper, Paper::for_subject(&destination));
                                }
                                subject.set(destination);
                            },
                                for item in catalog.subjects() {
                                    option { key: "{item.id}", value: "{item.id}", "{item.name}" }
                                }
                            }
                        }
                    }
                    label { "Activity title"
                        input { value: "{topic.title}", oninput: move |event| {
                            if let Some(topic) = draft.write().as_mut() { topic.title = event.value(); }
                        } }
                    }
                    label { "Class"
                        select { value: "{topic.grade}", onchange: move |event| {
                            if let (Ok(grade), Some(topic)) = (event.value().parse(), draft.write().as_mut()) {
                                topic.grade = grade;
                            }
                        },
                            option { value: "1", "Class 1" }
                            option { value: "2", "Class 2" }
                        }
                    }
                    h3 { "Exercises" }
                    for (index, exercise) in topic.exercises.iter().enumerate() {
                        ExerciseFields { key: "{exercise.id}", kind: exercise.kind.clone(), index, count: topic.exercises.len(), default_paper: paper,
                            on_change: move |kind| {
                                if let Some(topic) = draft.write().as_mut() { topic.exercises[index].kind = kind; }
                            },
                            on_move: move |target| {
                                if let Some(topic) = draft.write().as_mut() { topic.exercises.swap(index, target); }
                            },
                            on_remove: move |_| {
                                if let Some(topic) = draft.write().as_mut() { topic.exercises.remove(index); }
                            }
                        }
                    }
                    button { onclick: move |_| {
                        if let Some(topic) = draft.write().as_mut() {
                            topic.exercises.push(Exercise { id: content::new_id(), kind: empty_form("choice", paper) });
                        }
                    }, "Add exercise" }
                    h3 { "Recall questions" }
                    for (index, question) in topic.recall.iter().enumerate() {
                        fieldset { key: "{index}",
                            legend { "Recall {index + 1}" }
                            QuestionFields { question: question.clone(), on_change: move |question| {
                                if let Some(topic) = draft.write().as_mut() { topic.recall[index] = question; }
                            } }
                            button { class: "quiet", onclick: move |_| {
                                if let Some(topic) = draft.write().as_mut() { topic.recall.remove(index); }
                            }, "Remove recall question" }
                        }
                    }
                    button { onclick: move |_| {
                        if let Some(topic) = draft.write().as_mut() { topic.recall.push(blank_question()); }
                    }, "Add recall question" }
                    div { class: "actions",
                        button { onclick: move |_| {
                            if let Some(topic) = draft() {
                                match topic.validate() {
                                    Ok(()) => { error.set(String::new()); previous_writing.set(None); preview.set(Some(0)); }
                                    Err(message) => error.set(message),
                                }
                            }
                        }, "Preview" }
                        button { class: "primary", onclick: move |_| {
                            if let Some(topic) = draft() {
                                let mut updated = publish_topic.clone();
                                match updated.publish(&subject.peek(), topic).and_then(|()| publish.call(updated)) {
                                    Ok(()) => { draft.set(None); error.set(String::new()); notice.set("Activity published.".into()); }
                                    Err(message) => error.set(message),
                                }
                            }
                        }, "Publish" }
                        button { class: "quiet", onclick: move |_| { draft.set(None); error.set(String::new()); }, "Discard draft" }
                    }
                }
            } else {
                label { "Subject"
                    select { value: "{subject_id}", onchange: move |event| {
                        subject.set(event.value()); name.set(String::new()); error.set(String::new()); notice.set(String::new());
                    },
                        for item in catalog.subjects() {
                            option { key: "{item.id}", value: "{item.id}", "{item.name}" }
                        }
                    }
                }
                label { "Subject name"
                    input { value: "{name}", oninput: move |event| name.set(event.value()) }
                }
                div { class: "actions",
                    button { onclick: move |_| {
                        let mut updated = publish_subject.clone();
                        let result = updated.create_subject(&name()).and_then(|()| {
                            let id = updated.subjects.last().expect("subject just added").id.clone();
                            publish.call(updated)?;
                            Ok(id)
                        });
                        match result {
                            Ok(id) => { subject.set(id); name.set(String::new()); error.set(String::new()); notice.set("Subject created.".into()); }
                            Err(message) => error.set(message),
                        }
                    }, "Create subject" }
                    if custom_subject {
                        button { onclick: move |_| {
                            let mut updated = rename_subject.clone();
                            let title = name().trim().to_owned();
                            if title.is_empty() { error.set("Enter a subject name.".into()); return; }
                            let Some(item) = updated.subjects.iter_mut().find(|item| item.id == *subject.peek()) else { return };
                            item.name = title;
                            match publish.call(updated) {
                                Ok(()) => { name.set(String::new()); error.set(String::new()); notice.set("Subject renamed.".into()); }
                                Err(message) => error.set(message),
                            }
                        }, "Rename selected subject" }
                    }
                    button { class: "primary", onclick: move |_| { notice.set(String::new()); error.set(String::new()); draft.set(Some(blank_topic())); }, "New activity" }
                }
                if !authored.is_empty() {
                    h2 { "Published activities" }
                    for topic in authored {
                        {
                            let topic = topic.clone();
                            rsx! { button { key: "{topic.id}", onclick: move |_| { error.set(String::new()); notice.set(String::new()); draft.set(Some(topic.clone())); },
                                "Edit {topic.title}"
                            } }
                        }
                    }
                }
                if let Some(item) = current {
                    if !item.topics.is_empty() {
                        details {
                            summary { "Copy a bundled activity" }
                            div { class: "cards",
                                for topic in &item.topics {
                                    {
                                        let topic = topic.clone();
                                        rsx! { button { key: "{topic.id}", onclick: move |_| {
                                            let mut copy = topic.clone();
                                            copy.id.clear();
                                            copy.revision = 0;
                                            copy.title = format!("{} (copy)", copy.title);
                                            copy.group = Group::new("parent-practice", "अतिरिक्त अभ्यास");
                                            notice.set(String::new());
                                            error.set(String::new());
                                            draft.set(Some(copy));
                                        }, "{topic.title}" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExerciseFields(
    default_paper: Paper,
    kind: Kind,
    index: usize,
    count: usize,
    on_change: EventHandler<Kind>,
    on_move: EventHandler<usize>,
    on_remove: EventHandler<()>,
) -> Element {
    let form = form(&kind);
    let fields = match kind {
        Kind::Choice(question) => {
            rsx! { QuestionFields { question, on_change: move |question| on_change.call(Kind::Choice(question)) } }
        }
        Kind::Read(text, question) | Kind::Listen(text, question) => {
            let question_for_text = question.clone();
            let text_for_question = text.clone();
            let update = move |(text, question)| {
                on_change.call(if form == "read" {
                    Kind::Read(text, question)
                } else {
                    Kind::Listen(text, question)
                })
            };
            rsx! {
                label { if form == "read" { "Passage" } else { "Read-aloud script" }
                    textarea { value: "{text}", oninput: move |event| update((event.value(), question_for_text.clone())) }
                }
                QuestionFields { question, on_change: move |question| update((text_for_question.clone(), question)) }
            }
        }
        Kind::Trace {
            model,
            guidance,
            paper,
        } => {
            let previous_model = model.clone();
            let previous_guidance = guidance.clone();
            rsx! {
                label { "Trace model"
                    input { value: "{model}", oninput: move |event| on_change.call(Kind::Trace { model: event.value(), guidance: previous_guidance.clone(), paper }) }
                }
                label { "Writing guidance"
                    input { value: "{guidance}", oninput: move |event| on_change.call(Kind::Trace { model: previous_model.clone(), guidance: event.value(), paper }) }
                }
            }
        }
        Kind::Compose(prompt, lines, None) => {
            let previous_prompt = prompt.clone();
            rsx! {
                label { "Writing prompt"
                    textarea { value: "{prompt}", oninput: move |event| on_change.call(Kind::Compose(event.value(), lines, None)) }
                }
                label { "Writing lines"
                    select { value: "{lines}", onchange: move |event| {
                        if let Ok(lines) = event.value().parse() { on_change.call(Kind::Compose(previous_prompt.clone(), lines, None)); }
                    },
                        for line in 1..=5 { option { value: "{line}", "{line}" } }
                    }
                }
            }
        }
        _ => rsx! { p { "This copied exercise can be previewed, moved, or removed." } },
    };

    rsx! {
        fieldset {
            legend { "Exercise {index + 1}" }
            if form != "copied" {
                label { "Exercise type"
                    select { value: "{form}", onchange: move |event| on_change.call(empty_form(&event.value(), default_paper)),
                        option { value: "choice", "Multiple choice" }
                        option { value: "read", "Read and answer" }
                        option { value: "listen", "Listen and answer" }
                        option { value: "trace", "Trace and write" }
                        option { value: "write", "Blank writing" }
                    }
                }
            }
            {fields}
            div { class: "actions",
                button { disabled: index == 0, onclick: move |_| on_move.call(index.saturating_sub(1)), "Move up" }
                button { disabled: index + 1 == count, onclick: move |_| on_move.call(index + 1), "Move down" }
                button { class: "quiet", onclick: move |_| on_remove.call(()), "Remove exercise" }
            }
        }
    }
}

#[component]
fn QuestionFields(question: Question, on_change: EventHandler<Question>) -> Element {
    let prompt = question.clone();
    let correct = question.clone();
    rsx! {
        label { "Question"
            input { value: "{question.prompt}", oninput: move |event| {
                let mut updated = prompt.clone();
                updated.prompt = event.value();
                on_change.call(updated);
            } }
        }
        for (index, option) in question.options.iter().enumerate() {
            {
                let question = question.clone();
                rsx! {
                    label { key: "{index}", "Choice {index + 1}"
                        input { value: "{option}", oninput: move |event| {
                            let mut updated = question.clone();
                            updated.options[index] = event.value();
                            on_change.call(updated);
                        } }
                    }
                }
            }
        }
        label { "Correct choice"
            select { value: "{question.answer}", onchange: move |event| {
                if let Ok(index) = event.value().parse() {
                    let mut updated = correct.clone();
                    updated.answer = index;
                    on_change.call(updated);
                }
            },
                for index in 0..question.options.len() {
                    option { value: "{index}", "Choice {index + 1}" }
                }
            }
        }
    }
}
