use crate::story::{Story, StoryOrigin};
use chrono::{DateTime, Utc};
use getset::Getters;
use log::{debug, warn};
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Response {
    choices: Vec<Choice>,
    created: Option<i64>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct StoryResponse {
    title: String,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub(crate) struct Usage {
    prompt_tokens: f64,
    completion_tokens: f64,
    cost: f64,
}

const MODEL: &str = "google/gemini-2.5-flash";
const SYSTEM_PROMPT: &str = include_str!("./system-prompt.txt");

pub(crate) async fn generate_story(
    http_client: &reqwest::Client,
    api_key: &str,
    origin: StoryOrigin,
) -> miette::Result<Story> {
    let request = serde_json::json!({
        "model": MODEL ,
        "messages": [
            {
                "role": "system",
                "content": SYSTEM_PROMPT,
            },
            {
                "role": "user",
                "content": origin.prompt(),
            },
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "story",
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Title of the story"
                        },
                        "text": {
                            "type": "string",
                            "description": "Story text (do not include the title or word count in this field)"
                        }
                    },
                    "required": [
                        "title",
                        "text"
                    ],
                    "additionalProperties": false,
                }
            }
        },
    });
    debug!("Request: {request:?}");

    let response = http_client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .into_diagnostic()?
        .error_for_status()
        .into_diagnostic()?;
    debug!("Response meta: {response:#?}");

    let response: Response = response.json().await.into_diagnostic()?;
    debug!("Response: {response:#?}");

    let story_json = response
        .choices
        .first()
        .ok_or(miette::miette!("No choices in response"))?
        .clone()
        .message
        .content;
    let story: StoryResponse = serde_json::from_str(&story_json).into_diagnostic()?;

    // Use the OpenRouter response timestamp if present (seconds since epoch),
    // otherwise fall back to the current time.
    let timestamp = response
        .created
        .and_then(|secs| DateTime::from_timestamp(secs, 0).map(|t| t.with_timezone(&Utc)))
        .unwrap_or_else(|| {
            warn!("Failed to parse OpenRouter timestamp, using current time instead");
            Utc::now()
        });

    Ok(Story::new(
        origin,
        timestamp,
        story.title,
        story.text,
        response.model,
        response.usage,
    ))
}
