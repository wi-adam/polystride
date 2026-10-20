use crate::model::{
    Baseline, Capture, Job, MAX_ARTIFACT, MAX_OUTPUT, Outcome, PROTOCOL, Report, sha256,
};
use crate::{Result, read_bounded};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn empty_capture() -> Capture {
    Capture {
        bytes: Vec::new(),
        observed_size: 0,
        truncated: false,
    }
}

fn unix_millis() -> Result<u64> {
    Ok(u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

/// Construct a blocked report without claiming that staging or execution occurred.
///
/// # Errors
///
/// Returns an error if job serialization fails or the clock cannot be represented
/// as nonnegative Unix milliseconds.
pub fn initial_report(job: &Job, baseline: &Baseline, agent_sha256: &str) -> Result<Report> {
    Ok(Report {
        protocol: PROTOCOL,
        job_id: job.recipe.id.clone(),
        host_id: job.recipe.host_id.clone(),
        job_sha256: job.digest()?,
        agent_sha256: agent_sha256.into(),
        agent_version: env!("CARGO_PKG_VERSION").into(),
        agent_os: std::env::consts::OS.into(),
        agent_arch: std::env::consts::ARCH.into(),
        baseline_declared: baseline.clone(),
        recorded_unix_ms: unix_millis()?,
        started_unix_ms: None,
        elapsed_ms: 0,
        launch_backend: "rust_std_process_command".into(),
        invocation: std::iter::once("payload.exe".into()).chain(job.recipe.args.clone()).collect(),
        artifact_staged: false,
        artifact_size_before: job.artifact.len() as u64,
        artifact_sha256_before: job.artifact_sha256.clone(),
        artifact_size_after: None,
        artifact_sha256_after: None,
        exit_code: None,
        termination: None,
        timed_out: false,
        output_limit_exceeded: false,
        stdout: empty_capture(),
        stderr: empty_capture(),
        outcome: Outcome::Blocked,
        detail: "execution has not completed".into(),
        qualifications: vec![
            "Only the recipe's behavior is assessed; a pass is not platform acceptance.".into(),
            "Workload CPU translation/native execution: unknown; agent target is not workload evidence.".into(),
            "Host baseline is operator-declared; cleanliness, security state and CPU features are not audited.".into(),
            "Rust Command backend is not yet certified as a strict no-shell-fallback launcher on every host.".into(),
            "Desktop launch, quarantine/download, signing, offline execution and helper-cache state: not tested.".into(),
            "Environment cleared, then recipe entries applied; HOME/TMP are per-run. Windows SystemRoot is inherited.".into(),
            "Output captured to files and sampled every 10 ms; storage use is not a hard quota.".into(),
            "Only the direct child is supervised; descendant containment, disk/network tracing and cleanup are not implemented.".into(),
            "Payload bytes are staged as payload.exe; browser metadata and original permission bits are not transferred.".into(),
        ],
    })
}

fn capture(path: &Path) -> Result<Capture> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_OUTPUT as u64)
        .read_to_end(&mut bytes)?;
    let size = file.metadata()?.len();
    Ok(Capture {
        truncated: size > bytes.len() as u64,
        bytes,
        observed_size: size,
    })
}

