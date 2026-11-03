# Built-in RGB Effects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add two fully registered native RGB effects, Smooth RGB Cycle and ASUS Aura Rainbow, with deterministic absolute-time animation, controls, presets, and behavioral coverage.

**Architecture:** Each effect is a focused `EffectRenderer` module with only control-derived state. Smooth RGB Cycle converts one absolute-time HSV hue and fills the canvas; ASUS Aura Rainbow computes one horizontal HSV row from coordinate plus absolute time and copies it vertically. Existing registry metadata remains the single discovery path for Studio and CLI.

**Tech Stack:** Rust 2024, `hypercolor-core`, `hypercolor-color::Hsv`, `hypercolor-types::{Canvas, ControlValue, EffectMetadata}`, Cargo/just.

**Spec:** `docs/superpowers/specs/2026-11-03-built-in-rgb-effects-design.md`

## Global Constraints

- Do not replace or change the behavior of existing effects.
- Derive animation only from `FrameInput::time_secs`; never accumulate frame offsets or use `frame_number`/`delta_secs` as phase.
- Smooth RGB Cycle defaults/ranges: duration 120 s in 10–600, brightness 0.70 in 0–1, saturation 1.0 in 0–1, Forward/Reverse, start hue 0° in 0–360.
- ASUS Aura Rainbow defaults/ranges: density 5 in 1–20, duration 12 s in 2–120, Left/Right, brightness 0.70 in 0–1, saturation 1.0 in 0–1, start hue 0° in 0–360.
- Use configurable canvas dimensions and the existing sRGB canvas/color conversion conventions; do not apply gamma manually.
- Describe ASUS output as a configurable visual approximation, not firmware-identical behavior.
- Do not modify scenes, Stage layout, hardware configuration, OpenRGB, Lian Li Linux, installation state, or systemd services.

## Review Focus

- Very large `time_secs` values must remain cyclic and finite; Task 1 and Task 2 include long-uptime equivalence tests.
- Dropdown values may arrive as `Enum` or `Text`; each renderer's direction test covers both accepted serialization forms.
- Live canvas resize must update dimensions without stale stride or panic; both renderer tasks include caller-owned-canvas resize tests.
- Adding a second effect whose name contains “Rainbow” changes registry search cardinality; Task 3 updates the search contract explicitly.
- Zero and out-of-range renderer values must not create division-by-zero/non-finite output even if admission is bypassed; control tests in Tasks 1 and 2 apply defensive clamping paths.

---

### Task 1: Smooth RGB Cycle renderer

**Files:**
- Create: `crates/hypercolor-core/src/effect/builtin/smooth_rgb_cycle.rs`
- Modify: `crates/hypercolor-core/src/effect/builtin/mod.rs`
- Test: `crates/hypercolor-core/tests/builtin_effect_tests.rs`

**Interfaces:**
- Consumes: `EffectRenderer`, `FrameInput`, `prepare_target_canvas`, `Hsv`, shared metadata/control/preset constructors.
- Produces: public `SmoothRgbCycleRenderer::new() -> Self`; native source stem `smooth_rgb_cycle`; controls `cycle_duration`, `brightness`, `saturation`, `direction`, `start_hue`.

- [ ] **Step 1: Add failing Smooth RGB Cycle tests and timing fixture**

Add a `frame_with_timing(time_secs, delta_secs, frame_number, width, height)` fixture and tests named:

- `smooth_rgb_cycle_fills_every_pixel_uniformly`
- `smooth_rgb_cycle_forward_reverse_and_full_period_are_correct`
- `smooth_rgb_cycle_output_depends_on_absolute_time_not_fps`
- `smooth_rgb_cycle_large_time_wraps_without_drift`
- `smooth_rgb_cycle_controls_clamp_and_accept_serialized_direction`
- `smooth_rgb_cycle_resizes_caller_owned_canvas`

Assert uniform RGBA bytes, Forward hue 120° versus Reverse hue 240° at one-third of the default period, equality at `t=0` and `t=120`, byte-identical output for equal `time_secs` with different delta/frame values, equality for a large time and its reduced phase, finite non-black clamped output, and updated dimensions after reuse.

