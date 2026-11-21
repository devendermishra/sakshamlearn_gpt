use super::home_plans;
use crate::{
    language::Language,
    pictures::{self, Image},
    state,
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

pub(super) fn cards(id: &str) -> &'static [&'static str] {
    match id {
        "fruit" => &["apple", "banana", "mango"],
        "animals" => &["dog", "cat", "cow"],
        "actions" | "action-sentences" | "picture-description" => &["eating", "reading", "playing"],
        "boy-girl" => &["eating", "reading", "playing"],
        "transport-birds" => &["car", "bus", "train", "parrot", "crow", "sparrow"],
        "professionals" => &["doctor", "teacher", "farmer"],
        "seasonal-puzzle" => &["summer", "rain", "winter"],
        "request-action" => &["request", "ball", "cup"],
        "request-missing" => &["pencil", "book", "bag"],
        "pattern-box" => &["ball", "apple", "banana"],
        "give-item" => &["delivery", "ball", "book"],
        "greetings" | "social-question" => &["greeting", "family", "speech-happy"],
        "eye-contact" | "name-call" => &["family", "greeting", "waiting"],
        "waiting" => &["waiting", "ball", "request"],
        "what-questions" | "yes-no" | "object-description" => &["cup", "ball", "pencil"],
        "cheek-massage" => &["family"],
        "spirometer" => &["waiting"],
        "articulation-r" => &["rice", "parrot", "car"],
        "articulation-l" => &["speech-lamp", "milk", "ball"],
        "articulation-s" => &["summer", "pencil", "bus"],
        "where-cooking" => &["kitchen", "rice", "cup"],
        "where-bathing" => &["bathroom", "speech-tired", "water"],
        "where-money" => &["bank", "bag", "family"],
        "where-rubbish" => &["bin", "book", "pencil"],
        _ => &[],
    }
}
fn resolved(id: &str) -> &str {
    match id {
        "lamp" => "speech-lamp",
        "school" => "bag",
        _ => id,
    }
}
fn activity(id: &str) -> Option<&'static home_plans::Activity> {
    home_plans::PLANS
        .iter()
        .flat_map(|p| p.activities)
        .find(|a| a.id == id)
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Round {
    position: usize,
    boundary: bool,
    paused: bool,
    responses: BTreeMap<usize, String>,
    choices: BTreeMap<usize, usize>,
    helped: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Progress {
    selected: String,
    activities: BTreeMap<String, Round>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Identify,
    Match,
    Name,
    Sort,
    Pattern,
    YesNo,
    Adult,
}
fn mode(id: &str, position: usize) -> Mode {
    match id {
        "pattern-box" => Mode::Pattern,
        "yes-no" => Mode::YesNo,
        "transport-birds" if position % 3 == 2 => Mode::Sort,
        "fruit" | "animals" | "professionals" | "transport-birds" => match position % 3 {
            0 => Mode::Identify,
            1 => Mode::Match,
            _ => Mode::Name,
        },
        _ => Mode::Adult,
    }
}
fn cue<'a>(id: &str, l: Language, default: &'a str) -> String {
    let (hi, en) = match id {
        "actions" => ("तस्वीर में कौन-सा काम हो रहा है?", "What action is happening?"),
        "action-sentences" => (
            "तस्वीर के बारे में पूरा वाक्य बोलें।",
            "Say a full sentence about the picture.",
        ),
        "boy-girl" => (
            "बड़े तस्वीर के व्यक्ति का परिचय दें, फिर पहचानने का अभ्यास करें।",
            "An adult introduces the person in the picture, then practises identifying them.",
        ),
        "seasonal-puzzle" => (
            "मौसम का नाम बोलें। बड़े साथ में असली पहेली का एक टुकड़ा रखें।",
            "Name the season. Place one piece of a real seasonal puzzle together.",
        ),
        "request-action" => (
            "तस्वीर या इशारे से मनपसंद काम करने को कहें।",
            "Use words, the picture, or a gesture to request an action.",
        ),
        "request-missing" => (
            "अभ्यास में कौन-सी चीज़ चाहिए? बोलकर या इशारे से माँगें।",
            "What item do you need for this activity? Ask with words or a gesture.",
        ),
        "give-item" => (
            "बड़े एक निर्देश दें, फिर चीज़ सही व्यक्ति को दें।",
            "An adult gives one instruction. Deliver the object to that person.",
        ),
        "greetings" | "social-question" => (
            "हाथ हिलाकर नमस्ते कहें। फिर पूछें: आप कैसे हैं?",
            "Wave hello. Then ask: How are you?",
        ),
        "what-questions" => (
            "चीज़ का नाम बोलें। बड़े तस्वीर छिपाकर पूछें कि इसका क्या उपयोग है।",
            "Name the object. An adult then hides the picture and asks what it is used for.",
        ),
        "picture-description" => (
            "तस्वीर में कौन है और क्या हो रहा है?",
            "Who is in the picture, and what is happening?",
        ),
        "object-description" => (
            "यह क्या है? कैसी दिखती है? कहाँ काम आती है?",
            "What is it? What does it look like? Where is it used?",
        ),
        "where-cooking" => ("खाना कहाँ बनाते हैं?", "Where do we cook food?"),
        "where-bathing" => ("कहाँ नहाते हैं?", "Where do we bathe?"),
        "where-money" => ("पैसे कहाँ रखते हैं?", "Where do we keep money?"),
        "where-rubbish" => ("कूड़ा कहाँ डालते हैं?", "Where do we put rubbish?"),
        _ => return default.to_string(),
    };
    l.text(hi, en).to_string()
}

