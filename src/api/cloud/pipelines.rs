//
//  bitbucket-cli
//  api/cloud/pipelines.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Cloud pipelines API types and data structures.
//!
//! This module provides types for managing Bitbucket Pipelines, the built-in
//! CI/CD solution for Bitbucket Cloud repositories.
//!
//! # Overview
//!
//! Bitbucket Pipelines allows you to define build, test, and deployment
//! workflows using a `bitbucket-pipelines.yml` file in your repository.
//! This module provides types for querying pipeline status and triggering
//! new pipeline runs.
//!
//! # Pipeline Lifecycle
//!
//! ```text
//! PENDING -> IN_PROGRESS -> COMPLETED (SUCCESSFUL/FAILED/STOPPED)
//!                       \-> PAUSED -> IN_PROGRESS -> ...
//! ```
//!
//! # State Types
//!
//! * `pipeline_state_pending` - Pipeline is queued
//! * `pipeline_state_in_progress` - Pipeline is running
//! * `pipeline_state_completed` - Pipeline has finished
//! * `pipeline_state_paused` - Pipeline is paused (manual step)
//!
//! # Result Types (for completed pipelines)
//!
//! * `pipeline_state_completed_successful` - All steps passed
//! * `pipeline_state_completed_failed` - One or more steps failed
//! * `pipeline_state_completed_stopped` - Pipeline was manually stopped
//! * `pipeline_state_completed_error` - Infrastructure error
//!
//! # Example
//!
//! ```rust,no_run
//! use bitbucket_cli::api::cloud::pipelines::{
//!     TriggerPipelineRequest, TriggerTarget, PipelineVariable
//! };
//!
//! // Trigger a pipeline on the main branch
//! let request = TriggerPipelineRequest {
//!     target: TriggerTarget {
//!         target_type: "pipeline_ref_target".to_string(),
//!         ref_type: "branch".to_string(),
//!         ref_name: "main".to_string(),
//!         selector: None,
//!     },
//!     variables: vec![
//!         PipelineVariable {
//!             key: "DEPLOY_ENV".to_string(),
//!             value: "production".to_string(),
//!             secured: false,
//!         },
//!     ],
//! };
//! ```
//!
//! # Notes
//!
//! - Pipelines are configured via `bitbucket-pipelines.yml`
//! - Custom pipelines can be triggered manually with selectors
//! - Pipeline minutes are metered for most plans

use serde::{Deserialize, Serialize};

/// Represents a Bitbucket Pipeline run.
///
/// A pipeline is a single execution of your CI/CD workflow, triggered by
/// a push, pull request, or manual action.
///
/// # Fields
///
/// * `uuid` - Unique identifier for this pipeline run
/// * `build_number` - Sequential number within the repository
/// * `state` - Current state of the pipeline
/// * `target` - What triggered this pipeline (branch, tag, etc.)
/// * `created_on` - ISO 8601 timestamp of creation
/// * `completed_on` - ISO 8601 timestamp of completion (if finished)
/// * `duration_in_seconds` - Total runtime in seconds (if finished)
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::Pipeline;
///
/// fn display_pipeline(pipeline: &Pipeline) {
///     println!("Pipeline #{}: {}", pipeline.build_number, pipeline.state.name);
///     if let Some(duration) = pipeline.duration_in_seconds {
///         println!("  Duration: {}s", duration);
///     }
///     if let Some(ref result) = pipeline.state.result {
///         println!("  Result: {}", result.name);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Unique identifier for this pipeline run (includes curly braces).
    pub uuid: String,

    /// Sequential build number within the repository.
    /// Increments with each pipeline run.
    pub build_number: u64,

    /// Current state of the pipeline execution.
    pub state: PipelineState,

    /// The target that triggered this pipeline (branch, tag, etc.).
    pub target: PipelineTarget,

    /// ISO 8601 timestamp indicating when the pipeline was created.
    pub created_on: String,

    /// ISO 8601 timestamp indicating when the pipeline completed.
    /// Only present for completed pipelines.
    #[serde(default)]
    pub completed_on: Option<String>,

    /// Total duration of the pipeline run in seconds.
    /// Only present for completed pipelines.
    #[serde(default)]
    pub duration_in_seconds: Option<u64>,
}

