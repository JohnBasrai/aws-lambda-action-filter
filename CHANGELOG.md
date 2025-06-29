# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project uses [semantic versioning](https://semver.org/).

---
## [Unreleased]

### Changed
- **Test Organization**: Split integration tests into focused test files
  - `tests/basic_filter_tests.rs` - Core integration tests with static test data
  - `tests/edge_case_tests.rs` - Dynamic boundary condition tests
- **Test Infrastructure Enhancement**: Implemented dynamic test data generation
  - Added comprehensive boundary testing for time-based filtering rules
  - Self-documenting test cases with clear business rule descriptions
- **Enhanced Test Reporting**: Added aligned, formatted test output for better readability
- **Improved Test Validation**: Enhanced duplicate detection logic with precise timestamp validation
  - Added `verify_duplicate_testcase` function with comprehensive error messaging
  - Implemented detailed test case validation showing PASSED vs FILTERED status
  - Refactored test logic into organized `duplicate_test` module for better maintainability

### Added
- **Comprehensive Edge Case Testing**: Dynamic test data generation for boundary conditions
  - Automated testing of 7-day and 90-day filtering boundaries
  - Real-time duplicate detection validation with timing precision
- **Enhanced Test Output**: Clear, professional test result formatting
  - Color-coded test status (✅/❌) with descriptive failure messages
  - Detailed debug output for duplicate validation logic

### Technical
- Improved test maintainability with descriptive entity names and test descriptions
- Modularized duplicate testing logic for better code organization
- Enhanced error messages for easier debugging of test failures

---

## [0.2.0] – 2025-06-28

### Added

* **EMBP Architecture**: Adopted Explicit Module Boundary Pattern for clean crate structure

  * Created `mod.rs` gateways across all key modules (`domain/`, `repository/`, etc.)
  * Improved encapsulation, import hygiene, and module boundaries
* **Docker-Based Development Workflow**:

  * Added Docker Compose setup with Postgres and Redis
  * Hot-reload development via volume mounts
  * Introduced `scripts/build.sh` for unified format/lint/test/build pipeline
* **Standardized CI Pipeline**:

  * GitHub Actions CI mirrors local container-based workflow
  * Includes clippy, rustfmt, unit tests, and integration tests
  * Added containerized end-to-end tests using `cargo lambda`
* **Toolchain Pinning**: Rust version pinned to 1.85 for consistency

### Changed

* CI and local development now fully containerized
* Replaced fragile curl healthchecks with robust netcat-based port checks

### Fixed

* Integration test flakiness due to HashMap ordering:

  * Added order-agnostic assertions
  * Ensured reliable deduplication across input variants

### ⚠️ Breaking Changes

* Local development now **requires Docker**; native Rust-only workflow is no longer supported

---

## [0.1.1] – 2025-06-27

### Changed 
- Move action filtering logic to `process_actions` function
  - Improves modularity and reuse between production and test code.
  - Eliminated test-only duplicate of the filtering logic.
  - Maintains identical behavior with improved structure and testability.

### Added
- Added GitHub Actions CI with format check, build, and test steps
- Added `process_actions` function to encapsulate business logic
- Wrote unit tests using `anyhow::Result` and `ensure` for panic-free testing
- Added `tracing` and `tracing-subscriber` crates for structured logging.
- Added `parse_date` utility to safely parse ISO-8601 timestamps
- Added README summary and improvements section

### Fixed
- Corrected deprecated use of `lambda_runtime::handler_fn`
- Fixed business logic for:
  - Skipping actions less than 7 days old
  - Filtering future actions beyond 90 days
  - Sorting urgent priorities first
- Ensured only one action per `entity_id` is returned

---

## [0.0.1] – original version

- Initial assignment version, likely authored by Illya (via LinkedIn)
- Included broken logic, outdated dependencies, and no tests
