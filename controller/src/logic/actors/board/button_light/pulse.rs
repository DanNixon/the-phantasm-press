use super::super::BoardActor;
use kameo::{
    Actor,
    actor::{ActorRef, Spawn, WeakActorRef},
    error::ActorStopReason,
    prelude::{Context, Message},
};
use kameo_actors::scheduler::{Scheduler, SetInterval};
use protocol::ButtonLedIntensity;
use std::{convert::Infallible, time::Duration};

#[derive(Clone)]
struct AnimationStep;

pub(crate) struct ButtonLedPulseActor {
    intensity: u8,
    increasing: bool,
    target: WeakActorRef<BoardActor>,
    scheduler: ActorRef<Scheduler>,
}

pub(crate) struct ButtonLedPulseActorArgs {
    target: WeakActorRef<BoardActor>,
}

impl ButtonLedPulseActorArgs {
    pub(crate) fn new(target: WeakActorRef<BoardActor>) -> Self {
        Self { target }
    }
}

impl Actor for ButtonLedPulseActor {
    type Args = ButtonLedPulseActorArgs;
    type Error = Infallible;

    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        let scheduler = Scheduler::spawn(Scheduler::new());
        scheduler
            .tell(SetInterval::new(
                actor_ref.downgrade(),
                Duration::from_millis(10),
                AnimationStep,
            ))
            .await
            .unwrap();

        Ok(Self {
            intensity: 0,
            increasing: true,
            target: args.target,
            scheduler,
        })
    }

    async fn on_stop(
        &mut self,
        _: WeakActorRef<Self>,
        _: ActorStopReason,
    ) -> Result<(), Self::Error> {
        let _ = self.scheduler.stop_gracefully().await;
        self.scheduler.wait_for_shutdown().await;

        Ok(())
    }
}

impl Message<AnimationStep> for ButtonLedPulseActor {
    type Reply = ();

    async fn handle(
        &mut self,
        _: AnimationStep,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if self.increasing {
            self.intensity += 1;
            if self.intensity >= 100 {
                self.intensity = 100;
                self.increasing = false;
            }
        } else {
            self.intensity -= 1;
            if self.intensity == 0 {
                self.increasing = true;
            }
        }

        self.target
            .upgrade()
            .unwrap()
            .tell(ButtonLedIntensity::new(self.intensity))
            .await
            .unwrap();
    }
}
