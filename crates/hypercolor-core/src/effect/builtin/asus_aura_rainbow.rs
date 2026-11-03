//! ASUS Aura Rainbow renderer — configurable moving vertical rainbow bands.
//!
//! This is a visual approximation of the ASUS Aura Gen 3 cold-boot rainbow,
//! not an implementation of unknown proprietary firmware behavior.

use std::path::PathBuf;

use hypercolor_color::Hsv;
use hypercolor_types::canvas::{BYTES_PER_PIXEL, Canvas};
use hypercolor_types::control::{ControlDeltaBatch, ControlValue};
use hypercolor_types::effect::{
    ControlDefinition, EffectCategory, EffectMetadata, EffectSource, PresetTemplate,
};

use super::common::{builtin_effect_id, dropdown_control, preset_with_desc, slider_control};
use crate::effect::traits::{EffectRenderer, FrameInput, prepare_target_canvas};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuraDirection {
    Left,
    Right,
}

impl AuraDirection {
    fn from_str(value: &str) -> Self {
        if value.trim().eq_ignore_ascii_case("left") {
            Self::Left
        } else {
            Self::Right
        }
    }

    const fn motion_sign(self) -> f64 {
        match self {
            Self::Left => 1.0,
            Self::Right => -1.0,
        }
    }
}

/// Renders straight vertical rainbow bands moving horizontally at a fixed rate.
pub struct AsusAuraRainbowRenderer {
    band_density: f64,
    cycle_duration_secs: f64,
    direction: AuraDirection,
    brightness: f32,
    saturation: f32,
    start_hue_degrees: f64,
}

impl AsusAuraRainbowRenderer {
    /// Create an ASUS Aura Rainbow renderer with the catalog defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            band_density: 5.0,
            cycle_duration_secs: 12.0,
            direction: AuraDirection::Right,
            brightness: 0.7,
            saturation: 1.0,
            start_hue_degrees: 0.0,
        }
    }

    fn hue_at(&self, x: usize, width: f64, time_secs: f64) -> f64 {
        let time_secs = if time_secs.is_finite() {
            time_secs
        } else {
            0.0
        };
        let elapsed_in_cycle = time_secs.rem_euclid(self.cycle_duration_secs);
        let temporal_phase = elapsed_in_cycle / self.cycle_duration_secs * 360.0;
        #[allow(clippy::cast_precision_loss, clippy::as_conversions)]
        let spatial_phase = x as f64 / width * self.band_density * 360.0;

        (self.start_hue_degrees + spatial_phase + self.direction.motion_sign() * temporal_phase)
            .rem_euclid(360.0)
    }
}

