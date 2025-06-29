use anyhow::{ensure, Result};
use aws_lambda_action_filter::{Action, Priority};
use chrono::{Duration, Utc};
use serde_json::{self, Value};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

/// Helper function to run cargo lambda invoke and parse the result
fn run_lambda_invoke(data_file: &str) -> Result<Vec<Action>> {
    // ---
    let output =
        Command::new("cargo").args(["lambda", "invoke", "--data-file", data_file]).output()?;

    ensure!(
        output.status.success(),
        "cargo lambda invoke failed with status: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)?;

    // The output should be a JSON array of actions
    let json_value: Value = serde_json::from_str(&stdout)?;
    let actions: Vec<Action> = serde_json::from_value(json_value)?;

    Ok(actions)
}

struct TestCase {
    entity_id: &'static str,
    next_offset: i64, // Days from now (positive = future)
    last_offset: i64, // Days from now (negative = past)
    priority: Priority,
    should_pass: bool, // Expected to pass filtering?
    description: &'static str,
}

const DUP_TEST_ID: &str = "dedup_test_id";

#[rustfmt::skip]
const EDGE_CASES: &[TestCase] = &[
    TestCase { entity_id: DUP_TEST_ID,                   next_offset: 30, last_offset: -10, priority: Priority::Urgent, should_pass: false, description: "Rule: deduplication (first occurrence, expected to be FILTERED)" },
    TestCase { entity_id: DUP_TEST_ID,                   next_offset: 35, last_offset: -15, priority: Priority::Normal, should_pass: true,  description: "Rule: deduplication (last occurrence wins, expected to be PASSED)" },
    TestCase { entity_id: "more_than_7_days_ago_fail",   next_offset: 20, last_offset: -7,  priority: Priority::Urgent, should_pass: false, description: "Rule: more than 7 days ago (fail <7)" },
    TestCase { entity_id: "more_than_7_days_ago_pass",   next_offset: 20, last_offset: -8,  priority: Priority::Urgent, should_pass: true,  description: "Rule: more than 7 days ago (pass =7)" },
    TestCase { entity_id: "more_than_7_days_ago_pass_2", next_offset: 25, last_offset: -10, priority: Priority::Urgent, should_pass: true,  description: "Rule: more than 7 days ago (pass >7)" },
    TestCase { entity_id: "within_90_days_fail",         next_offset: 91, last_offset: -30, priority: Priority::Normal, should_pass: false, description: "Rule: within 90 days (fail >90)" },
    TestCase { entity_id: "within_90_days_pass",         next_offset: 90, last_offset: -30, priority: Priority::Normal, should_pass: true,  description: "Rule: within 90 days (pass =90)" },
    TestCase { entity_id: "within_90_days_pass_2",       next_offset: 89, last_offset: -20, priority: Priority::Normal, should_pass: true,  description: "Rule: within 90 days (pass <90)" },
];

fn create_action(
    entity_id: &str,
    last_offset: i64,
    next_offset: i64,
    priority: Priority,
) -> Action {
    // ---
    let now = Utc::now();
    Action {
        entity_id: entity_id.to_string(),
        last_action_time: now + Duration::days(last_offset),
        next_action_time: now + Duration::days(next_offset),
        priority,
    }
}

fn generate_test_data() -> Result<String> {
    // ---
    let actions: Vec<Action> = EDGE_CASES
        .iter()
        .map(|test_case| {
            // ---
            create_action(
                test_case.entity_id,
                test_case.last_offset,
                test_case.next_offset,
                test_case.priority.clone(),
            )
        })
        .collect();

    let json = serde_json::to_string_pretty(&actions)?;
    Ok(json)
}

fn verify_test_expectations(results: &[Action]) -> (bool, Vec<String>) {
    // ---
    // Convert results to a map for O(1) lookup
    let result_map: HashMap<&str, &Action> =
        results.iter().map(|action| (action.entity_id.as_str(), action)).collect();

    let mut lines = Vec::new();
    let mut passed = true; // To be set only if setting to false

    // Iterate over test expectations and verify against results
    for test_case in EDGE_CASES {
        // ---
        let found = result_map.contains_key(test_case.entity_id);

        if test_case.entity_id == DUP_TEST_ID {
            // ---
            let (pass, line) =
                duplicate_test::verify_duplicate_testcase(test_case, &result_map, found);

            lines.push(line);

            if !pass {
                passed = false;
            }
            continue; // Skip to next test case
        }

        match (test_case.should_pass, found) {
            // ---
            (true, false) => {
                // ---
                lines.push(format!(
                    "❌ {:<28}: FILTERED   - Expected to PASS but was not found in results. {}-2",
                    test_case.entity_id, test_case.description
                ));
                passed = false;
            }
            (false, true) => {
                // ---
                lines.push(format!(
                    "❌ {:<28}: FILTERED   - Expected to be FILTERED but was found in results. {}-3",
                    test_case.entity_id, test_case.description
                ));
                passed = false;
            }
            (true, true) => {
                // ---
                lines.push(format!(
                    "✅ {:<28}: PASSED   - {}",
                    test_case.entity_id, test_case.description
                ));
            }
            (false, false) => {
                // ---
                lines.push(format!(
                    "✅ {:<28}: FILTERED - {}",
                    test_case.entity_id, test_case.description
                ));
            }
        }
    }
    (passed, lines)
}

#[test]
fn test_dynamic_edge_cases() -> Result<()> {
    // ---
    println!("Generating dynamic edge case test data...");

    // Generate test data with current timestamps
    let test_data = generate_test_data()?;

    // Write to temporary file
    let temp_file = "testdata/edge-cases-dynamic.json";
    fs::write(temp_file, &test_data)?;

    println!("Generated test data written to: {}", temp_file);
    println!("Test data preview:");
    println!("{}", test_data);
    println!();

    // Run the lambda with our generated data
    let results = run_lambda_invoke(temp_file)?;

    println!("Lambda returned {} actions", results.len());
    results.iter().for_each(|action| {
        println!("  :: {}", action.entity_id);
    });
    print!("---\n\n");

    // Verify all test expectations
    let (passed, lines) = verify_test_expectations(&results);
    let lines = lines.join("\n");

    println!("\nAll TestCase Results:");
    println!("{lines}");
    ensure!(passed, "Test failed");

    // Additional verification: check expected count
    // Should have 5 actions: 6 that should pass - 1 duplicate = 5
    // (DUP_TEST_ID appears twice but deduplicated to 1)
    let expected_count = 5;
    ensure!(
        results.len() == expected_count,
        "Expected {} actions after filtering and deduplication, got {}",
        expected_count,
        results.len()
    );

    // Verify priority sorting (urgent before normal)
    let mut seen_normal = false;
    for action in &results {
        // ---
        if action.priority == Priority::Normal {
            // ---
            seen_normal = true;
        } else if action.priority == Priority::Urgent && seen_normal {
            // ---
            ensure!(false, "Found urgent priority after normal priority - sorting failed");
        }
    }

    // Verify deduplication worked correctly
    let duplicate_count = results.iter().filter(|a| a.entity_id == DUP_TEST_ID).count();
    ensure!(
        duplicate_count == 1,
        "Expected exactly 1 'duplicate' entity after deduplication, found {}",
        duplicate_count
    );

    // Verify that the duplicate kept the last occurrence (Normal priority)
    if let Some(duplicate_action) = results.iter().find(|a| a.entity_id == "duplicate") {
        // ---
        ensure!(
            duplicate_action.priority == Priority::Normal,
            "Expected duplicate entity to keep last occurrence (Normal priority), got {:?}",
            duplicate_action.priority
        );
    }

    // Cleanup
    fs::remove_file(temp_file).ok();

    println!();
    println!("✅ All dynamic edge case tests passed!");
    println!("   - Boundary conditions verified");
    println!("   - Deduplication working correctly");
    println!("   - Priority sorting maintained");

    Ok(())
}

mod duplicate_test {
    // ---
    use super::{Action, TestCase};
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    pub fn verify_duplicate_testcase(
        test_case: &TestCase,
        result_map: &HashMap<&str, &Action>,
        is_found: bool,
    ) -> (bool, String) {
        // ---
        // Early return: Handle case where entity was not found in results
        if !is_found {
            // ---
            return handle_entity_not_found(test_case);
        }

        // Entity was found - check if it's the correct duplicate
        let duplicate_check_result = check_duplicate_correctness(test_case, result_map);

        match (test_case.should_pass, duplicate_check_result.is_correct) {
            // ---
            (true, true) => {
                // Expected to pass, and correct duplicate was kept
                (true, format_success_message(test_case, "PASSED"))
            }
            (true, false) => {
                // Expected to pass, but wrong duplicate was kept
                (false, format_wrong_duplicate_error(test_case, &duplicate_check_result))
            }
            (false, true) => {
                // Expected to be filtered, but correct duplicate was kept (this shouldn't happen)
                (false, format_unexpected_correct_duplicate_error(test_case))
            }
            (false, false) => {
                // Expected to be filtered, and wrong duplicate was kept (which is fine)
                (true, format_success_message(test_case, "FILTERED"))
            }
        }
    }

    // Helper function to handle when entity is not found in results
    fn handle_entity_not_found(test_case: &TestCase) -> (bool, String) {
        // ---
        if test_case.should_pass {
            // Expected to pass but was filtered out
            (
                false,
                format!(
                    "❌ {:<28}: FAILED   - {} Expected to PASS but was FILTERED.",
                    test_case.entity_id, test_case.description
                ),
            )
        } else {
            // Expected to be filtered and was filtered
            (
                true,
                format!(
                    "✅ {:<28}: PASSED   - {} (was not passed, which is what we expected)",
                    test_case.entity_id, test_case.description
                ),
            )
        }
    }

    // Struct to hold duplicate correctness check results
    struct DuplicateCheckResult {
        is_correct: bool,
        time_diff: i64,
    }

    // Helper function to check if the correct duplicate was kept
    fn check_duplicate_correctness(
        test_case: &TestCase,
        result_map: &HashMap<&str, &Action>,
    ) -> DuplicateCheckResult {
        // ---
        let action = result_map.get(test_case.entity_id).unwrap();
        let now = Utc::now();
        let expected_next_time = now + Duration::days(test_case.next_offset);

        let time_diff = (action.next_action_time - expected_next_time).num_seconds().abs();
        let is_correct = time_diff < 60; // Allow small time differences

        DuplicateCheckResult { is_correct, time_diff }
    }

    // Helper functions for formatting messages
    fn format_success_message(test_case: &TestCase, status: &str) -> String {
        // ---
        format!("✅ {:<28}: {:<8} - {}", test_case.entity_id, status, test_case.description)
    }

    fn format_wrong_duplicate_error(
        test_case: &TestCase,
        check_result: &DuplicateCheckResult,
    ) -> String {
        // ---
        format!(
            "❌ {:<28}: FILTERED - {}. Wrong DUPLICATE kept, \
             Expected next_offset: {}, but found action with different timing. {}",
            test_case.entity_id,
            test_case.description,
            test_case.next_offset,
            check_result.time_diff
        )
    }

    fn format_unexpected_correct_duplicate_error(test_case: &TestCase) -> String {
        // ---
        format!(
            "❌ {:<28}: FAILED   - {} Expected to be FILTERED but correct duplicate was found in results.",
            test_case.entity_id, test_case.description
        )
    }
}
