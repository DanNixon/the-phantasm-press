mod actors;

use crate::{
    card::Card,
    openrouter::generate_story,
    story::{Story, StoryOrigin},
};
use actors::{
    board::{
        BoardActor, BoardActorArgs,
        ambient_light::{BoardLightingActor, BoardLightingActorArgs},
        button_light::{ButtonLedActor, ButtonLedActorArgs, ButtonLedMode},
    },
    card_lookup::{CardLookupActor, CardLookupActorArgs},
    printer::PrinterActor,
};
use core::time::Duration;
use kameo::{
    Actor,
    actor::{ActorRef, Spawn, WeakActorRef},
    prelude::{Context, Message},
};
use kameo_actors::{
    DeliveryStrategy,
    message_bus::{MessageBus, Publish, Register},
};
use log::{info, warn};
use miette::IntoDiagnostic;
use protocol::{ButtonState, CardReader, CardReaderState};
use std::{collections::HashMap, convert::Infallible, path::PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct CardsInReaders(HashMap<CardReader, ReaderState>);

impl Default for CardsInReaders {
    fn default() -> Self {
        Self(
            [
                CardReader::One,
                CardReader::Two,
                CardReader::Three,
                CardReader::Four,
            ]
            .iter()
            .map(|reader| (*reader, ReaderState::NoCard))
            .collect(),
        )
    }
}

#[derive(Debug, Clone)]
enum ReaderState {
    NoCard,
    UnknownCard,
    Card(Card),
}

struct TriggerActor {
    event_bus: WeakActorRef<MessageBus>,
    story_is_generating: bool,
    cards: CardsInReaders,
}

struct TriggerActorArgs {
    event_bus: WeakActorRef<MessageBus>,
}

impl Actor for TriggerActor {
    type Args = TriggerActorArgs;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(Self {
            event_bus: args.event_bus,
            story_is_generating: false,
            cards: CardsInReaders::default(),
        })
    }
}

impl TriggerActor {
    fn can_be_triggered(&self) -> bool {
        !self.story_is_generating
            && self
                .cards
                .0
                .values()
                .any(|state| matches!(state, ReaderState::Card(_)))
    }

    async fn set_button_mode(&self) {
        let mode = if self.can_be_triggered() {
            ButtonLedMode::Pulsing
        } else {
            ButtonLedMode::Off
        };

        self.event_bus
            .upgrade()
            .unwrap()
            .tell(Publish(mode))
            .await
            .unwrap();
    }
}

impl Message<ButtonState> for TriggerActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ButtonState,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if self.can_be_triggered()
            && let ButtonState::Released(duration) = msg
            && duration > Duration::from_millis(80)
        {
            info!("Triggering story generation");
            self.story_is_generating = true;

            self.set_button_mode().await;

            let origin = StoryOrigin::new(
                self.cards
                    .0
                    .values()
                    .filter_map(|state| {
                        if let ReaderState::Card(card) = state {
                            Some(card)
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .collect(),
            );
            info!("Story origin: {origin:#?}");

            self.event_bus
                .upgrade()
                .unwrap()
                .tell(Publish(origin))
                .await
                .unwrap();
        }
    }
}

impl Message<CardsInReaders> for TriggerActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: CardsInReaders,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Just record the current cards
        self.cards = msg;

        self.set_button_mode().await;
    }
}

impl Message<Story> for TriggerActor {
    type Reply = ();

    async fn handle(&mut self, _: Story, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // Story generation is done once we receive a story, so we can allow triggering again
        self.story_is_generating = false;

        self.set_button_mode().await;
    }
}

impl Message<StoryGenerationFailed> for TriggerActor {
    type Reply = ();

    async fn handle(
        &mut self,
        _: StoryGenerationFailed,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Story generation failed, so we can allow triggering again
        self.story_is_generating = false;

        self.set_button_mode().await;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StoryGenerationFailed;

struct StoryGenerationActor {
    http_client: reqwest::Client,
    openrouter_api_key: String,
    output_dir: PathBuf,
    event_bus: WeakActorRef<MessageBus>,
}

struct StoryGenerationActorArgs {
    openrouter_api_key: String,
    output_dir: PathBuf,
    event_bus: WeakActorRef<MessageBus>,
}

impl Actor for StoryGenerationActor {
    type Args = StoryGenerationActorArgs;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(Self {
            http_client: reqwest::Client::new(),
            openrouter_api_key: args.openrouter_api_key,
            output_dir: args.output_dir,
            event_bus: args.event_bus,
        })
    }
}

impl Message<StoryOrigin> for StoryGenerationActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: StoryOrigin,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match generate_story(&self.http_client, &self.openrouter_api_key, msg).await {
            Ok(story) => {
                // Save the story to disk
                if let Err(e) = story.save_in_dir(&self.output_dir) {
                    warn!("Failed to save story: {e}");
                }

                // Publish the story to the event bus
                self.event_bus
                    .upgrade()
                    .unwrap()
                    .tell(Publish(story))
                    .await
                    .unwrap();
            }
            Err(e) => {
                warn!("Failed to generate story: {e}");

                // Notify the rest of the system that story generation failed
                self.event_bus
                    .upgrade()
                    .unwrap()
                    .tell(Publish(StoryGenerationFailed))
                    .await
                    .unwrap();
            }
        }
    }
}

