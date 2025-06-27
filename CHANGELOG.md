# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project uses [semantic versioning](https://semver.org/).

---

## [Unreleased]

- Awaiting license and attribution confirmation from the original author

---

## [0.1.1] – 2025-06-27

### Changed 
- Move action filtering logic to `process_actions` function
  - Improves modularity and reuse between production and test code.
  - Eliminated test-only duplicate of the filtering logic.
  - Maintains identical behavior with improved structure and testability.

### Added
- Added `process_actions` function to encapsulate business logic
- Wrote unit tests using `anyhow::Result` and `ensure` for panic-free testing
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
