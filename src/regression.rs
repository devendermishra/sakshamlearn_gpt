use crate::{
    content::{self, CustomCatalog, Kind, Paper, PracticeSkill, Question},
    language::Language,
    progress::{Input, Outcome, Step},
    state::{self, HelperDraft, Pin, Session, State},
};

#[test]
fn english_mathematics_preserves_content_structure_and_progress_keys() {
    let check_english = |text: &str| {
        assert!(!text.is_empty());
        assert!(
            !text
                .chars()
                .any(|ch| ('\u{0900}'..='\u{097f}').contains(&ch)),
            "{text}"
        );
    };
    let check_question = |hindi: &Question, english: &Question| {
        check_english(&english.prompt);
        assert_eq!(hindi.options, english.options);
        assert_eq!(hindi.answer, english.answer);
    };
    for hindi in &bundled("math").topics {
        let english = content::english_math_topic(hindi);
        english.validate().unwrap();
        assert_eq!(hindi.progress_id(), english.progress_id());
        assert_eq!(hindi.grade, english.grade);
        assert_eq!(hindi.group.id, english.group.id);
        assert_eq!(hindi.group.separate_page, english.group.separate_page);
        assert_eq!(hindi.break_every, english.break_every);
        check_english(&english.title);
        check_english(&english.group.title);
        assert_eq!(hindi.exercises.len(), english.exercises.len());
        for (original, translated) in hindi.exercises.iter().zip(&english.exercises) {
            assert_eq!(original.id, translated.id);
            match (&original.kind, &translated.kind) {
                (Kind::Choice(original), Kind::Choice(translated)) => {
                    check_question(original, translated)
                }
                (Kind::Read(_, original), Kind::Read(passage, translated)) => {
                    check_english(passage);
                    check_question(original, translated);
                }
                (
                    Kind::Count {
                        quantity,
                        visual,
                        question,
                    },
                    Kind::Count {
                        quantity: translated_quantity,
                        visual: translated_visual,
                        question: translated,
                    },
                ) => {
                    assert_eq!(quantity, translated_quantity);
                    assert_eq!(visual, translated_visual);
                    check_question(question, translated);
                }
                (
                    Kind::Trace { model, paper, .. },
                    Kind::Trace {
                        model: translated_model,
                        paper: translated_paper,
                        guidance,
                    },
                ) => {
                    assert_eq!(model, translated_model);
                    assert_eq!(paper, translated_paper);
                    check_english(guidance);
                }
                (
                    Kind::Solve {
                        lines,
                        answer,
                        helper,
                        notebook,
                        ..
                    },
                    Kind::Solve {
                        prompt,
                        solution,
                        lines: translated_lines,
                        answer: translated_answer,
                        helper: translated_helper,
                        notebook: translated_notebook,
                    },
                ) => {
                    assert_eq!(lines, translated_lines);
                    assert!(!answer.is_empty());
                    assert_eq!(answer, translated_answer);
                    assert_eq!(helper, translated_helper);
                    assert_eq!(notebook, translated_notebook);
                    check_english(prompt);
                    check_english(solution);
                }
                _ => panic!("exercise kind changed in {}", hindi.id),
            }
        }
        assert_eq!(hindi.recall.len(), english.recall.len());
        for (original, translated) in hindi.recall.iter().zip(&english.recall) {
            check_question(original, translated);
        }
    }
    let mut state: State = serde_json::from_str("{}").unwrap();
    assert_eq!(state.math_language, Language::Hindi);
    state.math_language = Language::English;
    let reopened: State = serde_json::from_value(serde_json::to_value(state).unwrap()).unwrap();
    assert_eq!(reopened.math_language, Language::English);
}
use std::{collections::BTreeSet, fs};

fn bundled(id: &str) -> &'static content::Subject {
    content::BUNDLED
        .iter()
        .find(|subject| subject.id == id)
        .unwrap()
}

