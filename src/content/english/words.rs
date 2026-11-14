use super::{
    q, topic,
    writing::{dictation, trace, write},
    Group, Kind, Question, Topic,
};

fn read(text: &str, question: Question) -> Kind {
    Kind::Read(text.into(), question)
}

pub(super) fn topics(grade: u8) -> Vec<Topic> {
    let sounds = Group::new("word-sounds", "Sound out words");
    let common = Group::new("common-words", "Read and write common words");
    let mut writing = Group::new("word-writing", "Write practice words");
    writing.separate_page = true;
    writing.guidance = "Read each word aloud, trace it, then write it on the blank row.".into();
    if grade == 1 {
        vec![
            topic("en1-family-an", "Read -an words", grade, sounds.clone(), [
                ("blend", read("Make the short a sound in apple. Add the n sound to make an. Say m, then an. Slide the sounds together to read man.",
                    q!("What word do m and an make?", "Man", "Map", 0))),
                ("swap", Kind::Choice(q!("Change the first sound in man to the sound for p. What word do you get?", "Pen", "Pan", 1))),
                ("sentence", read("I can see a fan. It is on the desk.",
                    q!("What can I see?", "A fan", "A cap", 0))),
            ], [
                q!("Which word rhymes with man?", "Map", "Fan", 1),
                q!("What word do r and an make?", "Ran", "Rap", 0),
                q!("Which word ends in -an?", "Fan", "Fat", 0),
                q!("Read ‘I can sit.’ Which word ends in -an?", "Sit", "Can", 1),
            ]),
            topic("en1-family-ap", "Read -ap words", grade, sounds, [
                ("blend", read("Make the short a sound in apple. Add the p sound to make ap. Say m, then ap. Slide the sounds together to read map.",
                    q!("What word do m and ap make?", "Man", "Map", 1))),
                ("swap", Kind::Choice(q!("Change the first sound in map to the sound for t. What word do you get?", "Tap", "Tan", 0))),
                ("sentence", read("I tap my cap. Then I put it on.",
                    q!("What do I put on?", "A pan", "A cap", 1))),
            ], [
                q!("Which word rhymes with tap?", "Cap", "Can", 0),
                q!("What word do n and ap make?", "Nan", "Nap", 1),
                q!("Which word ends in -ap?", "Map", "Mat", 0),
                q!("Read ‘I take a nap.’ Which word ends in -ap?", "Take", "Nap", 1),
            ]),
            topic("en1-common-i-see", "Read and write I, see and a", grade, common.clone(), [
                ("read", read("Read aloud: I see a bag. I see a pen.",
                    q!("Which word comes just after I in both sentences?", "See", "Bag", 0))),
                ("trace", trace("see", "Say see. Keep the letters between the blue lines.")),
                ("copy", write("Copy ‘I see a bag.’ Keep I capital. Leave a space between each word.", 1)),
            ], [
                q!("Which word completes ‘I ___ a bag’?", "a", "see", 1),
                q!("Which word completes ‘I see ___ pen’?", "a", "I", 0),
                q!("Which is the correct way to write the word for yourself?", "i", "I", 1),
                q!("Which sentence has a space between every word?", "I see a pen.", "Isee apen.", 0),
            ]),
            topic("en1-common-my-is", "Read and write my, is and the", grade, common, [
                ("read", read("Read aloud: My cap is on the mat.",
                    q!("What is on the mat?", "My cap", "My bag", 0))),
                ("trace", trace("my", "Say my. Keep m between the blue lines. Let the tail of y go below the lower blue line.")),
                ("write", dictation("Write one word on each row. Say it before you write it.",
                    "Say ‘is’, as in ‘The cat is here.’ On the second row say ‘the’, as in ‘the cat.’", 2)),
            ], [
                q!("Which word means the cap belongs to me?", "on", "my", 1),
                q!("Which word completes ‘My bag ___ here’?", "is", "my", 0),
                q!("Which word completes ‘I see ___ cat’?", "the", "is", 0),
                q!("Which two letters spell is?", "s, i", "i, s", 1),
            ]),
            topic("en1-write-an-words", "Write can, fan and pan", grade, writing.clone(), [
                ("can", trace("can", "Say c, a, n. Keep all three letters between the blue lines.")),
                ("fan", trace("fan", "Keep a and n small. Make the f taller.")),
                ("pan", trace("pan", "Keep the round part of p between the blue lines. Take its stem below.")),
                ("dictation", dictation("Listen and write one word on each row.", "Say ‘fan’, as in ‘Turn on the fan.’ On the second row say ‘pan’, as in ‘Put it in the pan.’", 2)),
            ], [
                q!("Which word begins with f?", "can", "fan", 1),
                q!("Which word can you use to cook?", "pan", "fan", 0),
                q!("Which letters end the word can?", "ap", "an", 1),
                q!("Which word completes ‘I ___ run’?", "can", "pan", 0),
            ]),
            topic("en1-write-ap-words", "Write cap, map and tap", grade, writing, [
                ("cap", trace("cap", "Say c, a, p. Take the p stem below the lower blue line.")),
                ("map", trace("map", "Keep the two humps of m between the blue lines.")),
                ("tap", trace("tap", "Make t taller than a, and take the stem of p below the lower blue line.")),
                ("dictation", dictation("Listen and write one word on each row.", "Say ‘map’, as in ‘Look at the map.’ On the second row say ‘tap’, as in ‘Turn off the tap.’", 2)),
            ], [
                q!("Which word names something you wear?", "cap", "map", 0),
                q!("Which word starts with t?", "cap", "tap", 1),
                q!("Which letters end the word map?", "ap", "an", 0),
                q!("Which word names something that can show you the way?", "tap", "map", 1),
            ]),
        ]
    } else {
        vec![
            topic("en2-compare-an-ap", "Read and spell -an and -ap", grade, sounds, [
                ("compare", read("Read each pair aloud: can and cap, ran and rap. The short a stays the same. The last sound changes.",
                    q!("What changes from can to cap?", "The first sound", "The last sound", 1))),
                ("sentence", read("A man ran to the shop. He left his cap on the bench.",
                    q!("What did the man leave on the bench?", "His cap", "His bag", 0))),
                ("spell", dictation("Write one word on each row. Listen for its last sound.",
                    "Say ‘can’, as in ‘I can run.’ On the second row say ‘cap’, as in ‘I wear a cap.’", 2)),
            ], [
                q!("Which pair ends in -an?", "Cap and tap", "Fan and ran", 1),
                q!("Which word has the same ending as map?", "Tap", "Man", 0),
                q!("Read ‘I can see a cap.’ Which word has the last sound n?", "Can", "Cap", 0),
                q!("Change the last sound in lap from p to n. What ending does the new word have?", "-ap", "-an", 1),
            ]),
            topic("en2-common-they-have", "Read and write they, have and are", grade, common.clone(), [
                ("read", read("Read aloud: They have two books. The books are on the desk.",
                    q!("How many books do they have?", "One", "Two", 1))),
                ("trace", trace("they", "Read they. Notice the tall h and the tail of y.")),
                ("write", dictation("Write one word on each row. Reread both words.",
                    "Say ‘have’, as in ‘We have two books.’ On the second row say ‘are’, as in ‘The books are here.’", 2)),
            ], [
                q!("Which word completes ‘___ have two books’?", "They", "Are", 0),
                q!("Which word completes ‘The pens ___ here’?", "have", "are", 1),
                q!("Which word completes ‘We ___ two pens’?", "have", "are", 0),
                q!("Which is spelt correctly?", "thay", "they", 1),
            ]),
            topic("en2-common-with-for", "Read and write with, for and said", grade, common, [
                ("read", read("Read aloud: I went with Ravi. ‘This book is for you,’ he said.",
                    q!("Who went with me?", "Ravi", "Mina", 0))),
                ("trace", trace("said", "Say said. Notice the letters ai in the middle.")),
                ("write", dictation("Write one word on each row. Read the words back.",
                    "Say ‘with’, as in ‘I went with Ravi.’ On the second row say ‘for’, as in ‘This book is for you.’", 2)),
            ], [
                q!("Which word completes ‘I went ___ Ravi’?", "said", "with", 1),
                q!("Which word completes ‘This gift is ___ you’?", "for", "with", 0),
                q!("Which word tells you someone spoke?", "for", "said", 1),
                q!("Which is spelt correctly?", "said", "sed", 0),
            ]),
            topic("en2-write-mixed-words", "Write ran, rap, plan and flap", grade, writing, [
                ("ran", trace("ran", "Say ran. Keep r, a and n between the blue lines.")),
                ("rap", trace("rap", "Say rap. Its last sound is p. Take the stem of p below the lower blue line.")),
                ("copy", write("Read ‘plan’ and ‘flap’. Copy plan on the first row and flap on the second row. Say each word as you write.", 2)),
                ("dictation", dictation("Listen and write one word on each row. Check the last letter.", "Say ‘plan’, as in ‘Make a plan.’ On the second row say ‘flap’, as in ‘Birds flap their wings.’", 2)),
            ], [
                q!("Which word ends with the same sound as ran?", "flap", "plan", 1),
                q!("Which word ends in -ap?", "flap", "ran", 0),
                q!("Which letter changes when ran becomes rap?", "The last letter", "The first letter", 0),
                q!("Which word completes ‘Make a ___’?", "flap", "plan", 1),
            ]),
        ]
    }
}
