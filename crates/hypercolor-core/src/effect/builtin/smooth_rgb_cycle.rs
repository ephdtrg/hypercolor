//! Smooth RGB cycle renderer — one canvas-wide HSV color animated over time.

use std::path::PathBuf;

use hypercolor_color::Hsv;
use hypercolor_types::canvas::{Canvas, Rgba};
use hypercolor_types::control::{ControlDeltaBatch, ControlValue};
use hypercolor_types::effect::{
    ControlDefinition, EffectCategory, EffectMetadata, EffectSource, PresetTemplate,
};

use super::common::{builtin_effect_id, dropdown_control, preset_with_desc, slider_control};
use crate::effect::traits::{EffectRenderer, FrameInput, prepare_target_canvas};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CycleDirection {
    Forward,
    Reverse,
}

impl CycleDirection {
    fn from_str(value: &str) -> Self {
        if value.trim().eq_ignore_ascii_case("reverse") {
            Self::Reverse
        } else {
            Self::Forward
        }
    }

    const fn sign(self) -> f64 {
        match self {
            Self::Forward => 1.0,
            Self::Reverse => -1.0,
        }
    }
}

/// Fills the canvas with one color that cycles smoothly through HSV hue.
pub struct SmoothRgbCycleRenderer {
    cycle_duration_secs: f64,
    brightness: f32,
    saturation: f32,
    direction: CycleDirection,
    start_hue_degrees: f64,
}

impl SmoothRgbCycleRenderer {
    /// Create a smooth RGB cycle renderer with the catalog defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cycle_duration_secs: 120.0,
            brightness: 0.7,
            saturation: 1.0,
            direction: CycleDirection::Forward,
            start_hue_degrees: 0.0,
        }
    }

    fn hue_at_time(&self, time_secs: f64) -> f64 {
        let time_secs = if time_secs.is_finite() {
            time_secs
        } else {
            0.0
        };
        let elapsed_in_cycle = time_secs.rem_euclid(self.cycle_duration_secs);
        let phase_degrees = elapsed_in_cycle / self.cycle_duration_secs * 360.0;
        (self.start_hue_degrees + self.direction.sign() * phase_degrees).rem_euclid(360.0)
    }
}

impl Default for SmoothRgbCycleRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectRenderer for SmoothRgbCycleRenderer {
    fn init(&mut self, _metadata: &EffectMetadata) -> anyhow::Result<()> {
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
    fn render_into(&mut self, input: &FrameInput<'_>, canvas: &mut Canvas) -> anyhow::Result<()> {
        prepare_target_canvas(canvas, input.canvas_width, input.canvas_height);
        let rgb = Hsv::new(
            self.hue_at_time(input.time_secs) as f32,
            self.saturation,
            self.brightness,
        )
        .to_rgb();
        canvas.fill(Rgba::new(rgb.r, rgb.g, rgb.b, u8::MAX));
        Ok(())
    }

    fn apply_controls(&mut self, batch: &ControlDeltaBatch<'_>) -> anyhow::Result<()> {
        for (control_id, value) in batch.changes {
            match control_id.as_str() {
                "cycle_duration" => {
                    if let Some(value) = value.as_effect_f32() {
                        self.cycle_duration_secs = f64::from(value.clamp(10.0, 600.0));
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
                "direction" => {
                    if let ControlValue::Enum(choice) | ControlValue::Text(choice) = value {
                        self.direction = CycleDirection::from_str(choice);
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
            "cycle_duration",
            "Cycle Duration",
            120.0,
            10.0,
            600.0,
            1.0,
            "Motion",
            "Seconds required for one complete trip around the HSV hue wheel.",
        ),
        slider_control(
            "brightness",
            "Brightness",
            0.7,
            0.0,
            1.0,
            0.01,
            "Output",
            "Master HSV brightness for the uniform canvas color.",
        ),
        slider_control(
            "saturation",
            "Saturation",
            1.0,
            0.0,
            1.0,
            0.01,
            "Colors",
            "HSV color saturation; lower values move the cycle toward white.",
        ),
        dropdown_control(
            "direction",
            "Direction",
            "Forward",
            &["Forward", "Reverse"],
            "Motion",
            "Choose whether hue increases or decreases over time.",
        ),
        slider_control(
            "start_hue",
            "Start Hue",
            0.0,
            0.0,
            360.0,
            1.0,
            "Colors",
            "Hue in degrees at the beginning of each cycle.",
        ),
    ]
}

fn presets() -> Vec<PresetTemplate> {
    vec![
        preset_with_desc(
            "Smooth Spectrum",
            "The default two-minute, fully saturated forward color cycle.",
            &[
                ("cycle_duration", ControlValue::Float(120.0)),
                ("brightness", ControlValue::Float(0.7)),
                ("saturation", ControlValue::Float(1.0)),
                ("direction", ControlValue::Enum("Forward".to_owned())),
                ("start_hue", ControlValue::Float(0.0)),
            ],
        ),
        preset_with_desc(
            "Reverse Spectrum",
            "The default smooth spectrum moving backward around the hue wheel.",
            &[
                ("cycle_duration", ControlValue::Float(120.0)),
                ("brightness", ControlValue::Float(0.7)),
                ("saturation", ControlValue::Float(1.0)),
                ("direction", ControlValue::Enum("Reverse".to_owned())),
                ("start_hue", ControlValue::Float(0.0)),
            ],
        ),
        preset_with_desc(
            "Soft Pastel Cycle",
            "A slower, gentler cycle with reduced saturation and brightness.",
            &[
                ("cycle_duration", ControlValue::Float(240.0)),
                ("brightness", ControlValue::Float(0.6)),
                ("saturation", ControlValue::Float(0.6)),
                ("direction", ControlValue::Enum("Forward".to_owned())),
                ("start_hue", ControlValue::Float(210.0)),
            ],
        ),
    ]
}

pub(super) fn metadata() -> EffectMetadata {
    EffectMetadata {
        id: builtin_effect_id("smooth_rgb_cycle"),
        name: "Smooth RGB Cycle".into(),
        author: "Hypercolor".into(),
        version: "0.1.0".into(),
        description: "Uniform canvas-wide HSV color cycle driven by absolute frame time".into(),
        category: EffectCategory::Ambient,
        tags: vec!["rgb".into(), "hsv".into(), "cycle".into(), "uniform".into()],
        controls: controls(),
        presets: presets(),
        audio_reactive: false,
        screen_reactive: false,
        input_reactive: false,
        source: EffectSource::Native {
            path: PathBuf::from("builtin/smooth_rgb_cycle"),
        },
        license: Some("Apache-2.0".into()),
    }
}
