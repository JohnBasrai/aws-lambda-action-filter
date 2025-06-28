# AWS Lambda Action Filter

[![CI](https://github.com/JohnBasrai/aws-lambda-action-filter/actions/workflows/ci.yml/badge.svg)](https://github.com/JohnBasrai/aws-lambda-action-filter/actions)
![Rust](https://img.shields.io/badge/rust-1.85.0-blue?logo=rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Last Commit](https://img.shields.io/github/last-commit/JohnBasrai/aws-lambda-action-filter)

This project demonstrates a production-ready AWS Lambda function written in Rust, showcasing the **[Explicit Module Boundary Pattern (EMBP)](embp.md)** for clean architecture and comprehensive integration testing with `cargo-lambda`.

## ✅ Overview

This Lambda function processes a list of actions (in JSON format) and applies business rules to filter and sort them. The implementation demonstrates:

- **Clean Architecture** using the EMBP pattern
- **Domain-Driven Design** with clear separation of concerns
- **Comprehensive Testing** including real lambda integration tests
- **Production-Ready Error Handling** with proper logging and validation
- **Containerized Development** for consistent environments

## 🏗️ Architecture

### EMBP Pattern Implementation

The project follows the **Explicit Module Boundary Pattern** for maintainable, scalable code organization:

```
src/
├── lib.rs              ← EMBP Gateway: Public API exports
├── main.rs             ← Lambda entry point & business logic  
├── domain.rs           ← Domain entities (Action, Priority)
tests/
├── integration_tests.rs ← End-to-end lambda testing
testdata/
├── *.json              ← Test input files
scripts/
├── build.sh            ← Unified build and test pipeline
```

**Key EMBP Benefits Demonstrated:**
- **Explicit dependencies** - All inter-module dependencies visible in gateways
- **Controlled boundaries** - Clear separation between public API and internals  
- **Refactoring safety** - Internal changes don't break external consumers
- **Clean imports** - `domain::Action` vs deep imports like `types::internal::Action`

### Domain Model

```rust
pub enum Priority {
    Urgent,    // Higher priority, appears first in results
    Normal,    // Standard priority
}

pub struct Action {
    pub entity_id: String,           // Unique identifier for deduplication
    pub last_action_time: DateTime<Utc>,  // When action was last performed
    pub next_action_time: DateTime<Utc>,  // When action should be performed next  
    pub priority: Priority,          // Action priority level
}
```

## 🔧 Business Rules

The Lambda applies these filtering and processing rules:

1. **Time-based filtering:**
   - `next_action_time` must be within **90 days or less** from today (inclusive)
   - `last_action_time` must be **more than 7 days ago** (strictly less than)

2. **Deduplication:**
   - At most one action per `entity_id`
   - "Last occurrence wins" when duplicates exist

3. **Priority sorting:**
   - `Urgent` actions appear before `Normal` actions
   - Within same priority, order may vary (HashMap-dependent)

## 🧪 Testing Strategy

### Unit Tests (`src/main.rs`)
- Business logic validation
- Edge case boundary testing (exactly 7 days, exactly 90 days)
- Deduplication behavior with priority conflicts
- Date parsing and filtering logic

### Integration Tests (`tests/integration_tests.rs`)
- **Real lambda execution** using `cargo lambda invoke`
- **End-to-end validation** from JSON input to JSON output
- **Error handling** verification (invalid enum variants)
- **Order-agnostic testing** for robust HashMap-based results

**Test Data Files:**
- `01_sample-input.json` - Basic filtering and deduplication
- `02_priority-input.json` - Priority sorting validation  
- `03_bad-input.json` - Error handling (invalid priority variant)
- `04_edge-cases.json` - Boundary conditions and complex scenarios

## 🚀 Usage

### Prerequisites

```bash
# Docker and Docker Compose
docker --version
docker compose --version

# Rust toolchain is automatically managed via rust-toolchain.toml (Rust 1.85)
```

### Running the Lambda

```bash
# Run complete build and test pipeline
./scripts/build.sh

# Invoke with sample data
cargo lambda invoke --data-file testdata/01_sample-input.json

# Expected output:
# [{"entity_id":"entity_1","last_action_time":"2025-06-01T00:00:00Z",
#   "next_action_time":"2025-07-01T00:00:00Z","priority":"normal"},
#  {"entity_id":"entity_3","last_action_time":"2025-05-01T00:00:00Z",
#   "next_action_time":"2025-07-10T00:00:00Z","priority":"normal"}]
```

### Development Workflow

```bash
# First run (builds everything)
./scripts/build.sh  # ~9 minutes initial build

# Subsequent runs (cached)
./scripts/build.sh  # ~30 seconds with Docker layer cache

# Test different scenarios
cargo lambda invoke --data-file testdata/02_priority-input.json
cargo lambda invoke --data-file testdata/04_edge-cases.json

# View logs
docker logs aws-lambda-action-filter-lambda-1

# Clean up when done
docker compose down
```

## 🚀 Continuous Integration

The project includes comprehensive CI that runs on every push and pull request:

- **Containerized builds** ensuring consistent environments
- **Code formatting** validation with `rustfmt`
- **Unit tests** for business logic validation  
- **Integration tests** with real lambda runtime execution
- **Unified pipeline** using the same `scripts/build.sh` locally and in CI

All tests must pass before merging, ensuring production readiness.

## 📊 Example Processing

**Input:**
```json
[
  {
    "entity_id": "entity_1",
    "last_action_time": "2025-06-20T00:00:00Z",
    "next_action_time": "2025-07-10T00:00:00Z", 
    "priority": "urgent"
  },
  {
    "entity_id": "entity_1",
    "last_action_time": "2025-06-01T00:00:00Z",
    "next_action_time": "2025-07-01T00:00:00Z",
    "priority": "normal"
  },
  {
    "entity_id": "entity_2", 
    "last_action_time": "2025-03-01T00:00:00Z",
    "next_action_time": "2026-01-01T00:00:00Z",
    "priority": "urgent"
  }
]
```

**Processing Steps:**
1. **Time filtering:** entity_2 removed (next_action > 90 days away)
2. **Deduplication:** entity_1 keeps last occurrence (normal priority)  
3. **Sorting:** Results ordered by priority (urgent first, then normal)

**Output:**
```json
[
  {
    "entity_id": "entity_1",
    "last_action_time": "2025-06-01T00:00:00Z", 
    "next_action_time": "2025-07-01T00:00:00Z",
    "priority": "normal"
  }
]
```

## 🛠️ Development Features

- **Containerized development** with Docker Compose for consistency
- **Pinned Rust toolchain** (1.85) via `rust-toolchain.toml`
- **Structured logging** with `tracing` for observability
- **Comprehensive error handling** with proper error propagation
- **Serde integration** for robust JSON serialization/deserialization
- **Type safety** with strongly-typed domain models
- **Date/time handling** using `chrono` for UTC timestamps

## ⚠️ Attribution & License

> This project is based on a challenge provided during a technical interview.  
> The original version appears to have been authored by Illya, based on the GitHub repository he shared during the interview process.
> Permission to publish and extend the original code was granted by Illya.

## 🏛️ Architecture Patterns

This project serves as a reference implementation demonstrating:

- **EMBP (Explicit Module Boundary Pattern)** for Rust project organization
- **Domain-Driven Design** principles in Rust
- **Integration testing** strategies for AWS Lambda functions
- **Error handling** patterns in serverless Rust applications
- **Clean Architecture** with clear separation of concerns
- **Containerized development workflows** for Rust projects
