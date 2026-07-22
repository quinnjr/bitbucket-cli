use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Deploy keys
// ---------------------------------------------------------------------------

/// An SSH deploy key granting read access to a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployKey {
    pub id: Option<u64>,
    /// The SSH public key material (e.g. `ssh-ed25519 AAAA... user@host`).
    pub key: Option<String>,
    pub label: Option<String>,
    pub comment: Option<String>,
}

/// Request body for adding a deploy key to a repository.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct AddDeployKeyRequest {
    pub key: String,
    pub label: String,
}

// ---------------------------------------------------------------------------
// Deployment environments
// ---------------------------------------------------------------------------

/// A deployment environment (e.g. Test, Staging, Production).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub uuid: Option<String>,
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_type: Option<EnvironmentType>,
}

/// The category of a deployment environment: Test, Staging, or Production.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Request body for creating a deployment environment.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CreateEnvironmentRequest {
    pub name: String,
    pub environment_type: EnvironmentType,
}

// ---------------------------------------------------------------------------
// Deployments
// ---------------------------------------------------------------------------

/// A deployment of a release to an environment.
///
/// The Bitbucket deployments API is only loosely documented, so the nested
/// shapes here are kept permissive (`Option` everywhere, raw JSON for the
/// deep-nested parts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub uuid: Option<String>,
    pub state: Option<DeploymentState>,
    pub environment: Option<Environment>,
    pub release: Option<serde_json::Value>,
}

/// The state of a deployment (e.g. UNDEPLOYED, IN_PROGRESS, COMPLETED).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentState {
    pub name: Option<String>,
    /// Nested status object; shape varies by state, so it is kept raw.
    pub status: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Branching model
// ---------------------------------------------------------------------------

/// A repository's effective branching model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchingModel {
    pub development: Option<ModelBranch>,
    pub production: Option<ModelBranch>,
    pub branch_types: Option<Vec<BranchType>>,
}

/// The development or production branch in a branching model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBranch {
    pub name: Option<String>,
    /// The concrete branch this resolves to (only present in the effective
    /// model, not in the settings representation).
    pub branch: Option<crate::models::Branch>,
    pub use_mainbranch: Option<bool>,
}

/// A branch type prefix mapping (e.g. kind `feature` → prefix `feature/`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchType {
    pub kind: Option<String>,
    pub prefix: Option<String>,
}

/// Partial update for a repository's branching model settings. Unset fields
/// are omitted from the request body so the corresponding settings are left
/// untouched.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateBranchingModelRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub development: Option<BranchingModelBranchSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production: Option<BranchingModelBranchSettings>,
}

impl UpdateBranchingModelRequest {
    pub fn is_empty(&self) -> bool {
        self.development.is_none() && self.production.is_none()
    }
}

/// Settings for one branch (development or production) in a branching model
/// settings update. `enabled` only applies to the production branch.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BranchingModelBranchSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_mainbranch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::{
        BranchingModelBranchSettings, CreateEnvironmentRequest, EnvironmentType,
        UpdateBranchingModelRequest,
    };

    #[test]
    fn add_deploy_key_request_serializes_key_and_label() {
        let request = super::AddDeployKeyRequest {
            key: "ssh-ed25519 AAAA host".to_string(),
            label: "deploy".to_string(),
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "key": "ssh-ed25519 AAAA host", "label": "deploy" })
        );
    }

    #[test]
    fn create_environment_request_nests_environment_type() {
        let request = CreateEnvironmentRequest {
            name: "prod".to_string(),
            environment_type: EnvironmentType {
                name: Some("Production".to_string()),
            },
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "name": "prod",
                "environment_type": { "name": "Production" }
            })
        );
    }

    #[test]
    fn update_branching_model_request_omits_unset_fields() {
        let request = UpdateBranchingModelRequest {
            development: Some(BranchingModelBranchSettings {
                name: Some("develop".to_string()),
                use_mainbranch: Some(false),
                enabled: None,
            }),
            production: None,
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "development": { "name": "develop", "use_mainbranch": false }
            })
        );
    }

    #[test]
    fn update_branching_model_request_is_empty() {
        assert!(UpdateBranchingModelRequest::default().is_empty());

        let with_production = UpdateBranchingModelRequest {
            production: Some(BranchingModelBranchSettings::default()),
            ..Default::default()
        };
        assert!(!with_production.is_empty());
    }
}
