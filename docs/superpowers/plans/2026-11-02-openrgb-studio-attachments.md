# Studio Attachment Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve physical-component attachment metadata from daemon `Output` values through the scene API into Studio so attached controller components render while raw controller channels stay hidden.

**Architecture:** Extend the canonical `hypercolor-types::api::scene::ZoneMember` contract with a backward-compatible optional `OutputComponent`. Project the field in the daemon and reconstruct it in the UI without changing LED assignment fields or introducing ID-based heuristics.

**Tech Stack:** Rust, Serde/serde_json, Axum integration tests, Leptos UI tests, Cargo/Trunk.

**Spec:** `/home/ephdtrg/.codex/attachments/783ee74a-a042-498b-8ef2-47b2dc0253d1/Pasted text.txt`

## Global Constraints

- Work from source tag `v0.8.1` in `~/hypercolor-src` on a rollback-safe branch.
- Do not modify OpenRGB or Lian Li drivers without evidence.
- Do not use `attachment-*` or any other output-ID heuristic.
- Do not alter saved scenes, profiles, RGB settings, or the installed Hypercolor build.
- Do not install the build or restart `hypercolor.service` without explicit permission.

## Review Focus

- Old scene JSON without `attachment` must decode to `None`.
- Every `OutputComponent` field, including `led_mapping`, must round-trip exactly.
- An attachment must not alter `Output.led_mapping` or any LED allocation semantics.
- A controller attachment must render while the same controller's raw unattached channel remains hidden.
- Non-controller devices must retain their existing visibility behavior.

---

### Task 1: Extend the canonical scene API member contract

**Files:**
- Modify: `crates/hypercolor-types/src/api/scene.rs`
- Test: `crates/hypercolor-types/tests/api_scene_tests.rs`

**Interfaces:**
- Consumes: `hypercolor_types::spatial::OutputComponent`.
- Produces: `ZoneMember::attachment: Option<OutputComponent>` with Serde defaulting and omission when absent.

- [ ] Add failing tests for a full attachment round-trip and legacy JSON with the field absent.
- [ ] Run `cargo test -p hypercolor-types --test api_scene_tests` and confirm the new contract test fails because the field does not exist.
- [ ] Add the optional field and update every direct `ZoneMember` constructor with `attachment: None` where appropriate.
- [ ] Re-run the type contract tests and confirm they pass.

### Task 2: Preserve attachments in the daemon scene projection

**Files:**
- Modify: `crates/hypercolor-daemon/src/domain/scene_tree.rs`
- Test: `crates/hypercolor-daemon/tests/scene_tree_api_tests.rs`

**Interfaces:**
- Consumes: `Output::attachment` and the Task 1 `ZoneMember::attachment` field.
- Produces: `/api/v1/scene` members whose attachment is an exact clone of the source output metadata.

- [ ] Add a failing API test that seeds a full attachment and asserts every field in the GET response.
- [ ] Run the narrow daemon API test without Servo and confirm the attachment assertion fails.
- [ ] Clone `output.attachment` in `zone_member`.
- [ ] Re-run the daemon API test and confirm it passes.

### Task 3: Restore attachments in Studio and pin visibility behavior

**Files:**
- Modify: `crates/hypercolor-ui/src/api/scenes.rs`
- Modify: `crates/hypercolor-ui/src/components/layout_canvas.rs`
- Test: `crates/hypercolor-ui/tests/studio_history_tests.rs`
- Test: unit tests in `crates/hypercolor-ui/src/components/layout_canvas.rs`
- Modify: direct `ZoneMember` fixtures under `crates/hypercolor-ui/tests/`

**Interfaces:**
- Consumes: `ZoneMember::attachment` from Task 1.
- Produces: Studio `Output::attachment` values used by the unchanged `draws_output` policy.

- [ ] Add a failing `zone_outputs` regression test that asserts exact attachment restoration and unchanged placement/LED mapping fields.
- [ ] Add visibility tests for an attached controller, its unattached raw channel, and an unattached fixed device.
- [ ] Run the targeted UI tests and confirm attachment restoration fails before implementation.
- [ ] Clone the member attachment into the reconstructed UI output; keep `draws_output` free of ID/driver special cases.
- [ ] Re-run the targeted and full UI tests and confirm they pass.

### Task 4: Format, compile, build, and audit compatibility

**Files:**
- Verify all changed files only.

**Interfaces:**
- Consumes: Tasks 1–3.
- Produces: verified Rust formatting, shared API/daemon compilation, production WASM UI build, and a documented build-environment limitation if external system libraries are unavailable.

- [ ] Run Rust formatting for the workspace and excluded UI manifest.
- [ ] Run shared-type, daemon API, and full UI tests with the available linker.
- [ ] Run targeted `cargo check` for shared types and the daemon without Servo.
- [ ] Run `just ui-build` for the production WASM frontend.
- [ ] Attempt the project-prescribed daemon build; distinguish code failures from missing host libraries.
- [ ] Audit the final diff for LED assignment changes, ID heuristics, driver changes, and accidental installation/service actions.
