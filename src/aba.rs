mod home_plans;
mod practice;

use crate::{
    language::Language,
    state::{india_date, State},
};
use dioxus::prelude::*;
use time::{Date, Duration};

#[component]
pub fn AbaLearning(
    language: Language,
    name: &'static str,
    data: Signal<State>,
    commit: Callback<State>,
    directory: std::path::PathBuf,
) -> Element {
    let mut tab = use_signal(|| 0u8);
    let mut selected = use_signal(|| practice::last_activity(&directory));
    let mut today = use_signal(india_date);
    use_future(move || async move {
        let mut clock = document::eval(include_str!("day_clock.js"));
        while clock.recv::<bool>().await.is_ok() {
            let date = india_date();
            if *today.peek() != date {
                today.set(date);
            }
        }
    });
    let date = today();

    rsx! {
        section { "data-aba-learning": "true", lang: language.code(),
            h1 { "{name}" }
            nav { class: "actions", aria_label: name,
                button { class: if tab() == 0 { "primary" } else { "" },
                    aria_pressed: tab() == 0, onclick: move |_| tab.set(0), {language.text("कैलेंडर", "Calendar")} }
                button { class: if tab() == 1 { "primary" } else { "" },
                    aria_pressed: tab() == 1, onclick: move |_| tab.set(1), {checklist_title(language)} }
                button { class: if tab() == 2 { "primary" } else { "" }, aria_pressed: tab() == 2, onclick: move |_| tab.set(2), {language.text("चित्र कार्ड", "Flashcards")} }
                button { class: if tab() == 3 { "primary" } else { "" }, aria_pressed: tab() == 3, onclick: move |_| tab.set(3), {language.text("अभ्यास", "Exercises")} }
            }
            if tab() == 1 {
                Checklist { date, language, data, commit, on_practice: move |id| { selected.set(id); tab.set(3); } }
            } else if tab() == 2 || tab() == 3 {
                practice::Practice { directory, language, selected: selected(), flashcards: tab() == 2, on_select: move |id| selected.set(id) }
            } else {
                Calendar { key: "{date}", today: date, language }
            }
        }
    }
}

fn checklist_title(language: Language) -> &'static str {
    language.text("घर के अभ्यास की सूची", "Home plan checklist")
}

fn month_name(date: Date, language: Language) -> String {
    match language {
        Language::English => date.month().to_string(),
        Language::Hindi => [
            "जनवरी",
            "फ़रवरी",
            "मार्च",
            "अप्रैल",
            "मई",
            "जून",
            "जुलाई",
            "अगस्त",
            "सितंबर",
            "अक्टूबर",
            "नवंबर",
            "दिसंबर",
        ][usize::from(u8::from(date.month()) - 1)]
        .to_owned(),
    }
}

fn full_date(date: Date, language: Language) -> String {
    format!(
        "{} {} {}",
        date.day(),
        month_name(date, language),
        date.year()
    )
}

fn month_bounds(date: Date) -> (Date, Date) {
    let first = date.replace_day(1).expect("every month has a first day");
    let last = first
        .replace_day(date.month().length(date.year()))
        .expect("valid last day of month");
    (first, last)
}

fn calendar_bounds(today: Date) -> (Date, Date) {
    let (first, last) = month_bounds(today);
    let first = first.min(today - Duration::DAY);
    let last = last.max(today + Duration::DAY);
    let start = first - Duration::days(first.weekday().number_days_from_monday().into());
    let end = last + Duration::days(6 - i64::from(last.weekday().number_days_from_monday()));
    (start, end)
}

