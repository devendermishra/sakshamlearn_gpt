use crate::{
    content::{self, CustomCatalog, Kind, Paper, Question},
    progress::{Input, Outcome, Step},
    state::{self, Pin, Session, State},
};
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
            subject: step.subject.into(),
            topic: step.topic.progress_id(),
            step: step.index,
            review: step.review,
            missed: false,
        },
    );
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
    for topic in &bundled("math").topics {
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
