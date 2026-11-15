use crate::language::Language;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Keys {
    #[default]
    Whole,
    List,
    Decimal,
    Pin,
}

#[component]
pub fn NumericField(
    label: String,
    value: String,
    language: Language,
    limit: usize,
    #[props(default)] keys: Keys,
    #[props(default)] disabled: bool,
    on_change: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let keypad_label = language.text("अंकों का कीपैड", "Numeric keypad");
    let separator = match keys {
        Keys::List => Some((",", language.text("अल्पविराम", "Comma"))),
        Keys::Decimal => Some((".", language.text("दशमलव बिंदु", "Decimal point"))),
        _ => None,
    };
    let input_value = value.clone();
    let edit = use_callback(move |key: &'static str| {
        let mut next = value.clone();
        match key {
            "clear" => next.clear(),
            "backspace" => { next.pop(); }
            "." if next.contains('.') => return,
            _ if next.chars().count() < limit => next.push_str(key),
            _ => return,
        }
        on_change.call(next);
    });
    rsx! {
        div { class: "numeric-field",
            label { "{label}"
                input { r#type: if keys == Keys::Pin { "password" } else { "text" },
                    inputmode: "none", maxlength: limit as i64, autocomplete: "off",
                    value: "{input_value}", disabled,
                    onfocus: move |_| open.set(true),
                    oninput: move |event| on_change.call(event.value()),
                }
            }
            if !disabled && open() {
                div { class: "keypad", role: "group", aria_label: keypad_label,
                    for digit in ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] {
                        button { key: "{digit}", r#type: "button", onclick: move |_| edit.call(digit), "{digit}" }
                    }
                    if let Some((symbol, name)) = separator {
                        button { r#type: "button", aria_label: name, onclick: move |_| edit.call(symbol), "{symbol}" }
                    }
                    button { r#type: "button", aria_label: language.text("एक अंक मिटाओ", "Backspace"),
                        onclick: move |_| edit.call("backspace"), "⌫" }
                    button { r#type: "button", onclick: move |_| edit.call("clear"),
                        {language.text("मिटाओ", "Clear")} }
                    button { r#type: "button", onclick: move |_| open.set(false),
                        {language.text("कीपैड बंद करो", "Hide keypad")} }
                }
            } else if !disabled {
                button { class: "quiet", r#type: "button", onclick: move |_| open.set(true),
                    {language.text("कीपैड दिखाओ", "Show keypad")} }
            }
        }
    }
}
