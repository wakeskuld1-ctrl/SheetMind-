"""老板决策版业绩诊断工作簿兼容入口。"""

import sys
from pathlib import Path

# 2026-03-28 15:43 修改原因：本轮升级涉及月度预测、周度预警和动作拐点，单文件重构跨度较大。
# 2026-03-28 15:43 修改目的：把新实现收口到独立模块，保留现有入口与导入路径，降低对外部调用和测试路径的扰动。
PROJECT_ROOT = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    # 2026-03-28 21:48 修改原因：直接以脚本方式运行该入口时，Python 不会自动把项目根目录当成包搜索路径。
    # 2026-03-28 21:48 修改目的：让 `python tools\\boss_report_workbook.py` 和 `python -m tools.boss_report_workbook` 都能稳定工作。
    sys.path.insert(0, str(PROJECT_ROOT))

from tools.boss_report_workbook_v3_impl import *  # noqa: F401,F403


if __name__ == "__main__":
    main()
