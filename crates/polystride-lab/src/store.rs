use crate::model::{Job, PROTOCOL, Report, Request, Response, sha256, validate_id};
use crate::{Result, error, publish_json, read_json};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinatorConfig {
    pub listen: String,
    pub certificate: PathBuf,
    pub private_key: PathBuf,
    pub state_dir: PathBuf,
    /// SHA-256 of a separate random token for each host, never the raw token.
    pub hosts: BTreeMap<String, String>,
}

pub struct Store {
    root: PathBuf,
    hosts: BTreeMap<String, String>,
}

impl Store {
    /// Open or initialize a store with the enrolled host credential hashes.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid host IDs/hashes or directory creation failures.
    pub fn open(root: &Path, hosts: BTreeMap<String, String>) -> Result<Self> {
        for (id, token_hash) in &hosts {
            validate_id(id)?;
            if token_hash.len() != 64 || !token_hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(error("host credential must be a SHA-256 hex digest"));
            }
        }
        for directory in ["jobs", "assigned", "results"] {
            fs::create_dir_all(root.join(directory))?;
        }
        Ok(Self {
            root: root.into(),
            hosts,
        })
    }

    /// Publish an immutable job for an enrolled host.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid jobs, unenrolled hosts, existing IDs, or I/O failures.
    pub fn queue(&self, job: &Job) -> Result<()> {
        job.validate()?;
        if !self.hosts.contains_key(&job.recipe.host_id) {
            return Err(error("job host is not enrolled"));
        }
        publish_json(
            &self
                .root
                .join("jobs")
                .join(format!("{}.json", job.recipe.id)),
            job,
        )
    }

    fn job_path(&self, id: &str) -> PathBuf {
        self.root.join("jobs").join(format!("{id}.json"))
    }

    fn result_path(&self, id: &str) -> PathBuf {
        self.root.join("results").join(format!("{id}.json"))
    }

    fn assignment_path(&self, id: &str) -> PathBuf {
        self.root.join("assigned").join(format!("{id}.json"))
    }

    /// Read the queued, assigned, or completed status of each stored job.
    ///
    /// # Errors
    ///
    /// Returns an error if job or report enumeration, reading, or decoding fails.
    pub fn status(&self) -> Result<Vec<(String, String)>> {
        self.jobs()?
            .iter()
            .map(|job| {
                let id = &job.recipe.id;
                let status = if self.result_path(id).exists() {
                    let report: Report = read_json(&self.result_path(id))?;
                    format!("{:?}", report.outcome).to_lowercase()
                } else if self.assignment_path(id).exists() {
                    "assigned; awaiting evidence (not a test result)".into()
                } else {
                    "queued".into()
                };
                Ok((id.clone(), status))
            })
            .collect()
    }

    fn jobs(&self) -> Result<Vec<Job>> {
        let mut paths = fs::read_dir(self.root.join("jobs"))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        paths.retain(|p| p.extension().is_some_and(|e| e == "json"));
        paths.sort();
        paths.into_iter().map(|path| read_json(&path)).collect()
    }

    /// Authenticate and process one poll or report request.
    ///
    /// # Errors
    ///
    /// Returns an error if persisted state cannot be read, validated, or written.
    /// Invalid credentials and inconsistent submitted reports produce a rejection.
    pub fn handle(&self, request: Request) -> Result<Response> {
        let (protocol, host_id, token) = match &request {
            Request::Poll {
                protocol,
                host_id,
                token,
            }
            | Request::Complete {
                protocol,
                host_id,
                token,
                ..
            } => (*protocol, host_id, token),
        };
        if protocol != PROTOCOL
            || token.len() < 32
            || self.hosts.get(host_id) != Some(&sha256(token.as_bytes()))
        {
            return Ok(Response::Rejected);
        }
        match request {
            Request::Poll { host_id, .. } => {
                // Finish an existing assignment before offering any newer job.
                let jobs = self.jobs()?;
                let unfinished = |job: &&Job| {
                    job.recipe.host_id == host_id && !self.result_path(&job.recipe.id).exists()
                };
                let next = jobs
                    .iter()
                    .filter(unfinished)
                    .find(|job| self.assignment_path(&job.recipe.id).exists())
                    .or_else(|| jobs.iter().find(unfinished));
                if let Some(job) = next {
                    job.validate()?;
                    let path = self.assignment_path(&job.recipe.id);
                    if !path.exists() {
                        publish_json(&path, &job.digest()?)?;
                    }
                    return Ok(Response::Job {
                        job: Box::new(job.clone()),
                    });
                }
                Ok(Response::Idle)
            }
            Request::Complete {
                host_id, report, ..
            } => {
                if report.host_id != host_id || validate_id(&report.job_id).is_err() {
                    return Ok(Response::Rejected);
                }
                let path = self.job_path(&report.job_id);
                if !path.exists() || !self.assignment_path(&report.job_id).exists() {
                    return Ok(Response::Rejected);
                }
                let job: Job = read_json(&path)?;
                if job.recipe.host_id != host_id || report.validate_for(&job).is_err() {
                    return Ok(Response::Rejected);
                }
                let result_path = self.result_path(&report.job_id);
                if result_path.exists() {
                    let previous: Report = read_json(&result_path)?;
                    return Ok(if previous == *report {
                        Response::Accepted
                    } else {
                        Response::Rejected
                    });
                }
                publish_json(&result_path, &report)?;
                Ok(Response::Accepted)
            }
        }
    }
}
