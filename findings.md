# 发现记录

## 当前代码事实

- `src/ops/security_committee_vote.rs` 仍是 5 席投票实现，且所有席位都在同一进程内构造。
- `tests/security_committee_vote_cli.rs` 已新增红测，要求 `committee_engine == "seven_seat_committee_v3"` 且每席 `execution_mode == "child_process"`。
- `src/tools/contracts.rs` 当前 `ToolRequest` 只有 `Deserialize`，`ToolResponse` 只有 `Serialize`，不够支撑内部子进程通过 JSON 往返。
- `src/tools/dispatcher.rs` 与 `src/tools/dispatcher/stock_ops.rs` 是 CLI 正式入口分发链，适合挂内部 seat agent。
- `security_decision_briefing` 会直接调用 `security_committee_vote`，所以 vote 结果合同升级时要兼容 briefing 使用场景。

## 实现判断

- 正式 CLI 路径可以通过子进程调用当前二进制来证明“席位独立执行”。
- 直接函数测试路径若无法解析到 CLI 可执行文件，需要回退到进程内 seat agent，否则测试 harness 下会失效。
- 七席设计应落到现有 `security_committee_vote` 合同中，而不是再造新的 committee tool。

## 2026-04-08 补充发现

- 直接函数测试的 `current_exe()` 指向的是测试 harness，而不是 `excel_skill.exe`，所以需要从邻近 `target/debug` 路径回推正式二进制。
- `briefing` 内嵌 vote 与“重新调用一次 formal vote”的稳定业务语义应一致，但 `process_id / execution_instance_id` 属于每次独立执行的动态证据，不能再做整对象全等。
- 当前“独立证明”最直接的正式证据链是：
  - `committee_engine == "seven_seat_committee_v3"`
  - `votes.len() == 7`
  - 每席 `execution_mode == "child_process"`
  - 7 个 `process_id` 唯一
  - 7 个 `execution_instance_id` 唯一
