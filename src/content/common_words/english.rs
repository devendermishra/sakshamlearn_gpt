use super::{card, Paper, Question, Topic};

const GROUP: (&str, Paper) = ("Everyday words", Paper::English);

pub(super) fn topics() -> Vec<Topic> {
    vec![
        card!("en1-daily-home", "Home and family", 1, ["home", "mother", "father", "room"],
            "At home, my mother and father sit in the room.";
            "Where do they sit?" => "In the room", "In the park", 0;
            "Which word names a parent?" => "Room", "Mother", 1;
            "Complete ‘I go ___ after school.’" => "home", "father", 0;
            "Which word starts with the same sound as fan?" => "Room", "Father", 1;
            "Complete ‘My bed is in my ___.’" => "room", "mother", 0),
        card!("en1-daily-school", "School and supplies", 1, ["school", "bag", "book", "pen"],
            "I put my book and pen in my bag. I take my bag to school.";
            "Where do I put my pen?" => "In my bag", "On my chair", 0;
            "Which word names something to read?" => "Book", "Pen", 0;
            "Complete ‘I write with a ___.’" => "bag", "pen", 1;
            "Which word rhymes with tag?" => "Bag", "Book", 0;
            "Complete ‘I take my bag to ___.’" => "pen", "school", 1),
        card!("en1-daily-food", "Food and drink", 1, ["water", "milk", "rice", "apple"],
            "I drink water and milk. I eat rice and an apple.";
            "Which fruit do I eat?" => "Rice", "An apple", 1;
            "Which word names a drink?" => "Milk", "Rice", 0;
            "Complete ‘I filled my bottle with ___.’" => "rice", "water", 1;
            "Which word starts with the same sound as red?" => "Rice", "Milk", 0;
            "Complete ‘I ate an ___.’" => "water", "apple", 1),
        card!("en1-daily-actions", "Everyday actions", 1, ["eat", "drink", "read", "play"],
            "I eat rice and drink water. I read a book. Then I play.";
            "What do I do with the book?" => "Read it", "Drink it", 0;
            "Complete ‘I ___ water.’" => "play", "drink", 1;
            "Which word goes with ‘a story’?" => "Read", "Eat", 0;
            "Which word can go before ‘with a ball’?" => "Drink", "Play", 1;
            "Complete ‘I ___ an apple.’" => "eat", "read", 0),
        card!("en1-daily-describe", "Colours and sizes", 1, ["red", "blue", "big", "small"],
            "I have a big red ball and a small blue ball.";
            "What colour is the small ball?" => "Blue", "Red", 0;
            "Which word tells the size of an elephant?" => "Red", "Big", 1;
            "Which word means the opposite of big?" => "Small", "Blue", 0;
            "Which word names a colour?" => "Small", "Red", 1;
            "Complete ‘The clear sky looks ___.’" => "blue", "big", 0),
        card!("en1-daily-kind-words", "Polite words", 1, ["hello", "please", "thanks", "sorry"],
            "I say hello to my friend. I say please when I ask, thanks when she helps, and sorry if I bump into her.";
            "What do I say when I ask?" => "Please", "Sorry", 0;
            "What can you say when you meet a friend?" => "Sorry", "Hello", 1;
            "What can you say when a friend helps you?" => "Thanks", "Hello", 0;
            "You step on a friend's foot by accident. What can you say?" => "Please", "Sorry", 1;
            "Which word makes ‘May I join you?’ more polite?" => "Please", "Hello", 0),
        card!("en2-daily-questions", "Ask a question", 2, ["who", "what", "where", "when"],
            "Who is at the door? What is in the bag? Where is my book? When will we leave?";
            "Which word asks about a time?" => "Where", "When", 1;
            "Complete ‘___ is my pen?’ The answer is ‘On the desk.’" => "Where", "Who", 0;
            "Which word asks about a person?" => "What", "Who", 1;
            "Complete ‘___ is in the box?’ The answer is ‘A ball.’" => "What", "When", 0;
            "Complete ‘___ does school start?’ The answer is ‘At nine.’" => "Where", "When", 1),
        card!("en2-daily-positions", "Find things", 2, ["under", "near", "between", "behind"],
            "My shoes are under the bench. The bin is near the door. The ball is between two bags. I stand behind my friend.";
            "Where is the ball?" => "Behind the door", "Between two bags", 1;
            "Where are the shoes?" => "Under the bench", "Near the door", 0;
            "Which word means close to?" => "Behind", "Near", 1;
            "Complete ‘The middle chair is ___ two chairs.’" => "between", "under", 0;
            "Complete ‘I stood ___ my friend, at her back.’" => "near", "behind", 1),
        card!("en2-daily-time", "Time and sequence", 2, ["today", "tomorrow", "before", "after"],
            "Today I will finish my work before lunch. After lunch I will play. Tomorrow I will bring my ball.";
            "When will I bring my ball?" => "Tomorrow", "Today", 0;
            "Which means the day after today?" => "Before", "Tomorrow", 1;
            "Complete ‘I wash my hands ___ I eat.’" => "before", "tomorrow", 0;
            "I finish eating, then play. Which word goes in ‘I play ___ lunch’?" => "before", "after", 1;
            "Complete ‘Yesterday was Monday, so ___ is Tuesday.’" => "today", "tomorrow", 0),
        card!("en2-daily-people", "Words for people", 2, ["he", "she", "his", "her"],
            "This is Ravi. He has his bag. This is Meera. She has her book.";
            "Which word stands in for Meera?" => "He", "She", 1;
            "Which word stands in for Ravi?" => "He", "Her", 0;
            "Complete ‘Ravi opens ___ bag.’" => "he", "his", 1;
            "Complete ‘Meera reads ___ book.’" => "her", "she", 0;
            "Complete ‘Meera has a pen. ___ writes with it.’" => "His", "She", 1),
        card!("en2-daily-connect", "Join and explain ideas", 2, ["and", "but", "because", "then"],
            "I took a book and a pen. I wanted to go out, but it rained. I stayed in because it was wet. Then I read.";
            "Which word introduces the reason I stayed in?" => "Because", "Then", 0;
            "Complete ‘I took a book ___ a pen.’" => "but", "and", 1;
            "Complete ‘I stayed in ___ it was wet.’" => "because", "then", 0;
            "Complete ‘I want to play, ___ it is too late.’" => "and", "but", 1;
            "Which word can mark the next thing that happens?" => "Then", "Because", 0),
    ]
}
