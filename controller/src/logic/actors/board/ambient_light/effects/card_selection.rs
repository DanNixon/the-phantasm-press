use super::{BoardLightingEffect, BoardLightingEffectStepResult};
use protocol::LED_COUNT;
use smart_leds::RGB8;

/// Card selection effect with several slowly-evolving, overlapping elements.
///
/// The effect composes multiple "blobs" that drift around the ring at
/// different speeds, widths and base colours. Each blob also has a tiny
/// pseudo-random jitter that evolves over time so the pattern never looks
/// perfectly repeating. A subtle global background wave is added to keep
/// darker regions from being totally static.
#[derive(Debug, Default)]
pub(crate) struct CardSelectionEffect {
    /// time / phase counter (increments each step)
    phase: u32,
    /// simple LCG state used to generate low-frequency jitter per-element
    rng: u32,
}

impl BoardLightingEffect for CardSelectionEffect {
    fn step(&mut self) -> BoardLightingEffectStepResult {
        // advance time
        self.phase = self.phase.wrapping_add(1);
        // advance pseudo-random state (LCG)
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        BoardLightingEffectStepResult::Continue
    }

    fn led_data(&self) -> [RGB8; LED_COUNT] {
        // Parameters for the overlapping elements. Each element has:
        // - base_offset: starting angular offset (radians)
        // - speed: relative speed multiplier (bigger => faster)
        // - width: angular width (radians)
        // - base colour (r,g,b) as floats (max ~255)
        // - direction: 1.0 for clockwise, -1.0 for counter-clockwise
        let base_offsets: [f32; 4] = [
            0.0,
            core::f32::consts::PI * 0.6,
            core::f32::consts::PI * 1.4,
            core::f32::consts::PI * 2.0,
        ];
        let speeds: [f32; 4] = [0.02, 0.018, 0.015, 0.022]; // different slow speeds
        // Make some elements run in the opposite direction by using -1.0 entries here.
        let directions: [f32; 4] = [1.0, -1.0, 1.0, -1.0];
        let widths: [f32; 4] = [1.0, 1.5, 0.8, 1.1]; // angular widths (radians)
        let base_colors: [(f32, f32, f32); 4] = [
            (210.0, 50.0, 0.0),
            (0.0, 200.0, 100.0),
            (180.0, 120.0, 0.0),
            (200.0, 0.0, 200.0),
        ];

        let two_pi = core::f32::consts::PI * 2.0;
        let led_count_f = LED_COUNT as f32;

        // time base for this frame (scaled down so motion is slow)
        let t = (self.phase as f32) * 0.02;

        let mut data = [RGB8::default(); LED_COUNT];

        // helper: minimal angular distance between two angles in [0..PI]
        let min_ang_dist = |a: f32, b: f32| {
            let mut d = (a - b).abs();
            if d > core::f32::consts::PI {
                d = two_pi - d;
            }
            d
        };

        for (i, pixel) in data.iter_mut().enumerate() {
            let idx = i as f32;
            // LED position around the circle
            let led_angle = idx / led_count_f * two_pi;

            // accumulate float channels
            let mut rf = 0.0_f32;
            let mut gf = 0.0_f32;
            let mut bf = 0.0_f32;

            // combine contributions from each element
            for el in 0..4 {
                // element's instantaneous angle (wrap into 0..2PI)
                let mut angle = base_offsets[el] + (t * speeds[el] * directions[el]) * two_pi;
                // wrap into [0, 2PI)
                angle %= two_pi;
                if angle < 0.0 {
                    angle += two_pi;
                }
                let width = widths[el];

                let d = min_ang_dist(led_angle, angle);
                if d >= width {
                    continue;
                }
                // smooth envelope (squared ramp for nicer falloff)
                let s = 1.0 - (d / width);
                let env = s * s;

                // per-element slow colour modulation to add more motion
                let colour_mod = 0.6 + 0.4 * (t * 0.2 * (1.0 + el as f32)).sin();

                // compute raw per-channel contribution for this element
                let mut cr = base_colors[el].0 * env * colour_mod;
                let mut cg = base_colors[el].1 * env * colour_mod;
                let mut cb = base_colors[el].2 * env * colour_mod;

                // cap the element's peak proportionally so the largest channel <= ELEMENT_PEAK
                let element_peak = 40.0_f32;
                let max_chan = cr.max(cg).max(cb);
                if max_chan > element_peak && max_chan > 0.0 {
                    let scale = element_peak / max_chan;
                    cr *= scale;
                    cg *= scale;
                    cb *= scale;
                }

                // add the (scaled) per-element contributions
                rf += cr;
                gf += cg;
                bf += cb;
            }

            let r_u = rf.clamp(0.0, 255.0) as u8;
            let g_u = gf.clamp(0.0, 255.0) as u8;
            let b_u = bf.clamp(0.0, 255.0) as u8;

            *pixel = RGB8::new(r_u, g_u, b_u);
        }

        data
    }
}
