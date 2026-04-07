# 任务计划

## 目标

- 在 `codex/merge-cli-mod-batches` 上把 `security_committee_vote` 升级为七席委员会。
- 保持正式主链仍为 `security_decision_briefing -> security_committee_vote`。
- 为每席补齐独立执行证明，并通过现有 CLI 红测。
- 完成验证、交接文档、任务日志，并推送到远端同分支。

## 阶段

| 阶段 | 状态 | 说明 |
| --- | --- | --- |
| P1 合同差异核对 | 完成 | 已确认当前为 5 席、缺少独立执行字段，红测已钉住 7 席合同。 |
| P2 七席执行链实现 | 进行中 | 补 contracts、vote 主逻辑、内部 seat agent 分发。 |
| P3 测试回归与修正 | 待开始 | 跑 vote CLI 测试及相关回归，修正兼容问题。 |
| P4 文档与推送 | 待开始 | 更新执行说明、交接摘要、CHANGELOG_TASK，并提交推送。 |

## 已知约束

- 不能恢复旧 `security_decision_committee` 主链。
- 不能新增第二套正式入口。
- 必须保留 CLI 分支既有功能，并尽量兼容 briefing 链。
- Windows 上大体积 `apply_patch` 可能失败，需要分块修改。
- 直接函数测试环境不一定能拿到真正的 CLI exe，需要对子进程路径做保护。

## 错误记录

| 错误 | 尝试 | 处理 |
| --- | --- | --- |
| Windows 大块 `apply_patch` 失败，提示文件名或扩展名太长 | 1 | 改为小块 patch 与分文件推进。 |
| `rg.exe` 在当前环境执行被拒绝 | 1 | 改用 PowerShell `Select-String` / `Get-ChildItem` 检索。 |

## 2026-04-08 进度更新

- `P2 七席执行链实现` 已完成。
- `P3 测试回归与修正` 已完成。
- `P4 文档与推送` 进行中，当前正在补执行说明、AI 交接摘要与任务日志。
