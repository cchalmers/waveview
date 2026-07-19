use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchHistory {
    entries: Vec<String>,
    #[serde(skip)]
    cursor: Option<usize>,
    #[serde(skip)]
    draft: String,
}

impl SearchHistory {
    pub fn accept(&mut self, query: &str) {
        if !query.is_empty() && self.entries.last().is_none_or(|last| last != query) {
            self.entries.push(query.to_owned());
        }
        self.reset_navigation();
    }

    pub fn previous(&mut self, current: &str) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }
        let index = match self.cursor {
            Some(index) => index.saturating_sub(1),
            None => {
                self.draft = current.to_owned();
                self.entries.len() - 1
            }
        };
        self.cursor = Some(index);
        Some(self.entries[index].clone())
    }

    pub fn newer(&mut self) -> Option<String> {
        let index = self.cursor?;
        if index + 1 < self.entries.len() {
            let next = index + 1;
            self.cursor = Some(next);
            Some(self.entries[next].clone())
        } else {
            self.cursor = None;
            Some(self.draft.clone())
        }
    }

    pub fn reset_navigation(&mut self) {
        self.cursor = None;
        self.draft.clear();
    }
}

/// A compiled signal-name search shared by navigation and rendering.
pub struct SearchMatcher(regex::Regex);

impl SearchMatcher {
    pub fn new(pattern: &str) -> Option<Self> {
        if pattern.is_empty() {
            None
        } else {
            regex::Regex::new(pattern).ok().map(Self)
        }
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.0.is_match(text)
    }

    pub fn ranges<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Range<usize>> + 'a {
        self.0.find_iter(text).map(|matched| matched.range())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_invalid_patterns_do_not_match() {
        assert!(SearchMatcher::new("").is_none());
        assert!(SearchMatcher::new("[").is_none());
    }

    #[test]
    fn exposes_exact_regex_match_ranges() {
        let matcher = SearchMatcher::new(r"Read\w+").unwrap();
        assert!(matcher.is_match("top.ReadData"));
        assert_eq!(
            matcher
                .ranges("top.ReadData and ReadEnable")
                .collect::<Vec<_>>(),
            vec![4..12, 17..27]
        );
    }

    #[test]
    fn history_moves_backward_forward_and_restores_the_draft() {
        let mut history = SearchHistory::default();
        history.accept("clock");
        history.accept("reset");

        assert_eq!(history.previous("dra"), Some("reset".to_owned()));
        assert_eq!(history.previous("reset"), Some("clock".to_owned()));
        assert_eq!(history.previous("clock"), Some("clock".to_owned()));
        assert_eq!(history.newer(), Some("reset".to_owned()));
        assert_eq!(history.newer(), Some("dra".to_owned()));
        assert_eq!(history.newer(), None);
    }

    #[test]
    fn history_is_serialized_but_navigation_state_is_not() {
        let mut history = SearchHistory::default();
        history.accept("clock");
        assert_eq!(history.previous("draft"), Some("clock".to_owned()));

        let encoded = serde_json::to_string(&history).unwrap();
        let mut decoded: SearchHistory = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.previous("new draft"), Some("clock".to_owned()));
        assert_eq!(decoded.newer(), Some("new draft".to_owned()));
    }
}
