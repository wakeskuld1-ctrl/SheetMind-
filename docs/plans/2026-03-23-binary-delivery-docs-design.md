# Binary Delivery Docs Design

> Date: 2026-03-23
> Scope: README, customer-facing Skill wording, binary delivery guidance

## Goal

收口对外表达，明确 SheetMind 的普通用户只使用预编译二进制，不需要安装 Rust、cargo、Python 或其他脚本运行时；同时保留开发者构建路径，但将其下沉为维护者说明，不再作为普通用户主入口。

## Problem

当前 `D:\Rust\Excel_Skill\README.md` 的 Quick Start 仍把 `cargo run --quiet` 与 `cargo build --release` 暴露在主入口区域。虽然仓库多处已经声明“客户侧只接受 Rust 二进制”，但普通用户仍可能误读为必须先装 Rust 才能试用。

Skill 层虽然限制了 Python，但还没有把“不得建议普通用户安装 Rust / cargo”写成更明确的话术约束。

## Design Principles

1. 普通用户入口与开发者入口分离。
2. README 首页优先展示“直接使用预编译二进制”的路径。
3. `cargo` 只出现在“开发者构建 / 维护者构建”语境中。
4. Skill 明确禁止把安装 Rust / cargo 作为普通用户使用前提。
5. 保持现有 Rust binary-first 方向不变，不引入新的运行时依赖。

## Approach Options

### Option A: 文案轻收口
- 只调整 README 中的 Quick Start 标题与说明。
- 优点：改动小。
- 缺点：Skill 仍可能在对话里把 Rust 当成用户前提。

### Option B: README + Skills + Binary Guide（采用）
- 调整 README 的用户入口与开发者入口。
- 新增单独的二进制交付说明文档。
- 收口 `excel-orchestrator-v1`、`table-processing-v1`、`analysis-modeling-v1`、`decision-assistant-v1` 的运行时约束措辞。
- 优点：对外表达完整，适合 GitHub 展示与后续宣发。
- 缺点：会改多份文档，但风险可控。

### Option C: 再加打包骨架
- 在 Option B 基础上，新增发布目录与打包脚本。
- 优点：更像真实交付物。
- 缺点：超出本轮目标。

## Target Changes

### README
- 把 Quick Start 改为：
  - 普通用户：直接使用预编译二进制
  - 开发者：需要调试或构建时再用 `cargo`
- 增加醒目的“普通用户不需要 Rust / cargo / Python”。
- 保留 `cargo build --release`，但明确标注为开发者构建方式。

### Binary Guide
- 新增二进制交付说明文档，说明：
  - 给普通用户什么文件
  - 普通用户如何试用
  - 开发者何时才需要 Rust

### Skills
- 在运行时约束下追加：
  - 不要要求普通用户安装 Rust 或 cargo
  - 不要把 `cargo run` / `cargo build` 当成客户试用步骤
  - 如需提及 Rust，仅限说明产品底层由 Rust 构建，不代表用户要安装 Rust

## Testing Strategy

采用文档型 TDD：
1. 先写一个校验脚本，断言 README 不再把 `cargo run --quiet` 当作普通用户入口。
2. 断言 README 明确包含“预编译二进制 / 普通用户不需要 Rust 或 cargo”语义。
3. 断言 4 个 Skill 都包含“不要要求普通用户安装 Rust / cargo”语义。
4. 校验脚本先失败，再修改文档，再跑到通过。

## Risks

1. 旧 README 仍可能保留部分开发者措辞，导致校验不够严。
2. Skill 文件历史上有过乱码，需要尽量局部、UTF-8 地修改，避免扩散。
3. 推送 GitHub 时需要确认本次提交只包含合理范围内的文档与前置已完成变更。
