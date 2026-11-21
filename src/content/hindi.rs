use super::{Exercise, Group, Kind, Paper, Question, Topic};

enum HindiGroup {
    Lesson,
    Conversation,
    Writing,
    Barakhadi,
}

fn group(section: HindiGroup, grade: u8) -> Group {
    let (id, title, guidance, separate_page) = match section {
        HindiGroup::Lesson => ("lesson", "पढ़ो", "", false),
        HindiGroup::Conversation => ("conversation", "सुनो और बात करो", "", false),
        HindiGroup::Writing => (
            "writing",
            "लिखने का अभ्यास",
            if grade == 1 {
                "अक्षर → मात्राएँ → शब्द → छोटे वाक्य"
            } else {
                "वर्तनी → कहानी → निर्देश"
            },
            false,
        ),
        HindiGroup::Barakhadi => (
            "barakhadi",
            "पूरी बारहखड़ी",
            "एक व्यंजन चुनो। एक बार में तीन रूप लिखो, फिर आराम करो।",
            true,
        ),
    };
    Group {
        id: id.into(),
        title: title.into(),
        guidance: guidance.into(),
        separate_page,
    }
}

fn trace(model: &str) -> Kind {
    Kind::Trace {
        model: model.into(),
        guidance: formation_hint(model).into(),
        paper: Paper::Hindi,
    }
}
macro_rules! question {
    ($prompt:expr, $a:expr, $b:expr, $answer:literal) => {
        Question {
            prompt: ($prompt).into(),
            options: vec![($a).into(), ($b).into()],
            answer: $answer,
        }
    };
}

macro_rules! writing {
    ($($id:literal => $glyph:expr),+ $(,)?) => {
        vec![$(Exercise { picture: None, id: $id.into(), kind: trace($glyph) }),+]
    };
}

fn story() -> Vec<Exercise> {
    vec![
        Exercise {
            picture: None,
            id: "home".into(),
            kind: Kind::Read(
                "नीमा स्कूल से घर आई। घर पर दादी थीं।".into(),
                question!("घर पर कौन थीं?", "दादी", "माँ", 0),
            ),
        },
        Exercise {
            picture: None,
            id: "request".into(),
            kind: Kind::Read(
                "शाम हुई। नीमा खेलने जाने लगी। दादी ने उसे थोड़ी देर पास बैठने को कहा।".into(),
                question!("दादी ने नीमा से क्या कहा?", "खेलने जाओ", "पास बैठो", 1),
            ),
        },
        Exercise {
            picture: None,
            id: "field".into(),
            kind: Kind::Read(
                "नीमा दादी की चप्पलें ले आई। फिर दोनों खेलने के मैदान की ओर चल पड़ीं।".into(),
                question!("पहले क्या हुआ?", "नीमा चप्पलें ले आई", "दोनों मैदान की ओर चलीं", 0),
            ),
        },
        Exercise {
            picture: None,
            id: "short-answer".into(),
            kind: Kind::Answer(
                "नीमा स्कूल से घर आई। घर पर दादी थीं।".into(),
                question!("घर पर कौन थीं? एक शब्द लिखो।", "माँ", "दादी", 1),
            ),
        },
    ]
}

macro_rules! topic {
    ($id:literal, $title:literal, $grade:literal, $section:ident, $exercises:expr, $($recall:expr),+ $(,)?) => {
        Topic { id: $id.into(), title: $title.into(), grade: $grade, group: group(HindiGroup::$section, $grade), revision: 0, break_every: None,
            exercises: $exercises, recall: vec![$($recall),+] }
    };
}