fn begin(state: &mut State, step: &Step<'_>) {
    state.sessions.insert(
        step.session_key(),
        Session {
            step: step.index,
            ..Session::new(step.subject.into(), step.topic.progress_id(), step.review)
        },
    );
}

#[test]
fn generated_math_rounds_are_grade_specific_and_bilingual() {
    assert!(content::generate_practice(1, PracticeSkill::Tables, 1).is_none());
    for grade in [1, 2] {
        for &skill in PracticeSkill::for_grade(grade) {
            for seed in 0..24 {
                let set = content::generate_practice(grade, skill, seed).unwrap();
                let hindi = set.topic(Language::Hindi);
                let english = set.topic(Language::English);
                assert_eq!(hindi.grade, grade);
                assert_eq!(hindi.id, skill.id(grade));
                assert_eq!(hindi.exercises.len(), 5);
                assert_eq!(hindi.recall.len(), 1);
                hindi.validate().unwrap();
                english.validate().unwrap();
                assert_eq!(hindi.recall[0].answer, english.recall[0].answer);
                assert_eq!(hindi.recall[0].options, english.recall[0].options);
                for (left, right) in hindi.exercises.iter().zip(&english.exercises) {
                    assert_eq!(left.id, right.id);
                    match (&left.kind, &right.kind) {
                        (Kind::Choice(a), Kind::Choice(b))
                        | (Kind::Count { question: a, .. }, Kind::Count { question: b, .. }) => {
                            assert_eq!(a.answer, b.answer);
                            assert_eq!(a.options, b.options);
                        }
                        (
                            Kind::Solve {
                                answer: a, helper, ..
                            },
                            Kind::Solve {
                                answer: b,
                                helper: translated,
                                ..
                            },
                        ) => {
                            assert_eq!(a, b);
                            assert_eq!(helper, translated);
                            if let Some(help) = helper {
                                assert!(help.answer() <= if grade == 1 { 20 } else { 99 });
                                if grade == 2 {
                                    let regrouping = matches!(
                                        skill,
                                        PracticeSkill::Carrying | PracticeSkill::Borrowing
                                    );
                                    assert_eq!(
                                        help.notebook_paper().regrouping_cells() > 0,
                                        regrouping
                                    );
                                }
                            }
                        }
                        _ => panic!("generated exercise changed with the language"),
                    }
                }
            }
        }
    }
}

#[test]
fn generated_set_and_helper_work_resume_without_rerolling() {
    let mut state = State::default();
    let id = state
        .start_generated(2, PracticeSkill::Borrowing, 10)
        .unwrap();
    let key = state::session_key("math", &id, false);
    let set = state.sessions[&key].generated.clone().unwrap();
    let topic = set.topic(Language::English);
    let step = Step {
        subject: "math",
        topic,
        review: false,
        index: 0,
    };
    let pen = vec![vec![vec![[0.2, 0.2], [0.2, 0.8]]]];
    let helper = HelperDraft {
        first: Some(pen.clone()),
        borrowed_tens: true,
        borrowed_ones: true,
        ..Default::default()
    };
    assert_eq!(
        step.apply(&mut state, Input::Helper(helper.clone()), 100, ""),
        Some(Outcome::Continue)
    );
    assert_eq!(
        step.apply(&mut state, Input::Draft(pen.clone()), 100, ""),
        Some(Outcome::Continue)
    );

    let mut resumed: State = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    assert_eq!(
        resumed
            .start_generated(2, PracticeSkill::Borrowing, 11)
            .as_deref(),
        Some(id.as_str())
    );
    let session = &resumed.sessions[&key];
    assert_eq!(session.step, 0);
    assert_eq!(session.helper.as_ref(), Some(&helper));
    assert_eq!(
        session.generated.as_ref().unwrap().topic(Language::English),
        topic
    );
    assert_eq!(step.ink(&resumed), pen);
    let translated = Step {
        topic: set.topic(Language::Hindi),
        ..step
    };
    assert_eq!(translated.ink(&resumed), pen);
    for index in 0..topic.exercises.len() {
        let step = Step {
            index,
            ..translated
        };
        assert!(matches!(
            step.apply(
                &mut resumed,
                Input::SubmitSolution {
                    ink: pen.clone(),
                    correct: true
                },
                100,
                "2026-11-12"
            ),
            Some(Outcome::Continue | Outcome::Break)
        ));
        assert!(resumed.sessions[&key].helper.is_none());
    }
    let last = Step {
        index: topic.exercises.len(),
        ..translated
    };
    assert_eq!(
        last.apply(&mut resumed, Input::Answer(false), 100, ""),
        Some(Outcome::Done(false))
    );
    assert!(!resumed.sessions.contains_key(&key));
    assert!(resumed.finished.is_empty() && resumed.reviews.is_empty());
    resumed
        .start_generated(2, PracticeSkill::Borrowing, 11)
        .unwrap();
    let fresh = resumed.sessions[&key]
        .generated
        .as_ref()
        .unwrap()
        .topic(Language::English);
    assert_ne!(fresh.exercises[0].id, topic.exercises[0].id);
    assert_ne!(fresh.exercises[0].kind, topic.exercises[0].kind);
}

