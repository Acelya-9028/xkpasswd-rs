#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
use std::str::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaddingStrategy {
    Fixed,
    Adaptive(usize),
    AdaptiveFit(usize),
}

#[derive(Debug)]
pub enum PaddingResult {
    Unchanged,
    TrimTo(usize),
    Pad(String),
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum Preset {
    Default,
    AppleID,
    WindowsNtlmV1,
    SecurityQuestions,
    Web16,
    Web32,
    Wifi,
    Xkcd,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GuessTime {
    pub years: usize,
    pub months: u8,
    pub days: u8,
}

impl fmt::Display for GuessTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.years > 1_000_000_000 {
            return write!(f, "more than a billion years");
        }

        if self.years > 1_000_000 {
            return write!(f, "more than a million years");
        }

        if self.years > 1_000 {
            return write!(f, "more than a thousand years");
        }

        let mut comps: Vec<String> = vec![];

        if self.years > 0 {
            comps.push(format!("{} years", self.years))
        }

        if self.months > 0 {
            comps.push(format!("{} months", self.months))
        }

        if self.days > 0 {
            comps.push(format!("{} days", self.days))
        }

        if comps.is_empty() {
            write!(f, "less than a day")
        } else {
            write!(f, "{}", comps.join(" "))
        }
    }
}

impl GuessTime {
    pub const GUESSES_PER_SEC: usize = 1_000;
    const SECONDS_PER_DAY: f64 = 86_400.0;
    const DAYS_PER_MONTH: f64 = 30.0;
    const DAYS_PER_YEAR: f64 = 365.0;

