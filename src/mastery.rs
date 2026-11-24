//! Deliberately conservative, inspectable V1 rules; not a calibrated clinical score.
use crate::assessment::{Attempt, Evidence};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    NoEvidence,
    Practising,
    IndependentSuccess,
    Retained,
}
impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Self::NoEvidence => "No evidence",
            Self::Practising => "Practising",
            Self::IndependentSuccess => "Independent success",
            Self::Retained => "Retained",
        }
    }
}
pub struct Summary {
    pub level: Level,
    pub independent: usize,
    pub assisted: usize,
    pub unscored: usize,
    pub modalities: BTreeMap<String, usize>,
    pub assistance: BTreeMap<String, usize>,
    pub review_successes: usize,
    pub reviews: usize,
}
pub fn estimate(evidence: &Evidence, id: &str) -> Summary {
    let mut attempts: Vec<_> = evidence
        .attempts
        .values()
        .filter(|a| a.competency_id == id)
        .collect();
    attempts.sort_by_key(|a| (a.day, a.timestamp, &a.submission_id));
    let eligible: Vec<_> = attempts
        .iter()
        .copied()
        .filter(|a| a.independent() && a.correct.is_some())
        .collect();
    fn qualifies(items: &[&Attempt]) -> bool {
        let recent: Vec<_> = items.iter().rev().take(10).collect();
        recent.len() >= 3
            && recent.iter().map(|a| a.day).collect::<BTreeSet<_>>().len() >= 2
            && recent.iter().filter(|a| a.success()).count() * 5 >= recent.len() * 4
            && recent.first().is_some_and(|a| a.success())
    }
    let since_failure = eligible
        .iter()
        .rposition(|a| !a.success())
        .map_or(0, |i| i + 1);
    let retention = &eligible[since_failure..];
    let first_success = (3..=retention.len())
        .find(|&n| qualifies(&retention[..n]))
        .map(|n| retention[n - 1].day);
    let retained = first_success.is_some_and(|first| {
        let days: BTreeSet<_> = retention
            .iter()
            .filter(|a| a.review && a.success() && a.day > first)
            .map(|a| a.day)
            .collect();
        days.len() >= 2 && days.iter().any(|d| *d >= first.saturating_add(7))
    });
    let level = if attempts.is_empty() {
        Level::NoEvidence
    } else if !qualifies(&eligible) {
        Level::Practising
    } else if retained {
        Level::Retained
    } else {
        Level::IndependentSuccess
    };
    let mut modalities = BTreeMap::new();
    let mut assistance = BTreeMap::new();
    for a in &attempts {
        *modalities.entry(format!("{:?}", a.mode)).or_default() += 1;
        *assistance
            .entry(format!("{:?}", a.context.assistance))
            .or_default() += 1;
    }
    Summary {
        level,
        independent: eligible.len(),
        assisted: attempts.iter().filter(|a| !a.independent()).count(),
        unscored: attempts.iter().filter(|a| a.correct.is_none()).count(),
        modalities,
        assistance,
        reviews: attempts.iter().filter(|a| a.review).count(),
        review_successes: attempts.iter().filter(|a| a.review && a.success()).count(),
    }
}
