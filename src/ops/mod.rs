// 2026-03-21: 这里导出预览、列选择与行过滤操作模块，目的是把基于 DataFrame 的原子能力从 Tool 调度层分离出来。
pub mod preview;
// 2026-03-21: 这里导出列选择操作模块，目的是让后续更多表处理 Tool 共享统一的 DataFrame 操作层。
pub mod select;
// 2026-03-21: 这里导出行过滤操作模块，目的是为后续条件筛选、关系候选分析与汇总前裁剪提供基础能力。
pub mod filter;
// 2026-03-22: 这里导出通用去重模块，目的是补齐整行去重与按子集列去重的基础能力。
pub mod distinct_rows;
// 2026-03-22: 这里导出按业务键去重模块，目的是补齐“先排序、再按主键保留一条”的真实业务清洗能力。
pub mod deduplicate_by_key;
// 2026-03-21: 这里导出显式类型转换模块，目的是为聚合、统计建模和规则判断提供数值化前置能力。
pub mod cast;
// 2026-03-23: 这里导出文本标准化模块，目的是为 join、lookup 和口径统一提供可复用清洗底座。
pub mod normalize_text;
// 2026-03-23: 这里导出列改名模块，目的是把字段口径统一前置成独立 Tool，降低 Skill 编排负担。
pub mod rename;
// 2026-03-22: 这里导出导出前整理模块，目的是把客户交付前的列布局收口为独立输出桥接能力。
pub mod format_table_for_export;
// 2026-03-23: 这里导出 lookup 回填模块，目的是补齐“只补空值、不扩表”的主数据补齐能力。
pub mod fill_lookup;
// 2026-03-22: 这里导出通用补空模块，目的是补齐不依赖 lookup 的列级缺失值填补能力。
pub mod fill_missing_values;
// 2026-03-23: 这里导出透视模块，目的是把 Excel 用户最熟悉的宽表分析能力正式沉淀到 Tool 层。
pub mod pivot;
// 2026-03-23: 这里导出日期时间标准化模块，目的是为窗口、趋势和后续时间分析提供统一时间口径。
pub mod parse_datetime;
// 2026-03-23: 这里导出轻量查值模块，目的是补齐贴近 Excel VLOOKUP/XLOOKUP 心智的带列能力。
pub mod lookup_values;
// 2026-03-23: 这里导出窗口计算模块，目的是补齐排名、组内序号和累计值这类分析桥接能力。
pub mod window;
// 2026-03-22: 这里导出派生字段模块，目的是把条件打标、数值分桶和规则评分沉淀成可复用表处理能力。
pub mod derive;
// 2026-03-22: 这里导出报表写出模块，目的是补齐结果导出为 CSV / Excel 的客户交付能力。
pub mod export;
// 2026-03-23: 这里导出结果交付模板模块，目的是把“标准汇报 workbook 草稿”从零散 compose/export 调用中独立出来。
pub mod report_delivery;
// 2026-03-23: 这里导出独立 SVG 图表渲染模块，原因是 build_chart/export_chart_image 需要脱离 workbook 也能单独交付图表；目的是为纯 Rust 二进制图表导出建立可复用底座。
pub mod chart_svg;
mod excel_chart_writer;
// 2026-03-21: 这里导出分组聚合模块，目的是把表处理正式推进到多维分析入口。
pub mod group;
// 2026-03-21: 这里导出排序模块，目的是让原表、聚合结果与后续 top_n 共用统一排序底座。
pub mod sort;
// 2026-03-21: 这里导出 top_n 模块，目的是把常见排行榜、前几名筛选沉淀成独立原子能力。
pub mod top_n;
// 2026-03-21: 这里导出显性关联模块，目的是把多表等值关联能力下沉到独立算子层。
pub mod join;
// 2026-03-21: 这里导出纵向追加模块，目的是补齐结构相同表的多表拼接能力。
pub mod append;
// 2026-03-21: 这里导出统计摘要模块，目的是为问答分析与建模前画像提供统一列级概览能力。
pub mod summary;
// 2026-03-21: 这里导出统计诊断模块，目的是把表处理层平滑桥接到分析建模层。
pub mod analyze;
// 2026-03-21: 这里导出轻量列语义模块，目的是为业务观察和后续建模前检查提供统一识别规则。
pub mod semantic;
// 2026-03-21: 这里导出多表关系建议模块，目的是先识别显性关联候选，再交给上层 Skill 决定是否执行 join。
pub mod table_links;
// 2026-03-22: 这里导出多表流程建议模块，目的是先判断更像追加还是关联，再由上层 Skill 发起用户确认。
pub mod table_workflow;
// 2026-03-22: 这里导出多表顺序建议模块，目的是把多张表的追加与关联顺序沉淀成可消费的计划步骤。
pub mod multi_table_plan;
// 2026-03-21: 这里导出统计桥接模块，目的是为分析建模层提供独立且稳定的统计摘要入口。
pub mod stat_summary;
// 2026-03-25: 这里导出相关性分析模块，原因是统计诊断型能力第一步要补“目标列与候选特征列的关系排序”；目的是把建模前观察正式沉到 Tool 层。
pub mod correlation_analysis;
// 2026-03-25: 这里导出异常值检测模块，原因是统计诊断型能力第二步要先识别“可疑极端记录”；目的是把异常值标记沉到可链式复用的 Tool 计算层。
pub mod outlier_detection;
// 2026-03-25: 这里导出分布分析模块，原因是统计诊断型能力第三步要回答“数据偏不偏、主要集中在哪”；目的是给建模前观察补齐稳定的传统统计底座。
pub mod distribution_analysis;
// 2026-03-25: 这里导出趋势分析模块，原因是统计诊断型能力第四步要回答“时间上整体是在涨还是跌”；目的是把时间趋势观察正式纳入 Tool 层。
pub mod trend_analysis;
// 2026-03-28 23:54 CST: 这里导出统计诊断组合模块，原因是要把四个独立诊断结果收口成统一高层交付；
// 目的是让 CLI、Skill 和后续 workbook 能直接消费一份稳定的组合诊断合同。
pub mod diagnostics_report;
// 2026-03-29 00:08 CST：这里导出组合诊断 Excel 报表模块，原因是当前统计诊断已经有统一 JSON 合同，下一步要形成 workbook-first 交付；
// 目的：把“组合诊断 -> workbook/xlsx”沉成正式高层 Tool，继续沿 Rust / exe 主线增量推进。
pub mod diagnostics_report_excel_report;
// 2026-03-28 CST: 这里导出股票历史导入模块，原因是用户已经确认股票历史主线要走 SQLite；
// 目的：先把 `CSV -> SQLite` 打成正式 Tool，为后续技术面咨询和 Skill 铺底。
pub mod import_stock_price_history;
// 2026-03-29 CST: 这里导出股票历史 HTTP 同步模块，原因是方案 2+3 已确认要在保留 CSV 主线的同时新增腾讯/新浪双源入口；
// 目的：把“HTTP -> SQLite”沉成正式 Tool，而不是把网络逻辑塞进原 CSV 导入模块。
pub mod sync_stock_price_history;
// 2026-03-28 CST: 这里导出股票技术面基础咨询模块，原因是历史数据已经入 SQLite，下一步要形成真正可调用的分析能力；
// 目的：把 `读取历史行情 -> 计算基础指标 -> 输出技术面建议` 沉成正式 Rust Tool。
pub mod technical_consultation_basic;
// 2026-03-28 10:42 CST: 这里导出容量评估模块，原因是要把运维容量场景纳入现有分析 Tool 体系；目的是支持“可量化结论 + 数据不足指导”的弹性交付。
pub mod capacity_assessment;
// 2026-03-28 16:55 CST: 这里导出容量桥接模块，原因是要把 SSH 盘点结果自动映射进容量模型；目的是打通“采集 -> 证据 -> 结论”的正式链路。
pub mod capacity_assessment_from_inventory;
// 2026-03-28 22:19 CST: 这里导出容量评估 Excel 报表模块，原因是用户最终要的是可直接交付的 Excel；
// 目的是把“容量分析 -> workbook -> xlsx”收口成正式高层 Tool。
pub mod capacity_assessment_excel_report;
// 2026-03-28 16:12 CST: 这里导出受限 SSH 盘点模块，原因是容量评估需要可选的远程部署事实补数；目的是在只读白名单前提下补齐实例和主机资源证据。
pub mod ssh_inventory;
// 2026-03-21: 这里导出线性回归模块，目的是把分析建模层 V1 的首个传统回归能力下沉到独立 Tool 计算层。
pub mod linear_regression;
// 2026-03-21: 这里导出公共建模准备层，目的是让回归、分类和聚类共享统一前处理口径。
pub mod model_prep;
// 2026-03-21: 这里导出建模公共输出模块，目的是统一分析建模层的最外层 JSON 契约。
pub mod model_output;
// 2026-03-21: 这里导出逻辑回归模块，目的是补齐分析建模层 V1 的二分类建模能力。
pub mod logistic_regression;
// 2026-03-21: 这里导出 KMeans 聚类模块，目的是补齐分析建模层 V1 的最后一块传统建模能力。
pub mod cluster_kmeans;
// 2026-03-21: 这里导出决策助手模块，目的是把质量诊断优先的规则建议沉淀成高层 Tool。
pub mod decision_assistant;