    pub fn for_entropy(amount: usize) -> Self {
        if amount > 64 {
            return Self {
                years: 1_000_000_001,
                months: 0,
                days: 0,
            };
        }

        if amount > 54 {
            return Self {
                years: 1_000_001,
                months: 0,
                days: 0,
            };
        }

        if amount > 44 {
            return Self {
                years: 1001,
                months: 0,
                days: 0,
            };
        }

        let mut time_to_guess =
            2.0f64.powi(amount as i32) / (Self::SECONDS_PER_DAY * Self::GUESSES_PER_SEC as f64);

        let years = (time_to_guess / Self::DAYS_PER_YEAR).floor() as usize;
        time_to_guess -= Self::DAYS_PER_YEAR * years as f64;

        let months = (time_to_guess / Self::DAYS_PER_MONTH).floor() as u8;
        time_to_guess -= Self::DAYS_PER_MONTH * months as f64;

        let days = time_to_guess.floor() as u8;

        Self {
            days,
            months,
            years,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entropy {
    pub blind_min: usize,
    pub blind_max: usize,
    pub seen: usize,
    pub guess_time: GuessTime,
}

impl fmt::Display for Entropy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blind_entropies = if self.blind_min == self.blind_max {
            format!("{} bits", self.blind_min)
        } else {
            format!("between {} & {} bits", self.blind_min, self.blind_max)
        };

        write!(
            f,
            "{} blind and {} bits with full knowledge, which takes computers {} to break at {} guesses/sec",
            blind_entropies, self.seen, self.guess_time, GuessTime::GUESSES_PER_SEC
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    French,
    German,
    Portuguese,
    Spanish,
}

type Dict<'a> = HashMap<u8, Vec<&'a str>>;

pub trait L10n {
    fn for_language(language: Language) -> Self;
}

pub trait Builder: Default + fmt::Display + Sized {
    fn with_words_count(&self, words_count: u8) -> Result<Self, String>;
    fn with_word_lengths(
        &self,
        min_length: Option<u8>,
        max_length: Option<u8>,
    ) -> Result<Self, String>;
    fn with_separators(&self, separators: &str) -> Self;
    fn with_padding_digits(&self, prefix: Option<u8>, suffix: Option<u8>) -> Self;
    fn with_padding_symbols(&self, symbols: &str) -> Self;
    fn with_padding_symbol_lengths(&self, prefix: Option<u8>, suffix: Option<u8>) -> Self;
    fn with_padding_strategy(&self, strategy: PaddingStrategy) -> Result<Self, String>;
    fn with_word_transforms(&self, transform: u8) -> Result<Self, String>;
    fn from_preset(preset: Preset) -> Self;
}

pub trait Randomizer {
    fn word_lengths(&self) -> Range<u8>;
    fn rand_words(&self, pool: &[&str]) -> Vec<String>;
    fn rand_separator(&self) -> String;
    fn rand_prefix(&self) -> (String, String);
    fn rand_suffix(&self) -> (String, String);
    fn adjust_padding(&self, pass_length: usize) -> PaddingResult;
    fn calc_entropy(&self, pool_size: usize) -> Entropy;
    fn get_padding_overhead(&self) -> usize;
    fn get_words_count(&self) -> u8;
    fn get_padding_symbols(&self) -> &str;
    fn get_padding_digits(&self) -> (u8, u8);
    fn get_padding_symbol_lengths(&self) -> (u8, u8);
    fn get_word_transforms(&self) -> u8;
}

#[derive(Debug)]
pub struct Xkpasswd {
    dict: Dict<'static>,
}

impl Default for Xkpasswd {
    fn default() -> Self {
        if cfg!(feature = "lang_en") {
            Xkpasswd::for_language(Language::English)
        } else if cfg!(feature = "lang_de") {
            Xkpasswd::for_language(Language::German)
        } else if cfg!(feature = "lang_es") {
            Xkpasswd::for_language(Language::Spanish)
        } else if cfg!(feature = "lang_fr") {
            Xkpasswd::for_language(Language::French)
        } else if cfg!(feature = "lang_pt") {
            Xkpasswd::for_language(Language::Portuguese)
        } else {
            panic!("no language bundled")
        }
    }
}

impl L10n for Xkpasswd {
    fn for_language(language: Language) -> Self {
        let dict_bytes: &[u8] = match language {
            #[cfg(feature = "lang_en")]
            Language::English => include_bytes!("../assets/dict_en.txt"),
            #[cfg(feature = "lang_de")]
            Language::German => include_bytes!("../assets/dict_de.txt"),
            #[cfg(feature = "lang_es")]
            Language::Spanish => include_bytes!("../assets/dict_es.txt"),
            #[cfg(feature = "lang_fr")]
            Language::French => include_bytes!("../assets/dict_fr.txt"),
            #[cfg(feature = "lang_pt")]
            Language::Portuguese => include_bytes!("../assets/dict_pt.txt"),
            #[allow(unreachable_patterns)]
            _ => panic!("no language bundled"),
        };

        let dict = load_dict(dict_bytes);
        Xkpasswd { dict }
    }
}

impl Xkpasswd {
    pub fn gen_pass<S: Randomizer>(&self, settings: &S) -> (String, Entropy) {
        let mut all_words: Vec<&str> = vec![];

        settings.word_lengths().for_each(|len| {
            if let Some(words) = self.dict.get(&len) {
                all_words.extend(words);
            };
        });

        let separator = &settings.rand_separator();
        let mut words: Vec<String> = vec![];

        let (prefix_symbols, prefix_digits) = settings.rand_prefix();
        if !prefix_digits.is_empty() {
            words.push(prefix_digits);
        }

        words.extend(settings.rand_words(&all_words));

        let (suffix_digits, suffix_symbols) = settings.rand_suffix();
        if !suffix_digits.is_empty() {
            words.push(suffix_digits);
        }

        let passwd = format!(
            "{}{}{}",
            prefix_symbols,
            words.join(separator),
            suffix_symbols
        );

        let passwd = match settings.adjust_padding(passwd.len()) {
            PaddingResult::Unchanged => passwd,
            PaddingResult::TrimTo(len) => passwd[..len].to_string(),
            PaddingResult::Pad(padded_symbols) => passwd + &padded_symbols,
        };

        let entropy = settings.calc_entropy(all_words.len());

        (passwd, entropy)
    }

    pub fn gen_pass_fit<S: Randomizer>(&self, settings: &S, target_length: usize) -> Result<(String, Entropy), String> {
        // Calculate fixed overhead (digits + symbols, but NOT separators yet since word count is dynamic)
        let (prefix_digits, suffix_digits) = settings.get_padding_digits();
        let (prefix_symbols, suffix_symbols) = settings.get_padding_symbol_lengths();
        
        let digits_len = prefix_digits as usize + suffix_digits as usize;
        let symbols_len = prefix_symbols as usize + suffix_symbols as usize;
        let fixed_overhead = digits_len + symbols_len;
        
        if fixed_overhead >= target_length {
            return Err(format!(
                "Cannot generate password: padding overhead ({} chars) exceeds target length ({} chars)",
                fixed_overhead, target_length
            ));
        }

        let available_for_words_and_seps = target_length - fixed_overhead;
        let min_word_length = 4;
        
        // Minimum space needed: at least 1 word of min length
        if available_for_words_and_seps < min_word_length {
            return Err(format!(
                "Cannot generate password: need at least {} chars for 1 word, but only {} chars available after padding",
                min_word_length, available_for_words_and_seps
            ));
        }

        // Try to distribute space evenly across words, but respect word length constraints
        let word_lengths_range = settings.word_lengths();
        let min_len = word_lengths_range.start;
        let max_len = word_lengths_range.end - 1; // Range is exclusive at end
        
        // Build a pool of words that could fit
        let mut word_pool: Vec<&str> = vec![];
        for len in min_len..=max_len {
            if let Some(words) = self.dict.get(&len) {
                word_pool.extend(words);
            }
        }

        if word_pool.is_empty() {
            return Err("No words available in the specified length range".to_string());
        }

        // Try different word counts, starting from a reasonable estimate
        let avg_word_len = (min_len + max_len) / 2;
        let estimated_words = (available_for_words_and_seps / (avg_word_len as usize + 1)).max(1);
        
        let mut attempts = 0;
        let max_attempts = 500;
        
        // Try word counts from 1 to estimated * 2 (max 20)
        let max_word_count = (estimated_words * 2).min(20);
        
        loop {
            attempts += 1;
            if attempts > max_attempts {
                return Err(format!(
                    "Could not find word combination fitting in {} chars after {} attempts",
                    target_length, max_attempts
                ));
            }

            // Cycle through different word counts
            let words_count = 1 + (attempts % max_word_count);
            
            // Calculate total items that will be joined (words + digits if present)
            let has_prefix_digits = prefix_digits > 0;
            let has_suffix_digits = suffix_digits > 0;
            let total_items = words_count + (if has_prefix_digits { 1 } else { 0 }) + (if has_suffix_digits { 1 } else { 0 });
            let separator_space = total_items.saturating_sub(1);
            let space_for_words = available_for_words_and_seps.saturating_sub(separator_space);
            
            if space_for_words < min_word_length {
                continue;
            }

            // Generate random words
            let candidate_words = self.rand_words_count(&word_pool, words_count, settings);
            let total_word_length: usize = candidate_words.iter().map(|w| w.len()).sum();
            
            // We need EXACT match: total_word_length + separators = space_for_words
            if total_word_length == space_for_words {
                // Found an EXACT fitting combination!
                let separator = settings.rand_separator();
                let mut words: Vec<String> = vec![];

                let (prefix_symbols, prefix_digits) = settings.rand_prefix();
                if !prefix_digits.is_empty() {
                    words.push(prefix_digits);
                }

                words.extend(candidate_words.clone());

                let (suffix_digits, suffix_symbols) = settings.rand_suffix();
                if !suffix_digits.is_empty() {
                    words.push(suffix_digits);
                }

                let passwd = format!(
                    "{}{}{}",
                    prefix_symbols,
                    words.join(&separator),
                    suffix_symbols
                );

                let entropy = settings.calc_entropy(word_pool.len());
                
                log::debug!(
                    "AdaptiveFit: generated password with {} words, {} chars (target: {}) in {} attempts",
                    candidate_words.len(),
                    passwd.len(),
                    target_length,
                    attempts
                );

                return Ok((passwd, entropy));
            }
        }
    }

    fn rand_words_count<S: Randomizer>(&self, pool: &[&str], count: usize, settings: &S) -> Vec<String> {
        use rand::distributions::{Distribution, Uniform};
        use std::collections::HashMap;
        
        if pool.is_empty() {
            return vec![];
        }

        let mut rng = rand::thread_rng();
        let word_indices = Uniform::from(0..pool.len());

        // Get transform list for this word count
        let transforms_list = self.build_transforms_list_for_count(count as u8, settings);

        // not enough words to distinguishably randomize
        if pool.len() < count {
            return (0..count)
                .map(|i| {
                    let index: usize = word_indices.sample(&mut rng);
                    let word = pool[index];
                    let transform = transforms_list.get(i).copied().unwrap_or(crate::bit_flags::WordTransform::Lowercase);
                    self.transform_word(word, transform)
                })
                .collect();
        }

        // enough words, ensure no duplicates
        let mut index_marker: HashMap<usize, bool> = HashMap::new();
        (0..count)
            .map(|i| loop {
                let index: usize = word_indices.sample(&mut rng);
                let word = pool[index];

                if index_marker.get(&index).is_none() {
                    index_marker.insert(index, true);
                    let transform = transforms_list.get(i).copied().unwrap_or(crate::bit_flags::WordTransform::Lowercase);
                    break self.transform_word(word, transform);
                }
            })
            .collect()
    }

    fn build_transforms_list_for_count<S: Randomizer>(&self, count: u8, settings: &S) -> Vec<crate::bit_flags::WordTransform> {
        use rand::distributions::{Distribution, Uniform};
        use crate::bit_flags::{BitFlags, WordTransform};
        
        let word_transforms = settings.get_word_transforms();
        
        if word_transforms.has_flag(WordTransform::AltercaseLowerFirst) {
            return (0..count)
                .map(|idx| {
                    if idx % 2 == 0 {
                        WordTransform::Lowercase
                    } else {
                        WordTransform::Uppercase
                    }
                })
                .collect();
        }

        if word_transforms.has_flag(WordTransform::AltercaseUpperFirst) {
            return (0..count)
                .map(|idx| {
                    if idx % 2 == 0 {
                        WordTransform::Uppercase
                    } else {
                        WordTransform::Lowercase
                    }
                })
                .collect();
        }

        const ALL_SINGLE_WORD_TRANSFORMS: [WordTransform; 4] = [
            WordTransform::Lowercase,
            WordTransform::Titlecase,
            WordTransform::Uppercase,
            WordTransform::InversedTitlecase,
        ];

        let whitelisted_transforms: Vec<&WordTransform> = ALL_SINGLE_WORD_TRANSFORMS
            .iter()
            .filter(|&&transform| word_transforms & transform)
            .collect();

        let mut rng = rand::thread_rng();
        let transform_indices = Uniform::from(0..whitelisted_transforms.len());

        (0..count)
            .map(|_| {
                let index: usize = transform_indices.sample(&mut rng);
                *whitelisted_transforms[index]
            })
            .collect()
    }

    fn transform_word(&self, word: &str, transform: crate::bit_flags::WordTransform) -> String {
        use crate::bit_flags::WordTransform;
        
        match transform {
            WordTransform::Titlecase => word[..1].to_uppercase() + &word[1..],
            WordTransform::Uppercase => word.to_uppercase(),
            WordTransform::InversedTitlecase => word[..1].to_lowercase() + &word[1..].to_uppercase(),
            _ => word.to_lowercase(),
        }
    }
}

fn load_dict(dict_bytes: &[u8]) -> Dict {
    let dict_str = from_utf8(dict_bytes).unwrap_or("").trim();
    let mut dict: Dict = HashMap::new();

    log::debug!("loaded raw dict with {} lines", dict_str.lines().count());

    dict_str.lines().for_each(|line| {
        let mut comps = line.trim().split(':');

        if let Some(len_str) = comps.next() {
            let len = len_str.parse::<u8>().unwrap();
            let words_csv = comps.next().unwrap_or("");
            let words: Vec<&str> = words_csv.split(',').collect();
            dict.insert(len, words);
        }
    });

    log::debug!(
        "parsed dict with {:?} entries",
        dict.iter().fold(0, |acc, cur| acc + cur.1.len())
    );

    dict
}