- [ ] **Step 2: Run the new tests and verify RED**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests smooth_rgb_cycle -- --nocapture`

Expected: compile/test failure because `SmoothRgbCycleRenderer` and its module/export do not exist.

- [ ] **Step 3: Implement the minimal Smooth RGB Cycle renderer**

Create `SmoothRgbCycleRenderer` with fields `cycle_duration_secs: f64`, `brightness: f32`, `saturation: f32`, `direction`, and `start_hue_degrees: f64`. Implement `Default`, the four required `EffectRenderer` methods, absolute hue phase with `rem_euclid(360.0)`, one `Hsv::to_rgb()` conversion per frame, and `Canvas::fill`.

Add the module declaration and public renderer re-export to `builtin/mod.rs`; metadata, presets, and registry insertion are Task 3.

- [ ] **Step 4: Run Smooth RGB Cycle tests and verify GREEN**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests smooth_rgb_cycle -- --nocapture`

Expected: all Smooth RGB Cycle tests pass with no warnings.

- [ ] **Step 5: Refactor while green**

Keep phase math in a small private helper typed in `f64`; keep serialized direction parsing private and accept both `Enum` and `Text` through `apply_controls`.

- [ ] **Step 6: Re-run the focused tests**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests smooth_rgb_cycle -- --nocapture`

Expected: PASS.

### Task 2: ASUS Aura Rainbow renderer

**Files:**
- Create: `crates/hypercolor-core/src/effect/builtin/asus_aura_rainbow.rs`
- Modify: `crates/hypercolor-core/src/effect/builtin/mod.rs`
- Test: `crates/hypercolor-core/tests/builtin_effect_tests.rs`

**Interfaces:**
- Consumes: the Task 1 `frame_with_timing` test fixture plus the same renderer/color/control primitives.
- Produces: public `AsusAuraRainbowRenderer::new() -> Self`; native source stem `asus_aura_rainbow`; controls `band_density`, `cycle_duration`, `direction`, `brightness`, `saturation`, `start_hue`.

- [ ] **Step 1: Add failing ASUS Aura Rainbow behavior tests**

Add tests named:

- `asus_aura_rainbow_is_uniform_vertically_and_varies_horizontally`
- `asus_aura_rainbow_band_density_repeats_equal_full_cycles`
- `asus_aura_rainbow_direction_moves_left_and_right`
- `asus_aura_rainbow_is_seamless_after_one_cycle_duration`
- `asus_aura_rainbow_output_depends_on_absolute_time_not_fps`
- `asus_aura_rainbow_large_time_wraps_without_drift`
- `asus_aura_rainbow_has_no_black_or_transparent_gaps_at_defaults`
- `asus_aura_rainbow_controls_clamp_and_accept_serialized_direction`
- `asus_aura_rainbow_resizes_caller_owned_canvas`

Assert every row is byte-identical to the first, colors vary along x, a width divisible by density repeats the same hue at each cycle boundary, Left/Right produce the expected green/blue origin at one-third period, entire canvases match at `t=0` and `t=12`, equal absolute times ignore FPS inputs, large time reduces to the same phase, every default pixel has alpha 255 and at least one nonzero RGB channel, and reused canvas dimensions update.

- [ ] **Step 2: Run the new tests and verify RED**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests asus_aura_rainbow -- --nocapture`

Expected: compile/test failure because `AsusAuraRainbowRenderer` and its module/export do not exist.

- [ ] **Step 3: Implement the minimal ASUS Aura Rainbow renderer**

Create `AsusAuraRainbowRenderer` with fields `band_density: f64`, `cycle_duration_secs: f64`, `direction`, `brightness: f32`, `saturation: f32`, and `start_hue_degrees: f64`. Implement the required renderer methods. In `render_into`, prepare the target, compute each first-row pixel from normalized x and absolute phase, then copy that row into all remaining rows. Right subtracts temporal phase; Left adds it.

Add only the module declaration and public renderer re-export to `builtin/mod.rs`; metadata, presets, and registry insertion are Task 3.

