use crate::{
    content::{ArithmeticHelp, Operation, Paper},
    language::Language,
    state::Ink,
};
use dioxus::prelude::*;
use serde::Deserialize;

pub const CALIBRATION_MM: f64 = 80.0;
const CSS_PIXELS_PER_MM: f64 = 96.0 / 25.4;
const DIGIT_CELL_MM: f64 = 11.0;
const WORK_GRID_COLUMNS: usize = 10;
const HINDI_BAND_MM: f64 = 8.0;

#[derive(Clone, Copy, PartialEq)]
pub struct PaperScale {
    pub horizontal: f64,
    pub vertical: f64,
}

impl PaperScale {
    pub fn from_saved(horizontal: Option<f64>, vertical: Option<f64>) -> Self {
        let horizontal = horizontal
            .filter(|value| valid_scale(*value))
            .unwrap_or(CSS_PIXELS_PER_MM);
        let vertical = vertical
            .filter(|value| valid_scale(*value))
            .unwrap_or(horizontal);
        Self {
            horizontal,
            vertical,
        }
    }

    pub fn corrected(self, measured_width: f64, measured_height: f64) -> Option<Self> {
        let horizontal = self.horizontal * CALIBRATION_MM / measured_width;
        let vertical = self.vertical * CALIBRATION_MM / measured_height;
        (valid_scale(horizontal) && valid_scale(vertical)).then_some(Self {
            horizontal,
            vertical,
        })
    }
}

fn valid_scale(value: f64) -> bool {
    (1.0..=10.0).contains(&value)
}

#[derive(Deserialize)]
struct InkEvent {
    ink: Ink,
    done: bool,
}

