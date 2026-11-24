use crate::{
    assessment::{self, Assistance, Context, Evidence, Mode},
    mastery::{self, Level},
    planner,
    progress::Input,
    state::State,
};
fn sample(day: u64, correct: Option<bool>, review: bool) -> assessment::Attempt {
    let context = Context {
        submission_id: format!("attempt-{day}"),
        ..Context::default()
    };
    assessment::Attempt {
        submission_id: context.submission_id.clone(),
        source: "simulation".into(),
        exercise_id: "sample".into(),
        competency_id: "skill".into(),
        timestamp: day as i64 * 86400,
        day,
        correct,
        accuracy: correct.map(|b| if b { 1.0 } else { 0.0 }),
        mode: Mode::Selection,
        context,
        review,
    }
}
#[test]
fn completion_and_legacy_state_never_invent_mastery() {
    let mut state:State=serde_json::from_str(r#"{"finished":["math/numbers-1-3"],"reviews":{"math/numbers-1-3":{"started":1,"due":2,"interval":2}}}"#).unwrap();
    assessment::migrate(&mut state, 100);
    assert!(state.evidence.attempts.is_empty());
    assert!(!state.evidence.reviews.is_empty());
    assert!(state
        .evidence
        .reviews
        .values()
        .all(|c| c.due == 2 && c.interval_days == 1));
    let checks = state.evidence.reviews.len();
    assessment::migrate(&mut state, 200);
    assert_eq!(state.evidence.reviews.len(), checks);
    assert_eq!(
        mastery::estimate(&state.evidence, "skill").level,
        Level::NoEvidence
    );
    let restored: State = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
    assert_eq!(restored.finished, state.finished);
    assert_eq!(restored.evidence.reviews, state.evidence.reviews);
}
#[test]
fn mastery_requires_independent_evidence_and_later_retention() {
    let mut e = Evidence::default();
    for day in [1, 2, 3] {
        e.record(sample(day, Some(true), false));
    }
    assert_eq!(
        mastery::estimate(&e, "skill").level,
        Level::IndependentSuccess
    );
    e.record(sample(4, Some(true), true));
    e.record(sample(10, Some(true), true));
    assert_eq!(mastery::estimate(&e, "skill").level, Level::Retained);
    e.record(sample(11, Some(false), true));
    assert_eq!(mastery::estimate(&e, "skill").level, Level::Practising);
    for day in 12..20 {
        let mut a = sample(day, Some(true), true);
        a.context.assistance = Assistance::Guided;
        e.record(a);
    }
    assert_eq!(mastery::estimate(&e, "skill").level, Level::Practising);
    assert_eq!(mastery::estimate(&e, "skill").assisted, 8);
}
#[test]
fn unscored_model_help_errors_and_sparse_evidence_cannot_establish_mastery() {
    for variant in 0..6 {
        let mut e = Evidence::default();
        for day in 1..8 {
            let mut a = sample(day, Some(true), false);
            match variant {
                0 => a.correct = None,
                1 => a.context.model_visible = true,
                2 => a.context.hint_used = true,
                3 => a.context.helper_used = true,
                4 => a.context.solution_exposed = true,
                _ => {
                    a.context.prior_errors = true;
                    a.correct = Some(false);
                }
            }
            e.record(a);
        }
        assert_eq!(mastery::estimate(&e, "skill").level, Level::Practising);
    }
    let mut e = Evidence::default();
    e.record(sample(1, Some(true), false));
    assert_eq!(mastery::estimate(&e, "skill").level, Level::Practising);
}
#[test]
fn reviews_continue_cap_retry_and_deduplicate() {
    let mut e = Evidence::default();
    let first = sample(1, Some(true), false);
    assert!(e.record(first.clone()));
    assert!(!e.record(first));
    for day in 2..12 {
        e.record(sample(day, Some(true), true));
    }
    assert_eq!(e.reviews["skill"].interval_days, 60);
    assert_eq!(e.reviews["skill"].due, 71);
    e.record(sample(12, Some(false), true));
    assert_eq!(e.reviews["skill"].interval_days, 30);
    assert_eq!(e.reviews["skill"].due, 13);
    let mut assisted = sample(13, Some(true), true);
    assisted.context.assistance = Assistance::VerbalPrompt;
    e.record(assisted);
    assert_eq!(e.reviews["skill"].interval_days, 30);
    assert_eq!(e.reviews["skill"].due, 14);
    e.record(sample(14, None, true));
    assert_eq!(e.reviews["skill"].due, 15);
    assert_eq!(e.attempts.len(), 14);
    e.record(sample(u64::MAX, Some(true), true));
    assert_eq!(e.reviews["skill"].due, u64::MAX);
}
#[test]
fn catalog_covers_bundled_activities_and_translations_have_identical_ids() {
    let catalog = assessment::catalog();
    assert!(assessment::validate_catalog(&catalog));
    for s in crate::content::BUNDLED
        .iter()
        .filter(|s| matches!(s.id.as_str(), "en" | "hi" | "math"))
    {
        for t in &s.topics {
            for recall in [false, true] {
                for i in 0..if recall {
                    t.recall.len()
                } else {
                    t.exercises.len()
                } {
                    let m = assessment::mapping(&s.id, t, i, recall).unwrap();
                    if s.id == "math" {
                        assert_eq!(
                            m.competency,
                            assessment::mapping(
                                "math",
                                crate::content::english_math_topic(t),
                                i,
                                recall
                            )
                            .unwrap()
                            .competency
                        );
                    }
                }
            }
            let mut changed = t.clone();
            changed.revision = t.revision + 1;
            assert!(assessment::mapping(&s.id, &changed, 0, false).is_none());
        }
    }
}
#[test]
fn assessment_preserves_unknown_writing_and_first_response_errors() {
    let s = crate::content::BUNDLED
        .iter()
        .find(|s| s.id == "math")
        .unwrap();
    let t = s.topics.iter().find(|t| t.id == "numbers-1-3").unwrap();
    assert!(assessment::attempt(
        "math",
        t,
        0,
        false,
        &Input::Draft(vec![]),
        Context::default(),
        "lesson",
        false,
        1
    )
    .is_none());
    let a = assessment::attempt(
        "math",
        t,
        1,
        false,
        &Input::Submit(vec![]),
        Context::default(),
        "lesson",
        false,
        1,
    )
    .unwrap();
    assert_eq!(a.correct, None);
    assert_eq!(a.mode, Mode::Tracing);
    assert!(a.context.model_visible);
    let a = assessment::attempt(
        "math",
        t,
        0,
        false,
        &Input::Answer(false),
        Context {
            prior_errors: true,
            ..Context::default()
        },
        "lesson",
        false,
        1,
    )
    .unwrap();
    assert_eq!(a.correct, Some(false));
    assert!(!a.success());
}
#[test]
fn simulated_plans_fit_budgets_resume_and_keep_chains_together() {
    for minutes in [5, 15, 30] {
        for grade in [1, 2] {
            let mut state = State::default();
            state.daily_minutes = minutes;
            let catalog = assessment::catalog();
            for m in catalog.iter().filter(|m| m.grade == grade).take(200) {
                state.evidence.reviews.insert(
                    m.competency.clone(),
                    assessment::Check {
                        due: 1,
                        interval_days: 1,
                        rotation: 0,
                    },
                );
            }
            let plan = planner::build(&state, grade, 100);
            assert!(plan.minutes() <= minutes);
            assert!(
                plan.tasks
                    .iter()
                    .filter(|t| t.reason == planner::Reason::New)
                    .count()
                    <= 2
            );
            assert!(plan.tasks.iter().any(|t| t.reason == planner::Reason::New));
            assert_eq!(plan.tasks, planner::build(&state, grade, 100).tasks);
            let mut seen = std::collections::BTreeSet::new();
            for task in &plan.tasks {
                assert!(seen.insert(&task.id));
                let topic = planner::topic(task, crate::language::Language::Hindi);
                if !task.recall {
                    assert!(!matches!(
                        topic.exercises[task.indices[0]].kind,
                        crate::content::Kind::Revise(..)
                    ));
                    if matches!(
                        topic.exercises[task.indices[0]].kind,
                        crate::content::Kind::Compose(..)
                    ) {
                        let end = task.indices.last().unwrap() + 1;
                        assert!(topic
                            .exercises
                            .get(end)
                            .is_none_or(|e| !matches!(e.kind, crate::content::Kind::Revise(..))));
                    }
                }
            }
            state.adaptive = Some(plan);
            let restored: State =
                serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
            assert_eq!(
                restored.adaptive.as_ref().unwrap().tasks,
                state.adaptive.as_ref().unwrap().tasks
            );
            let mut s = restored.adaptive.unwrap();
            while !s.complete() {
                s.advance();
            }
            assert!(s.complete());
            assert!(state.finished.is_empty());
        }
    }
}
#[test]
fn failed_canonical_save_and_replay_leave_one_evidence_record() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("file");
    std::fs::write(&blocker, "block").unwrap();
    let mut state = State::default();
    let a = sample(1, Some(true), false);
    state.evidence.record(a.clone());
    assert!(crate::state::save(&blocker.join("state.json"), &state).is_err());
    let path = dir.path().join("state.json");
    crate::state::save(&path, &state).unwrap();
    let mut restored = crate::state::load(&path).unwrap();
    assert!(!restored.evidence.record(a));
    assert_eq!(restored.evidence.attempts.len(), 1);
}