#[test]
fn notebook_math_uses_a_single_answer_row_and_keeps_the_handwritten_result() {
    let topics = &bundled("math").topics;
    for grade in [1, 2] {
        for operation in ["add", "subtract"] {
            let id = format!("notebook-{operation}-{grade}");
            let topic = topics.iter().find(|topic| topic.id == id).unwrap();
            assert_eq!(topic.grade, grade);
            assert_eq!(topic.exercises.len(), 3);
            assert!(topic.exercises.iter().all(|exercise| {
                let writing = exercise.kind.writing(Paper::Hindi).unwrap();
                writing.paper == Paper::Notebook && writing.lines == 1 && writing.model.is_empty()
            }));
        }
    }
    let topic = topics
        .iter()
        .find(|topic| topic.id == "notebook-add-1")
        .unwrap();
    let step = Step {
        subject: "math",
        topic,
        review: false,
        index: 0,
    };
    let mut state = State::default();
    begin(&mut state, &step);
    let ink = vec![vec![], vec![vec![[0.2, 0.2], [0.8, 0.2], [0.4, 0.8]]]];
    assert_eq!(
        step.apply(
            &mut state,
            Input::SubmitSolution {
                ink: ink.clone(),
                correct: true
            },
            100,
            "2026-11-11"
        ),
        Some(Outcome::Continue)
    );
    let sample = &state.history["math/notebook-add-1/item-1"][0];
    assert_eq!(sample.paper, Paper::Notebook);
    assert_eq!(sample.ink, ink);
    assert_eq!(sample.solution.as_deref(), Some("3 + 4 = 7"));
}

#[test]
fn notebook_regrouping_has_separate_space_for_carry_and_borrow_notes() {
    let topics = &bundled("math").topics;
    for (id, paper, cells, answers) in [
        ("notebook-carry-2", Paper::NotebookCarry, 1, [65, 73, 82]),
        ("notebook-borrow-2", Paper::NotebookBorrow, 3, [34, 37, 37]),
    ] {
        let topic = topics.iter().find(|topic| topic.id == id).unwrap();
        assert_eq!(topic.grade, 2);
        assert_eq!(topic.exercises.len(), answers.len());
        for (exercise, expected) in topic.exercises.iter().zip(answers) {
            let writing = exercise.kind.writing(Paper::Hindi).unwrap();
            assert_eq!(writing.paper, paper);
            assert_eq!(writing.paper.regrouping_cells(), cells);
            assert_eq!(writing.lines, 1);
            let Kind::Solve {
                answer,
                helper: Some(helper),
                ..
            } = &exercise.kind
            else {
                panic!("expected a notebook problem with regrouping");
            };
            assert_eq!(answer, &[expected]);
            assert_eq!(helper.answer(), expected);
        }
    }
}

