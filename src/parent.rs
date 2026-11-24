use crate::{
    authoring::Authoring,
    content::CustomCatalog,
    keypad::{Keys, NumericField},
    language::Language,
    focus::{Controls, FocusButton},
    state::{self, Pin, State},
    writing::{InkPad, PaperScale, CALIBRATION_MM},
};
use dioxus::prelude::*;
#[component]
pub fn Parent(
    data: Signal<State>,
    durable: Callback<State,Result<(),String>>,
    save_pin: Callback<Pin, Result<(), String>>,
    focus_controls: Controls,
    publish: Callback<CustomCatalog, Result<(), String>>,
    save_calibration: Callback<PaperScale, Result<(), String>>,
) -> Element {
    let mut manage = use_signal(|| false);
    let mut authorized = use_signal(|| false);
    let mut input = use_signal(String::new);
    let mut error = use_signal(String::new);
    let current = data.read();
    let scale = PaperScale::from_saved(current.scale, current.vertical_scale);
    let measured = current.vertical_scale.is_some();
    drop(current);
    let mut selected = use_signal(|| None::<String>);
    let create = data.read().pin.is_none();
    let mut history_page = use_signal(|| 0usize);
    if authorized() && manage() {
        return rsx! { Authoring { catalog: data.read().catalog.clone(), scale, publish, on_back: move |_| manage.set(false) } };
    }
    let snapshot = data.read();
    // Borrow ink; only build lightweight rows after parent authorization.
    let mut samples = std::collections::BTreeMap::new();
    if authorized() {
        for (id, sample) in &snapshot.samples { samples.insert(id.clone(), sample); }
        for (exercise, history) in &snapshot.history {
            for (index, sample) in history.iter().enumerate() {
                samples.insert(format!("{exercise}/attempt/{index:08}"), sample);
            }
        }
    }
    let pages = samples.len().div_ceil(30).max(1);
    let current_page = history_page().min(pages - 1);
    let review_count = data.read().due_review_count(state::day());
    rsx! {
        section { lang: "en", class: "parent",
            h1 { "Parent settings" }
            if !authorized() {
                form { onsubmit: move |event| {
                    event.prevent_default();
                    let value = input();
                    let current = data.peek().pin.clone();
                    if value.len() < 4 || value.len() > 8 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                        error.set("Use 4 to 8 digits.".into());
                    } else if let Some(pin) = current {
                        if pin.matches(&value) { authorized.set(true); error.set(String::new()); }
                        else { error.set("Incorrect PIN.".into()); }
                    } else {
                        match save_pin.call(Pin::new(&value)) {
                            Ok(()) => { authorized.set(true); error.set(String::new()); }
                            Err(message) => error.set(message),
                        }
                    }
                    input.set(String::new());
                },
                    NumericField { label: if create { "Set a parent PIN" } else { "Enter your parent PIN" },
                        value: input(), language: Language::English, limit: 8, keys: Keys::Pin,
                        on_change: move |value| input.set(value) }
                    button { class: "primary", r#type: "submit", "Continue" }
                }
            } else {
                button { class: "primary", onclick: move |_| manage.set(true), "Manage activities" }
                FocusButton { controls: focus_controls, has_pin: true, language: Language::English }
                Calibration { scale, measured, on_save: save_calibration }
                MasteryDashboard { data, durable }
                h2 { "Writing review" }
                p { "Due reviews: {review_count}" }
                if samples.is_empty() { p { "Saved writing will appear here." } }
                if pages > 1 {
                    nav { class: "actions",
                        button { disabled: current_page == 0, onclick: move |_| history_page.set(current_page - 1), "Previous samples" }
                        span { "Page {current_page + 1} / {pages}" }
                        button { disabled: current_page + 1 >= pages, onclick: move |_| history_page.set(current_page + 1), "Next samples" }
                    }
                }
                for (id, sample) in samples.iter().skip(current_page * 30).take(30) {
                    {
                        let id = id.clone();
                        let date = sample.saved_on.as_deref().unwrap_or("Draft or earlier sample");
                        let attempt = id.rsplit_once("/attempt/").and_then(|(_, index)| index.parse::<usize>().ok()).map(|index| format!(" · Attempt {}", index + 1)).unwrap_or_default();
                        rsx! { button { key: "{id}", class: "quiet", onclick: move |_| selected.set(Some(id.clone())),
                            "{sample.label} · {date}{attempt}"
                        } }
                    }
                }
                if let Some(id) = selected() {
                    if let Some(sample) = samples.get(&id) {
                        InkPad { key: "{id}", id: "review-ink", model: sample.model.clone(),
                            initial: sample.ink.clone(), scale, lines: sample.lines, paper: sample.paper, readonly: true,
                            language: if sample.paper == crate::content::Paper::English { crate::language::Language::English } else { crate::language::Language::Hindi } }
                        if let Some(solution) = &sample.solution {
                            p { strong { "Solution " } "{solution}" }
                        }
                        if let Some(correct) = sample.correct {
                            p { if correct { "Correct answer entered" } else { "Worked solution opened after an incorrect answer" } }
                        }
                    }
                }
            }
            p { role: "status", "{error}" }
        }
    }
}