/// The state of a pipeline execution.
///
/// Represents the current execution state and optional result for
/// completed pipelines.
///
/// # Fields
///
/// * `name` - Human-readable state name (e.g., `PENDING`, `IN_PROGRESS`)
/// * `state_type` - API type string for the state
/// * `result` - The result details (only for completed pipelines)
///
/// # State Types
///
/// * `pipeline_state_pending` - Queued, waiting to start
/// * `pipeline_state_in_progress` - Currently running
/// * `pipeline_state_completed` - Finished (check result for outcome)
/// * `pipeline_state_paused` - Paused at a manual step
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::PipelineState;
///
/// fn is_running(state: &PipelineState) -> bool {
///     state.state_type == "pipeline_state_in_progress"
/// }
///
/// fn is_successful(state: &PipelineState) -> bool {
///     state.result.as_ref()
///         .map(|r| r.result_type == "pipeline_state_completed_successful")
///         .unwrap_or(false)
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    /// Human-readable name of the state (e.g., `PENDING`, `IN_PROGRESS`).
    pub name: String,

    /// API type string identifying the state type.
    #[serde(rename = "type")]
    pub state_type: String,

    /// The result of the pipeline, if completed.
    #[serde(default)]
    pub result: Option<PipelineResult>,
}

/// The result of a completed pipeline.
///
/// Provides details about how a completed pipeline finished.
///
/// # Fields
///
/// * `name` - Human-readable result name (e.g., `SUCCESSFUL`, `FAILED`)
/// * `result_type` - API type string for the result
///
/// # Result Types
///
/// * `pipeline_state_completed_successful` - All steps passed
/// * `pipeline_state_completed_failed` - One or more steps failed
/// * `pipeline_state_completed_stopped` - Manually stopped
/// * `pipeline_state_completed_error` - Infrastructure/system error
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::PipelineResult;
///
/// fn result_emoji(result: &PipelineResult) -> &'static str {
///     match result.result_type.as_str() {
///         "pipeline_state_completed_successful" => "OK",
///         "pipeline_state_completed_failed" => "FAILED",
///         "pipeline_state_completed_stopped" => "STOPPED",
///         _ => "?"
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Human-readable name of the result (e.g., `SUCCESSFUL`, `FAILED`).
    pub name: String,

    /// API type string identifying the result type.
    #[serde(rename = "type")]
    pub result_type: String,
}

/// The target that triggered a pipeline run.
///
/// Identifies what caused the pipeline to run, such as a branch push,
/// tag creation, or manual trigger.
///
/// # Fields
///
/// * `target_type` - The type of target (e.g., `pipeline_ref_target`)
/// * `ref_name` - The name of the reference (branch, tag, etc.)
/// * `ref_type` - The type of reference (`branch`, `tag`, etc.)
/// * `selector` - Optional selector for custom pipelines
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::PipelineTarget;
///
/// fn describe_target(target: &PipelineTarget) -> String {
///     match (target.ref_type.as_deref(), target.ref_name.as_deref()) {
///         (Some(ref_type), Some(ref_name)) => format!("{}: {}", ref_type, ref_name),
///         _ => "Unknown target".to_string(),
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTarget {
    /// The type of target that triggered the pipeline.
    #[serde(rename = "type")]
    pub target_type: String,

    /// The name of the reference (branch name, tag name, etc.).
    #[serde(default)]
    pub ref_name: Option<String>,

    /// The type of reference (`branch`, `tag`, `bookmark`).
    #[serde(default)]
    pub ref_type: Option<String>,

    /// Optional selector for custom pipeline configurations.
    #[serde(default)]
    pub selector: Option<PipelineSelector>,
}

/// Selector for custom pipeline configurations.
///
/// Used when running custom pipelines defined in `bitbucket-pipelines.yml`
/// under the `custom` section.
///
/// # Fields
///
/// * `selector_type` - The type of selector (e.g., `custom`)
/// * `pattern` - The name/pattern of the custom pipeline
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::PipelineSelector;
///
/// let selector = PipelineSelector {
///     selector_type: "custom".to_string(),
///     pattern: Some("deploy-production".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSelector {
    /// The type of selector (e.g., `custom`, `branches`, `tags`).
    #[serde(rename = "type")]
    pub selector_type: String,

    /// The pattern or name to match for the selector.
    #[serde(default)]
    pub pattern: Option<String>,
}

/// Represents an individual step within a pipeline.
///
/// A pipeline consists of multiple steps that execute sequentially or
/// in parallel as defined in the pipeline configuration.
///
/// # Fields
///
/// * `uuid` - Unique identifier for this step
/// * `name` - Human-readable name of the step
/// * `state` - Current state of the step execution
/// * `started_on` - ISO 8601 timestamp when the step started
/// * `completed_on` - ISO 8601 timestamp when the step completed
/// * `duration_in_seconds` - Duration of the step in seconds
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::PipelineStep;
///
/// fn display_steps(steps: &[PipelineStep]) {
///     for step in steps {
///         let duration = step.duration_in_seconds.map_or(
///             "running".to_string(),
///             |d| format!("{}s", d)
///         );
///         println!("  {} - {} ({})", step.name, step.state.name, duration);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Unique identifier for this step (includes curly braces).
    pub uuid: String,

    /// Human-readable name of the step as defined in the pipeline config.
    pub name: String,

    /// Current execution state of this step.
    pub state: PipelineState,

    /// ISO 8601 timestamp indicating when this step started executing.
    #[serde(default)]
    pub started_on: Option<String>,

    /// ISO 8601 timestamp indicating when this step completed.
    #[serde(default)]
    pub completed_on: Option<String>,

    /// Duration of this step in seconds.
    #[serde(default)]
    pub duration_in_seconds: Option<u64>,
}

