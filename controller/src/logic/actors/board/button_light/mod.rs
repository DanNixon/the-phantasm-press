mod pulse;

use super::BoardActor;
use kameo::{
    Actor,
    actor::{ActorRef, Spawn, WeakActorRef},
    prelude::{Context, Message},
};
use protocol::ButtonLedIntensity;
use pulse::{ButtonLedPulseActor, ButtonLedPulseActorArgs};
use std::convert::Infallible;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonLedMode {
    Off,
    Pulsing,
}

pub(crate) struct ButtonLedActor {
    target: WeakActorRef<BoardActor>,
    mode: ButtonLedMode,
    pulse_actor: Option<ActorRef<ButtonLedPulseActor>>,
}

pub(crate) struct ButtonLedActorArgs {
    target: WeakActorRef<BoardActor>,
}

impl ButtonLedActorArgs {
    pub(crate) fn new(target: WeakActorRef<BoardActor>) -> Self {
        Self { target }
    }
}

impl Actor for ButtonLedActor {
    type Args = ButtonLedActorArgs;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        // Ensure LED is off at start
        args.target
            .upgrade()
            .unwrap()
            .tell(ButtonLedIntensity::OFF)
            .await
            .unwrap();
        Ok(Self {
            target: args.target,
            mode: ButtonLedMode::Off,
            pulse_actor: None,
        })
    }
}

impl Message<ButtonLedMode> for ButtonLedActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ButtonLedMode,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // If the mode is the same as the current mode, do nothing
        if self.mode == msg {
            return;
        }

        self.mode = msg;

        match self.mode {
            ButtonLedMode::Off => {
                self.pulse_actor = None;

                if let Some(target) = self.target.upgrade() {
                    target.tell(ButtonLedIntensity::OFF).await.unwrap();
                }
            }
            ButtonLedMode::Pulsing => {
                let actor =
                    ButtonLedPulseActor::spawn(ButtonLedPulseActorArgs::new(self.target.clone()));
                self.pulse_actor = Some(actor);
            }
        }
    }
}