pub(super) async fn run_main(
    openrouter_api_key: String,
    output_dir: PathBuf,
    printer_port: Option<String>,
    printer_baud_rate: u32,
) -> miette::Result<()> {
    let event_bus = MessageBus::spawn(MessageBus::new(DeliveryStrategy::Guaranteed));

    // Hardware interface actors
    let board_actor = BoardActor::spawn(BoardActorArgs::new(event_bus.downgrade()));
    let printer_actor = match printer_port {
        Some(port) => {
            let printer_actor = PrinterActor::spawn(PrinterActor::new(&port, printer_baud_rate)?);

            // The printer will print a `Story` that is published to the event bus
            event_bus
                .tell(Register(printer_actor.clone().recipient::<Story>()))
                .await
                .into_diagnostic()?;

            Some(printer_actor)
        }
        None => {
            warn!("No printer serial port specified, will not use a printer");
            None
        }
    };

    let card_lookup_actor = CardLookupActor::spawn(CardLookupActorArgs::new(event_bus.downgrade()));

    // Card lookup actor handles card reader events
    event_bus
        .tell(Register(
            card_lookup_actor.clone().recipient::<CardReaderState>(),
        ))
        .await
        .into_diagnostic()?;

    let trigger_actor = TriggerActor::spawn(TriggerActorArgs {
        event_bus: event_bus.downgrade(),
    });

    // Trigger actor handles button presses
    event_bus
        .tell(Register(trigger_actor.clone().recipient::<ButtonState>()))
        .await
        .into_diagnostic()?;

    // Trigger actor also needs to know what cards are in the readers
    event_bus
        .tell(Register(
            trigger_actor.clone().recipient::<CardsInReaders>(),
        ))
        .await
        .into_diagnostic()?;

    // Trigger actor needs to know when story generation is done
    event_bus
        .tell(Register(trigger_actor.clone().recipient::<Story>()))
        .await
        .into_diagnostic()?;

    // Trigger actor needs to know when story generation failed
    event_bus
        .tell(Register(
            trigger_actor.clone().recipient::<StoryGenerationFailed>(),
        ))
        .await
        .into_diagnostic()?;

    let story_generation_actor = StoryGenerationActor::spawn(StoryGenerationActorArgs {
        openrouter_api_key,
        output_dir,
        event_bus: event_bus.downgrade(),
    });

    // A `StoryOrigin` published to the event bus will trigger generation of a `Story`
    event_bus
        .tell(Register(
            story_generation_actor.clone().recipient::<StoryOrigin>(),
        ))
        .await
        .into_diagnostic()?;

    let button_led_actor = ButtonLedActor::spawn(ButtonLedActorArgs::new(board_actor.downgrade()));

    // The button LED actor listens for `ButtonLedMode` messages to know how to set the LED
    event_bus
        .tell(Register(
            button_led_actor.clone().recipient::<ButtonLedMode>(),
        ))
        .await
        .into_diagnostic()?;

    let lighting_actor =
        BoardLightingActor::spawn(BoardLightingActorArgs::new(board_actor.downgrade()));

    // Lighting actor needs to know what cards are in the readers to set the appropriate lights
    event_bus
        .tell(Register(
            lighting_actor.clone().recipient::<CardsInReaders>(),
        ))
        .await
        .into_diagnostic()?;

    // Lighting actor also listens for `StoryOrigin` to set appropriate lighting when the story is being generated
    event_bus
        .tell(Register(lighting_actor.clone().recipient::<StoryOrigin>()))
        .await
        .into_diagnostic()?;

    // Lighting actor also listens for `Story` to set appropriate lighting when the story generation is done
    event_bus
        .tell(Register(lighting_actor.clone().recipient::<Story>()))
        .await
        .into_diagnostic()?;

    // Lighting actor also listens for `StoryGenerationFailed` to set appropriate lighting when the story generation fails
    event_bus
        .tell(Register(
            lighting_actor.clone().recipient::<StoryGenerationFailed>(),
        ))
        .await
        .into_diagnostic()?;

    tokio::signal::ctrl_c().await.into_diagnostic()?;
    eprintln!("Shutting down...");

    macro_rules! shutdown {
        ($a: expr) => {
            info!("Stopping {}", stringify!($a));
            $a.stop_gracefully().await.unwrap();
            $a.wait_for_shutdown().await;
        };
    }
    shutdown!(event_bus);
    shutdown!(board_actor);
    if let Some(printer_actor) = printer_actor {
        shutdown!(printer_actor);
    }
    shutdown!(card_lookup_actor);
    shutdown!(trigger_actor);
    shutdown!(story_generation_actor);
    shutdown!(button_led_actor);
    shutdown!(lighting_actor);

    Ok(())
}
