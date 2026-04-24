pub(crate) mod ambient_light;
pub(crate) mod button_light;

use crate::board::Board;
use kameo::{
    Actor,
    actor::{ActorRef, WeakActorRef},
    error::ActorStopReason,
    prelude::{Context, Message},
};
use kameo_actors::message_bus::{MessageBus, Publish};
use log::{debug, info, warn};
use protocol::{ButtonLedIntensity, Command, LedData};
use std::convert::Infallible;
use tokio::task::JoinHandle;

pub(crate) struct BoardActor {
    board: Board,
    state_poll_handle: Option<JoinHandle<()>>,
}

pub(crate) struct BoardActorArgs {
    event_bus: WeakActorRef<MessageBus>,
}

impl BoardActorArgs {
    pub(crate) fn new(event_bus: WeakActorRef<MessageBus>) -> Self {
        Self { event_bus }
    }
}

impl Actor for BoardActor {
    type Args = BoardActorArgs;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        let board = Board::default();

        let state_poll_handle = tokio::spawn({
            let board = board.clone();

            async move {
                loop {
                    match board.receive_state().await {
                        Ok(state) => {
                            debug!("Received state: {state:?}");
                            match state {
                                protocol::State::Uptime(duration) => {
                                    debug!("Board has been up for {duration:?}");
                                }
                                protocol::State::CardReaderInit(card_reader) => {
                                    info!("Card reader initialized: {card_reader:?}");
                                }
                                protocol::State::CardReader(card_reader_state) => {
                                    if let Some(event_bus) = args.event_bus.upgrade() {
                                        event_bus.tell(Publish(card_reader_state)).await.unwrap();
                                    }
                                }
                                protocol::State::Button(button_state) => {
                                    if let Some(event_bus) = args.event_bus.upgrade() {
                                        event_bus.tell(Publish(button_state)).await.unwrap();
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Board communication error: {e}");
                            board.open().await;
                        }
                    }
                }
            }
        });

        Ok(Self {
            board,
            state_poll_handle: Some(state_poll_handle),
        })
    }

    async fn on_stop(
        &mut self,
        _: WeakActorRef<Self>,
        _: ActorStopReason,
    ) -> Result<(), Self::Error> {
        // Stop the state polling task
        if let Some(handle) = self.state_poll_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        Ok(())
    }
}

impl Message<ButtonLedIntensity> for BoardActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ButtonLedIntensity,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Err(e) = self.board.send_command(Command::SetButtonLed(msg)).await {
            warn!("Failed to set button LED intensity: {e}");
        }
    }
}

impl Message<LedData> for BoardActor {
    type Reply = ();

    async fn handle(&mut self, msg: LedData, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        if let Err(e) = self.board.send_command(Command::SetLights(msg)).await {
            warn!("Failed to set lights: {e}");
        }
    }
}
