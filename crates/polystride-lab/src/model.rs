use crate::{Result, error};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PROTOCOL: u32 = 1;
pub const MAX_ARTIFACT: usize = 8 * 1024 * 1024;
pub const MAX_FRAME: usize = 40 * 1024 * 1024;
pub const MAX_OUTPUT: usize = 64 * 1024;

#[must_use]
pub fn sha256(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = ring::digest::digest(&ring::digest::SHA256, bytes);
    let mut hex = String::with_capacity(64);
    for &byte in digest.as_ref() {
        hex.push(char::from(HEX[usize::from(byte >> 4)]));
        hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    hex
}

/// Validate an identifier before using it as a filesystem path component.
///
/// # Errors
///
/// Returns an error unless `id` is 1–64 ASCII letters, digits, hyphens or underscores.
pub fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(error(
            "IDs must be 1-64 ASCII letters, digits, hyphens or underscores",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MachineKind {
    Physical,
    VirtualMachine,
    Emulator,
    Unknown,
}

/// Operator declarations are intentionally separate from observed agent values.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub os: String,
    pub os_version: String,
    pub hardware_arch: String,
    pub machine_kind: MachineKind,
    pub baseline_id: String,
    pub notes: String,
}

impl Baseline {
    /// Check that the declared baseline identifies an intended OS/CPU target.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown OS/CPU or an empty version or baseline ID.
    pub fn validate(&self) -> Result<()> {
        if !["linux", "macos", "windows"].contains(&self.os.as_str())
            || !["x86_64", "aarch64"].contains(&self.hardware_arch.as_str())
            || self.os_version.is_empty()
            || self.baseline_id.is_empty()
        {
            return Err(error(
                "baseline requires an OS, version, CPU and baseline ID",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub id: String,
    pub host_id: String,
    pub source_revision: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub timeout_ms: u64,
    pub expected_exit: i32,
    pub expected_stdout: Vec<u8>,
    pub expected_stderr: Vec<u8>,
}

impl Recipe {
    /// Check identifiers, execution limits, arguments, and environment entries.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid IDs, missing revision, out-of-range limits,
    /// or arguments/environment entries that cannot be passed to a process.
    pub fn validate(&self) -> Result<()> {
        validate_id(&self.id)?;
        validate_id(&self.host_id)?;
        if self.source_revision.is_empty()
            || !(1..=300_000).contains(&self.timeout_ms)
            || self.args.len() > 256
            || self.expected_stdout.len() > MAX_OUTPUT
            || self.expected_stderr.len() > MAX_OUTPUT
        {
            return Err(error(
                "invalid recipe revision, timeout, arguments or output size",
            ));
        }
        if self.args.iter().any(|arg| arg.contains('\0'))
            || self
                .env
                .iter()
                .any(|(k, v)| k.is_empty() || k.contains(['\0', '=']) || v.contains('\0'))
        {
            return Err(error("invalid argument or environment entry"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub protocol: u32,
    pub recipe: Recipe,
    pub artifact_sha256: String,
    pub artifact: Vec<u8>,
}

impl Job {
    /// Snapshot an artifact and validated recipe into a versioned job.
    ///
    /// # Errors
    ///
    /// Returns an error if the recipe is invalid or the artifact is empty or oversized.
    pub fn new(recipe: Recipe, artifact: Vec<u8>) -> Result<Self> {
        let job = Self {
            protocol: PROTOCOL,
            recipe,
            artifact_sha256: sha256(&artifact),
            artifact,
        };
        job.validate()?;
        Ok(job)
    }

    /// Verify the recipe, protocol, artifact bounds, and artifact digest.
    ///
    /// # Errors
    ///
    /// Returns an error if any validation fails, including an artifact hash mismatch.
    pub fn validate(&self) -> Result<()> {
        self.recipe.validate()?;
        if self.protocol != PROTOCOL
            || self.artifact.is_empty()
            || self.artifact.len() > MAX_ARTIFACT
            || sha256(&self.artifact) != self.artifact_sha256
        {
            return Err(error("invalid protocol, artifact size or artifact SHA-256"));
        }
        Ok(())
    }

    /// Hash the complete serialized job to bind a report to its exact inputs.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn digest(&self) -> Result<String> {
        Ok(sha256(&serde_json::to_vec(self)?))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pass,
    Fail,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    pub bytes: Vec<u8>,
    pub observed_size: u64,
    pub truncated: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub protocol: u32,
    pub job_id: String,
    pub host_id: String,
    pub job_sha256: String,
    pub agent_sha256: String,
    pub agent_version: String,
    pub agent_os: String,
    pub agent_arch: String,
    pub baseline_declared: Baseline,
    pub recorded_unix_ms: u64,
    pub started_unix_ms: Option<u64>,
    pub elapsed_ms: u64,
    pub launch_backend: String,
    pub invocation: Vec<String>,
    pub artifact_staged: bool,
    pub artifact_size_before: u64,
    pub artifact_sha256_before: String,
    pub artifact_size_after: Option<u64>,
    pub artifact_sha256_after: Option<String>,
    pub exit_code: Option<i32>,
    pub termination: Option<String>,
    pub timed_out: bool,
    pub output_limit_exceeded: bool,
    pub stdout: Capture,
    pub stderr: Capture,
    pub outcome: Outcome,
    pub detail: String,
    pub qualifications: Vec<String>,
}

impl Report {
    /// Verify the report's job binding and the consistency of a passing outcome.
    ///
    /// # Errors
    ///
    /// Returns an error for mismatched identifiers/digests, oversized output,
    /// contradictory passing evidence, or failed job serialization.
    pub fn validate_for(&self, job: &Job) -> Result<()> {
        if self.protocol != PROTOCOL
            || self.job_id != job.recipe.id
            || self.host_id != job.recipe.host_id
            || self.job_sha256 != job.digest()?
            || self.stdout.bytes.len() > MAX_OUTPUT
            || self.stderr.bytes.len() > MAX_OUTPUT
        {
            return Err(error(
                "report does not match its assigned job or size limits",
            ));
        }
        if self.outcome == Outcome::Pass
            && (!self.artifact_staged
                || self.started_unix_ms.is_none()
                || self.artifact_sha256_before != job.artifact_sha256
                || self.artifact_size_before != job.artifact.len() as u64
                || self.timed_out
                || self.output_limit_exceeded
                || self.exit_code != Some(job.recipe.expected_exit)
                || self.stdout.bytes != job.recipe.expected_stdout
                || self.stderr.bytes != job.recipe.expected_stderr
                || self.stdout.truncated
                || self.stderr.truncated
                || self.artifact_sha256_after.as_ref() != Some(&job.artifact_sha256)
                || self.artifact_size_after != Some(job.artifact.len() as u64))
        {
            return Err(error("passing report contradicts observations"));
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Poll {
        protocol: u32,
        host_id: String,
        token: String,
    },
    Complete {
        protocol: u32,
        host_id: String,
        token: String,
        report: Box<Report>,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Response {
    Job { job: Box<Job> },
    Idle,
    Accepted,
    Rejected,
}
