use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Trial,
    Free,
    Pro,
}

impl Tier {
    /// Whether this tier allows cross-device sync
    pub fn allows_sync(&self) -> bool {
        matches!(self, Tier::Trial | Tier::Pro)
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tier::Trial => write!(f, "trial"),
            Tier::Free => write!(f, "free"),
            Tier::Pro => write!(f, "pro"),
        }
    }
}

impl Default for Tier {
    fn default() -> Self {
        Tier::Trial
    }
}