#[component]
fn Calibration(
    scale: PaperScale,
    measured: bool,
    on_save: Callback<PaperScale, Result<(), String>>,
) -> Element {
    let mut draft = use_signal(|| scale);
    let mut width = use_signal(String::new);
    let mut height = use_signal(String::new);
    let mut adjusted = use_signal(|| false);
    let mut status = use_signal(String::new);
    let mut error = use_signal(String::new);
    let corrected = draft();
    let reference_style = format!(
        "width:{}px;height:{}px",
        CALIBRATION_MM * corrected.horizontal,
        CALIBRATION_MM * corrected.vertical
    );

    rsx! {
        section { class: "calibration",
            h2 { "Calibrate handwriting size" }
            p {
                if measured { "This device has a saved ruler calibration." }
                else { "This device is using an estimated size. Measure with a ruler before practising." }
            }
            p { "Measure the horizontal and vertical edges of the square from corner to corner. Enter both lengths in millimetres, then select Adjust guides." }
            div { class: "ink-scroll",
                div { class: "calibration-square", style: "{reference_style}",
                    span { "80 mm" }
                }
            }
            div { class: "actions",
                NumericField { label: "Measured width (mm)", value: width(), language: Language::English,
                    limit: 12, keys: Keys::Decimal,
                    on_change: move |value| { width.set(value); adjusted.set(false); status.set(String::new()); } }
                NumericField { label: "Measured height (mm)", value: height(), language: Language::English,
                    limit: 12, keys: Keys::Decimal,
                    on_change: move |value| { height.set(value); adjusted.set(false); status.set(String::new()); } }
            }
            div { class: "actions",
                button { onclick: move |_| {
                    let corrected = width().parse::<f64>().ok().zip(height().parse::<f64>().ok())
                        .and_then(|(width, height)| draft().corrected(width, height));
                    if let Some(corrected) = corrected {
                        draft.set(corrected);
                        adjusted.set(true);
                        width.set(String::new());
                        height.set(String::new());
                        error.set(String::new());
                        status.set("Check both guides below with your ruler, then save.".into());
                    } else {
                        error.set("Enter both ruler measurements in millimetres.".into());
                    }
                }, "Adjust guides" }
                button { class: "primary", disabled: !adjusted(), onclick: move |_| {
                    match on_save.call(draft()) {
                        Ok(()) => {
                            adjusted.set(false);
                            error.set(String::new());
                            status.set("Ruler calibration saved.".into());
                        }
                        Err(message) => error.set(format!("Could not save calibration: {message}")),
                    }
                }, "Save calibration" }
            }
            if !error().is_empty() { p { role: "alert", "{error}" } }
            p { role: "status", "{status}" }
            details { open: adjusted(),
                summary { "Check notebook guides" }
                p { "Hindi: 8 mm between the two writing rules on each row." }
                InkPad { id: "check-hindi", model: "क", initial: Vec::new(), scale: corrected, readonly: true }
                p { "Mathematics: 110 mm across ten squares. Each square is 11 mm wide and 11 mm tall." }
                InkPad { id: "check-math", model: "1", initial: Vec::new(), scale: corrected,
                    paper: crate::content::Paper::Digit, readonly: true }
                p { "English: 4 mm between adjacent writing guides, including the two blue lines." }
                InkPad { id: "check-english", model: "", initial: Vec::new(), scale: corrected,
                    paper: crate::content::Paper::English, readonly: true }
                p { "Repeat the ruler check if you change your device’s display scaling." }
            }
        }
    }
}


