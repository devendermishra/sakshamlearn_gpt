use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Hindi,
    English,
}

impl Language {
    pub fn text<'a>(self, hindi: &'a str, english: &'a str) -> &'a str {
        match self {
            Self::Hindi => hindi,
            Self::English => english,
        }
    }

    pub fn code(self) -> &'static str {
        self.text("hi", "en")
    }
}
