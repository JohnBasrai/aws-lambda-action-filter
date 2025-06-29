use anyhow::{ensure, Result};
use aws_lambda_action_filter::{Action, Priority};
use serde_json::Value;
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

/// Helper function that expects cargo lambda invoke to fail
fn expect_lambda_invoke_failure(data_file: &str) -> Result<String> {
    // ---
    let output =
        Command::new("cargo").args(["lambda", "invoke", "--data-file", data_file]).output()?;

    ensure!(!output.status.success(), "Expected cargo lambda invoke to fail, but it succeeded");

    let stderr = String::from_utf8(output.stderr)?;
    Ok(stderr)
}

#[test]
fn test_sample_input_integration() -> Result<()> {
    // ---
    let actions = run_lambda_invoke("testdata/01_sample-input.json")?;

    // Expected filtering results:
    // - entity_1: deduplication keeps the last occurrence (normal priority)
    // - entity_2: filtered out (next_action 2026-01-01 is > 90 days away)
    // - entity_3: passes all filters (normal priority)
    //
    // Both results have normal priority, so their relative order is not deterministic
    // (depends on HashMap::into_values() ordering after deduplication)

    ensure!(actions.len() == 2, "Expected exactly 2 actions, got {}", actions.len());

    // Collect entity_ids for verification
    let entity_ids: std::collections::HashSet<&String> =
        actions.iter().map(|a| &a.entity_id).collect();

    let expected_ids: std::collections::HashSet<&str> =
        ["entity_1", "entity_3"].iter().cloned().collect();

    // Verify we have exactly the expected entity_ids (order doesn't matter)
    for expected_id in &expected_ids {
        ensure!(
            entity_ids.iter().any(|&id| id == expected_id),
            "Expected to find entity_id '{}' in results",
            expected_id
        );
    }

    ensure!(
        entity_ids.len() == expected_ids.len(),
        "Expected exactly {} unique entity_ids, got {}",
        expected_ids.len(),
        entity_ids.len()
    );

    // Verify all results have normal priority (since both should be normal after deduplication)
    for action in &actions {
        ensure!(
            action.priority == Priority::Normal,
            "Expected all actions to have Normal priority after deduplication, got {:?} for {}",
            action.priority,
            action.entity_id
        );
    }

    // Verify entity_1 kept the second occurrence (normal priority, not urgent)
    let entity_1_action = actions.iter().find(|a| a.entity_id == "entity_1").unwrap();
    ensure!(
        entity_1_action.last_action_time.to_rfc3339() == "2025-06-01T00:00:00+00:00",
        "Expected entity_1 to keep the second occurrence with last_action_time 2025-06-01"
    );

    println!("Sample input returned expected {} actions (order may vary):", actions.len());
    for (i, action) in actions.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            i + 1,
            action.entity_id,
            if action.priority == Priority::Urgent { "urgent" } else { "normal" }
        );
    }

    Ok(())
}

#[test]
fn test_priority_input_integration() -> Result<()> {
    // ---
    let actions = run_lambda_invoke("testdata/02_priority-input.json")?;

    ensure!(!actions.is_empty(), "Expected some actions to be returned, got empty array");

    // Verify that urgent priorities come before normal priorities
    let mut seen_normal = false;
    for action in &actions {
        if action.priority == Priority::Normal {
            seen_normal = true;
        } else if action.priority == Priority::Urgent && seen_normal {
            panic!("Found urgent priority after normal priority - sorting is incorrect");
        }
    }

    // Count priorities for verification
    let urgent_count = actions.iter().filter(|a| a.priority == Priority::Urgent).count();
    let normal_count = actions.iter().filter(|a| a.priority == Priority::Normal).count();

    println!(
        "Priority input returned {} actions ({} urgent, {} normal):",
        actions.len(),
        urgent_count,
        normal_count
    );
    for (i, action) in actions.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            i + 1,
            action.entity_id,
            if action.priority == Priority::Urgent { "urgent" } else { "normal" }
        );
    }

    Ok(())
}

