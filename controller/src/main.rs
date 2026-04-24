mod board;
mod card;
mod card_art_generation;
mod logic;
mod openrouter;
mod printer;
mod story;

use crate::{
    card::CardLibrary,
    openrouter::generate_story,
    printer::PrinterExt,
    story::{Story, StoryOrigin},
};
use clap::{Parser, Subcommand};
use hex::FromHex;
use miette::IntoDiagnostic;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version = format!("v{} git={}", clap::crate_version!(), git_version::git_version!()), about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run {
        /// OpenRouter API key
        #[clap(long, env = "OPENROUTER_API_KEY")]
        openrouter_api_key: String,

        /// Directory to save generated stories in
        #[clap(short = 'o', long = "output", env = "STORY_OUTPUT_DIRECTORY")]
        output_dir: PathBuf,

        #[clap(long, env = "PRINTER_SERIAL_PORT")]
        printer_port: Option<String>,

        #[clap(long, default_value = "19200")]
        printer_baud_rate: u32,
    },

    /// Print out all the loaded data
    Data,

    /// Generate art for a given card
    CardArtGen {
        /// OpenRouter API key
        #[clap(long, env = "OPENROUTER_API_KEY")]
        openrouter_api_key: String,

        /// Directory to save the generated image in
        #[clap(short = 'o', long = "output")]
        output_dir: PathBuf,

        /// ID of the card to generate art for, as a hex string (e.g. "99aabbcc")
        #[clap(short = 'c', long = "card")]
        card_id: String,

        /// Extra text to be included in the prompt for art generation
        #[clap(long)]
        extra_prompt: Option<String>,
    },

    /// Generates a story based on a selection of cards
    GenerateStory {
        /// OpenRouter API key
        #[clap(long, env = "OPENROUTER_API_KEY")]
        openrouter_api_key: String,

        /// Directory to save the generated story in
        #[clap(short = 'o', long = "output", env = "STORY_OUTPUT_DIRECTORY")]
        output_dir: PathBuf,

        /// Card IDs to include in the story, as hex strings (e.g. "99aabbcc")
        cards: Vec<String>,
    },

    /// Pretty print a generated story to the console
    PrintStory {
        #[clap(long)]
        printer_port: Option<String>,

        #[clap(long, default_value = "38400")]
        printer_baud_rate: u32,

        story_file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    env_logger::init();

    let args = Cli::parse();

    match args.command {
        Command::Run {
            openrouter_api_key,
            output_dir,
            printer_port,
            printer_baud_rate,
        } => {
            crate::logic::run_main(
                openrouter_api_key,
                output_dir,
                printer_port,
                printer_baud_rate,
            )
            .await?;
        }

        Command::Data => {
            println!("Git revision:\n{}", git_version::git_version!());
            println!();

            let card_library = CardLibrary::load()?;
            println!("Cards ({}):", card_library.len());
            for c in card_library.iter() {
                println!("- id: {} ({:?})", c.id(), c.id());
                println!("  kind: {:?}", c.kind());
                println!("  title: {}", c.title());
                println!("  description: {}", c.description());
            }
        }

        Command::CardArtGen {
            openrouter_api_key,
            output_dir,
            card_id,
            extra_prompt,
        } => {
            return crate::card_art_generation::run(
                card_id,
                &output_dir,
                &openrouter_api_key,
                extra_prompt,
            )
            .await;
        }

        Command::GenerateStory {
            openrouter_api_key,
            output_dir,
            cards,
        } => {
            let library = CardLibrary::load()?;

            // Build the StoryOrigin from the provided card hex IDs.
            // Parse each hex string into a CardId and look it up in the library.
            let mut selected_cards = Vec::new();
            for hex in cards {
                // Convert the provided hex string into a CardId
                let card_id = protocol::CardId::from_hex(hex).into_diagnostic()?;

                // Look up the card in the loaded library
                let card = library
                    .get(&card_id)
                    .ok_or_else(|| miette::miette!("Card with ID {card_id} not found"))?;

                // Clone and collect the selected card
                selected_cards.push(card.clone());
            }

            let origin = StoryOrigin::new(selected_cards);

            // Generate the story
            let story =
                generate_story(&reqwest::Client::new(), &openrouter_api_key, origin).await?;

            // Pretty print the story to the console
            println!("{story}");

            // Save the story to a file in the output directory
            story.save_in_dir(&output_dir)?;
        }

        Command::PrintStory {
            printer_port,
            printer_baud_rate,
            story_file,
        } => {
            // Load the story from the specified file
            let story_json = std::fs::read_to_string(&story_file).into_diagnostic()?;
            let story: Story = serde_json::from_str(&story_json).into_diagnostic()?;

            // Pretty print the story to the console
            println!("{story}");

            // If a printer port was specified, also print the story to the printer
            if let Some(port) = printer_port {
                let mut printer =
                    crate::printer::init(&port, printer_baud_rate).into_diagnostic()?;
                printer.print_story(&story).into_diagnostic()?;
            }
        }
    }

    Ok(())
}