- [ ] **Step 4: Run ASUS Aura Rainbow tests and verify GREEN**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests asus_aura_rainbow -- --nocapture`

Expected: all ASUS tests pass with no warnings.

- [ ] **Step 5: Refactor while green**

Keep coordinate/time phase math in a private `f64` helper, avoid per-row recomputation, and keep defensive clamps at the control boundary.

- [ ] **Step 6: Re-run the focused tests**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests asus_aura_rainbow -- --nocapture`

Expected: PASS.

### Task 3: Registry, Studio/CLI discovery metadata, and presets

**Files:**
- Modify: `crates/hypercolor-core/src/effect/builtin/mod.rs`
- Test: `crates/hypercolor-core/tests/builtin_effect_tests.rs`

**Interfaces:**
- Consumes: `smooth_rgb_cycle::metadata()`, `asus_aura_rainbow::metadata()`, `SmoothRgbCycleRenderer::new()`, and `AsusAuraRainbowRenderer::new()` from Tasks 1–2.
- Produces: both effects in `register_builtin_effects`, `create_builtin_renderer`, registry search/listing, and therefore the shared Studio/CLI catalog.

- [ ] **Step 1: Add failing registry and metadata assertions**

Extend `factory_creates_all_builtins`, `register_builtin_effects_populates_registry`, and `registered_builtins_use_human_readable_names_and_stable_native_keys` for both names/keys and new counts: 12 without Servo or 13 with Servo; 8 Ambient effects.

Add `smooth_rgb_cycle_metadata_matches_public_contract` and `asus_aura_rainbow_metadata_matches_public_contract`. For each control, assert ID, default value, min/max/step, dropdown labels, and that presets are non-empty and include an exact-default preset. Assert ASUS description contains approximation language. Update `registered_effects_searchable_by_name` so a `rainbow` search expects both `Rainbow` and `ASUS Aura Rainbow`.

- [ ] **Step 2: Run registry/metadata tests and verify RED**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests -- --nocapture`

Expected: failures because the new modules are not yet in builtin metadata or factory dispatch, and old registry counts/search expectations no longer describe the target state.

- [ ] **Step 3: Register both effects**

Add exact `controls()`, `presets()`, and `metadata()` helpers to both new modules. Smooth presets are `Smooth Spectrum`, `Reverse Spectrum`, and `Soft Pastel Cycle`; ASUS presets are `Aura Cold Boot Approximation`, `Aura Reverse`, and `Aura Dense`. Update the module-level effect table, `builtin_metadata()`, and `create_builtin_renderer()` in `builtin/mod.rs`. Do not reorder or remove existing effects.

- [ ] **Step 4: Run the complete built-in effect integration test**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests -- --nocapture`

Expected: PASS, including preset ID uniqueness and registry search.

- [ ] **Step 5: Run the effect factory integration tests**

Run: `cargo test -p hypercolor-core --test effect_factory_tests -- --nocapture`

Expected: PASS.

### Task 4: Formatting and repository verification

**Files:**
- Verify all files changed by Tasks 1–3.

**Interfaces:**
- Consumes: completed renderer and registry implementation.
- Produces: formatted, lint-clean, test-verified, buildable workspace changes without installation or service mutation.

- [ ] **Step 1: Format and verify no unexpected files changed**

Run: `cargo fmt --all -- --check`

If it fails, run `cargo fmt --all`, then repeat the check. Run `git status --short` and confirm only the two new modules, builtin registry/test changes, and approved design/plan docs are present.

- [ ] **Step 2: Run focused crate tests**

Run: `cargo test -p hypercolor-core --test builtin_effect_tests --test effect_factory_tests`

Expected: PASS.

- [ ] **Step 3: Run repository-required verification**

Run: `just verify`

Expected: format, lint, and workspace tests all pass. If any unrelated pre-existing failure occurs, record the exact failing command/test and continue with the remaining independent checks.

- [ ] **Step 4: Check the workspace and separately build necessary components**

Run: `cargo check --workspace`

Run: `cargo build -p hypercolor-cli -p hypercolor-daemon`

Expected: both commands pass. Do not run installation recipes and do not restart any service.

- [ ] **Step 5: Inspect the final diff and prepare handoff**

Run: `git diff --check`

Run: `git diff --stat`

Report changed files, exact verification results, any unrelated failures, and non-executed commands the user can later use to install the locally built Hypercolor after granting separate permission.
