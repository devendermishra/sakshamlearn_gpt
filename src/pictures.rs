use crate::language::Language;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use dioxus::prelude::*;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::{LazyLock, OnceLock}};
#[derive(Deserialize)]
pub struct Card {
    pub id: String,
    pub en: String,
    pub hi: String,
    #[serde(default)]
    pub category: String,
    pub source: String,
    #[serde(default)]
    pub sound: String,
    #[serde(default)]
    pub page: usize,
}
impl Card {
    pub fn label(&self, l: Language) -> &str {
        l.text(&self.hi, &self.en)
    }
}
pub static CARDS: LazyLock<Vec<Card>> = LazyLock::new(|| {
    let mut cards: Vec<Card> =
        serde_json::from_str(include_str!("../assets/pictures/catalog.json")).unwrap();
    cards.extend(
        serde_json::from_str::<Vec<Card>>(include_str!("../assets/practice/speech.json")).unwrap(),
    );
    cards
});
pub fn card(id: &str) -> Option<&'static Card> {
    CARDS.iter().find(|c| c.id == id)
}
static IMAGES: LazyLock<BTreeMap<&str, (&'static [u8], OnceLock<String>)>> = LazyLock::new(|| {
    let entries: &[(&str, &[u8])] = &[
        ("under", include_bytes!("../assets/pictures/under.jpg")),
        ("near", include_bytes!("../assets/pictures/near.jpg")),
        ("between", include_bytes!("../assets/pictures/between.jpg")),
        ("behind", include_bytes!("../assets/pictures/behind.jpg")),
        ("apple", include_bytes!("../assets/pictures/apple.jpg")),
        ("banana", include_bytes!("../assets/pictures/banana.jpg")),
        ("mango", include_bytes!("../assets/pictures/mango.jpg")),
        ("dog", include_bytes!("../assets/pictures/dog.jpg")),
        ("cat", include_bytes!("../assets/pictures/cat.jpg")),
        ("cow", include_bytes!("../assets/pictures/cow.jpg")),
        ("eating", include_bytes!("../assets/pictures/eating.jpg")),
        ("reading", include_bytes!("../assets/pictures/reading.jpg")),
        ("playing", include_bytes!("../assets/pictures/playing.jpg")),
        ("car", include_bytes!("../assets/pictures/car.jpg")),
        ("bus", include_bytes!("../assets/pictures/bus.jpg")),
        ("train", include_bytes!("../assets/pictures/train.jpg")),
        ("parrot", include_bytes!("../assets/pictures/parrot.jpg")),
        ("crow", include_bytes!("../assets/pictures/crow.jpg")),
        ("sparrow", include_bytes!("../assets/pictures/sparrow.jpg")),
        ("doctor", include_bytes!("../assets/pictures/doctor.jpg")),
        ("teacher", include_bytes!("../assets/pictures/teacher.jpg")),
        ("farmer", include_bytes!("../assets/pictures/farmer.jpg")),
        ("summer", include_bytes!("../assets/pictures/summer.jpg")),
        ("rain", include_bytes!("../assets/pictures/rain.jpg")),
        ("winter", include_bytes!("../assets/pictures/winter.jpg")),
        ("kitchen", include_bytes!("../assets/pictures/kitchen.jpg")),
        (
            "bathroom",
            include_bytes!("../assets/pictures/bathroom.jpg"),
        ),
        ("bank", include_bytes!("../assets/pictures/bank.jpg")),
        ("bin", include_bytes!("../assets/pictures/bin.jpg")),
        ("pencil", include_bytes!("../assets/pictures/pencil.jpg")),
        ("ball", include_bytes!("../assets/pictures/ball.jpg")),
        ("cup", include_bytes!("../assets/pictures/cup.jpg")),
        ("waiting", include_bytes!("../assets/pictures/waiting.jpg")),
        (
            "greeting",
            include_bytes!("../assets/pictures/greeting.jpg"),
        ),
        (
            "delivery",
            include_bytes!("../assets/pictures/delivery.jpg"),
        ),
        ("request", include_bytes!("../assets/pictures/request.jpg")),
        ("family", include_bytes!("../assets/pictures/family.jpg")),
        ("book", include_bytes!("../assets/pictures/book.jpg")),
        ("bag", include_bytes!("../assets/pictures/bag.jpg")),
        ("water", include_bytes!("../assets/pictures/water.jpg")),
        ("milk", include_bytes!("../assets/pictures/milk.jpg")),
        ("rice", include_bytes!("../assets/pictures/rice.jpg")),
        (
            "speech-baby",
            include_bytes!("../assets/practice/speech-baby.jpg"),
        ),
        (
            "speech-dad",
            include_bytes!("../assets/practice/speech-dad.jpg"),
        ),
        (
            "speech-mom",
            include_bytes!("../assets/practice/speech-mom.jpg"),
        ),
        (
            "speech-panda",
            include_bytes!("../assets/practice/speech-panda.jpg"),
        ),
        (
            "speech-lamp",
            include_bytes!("../assets/practice/speech-lamp.jpg"),
        ),
        (
            "speech-fan",
            include_bytes!("../assets/practice/speech-fan.jpg"),
        ),
        (
            "speech-apple",
            include_bytes!("../assets/practice/speech-apple.jpg"),
        ),
        (
            "speech-cat",
            include_bytes!("../assets/practice/speech-cat.jpg"),
        ),
        (
            "speech-ball",
            include_bytes!("../assets/practice/speech-ball.jpg"),
        ),
        (
            "speech-doll",
            include_bytes!("../assets/practice/speech-doll.jpg"),
        ),
        (
            "speech-blocks",
            include_bytes!("../assets/practice/speech-blocks.jpg"),
        ),
        (
            "speech-train",
            include_bytes!("../assets/practice/speech-train.jpg"),
        ),
        (
            "speech-hat",
            include_bytes!("../assets/practice/speech-hat.jpg"),
        ),
        (
            "speech-shoes",
            include_bytes!("../assets/practice/speech-shoes.jpg"),
        ),
        (
            "speech-socks",
            include_bytes!("../assets/practice/speech-socks.jpg"),
        ),
        (
            "speech-shirt",
            include_bytes!("../assets/practice/speech-shirt.jpg"),
        ),
        (
            "speech-chair",
            include_bytes!("../assets/practice/speech-chair.jpg"),
        ),
        (
            "speech-table",
            include_bytes!("../assets/practice/speech-table.jpg"),
        ),
        (
            "speech-bed",
            include_bytes!("../assets/practice/speech-bed.jpg"),
        ),
        (
            "speech-book",
            include_bytes!("../assets/practice/speech-book.jpg"),
        ),
        (
            "speech-happy",
            include_bytes!("../assets/practice/speech-happy.jpg"),
        ),
        (
            "speech-sad",
            include_bytes!("../assets/practice/speech-sad.jpg"),
        ),
        (
            "speech-hungry",
            include_bytes!("../assets/practice/speech-hungry.jpg"),
        ),
        (
            "speech-tired",
            include_bytes!("../assets/practice/speech-tired.jpg"),
        ),
    ];
    entries
        .iter()
        .map(|(id, bytes)| (*id, (*bytes, OnceLock::new())))
        .collect()
});
pub fn source(id: &str) -> &'static str {
    IMAGES.get(id).map_or("", |(bytes, encoded)| {
        encoded.get_or_init(|| format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes))).as_str()
    })
}
static PRACTICE: LazyLock<BTreeMap<&str, (&'static [u8], OnceLock<String>)>> = LazyLock::new(|| {
    let entries: &[(&str, &[u8])] = &[
        (
            "pencil-1.jpg",
            include_bytes!("../assets/practice/pencil-1.jpg"),
        ),
        (
            "pencil-1-row-1.jpg",
            include_bytes!("../assets/practice/pencil-1-row-1.jpg"),
        ),
        (
            "pencil-1-row-2.jpg",
            include_bytes!("../assets/practice/pencil-1-row-2.jpg"),
        ),
        (
            "pencil-1-row-3.jpg",
            include_bytes!("../assets/practice/pencil-1-row-3.jpg"),
        ),
        (
            "pencil-1-row-4.jpg",
            include_bytes!("../assets/practice/pencil-1-row-4.jpg"),
        ),
        (
            "pencil-1-row-5.jpg",
            include_bytes!("../assets/practice/pencil-1-row-5.jpg"),
        ),
        (
            "pencil-1-row-6.jpg",
            include_bytes!("../assets/practice/pencil-1-row-6.jpg"),
        ),
        (
            "pencil-1-row-7.jpg",
            include_bytes!("../assets/practice/pencil-1-row-7.jpg"),
        ),
        (
            "pencil-2.jpg",
            include_bytes!("../assets/practice/pencil-2.jpg"),
        ),
        (
            "pencil-2-row-1.jpg",
            include_bytes!("../assets/practice/pencil-2-row-1.jpg"),
        ),
        (
            "pencil-2-row-2.jpg",
            include_bytes!("../assets/practice/pencil-2-row-2.jpg"),
        ),
        (
            "pencil-2-row-3.jpg",
            include_bytes!("../assets/practice/pencil-2-row-3.jpg"),
        ),
        (
            "pencil-2-row-4.jpg",
            include_bytes!("../assets/practice/pencil-2-row-4.jpg"),
        ),
        (
            "pencil-2-row-5.jpg",
            include_bytes!("../assets/practice/pencil-2-row-5.jpg"),
        ),
        (
            "pencil-2-row-6.jpg",
            include_bytes!("../assets/practice/pencil-2-row-6.jpg"),
        ),
        (
            "pencil-2-row-7.jpg",
            include_bytes!("../assets/practice/pencil-2-row-7.jpg"),
        ),
        (
            "pencil-3.jpg",
            include_bytes!("../assets/practice/pencil-3.jpg"),
        ),
        (
            "pencil-3-row-1.jpg",
            include_bytes!("../assets/practice/pencil-3-row-1.jpg"),
        ),
        (
            "pencil-3-row-2.jpg",
            include_bytes!("../assets/practice/pencil-3-row-2.jpg"),
        ),
        (
            "pencil-3-row-3.jpg",
            include_bytes!("../assets/practice/pencil-3-row-3.jpg"),
        ),
        (
            "pencil-3-row-4.jpg",
            include_bytes!("../assets/practice/pencil-3-row-4.jpg"),
        ),
        (
            "pencil-3-row-5.jpg",
            include_bytes!("../assets/practice/pencil-3-row-5.jpg"),
        ),
        (
            "pencil-3-row-6.jpg",
            include_bytes!("../assets/practice/pencil-3-row-6.jpg"),
        ),
        (
            "pencil-3-row-7.jpg",
            include_bytes!("../assets/practice/pencil-3-row-7.jpg"),
        ),
        (
            "pencil-3-row-8.jpg",
            include_bytes!("../assets/practice/pencil-3-row-8.jpg"),
        ),
        (
            "pencil-4.jpg",
            include_bytes!("../assets/practice/pencil-4.jpg"),
        ),
        (
            "pencil-4-row-1.jpg",
            include_bytes!("../assets/practice/pencil-4-row-1.jpg"),
        ),
        (
            "pencil-4-row-2.jpg",
            include_bytes!("../assets/practice/pencil-4-row-2.jpg"),
        ),
        (
            "pencil-4-row-3.jpg",
            include_bytes!("../assets/practice/pencil-4-row-3.jpg"),
        ),
        (
            "pencil-4-row-4.jpg",
            include_bytes!("../assets/practice/pencil-4-row-4.jpg"),
        ),
        (
            "pencil-4-row-5.jpg",
            include_bytes!("../assets/practice/pencil-4-row-5.jpg"),
        ),
        (
            "pencil-4-row-6.jpg",
            include_bytes!("../assets/practice/pencil-4-row-6.jpg"),
        ),
        (
            "pencil-4-row-7.jpg",
            include_bytes!("../assets/practice/pencil-4-row-7.jpg"),
        ),
        (
            "pencil-4-row-8.jpg",
            include_bytes!("../assets/practice/pencil-4-row-8.jpg"),
        ),
        (
            "pencil-5.jpg",
            include_bytes!("../assets/practice/pencil-5.jpg"),
        ),
        (
            "pencil-5-row-1.jpg",
            include_bytes!("../assets/practice/pencil-5-row-1.jpg"),
        ),
        (
            "pencil-5-row-2.jpg",
            include_bytes!("../assets/practice/pencil-5-row-2.jpg"),
        ),
        (
            "pencil-5-row-3.jpg",
            include_bytes!("../assets/practice/pencil-5-row-3.jpg"),
        ),
        (
            "pencil-5-row-4.jpg",
            include_bytes!("../assets/practice/pencil-5-row-4.jpg"),
        ),
        (
            "pencil-5-row-5.jpg",
            include_bytes!("../assets/practice/pencil-5-row-5.jpg"),
        ),
        (
            "pencil-5-row-6.jpg",
            include_bytes!("../assets/practice/pencil-5-row-6.jpg"),
        ),
        (
            "pencil-5-row-7.jpg",
            include_bytes!("../assets/practice/pencil-5-row-7.jpg"),
        ),
        (
            "pencil-5-row-8.jpg",
            include_bytes!("../assets/practice/pencil-5-row-8.jpg"),
        ),
        (
            "pencil-6.jpg",
            include_bytes!("../assets/practice/pencil-6.jpg"),
        ),
        (
            "pencil-6-row-1.jpg",
            include_bytes!("../assets/practice/pencil-6-row-1.jpg"),
        ),
        (
            "pencil-6-row-2.jpg",
            include_bytes!("../assets/practice/pencil-6-row-2.jpg"),
        ),
        (
            "pencil-6-row-3.jpg",
            include_bytes!("../assets/practice/pencil-6-row-3.jpg"),
        ),
        (
            "pencil-6-row-4.jpg",
            include_bytes!("../assets/practice/pencil-6-row-4.jpg"),
        ),
        (
            "pencil-6-row-5.jpg",
            include_bytes!("../assets/practice/pencil-6-row-5.jpg"),
        ),
        (
            "pencil-6-row-6.jpg",
            include_bytes!("../assets/practice/pencil-6-row-6.jpg"),
        ),
        (
            "pencil-6-row-7.jpg",
            include_bytes!("../assets/practice/pencil-6-row-7.jpg"),
        ),
        (
            "pencil-6-row-8.jpg",
            include_bytes!("../assets/practice/pencil-6-row-8.jpg"),
        ),
        (
            "pencil-7.jpg",
            include_bytes!("../assets/practice/pencil-7.jpg"),
        ),
        (
            "pencil-7-row-1.jpg",
            include_bytes!("../assets/practice/pencil-7-row-1.jpg"),
        ),
        (
            "pencil-7-row-2.jpg",
            include_bytes!("../assets/practice/pencil-7-row-2.jpg"),
        ),
        (
            "pencil-7-row-3.jpg",
            include_bytes!("../assets/practice/pencil-7-row-3.jpg"),
        ),
        (
            "pencil-7-row-4.jpg",
            include_bytes!("../assets/practice/pencil-7-row-4.jpg"),
        ),
        (
            "pencil-7-row-5.jpg",
            include_bytes!("../assets/practice/pencil-7-row-5.jpg"),
        ),
        (
            "pencil-7-row-6.jpg",
            include_bytes!("../assets/practice/pencil-7-row-6.jpg"),
        ),
        (
            "pencil-7-row-7.jpg",
            include_bytes!("../assets/practice/pencil-7-row-7.jpg"),
        ),
        (
            "pencil-7-row-8.jpg",
            include_bytes!("../assets/practice/pencil-7-row-8.jpg"),
        ),
        (
            "pencil-8.jpg",
            include_bytes!("../assets/practice/pencil-8.jpg"),
        ),
        (
            "pencil-8-row-1.jpg",
            include_bytes!("../assets/practice/pencil-8-row-1.jpg"),
        ),
        (
            "pencil-8-row-2.jpg",
            include_bytes!("../assets/practice/pencil-8-row-2.jpg"),
        ),
        (
            "pencil-8-row-3.jpg",
            include_bytes!("../assets/practice/pencil-8-row-3.jpg"),
        ),
        (
            "pencil-8-row-4.jpg",
            include_bytes!("../assets/practice/pencil-8-row-4.jpg"),
        ),
        (
            "pencil-8-row-5.jpg",
            include_bytes!("../assets/practice/pencil-8-row-5.jpg"),
        ),
        (
            "pencil-8-row-6.jpg",
            include_bytes!("../assets/practice/pencil-8-row-6.jpg"),
        ),
        (
            "pencil-8-row-7.jpg",
            include_bytes!("../assets/practice/pencil-8-row-7.jpg"),
        ),
        (
            "pencil-8-row-8.jpg",
            include_bytes!("../assets/practice/pencil-8-row-8.jpg"),
        ),
        (
            "pencil-9.jpg",
            include_bytes!("../assets/practice/pencil-9.jpg"),
        ),
        (
            "pencil-10.jpg",
            include_bytes!("../assets/practice/pencil-10.jpg"),
        ),
        (
            "pencil-11.jpg",
            include_bytes!("../assets/practice/pencil-11.jpg"),
        ),
        (
            "pencil-12.jpg",
            include_bytes!("../assets/practice/pencil-12.jpg"),
        ),
        (
            "pencil-13.jpg",
            include_bytes!("../assets/practice/pencil-13.jpg"),
        ),
        (
            "pencil-14.jpg",
            include_bytes!("../assets/practice/pencil-14.jpg"),
        ),
    ];
    entries
        .iter()
        .map(|(id, bytes)| (*id, (*bytes, OnceLock::new())))
        .collect()
});
pub fn practice(file: &str) -> String {
    PRACTICE.get(file).map(|(bytes, encoded)| {
        encoded.get_or_init(|| format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes))).clone()
    }).unwrap_or_default()
}
#[component]
pub fn Image(id: String, language: Language, #[props(default)] compact: bool) -> Element {
    let Some(c) = card(&id) else {
        return rsx! {};
    };
    rsx! { img { class:if compact {"practice-picture compact"} else {"practice-picture"},src:source(&id),alt:c.label(language),loading:"lazy" } }
}
// Explicit bilingual vocabulary mappings. Never infer pictures from partial prompt text.
pub fn word(word: &str) -> Option<&'static str> {
    match word.trim().to_lowercase().as_str() {
        "under" | "नीचे" => Some("under"),
        "near" | "पास" => Some("near"),
        "between" | "बीच" => Some("between"),
        "behind" | "पीछे" => Some("behind"),
        "apple" | "सेब" => Some("apple"),
        "banana" | "केला" => Some("banana"),
        "mango" | "आम" => Some("mango"),
        "dog" | "कुत्ता" => Some("dog"),
        "cat" | "बिल्ली" => Some("cat"),
        "cow" | "गाय" => Some("cow"),
        "eat" | "खाना" => Some("eating"),
        "drink" | "पानी" | "water" => Some("water"),
        "read" | "पढ़ना" => Some("reading"),
        "play" | "खेलना" => Some("playing"),
        "home" | "mother" | "father" | "माँ" | "पापा" | "घर" => Some("family"),
        "bag" | "बस्ता" => Some("bag"),
        "book" | "किताब" => Some("book"),
        "pencil" | "पेंसिल" => Some("pencil"),
        "rice" | "चावल" => Some("rice"),
        "milk" | "दूध" => Some("milk"),
        "hello" | "नमस्ते" => Some("greeting"),
        "chair" | "कुर्सी" => Some("speech-chair"),
        "table" | "मेज़" => Some("speech-table"),
        "shoes" | "जूते" => Some("speech-shoes"),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_complete_and_unique() {
        let mut ids = std::collections::BTreeSet::new();
        for c in CARDS.iter() {
            assert!(ids.insert(&c.id));
            assert!(!source(&c.id).is_empty());
            assert!(!c.hi.is_empty());
            assert!(!c.source.is_empty());
        }
        assert_eq!(
            CARDS.iter().filter(|c| c.id.starts_with("speech-")).count(),
            24
        );
    }
}