macro_rules! barakhadi {
    ($id:literal, $letter:literal) => {
        Topic {
            id: $id.into(), title: $letter.into(), grade: 1, group: group(HindiGroup::Barakhadi, 1), revision: 0, break_every: Some(3),
            exercises: writing![
                "a" => $letter, "aa" => concat!($letter,"ा"), "i" => concat!($letter,"ि"),
                "ii" => concat!($letter,"ी"), "u" => concat!($letter,"ु"), "uu" => concat!($letter,"ू"),
                "e" => concat!($letter,"े"), "ai" => concat!($letter,"ै"), "o" => concat!($letter,"ो"),
                "au" => concat!($letter,"ौ"), "am" => concat!($letter,"ं"), "ah" => concat!($letter,"ः"),
            ],
            recall: vec![
                question!(concat!($letter," का बिना मात्रा वाला रूप चुनो।"), $letter, concat!($letter,"ा"), 0),
                question!(concat!($letter," में आ की मात्रा लगाओ।"), concat!($letter,"ि"), concat!($letter,"ा"), 1),
                question!(concat!($letter," में इ की मात्रा लगाओ।"), concat!($letter,"ि"), concat!($letter,"ी"), 0),
                question!(concat!($letter," में ई की मात्रा लगाओ।"), concat!($letter,"ि"), concat!($letter,"ी"), 1),
                question!(concat!($letter," में उ की मात्रा लगाओ।"), concat!($letter,"ु"), concat!($letter,"ू"), 0),
                question!(concat!($letter," में ऊ की मात्रा लगाओ।"), concat!($letter,"ु"), concat!($letter,"ू"), 1),
                question!(concat!($letter," में ए की मात्रा लगाओ।"), concat!($letter,"े"), concat!($letter,"ै"), 0),
                question!(concat!($letter," में ऐ की मात्रा लगाओ।"), concat!($letter,"े"), concat!($letter,"ै"), 1),
                question!(concat!($letter," में ओ की मात्रा लगाओ।"), concat!($letter,"ो"), concat!($letter,"ौ"), 0),
                question!(concat!($letter," में औ की मात्रा लगाओ।"), concat!($letter,"ो"), concat!($letter,"ौ"), 1),
                question!(concat!($letter," में अनुस्वार लगाओ।"), concat!($letter,"ं"), concat!($letter,"ः"), 0),
                question!(concat!($letter," में विसर्ग लगाओ।"), concat!($letter,"ं"), concat!($letter,"ः"), 1),
            ],
        }
    };
}

fn class_one_reading() -> Vec<Exercise> {
    vec![
        Exercise {
            picture: None,
            id: "sounds".into(),
            kind: Kind::Read(
                "म से माला। क से किताब। दोनों शब्द धीरे पढ़ो।".into(),
                question!("किताब की पहली ध्वनि किस अक्षर से शुरू होती है?", "क", "म", 0),
            ),
        },
        Exercise {
            picture: None,
            id: "sentences".into(),
            kind: Kind::Read(
                "मीना के पास माला है। रवि के पास किताब है। हर वाक्य के बाद रुको।".into(),
                question!("माला किसके पास है?", "रवि", "मीना", 1),
            ),
        },
        Exercise {
            picture: None,
            id: "picture".into(),
            kind: Kind::Answer(
                "📚  मीना किताब पढ़ रही है।".into(),
                question!("मीना क्या पढ़ रही है? एक शब्द लिखो।", "किताब", "माला", 0),
            ),
        },
    ]
}

fn class_one_talk() -> Vec<Exercise> {
    vec![
        Exercise {
            picture: None,
            id: "first-sound".into(),
            kind: Kind::Listen(
                "पहले ‘म’ की ध्वनि बोलें। फिर ‘माला’ धीरे बोलें और शुरुआत की ध्वनि दोहराएँ।".into(),
                question!("जो ध्वनि सुनी, उसका अक्षर चुनो।", "म", "क", 0),
            ),
        },
        Exercise {
            picture: None,
            id: "story".into(),
            kind: Kind::Listen(
                "रीना ने अपना थैला खोला। थैले में एक लाल गेंद थी। रीना गेंद लेकर खेलने गई।".into(),
                question!("थैले में क्या था?", "गेंद", "किताब", 0),
            ),
        },
        Exercise {
            picture: None,
            id: "poem".into(),
            kind: Kind::Listen(
                "छोटी चिड़िया दाना खाए। पंख हिलाकर उड़ती जाए।".into(),
                question!("चिड़िया क्या खाती है?", "फल", "दाना", 1),
            ),
        },
        Exercise {
            picture: None,
            id: "picture-talk".into(),
            kind: Kind::Talk {
                picture: "🎒 📕 🍎".into(),
                prompt: "तुम अपने थैले में क्या रखते हो?".into(),
                starters: vec!["किताब".into(), "फल".into(), "कुछ और".into()],
                adult: "कहें, ‘मेरे थैले में किताब है। तुम्हारे थैले में क्या है?’ बच्चे के जवाब के बाद पूछें, ‘और क्या?’"
                    .into(),
            },
        },
    ]
}

