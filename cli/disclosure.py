from __future__ import annotations

from pathlib import Path

import typer

from tradingagents.disclosure_runner import run_disclosure_pipeline


app = typer.Typer(
    name="TradingAgents Disclosure",
    help="Disclosure data runner for CNInfo ingestion and SSE verification.",
    add_completion=False,
)


@app.callback()
def main() -> None:
    # 2026-03-27: M3-4 追加 CLI 根回调，原因是 Typer 在单命令场景下会把应用折叠成默认命令；
    # 这会让未来打包后的统一入口和当前测试都无法稳定使用 `run` 子命令，因此这里显式固定为命令组形态。
    return None


@app.command("run")
def run(
    ticker: str = typer.Option(..., "--ticker", help="证券代码，例如 600519"),
    start_date: str = typer.Option(..., "--start-date", help="开始日期，格式 YYYY-MM-DD"),
    end_date: str = typer.Option(..., "--end-date", help="结束日期，格式 YYYY-MM-DD"),
    data_root: Path | None = typer.Option(
        None,
        "--data-root",
        help="运行时数据目录；不传则写入用户目录下的 TradingAgents 数据路径",
    ),
    fetch_cninfo: bool = typer.Option(
        True,
        "--fetch-cninfo/--no-fetch-cninfo",
        help="是否执行巨潮抓取并入库",
    ),
    verify_sse: bool = typer.Option(
        True,
        "--verify-sse/--no-verify-sse",
        help="是否执行上交所校验",
    ),
) -> None:
    # 2026-03-27: CLI 先保持单命令、非交互、固定输出，目的是从一开始就为免环境打包设计。
    summary = run_disclosure_pipeline(
        ticker=ticker,
        start_date=start_date,
        end_date=end_date,
        data_root=data_root,
        fetch_cninfo=fetch_cninfo,
        verify_sse=verify_sse,
    )
    typer.echo(f"Ticker: {summary.ticker}")
    typer.echo(f"时间窗: {summary.start_date} -> {summary.end_date}")
    typer.echo(f"CNInfo 入库: {summary.cninfo_ingested_count}")
    typer.echo(f"SSE 抓取: {summary.sse_fetched_count}")
    typer.echo(f"SSE 匹配: {summary.matched_count}")
    typer.echo(f"SSE 独有: {summary.sse_only_count}")
    typer.echo(f"CNInfo 独有: {summary.cninfo_only_count}")
    typer.echo(f"数据库: {summary.db_path}")
    typer.echo(f"快照目录: {summary.snapshot_root}")
    typer.echo(f"摘要报告: {summary.report_path}")


if __name__ == "__main__":
    app()
