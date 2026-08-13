"""Helpers to build and quick-run the three topology-proximity-comm samples."""

from __future__ import annotations

import os
import shutil
import signal
import subprocess
import tempfile
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


REPO_ROOT = Path(__file__).resolve().parents[1]
ROS_DISTRO = os.environ.get("ROS_DISTRO", "jazzy")
ROS_SETUP = Path(f"/opt/ros/{ROS_DISTRO}/setup.bash")
ZENOH_COMPOSE = REPO_ROOT / "infra" / "docker-compose.yml"

SAMPLES = {
    "01-traditional-dds": REPO_ROOT / "samples/01-traditional-dds/cpp",
    "02-rmw-zenoh": REPO_ROOT / "samples/02-rmw-zenoh/cpp",
    "03-dds-zenoh-bridge": REPO_ROOT / "samples/03-dds-zenoh-bridge/cpp",
}

SAMPLE3_RUST = REPO_ROOT / "samples/03-dds-zenoh-bridge/rust"
SAMPLE3_BRIDGE_CFG = (
    REPO_ROOT / "samples/03-dds-zenoh-bridge/configs/zenoh_bridge-as-pub-client.json5"
)
SAMPLE2_ZENOH_CFG = REPO_ROOT / "samples/02-rmw-zenoh/configs/zenoh-client.json5"

SAMPLE1_DIR = REPO_ROOT / "samples/01-traditional-dds"
SAMPLE1_COMPOSE = SAMPLE1_DIR / "docker-compose.yml"


@dataclass
class ManagedProcess:
    name: str
    popen: subprocess.Popen[str]
    log_path: Path

    def stop(self) -> None:
        if self.popen.poll() is not None:
            return
        self.popen.send_signal(signal.SIGINT)
        try:
            self.popen.wait(timeout=3)
        except subprocess.TimeoutExpired:
            self.popen.kill()
            self.popen.wait(timeout=2)


@dataclass
class DemoSession:
    processes: list[ManagedProcess] = field(default_factory=list)

    def stop_all(self) -> None:
        for proc in reversed(self.processes):
            proc.stop()
        self.processes.clear()

    def tail_logs(self, lines: int = 20) -> str:
        chunks: list[str] = []
        for proc in self.processes:
            chunks.append(f"=== {proc.name} (last {lines} lines) ===")
            if proc.log_path.is_file():
                text = proc.log_path.read_text(errors="replace").splitlines()
                chunks.extend(text[-lines:] if text else ["(no output yet)"])
            else:
                chunks.append("(log missing)")
        return "\n".join(chunks)


def _which(name: str) -> str | None:
    return shutil.which(name)


def check_prerequisites() -> dict[str, bool | str | None]:
    """Return a quick readiness map for notebook display."""
    return {
        "repo_root": str(REPO_ROOT),
        "ros_setup": str(ROS_SETUP),
        "ros_setup_exists": ROS_SETUP.is_file(),
        "ros2": _which("ros2"),
        "colcon": _which("colcon"),
        "cargo": _which("cargo"),
        "docker": _which("docker"),
        "docker_compose": _which("docker-compose") or (
            "docker compose" if _which("docker") else None
        ),
        "sample1_compose": str(SAMPLE1_COMPOSE),
        "zenoh_bridge": _which("zenoh-bridge-ros2dds"),
        "rmw_zenoh_cpp": _which("rmw_zenoh_cpp") or "via RMW_IMPLEMENTATION",
    }


def run_shell(
    cmd: str,
    *,
    cwd: Path | None = None,
    check: bool = True,
    timeout: float | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "-lc", cmd],
        cwd=cwd or REPO_ROOT,
        text=True,
        capture_output=True,
        check=check,
        timeout=timeout,
    )


def ros_env_prefix(install_dir: Path) -> str:
    install_setup = install_dir / "install" / "setup.bash"
    if not ROS_SETUP.is_file():
        raise FileNotFoundError(f"ROS setup not found: {ROS_SETUP}")
    if not install_setup.is_file():
        raise FileNotFoundError(
            f"Sample not built — run build_sample() first: {install_setup}"
        )
    return f"source {ROS_SETUP} && source {install_setup} && "


