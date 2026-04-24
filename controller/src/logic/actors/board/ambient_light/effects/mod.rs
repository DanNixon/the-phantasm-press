pub(super) mod card_selection;
pub(super) mod story_generated;
pub(super) mod story_generating;
pub(super) mod story_generation_failed;

pub(super) use card_selection::CardSelectionEffect;
pub(super) use story_generated::StoryGeneratedEffect;
pub(super) use story_generating::StoryGeneratingEffect;
pub(super) use story_generation_failed::StoryGenerationFailedEffect;

use protocol::LED_COUNT;
use smart_leds::RGB8;

pub(super) trait BoardLightingEffect {
    fn step(&mut self) -> BoardLightingEffectStepResult;
    fn led_data(&self) -> [RGB8; LED_COUNT];
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum BoardLightingEffectStepResult {
    Continue,
    Finished,
}

#[derive(Clone)]
pub(super) struct EffectStep;
