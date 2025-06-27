use chrono::{DateTime, Duration, Utc};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::{Eq, Ordering, PartialOrd};
use std::collections::HashMap;

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

        if self.next_action_time < other.next_action_time {
            return Ordering::Less;
        } else if self.next_action_time > other.next_action_time {
            return Ordering::Greater;
        }
        if self.next_action_time < other.next_action_time {
            return Ordering::Less;
        } else if self.next_action_time > other.next_action_time {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // ---

    let func = service_fn(filter_actions);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn filter_actions(event: LambdaEvent<Value>) -> Result<Value, Error> {
    // ---

    let now = Utc::now();

    let (value, _context) = event.into_parts();
    let input: Vec<Action> = serde_json::from_value(value)?;

    let mut selected: HashMap<String, Action> = HashMap::new();

    for action in input {
        let days_since_last = (now - action.last_action_time).to_std()?.as_secs() / 86400;
        if days_since_last < 7 {
            continue;
        }

        if action.next_action_time > now + Duration::days(90) {
            continue;
        }

        selected.entry(action.entity_id.clone()).or_insert(action);
    }

    let mut actions: Vec<Action> = selected.into_iter().map(|(_, v)| v).collect();

    actions.sort_by_key(|a| a.priority.clone());

    Ok(json!(actions))
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

    fn process_actions(input: Vec<Action>) -> Vec<Action> {
        // ---

        let today = Utc::now();

        let filtered: Vec<Action> = input
            .into_iter()
            .filter(|a| {
                // Skip if next_action_time is more than 90 days away
                a.next_action_time <= today + chrono::Duration::days(90)
            })
            .filter(|a| {
                // Skip if last_action_time is within the past 7 days
                a.last_action_time <= today - chrono::Duration::days(7)
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

        let expected_order = vec!["entity_1", "entity_2"];

        let output = process_actions(input);
        let ids: Vec<_> = output.iter().map(|a| a.entity_id.as_str()).collect();

        assert_eq!(ids, expected_order);
        assert_eq!(output[0].priority, Priority::Urgent);
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
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].entity_id, "duplicate");
        // Currently keeps last seen, so should be Urgent
        assert_eq!(output[0].priority, Priority::Urgent);
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
