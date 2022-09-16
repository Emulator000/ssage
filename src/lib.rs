use std::collections::{HashMap, VecDeque};

use iter_tools::Itertools;
use priority_queue::PriorityQueue;
use unicase::UniCase;

type SsageString = UniCase<String>;
type SsageQueue = PriorityQueue<SsageString, Weight>;

#[derive(Debug)]
pub struct Configuration {
    pub threshold: Weight,
    pub take_words_min: usize,
    pub take_words_max: usize,
    pub take_words_percentage: usize,
    pub min_word_length: usize,
}

impl Configuration {
    const DEFAULT_THRESHOLD: u64 = 1;
    const TAKE_WORDS_MIN: usize = 3;
    const TAKE_WORDS_MAX: usize = 30;
    const TAKE_WORDS_PERCENTAGE: usize = 10;
    const MIN_WORD_LENGTH: usize = 4;

    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            threshold: Weight::new(Self::DEFAULT_THRESHOLD),
            take_words_min: Self::TAKE_WORDS_MIN,
            take_words_max: Self::TAKE_WORDS_MAX,
            take_words_percentage: Self::TAKE_WORDS_PERCENTAGE,
            min_word_length: Self::MIN_WORD_LENGTH,
        }
    }
}

#[derive(Debug)]
pub struct Ssage {
    messages: VecDeque<SsageString>,
    keywords: SsageQueue,
    configuration: Configuration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Weight {
    w: u64,
}

impl Weight {
    pub fn new(w: u64) -> Self {
        Self { w }
    }
}

impl Ssage {
    const MIN_THRESHOLD: u64 = 1;
    const MAX_THRESHOLD: u64 = 20;
    const WEIGHT_INCREMENT: u64 = 1;

    pub fn new(configuration: Configuration) -> Self {
        Self {
            messages: VecDeque::new(),
            keywords: SsageQueue::new(),
            configuration,
        }
    }

    /// Example
    ///
    /// ```
    ///     use ssage::Ssage;
    ///
    ///     let mut ssage = Ssage::new(Default::default());
    ///
    ///     let _ = ssage.feed("hi! this is just a sample message with distinct words.");
    ///     ssage.prioritize_keyword("message");
    ///
    ///     println!("Output: {}", ssage.feed("just a message"));
    /// ```
    pub fn feed<S: AsRef<str>>(&mut self, message: S) -> String {
        let clean_message = message
            .as_ref()
            .chars()
            .map(|ch| match ch {
                'A'..='Z' => ch,
                'a'..='z' => ch,
                'À'..='ÿ' => ch,
                _ => ' ',
            })
            .collect();
        let message = UniCase::new(clean_message);
        let mut keywords = self.fetch_important_keywords(&message);

        let output = self.fetch(
            &keywords,
            Some(message.len() * self.configuration.take_words_percentage / 100),
        );

        self.messages.push_back(message);
        self.keywords.append(&mut keywords);

        output
    }

    /// Example
    ///
    /// ```
    ///     use ssage::Ssage;
    ///
    ///     let mut ssage = Ssage::new(Default::default());
    ///
    ///     let _ = ssage.feed("hi! how are you mate?");
    ///     let _ = ssage.feed("this is just a sample message.");
    ///
    ///     println!("Output: {}", ssage.feed_empty());
    /// ```
    pub fn feed_empty(&self) -> String {
        self.fetch(&self.keywords, Some(self.configuration.take_words_max))
    }

    fn fetch(&self, keywords: &SsageQueue, words: Option<usize>) -> String {
        let keywords = keywords
            .clone()
            .into_sorted_iter()
            .filter(|(word, weight)| {
                *weight >= self.configuration.threshold
                    && word.len() >= self.configuration.min_word_length
            });

        if let Some(mut words) = words {
            if words < self.configuration.min_word_length {
                words = self.configuration.min_word_length;
            } else if words > self.configuration.take_words_max {
                words = self.configuration.take_words_max;
            }

            keywords.take(words).map(|(word, _)| word).join(" ")
        } else {
            keywords.map(|(word, _)| word).join(" ")
        }
    }

    pub fn prioritize_keyword<S: AsRef<str>>(&mut self, keyword: S) -> bool {
        Self::change_keyword_weight(
            &mut self.keywords,
            keyword,
            false,
            Self::WEIGHT_INCREMENT as i64,
        )
    }

    pub fn trivialize_keyword<S: AsRef<str>>(&mut self, keyword: S) -> bool {
        Self::change_keyword_weight(
            &mut self.keywords,
            keyword,
            false,
            -(Self::WEIGHT_INCREMENT as i64),
        )
    }

    fn change_keyword_weight<S: AsRef<str>>(
        keywords: &mut SsageQueue,
        keyword: S,
        insert_if_not_exists: bool,
        increment: i64,
    ) -> bool {
        let key = UniCase::new(keyword.as_ref().into());
        if let Some(weight) = keywords.get_priority(&key) {
            let mut weight = weight.clone();

            if increment >= 0 {
                weight.w += increment.abs() as u64;
            } else {
                weight.w -= increment.abs() as u64;
            }

            if weight.w < Self::MIN_THRESHOLD {
                weight.w = Self::MIN_THRESHOLD;
            } else if weight.w > Self::MAX_THRESHOLD {
                weight.w = Self::MAX_THRESHOLD;
            }

            keywords.change_priority(&key, weight).is_some()
        } else if insert_if_not_exists {
            let weight = if increment >= 0 {
                increment.abs() as u64
            } else {
                Self::MIN_THRESHOLD
            };

            keywords.push(key, Weight::new(weight));

            true
        } else {
            false
        }
    }

    fn fetch_important_keywords(&mut self, message: &SsageString) -> SsageQueue {
        let words: Vec<&str> = message.split_whitespace().collect();

        let mut scanned_words = HashMap::new();
        words.iter().for_each(|word| {
            scanned_words.insert(word, scanned_words.get(&word).unwrap_or(&0) + 1);
        });

        let mut weighted_words = SsageQueue::new();

        words.iter().for_each(|word| {
            let _ = Self::change_keyword_weight(
                &mut weighted_words,
                word,
                true,
                Self::WEIGHT_INCREMENT as i64,
            );
        });

        scanned_words.keys().for_each(|word| {
            let key = UniCase::new((**word).into());
            let _ = Self::change_keyword_weight(
                &mut weighted_words,
                word,
                true,
                if let Some(weight) = self.keywords.get_priority(&key) {
                    weight.w as i64
                } else {
                    -(Self::WEIGHT_INCREMENT as i64)
                },
            );
        });

        weighted_words
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_basic_feeding() {
        let mut ssage = Ssage::new(Default::default());

        let _ = ssage.feed("hi! how are you mate?");
        let _ = ssage.feed("this is just a sample message.");

        assert_eq!(ssage.feed_empty(), "this mate message sample just");
        assert_eq!(ssage.feed("are you there mate?"), "mate there");
    }

    #[test]
    pub fn test_basic_prioritize_and_trivialize() {
        let mut ssage = Ssage::new(Default::default());

        let _ = ssage.feed("hi! this is just a sample message with distinct words.");
        ssage.prioritize_keyword("message");

        assert_eq!(ssage.feed("just a message"), "message just");

        ssage.prioritize_keyword("just");
        ssage.prioritize_keyword("just");

        assert_eq!(ssage.feed("just a message"), "just message");

        ssage.prioritize_keyword("message");
        ssage.prioritize_keyword("message");
        ssage.prioritize_keyword("just");
        ssage.prioritize_keyword("message");

        assert_eq!(ssage.feed("just a message"), "message just");
    }
}
