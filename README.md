# AWS Lambda Action Filter

This project is based on a coding interview assignment originally authored by Illya, as identified via the GitHub repository he shared during the interview.

## ✅ Overview

This Lambda function processes a list of actions (in JSON format) and applies business rules to filter and sort them. The implementation includes:

- Deduplication (at most one action per `entity_id`)
- Time-based filtering:
  - `next_action_time` must be within 90 days from today
  - `last_action_time` must be at least 7 days ago
- Priority sorting: `"urgent"` actions appear first

## 🔧 Fixes & Improvements

The original version had compilation errors, deprecated dependencies, and incorrect logic. This version includes:

- ✅ Fixed the deprecated `lambda_runtime::handler_fn` usage
- ✅ Corrected priority sorting logic
- ✅ Implemented proper time filtering logic (7-day and 90-day cutoffs)
- ✅ Ensured deduplication by `entity_id`
- ✅ Added complete, panic-free unit tests using `anyhow::ensure`
- ✅ Adopted idiomatic error handling and refactored logic into a reusable `process_actions` function

## ⚠️ Attribution & License

> This project is based on a challenge provided during a technical interview.  
> The original version appears to have been authored by Illya, based on the GitHub repository he shared during the interview process.
> Permission to publish and extend the original code was granted by Illya.

---

## 📄 Original Assignment Instructions (preserved below)

> The following section is retained from the original assignment prompt for context.

---

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

## 🛠 Getting Started

1. **Install Rust** (stable) and [cargo‑lambda](https://github.com/cargo-lambda/cargo-lambda):

   ```bash
   rustup update stable
   cargo install cargo-lambda
````

2. **Run the Lambda locally** with sample data:

   ```bash
   cargo lambda invoke --data-file testdata/01_sample-input.json
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
* Propose a CDK deploy step or GitHub Actions workflow.

Good luck — happy debugging!
