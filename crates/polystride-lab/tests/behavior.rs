use polystride_lab::agent::{lock_directory, run_once};
use polystride_lab::model::{
    Baseline, Job, MAX_FRAME, MAX_OUTPUT, MachineKind, Outcome, PROTOCOL, Recipe, Request,
    Response, sha256,
};
use polystride_lab::store::Store;
use polystride_lab::{publish_json, transport};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tempfile::TempDir;

const TOKEN: &str = "only-a-test-token-never-use-this-for-enrollment";

fn directory() -> TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/lab-tests");
    fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root).unwrap()
}

fn control() -> &'static [u8] {
    static CONTROL: OnceLock<Vec<u8>> = OnceLock::new();
    CONTROL.get_or_init(|| {
        let directory = directory();
        let output = directory.path().join("control.exe");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/lab-control.rs");
        let status = Command::new("rustc")
            .args(["--edition=2024", "-C", "debuginfo=0"])
            .arg(source)
            .arg("-o")
            .arg(&output)
            .status()
            .unwrap();
        assert!(status.success());
        fs::read(output).unwrap()
    })
}

fn baseline() -> Baseline {
    Baseline {
        os: std::env::consts::OS.into(),
        os_version: "test fixture; not a qualified host".into(),
        hardware_arch: std::env::consts::ARCH.into(),
        machine_kind: MachineKind::Unknown,
        baseline_id: "behavioral-tests".into(),
        notes: "developer host".into(),
    }
}

fn job(id: &str, args: &[&str]) -> Job {
    Job::new(
        Recipe {
            id: id.into(),
            host_id: "host-a".into(),
            source_revision: "test-fixture".into(),
            args: args.iter().map(ToString::to_string).collect(),
            env: BTreeMap::new(),
            timeout_ms: 2_000,
            expected_exit: 0,
            expected_stdout: Vec::new(),
            expected_stderr: Vec::new(),
        },
        control().to_vec(),
    )
    .unwrap()
}

fn store(root: &Path) -> Store {
    Store::open(
        root,
        BTreeMap::from([
            ("host-a".into(), sha256(TOKEN.as_bytes())),
            (
                "host-b".into(),
                sha256(b"another-test-token-not-an-enrollment-secret"),
            ),
        ]),
    )
    .unwrap()
}

fn poll(host: &str, token: &str) -> Request {
    Request::Poll {
        protocol: PROTOCOL,
        host_id: host.into(),
        token: token.into(),
    }
}

#[test]
fn captures_unicode_arguments_nonzero_exit_and_both_streams() {
    let directory = directory();
    let mut job = job("unicode", &["echo", "hello λ 世界 & \"literal\""]);
    job.recipe.expected_exit = 7;
    job.recipe.expected_stdout = "hello λ 世界 & \"literal\"\n".as_bytes().to_vec();
    job.recipe.expected_stderr = b"control stderr\n".to_vec();
    let report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert_eq!(report.outcome, Outcome::Pass);
    assert_eq!(report.exit_code, Some(7));
    assert_eq!(
        report.artifact_sha256_after,
        Some(job.artifact_sha256.clone())
    );
    report.validate_for(&job).unwrap();
}

#[test]
fn preserves_non_utf8_output() {
    let directory = directory();
    let mut job = job("bytes", &["bytes"]);
    job.recipe.expected_stdout = vec![0, 255, 10];
    let report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert_eq!(report.outcome, Outcome::Pass);
    assert_eq!(report.stdout.bytes, vec![0, 255, 10]);
}

#[test]
fn uses_isolated_cwd_and_explicit_environment() {
    let directory = directory();
    let mut job = job("environment", &["environment"]);
    job.recipe
        .env
        .insert("LAB_VALUE".into(), "explicit-value".into());
    job.recipe.expected_stdout = b"explicit-value\n".to_vec();
    let report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert_eq!(report.outcome, Outcome::Pass);
    assert!(
        !directory
            .path()
            .join("environment/work/control-file")
            .exists()
    );
}

#[test]
fn enforces_timeout_and_preserves_a_failed_result() {
    let directory = directory();
    let mut job = job("timeout", &["sleep"]);
    job.recipe.timeout_ms = 80;
    let report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert!(report.timed_out);
    assert_eq!(report.outcome, Outcome::Fail);
    assert!(report.elapsed_ms < 5_000);
}

#[test]
fn bounds_returned_output_and_fails_output_overflow() {
    let directory = directory();
    let job = job("flood", &["flood"]);
    let report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert!(report.output_limit_exceeded);
    assert!(report.stdout.truncated);
    assert!(report.stdout.bytes.len() <= MAX_OUTPUT);
    assert_eq!(report.outcome, Outcome::Fail);
}

#[test]
fn rejects_corruption_before_execution() {
    let directory = directory();
    let mut job = job("corrupt", &["sleep"]);
    job.artifact[0] ^= 255;
    assert!(run_once(&job, &baseline(), "agent-hash", directory.path()).is_err());
    assert!(!directory.path().join("corrupt/receipt.json").exists());
}

#[test]
fn preserves_invalid_executable_failure_without_assuming_no_shell_fallback() {
    let directory = directory();
    let mut job = job("invalid-executable", &[]);
    job.artifact = vec![0; 128];
    job.artifact_sha256 = sha256(&job.artifact);
    let report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert_eq!(report.outcome, Outcome::Fail);
    assert!(report.exit_code.is_none() || report.exit_code.is_some_and(|code| code != 0));
    assert_eq!(report.artifact_sha256_after, Some(job.artifact_sha256));
}

#[cfg(unix)]
#[test]
fn detects_executable_replacement() {
    let directory = directory();
    let job = job("mutation", &["mutate"]);
    let report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert_eq!(report.exit_code, Some(0));
    assert_eq!(report.outcome, Outcome::Fail);
    assert_eq!(report.artifact_sha256_after, Some(sha256(b"changed")));
}