fn class_two_reading() -> Vec<Exercise> {
    vec![
    Exercise { picture: None, id: "clusters".into(), kind: Kind::Read("बच् + चा = बच्चा। बिल् + ली = बिल्ली। बस् + ता = बस्ता। पहले हिस्से पढ़ो, फिर पूरे शब्द।".into(),
        question!("किस शब्द में ल् + ल आता है?", "बच्चा", "बिल्ली", 1)) },
    Exercise { picture: None, id: "story".into(), kind: Kind::Read("एक बच्चा बस्ता लेकर निकला। रास्ते में उसे एक बिल्ली दिखी। उसने बिल्ली के लिए पानी रखा। पूर्ण विराम पर थोड़ा रुको।".into(),
        question!("बच्चे ने पानी क्यों रखा?", "बिल्ली के लिए", "बस्ता धोने के लिए", 0)) },
    Exercise { picture: None, id: "poem".into(), kind: Kind::Read("बादल आए, हवा चली। छत पर बूँदें टप-टप पड़ीं। कविता लय से पढ़ो।".into(),
        question!("टप-टप किसकी आवाज़ है?", "हवा की", "बूँदों की", 1)) },
]
}

fn class_two_talk() -> Vec<Exercise> {
    vec![
    Exercise { picture: None, id: "story".into(), kind: Kind::Listen("आरव की नाव पानी में फँस गई। उसकी बहन ने एक डंडी से उसे किनारे किया। फिर दोनों ने मिलकर नाव चलाई।".into(),
        question!("नाव को किनारे किसने किया?", "आरव की बहन ने", "आरव ने", 0)) },
    Exercise { picture: None, id: "poem".into(), kind: Kind::Listen("रात हुई तो तारा आया। चाँद ने उजला दीप जलाया।".into(),
        question!("कविता में तारा कब आया?", "सुबह", "रात में", 1)) },
    Exercise { picture: None, id: "retell".into(), kind: Kind::Talk {
        picture: "⛵ 💧 🪵".into(), prompt: "नाव के साथ पहले क्या हुआ? फिर क्या हुआ?".into(),
        starters: vec!["पहले नाव फँसी".into(), "फिर नाव किनारे आई".into()],
        adult: "पहले और फिर कहकर कहानी दोहराएँ। बच्चे को एक संकेत चुनने, लिखने या बोलने का समय दें।".into(),
    }},
    Exercise { picture: None, id: "ask".into(), kind: Kind::Talk {
        picture: "⛵".into(), prompt: "नाव के बारे में एक सवाल पूछो।".into(),
        starters: vec!["नाव कहाँ थी?".into(), "नाव किसने निकाली?".into()],
        adult: "बच्चे के चुने, लिखे या बोले सवाल का जवाब दें। फिर अपनी बारी में एक सवाल पूछें।".into(),
    }},
    Exercise { picture: None, id: "reason".into(), kind: Kind::Talk {
        picture: "⛵ 🪵".into(), prompt: "क्या बहन ने अच्छा काम किया? क्यों?".into(),
        starters: vec!["हाँ, क्योंकि उसने मदद की".into(), "मेरी अपनी वजह".into()],
        adult: "बच्चे की वजह सुनें। जरूरत पर ‘क्योंकि…’ से शुरुआत करने में मदद करें।".into(),
    }},
]
}

fn class_one_writing() -> Vec<Exercise> {
    vec![
        Exercise {
            picture: None,
            id: "mala".into(),
            kind: trace("माला"),
        },
        Exercise {
            picture: None,
            id: "spelling".into(),
            kind: Kind::Compose("शब्द सुनकर खुद लिखो।".into(), 1, Some("माला".into())),
        },
        Exercise {
            picture: None,
            id: "sentence".into(),
            kind: Kind::Compose("📕 किताब देखकर एक छोटा वाक्य लिखो।".into(), 1, None),
        },
    ]
}