#[test]
fn handwritten_math_saves_a_draft_and_a_dated_solution_for_review() {
    let topic = bundled("math")
        .topics
        .iter()
        .find(|topic| topic.id == "write-add-2")
        .unwrap();
    let step = Step {
        subject: "math",
        topic,
        review: false,
        index: 0,
    };
    let mut state = State::default();
    begin(&mut state, &step);
    assert_eq!(
        step.apply(&mut state, Input::Submit(Vec::new()), 100, ""),
        None
    );

    let mut ink = vec![Vec::new(); 30];
    ink[14].push(vec![[0.1, 0.2], [0.3, 0.4]]);
    assert_eq!(
        step.apply(&mut state, Input::Draft(ink.clone()), 100, ""),
        Some(Outcome::Continue)
    );
    assert_eq!(step.ink(&state), ink);
    assert!(state.history.is_empty());
    let english = Step {
        topic: content::english_math_topic(topic),
        ..step
    };
    assert_eq!(english.ink(&state), ink);
    assert_eq!(
        english.apply(
            &mut state,
            Input::SubmitSolution {
                ink: ink.clone(),
                correct: true
            },
            100,
            "2026-11-11"
        ),
        Some(Outcome::Continue)
    );
    let sample = &state.history["math/write-add-2/item-1"][0];
    assert_eq!(sample.paper, Paper::Digit);
    assert_eq!(sample.lines, 3);
    assert!(sample.model.is_empty());
    assert_eq!(sample.solution.as_deref(), Some("23 + 14 = 37"));
    assert_eq!(sample.correct, Some(true));
    assert_eq!(sample.saved_on.as_deref(), Some("2026-11-11"));
    assert_eq!(sample.ink, ink);
    assert!(state.samples.is_empty());
}

#[test]
fn bundled_catalog_and_hindi_recall_are_valid() {
    let mut subject_ids = BTreeSet::new();
    for subject in content::BUNDLED.iter() {
        assert!(subject_ids.insert(&subject.id));
        let mut topic_ids = BTreeSet::new();
        for topic in &subject.topics {
            assert!(topic_ids.insert(&topic.id));
            assert!(topic.validate().is_ok(), "Invalid topic {}", topic.id);
            let exercise_ids: BTreeSet<_> = topic
                .exercises
                .iter()
                .map(|exercise| &exercise.id)
                .collect();
            assert_eq!(exercise_ids.len(), topic.exercises.len());
        }
    }

    let rows: Vec<_> = bundled("hi")
        .topics
        .iter()
        .filter(|topic| topic.group.id == "barakhadi")
        .collect();
    assert_eq!(rows.len(), 37);
    for topic in rows {
        assert_eq!(topic.exercises.len(), 12);
        assert_eq!(topic.break_every, Some(3));
        let covered: BTreeSet<_> = [None, Some(0), Some(1), Some(2)]
            .into_iter()
            .flat_map(|interval| content::review_questions(topic, interval))
            .map(|question| &question.prompt)
            .collect();
        assert_eq!(covered.len(), 12);
    }
}

