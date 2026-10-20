#!/usr/bin/env python3
"""Local TLS/control smoke test. Keeps raw evidence under ignored artifacts/.

Requires Python 3, OpenSSL and the pinned Rust toolchain on the developer host.
These are test-harness dependencies, not dependencies of deployed agents.
"""
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile
import time


ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    return subprocess.run(args, cwd=ROOT, check=True, **kwargs)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def main():
    run("cargo", "build", "--locked", "--offline", "-p", "polystride-lab")
    root = ROOT / "artifacts" / "lab-smoke"
    root.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="run-", dir=root))
    os.chmod(work, 0o700)
    binary = ROOT / "target" / "debug" / ("polystride-lab.exe" if os.name == "nt" else "polystride-lab")
    control = work / "native-control.exe"
    run("rustc", "--edition=2024", "-C", "debuginfo=0",
        str(ROOT / "examples" / "lab-control.rs"), "-o", str(control))
    with (work / "certificate-setup.log").open("wb") as log:
        run("openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes",
            "-keyout", str(work / "ca.key"), "-out", str(work / "ca.pem"),
            "-days", "2", "-subj", "/CN=Polystride local smoke CA",
            "-addext", "basicConstraints=critical,CA:TRUE", stdout=log, stderr=log)
        run("openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes",
            "-keyout", str(work / "server.key"), "-out", str(work / "server.csr"),
            "-subj", "/CN=localhost", stdout=log, stderr=log)
        (work / "server.ext").write_text(
            "subjectAltName=DNS:localhost\nbasicConstraints=critical,CA:FALSE\n"
            "keyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
        run("openssl", "x509", "-req", "-in", str(work / "server.csr"),
            "-CA", str(work / "ca.pem"), "-CAkey", str(work / "ca.key"),
            "-CAcreateserial", "-out", str(work / "server.pem"), "-days", "2",
            "-extfile", str(work / "server.ext"), stdout=log, stderr=log)
    token = secrets.token_hex(32)
    token_path = work / "agent.token"
    token_path.write_text(token + "\n")
    for path in [token_path, work / "ca.key", work / "server.key"]:
        os.chmod(path, 0o600)
    coordinator_path = work / "coordinator.json"
    write_json(coordinator_path, {
        "listen": "127.0.0.1:0", "certificate": str(work / "server.pem"),
        "private_key": str(work / "server.key"), "state_dir": str(work / "coordinator"),
        "hosts": {"local-control": hashlib.sha256(token.encode()).hexdigest()},
    })
    revision = run("git", "rev-parse", "HEAD", capture_output=True, text=True).stdout.strip()
    dirty = bool(run("git", "status", "--porcelain", capture_output=True, text=True).stdout.strip())
    recipe = {
        "id": "001-native-control", "host_id": "local-control",
        "source_revision": revision + ("+dirty" if dirty else ""),
        "args": ["echo", "hello λ 世界 & literal"], "env": {}, "timeout_ms": 2000,
        "expected_exit": 7,
        "expected_stdout": list("hello λ 世界 & literal\n".encode()),
        "expected_stderr": list(b"control stderr\n"),
    }
    recipe_path = work / "recipe.json"
    write_json(recipe_path, recipe)
    run(str(binary), "queue", str(coordinator_path), str(recipe_path), str(control))
    coordinator_log = (work / "coordinator.log").open("w")
    coordinator = subprocess.Popen([str(binary), "coordinator", str(coordinator_path)],
                                   cwd=ROOT, stdout=coordinator_log, stderr=coordinator_log)
    try:
        deadline = time.monotonic() + 10
        address = None
        while time.monotonic() < deadline:
            if coordinator.poll() is not None:
                raise RuntimeError("coordinator exited before becoming ready")
            lines = (work / "coordinator.log").read_text().splitlines()
            for line in lines:
                if line.startswith("coordinator listening on "):
                    address = line.removeprefix("coordinator listening on ")
            if address:
                break
            time.sleep(0.05)
        if not address:
            raise RuntimeError("coordinator did not become ready")
        import platform
        os_name = {"Darwin": "macos", "Linux": "linux", "Windows": "windows"}[platform.system()]
        arch = {"arm64": "aarch64", "aarch64": "aarch64",
                "x86_64": "x86_64", "AMD64": "x86_64"}[platform.machine()]
        config = {
            "coordinator": address, "server_name": "localhost",
            "ca_certificate": str(work / "ca.pem"), "host_id": "local-control",
            "token_file": str(token_path), "state_dir": str(work / "agent"),
            "poll_interval_ms": 100,
            "baseline": {"os": os_name, "os_version": platform.platform(),
                         "hardware_arch": arch, "machine_kind": "unknown",
                         "baseline_id": "local-developer-smoke",
                         "notes": "Developer machine; no clean-host or native-workload qualification"},
        }
        agent_path = work / "agent.json"
        write_json(agent_path, config)
        with (work / "agent.log").open("w") as log:
            # Failed security controls must fail before the queued executable runs.
            config["server_name"] = "incorrect.invalid"
            write_json(agent_path, config)
            bad_name = subprocess.run([str(binary), "agent", str(agent_path), "--once"],
                                      cwd=ROOT, stdout=log, stderr=log, timeout=30)
            assert bad_name.returncode != 0
            assert not (work / "agent" / recipe["id"]).exists()
            config["server_name"] = "localhost"
            wrong_token = work / "wrong.token"
            wrong_token.write_text(secrets.token_hex(32))
            os.chmod(wrong_token, 0o600)
            config["token_file"] = str(wrong_token)
            write_json(agent_path, config)
            bad_auth = subprocess.run([str(binary), "agent", str(agent_path), "--once"],
                                      cwd=ROOT, stdout=log, stderr=log, timeout=30)
            assert bad_auth.returncode != 0
            assert not (work / "agent" / recipe["id"]).exists()
            config["token_file"] = str(token_path)
            write_json(agent_path, config)
            run(str(binary), "agent", str(agent_path), "--once", stdout=log, stderr=log, timeout=30)
            result_path = work / "coordinator" / "results" / (recipe["id"] + ".json")
            report = json.loads(result_path.read_text())
            assert report["outcome"] == "pass", report["detail"]
            assert report["artifact_sha256_before"] == report["artifact_sha256_after"]
            # An actual executable that exceeds its deadline is a retained failure.
            recipe.update(id="002-timeout-control", args=["sleep"], timeout_ms=80,
                          expected_exit=0, expected_stdout=[], expected_stderr=[])
            write_json(recipe_path, recipe)
            run(str(binary), "queue", str(coordinator_path), str(recipe_path), str(control))
            run(str(binary), "agent", str(agent_path), "--once", stdout=log, stderr=log, timeout=30)
            timeout_report = json.loads((work / "coordinator/results/002-timeout-control.json").read_text())
            assert timeout_report["outcome"] == "fail" and timeout_report["timed_out"]
            # A malformed executable is preserved as a launch failure.
            invalid = work / "invalid.exe"
            invalid.write_bytes(bytes(128))
            recipe.update(id="003-invalid-control", args=[], timeout_ms=2000)
            write_json(recipe_path, recipe)
            run(str(binary), "queue", str(coordinator_path), str(recipe_path), str(invalid))
            run(str(binary), "agent", str(agent_path), "--once", stdout=log, stderr=log, timeout=30)
            invalid_report = json.loads((work / "coordinator/results/003-invalid-control.json").read_text())
            assert invalid_report["outcome"] == "fail"
            assert invalid_report["exit_code"] is None or invalid_report["exit_code"] != 0
            # A no-shebang sentinel detects a successful implicit shell fallback.
            sentinel = work / "shell-fallback-sentinel.exe"
            sentinel.write_bytes(b"printf 'fallback-observed\\n'\nexit 23\n")
            recipe.update(id="004-shell-fallback-sentinel")
            write_json(recipe_path, recipe)
            run(str(binary), "queue", str(coordinator_path), str(recipe_path), str(sentinel))
            run(str(binary), "agent", str(agent_path), "--once", stdout=log, stderr=log, timeout=30)
            fallback_report = json.loads((work / "coordinator/results/004-shell-fallback-sentinel.json").read_text())
            assert fallback_report["outcome"] == "fail"
            fallback_observed = (
                fallback_report["exit_code"] == 23
                and bytes(fallback_report["stdout"]["bytes"]) == b"fallback-observed\n")
        status = run(str(binary), "status", str(coordinator_path), capture_output=True, text=True)
        (work / "status.json").write_text(status.stdout)
        # Hash source files as well as binaries: HEAD alone does not identify a dirty tree.
        sources = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "rust-toolchain.toml",
                   ROOT / "examples/lab-control.rs", ROOT / "scripts/lab-smoke.py"]
        sources += list((ROOT / "crates/polystride-lab").rglob("*.rs"))
        sources += [ROOT / "crates/polystride-lab/Cargo.toml"]
        summary = {
            "source_revision": revision, "dirty_tree": dirty,
            "source_files_sha256": {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                                    for p in sorted(sources)},
            "agent_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "control_sha256": hashlib.sha256(control.read_bytes()).hexdigest(),
            "control_size": control.stat().st_size,
            "invalid_sha256": hashlib.sha256(invalid.read_bytes()).hexdigest(),
            "shell_fallback_sentinel_sha256": hashlib.sha256(sentinel.read_bytes()).hexdigest(),
            "tls_name_rejection": "pass", "token_rejection": "pass",
            "native_control_recipe": "pass", "timeout_recorded_as_failure": "pass",
            "invalid_executable_recorded_as_failure": "pass",
            "implicit_shell_fallback_observed": fallback_observed,
            "qualified_platform_acceptance": False,
        }
        write_json(work / "summary.json", summary)
        print(f"Local TLS smoke passed. Evidence: {work.relative_to(ROOT)}")
        print("Native workload qualification and cross-platform acceptance remain open.")
    finally:
        coordinator.terminate()
        try:
            coordinator.wait(timeout=5)
        except subprocess.TimeoutExpired:
            coordinator.kill()
            coordinator.wait()
        coordinator_log.close()


if __name__ == "__main__":
    main()
