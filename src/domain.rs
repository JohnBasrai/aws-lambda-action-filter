use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Priority level for actions, with Urgent taking precedence over Normal
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Urgent,
    Normal,
}

/// Represents an action to be performed on an entity
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct Action {
    /// Unique identifier for the entity this action applies to
    pub entity_id: String,
    /// Timestamp of when this action was last performed
    pub last_action_time: DateTime<Utc>,
    /// Timestamp of when this action should be performed next
    pub next_action_time: DateTime<Utc>,
    /// Priority level of this action
    pub priority: Priority,
}

impl Ord for Action {
    /// Orders actions by their next_action_time (earliest first)
    fn cmp(&self, other: &Self) -> Ordering {
        // ---
        self.next_action_time.cmp(&other.next_action_time)
    }
}

impl PartialOrd for Action {
    // ---
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