#[component]
fn MasteryDashboard(data: Signal<State>, durable: Callback<State,Result<(),String>>) -> Element {
    let mut error = use_signal(String::new);
    let snapshot = data.read();
    let mut selected_subject = use_signal(|| "en".to_string());
    let mut selected_grade = use_signal(|| 2u8);
    let mut evidence_page = use_signal(||0usize);
    let catalog = crate::assessment::catalog();
    let mut competencies = std::collections::BTreeMap::new();
    for m in &catalog { if m.subject==selected_subject() && m.grade==selected_grade() { competencies.entry(m.competency.clone()).or_insert(m); } }
    let mut competencies:Vec<_>=competencies.into_iter().collect();
    competencies.sort_by_key(|(id,_)|(!snapshot.evidence.attempts.values().any(|a|a.competency_id==*id),id.clone()));
    let overdue=snapshot.evidence.reviews.values().filter(|c|c.due<=state::day()).count();
    let pages=competencies.len().div_ceil(40).max(1);
    let current_page=evidence_page().min(pages-1);
    let recommendation = crate::planner::build(&snapshot,selected_grade(),state::day());
    rsx! { section { class: "mastery-dashboard",
        h2 { "Learning evidence" }
        p { "These are rule-based estimates. Independence is observed within the app; adult help may be unreported. Completion alone is not mastery." }
        p { "Overdue competency checks: {overdue}" }
        p { "Legacy and current topic completions: {snapshot.finished.len()}" }
        label { "Daily estimated minutes "
            select { aria_label: "Daily estimated minutes", value: "{snapshot.daily_minutes}",
                onchange: move |e| {
                    if let Ok(value) = e.value().parse::<u16>() { let mut next=data.peek().clone(); next.daily_minutes=value.clamp(5,30);
                        match durable.call(next) { Ok(())=>error.set(String::new()), Err(e)=>error.set(e) } }
                },
                for minutes in 5..=30 { option { value: "{minutes}", "{minutes}" } }
            }
        }
        h3 { "Subject summaries" }
        for subject in ["en","hi","math"] {
            {
                let ids: std::collections::BTreeSet<_> = catalog.iter().filter(|m|m.subject==subject).map(|m|m.competency.as_str()).collect();
                let established = ids.iter().filter(|id|matches!(crate::mastery::estimate(&snapshot.evidence,id).level,crate::mastery::Level::IndependentSuccess|crate::mastery::Level::Retained)).count();
                let retained = ids.iter().filter(|id|crate::mastery::estimate(&snapshot.evidence,id).level==crate::mastery::Level::Retained).count();
                rsx! { p { "{subject}: {established} / {ids.len()} with independent success; {retained} retained" } }
            }
        }
        label { "Subject " select { aria_label: "Evidence subject", value: selected_subject(), onchange: move |e| { selected_subject.set(e.value()); evidence_page.set(0); },
            option { value: "en", "English" } option { value: "hi", "Hindi" } option { value: "math", "Mathematics" }
        } }
        label { "Class " select { aria_label: "Evidence class", value: "{selected_grade()}", onchange: move |e| { selected_grade.set(e.value().parse().unwrap_or(2)); evidence_page.set(0); },
            option { value: "1", "Class 1" } option { value: "2", "Class 2" }
        } }
        h3 { "Recommended next targets" }
        for t in recommendation.tasks.iter().take(6) { p { "{t.label}: {t.reason.label()} · about {t.minutes} min" } }
        h3 { "Competencies and retention" }
        nav { class: "actions", aria_label: "Competency pages",
            button { disabled:current_page==0,onclick:move |_|evidence_page.set(current_page-1), "Previous competencies" }
            span { "Page {current_page+1} / {pages}" }
            button { disabled:current_page+1>=pages,onclick:move |_|evidence_page.set(current_page+1), "Next competencies" }
        }
        for (id,mapping) in competencies.into_iter().skip(current_page*40).take(40) {
            {
                let summary=crate::mastery::estimate(&snapshot.evidence,&id);
                let check=snapshot.evidence.reviews.get(&id);
                let due=check.map(|c| if c.due<=state::day() { "Overdue check".to_string() } else { format!("Review in {} days",c.due-state::day()) }).unwrap_or_else(||"No review evidence".into());
                let mut history:Vec<_>=snapshot.evidence.attempts.values().filter(|a|a.competency_id==id&&a.review).collect();
                history.sort_by_key(|a|(a.day,a.timestamp,&a.submission_id));
                rsx! { details { summary { "{mapping.label} · {summary.level.label()}" }
                    p { "{due}" }
                    p { "Independent scored: {summary.independent} · Assisted: {summary.assisted} · Unscored: {summary.unscored}" }
                    p { "Review successes: {summary.review_successes} / {summary.reviews}" }
                    for (mode,count) in summary.modalities { p { "{mode}: {count}" } }
                    for (help,count) in summary.assistance { p { "{help}: {count}" } }
                    for attempt in history.iter().rev().take(10) {
                        { let result=match attempt.correct {Some(true)=>"Correct",Some(false)=>"Needs practice",None=>"Unscored"};
                          let date=i32::try_from(attempt.day).ok().and_then(|d|d.checked_add(2_440_588)).and_then(|d|time::Date::from_julian_day(d).ok()).map(|d|d.to_string()).unwrap_or_else(||"Earlier review".into());
                          rsx! { p { "{date} · {result} · {attempt.context.assistance.label()} · {attempt.mode.label()}" } }
                        }
                    }
                } }
            }
        }
        p { role: "alert", "{error}" }
    } }
}