#[test]
fn test_bad_input_integration() -> Result<()> {
    // ---
    // NOTE: This test file fails during JSON deserialization in our filter_actions callback
    // at this line: `let input: Vec<Action> = serde_json::from_value(value)?;`
    // because "unknown" is not a valid Priority enum variant. The lambda runtime successfully
    // passes the JSON to our callback, but we fail when trying to deserialize it into Action structs.
    //
    // This DOES test our lambda's error handling, showing that serde deserialization errors
    // are properly propagated up as lambda errors.
    let error_output = expect_lambda_invoke_failure("testdata/03_bad-input.json")?;

    // Verify we get the expected deserialization error from our lambda
    ensure!(
        error_output.contains("unknown variant") && error_output.contains("unknown"),
        "Expected serde deserialization error about 'unknown variant', got: {}",
        error_output
    );

    ensure!(
        error_output.contains("urgent") && error_output.contains("normal"),
        "Expected error to mention valid variants 'urgent' and 'normal', got: {}",
        error_output
    );

    println!("Bad input correctly failed during our lambda's JSON deserialization:");
    println!("  Error contains 'unknown variant': ✓");
    println!("  Error mentions valid variants: ✓");
    println!("  This tests our serde error handling in filter_actions callback");

    Ok(())
}

#[test]
fn test_empty_input_array() -> Result<()> {
    // ---
    // Test that we can handle empty input arrays (this DOES test our callback)

    // Create a temporary test file with empty array
    let empty_input = "[]";
    std::fs::write("testdata/empty-input.json", empty_input)?;

    let actions = run_lambda_invoke("testdata/empty-input.json")?;

    ensure!(
        actions.is_empty(),
        "Expected empty array for empty input, got {} actions",
        actions.len()
    );

    // Clean up
    std::fs::remove_file("testdata/empty-input.json")?;

    println!("Empty input correctly returned empty array");

    Ok(())
}

#[test]
fn test_deduplication_integration() -> Result<()> {
    // ---
    let actions = run_lambda_invoke("testdata/01_sample-input.json")?;

    // Verify no duplicate entity_ids
    let mut entity_ids = std::collections::HashSet::new();
    for action in &actions {
        ensure!(
            entity_ids.insert(&action.entity_id),
            "Found duplicate entity_id: {}",
            action.entity_id
        );
    }

    println!("Deduplication test passed - all {} entity_ids are unique", actions.len());

    Ok(())
}

#[test]
fn test_priority_sorting_integration() -> Result<()> {
    // ---
    let actions = run_lambda_invoke("testdata/02_priority-input.json")?;

    // Group actions by priority
    let urgent_actions: Vec<_> =
        actions.iter().filter(|a| a.priority == Priority::Urgent).collect();
    let normal_actions: Vec<_> =
        actions.iter().filter(|a| a.priority == Priority::Normal).collect();

    println!(
        "Found {} urgent and {} normal priority actions",
        urgent_actions.len(),
        normal_actions.len()
    );

    // Verify that if we have both priorities, all urgent come before all normal
    if !urgent_actions.is_empty() && !normal_actions.is_empty() {
        // Find the last urgent and first normal in the original array
        let last_urgent_pos = actions.iter().rposition(|a| a.priority == Priority::Urgent).unwrap();
        let first_normal_pos = actions.iter().position(|a| a.priority == Priority::Normal).unwrap();

        ensure!(
            last_urgent_pos < first_normal_pos,
            "Urgent actions should come before normal actions"
        );

        println!("Priority sorting verified: all urgent actions come before normal actions");
    }

    Ok(())
}

#[test]
fn test_time_filtering_integration() -> Result<()> {
    // ---
    let actions = run_lambda_invoke("testdata/01_sample-input.json")?;

    // Verify all returned actions pass the time filters
    // Note: This test is date-dependent, so we just verify the structure is correct
    let now = chrono::Utc::now();

    for action in &actions {
        // Verify last_action_time is at least 7 days ago (or exactly 7 days)
        let days_since_last = now.signed_duration_since(action.last_action_time).num_days();
        ensure!(
            days_since_last >= 7,
            "Action {} has last_action_time only {} days ago (should be >= 7)",
            action.entity_id,
            days_since_last
        );

        // Verify next_action_time is within 90 days (or exactly 90 days)
        let days_until_next = action.next_action_time.signed_duration_since(now).num_days();
        ensure!(
            days_until_next <= 90,
            "Action {} has next_action_time {} days away (should be <= 90)",
            action.entity_id,
            days_until_next
        );

        // Verify timestamps make logical sense
        ensure!(
            action.last_action_time <= action.next_action_time,
            "Action {} has last_action_time after next_action_time",
            action.entity_id
        );
    }

    println!(
        "Time filtering verified: all {} actions pass the 7-day and 90-day filters",
        actions.len()
    );

    Ok(())
}