#[test]
fn incompatible_saved_session_is_rejected_without_replacing_progress() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let bytes = br#"{"adaptive":{"grade":1,"position":1,"tasks":[]}}"#;
    std::fs::write(&path, bytes).unwrap();
    assert!(crate::state::load(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn continuing_topic_review_rotates_questions_separately_from_interval() {
    let t = crate::content::BUNDLED
        .iter()
        .flat_map(|s| &s.topics)
        .find(|t| t.recall.len() > 4)
        .unwrap();
    assert_eq!(
        crate::content::review_questions(t, Some(0)),
        crate::content::review_questions(t, Some(3))
    );
    let mut state = State::default();
    state.finish("test", 1);
    for day in 2..12 {
        state.review("test", day, false);
    }
    assert!(state.reviews.contains_key("test"));
    assert!(state.reviews["test"].due > 12);
}

#[test]
fn adaptive_submission_keeps_writing_archive_without_rewards_or_duplicate_samples() {
    let mut state = State::default();
    let mut plan = planner::build(&state, 1, 10);
    let task = plan
        .tasks
        .iter()
        .find(|t| !t.recall && t.competency.contains("formation"))
        .unwrap()
        .clone();
    let ink = vec![vec![vec![[0.2, 0.2], [0.3, 0.4]]]];
    let ctx = Context::default();
    assert!(!planner::record_submission(
        &mut state,
        &task,
        task.indices[0],
        &Input::Submit(vec![]),
        ctx.clone(),
        10
    ));
    assert!(planner::record_submission(
        &mut state,
        &task,
        task.indices[0],
        &Input::Submit(ink.clone()),
        ctx.clone(),
        10
    ));
    assert!(!planner::record_submission(
        &mut state,
        &task,
        task.indices[0],
        &Input::Submit(ink),
        ctx,
        10
    ));
    assert!(state.finished.is_empty());
    assert_eq!(state.history.values().map(Vec::len).sum::<usize>(), 1);
    assert!(state
        .evidence
        .attempts
        .values()
        .all(|a| a.correct.is_none()));
    plan.advance();
    state.adaptive = Some(plan);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    crate::state::save(&path, &state).unwrap();
    let restored = crate::state::load(&path).unwrap();
    assert_eq!(restored.history.len(), 1);
    assert!(restored.finished.is_empty());
}

#[test]
fn prerequisite_remediation_follows_due_reviews_for_an_encountered_skill() {
    let mut state = State::default();
    let catalog = assessment::catalog();
    let m = catalog
        .iter()
        .find(|m| m.grade == 2 && !m.prerequisites.is_empty())
        .unwrap();
    state.evidence.reviews.insert(
        m.competency.clone(),
        assessment::Check {
            due: 1,
            interval_days: 1,
            rotation: 0,
        },
    );
    let plan = planner::build(&state, 2, 100);
    let review = plan
        .tasks
        .iter()
        .position(|t| t.reason == planner::Reason::Review)
        .unwrap();
    let remediation = plan
        .tasks
        .iter()
        .position(|t| {
            t.reason == planner::Reason::Remediation && m.prerequisites.contains(&t.competency)
        })
        .unwrap();
    assert!(review < remediation);
    assert!(!plan
        .tasks
        .iter()
        .any(|t| t.reason == planner::Reason::New && t.competency == m.competency));
}
#[test]
fn retention_needs_new_spaced_evidence_after_a_lapse() {
    let mut e = Evidence::default();
    for d in [1, 2, 3, 4, 10] {
        e.record(sample(d, Some(true), true));
    }
    assert_eq!(mastery::estimate(&e, "skill").level, Level::Retained);
    e.record(sample(11, Some(false), true));
    e.record(sample(12, Some(true), true));
    assert_ne!(mastery::estimate(&e, "skill").level, Level::Retained);
}

#[test]
fn new_work_rotates_subjects_over_learning_dates() {
    let state = State::default();
    let subjects: std::collections::BTreeSet<_> = (0..3)
        .map(|day| planner::build(&state, 1, day).tasks[0].subject.clone())
        .collect();
    assert_eq!(subjects.len(), 3);
}
