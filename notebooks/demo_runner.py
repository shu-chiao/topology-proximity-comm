"""Docker helpers to build and quick-run the three topology-proximity-comm samples."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]

SAMPLE1_COMPOSE = REPO_ROOT / "samples/01-traditional-dds/docker-compose.yml"
SAMPLE2_COMPOSE = REPO_ROOT / "samples/02-rmw-zenoh/docker-compose.yml"
SAMPLE3_COMPOSE = REPO_ROOT / "samples/03-dds-zenoh-bridge/docker-compose.yml"

_SAMPLE3_NOISE = (
    "zenohd:",
    "zenoh_bridge",
    "zenoh_plugin",
    "zenoh::",
    "Building ",
    "Built ",
    "Attaching to",
    "resolving provenance",
    "Container ",
    "Recreate",
    "Recreated",
    "Starting ",
    "Started ",
    "Image ",
    "[+] Building",
    " DONE ",
    "exited with code",
    "Aborting on container",
    "Stopping ",
    "Stopped ",
)


def format_ros_demo_log(raw: str) -> str:
    """Drop Docker noise; keep talker/listener lines (samples 1–2)."""
    import re

    prefix = re.compile(r"^[\w-]+\s*\|\s*")
    keep = ("(docker demo)", "Publishing:", "I heard:")
    noise = (
        "Building ",
        "Built ",
        "Attaching to",
        "Container ",
        "Creating ",
        "Created ",
        "Starting ",
        "Started ",
        "Image ",
        "[+] Building",
        " DONE ",
        "zenoh::",
        " INFO ",
    )
    lines: list[str] = []
    for line in raw.splitlines():
        if any(n in line for n in noise):
            continue
        if any(k in line for k in keep):
            lines.append(prefix.sub("", line))
    return "\n".join(lines)


def format_sample3_demo_log(raw: str) -> str:
    """Drop Docker/Zenoh noise and group lines by demo phase."""
    import re

    prefix = re.compile(r"^(?:ros2-1|rust-1)\s*\|\s*")
    lines = [prefix.sub("", ln) for ln in raw.splitlines()]

    meta: list[str] = []
    p1_ros: list[str] = []
    p1_rust: list[str] = []
    p2_ros: list[str] = []
    p2_rust: list[str] = []
    errors: list[str] = []

    for line in lines:
        if any(n in line for n in _SAMPLE3_NOISE):
            continue
        if "syntax error" in line or "Error" in line:
            errors.append(line)
            continue
        if "Publishing:" in line:
            p1_ros.append(line)
        elif "[info] demo/chatter" in line or "data='Hello" in line or "(discover)" in line:
            p1_rust.append(line)
        elif "I heard:" in line:
            p2_ros.append(line)
        elif "[pub] put" in line:
            p2_rust.append(line)
        elif any(
            k in line
            for k in (
                "(ros2 demo)",
                "(rust demo)",
                "(main_sub)",
                "(main_pub)",
                "(sub) ",
                "(pub) ",
            )
        ):
            meta.append(line)

    out: list[str] = []
    if errors:
        out.extend(errors)
        out.append("")

    out.append("=== phase 1: ROS talker → Rust sub ===")
    if p1_ros:
        out.append("--- ros2 (talker) ---")
        out.extend(p1_ros)
    else:
        out.append("(no ROS talker lines captured)")
    if p1_rust:
        out.append("--- rust (main_sub) ---")
        out.extend(p1_rust)
    else:
        out.append("(no Rust sub lines captured)")
    out.append("--- phase 1 done ---")
    out.append("")

    out.append("=== phase 2: Rust pub → ROS listener ===")
    if p2_rust:
        out.append("--- rust (main_pub) ---")
        out.extend(p2_rust)
    else:
        out.append("(no Rust pub lines captured)")
    if p2_ros:
        out.append("--- ros2 (listener) ---")
        out.extend(p2_ros)
    else:
        out.append("(no ROS listener lines captured)")
    out.append("--- phase 2 done ---")

    if meta:
        out.append("")
        out.append("--- setup ---")
        out.extend(meta)

    return "\n".join(out)


def _which(name: str) -> str | None:
    return shutil.which(name)


def check_prerequisites() -> dict[str, bool | str | None]:
    """Return a quick readiness map for the Docker notebook workflow."""
    return {
        "repo_root": str(REPO_ROOT),
        "docker": _which("docker"),
        "docker_compose": _which("docker-compose") or (
            "docker compose" if _which("docker") else None
        ),
        "sample1_compose": str(SAMPLE1_COMPOSE),
        "sample2_compose": str(SAMPLE2_COMPOSE),
        "sample3_compose": str(SAMPLE3_COMPOSE),
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


def run_sample1_docker_demo(
    *, duration_sec: float = 6.0, ros_domain_id: int = 42, compact: bool = True
) -> str:
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
    if compact:
        return format_ros_demo_log(output)
    return output


def build_sample2_docker() -> str:
    """Build the Jazzy Docker image for sample 2 (rmw_zenoh_cpp)."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    if not SAMPLE2_COMPOSE.is_file():
        raise FileNotFoundError(SAMPLE2_COMPOSE)
    result = run_shell(
        f"docker compose -f {SAMPLE2_COMPOSE} build",
        check=False,
        timeout=600,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return (result.stdout or "") + (result.stderr or "")


def run_sample2_docker_demo(
    *, duration_sec: float = 6.0, ros_domain_id: int = 42, compact: bool = True
) -> str:
    """Run sample 2 rmw_zenohd + talker + listener inside the Jazzy container."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    result = run_shell(
        f"docker compose -f {SAMPLE2_COMPOSE} run --rm "
        f"-e ROS_DOMAIN_ID={ros_domain_id} "
        f"-e DEMO_DURATION_SEC={duration_sec} demo",
        check=False,
        timeout=duration_sec + 120,
    )
    output = (result.stdout or "") + (result.stderr or "")
    if result.returncode not in (0, 124):
        raise RuntimeError(output or f"docker demo failed with code {result.returncode}")
    if compact:
        return format_ros_demo_log(output)
    return output


def build_sample3_docker() -> str:
    """Build the two-container Docker images for sample 3."""
    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    if not SAMPLE3_COMPOSE.is_file():
        raise FileNotFoundError(SAMPLE3_COMPOSE)
    result = run_shell(
        f"docker compose -f {SAMPLE3_COMPOSE} build",
        check=False,
        timeout=900,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout)
    return (result.stdout or "") + (result.stderr or "")


def run_sample3_docker_demo(
    *,
    phase1_sec: float | None = None,
    phase2_sec: float | None = None,
    duration_sec: float | None = None,
    ros_domain_id: int = 42,
    rebuild: bool = False,
    compact: bool = True,
) -> str:
    """Run sample 3 ros2 + rust containers (phased bridge demo).

    Use ``phase1_sec`` / ``phase2_sec``. ``duration_sec`` is kept for older
    notebook cells and maps to ``phase1_sec=5``, ``phase2_sec=4``.
    """
    if duration_sec is not None:
        if phase1_sec is not None or phase2_sec is not None:
            raise TypeError(
                "pass either duration_sec or phase1_sec/phase2_sec, not both"
            )
        phase1_sec = 5.0
        phase2_sec = 4.0
    else:
        phase1_sec = 5.0 if phase1_sec is None else phase1_sec
        phase2_sec = 4.0 if phase2_sec is None else phase2_sec

    p1 = int(phase1_sec)
    p2 = int(phase2_sec)

    if not _which("docker"):
        raise RuntimeError("docker not on PATH")
    wait_ros = 3
    total = wait_ros + p1 + p2 + 15
    build_flag = " --build" if rebuild else ""
    result = run_shell(
        f"ROS_DOMAIN_ID={ros_domain_id} "
        f"PHASE1_SEC={p1} "
        f"PHASE2_SEC={p2} "
        f"RUST_WAIT_ROS_SEC={wait_ros} "
        f"docker compose -f {SAMPLE3_COMPOSE} up{build_flag} --abort-on-container-exit",
        check=False,
        timeout=total + 300,
    )
    output = (result.stdout or "") + (result.stderr or "")
    run_shell(f"docker compose -f {SAMPLE3_COMPOSE} down", check=False)
    if result.returncode != 0:
        raise RuntimeError(output or f"docker demo failed with code {result.returncode}")
    if compact:
        return format_sample3_demo_log(output)
    return output
