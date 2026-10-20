use crate::model::{
    Baseline, Job, MAX_ARTIFACT, Outcome, PROTOCOL, Report, Request, Response, sha256, validate_id,
};
use crate::{Result, error, publish_json, read_bounded, read_json, runner, transport};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    pub coordinator: String,
    pub server_name: String,
    pub ca_certificate: PathBuf,
    pub host_id: String,
    pub token_file: PathBuf,
    pub state_dir: PathBuf,
    pub poll_interval_ms: u64,
    pub baseline: Baseline,
}

/// Hold an exclusive OS lock for a state directory until the returned file drops.
///
/// # Errors
///
/// Returns an error if directory/file creation or exclusive locking fails.
pub fn lock_directory(directory: &Path) -> Result<File> {
    fs::create_dir_all(directory)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join(".lock"))?;
    lock.try_lock()
        .map_err(|_| error("state directory is already locked"))?;
    Ok(lock)
}

/// The receipt is durable before spawn. Repeated deliveries never re-execute an
/// ambiguous attempt; recovery emits blocked evidence and requires a new job ID.
///
/// # Errors
///
/// Returns an error for an invalid job, conflicting receipt/report, or failed
/// state I/O or report construction. Execution errors become blocked reports.
pub fn run_once(job: &Job, baseline: &Baseline, agent_hash: &str, root: &Path) -> Result<Report> {
    job.validate()?;
    let directory = root.join(&job.recipe.id);
    fs::create_dir_all(&directory)?;
    let receipt = directory.join("receipt.json");
    let result = directory.join("report.json");
    if receipt.exists() {
        let prior_digest: String = read_json(&receipt)?;
        if prior_digest != job.digest()? {
            return Err(error("job ID reused for different content"));
        }
        if result.exists() {
            let report: Report = read_json(&result)?;
            report.validate_for(job)?;
            return Ok(report);
        }
        let mut report = runner::initial_report(job, baseline, agent_hash)?;
        report.detail = "agent recovered an interrupted attempt; execution state unknown; not automatically retried".into();
        publish_json(&result, &report)?;
        return Ok(report);
    }
    publish_json(&receipt, &job.digest()?)?;
    publish_json(&directory.join("job.json"), job)?;
    let report = match runner::execute(job, baseline, agent_hash, &directory) {
        Ok(report) => report,
        Err(err) => {
            let mut report = runner::initial_report(job, baseline, agent_hash)?;
            report.detail = format!("runner error; execution state may be incomplete: {err}");
            report
        }
    };
    publish_json(&result, &report)?;
    Ok(report)
}

/// Poll for jobs and deliver retained reports, optionally stopping after one poll.
///
/// # Errors
///
/// Returns an error if configuration, credentials, TLS setup, executable hashing,
/// or the state lock fails. In one-shot mode, also returns poll/execution/report
/// errors; resident mode logs and retries those errors.
pub fn run(config: &AgentConfig, once: bool) -> Result<()> {
    validate_id(&config.host_id)?;
    config.baseline.validate()?;
    if config.baseline.os != std::env::consts::OS
        || !(100..=60_000).contains(&config.poll_interval_ms)
    {
        return Err(error(
            "baseline OS does not match agent, or invalid poll interval",
        ));
    }
    let _lock = lock_directory(&config.state_dir)?;
    let tls = transport::client_config(&config.ca_certificate)?;
    let token = String::from_utf8(read_bounded(&config.token_file, 1024)?)?
        .trim()
        .to_owned();
    if token.len() < 32 {
        return Err(error(
            "token must contain at least 32 characters of random material",
        ));
    }
    let agent_hash = sha256(&read_bounded(&std::env::current_exe()?, MAX_ARTIFACT * 8)?);
    loop {
        let poll = Request::Poll {
            protocol: PROTOCOL,
            host_id: config.host_id.clone(),
            token: token.clone(),
        };
        let attempt = (|| -> Result<()> {
            match transport::exchange(&config.coordinator, &config.server_name, tls.clone(), &poll)?
            {
                Response::Idle => Ok(()),
                Response::Job { job } => {
                    if job.recipe.host_id != config.host_id {
                        return Err(error("coordinator returned another host's job"));
                    }
                    let report = run_once(&job, &config.baseline, &agent_hash, &config.state_dir)?;
                    let blocked = report.outcome == Outcome::Blocked;
                    let request = Request::Complete {
                        protocol: PROTOCOL,
                        host_id: config.host_id.clone(),
                        token: token.clone(),
                        report: Box::new(report),
                    };
                    if !matches!(
                        transport::exchange(
                            &config.coordinator,
                            &config.server_name,
                            tls.clone(),
                            &request,
                        )?,
                        Response::Accepted
                    ) {
                        return Err(error("report was not acknowledged"));
                    }
                    eprintln!(
                        "job {}: evidence acknowledged{}",
                        job.recipe.id,
                        if blocked { " (blocked)" } else { "" }
                    );
                    Ok(())
                }
                Response::Rejected => {
                    Err(error("coordinator rejected agent credentials or protocol"))
                }
                Response::Accepted => Err(error("unexpected coordinator response")),
            }
        })();
        if once {
            return attempt;
        }
        if attempt.is_err() {
            eprintln!("poll/report failed; local receipts and evidence retained; retrying");
        }
        std::thread::sleep(Duration::from_millis(config.poll_interval_ms));
    }
}
