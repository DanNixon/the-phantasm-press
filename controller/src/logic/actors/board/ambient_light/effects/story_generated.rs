use super::{BoardLightingEffect, BoardLightingEffectStepResult};
use protocol::LED_COUNT;
use smart_leds::RGB8;

#[derive(Debug)]
enum Stage {
    PulseUp { intensity: u8 },
    FadeOut { intensity: u8 },
    Finished,
}

#[derive(Debug)]
pub(crate) struct StoryGeneratedEffect {
    stage: Stage,
}

impl Default for StoryGeneratedEffect {
    fn default() -> Self {
        Self {
            stage: Stage::PulseUp { intensity: 20 },
        }
    }
}

impl BoardLightingEffect for StoryGeneratedEffect {
    fn step(&mut self) -> BoardLightingEffectStepResult {
        match &mut self.stage {
            Stage::PulseUp { intensity } => {
                *intensity = intensity.saturating_add(2);
                if *intensity >= 127 {
                    self.stage = Stage::FadeOut { intensity: 127 };
                }
                BoardLightingEffectStepResult::Continue
            }
            Stage::FadeOut { intensity } => {
                *intensity -= 2;
                if *intensity <= 20 {
                    self.stage = Stage::Finished;
                }
                BoardLightingEffectStepResult::Continue
            }
            Stage::Finished => BoardLightingEffectStepResult::Finished,
        }
    }

    fn led_data(&self) -> [RGB8; LED_COUNT] {
        let mut data = [RGB8::default(); LED_COUNT];

        match self.stage {
            Stage::PulseUp { intensity } | Stage::FadeOut { intensity } => {
                for pixel in data.iter_mut() {
                    *pixel = RGB8::new(0, intensity, 0);
                }
            }
            Stage::Finished => {}
        }

        data
    }
}
