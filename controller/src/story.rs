use crate::{card::Card, openrouter::Usage};
use chrono::{DateTime, Utc};
use getset::Getters;
use log::info;
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub(crate) struct StoryOrigin {
    git_revision: String,
    cards: Vec<Card>,
}

impl StoryOrigin {
    pub(crate) fn new(cards: Vec<Card>) -> Self {
        Self {
            git_revision: git_version::git_version!().to_string(),
            cards,
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter()
    }

    pub(crate) fn prompt(&self) -> String {
        self.cards
            .iter()
            .map(|c| c.prompt())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub(crate) struct Story {
    origin: StoryOrigin,

    timestamp: DateTime<Utc>,

    title: String,
    text: String,

    model: String,
    usage: Usage,
}

impl std::fmt::Display for Story {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.title())?;
        writeln!(f)?;
        writeln!(f, "{}", self.text())?;
        writeln!(f)?;
        writeln!(f, "Timestamp: {}", self.timestamp())?;
        write!(f, "Cards:")?;
        if self.origin().cards().is_empty() {
            writeln!(f, " (none)")?;
        } else {
            writeln!(f)?;
            for card in self.origin().cards() {
                writeln!(f, "- {:?}: {} ({})", card.kind(), card.title(), card.id())?;
                writeln!(f, "  {}", card.description())?;
            }
        }
        writeln!(f, "Model: {}", self.model())?;
        writeln!(
            f,
            "Tokens (prompt/completion): {}/{}",
            self.usage().prompt_tokens(),
            self.usage().completion_tokens()
        )?;
        writeln!(f, "Cost: ${}", self.usage().cost())?;
        writeln!(f, "Git revision: {}", self.origin().git_revision())?;
        Ok(())
    }
}

impl Story {
    pub(crate) const fn new(
        origin: StoryOrigin,
        timestamp: DateTime<Utc>,
        title: String,
        text: String,
        model: String,
        usage: Usage,
    ) -> Self {
        Self {
            origin,
            timestamp,
            title,
            text,
            model,
            usage,
        }
    }

    pub(crate) fn save_in_dir(&self, dir: &Path) -> miette::Result<()> {
        // Ensure the directory exists
        std::fs::create_dir_all(dir).into_diagnostic()?;

        // Serialise self to pretty JSON
        let json = serde_json::to_string_pretty(self).into_diagnostic()?;

        // Use the timestamp formatted as YYYY-MM-DD_HHMMSS_mmm for the filename
        let filename = format!(
            "{}_{:03}.json",
            self.timestamp.format("%Y-%m-%d_%H%M%S"),
            self.timestamp.timestamp_subsec_millis()
        );
        let path = dir.join(filename);

        // Write the JSON to the file
        std::fs::write(&path, json).into_diagnostic()?;
        info!("Saved story to {path:?}");

        Ok(())
    }
}
