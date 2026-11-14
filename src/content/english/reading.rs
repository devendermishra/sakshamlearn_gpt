use super::{q, topic, Group, Kind, Question, Topic};
use crate::content::Picture;

pub(super) fn topics(grade: u8) -> Vec<Topic> {
    let group = Group::new("reading", "Read and understand");
    macro_rules! lesson {
        ($id:literal, $title:literal, $passage:literal, $check:expr, $practice:expr;
            $($recall:expr),+ $(,)?) => {
            lesson!($id, $title, Kind::Read($passage.into(), $check), $practice; $($recall),+)
        };
        ($id:literal, $title:literal, $opening:expr, $practice:expr;
            $($recall:expr),+ $(,)?) => {
            topic($id, $title, grade, group.clone(), [
                ("read", $opening),
                ("practice", $practice),
            ], [$($recall),+])
        };
    }
    if grade == 1 {
        // Mridang 1, aemr101–109. Passages and questions are original practice.
        vec![
            topic("en1-sound-out", "Sound out short words", grade, group.clone(), [
                ("blend", Kind::Read("Say the sounds for c, a and p. Say them together to read cap.".into(),
                    q!("Which word did you make?", "Cap", "Cup", 0))),
                ("change", Kind::Read("Read cap. Change the first sound to the sound for t. Read the new word.".into(),
                    q!("What is the new word?", "Tip", "Tap", 1))),
                ("sentence", Kind::Read("I see a cap on the mat.".into(),
                    q!("Where is the cap?", "On the mat", "In a bag", 0))),
            ], [
                q!("Which starts with the same sound as cap?", "Tap", "Cat", 1),
                q!("Say the sounds for m, a and p together. Which word do you make?", "Map", "Mat", 0),
                q!("Which sound changes when cap becomes cup?", "The last sound", "The middle sound", 1),
                q!("Read ‘The cat is on the mat.’ What is on the mat?", "A cat", "A cap", 0),
            ]),
            lesson!("en1-hands", "Two Little Hands",
                "I clap my hands. I tap my feet.",
                q!("What do I clap?", "My hands", "My feet", 0),
                Kind::Choice(q!("Say clap and tap. Which word also rhymes with tap?", "Run", "Cap", 1));
                q!("What do I tap?", "My head", "My feet", 1),
                q!("Which begins with the same sound as hand?", "Hat", "Mat", 0),
                q!("Which word rhymes with clap?", "Cup", "Map", 1),
                q!("Which body part helps you see?", "Eyes", "Ears", 0),
            ),
            lesson!("en1-greetings", "Greetings",
                "Asha sees her friend before class. Asha says, ‘Good morning!’",
                q!("What does Asha say?", "Good night!", "Good morning!", 1),
                Kind::Answer("Good morning, Asha!".into(), q!("Write the name you see.", "Asha", "Mina", 0));
                q!("When does Asha meet her friend?", "Before class", "At bedtime", 0),
                q!("Which word begins with a capital letter in ‘Good morning’?", "morning", "Good", 1),
                q!("What can you say when you go to bed?", "Good night!", "Good afternoon!", 0),
                q!("Which words can start a chat with a friend?", "Goodbye!", "Hello, Ria!", 1),
            ),
            lesson!("en1-family", "A Sparrow Family",
                "A small sparrow sits in a nest. Two grown sparrows bring it food.",
                q!("Where does the small sparrow sit?", "In a nest", "On a mat", 0),
                Kind::Choice(q!("Say nest. Which word starts with the same sound?", "Best", "Net", 1));
                q!("What do the grown birds bring?", "Food", "A ball", 0),
                q!("‘The bird is in the nest.’ Which word tells where it is?", "Bird", "In", 1),
                q!("Which word starts with the same sound as net?", "Nose", "Rose", 0),
                q!("‘Two birds sit in a tree.’ How many birds sit there?", "One", "Two", 1),
            ),
            lesson!("en1-picture-time", "Picture Time",
                Kind::Picture(Picture::Animals, q!("Which animal is holding on to the tree trunk? Say its name.", "A monkey", "A frog", 0)),
                Kind::Answer("I see a monkey in the tree.".into(), q!("Which animal do I see? Write one word.", "Monkey", "Cat", 0));
                q!("Which animal held on to the tree trunk?", "A monkey", "A frog", 0),
                q!("‘A cat sits by a box.’ What is by the box?", "A frog", "A cat", 1),
                q!("Which animal name begins like fish?", "Frog", "Dog", 0),
                q!("‘A duck swims.’ What is the duck doing?", "Sitting", "Swimming", 1),
            ),
            lesson!("en1-cap-seller", "The Cap-seller and the Monkeys",
                "A man puts his caps by a tree. A monkey takes a cap up the tree. The cap falls down.",
                q!("What does the monkey take?", "A cap", "A cup", 0),
                Kind::Choice(q!("The monkey climbs from the ground to a branch. Which way does it go?", "Down", "Up", 1));
                q!("Which way does the cap fall?", "Up", "Down", 1),
                q!("Which word starts with the same sound as cap?", "Cat", "Mat", 0),
                q!("A ball rolls off a table. Which way does it go?", "Up", "Down", 1),
                q!("‘The girl picks up a hat.’ What does she pick up?", "A hat", "A bag", 0),
            ),
            lesson!("en1-farm", "A Farm",
                "A cow eats grass. A duck swims. The cow says moo.",
                q!("Which animal swims?", "The cow", "The duck", 1),
                Kind::Choice(q!("Say duck. Which word ends with the same sound?", "Bus", "Truck", 1));
                q!("What does the cow eat?", "Grass", "Rice", 0),
                q!("Which animal says moo?", "A duck", "A cow", 1),
                q!("Which word rhymes with hen?", "Pen", "Hat", 0),
                q!("‘The goat eats leaves.’ What does the goat eat?", "Grass", "Leaves", 1),
            ),
            lesson!("en1-food-pictures", "Fun with Pictures",
                Kind::Picture(Picture::Food, q!("Which fruit is cut open? Point to it and say its name.", "A banana", "A watermelon", 1)),
                Kind::Answer("I see yellow bananas.".into(), q!("What fruit do I see? Write one word.", "Bananas", "Apples", 0));
                q!("Which fruit was cut open in the picture?", "A watermelon", "A banana", 0),
                q!("‘A pear is in the bag.’ Which fruit is in the bag?", "Apple", "Pear", 1),
                q!("Which word starts with the same sound as peas?", "Pen", "Hen", 0),
                q!("‘The mango is yellow.’ Which word names its colour?", "Mango", "Yellow", 1),
            ),
            lesson!("en1-food", "The Food We Eat",
                "Ali has an idli. Tara has a banana. They sit together to eat.",
                q!("Who has an idli?", "Tara", "Ali", 1),
                Kind::Answer("Tara has a banana.".into(), q!("What does Tara have? Write one word.", "Banana", "Idli", 0));
                q!("What do the friends do together?", "Eat", "Run", 0),
                q!("‘I have rice.’ What food do I have?", "Roti", "Rice", 1),
                q!("Which word can finish ‘I like to eat ___’?", "Roti", "Chair", 0),
                q!("‘Sam has a pear.’ Who has the pear?", "Tara", "Sam", 1),
            ),
            lesson!("en1-seasons", "The Four Seasons",
                "In summer, the sun feels hot. In winter, I put on a warm cap.",
                q!("When do I put on a warm cap?", "In winter", "In summer", 0),
                Kind::Choice(q!("Say hot. Which word rhymes with hot?", "Hat", "Pot", 1));
                q!("How does the summer sun feel?", "Hot", "Cold", 0),
                q!("Which word rhymes with sun?", "Rain", "Fun", 1),
                q!("‘It is cold. I wear a coat.’ What do I wear?", "A coat", "A hat", 0),
                q!("Which season is usually hotter?", "Winter", "Summer", 1),
            ),
            lesson!("en1-rainbow", "Anandi’s Rainbow",
                "Rain stops. Anandi looks outside and sees a rainbow. She gets her paints.",
                q!("What does Anandi see?", "The moon", "A rainbow", 1),
                Kind::Answer("Anandi paints a red flower.".into(), q!("What colour is the flower? Write one word.", "Red", "Blue", 0));
                q!("What does Anandi get?", "Her paints", "Her books", 0),
                q!("‘The sky is blue.’ Which word names the colour?", "Sky", "Blue", 1),
                q!("‘Rain stops. I go out.’ What happens first?", "Rain stops.", "I go out.", 0),
                q!("Which word starts with the same sound as rain?", "Sun", "Red", 1),
            ),
        ]
    } else {
        // Mridang 2, bemr101–113. The practice progresses from details to inference.
        vec![
            topic(
                "en2-common-words",
                "Read common words in sentences",
                grade,
                group.clone(),
                [
                    (
                        "this-that",
                        Kind::Read(
                            "This book is in my hand. That book is across the room.".into(),
                            q!(
                                "Which word points to the book in my hand?",
                                "That",
                                "This",
                                1
                            ),
                        ),
                    ),
                    (
                        "like",
                        Kind::Read(
                            "I like to read. My friend likes to draw.".into(),
                            q!("What do I like to do?", "Read", "Draw", 0),
                        ),
                    ),
                    (
                        "or",
                        Kind::Read(
                            "Shall we take the bus or walk? We can choose.".into(),
                            q!("Which word shows we have a choice?", "Take", "Or", 1),
                        ),
                    ),
                ],
                [
                    q!(
                        "Complete the sentence ‘___ is the bag in my hand.’",
                        "This",
                        "Or",
                        0
                    ),
                    q!(
                        "Read ‘I like to swim.’ What do I like to do?",
                        "Draw",
                        "Swim",
                        1
                    ),
                    q!(
                        "Complete the choice ‘Will you have milk ___ water?’",
                        "or",
                        "to",
                        0
                    ),
                    q!(
                        "Read ‘That kite is far away.’ Which word points to the kite?",
                        "Far",
                        "That",
                        1
                    ),
                ],
            ),
            lesson!("en2-bicycle", "My Bicycle",
                "Nila rings her bicycle bell before she passes her friends. They hear it and leave room for her.",
                q!("Why does Nila ring the bell?", "To let her friends know she is coming", "To make her bicycle go faster", 0),
                Kind::Answer("Nila rides home on her blue bicycle.".into(), q!("What does Nila ride? Write a short answer.", "A bus", "A blue bicycle", 1));
                q!("What do her friends do when they hear the bell?", "Leave room", "Ring another bell", 0),
                q!("Read ‘Nila rides home.’ Which word tells what Nila does?", "Rides", "Home", 0),
                q!("Which word rhymes with bell?", "Shell", "Ball", 0),
                q!("‘I slowed down because the path was wet.’ Why did I slow down?", "The bell rang.", "The path was wet.", 1),
            ),
            lesson!("en2-picture-reading", "Picture Reading",
                Kind::Picture(Picture::Play, q!("One child covers his eyes. Another peeks from behind a tree. Which game are they likely playing?", "Skipping rope", "Hide-and-seek", 1)),
                Kind::Answer("A child peeks out from behind a tree.".into(), q!("Where is the child hiding? Write a short answer.", "Behind a tree", "Behind a door", 0));
                q!("What did one child cover during the game?", "His eyes", "His ears", 0),
                q!("‘I hide, then my friend looks for me.’ What comes after hiding?", "Counting", "Looking", 1),
                q!("Which question asks who did an action?", "Where is the tree?", "Who hid behind the tree?", 1),
                q!("Which sentence has a space between every word?", "Weplay a game.", "We play a game.", 1),
            ),
            lesson!("en2-it-is-fun", "It Is Fun",
                "I can hop like a frog. My sister can stretch like a cat. We take turns guessing each animal.",
                q!("Which animal does the speaker act like?", "A frog", "A cat", 0),
                Kind::Choice(q!("Which word completes the sentence ‘I can ___ like a bird’?", "Nest", "Flap", 1));
                q!("What do the two children take turns doing?", "Guessing the animals", "Feeding the animals", 0),
                q!("‘I swim like a fish.’ Which word tells what I do?", "Fish", "Swim", 1),
                q!("Which word rhymes with hop?", "Top", "Hot", 0),
                q!("What can a lamb do in ‘Leap like a lamb’?", "Leap", "Climb", 0),
            ),
            lesson!("en2-seeing", "Seeing without Seeing",
                "Anu closes her eyes. She feels a small object with corners and flat sides. Her friend tells her it is a block.",
                q!("How does Anu explore the object?", "By listening", "By touching", 1),
                Kind::Answer("Anu feels the block. It has flat sides.".into(), q!("What are the sides like? Write one word.", "Flat", "Round", 0));
                q!("What does Anu feel?", "A block", "A ball", 0),
                q!("‘I hear a bell with my eyes closed.’ What helps me notice it?", "Hearing", "Sight", 0),
                q!("Which word could describe a ball?", "Round", "Flat", 0),
                q!("‘Anu asks a friend for help.’ Who helps Anu?", "A friend", "A teacher", 0),
            ),
            lesson!("en2-come-back", "Come Back Soon",
                "Farah takes a train to visit her aunt. Her brother waves from the platform. He asks her to call when she gets there.",
                q!("Where is her brother when he waves?", "On the train", "On the platform", 1),
                Kind::Answer("Farah will call her brother when she arrives.".into(), q!("Who will Farah call? Write a short answer.", "Her brother", "Her aunt", 0));
                q!("How does Farah travel?", "By train", "By boat", 0),
                q!("Which word rhymes with train?", "Town", "Rain", 1),
                q!("Which question asks how someone will travel?", "When will you leave?", "Will you go by bus or by train?", 1),
                q!("Which place would you go to catch a train?", "A station", "A bus stop", 0),
            ),
            lesson!("en2-home-school", "Between Home and School",
                "Meera walks to school. First she passes a pond. Then she crosses a small bridge. The school is just past the bridge.",
                q!("What does Meera pass before she crosses the bridge?", "A pond", "The school", 0),
                Kind::Answer("First Meera passes the pond. Then she crosses the bridge.".into(), q!("What does she cross? Write a short answer.", "The pond", "The bridge", 1));
                q!("Where is the school?", "Just past the bridge", "Before the pond", 0),
                q!("Which word marks the first step in ‘First wash, then dry’?", "First", "Then", 0),
                q!("‘Ravi goes on foot.’ How does he travel?", "He rides.", "He walks.", 1),
                q!("‘Go past the shop, then turn left.’ What comes second?", "Pass the shop.", "Turn left.", 1),
            ),
            lesson!("en2-town", "This Is My Town",
                "My street has a shop and a park. At home, a basket sits on a chair. A small toy is inside the basket.",
                q!("Where is the toy?", "Inside the basket", "Under the chair", 0),
                Kind::Answer("A basket sits on a chair.".into(), q!("Where does the basket sit? Write a short answer.", "Under the chair", "On the chair", 1));
                q!("What two places are on the street?", "A shop and a park", "A school and a pond", 0),
                q!("‘The shoe is under the bed.’ Where should you look?", "On top of the bed", "Below the bed", 1),
                q!("Which sentence describes more than one tree?", "There is a tree.", "There are two trees.", 1),
                q!("Which question asks where the park is?", "What colour is the park gate?", "Where is the park?", 1),
            ),
            lesson!("en2-clouds", "A Show of Clouds",
                "One cloud looks like a rabbit to Kabir. Leela thinks it looks like a boat. The wind changes its shape.",
                q!("What does the cloud look like to Leela?", "A rabbit", "A boat", 1),
                Kind::Answer("The wind changes the shape of the cloud.".into(), q!("What changes the cloud’s shape? Write one or two words.", "The wind", "The sun", 0));
                q!("Which shape does Kabir see?", "A rabbit", "A boat", 0),
                q!("Which word rhymes with cloud?", "Cold", "Loud", 1),
                q!("‘The cloud looks like a fish.’ What is being compared to a fish?", "A cloud", "The wind", 0),
                q!("Why might Leela see a new shape later?", "The cloud changes shape.", "The rabbit moves.", 0),
            ),
            lesson!("en2-my-name", "My Name",
                "A little fly forgets its name. It asks an ant, then a cow. The cow says, ‘You are a fly!’",
                q!("Who helps the fly remember its name?", "The cow", "The ant", 0),
                Kind::Choice(q!("Which mark should end ‘What is my name’?", "A full stop (.)", "A question mark (?)", 1));
                q!("Who does the fly ask first?", "The ant", "The cow", 0),
                q!("Which sentence asks a question?", "Do you know my name?", "I know your name.", 0),
                q!("‘The duck forgets where its nest is.’ What has it forgotten?", "The place", "Its name", 0),
                q!("‘Mira asks her friend, then her dad.’ Who does she ask second?", "Her friend", "Her dad", 1),
            ),
            lesson!("en2-crow", "The Crow",
                "A crow finds a bright feather on the ground. He tucks it into his tail so that he can look like a peacock.",
                q!("Why does the crow put the feather in his tail?", "To make a nest", "To look like a peacock", 1),
                Kind::Answer("A crow finds a bright feather.".into(), q!("Which word describes the feather? Write one word.", "Bright", "Crow", 0));
                q!("Where does the crow find the feather?", "On the ground", "In his nest", 0),
                q!("Which word rhymes with tail?", "Pail", "Tell", 0),
                q!("‘The crow has a shiny feather.’ Which word describes the feather?", "Has", "Shiny", 1),
                q!("‘The bird puts a leaf in its nest to make it soft.’ Why does it add the leaf?", "To make the nest soft", "To hide its tail", 0),
            ),
            lesson!("en2-monkey", "The Smart Monkey",
                "Two friends eat groundnuts on a bench. One drops a wrapper. A monkey picks it up and puts it in a bin.",
                q!("What does the monkey put in the bin?", "A wrapper", "A groundnut", 0),
                Kind::Answer("The friends see what the monkey does.".into(), q!("Where should they put the next wrapper? Write a short answer.", "On the bench", "In the bin", 1));
                q!("Where are the friends sitting?", "On a bench", "In a tree", 0),
                q!("What happens before the monkey picks up the wrapper?", "It puts the wrapper in the bin.", "A friend drops the wrapper.", 1),
                q!("‘I put my rubbish in a bin.’ What does this help keep clean?", "The food inside my bag", "The place around me", 1),
                q!("Which word tells what the monkey did with the wrapper first?", "Picked up", "Ate", 0),
            ),
            lesson!("en2-water", "Little Drops of Water",
                "A tap drips into an empty cup. By lunchtime, the cup is full. Little drops can add up.",
                q!("Why is the cup full at lunchtime?", "Someone puts sand in it.", "Water keeps dripping into it.", 1),
                Kind::Answer("The tap keeps dripping after you wash your hands.".into(), q!("How can you stop wasting water? Write a short answer.", "Turn the tap off.", "Use another cup.", 0));
                q!("What goes into the cup little by little?", "Water", "Sand", 0),
                q!("Which word means the opposite of empty?", "Full", "Small", 0),
                q!("‘One drop, then another, fills the spoon.’ What fills the spoon?", "Drops of water", "The spoon itself", 0),
                q!("What could you do to save water while brushing?", "Turn the tap off.", "Leave the tap running.", 0),
            ),
            lesson!("en2-indians", "We Are All Indians",
                "Sana speaks Urdu at home. Leela speaks Malayalam. At school they teach each other how to say hello.",
                q!("What do Sana and Leela teach each other?", "How to say hello", "How to draw a map", 0),
                Kind::Answer("Leela speaks Malayalam at home.".into(), q!("Which language does Leela speak at home? Write the name.", "Urdu", "Malayalam", 1));
                q!("Which language does Sana speak at home?", "Urdu", "Malayalam", 0),
                q!("Which question asks about a language?", "What games do you play?", "What language do you speak at home?", 1),
                q!("Which name begins with a capital letter?", "sana", "Sana", 1),
                q!("A friend teaches you a new greeting. What is a good reply?", "Where is my bag?", "Thank you for teaching me.", 1),
            ),
        ]
    }
}