/// Request payload for triggering a new pipeline run.
///
/// Used when making POST requests to manually trigger a pipeline.
///
/// # Fields
///
/// * `target` - The target specification (branch, tag, custom pipeline)
/// * `variables` - Optional variables to pass to the pipeline
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::{
///     TriggerPipelineRequest, TriggerTarget, TriggerSelector, PipelineVariable
/// };
///
/// // Trigger a custom pipeline with variables
/// let request = TriggerPipelineRequest {
///     target: TriggerTarget {
///         target_type: "pipeline_ref_target".to_string(),
///         ref_type: "branch".to_string(),
///         ref_name: "main".to_string(),
///         selector: Some(TriggerSelector {
///             selector_type: "custom".to_string(),
///             pattern: "deploy-staging".to_string(),
///         }),
///     },
///     variables: vec![
///         PipelineVariable {
///             key: "VERSION".to_string(),
///             value: "1.2.3".to_string(),
///             secured: false,
///         },
///     ],
/// };
/// ```
///
/// # Notes
///
/// - The target branch/tag must exist
/// - Custom pipeline must be defined in `bitbucket-pipelines.yml`
/// - Secured variables are masked in logs
#[derive(Debug, Clone, Serialize)]
pub struct TriggerPipelineRequest {
    /// The target specification for the pipeline.
    pub target: TriggerTarget,

    /// Optional variables to pass to the pipeline execution.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<PipelineVariable>,
}

/// Target specification for triggering a pipeline.
///
/// Specifies what branch, tag, or custom pipeline to run.
///
/// # Fields
///
/// * `target_type` - Type of target (typically `pipeline_ref_target`)
/// * `ref_type` - Type of reference (`branch`, `tag`, `bookmark`)
/// * `ref_name` - Name of the branch, tag, or bookmark
/// * `selector` - Optional selector for custom pipelines
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::TriggerTarget;
///
/// // Trigger default pipeline on a branch
/// let target = TriggerTarget {
///     target_type: "pipeline_ref_target".to_string(),
///     ref_type: "branch".to_string(),
///     ref_name: "develop".to_string(),
///     selector: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct TriggerTarget {
    /// The type of target. Typically `pipeline_ref_target`.
    #[serde(rename = "type")]
    pub target_type: String,

    /// The type of reference: `branch`, `tag`, or `bookmark`.
    pub ref_type: String,

    /// The name of the branch, tag, or bookmark.
    pub ref_name: String,

    /// Optional selector for custom pipeline configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<TriggerSelector>,
}

/// Selector for triggering custom pipelines.
///
/// Used to specify which custom pipeline definition to run.
///
/// # Fields
///
/// * `selector_type` - Type of selector (e.g., `custom`)
/// * `pattern` - Name of the custom pipeline
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::TriggerSelector;
///
/// let selector = TriggerSelector {
///     selector_type: "custom".to_string(),
///     pattern: "nightly-build".to_string(),
/// };
/// ```
///
/// # Notes
///
/// - The pattern must match a pipeline defined under `custom:` in the config
#[derive(Debug, Clone, Serialize)]
pub struct TriggerSelector {
    /// The type of selector. Use `custom` for custom pipelines.
    #[serde(rename = "type")]
    pub selector_type: String,

    /// The name of the custom pipeline to run.
    pub pattern: String,
}

/// A variable to pass to a pipeline run.
///
/// Variables can be used to customize pipeline behavior and pass
/// runtime configuration to build steps.
///
/// # Fields
///
/// * `key` - The variable name (must be a valid environment variable name)
/// * `value` - The variable value
/// * `secured` - Whether the value should be masked in logs
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pipelines::PipelineVariable;
///
/// // Regular variable
/// let version = PipelineVariable {
///     key: "BUILD_VERSION".to_string(),
///     value: "1.0.0".to_string(),
///     secured: false,
/// };
///
/// // Secret variable (masked in logs)
/// let token = PipelineVariable {
///     key: "DEPLOY_TOKEN".to_string(),
///     value: "secret-token-value".to_string(),
///     secured: true,
/// };
/// ```
///
/// # Notes
///
/// - Secured variables are masked with `***` in pipeline logs
/// - Variable keys should follow environment variable naming conventions
/// - Variables override repository-level variables with the same key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineVariable {
    /// The name of the variable. Should be a valid environment variable name.
    pub key: String,

    /// The value of the variable.
    pub value: String,

    /// Whether this variable's value should be masked in logs.
    /// Set to `true` for sensitive data like tokens or passwords.
    #[serde(default)]
    pub secured: bool,
}
