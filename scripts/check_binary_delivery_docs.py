# -*- coding: utf-8 -*-
"""
2026-03-23 修改原因：为“普通用户只使用预编译二进制”补一个可重复执行的文档回归校验。
2026-03-23 修改目的：防止 README 或 Skill 再把 cargo / Rust 安装误暴露为普通用户主入口。
"""

from __future__ import annotations

# 2026-03-23 修改原因：使用 pathlib 统一处理仓库内文件路径，避免手写路径分隔符带来额外噪音。
# 2026-03-23 修改目的：让这个研发校验脚本在仓库内可稳定复用，但它不属于客户运行时依赖。
from pathlib import Path
import sys

# 2026-03-23 修改原因：把仓库根目录显式算出来，便于后续 README/Skill 文本检查复用。
# 2026-03-23 修改目的：集中管理检查目标，减少散落的硬编码路径。
ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "README.md"
SKILL_FILES = [
    ROOT / "skills" / "excel-orchestrator-v1" / "SKILL.md",
    ROOT / "skills" / "table-processing-v1" / "SKILL.md",
    ROOT / "skills" / "analysis-modeling-v1" / "SKILL.md",
    ROOT / "skills" / "decision-assistant-v1" / "SKILL.md",
]
GUIDE = ROOT / "docs" / "acceptance" / "2026-03-23-binary-delivery-guide.md"


def read_text(path: Path) -> str:
    """以 UTF-8 读取文本，统一文档检查入口。"""
    return path.read_text(encoding="utf-8")


def assert_contains(text: str, needle: str, label: str, errors: list[str]) -> None:
    """检查必须存在的关键语义。"""
    if needle not in text:
        errors.append(f"缺少 {label}: {needle}")


def main() -> int:
    """执行文档约束检查；失败时打印所有缺口，便于按 TDD 逐条补齐。"""
    errors: list[str] = []
    readme_text = read_text(README)

    # 2026-03-23 修改原因：普通用户入口必须显式说明“预编译二进制”，否则容易误读成要先装 Rust。
    # 2026-03-23 修改目的：把用户入口与开发者入口硬性分离。
    assert_contains(readme_text, "预编译二进制", "README 普通用户入口语义", errors)
    assert_contains(readme_text, "普通用户不需要安装 Rust", "README 普通用户无需 Rust", errors)
    assert_contains(readme_text, "普通用户不需要安装 cargo", "README 普通用户无需 cargo", errors)
    assert_contains(readme_text, "开发者构建", "README 开发者入口标题", errors)
    assert_contains(readme_text, str(GUIDE.relative_to(ROOT)).replace("\\", "/"), "README 二进制交付说明链接", errors)

    # 2026-03-23 修改原因：README 仍可保留 cargo，但只能留在开发者构建语境。
    # 2026-03-23 修改目的：防止 cargo 再成为面向业务用户的第一使用动作。
    if "### Option A: See the current tool catalog / 查看当前 Tool 目录" in readme_text:
        errors.append("README 仍把 cargo run 放在普通用户 Quick Start 主入口中")

    if "cargo run --quiet" in readme_text and "开发者构建" not in readme_text:
        errors.append("README 出现 cargo run，但没有清楚标注为开发者语境")

    # 2026-03-23 修改原因：总入口与三层 Skill 都必须禁止把 Rust/cargo 安装要求抛给普通用户。
    # 2026-03-23 修改目的：锁死对外话术，避免会话中再次把源码构建说成用户前置条件。
    for skill_path in SKILL_FILES:
        skill_text = read_text(skill_path)
        assert_contains(skill_text, "不要要求普通用户安装 Rust", f"{skill_path.name} 禁止要求普通用户安装 Rust", errors)
        assert_contains(skill_text, "不要要求普通用户安装 cargo", f"{skill_path.name} 禁止要求普通用户安装 cargo", errors)
        assert_contains(skill_text, "不要把 `cargo run` 或 `cargo build` 当成普通用户试用步骤", f"{skill_path.name} 禁止把 cargo 当试用步骤", errors)

    # 2026-03-23 修改原因：新增的二进制交付说明必须真实存在，才能成为 README 的稳定引用目标。
    # 2026-03-23 修改目的：让 GitHub 访问者能顺着首页进入正确试用途径。
    if not GUIDE.exists():
        errors.append(f"缺少二进制交付说明文档: {GUIDE}")

    if errors:
        print("Binary delivery doc check FAILED:")
        for item in errors:
            print(f"- {item}")
        return 1

    print("Binary delivery doc check PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
