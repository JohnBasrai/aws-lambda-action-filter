# Rust Lambda Assignment: Action Filter

This repository contains a **broken** AWS Lambda written in Rust. Your task is to debug and fix it so that it compiles, runs locally, and produces correct results.

## 📋 Scenario

The Lambda receives a JSON list of **actions**, each with:

* `entity_id` — string identifier
* `last_action_time` — ISO‑8601 timestamp
* `next_action_time` — ISO‑8601 timestamp
* `priority` — `"high"` or `"low"`

### Business Rules

1. **At most one** action per `entity_id`.
2. Only include actions where **`next_action_time` is within 90 days** of *today*.
3. **High‑priority** actions should appear **first** in the output.
4. Skip any action where **`last_action_time` is < 7 days ago**.

Unfortunately, the code has **three deliberate problems**:

| Type | What you’ll see |
|------|-----------------|
| Compilation error | Lifetime/key mismatch in a `HashMap` declaration |
| Runtime panic | Timestamp math can panic with certain input |
| Logic bug | Priority sorting is wrong (`"high"` can end up after `"low"`) |

## 🛠 Getting Started

1. **Install Rust** (stable) and [cargo‑lambda](https://github.com/cargo-lambda/cargo-lambda):

   ```bash
   rustup update stable
   cargo install cargo-lambda
   ```

2. **Run the Lambda locally** with sample data:

   ```bash
   cargo lambda invoke --data-file testdata/sample-input.json
   ```

   You should observe a compilation error first. Fix it, then re‑run to expose the panic and logic bug.

3. **Fix all three problems** so the Lambda prints a correct, filtered list.

## ✅ Acceptance Criteria

* The project **compiles cleanly** (`cargo check` passes).
* `cargo lambda invoke …` returns the correct, filtered JSON.
* No panics for well‑formed input.
* Clear, idiomatic Rust code with proper error handling and logging (`tracing` or `log` welcome).

## 🧪 Optional Stretch Goals

* Add unit tests in `tests/`.
* Improve error messages and JSON schema validation.
* Propose a CDK deploy step or GitHub Actions workflow.

Good luck — happy debugging!