// Only concrete trace models and explicit practice prompts receive a shared picture.
// Recall questions are rendered without these contextual cues.
pub fn for_kind(kind: &crate::content::Kind) -> Option<&'static str> {
    use crate::content::Kind;
    match kind {
        Kind::Trace { model, .. } => word(model),
        Kind::Talk { prompt, .. } => match prompt.as_str() {
            "What is happening in the picture?" | "तस्वीर में क्या हो रहा है?" => {
                Some("playing")
            }
            _ => None,
        },
        Kind::Choice(q) => match q.prompt.as_str() {
            "There are 5 balls in each of 2 boxes. How many balls are there altogether?"
            | "2 डिब्बों में 5-5 गेंदें हैं। कुल कितनी गेंदें हैं?" => {
                Some("ball")
            }
            "There are 3 marbles in each of 3 bags. How many marbles are there altogether?"
            | "3 थैलियों में 3-3 कंचे हैं। कुल कितने कंचे हैं?" => {
                Some("bag")
            }
            _ => None,
        },
        Kind::Read(script, _) => match script.as_str() {
            "Count shoes in pairs: 2, 4, 6, 8. Add 2 each time."
            | "जूते जोड़ी में गिनो: 2, 4, 6, 8। हर बार 2 बढ़े।" => {
                Some("speech-shoes")
            }
            _ => None,
        },
        Kind::Solve { prompt, .. } => match prompt.as_str() {
            "There are 3 apples. Add 2 more. How many apples?" => Some("apple"),
            _ => None,
        },
        _ => None,
    }
}

