use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::repo::Repository;
use super::user::{Link, User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub uuid: String,
    pub build_number: u64,
    pub creator: Option<User>,
    pub repository: Option<Repository>,
    pub target: PipelineTarget,
    pub trigger: Option<PipelineTrigger>,
    pub state: PipelineState,
    pub created_on: DateTime<Utc>,
    pub completed_on: Option<DateTime<Utc>>,
    pub build_seconds_used: Option<u64>,
    pub links: Option<PipelineLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub ref_type: Option<String>,
    pub ref_name: Option<String>,
    pub selector: Option<PipelineSelector>,
    pub commit: Option<PipelineCommit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSelector {
    #[serde(rename = "type")]
    pub selector_type: String,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCommit {
    pub hash: String,
    pub message: Option<String>,
    #[serde(rename = "type")]
    pub commit_type: Option<String>,
    pub links: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTrigger {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub trigger_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    pub name: PipelineStateName,
    #[serde(rename = "type")]
    pub state_type: String,
    pub result: Option<PipelineResult>,
    pub stage: Option<PipelineStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PipelineStateName {
    Pending,
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    Completed,
    Halted,
    Paused,
}

impl std::fmt::Display for PipelineStateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineStateName::Pending => write!(f, "PENDING"),
            PipelineStateName::InProgress => write!(f, "IN_PROGRESS"),
            PipelineStateName::Completed => write!(f, "COMPLETED"),
            PipelineStateName::Halted => write!(f, "HALTED"),
            PipelineStateName::Paused => write!(f, "PAUSED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub name: PipelineResultName,
    #[serde(rename = "type")]
    pub result_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PipelineResultName {
    Successful,
    Failed,
    Error,
    Stopped,
    Expired,
}

impl std::fmt::Display for PipelineResultName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineResultName::Successful => write!(f, "SUCCESSFUL"),
            PipelineResultName::Failed => write!(f, "FAILED"),
            PipelineResultName::Error => write!(f, "ERROR"),
            PipelineResultName::Stopped => write!(f, "STOPPED"),
            PipelineResultName::Expired => write!(f, "EXPIRED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub name: String,
    #[serde(rename = "type")]
    pub stage_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
    pub steps: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub uuid: String,
    pub name: Option<String>,
    pub started_on: Option<DateTime<Utc>>,
    pub completed_on: Option<DateTime<Utc>>,
    pub state: Option<PipelineStepState>,
    pub image: Option<PipelineImage>,
    pub setup_commands: Option<Vec<PipelineCommand>>,
    pub script_commands: Option<Vec<PipelineCommand>>,
    pub links: Option<PipelineStepLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStepState {
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
    pub result: Option<PipelineStepResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStepResult {
    pub name: String,
    #[serde(rename = "type")]
    pub result_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineImage {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCommand {
    pub name: String,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStepLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
    pub log: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPipelineRequest {
    pub target: TriggerPipelineTarget,
    /// Pipeline variables passed to the run (secured ones are masked in logs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<PipelineVariableInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPipelineTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    /// Set for `pipeline_ref_target` targets; absent for commit targets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_type: Option<String>,
    /// Set for `pipeline_ref_target` targets; absent for commit targets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<TriggerPipelineSelector>,
    /// Set for `pipeline_commit_target` targets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<TriggerPipelineCommit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPipelineSelector {
    #[serde(rename = "type")]
    pub selector_type: String,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPipelineCommit {
    pub hash: String,
    #[serde(rename = "type")]
    pub commit_type: String,
}

/// A key/value variable supplied when triggering a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineVariableInput {
    pub key: String,
    pub value: String,
    pub secured: bool,
}

impl TriggerPipelineRequest {
    /// Trigger a pipeline on the head of `branch`.
    pub fn for_branch(branch: &str) -> Self {
        Self {
            target: TriggerPipelineTarget {
                target_type: "pipeline_ref_target".to_string(),
                ref_type: Some("branch".to_string()),
                ref_name: Some(branch.to_string()),
                selector: None,
                commit: None,
            },
            variables: None,
        }
    }

    /// Trigger a pipeline on a specific commit hash.
    pub fn for_commit(hash: &str) -> Self {
        Self {
            target: TriggerPipelineTarget {
                target_type: "pipeline_commit_target".to_string(),
                ref_type: None,
                ref_name: None,
                selector: None,
                commit: Some(TriggerPipelineCommit {
                    hash: hash.to_string(),
                    commit_type: "commit".to_string(),
                }),
            },
            variables: None,
        }
    }

    /// Select a custom pipeline (from `bitbucket-pipelines.yml`) to run.
    pub fn with_pipeline(mut self, pipeline: &str) -> Self {
        self.target.selector = Some(TriggerPipelineSelector {
            selector_type: "custom".to_string(),
            pattern: pipeline.to_string(),
        });
        self
    }

    /// Attach pipeline variables to the trigger request.
    pub fn with_variables(mut self, variables: Vec<PipelineVariableInput>) -> Self {
        self.variables = if variables.is_empty() {
            None
        } else {
            Some(variables)
        };
        self
    }
}

/// A pipeline variable configured on a repository or workspace.
///
/// Secured variables come back from the API without a `value`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineVariable {
    pub uuid: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub secured: Option<bool>,
}

/// A scheduled pipeline run configured on a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSchedule {
    pub uuid: String,
    pub enabled: Option<bool>,
    pub cron_pattern: Option<String>,
    /// Raw target object; `ref_name` inside identifies the branch.
    pub target: Option<serde_json::Value>,
}

/// A dependency cache stored for a repository's pipelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCache {
    pub uuid: String,
    pub name: Option<String>,
    pub path: Option<String>,
    pub file_size_bytes: Option<u64>,
}

/// Repository-level pipelines configuration (`pipelines_config`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelinesConfig {
    pub enabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{PipelineVariableInput, TriggerPipelineRequest};

    #[test]
    fn for_branch_builds_ref_target() {
        let req = TriggerPipelineRequest::for_branch("main");
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            json!({
                "target": {
                    "type": "pipeline_ref_target",
                    "ref_type": "branch",
                    "ref_name": "main"
                }
            })
        );
    }

    #[test]
    fn for_commit_builds_commit_target_without_ref() {
        let req = TriggerPipelineRequest::for_commit("abc123");
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            json!({
                "target": {
                    "type": "pipeline_commit_target",
                    "commit": {"hash": "abc123", "type": "commit"}
                }
            })
        );
    }

    #[test]
    fn with_pipeline_adds_custom_selector() {
        let req = TriggerPipelineRequest::for_branch("main").with_pipeline("nightly");
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            value["target"]["selector"],
            json!({"type": "custom", "pattern": "nightly"})
        );
    }

    #[test]
    fn with_variables_empty_leaves_field_absent() {
        let req = TriggerPipelineRequest::for_branch("main").with_variables(vec![]);
        assert!(req.variables.is_none());
        let value = serde_json::to_value(&req).unwrap();
        assert!(value.get("variables").is_none());
    }

    #[test]
    fn with_variables_serializes_key_value_secured() {
        let req = TriggerPipelineRequest::for_branch("main").with_variables(vec![
            PipelineVariableInput {
                key: "FOO".to_string(),
                value: "bar".to_string(),
                secured: true,
            },
        ]);
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(
            value["variables"],
            json!([{"key": "FOO", "value": "bar", "secured": true}])
        );
    }
}
