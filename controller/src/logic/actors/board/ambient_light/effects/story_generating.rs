use super::{BoardLightingEffect, BoardLightingEffectStepResult};
use protocol::LED_COUNT;
use smart_leds::RGB8;

#[derive(Debug, Default)]
pub(crate) struct StoryGeneratingEffect {
    phase: u16,
}

impl BoardLightingEffect for StoryGeneratingEffect {
    fn step(&mut self) -> BoardLightingEffectStepResult {
        self.phase = self.phase.wrapping_add(1);
        BoardLightingEffectStepResult::Continue
    }

    fn led_data(&self) -> [RGB8; LED_COUNT] {
        // Rotation parameters: faster than the pulse so pulses sweep around the strip.
        let rotation_period: u16 = 120; // smaller => faster rotation
        let p_rot = (self.phase % rotation_period) as f32;
        let rotation = p_rot / (rotation_period as f32) * (core::f32::consts::PI * 2.0);

        // We'll place three pulses:
        // A: at `rotation` (forward)
        // B: forward with offset (rotation + offset)
        // C: reverse (rotation reversed) with a different offset
        let two_pi = core::f32::consts::PI * 2.0;
        let offset_b = two_pi / 3.0; // stagger B by 120 degrees
        let offset_c = two_pi * 2.0 / 3.0; // stagger C by 240 degrees

        let angle_a = rotation % two_pi;
        let angle_b = (rotation + offset_b) % two_pi;
        // reverse: use -rotation (i.e. two_pi - rotation) then add offset_c
        let angle_c = (two_pi - rotation + offset_c) % two_pi;

        // angular width of each pulse (radians). Smaller => tighter spot.
        let width = 0.6_f32;

        // Base maxima for each pulse (you can tweak to balance colors)
        let base_max_a = 170.0_f32;
        let base_max_b = 140.0_f32;
        let base_max_c = 140.0_f32;

        let mut data = [RGB8::default(); LED_COUNT];

        // helper: minimal angular distance between two angles, result in [0..PI]
        let min_ang_dist = |a: f32, b: f32| {
            let mut d = (a - b).abs();
            if d > core::f32::consts::PI {
                d = two_pi - d;
            }
            d
        };

        let led_count_f = LED_COUNT as f32;

        for (i, pixel) in data.iter_mut().enumerate() {
            let idx = i as f32;
            // convert LED index to angle around the circle
            let led_angle = idx / led_count_f * two_pi;

            // compute envelope for each pulse (0..1), smoother peak via squared ramp
            let env = |angle_center: f32| {
                let d = min_ang_dist(led_angle, angle_center);
                if d >= width {
                    0.0
                } else {
                    let t = 1.0 - (d / width);
                    // squared curve for smoother falloff
                    t * t
                }
            };

            let env_a = env(angle_a);
            let env_b = env(angle_b);
            let env_c = env(angle_c);

            // intensity contributions (apply the same global pulsing amplitude)
            let ia = base_max_a * env_a;
            let ib = base_max_b * env_b;
            let ic = base_max_c * env_c;

            // Colors:
            // - A: cyan  (0, G, B)
            // - B: magenta (R, 0, B)
            // - C: yellow (R, G, 0)
            let mut r_f = 0.0_f32;
            let mut g_f = 0.0_f32;
            let mut b_f = 0.0_f32;

            // add contributions
            g_f += ia;
            b_f += ia;

            r_f += ib;
            b_f += ib;

            r_f += ic;
            g_f += ic;

            let r = r_f.clamp(0.0, 255.0) as u8;
            let g = g_f.clamp(0.0, 255.0) as u8;
            let b = b_f.clamp(0.0, 255.0) as u8;

            *pixel = RGB8::new(r, g, b);
        }

        data
    }
}