#[test]
fn english_has_writing_plans_and_distinct_recall_for_both_classes() {
    let topics = &bundled("en").topics;
    for (grade, reading_count) in [(1, 11), (2, 14)] {
        let class: Vec<_> = topics.iter().filter(|topic| topic.grade == grade).collect();
        for (group, count) in [
            ("reading", reading_count),
            ("word-sounds", if grade == 1 { 2 } else { 1 }),
            ("common-words", 2),
            ("word-writing", if grade == 1 { 2 } else { 1 }),
            ("lowercase", if grade == 1 { 13 } else { 0 }),
            ("conversation", 2),
            ("writing-plan", 5),
        ] {
            assert_eq!(
                class.iter().filter(|topic| topic.group.id == group).count(),
                count
            );
        }
        for topic in class {
            assert_eq!(topic.break_every, Some(2));
            let prompts: BTreeSet<_> = [None, Some(0), Some(1), Some(2)]
                .into_iter()
                .flat_map(|interval| content::review_questions(topic, interval))
                .map(|question| &question.prompt)
                .collect();
            assert_eq!(prompts.len(), 4, "{}", topic.id);
        }
    }
    let alphabet: BTreeSet<_> = topics
        .iter()
        .filter(|topic| topic.group.id == "alphabet")
        .flat_map(|topic| &topic.exercises)
        .map(|exercise| match &exercise.kind {
            Kind::Trace {
                model,
                paper: Paper::English,
                ..
            } => {
                assert_eq!(model.len(), 1);
                model.chars().next().unwrap()
            }
            _ => panic!("The alphabet exercise needs an English trace model"),
        })
        .collect();
    assert_eq!(alphabet, ('A'..='Z').chain('a'..='z').collect());
    let lowercase: Vec<_> = topics
        .iter()
        .filter(|topic| topic.group.id == "lowercase")
        .flat_map(|topic| &topic.exercises)
        .filter_map(|exercise| match &exercise.kind {
            Kind::Trace {
                model,
                paper: Paper::English,
                ..
            } => model.chars().next(),
            _ => None,
        })
        .collect();
    assert_eq!(lowercase, ('a'..='z').collect::<Vec<_>>());

    let class_two_lessons: Vec<_> = topics
        .iter()
        .filter(|topic| {
            topic.grade == 2 && topic.group.id == "reading" && topic.id != "en2-common-words"
        })
        .collect();
    for interval in 0..3 {
        let positions: BTreeSet<_> = class_two_lessons
            .iter()
            .map(|topic| content::review_questions(topic, Some(interval))[0].answer)
            .collect();
        assert_eq!(positions, BTreeSet::from([0, 1]));
    }

    for id in [
        "en1-picture-time",
        "en1-food-pictures",
        "en2-picture-reading",
    ] {
        let topic = topics.iter().find(|topic| topic.id == id).unwrap();
        let Kind::Picture(picture, _) = topic.exercises[0].kind else {
            panic!("{id} must open with picture reading");
        };
        let (src, alt) = picture.illustration();
        assert!(src.starts_with("data:image/jpeg;base64,/9j/"));
        assert!(!alt.is_empty());
    }
}

#[test]
fn english_writing_resumes_and_is_saved_on_four_line_paper() {
    let topic = bundled("en")
        .topics
        .iter()
        .find(|topic| topic.id == "en2-write-5")
        .unwrap();
    let step = Step {
        subject: "en",
        topic,
        review: false,
        index: 0,
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let mut state = state::load(&path).unwrap();
    state.sessions.insert(
        step.session_key(),
        Session::new("en".into(), topic.id.clone(), false),
    );
    let ink = vec![vec![vec![[0.2, 0.3]]], vec![vec![[0.3, 0.4]]]];
    assert_eq!(
        step.apply(&mut state, Input::Draft(ink.clone()), 100, "2026-11-12"),
        Some(Outcome::Continue)
    );
    state::save(&path, &state).unwrap();
    let mut state = state::load(&path).unwrap();
    assert_eq!(step.ink(&state), ink);
    assert_eq!(
        step.apply(
            &mut state,
            Input::Submit(vec![ink[0].clone(), Vec::new()]),
            100,
            "2026-11-12"
        ),
        None
    );
    assert_eq!(state.sessions[&step.session_key()].step, 0);
    assert_eq!(step.ink(&state), ink);
    assert_eq!(
        step.apply(&mut state, Input::Submit(ink.clone()), 100, "2026-11-12"),
        Some(Outcome::Continue)
    );
    state::save(&path, &state).unwrap();
    let state = state::load(&path).unwrap();
    let sample = &state.history["en/en2-write-5/model"][0];
    assert_eq!(sample.paper, Paper::English);
    assert_eq!(sample.lines, 2);
    assert_eq!(sample.ink, ink);
    assert_eq!(sample.saved_on.as_deref(), Some("2026-11-12"));
    let revision = Step { index: 1, ..step };
    assert_eq!(revision.revision_source(&state), Some((&ink, 2)));
    assert!(revision.ink(&state).is_empty());
}

#[test]
fn legacy_hindi_save_preserves_settings_progress_and_writing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let mut value = serde_json::to_value(State::default()).unwrap();
    value.as_object_mut().unwrap().remove("catalog");
    value["session"] =
        serde_json::json!({"subject":"hi","topic":"neema","step":2,"review":false,"missed":true});
    value["finished"] = serde_json::json!(["hi/neema"]);
    value["reviews"] = serde_json::json!({"hi/neema":{"started":100,"due":101,"interval":0}});
    value["samples"] = serde_json::json!({"hi/neema/answer":{"label":"Earlier writing","model":"","ink":[[[[0.2,0.3]]]]}});
    value["scale"] = serde_json::json!(4.2);
    value["pin"] = serde_json::to_value(Pin::new("1324")).unwrap();
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();

    let state = state::load(&path).unwrap();
    assert!(state.catalog.subjects.is_empty());
    assert!(state.catalog.additions.is_empty());
    assert_eq!(state.sessions["hi/neema/practice"].step, 2);
    assert!(state.finished.contains("hi/neema"));
    assert_eq!(state.due_review_count(101), 1);
    assert_eq!(state.history["hi/neema/answer"][0].paper, Paper::Hindi);
    assert_eq!(state.scale, Some(4.2));
    assert!(state.pin.unwrap().matches("1324"));
}