fn class_two_writing() -> Vec<Exercise> {
    vec![
        Exercise {
            picture: None,
            id: "billi".into(),
            kind: trace("बिल्ली"),
        },
        Exercise {
            picture: None,
            id: "spelling".into(),
            kind: Kind::Compose(
                "दोनों शब्द सुनकर खुद लिखो।".into(),
                1,
                Some("बिल्ली, बच्चा".into()),
            ),
        },
        Exercise {
            picture: None,
            id: "story".into(),
            kind: Kind::Compose("🌱 💧 🌼 चित्रों को देखकर दो जुड़े वाक्य लिखो।".into(), 3, None),
        },
        Exercise {
            picture: None,
            id: "steps".into(),
            kind: Kind::Compose("🌱 पौधे की देखभाल कैसे करें? दो निर्देश लिखो।".into(), 3, None),
        },
    ]
}

pub(super) fn topics() -> Vec<Topic> {
    vec![
        topic!(
            "sounds-and-sentences",
            "ध्वनि, शब्द और वाक्य",
            1,
            Lesson,
            class_one_reading(),
            question!("माला का पहला अक्षर कौन-सा है?", "म", "क", 0),
            question!("किताब की पहली मात्रा कौन-सी है?", "ा", "ि", 1),
            question!("वाक्य पूरा करो: मीना किताब ___ रही है।", "पढ़", "खेल", 0)
        ),
        topic!(
            "bag-talk",
            "सुनो और बात करो · मेरा थैला",
            1,
            Conversation,
            class_one_talk(),
            question!("लाल गेंद कहाँ थी?", "थैले में", "मेज़ पर", 0),
            question!("चिड़िया कैसे उड़ती है?", "पंख हिलाकर", "दौड़कर", 0),
            question!("थैले में रखी जाने वाली चीज़ चुनो।", "पेड़", "किताब", 1)
        ),
        topic!(
            "neema",
            "नीमा की दादी",
            2,
            Lesson,
            story(),
            question!("नीमा किसके साथ मैदान की ओर चली?", "माँ", "दादी", 1),
            question!("दादी ने नीमा से क्या कहा?", "पास बैठो", "किताब पढ़ो", 0),
            question!("मैदान जाने से पहले नीमा क्या लाई?", "गेंद", "चप्पलें", 1),
            question!("नीमा पहले कहाँ से लौटी?", "स्कूल से", "मैदान से", 0)
        ),
        topic!(
            "clusters-and-poem",
            "संयुक्ताक्षर, कहानी और कविता",
            2,
            Lesson,
            class_two_reading(),
            question!("बच् + चा से कौन-सा शब्द बनता है?", "बच्चा", "बिल्ली", 0),
            question!("बच्चे ने किसके लिए पानी रखा?", "अपने लिए", "बिल्ली के लिए", 1),
            question!("कविता में टप-टप क्या पड़ीं?", "बूँदें", "पत्तियाँ", 0)
        ),
        topic!(
            "boat-talk",
            "सुनो, पूछो और बताओ · नाव",
            2,
            Conversation,
            class_two_talk(),
            question!("नाव निकालने के लिए बहन ने क्या लिया?", "डंडी", "पत्थर", 0),
            question!("नाव फँसी तो किसने मदद की?", "आरव की बहन ने", "किसी ने नहीं", 0),
            question!("कविता में उजला दीप किसने जलाया?", "चाँद ने", "बादल ने", 0)
        ),
        topic!(
            "spelling-and-story",
            "वर्तनी, कहानी और निर्देश",
            2,
            Writing,
            class_two_writing(),
            question!("सही वर्तनी चुनो।", "बिली", "बिल्ली", 1),
            question!(
                "कहानी में बीज बोने के बाद क्या किया?",
                "पानी दिया",
                "बीज खरीदा",
                0
            ),
            question!(
                "पौधे की देखभाल का निर्देश चुनो।",
                "पौधे को पानी दो।",
                "पत्तियाँ तोड़ो।",
                0
            )
        ),
        topic!(
            "vowels-one",
            "स्वर · अ आ इ",
            1,
            Writing,
            writing!["a"=>"अ","aa"=>"आ","i"=>"इ"],
            question!("अनार का पहला अक्षर कौन-सा है?", "अ", "आ", 0),
            question!("आम का पहला अक्षर कौन-सा है?", "अ", "आ", 1),
            question!("इमली का पहला अक्षर कौन-सा है?", "इ", "ई", 0)
        ),
        topic!(
            "vowels-two",
            "स्वर · ई उ ऊ",
            1,
            Writing,
            writing!["ii"=>"ई","u"=>"उ","uu"=>"ऊ"],
            question!("ईख का पहला अक्षर कौन-सा है?", "इ", "ई", 1),
            question!("उल्लू का पहला अक्षर कौन-सा है?", "उ", "ऊ", 0),
            question!("ऊन का पहला अक्षर कौन-सा है?", "उ", "ऊ", 1)
        ),
        topic!(
            "vowels-three",
            "स्वर · ऋ ए ऐ",
            1,
            Writing,
            writing!["ri"=>"ऋ","e"=>"ए","ai"=>"ऐ"],
            question!("ऋषि का पहला अक्षर कौन-सा है?", "ऋ", "ए", 0),
            question!("एक का पहला अक्षर कौन-सा है?", "ऐ", "ए", 1),
            question!("ऐनक का पहला अक्षर कौन-सा है?", "ऐ", "ए", 0)
        ),
        topic!(
            "vowels-four",
            "स्वर · ओ औ अं अः",
            1,
            Writing,
            writing!["o"=>"ओ","au"=>"औ","am"=>"अं","ah"=>"अः"],
            question!("ओस का पहला अक्षर कौन-सा है?", "ओ", "औ", 0),
            question!("और का पहला अक्षर कौन-सा है?", "ओ", "औ", 1),
            question!("अंगूर की शुरुआत कैसे होती है?", "अं", "अः", 0),
            question!("विसर्ग वाला रूप कौन-सा है?", "अं", "अः", 1)
        ),
        topic!(
            "consonants",
            "व्यंजन · क म न",
            1,
            Writing,
            writing!["ka"=>"क","ma"=>"म","na"=>"न"],
            question!("कमल का पहला अक्षर कौन-सा है?", "क", "म", 0),
            question!("मछली का पहला अक्षर कौन-सा है?", "न", "म", 1),
            question!("नल का पहला अक्षर कौन-सा है?", "न", "क", 0)
        ),
        topic!(
            "matra-forms",
            "मात्राएँ · ा ि ी",
            1,
            Writing,
            writing!["kaa"=>"का","ki"=>"कि","kii"=>"की"],
            question!("क में आ की मात्रा लगाओ।", "का", "कि", 0),
            question!("किताब की शुरुआत कैसे होती है?", "की", "कि", 1),
            question!("कील की शुरुआत कैसे होती है?", "की", "कि", 0)
        ),
        topic!(
            "matra-words",
            "शब्द · नाम किताब नीमा",
            1,
            Writing,
            writing!["naam"=>"नाम","kitaab"=>"किताब","neema"=>"नीमा"],
            question!("सही वर्तनी चुनो।", "नाम", "नम", 0),
            question!("सही वर्तनी चुनो।", "कीताब", "किताब", 1),
            question!("किस शब्द में ी की मात्रा है?", "नीमा", "नाम", 0)
        ),
        topic!(
            "first-writing",
            "शब्द और छोटा वाक्य",
            1,
            Writing,
            class_one_writing(),
            question!("सही शब्द कौन-सा है?", "माला", "मला", 0),
            question!("सही वाक्य चुनो।", "मीना किताब पढ़ती है।", "किताब मीना पढ़ती।", 0)
        ),
        topic!(
            "ri-one",
            "ऋ की मात्रा · कृ गृ तृ",
            1,
            Writing,
            writing!["0"=>"कृ","1"=>"गृ","2"=>"तृ"],
            question!("क में ऋ की मात्रा लगाओ।", "कृ", "कु", 0),
            question!("ग में ऋ की मात्रा लगाओ।", "गु", "गृ", 1),
            question!("त में ऋ की मात्रा लगाओ।", "तृ", "तु", 0)
        ),
        topic!(
            "ri-two",
            "ऋ की मात्रा · दृ धृ नृ",
            1,
            Writing,
            writing!["0"=>"दृ","1"=>"धृ","2"=>"नृ"],
            question!("द में ऋ की मात्रा लगाओ।", "दृ", "दु", 0),
            question!("ध में ऋ की मात्रा लगाओ।", "धु", "धृ", 1),
            question!("न में ऋ की मात्रा लगाओ।", "नृ", "नु", 0)
        ),
        topic!(
            "ri-three",
            "ऋ की मात्रा · पृ मृ वृ हृ",
            1,
            Writing,
            writing!["0"=>"पृ","1"=>"मृ","2"=>"वृ","3"=>"हृ"],
            question!("प में ऋ की मात्रा लगाओ।", "पृ", "पु", 0),
            question!("म में ऋ की मात्रा लगाओ।", "मु", "मृ", 1),
            question!("व में ऋ की मात्रा लगाओ।", "वृ", "वु", 0),
            question!("ह में ऋ की मात्रा लगाओ।", "हु", "हृ", 1)
        ),
        topic!(
            "nukta",
            "नुक्ता · ड़ ढ़",
            1,
            Writing,
            writing!["0"=>"ड़","1"=>"ढ़"],
            question!("नुक्ते वाला रूप चुनो।", "ड़", "ड", 0),
            question!("नुक्ते वाला रूप चुनो।", "ढ", "ढ़", 1)
        ),
        barakhadi!("barakhadi-1", "क"),
        barakhadi!("barakhadi-2", "ख"),
        barakhadi!("barakhadi-3", "ग"),
        barakhadi!("barakhadi-4", "घ"),
        barakhadi!("barakhadi-5", "ङ"),
        barakhadi!("barakhadi-6", "च"),
        barakhadi!("barakhadi-7", "छ"),
        barakhadi!("barakhadi-8", "ज"),
        barakhadi!("barakhadi-9", "झ"),
        barakhadi!("barakhadi-10", "ञ"),
        barakhadi!("barakhadi-11", "ट"),
        barakhadi!("barakhadi-12", "ठ"),
        barakhadi!("barakhadi-13", "ड"),
        barakhadi!("barakhadi-14", "ढ"),
        barakhadi!("barakhadi-15", "ण"),
        barakhadi!("barakhadi-16", "त"),
        barakhadi!("barakhadi-17", "थ"),
        barakhadi!("barakhadi-18", "द"),
        barakhadi!("barakhadi-19", "ध"),
        barakhadi!("barakhadi-20", "न"),
        barakhadi!("barakhadi-21", "प"),
        barakhadi!("barakhadi-22", "फ"),
        barakhadi!("barakhadi-23", "ब"),
        barakhadi!("barakhadi-24", "भ"),
        barakhadi!("barakhadi-25", "म"),
        barakhadi!("barakhadi-26", "य"),
        barakhadi!("barakhadi-27", "र"),
        barakhadi!("barakhadi-28", "ल"),
        barakhadi!("barakhadi-29", "व"),
        barakhadi!("barakhadi-30", "श"),
        barakhadi!("barakhadi-31", "ष"),
        barakhadi!("barakhadi-32", "स"),
        barakhadi!("barakhadi-33", "ह"),
        barakhadi!("barakhadi-36", "क्ष"),
        barakhadi!("barakhadi-37", "त्र"),
        barakhadi!("barakhadi-38", "ज्ञ"),
        barakhadi!("barakhadi-39", "श्र"),
    ]
}
fn formation_hint(text: &str) -> &'static str {
    if text.contains('ि') {
        "पहले अक्षर की बनावट देखो। छोटी इ की मात्रा बाईं ओर लिखो और ऊपर की रेखा से जोड़ो।"
    } else if text.contains('ी') {
        "पहले अक्षर की बनावट देखो। बड़ी ई की मात्रा दाईं ओर लिखो और ऊपर की रेखा से जोड़ो।"
    } else if text.contains(['ु', 'ू', 'ृ']) {
        "अक्षर की मुख्य बनावट बीच की पट्टी में रखो। नीचे वाली मात्रा के लिए अगली पट्टी का उपयोग करो।"
    } else if text.contains(['े', 'ै', 'ो', 'ौ', 'ं']) {
        "मुख्य अक्षर बीच की पट्टी में लिखो। ऊपर का चिह्न पहली पट्टी में रखो।"
    } else if text == "क" {
        "बीच का घुमाव और दाईं ओर की खड़ी रेखा देखो। आकार पूरा कर ऊपर की रेखा बनाओ।"
    } else {
        "नमूने में मुख्य आकार देखो। उसे बीच की पट्टी में बनाओ। ऊपर की रेखा और मात्राओं की जगह जाँचो।"
    }
}
