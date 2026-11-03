# Built-in RGB Effects Design

## Intent

Add two always-available native effects for Hypercolor 0.8.1: a canvas-wide HSV color cycle and a configurable visual approximation of the ASUS Aura Gen 3 cold-boot rainbow. Both effects must be discoverable through the existing effect registry used by Studio and `hypercolor effects list`.

## Goals

- Add `Smooth RGB Cycle` under the stable native key `smooth_rgb_cycle`.
- Add `ASUS Aura Rainbow` under the stable native key `asus_aura_rainbow`.
- Expose the requested controls, defaults, ranges, dropdown choices, and built-in presets through `EffectMetadata`.
- Derive every frame directly from `FrameInput::time_secs`, so output at a given time is independent of FPS, frame number, render history, and `delta_secs`.
- Render in HSV with the project's existing `Hsv` conversion and write sRGB-encoded `Canvas` pixels without an extra gamma pass.
- Register both renderers without changing or replacing existing effects.

## Non-goals and anti-goals

- Do not claim bit-for-bit or firmware-level compatibility with ASUS Aura.
- Do not modify scenes, Stage layout, hardware configuration, OpenRGB, Lian Li Linux, installation state, or systemd services.
- Do not add randomness, retained temporal state, gradients along the vertical axis, black gaps, dimming modulation, or frame-accumulated offsets.
- Do not add a separate Studio implementation; Studio and CLI must consume the shared native registry metadata.

## Constraints

- Smooth RGB Cycle controls: Cycle Duration 10–600 s (default 120), Brightness 0–100% (default 70%), Saturation 0–100% (default 100%), Direction Forward/Reverse, Start Hue 0–360° (default 0°).
- ASUS Aura Rainbow controls: Band Density 1–20 full cycles (default 5), Cycle Duration 2–120 s (default 12), Direction Left/Right, Brightness 0–100% (default 70%), Saturation 0–100% (default 100%), Start Hue 0–360° (default 0°).
- Percent controls use Hypercolor's normalized `0.0..1.0` renderer convention while UI metadata supplies the requested percentage range semantically.
- Canvas dimensions always come from `FrameInput`; zero-width output must remain safe.
- Absolute phase calculations use `f64` and cyclic reduction before narrowing to the `f32` hue expected by `Hsv`.

## Approaches

1. **Dedicated native renderers (selected).** Add one focused module per effect and register both in `builtin/mod.rs`. This preserves existing effect behavior, works without Servo, gives Studio and CLI the metadata automatically, and matches the requested module boundaries.
2. Extend the existing `rainbow` renderer with extra modes. Rejected because its `speed`, `scale`, and axis controls have different semantics and changing it would conflate old and new serialized configurations.
3. Add SDK/HTML effects. Rejected because they would not be the requested always-available native built-ins and would introduce a Servo/runtime dependency.

## Rendering design

### Smooth RGB Cycle

The renderer has only control-derived state. For every frame it computes:

`hue = (start_hue + direction_sign * 360 * time_secs.rem_euclid(cycle_duration) / cycle_duration).rem_euclid(360)`

Forward uses `direction_sign = +1`; Reverse uses `-1`. The HSV color uses configured saturation and brightness, is converted through `hypercolor_color::Hsv`, and fills the complete canvas with one encoded color.

### ASUS Aura Rainbow

The renderer computes one horizontal row and copies it to every remaining row, which guarantees strictly vertical, spatially uniform bands. At horizontal coordinate `x`:

`hue = (start_hue + 360 * band_density * x / width + motion_sign * 360 * time_secs.rem_euclid(cycle_duration) / cycle_duration).rem_euclid(360)`

Left uses `motion_sign = +1`; Right uses `-1`, so the underlying pattern moves in the direction named by the control. One duration advances the pattern by exactly one complete color cycle, making the temporal loop seamless. The description and preset copy identify this as a visual approximation because the hardware algorithm is unknown.

## Metadata and presets

Both effects are Ambient native effects authored by Hypercolor, with stable IDs derived by `builtin_effect_id` and source paths under `builtin/`. Controls use shared `slider_control` and `dropdown_control` constructors.

Smooth RGB Cycle presets:

- `Smooth Spectrum` — exact defaults.
- `Reverse Spectrum` — exact defaults with Reverse direction.
- `Soft Pastel Cycle` — slower, lower-saturation ambient variant.

ASUS Aura Rainbow presets:

- `Aura Cold Boot Approximation` — exact defaults.
- `Aura Reverse` — exact defaults moving Left.
- `Aura Dense` — denser, faster configurable variant.

Preset names are unique within each effect, and every preset value uses the same serialized `ControlValue` type as its control.

## Verification

Integration tests in `crates/hypercolor-core/tests/builtin_effect_tests.rs` will prove:

- factory creation, registry counts/categories, stable native keys, requested metadata, control ranges/defaults, and non-empty unique presets;
- uniform Smooth RGB Cycle fill, expected quarter/full-cycle colors, direction reversal, and identical output for the same absolute time under different frame/delta inputs;
- ASUS vertical uniformity, horizontal spectrum variation, band density, direction, absence of black/transparent gaps at default output, temporal seam equality after one duration, and identical output for the same absolute time under different frame/delta inputs;
- canvas resize behavior through the existing caller-owned target path.

Run rustfmt, focused `hypercolor-core` tests, workspace checks/tests required by repository policy, and the necessary build commands. Do not install binaries or restart services.
