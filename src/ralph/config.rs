use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode {
    #[default]
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "acceptEdits")]
    AcceptEdits,
    #[serde(rename = "dontAsk")]
    DontAsk,
}

impl PermissionMode {
    pub fn cycle(self) -> Self {
        match self {
            Self::Default => Self::AcceptEdits,
            Self::AcceptEdits => Self::DontAsk,
            Self::DontAsk => Self::Default,
        }
    }

    pub fn as_cli_value(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::AcceptEdits => Some("acceptEdits"),
            Self::DontAsk => Some("dontAsk"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::AcceptEdits => "acceptEdits",
            Self::DontAsk => "dontAsk",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RalphConfig {
    #[serde(rename = "dangerouslySkipPermissions")]
    pub dangerously_skip_permissions: bool,

    #[serde(rename = "permissionMode", default)]
    pub permission_mode: PermissionMode,
}
