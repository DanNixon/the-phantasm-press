use crate::card::CardLibrary;
use base64::{Engine, engine::general_purpose::STANDARD};
use hex::FromHex;
use log::{debug, info};
use miette::{IntoDiagnostic, miette};
use protocol::CardId;

pub(super) async fn run(
    card_id: String,
    output_dir: &std::path::Path,
    openrouter_api_key: &str,
    extra_prompt: Option<String>,
) -> miette::Result<()> {
    let library = CardLibrary::load()?;

    // Get the card to generate art for
    let card_id = CardId::from_hex(card_id).into_diagnostic()?;
    let card = library
        .get(&card_id)
        .ok_or(miette::miette!("Card with ID {card_id} not found"))?;
    info!("Selected: {card:?}");

    let filename = format!("{:?} - {}", card.kind(), card.title());
    info!("Filename: {filename}");

    // Ensure the output directory exists
    std::fs::create_dir_all(output_dir).into_diagnostic()?;

    // Fail early if an image for this card already exists
    for entry in std::fs::read_dir(output_dir).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let stem = std::path::Path::new(&name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name.as_str());
        if stem == filename.as_str() {
            return Err(miette::miette!("Image already exists for card {card_id}"));
        }
    }

    // Build the prompt
    let prompt = format!(
        "Adjust the text and add a picture as per the following information:\nType: {:?}\nTitle: {}\nDescription: {}{}\n\nThe type and title should be in uppercase. The description should be left justified.\n{}",
        card.kind(),
        card.title(),
        card.description(),
        match card.extra_description() {
            Some(extra_description) => format!("\n\nExtra details: {extra_description}"),
            None => "".into(),
        },
        extra_prompt.unwrap_or_default(),
    );
    info!("Prompt: \"{prompt}\"");

    // Build the request
    let request = serde_json::json!({
        "model": MODEL ,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": prompt,
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", STANDARD.encode(TEMPLATE_IMAGE_BYTES)),
                        }
                    }
                ],
            },
        ],
    });
    debug!("Request: {request:?}");

    // Make the request
    let response = reqwest::Client::new()
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", openrouter_api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .into_diagnostic()?
        .error_for_status()
        .into_diagnostic()?;
    debug!("Response meta: {response:#?}");

    // Parse the response and extract the image
    let response: serde_json::Value = response.json().await.into_diagnostic()?;
    debug!("Response: {response:#?}");

    let image_url = response["choices"][0]["message"]["images"][0]["image_url"]["url"]
        .as_str()
        .ok_or_else(|| miette!("Image URL not found in response"))?;
    let (extension, image_data) = parse_data_url(image_url)?;

    // Save the output image
    let filename = format!("{filename}.{extension}");
    let path = output_dir.join(&filename);
    std::fs::write(&path, image_data).into_diagnostic()?;
    info!("Saved image to {}", path.display());

    Ok(())
}

const MODEL: &str = "google/gemini-3-pro-image-preview";
const TEMPLATE_IMAGE_BYTES: &[u8] = include_bytes!("../../cards/template.png");

fn parse_data_url(url: &str) -> miette::Result<(&str, Vec<u8>)> {
    let url = url
        .strip_prefix("data:")
        .ok_or_else(|| miette!("Invalid data URL format: missing 'data:' prefix"))?;
    let (mime_and_ext, rest) = url
        .split_once(';')
        .ok_or_else(|| miette!("Invalid data URL format: missing mime/ext"))?;
    if !rest.starts_with("base64,") {
        return Err(miette!("Missing base64 marker in data URL"));
    }
    let data = rest.strip_prefix("base64,").unwrap();

    let (mime_type, extension) = mime_and_ext
        .split_once('/')
        .ok_or_else(|| miette!("Invalid data URL format: mime/ext is not valid"))?;
    if mime_type != "image" {
        return Err(miette!("Unsupported scheme in data URL: {}", mime_type));
    }

    let image_data = STANDARD.decode(data).into_diagnostic()?;
    Ok((extension, image_data))
}