#[test]
fn redelivery_returns_evidence_without_executing_twice() {
    let directory = directory();
    let marker = directory.path().join("marker");
    let job = job("replay", &["mark", marker.to_str().unwrap()]);
    let first = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    let second = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert_eq!(first, second);
    assert_eq!(fs::read_to_string(marker).unwrap(), "executed\n");
    let mut changed = job.clone();
    changed.recipe.args = vec!["sleep".into()];
    assert!(run_once(&changed, &baseline(), "agent-hash", directory.path()).is_err());
}

#[test]
fn crash_receipt_is_blocked_not_reexecuted() {
    let directory = directory();
    let marker = directory.path().join("marker");
    let job = job("interrupted", &["mark", marker.to_str().unwrap()]);
    let attempt = directory.path().join("interrupted");
    fs::create_dir(&attempt).unwrap();
    publish_json(&attempt.join("receipt.json"), &job.digest().unwrap()).unwrap();
    let report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert_eq!(report.outcome, Outcome::Blocked);
    assert!(!marker.exists());
}

#[test]
fn coordinator_scopes_auth_persists_assignments_and_accepts_idempotent_reports() {
    let directory = directory();
    let job = job("queue", &["bytes"]);
    let mut job = job;
    job.recipe.expected_stdout = vec![0, 255, 10];
    let coordinator = store(directory.path());
    coordinator.queue(&job).unwrap();
    assert!(coordinator.queue(&job).is_err());
    assert!(matches!(
        coordinator.handle(poll("host-a", "wrong-token")).unwrap(),
        Response::Rejected
    ));
    assert!(matches!(
        coordinator
            .handle(poll(
                "host-b",
                "another-test-token-not-an-enrollment-secret"
            ))
            .unwrap(),
        Response::Idle
    ));
    assert!(matches!(
        coordinator.handle(poll("host-a", TOKEN)).unwrap(),
        Response::Job { .. }
    ));
    drop(coordinator);
    let coordinator = store(directory.path());
    assert!(coordinator.status().unwrap()[0].1.starts_with("assigned"));
    assert!(matches!(
        coordinator.handle(poll("host-a", TOKEN)).unwrap(),
        Response::Job { .. }
    ));
    let agent_directory = directory.path().join("agent");
    let report = run_once(&job, &baseline(), "agent-hash", &agent_directory).unwrap();
    let complete = |report| Request::Complete {
        protocol: PROTOCOL,
        host_id: "host-a".into(),
        token: TOKEN.into(),
        report: Box::new(report),
    };
    assert!(matches!(
        coordinator.handle(complete(report.clone())).unwrap(),
        Response::Accepted
    ));
    assert!(matches!(
        coordinator.handle(complete(report.clone())).unwrap(),
        Response::Accepted
    ));
    let mut conflicting = report;
    conflicting.detail = "different evidence".into();
    assert!(matches!(
        coordinator.handle(complete(conflicting)).unwrap(),
        Response::Rejected
    ));
    assert!(matches!(
        coordinator.handle(poll("host-a", TOKEN)).unwrap(),
        Response::Idle
    ));
    assert_eq!(coordinator.status().unwrap()[0].1, "pass");
}

#[test]
fn passing_reports_must_match_observations() {
    let directory = directory();
    let job = job("mismatch", &["bytes"]);
    let mut report = run_once(&job, &baseline(), "agent-hash", directory.path()).unwrap();
    assert_eq!(report.outcome, Outcome::Fail);
    report.outcome = Outcome::Pass;
    assert!(report.validate_for(&job).is_err());
}

#[test]
fn rejects_path_traversal_and_unknown_protocol() {
    let mut job = job("valid", &["bytes"]);
    job.recipe.id = "../escape".into();
    assert!(job.validate().is_err());
    job.recipe.id = "valid".into();
    job.protocol = 999;
    assert!(job.validate().is_err());
}

#[test]
fn framed_protocol_handles_fragmentation_and_rejects_oversize() {
    struct Fragmented(std::io::Cursor<Vec<u8>>);
    impl std::io::Read for Fragmented {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            let len = bytes.len().min(3);
            self.0.read(&mut bytes[..len])
        }
    }
    let request = poll("host-a", TOKEN);
    let mut wire = Vec::new();
    transport::write_frame(&mut wire, &request).unwrap();
    let decoded: Request =
        transport::read_frame(&mut Fragmented(std::io::Cursor::new(wire))).unwrap();
    assert!(matches!(decoded, Request::Poll { host_id, .. } if host_id == "host-a"));
    let oversized = u32::try_from(MAX_FRAME + 1).unwrap().to_be_bytes();
    assert!(transport::read_frame::<Request>(&mut &oversized[..]).is_err());
    assert!(transport::read_frame::<Request>(&mut &[0, 0, 0, 5, b'{'][..]).is_err());
}

#[test]
fn state_lock_prevents_two_agents_from_running_the_same_job() {
    let directory = directory();
    let lock = lock_directory(directory.path()).unwrap();
    assert!(lock_directory(directory.path()).is_err());
    drop(lock);
    assert!(lock_directory(directory.path()).is_ok());
}

#[test]
fn standard_sha256_vector() {
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn no_partial_publication_or_overwrite() {
    let directory = directory();
    let path: PathBuf = directory.path().join("immutable.json");
    publish_json(&path, &vec![1, 2, 3]).unwrap();
    assert!(publish_json(&path, &vec![4, 5, 6]).is_err());
    assert_eq!(
        polystride_lab::read_json::<Vec<i32>>(&path).unwrap(),
        [1, 2, 3]
    );
}
