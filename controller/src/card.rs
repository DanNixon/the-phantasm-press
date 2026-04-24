use getset::Getters;
use miette::IntoDiagnostic;
use protocol::CardId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CardType {
    Protagonist,
    Setting,
    Engine,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[getset(get = "pub(crate)")]
pub(crate) struct Card {
    /// The ID of the RFID card
    #[serde(
        serialize_with = "hex::serde::serialize_upper",
        deserialize_with = "hex::serde::deserialize"
    )]
    id: CardId,

    /// Type of the card
    kind: CardType,

    /// Title/name of the card
    title: String,

    /// User facing description of the card
    description: String,

    /// Extra information that is used in prompts but not visible to the user
    extra_description: Option<String>,
}

impl Card {
    pub(crate) fn prompt(&self) -> String {
        let extra = match self.extra_description {
            Some(ref extra_description) => format!(" ({extra_description})"),
            None => "".into(),
        };
        format!(
            "{:?}: \"{}\" - {}{extra}",
            self.kind, self.title, self.description
        )
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CardLibrary {
    #[serde(rename = "card")]
    cards: Vec<Card>,
}

impl CardLibrary {
    pub(crate) fn load() -> miette::Result<Self> {
        let toml = concat!(
            include_str!("../../cards/protagonist.toml"),
            include_str!("../../cards/setting.toml"),
            include_str!("../../cards/engine.toml"),
            include_str!("../../cards/object.toml"),
        );

        // Parse the card collection from TOML
        let library = toml::from_str::<CardLibrary>(toml).into_diagnostic()?;

        // Check for multiple uses of the same card ID
        {
            let mut seen = std::collections::HashSet::new();
            let mut duplicates = Vec::new();

            for c in &library.cards {
                if !seen.insert(c.id) {
                    duplicates.push(c.id);
                }
            }

            if !duplicates.is_empty() {
                let duplicates = duplicates
                    .into_iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(miette::miette!("Duplicate card IDs found: {duplicates}"));
            }
        }

        Ok(library)
    }

    pub(crate) fn len(&self) -> usize {
        self.cards.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter()
    }

    pub(crate) fn get(&self, id: &CardId) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == *id)
    }
}
