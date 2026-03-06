use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RalphConfig {
    #[serde(rename = "dangerouslySkipPermissions")]
    pub dangerously_skip_permissions: bool,
}