#[test]
fn authored_content_publishes_atomically_and_revisions_keep_prior_progress() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let original = State::default();
    let mut catalog = CustomCatalog::default();
    catalog.create_subject("Practice").unwrap();
    let subject = catalog.subjects[0].id.clone();
    let mut copy = bundled("hi")
        .topics
        .iter()
        .find(|topic| topic.id == "neema")
        .unwrap()
        .clone();
    copy.title.clear();
    assert!(catalog.publish(&subject, copy.clone()).is_err());
    copy.title = "A reading activity".into();
    catalog.publish(&subject, copy).unwrap();
    assert!(state::save_catalog(dir.path(), &original, catalog.clone()).is_err());
    assert!(original.catalog.subjects.is_empty());

    state::save_catalog(&path, &original, catalog).unwrap();
    let mut state = state::load(&path).unwrap();
    let mut activity = state.catalog.additions[&subject][0].clone();
    assert_ne!(activity.id, "neema");
    assert!(matches!(
        activity.exercises.last().unwrap().kind,
        Kind::Answer(..)
    ));
    let old_key = format!("{subject}/{}", activity.progress_id());
    state.finish(&old_key, 100);
    let mut catalog = state.catalog.clone();
    activity.title = "A new title".into();
    catalog.publish(&subject, activity).unwrap();
    assert_eq!(catalog.additions[&subject][0].revision, 0);

    let mut activity = catalog.additions[&subject][0].clone();
    activity
        .recall
        .push(Question::pair("New prompt", "Yes", "No", 0));
    catalog.publish(&subject, activity).unwrap();
    let next = state::save_catalog(&path, &state, catalog).unwrap();
    assert_eq!(next.catalog.additions[&subject][0].revision, 1);
    assert!(next.finished.contains(&old_key));
    assert_eq!(next.due_review_count(101), 0);
    assert_eq!(state::load(&path).unwrap().catalog, next.catalog);
}

