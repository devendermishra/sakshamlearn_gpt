use crate::language::Language;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub palette: Palette,
    pub background: Background,
}

pub enum AppearanceChange {
    Palette(Palette),
    Background(Background),
}

impl Appearance {
    pub fn apply(&mut self, change: AppearanceChange) {
        match change {
            AppearanceChange::Palette(palette) => self.palette = palette,
            AppearanceChange::Background(background) => self.background = background,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Palette {
    Sky,
    Marigold,
    Plum,
    #[default]
    #[serde(other)]
    Forest,
}

impl Palette {
    const ALL: [Self; 4] = [Self::Forest, Self::Sky, Self::Marigold, Self::Plum];

    pub fn id(self) -> &'static str {
        match self {
            Self::Forest => "forest",
            Self::Sky => "sky",
            Self::Marigold => "marigold",
            Self::Plum => "plum",
        }
    }

    fn label(self, language: Language) -> &'static str {
        match self {
            Self::Forest => language.text("जंगल", "Forest"),
            Self::Sky => language.text("आसमानी", "Sky"),
            Self::Marigold => language.text("गेंदा", "Marigold"),
            Self::Plum => language.text("जामुनी", "Plum"),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Background {
    Playground,
    Animals,
    Plain,
    #[default]
    #[serde(other)]
    Storybook,
}

impl Background {
    const ALL: [Self; 4] = [
        Self::Storybook,
        Self::Playground,
        Self::Animals,
        Self::Plain,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Storybook => "storybook",
            Self::Playground => "playground",
            Self::Animals => "animals",
            Self::Plain => "plain",
        }
    }

    fn label(self, language: Language) -> &'static str {
        match self {
            Self::Storybook => language.text("कहानी की किताब", "Storybook"),
            Self::Playground => language.text("खेल का मैदान", "Playground"),
            Self::Animals => language.text("जानवरों की दुनिया", "Animal friends"),
            Self::Plain => language.text("सादा", "Plain"),
        }
    }

    fn art(self) -> Option<(&'static str, &'static [u8])> {
        match self {
            Self::Storybook => Some((
                "webp",
                include_bytes!("../assets/reading-under-mango-tree.webp"),
            )),
            Self::Playground => Some(("jpeg", include_bytes!("content/english/pictures/play.jpg"))),
            Self::Animals => Some((
                "jpeg",
                include_bytes!("content/english/pictures/animals.jpg"),
            )),
            Self::Plain => None,
        }
    }
}

pub fn background_styles() -> &'static str {
    static STYLES: LazyLock<String> = LazyLock::new(|| {
        Background::ALL
            .iter()
            .filter_map(|background| {
                let (mime, bytes) = background.art()?;
                Some(format!(
                    "[data-cover='{}'] {{ --cover-art: url(data:image/{mime};base64,{}); }}\n",
                    background.id(),
                    STANDARD.encode(bytes)
                ))
            })
            .collect()
    });
    &STYLES
}

#[component]
pub fn AppearanceButton(language: Language) -> Element {
    rsx! {
        button { class: "appearance-toggle", aria_haspopup: "dialog",
            aria_controls: "appearance-options",
            onclick: move |_| {
                let _ = document::eval("document.getElementById('appearance-options').showModal();");
            },
            span { class: "appearance-icon", aria_hidden: "true" }
            {language.text("रंग और चित्र", "Colors & backgrounds")}
        }
    }
}

#[component]
pub fn AppearanceOptions(
    appearance: Appearance,
    language: Language,
    on_change: EventHandler<AppearanceChange>,
) -> Element {
    rsx! {
        dialog { id: "appearance-options", aria_labelledby: "appearance-title",
            h2 { id: "appearance-title", {language.text("रंग और चित्र", "Colors & backgrounds")} }
            fieldset {
                legend { {language.text("रंग", "Color")} }
                div { class: "appearance-choices",
                    for palette in Palette::ALL {
                        label { class: "appearance-choice", key: "{palette.id()}",
                            input { r#type: "radio", name: "appearance-palette",
                                checked: appearance.palette == palette,
                                onchange: move |_| on_change.call(AppearanceChange::Palette(palette))
                            }
                            span { class: "palette-swatch", "data-palette": palette.id(), aria_hidden: "true" }
                            {palette.label(language)}
                        }
                    }
                }
            }
            fieldset {
                legend { {language.text("पीछे का चित्र", "Background")} }
                div { class: "appearance-choices",
                    for background in Background::ALL {
                        label { class: "appearance-choice appearance-cover", key: "{background.id()}",
                            input { r#type: "radio", name: "appearance-background",
                                checked: appearance.background == background,
                                onchange: move |_| on_change.call(AppearanceChange::Background(background))
                            }
                            span { class: "cover-swatch", "data-cover": background.id(), aria_hidden: "true" }
                            {background.label(language)}
                        }
                    }
                }
            }
            form { method: "dialog",
                button { class: "primary", {language.text("हो गया", "Done")} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;

    #[test]
    fn appearance_survives_save_and_load_and_defaults_for_old_saves() {
        let mut state: State = serde_json::from_str("{}").unwrap();
        assert!(state.appearance == Appearance::default());
        state.appearance = Appearance {
            palette: Palette::Plum,
            background: Background::Animals,
        };
        let saved = serde_json::to_string(&state).unwrap();
        let reopened: State = serde_json::from_str(&saved).unwrap();
        assert!(reopened.appearance == state.appearance);
    }
}