#[component]
pub fn InkPad(
    id: String,
    model: String,
    initial: Ink,
    scale: PaperScale,
    #[props(default = 1)] lines: usize,
    #[props(default)] work_columns: Option<usize>,
    #[props(default)] readonly: bool,
    #[props(default)] paper: Paper,
    #[props(default)] notebook: Option<ArithmeticHelp>,
    #[props(default)] language: Language,
    #[props(default)] done_label: Option<&'static str>,
    #[props(default)] on_ink: Option<EventHandler<Ink>>,
    #[props(default)] on_done: Option<EventHandler<Ink>>,
) -> Element {
    let digit_width = match (paper, model.as_bytes()) {
        (Paper::Digit, [b'0'..=b'9', b'0'..=b'9']) => 2,
        (Paper::Digit, [b'0'..=b'9', b'0'..=b'9', b'0'..=b'9']) => 3,
        _ => 0,
    };
    let tracing = !model.is_empty();
    let notes = paper.regrouping_cells();
    let blank_columns = match paper {
        Paper::Digit => work_columns.unwrap_or(WORK_GRID_COLUMNS),
        Paper::Notebook | Paper::NotebookCarry | Paper::NotebookBorrow => 2,
        Paper::Hindi => 1,
    };
    let config = serde_json::json!({ "id": id, "model": model, "initial": initial, "readonly": readonly, "square": paper != Paper::Hindi, "digitWidth": digit_width, "blankColumns": blank_columns, "noteColumns": notes, "lines": lines, "allowMouse": !cfg!(target_os = "android"), "english": language == Language::English });
    use_effect(move || {
        let script = format!(
            "const config = {config};\n{}\n{}\nwhile (container && !readonly && !submitted && container.isConnected) await new Promise(resolve => setTimeout(resolve, 250));",
            include_str!("trace.js"), include_str!("ink.js")
        );
        let mut eval = document::eval(&script);
        if readonly {
            return;
        }
        spawn(async move {
            while let Ok(event) = eval.recv::<InkEvent>().await {
                if event.done {
                    if let Some(done) = on_done {
                        done.call(event.ink);
                    }
                    break;
                }
                if let Some(save) = on_ink {
                    save.call(event.ink);
                }
            }
        });
    });
    let (cell, height) = match paper {
        Paper::Hindi if model.is_empty() => (96.0, 3.0 * HINDI_BAND_MM),
        Paper::Hindi => (30.0, 3.0 * HINDI_BAND_MM),
        _ => (DIGIT_CELL_MM, DIGIT_CELL_MM),
    };
    let columns = match (paper, tracing) {
        (Paper::Digit, true) if digit_width > 0 => 2 * digit_width,
        (Paper::Digit, true) => 10,
        (Paper::Hindi, true) => 6,
        _ => blank_columns,
    };
    let class = match (paper, tracing) {
        (Paper::Hindi, true) => "ink ink-hindi ink-hindi-trace",
        (Paper::Hindi, false) => "ink ink-hindi",
        _ => "ink ink-square",
    };
    let paper_style = format!(
        "--cell:{}px;--height:{}px;--columns:{columns};--notes:{notes}",
        cell * scale.horizontal,
        height * scale.vertical
    );
    let model_label = language.text("नमूना", "Example");
    let full_stop = language.text("।", ".");
    let two_rows = language.text(
        "ऊपर बिंदियों पर लिखो, नीचे खुद लिखो।",
        "Trace the dotted digits above, then write in the blank squares below.",
    );
    let done = done_label.unwrap_or_else(|| language.text("हो गया", "Done"));
    let can_finish = on_done.is_some();
    rsx! {
        section { id: "{id}", class: "{class}", style: "{paper_style}",
            if tracing && paper == Paper::Hindi {
                p { class: "instructions", {language.text("पहली पंक्ति में बिंदियों पर लिखो। दूसरी पंक्ति में खुद लिखो।", "Trace the dots on the first row. Write on your own on the second row.")} }
            } else if digit_width > 0 {
                p { class: "instructions", "{model_label} {model}{full_stop} {two_rows}" }
            } else if tracing && paper == Paper::Digit {
                p { class: "instructions", {language.text("पहले पाँच बिंदियों वाले खाने, फिर पाँच खाली खाने", "Trace the first five squares, then write in the five blank squares.")} }
            }
            div { class: if notebook.is_some() { "ink-scroll notebook-stack" } else { "ink-scroll" },
                if notes > 0 { div { class: "regroup-grid" } }
                if let Some(problem) = notebook {
                    {
                        let sign = match problem.operation { Operation::Add => "+", Operation::Subtract => "−" };
                        rsx! {
                            for (value, operator) in [(problem.left, ""), (problem.right, sign)] {
                                div { class: "notebook-row", aria_hidden: "true",
                                    span { "{operator}" }
                                    span { if value >= 10 { "{value / 10}" } }
                                    span { "{value % 10}" }
                                }
                            }
                        }
                    }
                    div { class: "notebook-rule" }
                }
                div { class: "ink-grid" }
                if notebook.is_some() { div { class: "notebook-rule" } }
            }
            if !readonly {
                div { class: "actions",
                    button { "data-undo": "", {language.text("एक रेखा हटाओ", "Undo last stroke")} }
                    button { "data-clear": "", {language.text("फिर से लिखो", "Clear writing")} }
                    if can_finish { button { class: "primary", "data-done": "", "{done}" } }
                }
                p { role: "status", class: "ink-status" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjusts_both_axes_from_physical_measurements() {
        let initial = PaperScale::from_saved(Some(4.0), None);
        let corrected = initial.corrected(100.0, 64.0).unwrap();
        assert_eq!(corrected.horizontal, 3.2);
        assert_eq!(corrected.vertical, 5.0);
        assert!(initial.corrected(0.0, 80.0).is_none());
        assert!(initial.corrected(80.0, f64::NAN).is_none());
        assert!(initial.corrected(-80.0, 80.0).is_none());
    }

    #[test]
    fn keeps_legacy_calibration_and_defaults_invalid_values() {
        let legacy = PaperScale::from_saved(Some(4.2), None);
        assert_eq!(legacy.horizontal, 4.2);
        assert_eq!(legacy.vertical, 4.2);
        let default = PaperScale::from_saved(Some(f64::NAN), Some(0.0));
        assert_eq!(default.horizontal, CSS_PIXELS_PER_MM);
        assert_eq!(default.vertical, CSS_PIXELS_PER_MM);
    }
}