#[test]
fn mathematics_counts_and_writes_every_digit_and_schedules_reviews() {
    let mut state = State::default();
    let mut seen = BTreeSet::new();
    for (start, end) in [(1, 3), (4, 6), (7, 9)] {
        let id = format!("numbers-{start}-{end}");
        let topic = bundled("math")
            .topics
            .iter()
            .find(|topic| topic.id == id)
            .unwrap();
        assert_eq!(topic.grade, 1);
        assert_eq!(topic.progress_id(), id);
        let step = |index| Step {
            subject: "math",
            topic,
            review: false,
            index,
        };
        begin(&mut state, &step(0));
        for (index, exercise) in topic.exercises.iter().enumerate() {
            let action = match &exercise.kind {
                Kind::Count {
                    quantity, question, ..
                } => {
                    assert_eq!(question.options[question.answer], quantity.to_string());
                    assert_ne!(question.options[1 - question.answer], quantity.to_string());
                    assert!(seen.insert(*quantity));
                    Input::Answer(index == 0)
                }
                Kind::Trace { paper, model, .. } => {
                    assert_eq!(*paper, Paper::Digit);
                    assert!(seen.contains(&model.parse::<u8>().unwrap()));
                    let ink = vec![vec![vec![[0.1, 0.2]]]];
                    assert_eq!(
                        step(index).apply(&mut state, Input::Draft(ink.clone()), 100, ""),
                        Some(Outcome::Continue)
                    );
                    assert_eq!(step(index).ink(&state), ink);
                    Input::Submit(ink)
                }
                _ => panic!("unexpected Mathematics exercise"),
            };
            assert_eq!(
                step(index).apply(&mut state, action, 100, "2026-11-11"),
                Some(Outcome::Continue)
            );
        }
        let last = step(topic.exercises.len());
        assert_eq!(
            last.apply(&mut state, Input::Answer(false), 100, ""),
            Some(Outcome::Done(true))
        );
        assert_eq!(state.reviews[&last.key()].due, 101);
        assert_eq!(last.apply(&mut state, Input::Answer(false), 100, ""), None);
    }
    assert_eq!(seen, (1..=9).collect());
    assert_eq!(state.history.len(), 9);
    assert!(state.samples.is_empty());
    assert!(state
        .history
        .values()
        .flatten()
        .all(|sample| sample.paper == Paper::Digit));
}

#[test]
fn mathematics_covers_both_grades_and_all_table_facts() {
    let topics = &bundled("math").topics;
    for grade in [1, 2] {
        for group in ["numbers", "add-subtract"] {
            assert!(topics
                .iter()
                .any(|topic| topic.grade == grade && topic.group.id == group));
        }
    }
    for size in 1..=10 {
        let id = format!("table-{size}");
        let topic = topics.iter().find(|topic| topic.id == id).unwrap();
        assert_eq!(topic.grade, 2);
        assert_eq!(topic.break_every, Some(3));
        assert_eq!(topic.exercises.len(), 10);
        let mut answer_positions = BTreeSet::new();
        let mut distractors_above = BTreeSet::new();
        for (index, exercise) in topic.exercises.iter().enumerate() {
            let groups = index + 1;
            let question = match &exercise.kind {
                Kind::Choice(question) | Kind::Read(_, question) => question,
                _ => panic!("expected multiplication question"),
            };
            assert_eq!(question.prompt, format!("{size} × {groups} = __"));
            assert_eq!(
                question.options[question.answer],
                (size * groups).to_string()
            );
            answer_positions.insert(question.answer);
            let wrong: usize = question.options[1 - question.answer].parse().unwrap();
            distractors_above.insert(wrong > size * groups);
        }
        assert_eq!(answer_positions.len(), 2);
        assert_eq!(distractors_above.len(), 2);
    }
    let mut division_positions = BTreeSet::new();
    let mut division_distractors = BTreeSet::new();
    for topic in topics.iter().filter(|topic| topic.group.id == "division") {
        for question in topic
            .recall
            .iter()
            .chain(
                topic
                    .exercises
                    .iter()
                    .filter_map(|exercise| match &exercise.kind {
                        Kind::Choice(question) => Some(question),
                        _ => None,
                    }),
            )
            .filter(|question| question.prompt.contains('÷'))
        {
            division_positions.insert(question.answer);
            let correct: u8 = question.options[question.answer].parse().unwrap();
            let wrong: u8 = question.options[1 - question.answer].parse().unwrap();
            division_distractors.insert(wrong > correct);
        }
    }
    assert_eq!(division_positions.len(), 2);
    assert_eq!(division_distractors.len(), 2);
    for topic in topics
        .iter()
        .filter(|topic| !topic.id.starts_with("numbers-"))
    {
        let prompts: BTreeSet<_> = [None, Some(0), Some(1), Some(2)]
            .into_iter()
            .flat_map(|interval| content::review_questions(topic, interval))
            .map(|question| &question.prompt)
            .collect();
        assert_eq!(prompts.len(), 4, "{}", topic.id);
    }
}