#[component]
pub(super) fn Practice(
    directory: PathBuf,
    language: Language,
    selected: String,
    flashcards: bool,
    on_select: EventHandler<String>,
) -> Element {
    let path = directory.join("aba-exercises.json");
    let loaded = use_hook(|| state::load_local::<Progress>(&path).map_err(|e| e.to_string()));
    let mut progress = use_signal(|| loaded.clone().unwrap_or_default());
    let mut failed = use_signal(|| false);
    let mut show_labels = use_signal(|| true);
    let mut hidden = use_signal(|| String::new());
    let persist = use_callback(move |next: Progress| {
        failed.set(state::save_local(&path, &next).is_err());
        progress.set(next);
    });
    let selection_to_save = selected.clone();
    let can_save = loaded.is_ok();
    use_effect(move || {
        if !can_save {
            return;
        }
        let mut n = progress();
        if n.selected != selection_to_save {
            n.selected = selection_to_save.clone();
            persist.call(n);
        }
    });
    let is_speech = selected == "speech";
    let item = activity(&selected);
    let ids: Vec<&str> = if is_speech {
        pictures::CARDS
            .iter()
            .filter(|c| c.id.starts_with("speech-"))
            .map(|c| c.id.as_str())
            .collect()
    } else {
        cards(&selected).iter().map(|id| resolved(id)).collect()
    };
    let round = progress
        .read()
        .activities
        .get(&selected)
        .cloned()
        .unwrap_or_default();
    let edit_selected = selected.clone();
    let record_selected = selected.clone();
    let record = use_callback(move |response: String| {
        let mut next = progress();
        let r = next.activities.entry(record_selected.clone()).or_default();
        r.responses.insert(r.position, response);
        r.position += 1;
        r.helped = false;
        r.boundary = r.position % 3 == 0;
        persist.call(next);
    });
    let edit = use_callback(move |r: Round| {
        let mut next = progress();
        next.activities.insert(edit_selected.clone(), r);
        persist.call(next);
    });
    if loaded.is_err() {
        return rsx! {p {role:"alert",{language.text("सहेजा अभ्यास नहीं खुला। वापस जाकर फिर कोशिश करें।","Saved practice could not be opened. Go back and try again.")}}};
    }
    rsx! {
     section { class:"aba-practice",
      h2 { {if flashcards {language.text("चित्र कार्ड","Flashcards")} else {language.text("अभ्यास","Exercises")}} }
      label { {language.text("अभ्यास चुनें","Choose practice")}
       select { value:"{selected}",onchange:move|e:Event<FormData>|{let id=e.value();let mut n=progress();n.selected=id.clone();persist.call(n);on_select.call(id);},
        for plan in home_plans::PLANS {
         optgroup { label:language.text(plan.hindi_title,plan.title),
          for a in plan.activities {option {value:a.id,{language.text(a.hindi_title,a.title)}}}
         }
        }
        option {value:"speech",{language.text("बोलने के कार्ड","Speech practice")}}
       }
      }
      if failed(){p {role:"alert",{language.text("अभ्यास नहीं सहेजा गया।","Practice could not be saved.")}} button {onclick:move|_|persist.call(progress()),{language.text("फिर सहेजें","Retry save")}}}
      if flashcards {
       label {input {r#type:"checkbox",checked:show_labels(),onchange:move|e|show_labels.set(e.checked())} {language.text("शब्द दिखाएँ","Show labels")}}
       if is_speech {p {{language.text("मूल कार्ड पर छपे शब्द तस्वीर में बने रहते हैं।","Original cards retain their printed labels inside the picture.")}}}
       div {class:"flashcard-grid",
        for id in ids {
         if let Some(card)=pictures::card(id) {
          article {class:"flashcard",key:"{id}",
           Image {id:id.to_string(),language}
           if show_labels(){h3 {"{card.label(language)}"} if !card.sound.is_empty(){p {"{card.sound}"}}}
           details {summary {{language.text("बड़ों के लिए / स्रोत","Adult notes / source")}}
            p {"{card.source}" if card.page>0 {" · {language.text(\"पन्ना\",\"Page\")} {card.page}"}}
            if !card.sound.is_empty(){p {{language.text("मूल कार्ड का ध्वनि संकेत; बड़े बोलकर उदाहरण दें।","Original card sound prompt; an adult models the practice.")}}}
           }
          }
         }
        }
       }
      } else if round.paused {
       h3 {{language.text("थोड़ा आराम करें","Take a short break")}}
       button {class:"primary",onclick:move|_|{let mut r=round.clone();r.paused=false;edit.call(r);},{language.text("जारी रखें","Continue")}}
      } else if round.boundary {
       h3 {{language.text("तीन अभ्यास पूरे","Three tasks practised")}}
       p {{language.text("फिर से करें या अगले तीन अभ्यास चुनें।","Repeat this round or continue with three more tasks.")}}
       button {onclick:{let r=round.clone();move|_|{let mut r=r.clone();r.position=r.position.saturating_sub(3);r.boundary=false;r.choices.retain(|p,_|*p<r.position);edit.call(r);}},{language.text("दोहराएँ","Repeat")}}
       button {class:"primary",onclick:{let r=round.clone();move|_|{let mut r=r.clone();r.boundary=false;edit.call(r);}},{language.text("अगले तीन","Next three")}}
       button {onclick:move|_|{let mut r=round.clone();r.paused=true;edit.call(r);},{language.text("आराम करें","Take a break")}}
      } else if !ids.is_empty() {
       {
        let position=round.position;
        let mode=if is_speech {Mode::Name}else{mode(&selected,position)};
        let current=ids[(position/3)%ids.len()];
        let other=ids[((position/3)+1)%ids.len()];
        let card=pictures::card(current).unwrap();
        let title=card.label(language);
        let model=if is_speech {format!("{} {}",card.sound,title)}else{item.map(|a|language.text(a.hindi_prompt,a.prompt).to_string()).unwrap_or_default()};
        let prompt=match mode {
         Mode::Identify=>format!("{}: {title}",language.text("छुएँ","Tap")),Mode::Match=>language.text("वही तस्वीर ढूँढें।","Find the same picture.").into(),
         Mode::Sort=>language.text("यह किस समूह में है?","Which group does this belong to?").into(),
         Mode::Pattern=>language.text("क्रम में अगला चित्र चुनें।","Choose the next picture in the pattern.").into(),
         Mode::YesNo=>format!("{} {}?",language.text("क्या यह है:","Is this a"),if position%2==0 {title}else{pictures::card(other).unwrap().label(language)}),
         Mode::Name=>language.text("यह क्या है? नाम बोलें या इशारा करें।","What is this? Say its name or point.").into(),
         Mode::Adult=>cue(&selected,language,item.map(|a|language.text(a.hindi_prompt,a.prompt)).unwrap_or(""))
        };
        let expected=match mode {Mode::YesNo=>usize::from(position%2!=0),Mode::Sort=>usize::from(card.category=="bird"),_=>position%2};
        let choices=if expected==0 {[current,other]}else{[other,current]};
        let chosen=round.choices.get(&position).copied();
        let interactive=matches!(mode,Mode::Identify|Mode::Match|Mode::Sort|Mode::Pattern|Mode::YesNo);
        let response=if round.helped {"With help"}else{"Independent"};
        rsx! {
         div {key:"{selected}-{position}",
          p {"{position%3+1} / 3"}
          button {onclick:{let r=round.clone();move|_|{let mut r=r.clone();r.paused=true;edit.call(r);}},{language.text("आराम करें","Take a break")}}
          h3 {"{prompt}"}
          if mode!=Mode::Identify && mode!=Mode::Pattern {
           if hidden()!=format!("{selected}-{position}") {Image {id:current.to_string(),language}}
           button {onclick:{let key=format!("{selected}-{position}");move|_|{if hidden()==key{hidden.set(String::new());}else{hidden.set(key.clone());}}},{language.text("तस्वीर छिपाएँ / दिखाएँ","Hide / show picture")}}
          }
          if mode==Mode::Pattern {
           div {class:"pattern-strip",Image {id:current.to_string(),language,compact:true} Image {id:other.to_string(),language,compact:true} Image {id:current.to_string(),language,compact:true} Image {id:other.to_string(),language,compact:true} strong {"?"}}
          }
          if interactive {
           div {class:"picture-choices",
            for i in 0..2 {
             button {disabled:chosen==Some(expected),aria_pressed:chosen==Some(i),onclick:{let r=round.clone();move|_|{let mut r=r.clone();r.choices.insert(position,i);if i!=expected{r.helped=true;}edit.call(r);}},
              if mode==Mode::YesNo {{if i==0{language.text("हाँ","Yes")}else{language.text("नहीं","No")}}}
              else if mode==Mode::Sort {{if i==0{language.text("वाहन","Transport")}else{language.text("पक्षी","Bird")}}}
              else {Image {id:choices[i].to_string(),language}}
             }
            }
           }
           if let Some(answer)=chosen {p {role:"status",{if answer==expected {language.text("अच्छा प्रयास!","Well done!")}else{language.text("फिर देखें और कोशिश करें।","Look again and try once more.")}}}}
           if chosen==Some(expected){button {class:"primary",onclick:move|_|record.call(response.to_string()),{language.text("अगला","Next")}}}
          }
          details {class:"learner-help",summary {{language.text("बड़ों के लिए उदाहरण","Adult model")}} p {"{model}"}}
          p {{language.text("बड़े जवाब दर्ज करें।","An adult records the response.")}}
          div {class:"actions",
           button {onclick:move|_|record.call("Independent".into()),{language.text("खुद किया","Independent")}}
           button {onclick:move|_|record.call("With help".into()),{language.text("मदद से किया","With help")}}
           button {onclick:move|_|record.call("Try later".into()),{language.text("बाद में करें","Try later")}}
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
    #[test]
    fn every_checklist_activity_has_cards() {
        let mut count = 0;
        for a in home_plans::PLANS.iter().flat_map(|p| p.activities) {
            count += 1;
            assert!(!cards(a.id).is_empty(), "{}", a.id);
            for id in cards(a.id) {
                assert!(pictures::card(resolved(id)).is_some(), "{}: {}", a.id, id);
            }
        }
        assert_eq!(count, 30);
    }
    #[test]
    fn rounds_persist_without_checklist_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aba.json");
        let mut p = Progress::default();
        let r = p.activities.entry("fruit".into()).or_default();
        r.position = 3;
        r.boundary = true;
        r.responses.insert(2, "With help".into());
        state::save_local(&path, &p).unwrap();
        let p: Progress = state::load_local(&path).unwrap();
        assert_eq!(p.activities["fruit"].responses[&2], "With help");
        assert!(p.activities["fruit"].boundary);
        assert!(serde_json::from_str::<Progress>("{}")
            .unwrap()
            .activities
            .is_empty());
    }
    #[test]
    fn task_modes_cover_picture_practice() {
        assert_eq!(mode("fruit", 0), Mode::Identify);
        assert_eq!(mode("fruit", 1), Mode::Match);
        assert_eq!(mode("fruit", 2), Mode::Name);
        assert_eq!(mode("transport-birds", 2), Mode::Sort);
        assert_eq!(mode("pattern-box", 0), Mode::Pattern);
        assert_eq!(mode("yes-no", 0), Mode::YesNo);
    }
}

pub(super) fn last_activity(directory: &std::path::Path) -> String {
    let id = state::load_local::<Progress>(&directory.join("aba-exercises.json"))
        .unwrap_or_default()
        .selected;
    if id == "speech" || activity(&id).is_some() {
        id
    } else {
        "fruit".to_string()
    }
}
