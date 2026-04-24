mod effects;

use super::BoardActor;
use crate::{
    logic::{CardsInReaders, ReaderState, StoryGenerationFailed},
    story::{Story, StoryOrigin},
};
use core::convert::Infallible;
use effects::{
    BoardLightingEffect, BoardLightingEffectStepResult, CardSelectionEffect, EffectStep,
    StoryGeneratedEffect, StoryGeneratingEffect, StoryGenerationFailedEffect,
};
use kameo::{
    Actor,
    actor::{ActorRef, Spawn, WeakActorRef},
    error::ActorStopReason,
    prelude::{Context, Message},
};
use kameo_actors::scheduler::{Scheduler, SetInterval};
use log::trace;
use protocol::{CardReader, LED_COUNT, LedData};
use smart_leds::RGB8;
use std::time::Duration;

#[derive(Debug)]
enum BoardLightingMode {
    CardSelection(CardSelectionEffect),
    StoryGenerating(StoryGeneratingEffect),
    StoryGenerated(StoryGeneratedEffect),
    StoryGenerationFailed(StoryGenerationFailedEffect),
}

impl BoardLightingEffect for BoardLightingMode {
    fn step(&mut self) -> BoardLightingEffectStepResult {
        match self {
            BoardLightingMode::CardSelection(effect) => effect.step(),
            BoardLightingMode::StoryGenerating(effect) => effect.step(),
            BoardLightingMode::StoryGenerated(effect) => effect.step(),
            BoardLightingMode::StoryGenerationFailed(effect) => effect.step(),
        }
    }

    fn led_data(&self) -> [RGB8; LED_COUNT] {
        match self {
            BoardLightingMode::CardSelection(effect) => effect.led_data(),
            BoardLightingMode::StoryGenerating(effect) => effect.led_data(),
            BoardLightingMode::StoryGenerated(effect) => effect.led_data(),
            BoardLightingMode::StoryGenerationFailed(effect) => effect.led_data(),
        }
    }
}

pub(crate) struct BoardLightingActor {
    target: WeakActorRef<BoardActor>,
    mode: BoardLightingMode,
    scheduler: ActorRef<Scheduler>,
    cards: CardsInReaders,
}

pub(crate) struct BoardLightingActorArgs {
    target: WeakActorRef<BoardActor>,
}

impl BoardLightingActorArgs {
    pub(crate) fn new(target: WeakActorRef<BoardActor>) -> Self {
        Self { target }
    }
}

impl Actor for BoardLightingActor {
    type Args = BoardLightingActorArgs;
    type Error = Infallible;

    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        let scheduler = Scheduler::spawn(Scheduler::new());
        scheduler
            .tell(SetInterval::new(
                actor_ref.downgrade(),
                Duration::from_millis(10),
                effects::EffectStep,
            ))
            .await
            .unwrap();

        Ok(Self {
            target: args.target,
            mode: BoardLightingMode::CardSelection(Default::default()),
            scheduler,
            cards: CardsInReaders::default(),
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

impl BoardLightingActor {
    pub(crate) async fn set_lights(&mut self) {
        let mut data = self.mode.led_data();

        if matches!(self.mode, BoardLightingMode::CardSelection(_)) {
            for (reader, leds) in [
                (CardReader::One, [17, 18, 19, 20]),
                (CardReader::Two, [2, 3, 4, 5]),
                (CardReader::Three, [47, 48, 49, 50]),
                (CardReader::Four, [31, 32, 33, 34]),
            ] {
                let pixel = match self.cards.0.get(&reader).unwrap() {
                    ReaderState::NoCard => RGB8::default(),
                    ReaderState::UnknownCard => RGB8::new(255, 0, 0),
                    ReaderState::Card(_) => RGB8::new(127, 100, 100),
                };

                for led in leds {
                    data[led] = pixel;
                }
            }
        }

        trace!("LED data: {data:?}");
        if let Some(target) = self.target.upgrade() {
            target.tell(LedData { data: data.into() }).await.unwrap();
        }
    }
}

impl Message<EffectStep> for BoardLightingActor {
    type Reply = ();

    async fn handle(&mut self, _: EffectStep, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let res = self.mode.step();

        if res == BoardLightingEffectStepResult::Finished
            && matches!(
                self.mode,
                BoardLightingMode::StoryGenerated(_) | BoardLightingMode::StoryGenerationFailed(_)
            )
        {
            self.mode = BoardLightingMode::CardSelection(Default::default());
        }

        self.set_lights().await;
    }
}

impl Message<CardsInReaders> for BoardLightingActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: CardsInReaders,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Record the new card state
        self.cards = msg;

        self.set_lights().await;
    }
}

impl Message<StoryOrigin> for BoardLightingActor {
    type Reply = ();

    async fn handle(&mut self, _: StoryOrigin, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // Record the fact that the story is now generating
        self.mode = BoardLightingMode::StoryGenerating(Default::default());

        self.set_lights().await;
    }
}

impl Message<Story> for BoardLightingActor {
    type Reply = ();

    async fn handle(&mut self, _: Story, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // Record the fact that the story has been generated
        self.mode = BoardLightingMode::StoryGenerated(Default::default());

        self.set_lights().await;
    }
}

impl Message<StoryGenerationFailed> for BoardLightingActor {
    type Reply = ();

    async fn handle(
        &mut self,
        _: StoryGenerationFailed,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Record the fact that story generation has failed
        self.mode = BoardLightingMode::StoryGenerationFailed(Default::default());

        self.set_lights().await;
    }
}
