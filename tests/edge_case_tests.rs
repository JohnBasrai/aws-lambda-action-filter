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

#[rustfmt::skip]
const EDGE_CASES: &[TestCase] = &[
    TestCase { entity_id: "dedup_first_occurrence",        next_offset: 30, last_offset: -10, priority: Priority::Urgent, should_pass: true,  description: "Tests deduplication (first occurrence)" },
    TestCase { entity_id: "dedup_first_occurrence",        next_offset: 35, last_offset: -15, priority: Priority::Normal, should_pass: true,  description: "Tests deduplication (last occurrence wins)" },
    TestCase { entity_id: "more_than_7_days_ago_fail",     next_offset: 20, last_offset: -7,  priority: Priority::Urgent, should_pass: false, description: "Tests 'more than 7 days ago' rule (should fail)" },
    TestCase { entity_id: "more_than_7_days_ago_pass",     next_offset: 20, last_offset: -8,  priority: Priority::Urgent, should_pass: true,  description: "Tests 'more than 7 days ago' rule (should pass)" },
    TestCase { entity_id: "more_than_7_days_ago_pass_2",   next_offset: 25, last_offset: -10, priority: Priority::Urgent, should_pass: true,  description: "Tests 'more than 7 days ago' rule (should pass)" },
    TestCase { entity_id: "within_90_days_fail",           next_offset: 91, last_offset: -30, priority: Priority::Normal, should_pass: false, description: "Tests 'within 90 days' rule (should fail at 91 days)" },
    TestCase { entity_id: "within_90_days_pass",           next_offset: 90, last_offset: -30, priority: Priority::Normal, should_pass: true,  description: "Tests 'within 90 days' rule boundary (should pass)" },
    TestCase { entity_id: "within_90_days_pass_2",         next_offset: 89, last_offset: -20, priority: Priority::Normal, should_pass: true,  description: "Tests 'within 90 days' rule (should pass)" },
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

fn verify_test_expectations(results: &[Action]) -> Result<()> {
    // ---
    let prefix = "verify_test_expectations";

    // Convert results to a map for O(1) lookup
    let result_map: HashMap<&str, &Action> =
        results.iter().map(|action| (action.entity_id.as_str(), action)).collect();

    // Iterate over test expectations and verify against results
    for test_case in EDGE_CASES {
        // ---
        let found_in_results = result_map.contains_key(test_case.entity_id);

        match (test_case.should_pass, found_in_results) {
            // ---
            (true, false) => {
                // ---
                ensure!(
                    false,
                    "{prefix}: {} - Expected to pass but was filtered out. {}",
                    test_case.entity_id,
                    test_case.description
                );
            }
            (false, true) => {
                // ---
                ensure!(
                    false,
                    "{prefix}: {} - Expected to be filtered out but found in results. {}",
                    test_case.entity_id,
                    test_case.description
                );
            }
            (true, true) => {
                // ---
                println!("✓ {:<28}: PASS     - {}", test_case.entity_id, test_case.description);
            }
            (false, false) => {
                // ---
                println!("✓ {:<28}: FILTERED - {}", test_case.entity_id, test_case.description);
            }
        }
    }

    Ok(())
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

    // Verify all test expectations
    verify_test_expectations(&results)?;

    // Additional verification: check expected count
    // Should have 5 actions: 6 that should pass - 1 duplicate = 5
    // (dedup_first_occurrence appears twice but deduplicated to 1)
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
    let duplicate_count =
        results.iter().filter(|a| a.entity_id == "dedup_first_occurrence").count();
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