def build_sample(sample_key: str) -> str:
    cpp_dir = SAMPLES[sample_key]
    result = run_shell(
        f"source {ROS_SETUP} && colcon build",
        cwd=cpp_dir,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return result.stdout


def build_sample3_rust() -> str:
    result = run_shell(
        "cargo build --bins",
        cwd=SAMPLE3_RUST,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return result.stdout


def build_all() -> list[str]:
    outputs = [build_sample(key) for key in SAMPLES]
    outputs.append(build_sample3_rust())
    return outputs


def zenohd_up() -> str:
    if not ZENOH_COMPOSE.is_file():
        raise FileNotFoundError(ZENOH_COMPOSE)
    result = run_shell(f"docker compose -f {ZENOH_COMPOSE} up -d", check=False)
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return (result.stdout or "") + (result.stderr or "")


def zenohd_down() -> str:
    result = run_shell(f"docker compose -f {ZENOH_COMPOSE} down", check=False)
    return (result.stdout or "") + (result.stderr or "")


def _start_logged(name: str, bash_cmd: str, *, cwd: Path | None = None) -> ManagedProcess:
    log_fd, log_path_str = tempfile.mkstemp(prefix=f"demo-{name}-", suffix=".log")
    os.close(log_fd)
    log_path = Path(log_path_str)
    with open(log_path, "ab") as log_file:
        popen = subprocess.Popen(
            ["bash", "-lc", bash_cmd],
            cwd=cwd or REPO_ROOT,
            stdout=log_file,
            stderr=subprocess.STDOUT,
            text=False,
            start_new_session=True,
        )
    return ManagedProcess(name=name, popen=popen, log_path=log_path)


def _ros_run(sample_key: str, executable: str, extra_env: dict[str, str] | None = None) -> str:
    install_dir = SAMPLES[sample_key]
    env = " ".join(f'{k}="{v}"' for k, v in (extra_env or {}).items())
    prefix = ros_env_prefix(install_dir)
    return f"{env} {prefix} ros2 run demo_nodes {executable}"


def run_sample1_demo(*, duration_sec: float = 6.0) -> DemoSession:
    """Traditional DDS: talker + listener on /demo/chatter (host ROS)."""
    session = DemoSession()
    session.processes.append(
        _start_logged("sample1-listener", _ros_run("01-traditional-dds", "listener"))
    )
    time.sleep(1.0)
    session.processes.append(
        _start_logged("sample1-talker", _ros_run("01-traditional-dds", "talker"))
    )
    time.sleep(duration_sec)
    return session


def build_sample1_docker() -> str:
    """Build the Jazzy Docker image for sample 1."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    if not SAMPLE1_COMPOSE.is_file():
        raise FileNotFoundError(SAMPLE1_COMPOSE)
    result = run_shell(
        f"docker compose -f {SAMPLE1_COMPOSE} build",
        check=False,
        timeout=600,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return (result.stdout or "") + (result.stderr or "")


def run_sample1_docker_demo(*, duration_sec: float = 6.0, ros_domain_id: int = 42) -> str:
    """Run sample 1 talker+listener inside the Jazzy container."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    result = run_shell(
        f"docker compose -f {SAMPLE1_COMPOSE} run --rm "
        f"-e ROS_DOMAIN_ID={ros_domain_id} "
        f"-e DEMO_DURATION_SEC={duration_sec} demo",
        check=False,
        timeout=duration_sec + 120,
    )
    output = (result.stdout or "") + (result.stderr or "")
    if result.returncode not in (0, 124):
        raise RuntimeError(output or f"docker demo failed with code {result.returncode}")
    return output


def run_sample2_demo(*, duration_sec: float = 8.0, start_router: bool = True) -> DemoSession:
    """rmw_zenoh_cpp talker + listener via zenohd."""
    if start_router:
        zenohd_up()
        time.sleep(1.5)

    rmw_env = {
        "RMW_IMPLEMENTATION": "rmw_zenoh_cpp",
        "ZENOH_SESSION_CONFIG_URI": str(SAMPLE2_ZENOH_CFG),
    }
    session = DemoSession()
    session.processes.append(
        _start_logged(
            "sample2-listener",
            _ros_run("02-rmw-zenoh", "listener", rmw_env),
        )
    )
    time.sleep(1.5)
    session.processes.append(
        _start_logged(
            "sample2-talker",
            _ros_run("02-rmw-zenoh", "talker", rmw_env),
        )
    )
    time.sleep(duration_sec)
    return session


def run_sample3_demo(*, duration_sec: float = 10.0, start_router: bool = True) -> DemoSession:
    """DDS talker + bridge + Rust Zenoh subscriber."""
    if start_router:
        zenohd_up()
        time.sleep(1.5)

    if not _which("zenoh-bridge-ros2dds"):
        raise RuntimeError("zenoh-bridge-ros2dds not on PATH")

    session = DemoSession()
    bridge_cmd = (
        f"zenoh-bridge-ros2dds --no-multicast-scouting -c {SAMPLE3_BRIDGE_CFG}"
    )
    session.processes.append(_start_logged("sample3-bridge", bridge_cmd))
    time.sleep(1.5)

    session.processes.append(
        _start_logged(
            "sample3-talker",
            _ros_run("03-dds-zenoh-bridge", "talker"),
        )
    )
    time.sleep(1.0)

    rust_cmd = (
        f"cd {SAMPLE3_RUST} && "
        "MAIN_SUB_ROUTER=tcp/127.0.0.1:7447 "
        "cargo run --quiet --bin main_sub"
    )
    session.processes.append(_start_logged("sample3-rust-sub", rust_cmd))
    time.sleep(duration_sec)
    return session


def run_all_demos(
    *,
    duration_sec: float = 6.0,
    include_sample2: bool = True,
    include_sample3: bool = True,
    stop_router_after: bool = False,
) -> list[tuple[str, str]]:
    """Run samples sequentially; returns [(sample_name, log_tail), ...]."""
    results: list[tuple[str, str]] = []

    s1 = run_sample1_demo(duration_sec=duration_sec)
    results.append(("Sample 1 — Traditional DDS", s1.tail_logs()))
    s1.stop_all()

    if include_sample2:
        s2 = run_sample2_demo(duration_sec=duration_sec + 2)
        results.append(("Sample 2 — rmw_zenoh", s2.tail_logs()))
        s2.stop_all()

    if include_sample3:
        s3 = run_sample3_demo(duration_sec=duration_sec + 4)
        results.append(("Sample 3 — DDS + Bridge", s3.tail_logs()))
        s3.stop_all()

    if stop_router_after:
        zenohd_down()

    return results


def print_results(results: Iterable[tuple[str, str]]) -> None:
    for title, logs in results:
        print(f"\n{'=' * 60}\n{title}\n{'=' * 60}\n{logs}\n")
