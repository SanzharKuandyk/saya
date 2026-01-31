mod client;
mod template;

pub use client::AnkiConnectClient;
pub use template::{CardTemplate, NoteField};

use anyhow::Result;

/// Add a card to Anki using the provided client and template
pub async fn add_card(
    client: &AnkiConnectClient,
    template: &CardTemplate,
    term: &str,
    reading: &str,
    definition: &str,
) -> Result<u64> {
    let front = template.format_front(term, reading, definition);
    let back = template.format_back(term, reading, definition);

    client
        .add_note(&template.deck, &template.model, &front, &back)
        .await
}
