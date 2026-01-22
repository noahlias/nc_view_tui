use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ParserSettings {
    pub ignore_missing_words: Vec<char>,
    pub ignore_unknown_words: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct ParserConfig {
    ignore_missing_words: Vec<String>,
    ignore_unknown_words: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            ignore_missing_words: vec!["E".to_string()],
            ignore_unknown_words: true,
        }
    }
}

impl TryFrom<ParserConfig> for ParserSettings {
    type Error = anyhow::Error;

    fn try_from(value: ParserConfig) -> Result<Self> {
        let mut ignore_missing_words = Vec::new();
        for item in value.ignore_missing_words {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut chars = trimmed.chars();
            let ch = chars
                .next()
                .ok_or_else(|| anyhow!("empty ignore_missing_words entry"))?;
            if chars.next().is_some() {
                return Err(anyhow!(
                    "ignore_missing_words entry must be a single letter: {}",
                    item
                ));
            }
            ignore_missing_words.push(ch.to_ascii_uppercase());
        }
        Ok(Self {
            ignore_missing_words,
            ignore_unknown_words: value.ignore_unknown_words,
        })
    }
}