fn supervise(
    child: &mut Child,
    stdout_path: &Path,
    stderr_path: &Path,
    timeout: Duration,
    start: Instant,
    report: &mut Report,
) -> Result<()> {
    loop {
        let status = child.try_wait()?;
        report.output_limit_exceeded = fs::metadata(stdout_path)?.len() > MAX_OUTPUT as u64
            || fs::metadata(stderr_path)?.len() > MAX_OUTPUT as u64;
        if let Some(status) = status {
            report.exit_code = status.code();
            report.termination = Some(status.to_string());
            return Ok(());
        }
        report.timed_out = start.elapsed() >= timeout;
        if report.timed_out || report.output_limit_exceeded {
            // kill() can race with a natural exit. Always reap if it has exited.
            if let Err(err) = child.kill()
                && child.try_wait()?.is_none()
            {
                return Err(err.into());
            }
            let status = child.wait()?;
            report.exit_code = status.code();
            report.termination = Some(status.to_string());
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Runs trusted single-process controls. This is deliberately not a sandbox.
///
/// # Errors
///
/// Returns an error for invalid jobs, staging/capture I/O failures, clock
/// conversion failures, or process-supervision errors. Spawn errors and
/// mismatched observations are retained in failed reports.
pub fn execute(
    job: &Job,
    baseline: &Baseline,
    agent_hash: &str,
    directory: &Path,
) -> Result<Report> {
    job.validate()?;
    let mut report = initial_report(job, baseline, agent_hash)?;
    let directory = directory.canonicalize()?;
    let payload = directory.join("payload.exe");
    let work = directory.join("work");
    let home = directory.join("home");
    let tmp = directory.join("tmp");
    for path in [&work, &home, &tmp] {
        fs::create_dir(path)?;
    }
    fs::write(&payload, &job.artifact)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&payload, fs::Permissions::from_mode(0o500))?;
        report
            .qualifications
            .push("Staged payload permissions explicitly set to 0500.".into());
    }
    // Verify the bytes actually on disk, not only the incoming job.
    let staged = read_bounded(&payload, MAX_ARTIFACT)?;
    report.artifact_sha256_before = sha256(&staged);
    report.artifact_size_before = staged.len() as u64;
    report.artifact_staged = true;
    if report.artifact_sha256_before != job.artifact_sha256 {
        report.detail = "staged artifact hash mismatch; execution refused".into();
        return Ok(report);
    }
    let stdout_path = directory.join("stdout.bin");
    let stderr_path = directory.join("stderr.bin");
    let stdout = File::create(&stdout_path)?;
    let stderr = File::create(&stderr_path)?;
    let mut command = Command::new(&payload);
    command
        .args(&job.recipe.args)
        .current_dir(&work)
        .env_clear()
        .envs(&job.recipe.env)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("TMPDIR", &tmp)
        .env("TMP", &tmp)
        .env("TEMP", &tmp)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr);
    #[cfg(windows)]
    if let Some(root) = std::env::var_os("SystemRoot") {
        command.env("SystemRoot", root);
    }
    let start = Instant::now();
    report.started_unix_ms = Some(unix_millis()?);
    let mut spawn_failed = false;
    match command.spawn() {
        Err(err) => {
            spawn_failed = true;
            report.detail = format!("spawn failed: {err}");
        }
        Ok(mut child) => supervise(
            &mut child,
            &stdout_path,
            &stderr_path,
            Duration::from_millis(job.recipe.timeout_ms),
            start,
            &mut report,
        )?,
    }
    report.elapsed_ms = u64::try_from(start.elapsed().as_millis())?;
    report.stdout = capture(&stdout_path)?;
    report.stderr = capture(&stderr_path)?;
    report.output_limit_exceeded |= report.stdout.truncated || report.stderr.truncated;
    if let Ok(after) = read_bounded(&payload, MAX_ARTIFACT) {
        report.artifact_size_after = Some(after.len() as u64);
        report.artifact_sha256_after = Some(sha256(&after));
    }
    report.outcome = if !spawn_failed
        && !report.timed_out
        && !report.output_limit_exceeded
        && report.exit_code == Some(job.recipe.expected_exit)
        && report.stdout.bytes == job.recipe.expected_stdout
        && report.stderr.bytes == job.recipe.expected_stderr
        && report.artifact_sha256_after.as_ref() == Some(&job.artifact_sha256)
        && report.artifact_size_after == Some(job.artifact.len() as u64)
    {
        Outcome::Pass
    } else {
        Outcome::Fail
    };
    if !spawn_failed {
        report.detail = if report.outcome == Outcome::Pass {
            "observed exit, output and unchanged artifact match the recipe"
        } else {
            "exit, output, deadline, output limit or artifact integrity did not match the recipe"
        }
        .into();
    }
    Ok(report)
}