impl Default for AsusAuraRainbowRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectRenderer for AsusAuraRainbowRenderer {
    fn init(&mut self, _metadata: &EffectMetadata) -> anyhow::Result<()> {
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
    fn render_into(&mut self, input: &FrameInput<'_>, canvas: &mut Canvas) -> anyhow::Result<()> {
        prepare_target_canvas(canvas, input.canvas_width, input.canvas_height);
        let row_len = input.canvas_width as usize * BYTES_PER_PIXEL;
        let pixels = canvas.as_rgba_bytes_mut();
        if row_len == 0 || pixels.len() < row_len {
            return Ok(());
        }

        let width = f64::from(input.canvas_width.max(1));
        let (first_row, remaining_rows) = pixels.split_at_mut(row_len);
        for (x, pixel) in first_row
            .as_chunks_mut::<BYTES_PER_PIXEL>()
            .0
            .iter_mut()
            .enumerate()
        {
            let rgb = Hsv::new(
                self.hue_at(x, width, input.time_secs) as f32,
                self.saturation,
                self.brightness,
            )
            .to_rgb();
            pixel.copy_from_slice(&[rgb.r, rgb.g, rgb.b, u8::MAX]);
        }

        for row in remaining_rows.chunks_exact_mut(row_len) {
            row.copy_from_slice(first_row);
        }

        Ok(())
    }

    fn apply_controls(&mut self, batch: &ControlDeltaBatch<'_>) -> anyhow::Result<()> {
        for (control_id, value) in batch.changes {
            match control_id.as_str() {
                "band_density" => {
                    if let Some(value) = value.as_effect_f32() {
                        self.band_density = f64::from(value.clamp(1.0, 20.0));
                    }
                }
                "cycle_duration" => {
                    if let Some(value) = value.as_effect_f32() {
                        self.cycle_duration_secs = f64::from(value.clamp(2.0, 120.0));
                    }
                }
                "direction" => {
                    if let ControlValue::Enum(choice) | ControlValue::Text(choice) = value {
                        self.direction = AuraDirection::from_str(choice);
                    }
                }
                "brightness" => {
                    if let Some(value) = value.as_effect_f32() {
                        self.brightness = value.clamp(0.0, 1.0);
                    }
                }
                "saturation" => {
                    if let Some(value) = value.as_effect_f32() {
                        self.saturation = value.clamp(0.0, 1.0);
                    }
                }
                "start_hue" => {
                    if let Some(value) = value.as_effect_f32() {
                        self.start_hue_degrees = f64::from(value.clamp(0.0, 360.0));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn destroy(&mut self) {}
}

fn controls() -> Vec<ControlDefinition> {
    vec![
        slider_control(
            "band_density",
            "Band Density",
            5.0,
            1.0,
            20.0,
            1.0,
            "Shape",
            "Number of complete rainbow cycles repeated across the canvas width.",
        ),
        slider_control(
            "cycle_duration",
            "Cycle Duration",
            12.0,
            2.0,
            120.0,
            1.0,
            "Motion",
            "Seconds required to move the pattern by one complete color cycle.",
        ),
        dropdown_control(
            "direction",
            "Direction",
            "Right",
            &["Left", "Right"],
            "Motion",
            "Direction in which the vertical rainbow bands move.",
        ),
        slider_control(
            "brightness",
            "Brightness",
            0.7,
            0.0,
            1.0,
            0.01,
            "Output",
            "Master HSV brightness for every rainbow band.",
        ),
        slider_control(
            "saturation",
            "Saturation",
            1.0,
            0.0,
            1.0,
            0.01,
            "Colors",
            "HSV color saturation across the repeated spectrum.",
        ),
        slider_control(
            "start_hue",
            "Start Hue",
            0.0,
            0.0,
            360.0,
            1.0,
            "Colors",
            "Hue in degrees at the left edge when the animation begins.",
        ),
    ]
}

fn presets() -> Vec<PresetTemplate> {
    vec![
        preset_with_desc(
            "Aura Cold Boot Approximation",
            "Default configurable approximation of the ASUS Aura Gen 3 cold-boot rainbow.",
            &[
                ("band_density", ControlValue::Float(5.0)),
                ("cycle_duration", ControlValue::Float(12.0)),
                ("direction", ControlValue::Enum("Right".to_owned())),
                ("brightness", ControlValue::Float(0.7)),
                ("saturation", ControlValue::Float(1.0)),
                ("start_hue", ControlValue::Float(0.0)),
            ],
        ),
        preset_with_desc(
            "Aura Reverse",
            "The default approximation with the bands moving left.",
            &[
                ("band_density", ControlValue::Float(5.0)),
                ("cycle_duration", ControlValue::Float(12.0)),
                ("direction", ControlValue::Enum("Left".to_owned())),
                ("brightness", ControlValue::Float(0.7)),
                ("saturation", ControlValue::Float(1.0)),
                ("start_hue", ControlValue::Float(0.0)),
            ],
        ),
        preset_with_desc(
            "Aura Dense",
            "A denser, faster variation with ten rainbow cycles across the canvas.",
            &[
                ("band_density", ControlValue::Float(10.0)),
                ("cycle_duration", ControlValue::Float(8.0)),
                ("direction", ControlValue::Enum("Right".to_owned())),
                ("brightness", ControlValue::Float(0.7)),
                ("saturation", ControlValue::Float(1.0)),
                ("start_hue", ControlValue::Float(0.0)),
            ],
        ),
    ]
}

pub(super) fn metadata() -> EffectMetadata {
    EffectMetadata {
        id: builtin_effect_id("asus_aura_rainbow"),
        name: "ASUS Aura Rainbow".into(),
        author: "Hypercolor".into(),
        version: "0.1.0".into(),
        description:
            "Configurable visual approximation of the moving ASUS Aura Gen 3 cold-boot rainbow"
                .into(),
        category: EffectCategory::Ambient,
        tags: vec![
            "asus".into(),
            "aura".into(),
            "rainbow".into(),
            "bands".into(),
        ],
        controls: controls(),
        presets: presets(),
        audio_reactive: false,
        screen_reactive: false,
        input_reactive: false,
        source: EffectSource::Native {
            path: PathBuf::from("builtin/asus_aura_rainbow"),
        },
        license: Some("Apache-2.0".into()),
    }
}
