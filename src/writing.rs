use crate::{content::Paper, state::Ink};
use dioxus::prelude::*;
use serde::Deserialize;

pub const CALIBRATION_MM: f64 = 80.0;
const CSS_PIXELS_PER_MM: f64 = 96.0 / 25.4;
const DIGIT_CELL_MM: f64 = 11.0;
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
    #[props(default)] readonly: bool,
    #[props(default)] paper: Paper,
    #[props(default)] on_ink: Option<EventHandler<Ink>>,
    #[props(default)] on_done: Option<EventHandler<Ink>>,
) -> Element {
    let config = serde_json::json!({ "id": id, "model": model, "initial": initial, "readonly": readonly, "square": paper == Paper::Digit, "lines": lines, "allowMouse": !cfg!(target_os = "android") });
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
        Paper::Digit => (DIGIT_CELL_MM, DIGIT_CELL_MM),
        Paper::Hindi if model.is_empty() => (96.0, 3.0 * HINDI_BAND_MM),
        Paper::Hindi => (30.0, 3.0 * HINDI_BAND_MM),
    };
    let tracing = !model.is_empty();
    let columns = match (paper, tracing) {
        (Paper::Digit, true) => 10,
        (Paper::Hindi, true) => 6,
        (_, false) => 1,
    };
    let class = match (paper, tracing) {
        (Paper::Digit, _) => "ink ink-square",
        (Paper::Hindi, true) => "ink ink-hindi ink-hindi-trace",
        (Paper::Hindi, false) => "ink ink-hindi",
    };
    let paper_style = format!(
        "--cell:{}px;--height:{}px;--columns:{columns}",
        cell * scale.horizontal,
        height * scale.vertical
    );
    rsx! {
        section { id: "{id}", class: "{class}", style: "{paper_style}",
            if tracing && paper == Paper::Hindi {
                p { class: "instructions", "पहली पंक्ति में बिंदियों पर लिखो। दूसरी पंक्ति में खुद लिखो।" }
            } else if tracing && paper == Paper::Digit {
                p { class: "instructions", "पहले पाँच बिंदियों वाले खाने, फिर पाँच खाली खाने" }
            }
            div { class: "ink-scroll", div { class: "ink-grid" } }
            if !readonly {
                div { class: "actions",
                    button { "data-undo": "", "एक रेखा हटाओ" }
                    button { "data-clear": "", "फिर से लिखो" }
                    button { class: "primary", "data-done": "", "हो गया" }
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
