<!-- 2026-03-23: 新增这份说明，原因是 README 需要把“普通用户使用方式”和“开发者构建方式”拆开；目的是给 GitHub 访问者一个不依赖 Rust/cargo 的清晰试用入口。 -->
# Binary Delivery Guide / 二进制交付说明

## 中文

### 这份说明是给谁看的

- 普通业务用户
- 客户侧试用人员
- 不希望安装 Rust / cargo / Python 的团队

### 普通用户怎么用

普通用户只需要拿到维护者已经编译好的 `.exe`，然后通过问答入口或预设 Skill 开始使用。

普通用户不需要：

- 安装 Rust
- 安装 cargo
- 安装 Python
- 打开源码工程自己构建

### 推荐交付物

建议面向客户提供至少这些内容：

- `excel_skill.exe` 或后续统一命名的产品主程序
- 一份简单使用说明
- 一份试用示例或真实走查说明
- 必要时附带预置的 Skill 文档

### 开发者什么时候才需要 Rust

只有以下场景才需要 Rust / cargo：

- 本地开发
- 调试 Tool
- 构建新的预编译二进制
- 参与开源贡献

也就是说，Rust 是研发构建链，不是普通用户运行前提。

## English

### Who this guide is for

- ordinary business users
- customer-side trial users
- teams that do not want Rust, cargo, or Python setup

### How ordinary users should use SheetMind

Ordinary users should receive a prebuilt `.exe` from the maintainer and start from the conversational entry or the prepared Skill flow.

Ordinary users do not need to:

- install Rust
- install cargo
- install Python
- build from source

### Recommended delivery package

A customer-ready package should include at least:

- `excel_skill.exe` or the later unified product executable
- a short usage note
- a trial walkthrough or acceptance note
- prepared Skill documents when needed

### When Rust is actually needed

Rust and cargo are only needed for:

- local development
- Tool debugging
- building a new precompiled binary
- contributing to the open-source repository

Rust is part of the engineering build chain, not a customer prerequisite.