#[test]
fn mathematics_number_order_and_division_answers_match_worked_examples() {
    let topic = |id| {
        bundled("math")
            .topics
            .iter()
            .find(|topic| topic.id == id)
            .unwrap()
    };
    for (id, index, expected) in [
        ("count-to-twenty", 2, "0"),
        ("count-to-100", 1, "49"),
        ("count-to-100", 2, "100"),
        ("neighbors-to-twenty", 3, "0"),
        ("neighbors-to-twenty", 4, "20"),
        ("neighbors-to-100", 2, "39"),
        ("neighbors-to-100", 3, "99"),
        ("neighbors-to-100", 4, "99"),
        ("neighbors-to-100", 5, "1"),
    ] {
        let question = match &topic(id).exercises[index].kind {
            Kind::Choice(question) | Kind::Read(_, question) => question,
            _ => panic!("expected number question"),
        };
        assert_eq!(question.options[question.answer], expected, "{id}");
    }
    for (id, divisor, quotient) in [("division-take-away", 3, 4), ("division-practice", 4, 4)] {
        let topic = topic(id);
        let Kind::Read(example, _) = &topic.exercises[0].kind else {
            panic!("expected worked division example");
        };
        let sequence = example.split('।').nth(1).unwrap();
        let amounts: Vec<_> = sequence
            .split('→')
            .map(|value| value.trim().parse::<u8>().unwrap())
            .collect();
        assert_eq!(amounts.first(), Some(&(divisor * quotient)));
        assert_eq!(amounts.last(), Some(&0));
        assert_eq!(amounts.len(), usize::from(quotient) + 1);
        assert!(amounts.windows(2).all(|pair| pair[0] - pair[1] == divisor));
        assert!(example.contains(&format!("÷ {divisor} = {quotient}")));
    }
}

#[test]
fn mathematics_practises_ascending_and_descending_order_in_both_grades() {
    let topics = &bundled("math").topics;
    for (grade, length, maximum) in [(1, 3, 99), (2, 4, 100)] {
        let id = format!("number-order-{grade}");
        let topic = topics.iter().find(|topic| topic.id == id).unwrap();
        assert_eq!(topic.grade, grade);
        assert_eq!(topic.group.id, "numbers");
        assert_eq!(topic.break_every, Some(3));
        let exercises: Vec<_> = topic
            .exercises
            .iter()
            .map(|exercise| match &exercise.kind {
                Kind::Read(_, question) | Kind::Choice(question) => question,
                _ => panic!("expected ordering question"),
            })
            .collect();
        assert_eq!(exercises.len(), 6);
        assert_eq!(topic.recall.len(), 4);

        for (index, question) in exercises.into_iter().chain(&topic.recall).enumerate() {
            let (direction, numbers) = question.prompt.split_once("। संख्याएँ: ").unwrap();
            let parse = |list: &str, separator| {
                list.split(separator)
                    .map(|part| part.parse::<u8>().unwrap())
                    .collect::<Vec<_>>()
            };
            let mut expected = parse(numbers, ", ");
            assert_eq!(expected.len(), length);
            assert!(expected.iter().all(|&number| number <= maximum));
            expected.sort_unstable();
            let mut correct = expected.clone();
            if index % 2 == 0 {
                assert!(direction.starts_with("छोटे से बड़े"));
            } else {
                assert!(direction.starts_with("बड़े से छोटे"));
                correct.reverse();
            }
            assert_eq!(question.options.len(), 3);
            for (option_index, option) in question.options.iter().enumerate() {
                let mut values = parse(option, " → ");
                assert_eq!(values == correct, option_index == question.answer);
                values.sort_unstable();
                assert_eq!(values, expected);
            }
        }
    }
}