#[component]
fn Calendar(today: Date, language: Language) -> Element {
    let yesterday = today - Duration::DAY;
    let tomorrow = today + Duration::DAY;
    let (start, end) = calendar_bounds(today);
    let weeks = (end - start).whole_days() / 7 + 1;
    let month = month_name(today, language);
    let weekdays = match language {
        Language::Hindi => [
            "सोमवार",
            "मंगलवार",
            "बुधवार",
            "गुरुवार",
            "शुक्रवार",
            "शनिवार",
            "रविवार",
        ],
        Language::English => [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ],
    };
    let column_labels = match language {
        Language::Hindi => ["सोम", "मंगल", "बुध", "गुरु", "शुक्र", "शनि", "रवि"],
        Language::English => ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    };
    let days = [
        (
            language.text("बीता कल", "Yesterday"),
            language.text("कल कौन-सा दिन था?", "What day was it yesterday?"),
            language.text("कल कितनी तारीख थी?", "What was yesterday’s date?"),
            yesterday,
        ),
        (
            language.text("आज", "Today"),
            language.text("आज कौन-सा दिन है?", "What day is it today?"),
            language.text("आज कितनी तारीख है?", "What is today’s date?"),
            today,
        ),
        (
            language.text("आने वाला कल", "Tomorrow"),
            language.text("कल कौन-सा दिन होगा?", "What day will it be tomorrow?"),
            language.text("कल कितनी तारीख होगी?", "What will tomorrow’s date be?"),
            tomorrow,
        ),
    ];

    rsx! {
        section { aria_label: language.text("कैलेंडर", "Calendar"),
            table { class: "aba-month",
                caption { "{month} {today.year()}" }
                thead {
                    tr {
                        for day in column_labels {
                            th { scope: "col", "{day}" }
                        }
                    }
                }
                tbody {
                    for week in 0..weeks {
                        tr { key: "{week}",
                            for offset in 0..7 {
                                {
                                    let date = start + Duration::days(week * 7 + offset);
                                    let relation = days.iter().find(|(_, _, _, value)| *value == date);
                                    let class = match relation {
                                        Some(_) if date == today => "aba-today",
                                        Some(_) => "aba-nearby",
                                        None if date.month() != today.month() => "aba-outside",
                                        None => "",
                                    };
                                    rsx! {
                                        td { key: "{date}", class,
                                            span { "{date.day()}" }
                                            if let Some((label, _, _, _)) = relation {
                                                small { "{label}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            h2 { {language.text("कैलेंडर से अभ्यास करें", "Practise with the calendar")} }
            p { {language.text("पहले अपना जवाब बोलें, फिर सवाल पर टैप करके जवाब देखें।", "Say your answer, then tap the question to check.")} }
            div { class: "aba-prompts",
                for (label, weekday_question, date_question, date) in days {
                    section { class: "aba-panel", key: "{date}",
                        h3 { "{label}" }
                        details {
                            summary { "{weekday_question}" }
                            p { {weekdays[usize::from(date.weekday().number_days_from_monday())]} }
                        }
                        details {
                            summary { "{date_question}" }
                            p { {full_date(date, language)} }
                        }
                        if date == today {
                            details {
                                summary { {language.text("अभी कौन-सा महीना है?", "What month is it today?")} }
                                p { "{month}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Checklist(
    date: Date,
    language: Language,
    data: Signal<State>,
    commit: Callback<State>,
    on_practice: EventHandler<String>,
) -> Element {
    let snapshot = data.read();
    let completed = snapshot.aba_practice.get(&date.to_string());
    let total: usize = home_plans::PLANS
        .iter()
        .map(|plan| plan.activities.len())
        .sum();
    let count = completed.map_or(0, |items| items.len());
    let progress = match language {
        Language::Hindi => format!("{total} में से {count} अभ्यास पूरे"),
        Language::English => format!("{count} of {total} practised"),
    };
    let heading = checklist_title(language);

    rsx! {
        section { aria_label: heading,
            h2 { "{heading}" }
            p { aria_live: "polite", "{full_date(date, language)} · {progress}" }
            p { {language.text("साथ में अभ्यास करने के बाद उस गतिविधि पर सही का निशान लगाएँ। हर दिन नई सूची शुरू होती है।", "Tick each activity after you practise it together. The checklist starts fresh each day.")} }
            for plan in home_plans::PLANS {
                section { class: "aba-plan", key: "{plan.title}",
                    h3 { {language.text(plan.hindi_title, plan.title)} }
                    ul { class: "aba-checklist",
                        for item in plan.activities {
                            li { key: "{item.id}",
                                label {
                                    input { r#type: "checkbox", checked: completed.is_some_and(|items| items.contains(item.id)),
                                        onchange: move |event| {
                                            let mut next = data.peek().clone();
                                            next.mark_aba_practice(date, item.id, event.checked());
                                            commit.call(next);
                                        },
                                    }
                                    span {
                                        strong { {language.text(item.hindi_title, item.title)} }
                                        small { {language.text(item.hindi_prompt, item.prompt)} }
                                    }
                                }
                                button { onclick: move |_| on_practice.call(item.id.to_string()), {language.text("अभ्यास खोलें", "Open practice")} }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    #[test]
    fn calendar_rolls_across_leap_days_and_years() {
        let march = Date::from_calendar_date(2024, Month::March, 1).unwrap();
        let february = march - Duration::DAY;
        assert_eq!(full_date(february, Language::English), "29 February 2024");
        assert_eq!(full_date(february, Language::Hindi), "29 फ़रवरी 2024");
        assert_eq!(month_bounds(february).1, february);
        let new_year = Date::from_calendar_date(2027, Month::January, 1).unwrap();
        assert_eq!(
            full_date(new_year - Duration::DAY, Language::English),
            "31 December 2026"
        );
        assert_eq!(
            full_date(new_year + Duration::DAY, Language::English),
            "2 January 2027"
        );
        let sunday = Date::from_calendar_date(2026, Month::May, 31).unwrap();
        let monday = sunday + Duration::DAY;
        assert!(calendar_bounds(sunday).1 >= monday);
        assert!(calendar_bounds(monday).0 <= sunday);
    }

    #[test]
    fn checklist_is_compatible_with_older_saves_and_keeps_dates_separate() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("progress.json");
        std::fs::write(&path, "{}").unwrap();
        let mut state = crate::state::load(&path).unwrap();
        let today = Date::from_calendar_date(2026, time::Month::November, 30).unwrap();
        let tomorrow = today + Duration::DAY;
        state.mark_aba_practice(today, "fruit", true);
        state.mark_aba_practice(tomorrow, "fruit", true);
        state.mark_aba_practice(tomorrow, "fruit", false);
        crate::state::save(&path, &state).unwrap();
        let state = crate::state::load(&path).unwrap();
        assert!(state.aba_practice[&today.to_string()].contains("fruit"));
        assert!(!state.aba_practice.contains_key(&tomorrow.to_string()));
    }
}
