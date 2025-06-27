use chrono::{DateTime, Duration, Utc};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::{Eq, Ordering, PartialOrd};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
enum Priority {
    Urgent,
    Normal,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, PartialOrd)]
struct Action {
    entity_id: String,
    last_action_time: DateTime<Utc>,
    next_action_time: DateTime<Utc>,
    priority: Priority,
}

impl Ord for Action {
    // ---
    fn cmp(&self, other: &Self) -> Ordering {
        // ---

        self.next_action_time.cmp(&other.next_action_time)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // ---
    tracing_subscriber::fmt()
        .with_env_filter("info") // or customize with RUST_LOG
        .with_target(false)
        .without_time()
        .init();

    tracing::info!("Lambda starting...");

    let func = service_fn(filter_actions);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn filter_actions(event: LambdaEvent<Value>) -> Result<Value, Error> {
    // ---

    tracing::info!(
        "Processing event with {} actions",
        event.payload.as_array().map(|v| v.len()).unwrap_or(0),
    );

    let (value, _context) = event.into_parts();
    let input: Vec<Action> = serde_json::from_value(value)?;

    let actions = process_actions(input);

    tracing::info!("Returning {} filtered actions", actions.len());

    Ok(json!(actions))
}

fn process_actions(input: Vec<Action>) -> Vec<Action> {
    // ---

    let today = Utc::now();

    let filtered: Vec<Action> = input
        .into_iter()
        .filter(|a| {
            // Skip if next_action_time is more than 90 days away
            a.next_action_time <= today + Duration::days(90)
        })
        .filter(|a| {
            // Skip if last_action_time is within the past 7 days
            a.last_action_time <= today - Duration::days(7)
        })
        .collect();

    // Deduplicate: keep one per entity_id (last one wins)
    use std::collections::HashMap;
    let mut map = HashMap::new();
    for action in filtered {
        map.insert(action.entity_id.clone(), action);
    }

    let mut deduped: Vec<Action> = map.into_values().collect();

    deduped.sort_by(|a, b| a.priority.cmp(&b.priority));

    deduped
}

#[cfg(test)]
mod tests {
    // ---

    use super::*;
    use anyhow::{ensure, Result};

    fn parse_date(s: &str) -> Result<DateTime<Utc>> {
        // ---
        let temp = DateTime::parse_from_rfc3339(s)?;
        Ok(temp.with_timezone(&Utc))
    }
    #[test]
    fn test_filter_and_sort_actions() -> Result<()> {
        // ---

        let input = vec![
            Action {
                entity_id: "entity_1".to_string(),
                last_action_time: parse_date("2025-06-20T00:00:00Z")?,
                next_action_time: parse_date("2025-07-10T00:00:00Z")?,
                priority: Priority::Urgent,
            },
            Action {
                entity_id: "entity_2".to_string(),
                last_action_time: parse_date("2025-06-01T00:00:00Z")?,
                next_action_time: parse_date("2025-07-01T00:00:00Z")?,
                priority: Priority::Normal,
            },
            Action {
                entity_id: "entity_3".to_string(),
                last_action_time: parse_date("2025-03-01T00:00:00Z")?,
                next_action_time: parse_date("2026-01-01T00:00:00Z")?,
                priority: Priority::Urgent, // should be excluded (next_action too far)
            },
            Action {
                entity_id: "entity_4".to_string(),
                last_action_time: parse_date("2025-06-25T00:00:00Z")?,
                next_action_time: parse_date("2025-07-10T00:00:00Z")?,
                priority: Priority::Urgent, // should be excluded (last_action < 7 days ago)
            },
        ];

        let output = process_actions(input);

        // Verify we have exactly 2 actions after filtering
        ensure!(
            output.len() == 2,
            "Expected 2 actions after filtering, got {}",
            output.len()
        );

        // Verify the complete order: Urgent priority comes first, then Normal
        ensure!(
            output[0].entity_id == "entity_1",
            "Expected first action to be entity_1, got {}",
            output[0].entity_id
        );
        ensure!(
            output[0].priority == Priority::Urgent,
            "Expected first action to have Urgent priority, got {:?}",
            output[0].priority
        );

        ensure!(
            output[1].entity_id == "entity_2",
            "Expected second action to be entity_2, got {}",
            output[1].entity_id
        );
        ensure!(
            output[1].priority == Priority::Normal,
            "Expected second action to have Normal priority, got {:?}",
            output[1].priority
        );

        Ok(())
    }

    #[test]
    fn test_deduplication_with_priority_conflict() -> Result<()> {
        // ---

        let input = vec![
            Action {
                entity_id: "duplicate".to_string(),
                last_action_time: parse_date("2025-05-01T00:00:00Z")?,
                next_action_time: parse_date("2025-07-01T00:00:00Z")?,
                priority: Priority::Normal,
            },
            Action {
                entity_id: "duplicate".to_string(),
                last_action_time: parse_date("2025-05-01T00:00:00Z")?,
                next_action_time: parse_date("2025-07-01T00:00:00Z")?,
                priority: Priority::Urgent,
            },
        ];

        let output = process_actions(input);
        ensure!(
            output[0].entity_id == "duplicate",
            "Expected action to be for entity 'duplicate', got {}",
            output[0].entity_id
        );

        // Currently keeps last seen, so should be Urgent
        ensure!(
            output[0].priority == Priority::Urgent,
            "Expected single remaining item to be Urgent"
        );

        Ok(())
    }

    #[test]
    fn test_last_action_time_exactly_7_days() -> Result<()> {
        // ---
        let today = Utc::now();
        let input = vec![Action {
            entity_id: "edge_7_days".to_string(),
            last_action_time: today - Duration::days(7),
            next_action_time: today + Duration::days(10),
            priority: Priority::Normal,
        }];

        let output = process_actions(input);
        ensure!(output.len() == 1, "Action 7 days old should be included");
        Ok(())
    }

    #[test]
    fn test_next_action_time_exactly_90_days() -> Result<()> {
        // ---
        let today = Utc::now();
        let input = vec![Action {
            entity_id: "edge_90_days".to_string(),
            last_action_time: today - Duration::days(10),
            next_action_time: today + Duration::days(90),
            priority: Priority::Normal,
        }];

        let output = process_actions(input);
        ensure!(output.len() == 1, "Action 90 days out should be included");
        Ok(())
    }
}