pub fn groups(kind: &crate::content::Kind) -> Option<(usize, usize)> {
    if let crate::content::Kind::Read(script, _) = kind {
        match script.as_str(){
 "● ● | ● ● | ● ●. There are 3 equal groups with 2 dots in each group. 2 + 2 + 2 = 6."|"● ● | ● ● | ● ●। 3 बराबर समूह हैं। हर समूह में 2 बिंदु हैं। 2 + 2 + 2 = 6।"=>Some((3,2)),
 "● ● ● | ● ● ●. There are 2 groups with 3 dots each. 3 + 3 = 6. We can also write 3 × 2 = 6."|"● ● ● | ● ● ●। 2 समूह, हर समूह में 3। 3 + 3 = 6। इसे 3 × 2 = 6 भी लिखते हैं।"=>Some((2,3)),_=>None
 }
    } else {
        None
    }
}

#[cfg(test)]
mod compatibility_tests {
    #[test]
    fn older_exercises_and_optional_picture_round_trip() {
        let old = serde_json::json!({"id":"legacy","kind":crate::content::Kind::Trace{model:"apple".into(),guidance:"Trace".into(),paper:crate::content::Paper::English}});
        let mut e: crate::content::Exercise = serde_json::from_value(old.clone()).unwrap();
        assert!(e.picture.is_none());
        assert_eq!(serde_json::to_value(&e).unwrap(), old);
        e.picture = Some("apple".into());
        let r: crate::content::Exercise =
            serde_json::from_str(&serde_json::to_string(&e).unwrap()).unwrap();
        assert_eq!(r.picture.as_deref(), Some("apple"));
    }
    #[test]
    fn everyday_picture_mappings_cover_both_classes_and_languages() {
        for subject in crate::content::BUNDLED
            .iter()
            .filter(|s| matches!(s.id.as_str(), "en" | "hi"))
        {
            for grade in [1, 2] {
                assert!(subject
                    .topics
                    .iter()
                    .filter(|t| t.grade == grade)
                    .flat_map(|t| &t.exercises)
                    .any(|e| e.picture.is_some()));
            }
        }
    }
}
