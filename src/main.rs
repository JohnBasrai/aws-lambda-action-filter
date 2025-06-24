use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use lambda_runtime::{handler_fn, Context, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Action {
    entity_id: String,
    last_action_time: DateTime<Utc>,
    next_action_time: DateTime<Utc>,
    priority: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(filter_actions);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn filter_actions(event: Value, _: Context) -> Result<Value, Error> {
    let now = Utc::now();

    let input: Vec<Action> = serde_json::from_value(event)?;

    let mut selected: HashMap<String, Action> = HashMap::new();

    for action in input {
        let days_since_last = (now - action.last_action_time).to_std().unwrap().as_secs() / 86400;
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
