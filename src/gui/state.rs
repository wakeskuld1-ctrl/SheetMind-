// 2026-03-29 CST: 这里定义 GUI 一级页面枚举，原因是桌面工作台需要稳定的导航状态源；
// 目的：让左侧导航、顶部标题和中心区路由共享同一套页面标识，避免页面切换逻辑散落在 UI 代码里。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Dashboard,
    FilesAndTables,
    DataProcessing,
    AnalysisModeling,
    Reports,
    AiAssistant,
    LicenseSettings,
}

// 2026-03-29 CST: 这里定义工作台快捷动作，原因是首屏不能只停留在空白欢迎语；
// 目的：先把“打开文件、继续工作、新建分析、查看报告”固化成稳定入口，后续再接真实动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardQuickAction {
    pub id: &'static str,
    pub label: &'static str,
}

// 2026-03-29 CST: 这里定义工作台状态，原因是首页需要承载可测试的固定内容；
// 目的：把快捷动作和推荐任务收口成单独状态块，避免首页长期依赖硬编码零散字符串。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardState {
    pub quick_actions: Vec<DashboardQuickAction>,
    pub recommended_tasks: Vec<&'static str>,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            quick_actions: vec![
                DashboardQuickAction {
                    id: "open_file",
                    label: "打开 Excel 文件",
                },
                DashboardQuickAction {
                    id: "resume_work",
                    label: "继续上次工作",
                },
                DashboardQuickAction {
                    id: "new_analysis",
                    label: "新建分析任务",
                },
                DashboardQuickAction {
                    id: "view_reports",
                    label: "查看报告导出",
                },
            ],
            recommended_tasks: vec![
                "单表清洗",
                "多表合并",
                "统计概览",
                "异常诊断",
                "回归分析",
                "一键报告",
            ],
        }
    }
}

// 2026-03-29 CST: 这里定义文件页状态，原因是“文件与表”页要保存当前文件、Sheet 和预览上下文；
// 目的：为 workbook 打开、Sheet 列表读取和后续表确认流程提供统一的 GUI 状态承接点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesPageState {
    pub selected_file_path: Option<String>,
    pub selected_sheet: Option<String>,
    pub sheet_names: Vec<String>,
    pub detected_tables: Vec<String>,
    pub preview_message: String,
}

impl Default for FilesPageState {
    fn default() -> Self {
        Self {
            selected_file_path: None,
            selected_sheet: None,
            sheet_names: Vec::new(),
            detected_tables: Vec::new(),
            preview_message: "尚未选择文件。".to_string(),
        }
    }
}

// 2026-03-29 CST: 这里定义数据处理页里的预设操作项，原因是方案 B 已确定要把常用处理动作模板做成首版 GUI 的真实内容；
// 目的：让页面先具备“能展示哪些处理能力”的产品语义，后续只需把模板逐个接到真实 Tool。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataProcessingPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

// 2026-03-29 CST: 这里定义数据处理页的模板分组，原因是后续会持续增加算法和处理动作，必须先固定信息架构；
// 目的：把清洗、转换、筛选排序、汇总、增强等能力分区稳定下来，减少后续新增功能时的结构波动。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataProcessingOperationGroup {
    pub id: &'static str,
    pub label: &'static str,
    pub presets: Vec<DataProcessingPreset>,
}

// 2026-03-29 CST: 这里定义数据处理页状态，原因是数据处理页已经不能继续停留在 app.rs 的占位文案；
// 目的：把预设模板、当前选中动作、参数区说明、预览说明和操作历史统一沉淀到状态层，为后续真实处理接线做准备。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataProcessingState {
    pub operation_groups: Vec<DataProcessingOperationGroup>,
    pub selected_group_id: String,
    pub selected_preset_id: String,
    pub parameter_hint: String,
    pub preview_message: String,
    pub history: Vec<String>,
}

impl Default for DataProcessingState {
    fn default() -> Self {
        let operation_groups = vec![
            DataProcessingOperationGroup {
                id: "cleaning",
                label: "清洗",
                presets: vec![
                    DataProcessingPreset {
                        id: "fill_missing",
                        label: "填补缺失值",
                        description: "按列配置均值、固定值或前值填补缺失数据。",
                    },
                    DataProcessingPreset {
                        id: "deduplicate",
                        label: "按键去重",
                        description: "根据主键列去重，保留第一条或最后一条记录。",
                    },
                    DataProcessingPreset {
                        id: "normalize_text",
                        label: "文本标准化",
                        description: "统一空白、大小写和常见符号，便于后续匹配。",
                    },
                ],
            },
            DataProcessingOperationGroup {
                id: "transform",
                label: "转换",
                presets: vec![
                    DataProcessingPreset {
                        id: "cast_types",
                        label: "列类型转换",
                        description: "把文本列转为数值、日期或布尔列。",
                    },
                    DataProcessingPreset {
                        id: "rename_columns",
                        label: "重命名字段",
                        description: "批量修正字段名称，统一业务口径。",
                    },
                    DataProcessingPreset {
                        id: "derive_columns",
                        label: "派生新字段",
                        description: "按公式或拼接规则生成新的分析字段。",
                    },
                ],
            },
            DataProcessingOperationGroup {
                id: "filter_sort",
                label: "筛选排序",
                presets: vec![
                    DataProcessingPreset {
                        id: "filter_rows",
                        label: "条件筛选",
                        description: "按字段条件保留目标记录，形成子集数据。",
                    },
                    DataProcessingPreset {
                        id: "sort_rows",
                        label: "多列排序",
                        description: "按一个或多个字段升序或降序整理数据。",
                    },
                    DataProcessingPreset {
                        id: "top_n",
                        label: "Top N 截取",
                        description: "按指标保留前 N 名，快速得到重点对象。",
                    },
                ],
            },
            DataProcessingOperationGroup {
                id: "aggregate",
                label: "汇总",
                presets: vec![
                    DataProcessingPreset {
                        id: "group_aggregate",
                        label: "分组汇总",
                        description: "按维度聚合求和、均值、计数等统计指标。",
                    },
                    DataProcessingPreset {
                        id: "pivot_table",
                        label: "透视汇总",
                        description: "将明细数据快速重排成透视结构。",
                    },
                ],
            },
            DataProcessingOperationGroup {
                id: "enrichment",
                label: "增强",
                presets: vec![
                    DataProcessingPreset {
                        id: "lookup_fill",
                        label: "查找补全",
                        description: "从参考表回填缺失字段，补齐分析维度。",
                    },
                    DataProcessingPreset {
                        id: "window_calc",
                        label: "窗口计算",
                        description: "计算滚动值、累计值和分组内排名。",
                    },
                    DataProcessingPreset {
                        id: "join_tables",
                        label: "表连接预留",
                        description: "预留多表连接入口，后续接到真实 join 能力。",
                    },
                ],
            },
        ];

        let selected_group_id = operation_groups
            .first()
            .map(|group| group.id)
            .unwrap_or_default()
            .to_string();
        let selected_preset_id = operation_groups
            .first()
            .and_then(|group| group.presets.first())
            .map(|preset| preset.id)
            .unwrap_or_default()
            .to_string();

        Self {
            operation_groups,
            selected_group_id,
            selected_preset_id,
            parameter_hint: "先从左侧选择一个预设操作，右侧会显示参数入口和执行说明。".to_string(),
            preview_message: "这里将承载处理前后预览、样本行对比和字段变化摘要。".to_string(),
            history: Vec::new(),
        }
    }
}

impl DataProcessingState {
    // 2026-03-29 CST: 这里补充历史记录写入方法，原因是用户明确要求先具备操作历史模型；
    // 目的：让每次处理动作都走统一追加入口，后续接入真实算法时可以直接复用。
    pub fn push_history(&mut self, entry: impl Into<String>) {
        self.history.push(entry.into());
    }

    // 2026-03-29 CST: 这里提供当前选中分组查询，原因是页面渲染需要稳定获取当前分组上下文；
    // 目的：把分组查找逻辑收口在状态层，避免页面模块重复遍历分组列表。
    pub fn selected_group(&self) -> Option<&DataProcessingOperationGroup> {
        self.operation_groups
            .iter()
            .find(|group| group.id == self.selected_group_id)
    }

    // 2026-03-29 CST: 这里提供当前选中模板查询，原因是参数区和说明区都围绕当前预设动作展开；
    // 目的：让页面层和后续 Tool 桥接层共用同一个动作定位入口。
    pub fn selected_preset(&self) -> Option<&DataProcessingPreset> {
        self.selected_group().and_then(|group| {
            group.presets
                .iter()
                .find(|preset| preset.id == self.selected_preset_id)
        })
    }

    // 2026-03-29 CST: 这里补充模板选择方法，原因是点击模板后需要同步切换当前动作和参数说明；
    // 目的：把选择动作的联动逻辑固定在状态层，避免页面代码自行拼装状态。
    pub fn select_preset(&mut self, group_id: &str, preset_id: &str) {
        self.selected_group_id = group_id.to_string();
        self.selected_preset_id = preset_id.to_string();

        if let Some(preset) = self.selected_preset() {
            self.parameter_hint = format!(
                "当前预设：{}。这里会继续接入该操作的参数表单、校验提示和执行按钮。",
                preset.label
            );
        }
    }
}

// 2026-03-29 CST: 这里定义分析建模页任务类型，原因是 Task 8 明确要求分析页必须以“任务导向入口”组织；
// 目的：把数据概览、质量诊断、关系分析、趋势分析、模型分析和决策建议固定成稳定任务枚举，便于页面与后续 Tool 对齐。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisTaskKind {
    Overview,
    QualityDiagnosis,
    RelationshipAnalysis,
    TrendAnalysis,
    Modeling,
    DecisionSupport,
}

// 2026-03-29 CST: 这里定义分析页任务卡片，原因是方案 B 要求分析页不是简单的 tab，而是有明确语义的任务入口；
// 目的：让 GUI 在未接真实算法前，就能直观表达每类分析要做什么、需要什么输入以及结果会落到哪里。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisTaskCard {
    pub kind: AnalysisTaskKind,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub recommended_input: &'static str,
}

// 2026-03-29 CST: 这里定义分析建模页状态，原因是分析页需要同时承载任务选择、参数说明、结果摘要、图表位和风险解释；
// 目的：把分析页的骨架信息统一沉淀在状态层，为后续统计分析和建模 Tool 接线预留稳定承接点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisState {
    pub task_cards: Vec<AnalysisTaskCard>,
    pub selected_task: AnalysisTaskKind,
    pub parameter_hint: String,
    pub result_summary: String,
    pub chart_hint: String,
    pub risk_notes: Vec<&'static str>,
}

impl Default for AnalysisState {
    fn default() -> Self {
        Self {
            task_cards: vec![
                AnalysisTaskCard {
                    kind: AnalysisTaskKind::Overview,
                    title: "数据概览",
                    subtitle: "先确认样本规模、字段类型和核心分布。",
                    recommended_input: "适合刚导入完成、准备进入分析前的首轮扫描。",
                },
                AnalysisTaskCard {
                    kind: AnalysisTaskKind::QualityDiagnosis,
                    title: "质量诊断",
                    subtitle: "关注缺失值、异常值、重复值和数据偏态。",
                    recommended_input: "适合需要先判断数据是否足够可靠时使用。",
                },
                AnalysisTaskCard {
                    kind: AnalysisTaskKind::RelationshipAnalysis,
                    title: "关系分析",
                    subtitle: "查看变量间相关性、联动关系和结构差异。",
                    recommended_input: "适合先探索影响因素，再决定是否建模。",
                },
                AnalysisTaskCard {
                    kind: AnalysisTaskKind::TrendAnalysis,
                    title: "趋势分析",
                    subtitle: "围绕时间序列观察波动、季节性和增长趋势。",
                    recommended_input: "适合销售、库存、产能等带时间轴的数据。",
                },
                AnalysisTaskCard {
                    kind: AnalysisTaskKind::Modeling,
                    title: "模型分析",
                    subtitle: "为回归、分类或聚类留出任务化入口。",
                    recommended_input: "适合已经明确目标字段，准备进入建模阶段时使用。",
                },
                AnalysisTaskCard {
                    kind: AnalysisTaskKind::DecisionSupport,
                    title: "决策建议",
                    subtitle: "把分析结果收敛为结论、风险和建议动作。",
                    recommended_input: "适合需要把分析结果交付给业务侧或老板时使用。",
                },
            ],
            selected_task: AnalysisTaskKind::Overview,
            parameter_hint: "先选择一个分析任务，右侧会显示该任务的输入要求、参数入口和执行建议。".to_string(),
            result_summary: "这里将承载统计摘要、关键发现和结论摘要。".to_string(),
            chart_hint: "这里将承载图表预览、趋势线、相关性热力图或模型结果图。".to_string(),
            risk_notes: vec![
                "样本不足时，图表和统计结论可能不稳定。",
                "字段类型错误会直接影响诊断、相关性和建模结果。",
                "模型结果需要结合业务语境解释，不能只看指标高低。",
            ],
        }
    }
}

impl AnalysisState {
    // 2026-03-29 CST: 这里提供当前任务卡片查询，原因是参数区、结果区和右侧解释区都围绕当前任务展开；
    // 目的：把任务查找逻辑收口在状态层，避免页面层每次都自行扫描卡片列表。
    pub fn selected_card(&self) -> Option<&AnalysisTaskCard> {
        self.task_cards
            .iter()
            .find(|card| card.kind == self.selected_task)
    }

    // 2026-03-29 CST: 这里补充任务切换方法，原因是任务卡片点击后不仅要更新当前任务，还要同步更新页面说明文案；
    // 目的：让分析页的联动逻辑统一留在状态层，后续接真实 Tool 时只需继续扩展这里。
    pub fn select_task(&mut self, task: AnalysisTaskKind) {
        self.selected_task = task;

        if let Some(title) = self.selected_card().map(|card| card.title) {
            self.parameter_hint = format!(
                "当前任务：{}。这里会继续接入该任务的字段选择、参数配置和执行按钮。",
                title
            );
            self.result_summary = format!(
                "当前聚焦：{}。这里会展示关键发现、核心指标和可导出的分析结论。",
                title
            );
        }
    }
}

// 2026-03-29 CST: 这里定义报告导出模板卡片，原因是方案 B 要求导出页也具备明确的产品化入口；
// 目的：让用户先看到“我要导出成什么交付物”，后续再把模板逐步接入真实导出能力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportTemplateCard {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub target_audience: &'static str,
}

// 2026-03-29 CST: 这里定义报告导出页状态，原因是导出页需要同时保存模板、输出配置和最近导出记录；
// 目的：把导出页的模板化交付骨架固定下来，为后续接 Excel、CSV 和交付包导出做准备。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportsState {
    pub templates: Vec<ReportTemplateCard>,
    pub selected_template_id: String,
    pub output_path: String,
    pub export_format: String,
    pub recent_exports: Vec<String>,
}

impl Default for ReportsState {
    fn default() -> Self {
        let templates = vec![
            ReportTemplateCard {
                id: "executive_summary",
                title: "管理层摘要",
                summary: "适合把关键发现、风险和结论压缩成一页式交付。",
                target_audience: "老板 / 管理层",
            },
            ReportTemplateCard {
                id: "analysis_pack",
                title: "分析结果包",
                summary: "适合导出明细、图表、摘要和说明，形成完整分析交付。",
                target_audience: "业务团队 / 项目方",
            },
            ReportTemplateCard {
                id: "data_delivery",
                title: "数据交付包",
                summary: "适合输出清洗结果、说明文档和后续接手所需材料。",
                target_audience: "客户 / 交付对象",
            },
        ];

        Self {
            selected_template_id: templates
                .first()
                .map(|card| card.id)
                .unwrap_or_default()
                .to_string(),
            templates,
            output_path: ".\\exports".to_string(),
            export_format: "xlsx".to_string(),
            recent_exports: vec![
                "2026-03-29 09:30 管理层摘要（占位）".to_string(),
                "2026-03-29 10:15 分析结果包（占位）".to_string(),
            ],
        }
    }
}

impl ReportsState {
    // 2026-03-29 CST: 这里提供当前导出模板查询，原因是导出说明区和输出配置都需要围绕当前模板展开；
    // 目的：把模板定位逻辑收口在状态层，避免页面层到处重复扫描模板列表。
    pub fn selected_template(&self) -> Option<&ReportTemplateCard> {
        self.templates
            .iter()
            .find(|template| template.id == self.selected_template_id)
    }

    // 2026-03-29 CST: 这里补充导出模板切换方法，原因是模板卡片点击后需要统一更新当前模板；
    // 目的：为后续不同模板映射不同导出参数和导出动作预留稳定入口。
    pub fn select_template(&mut self, template_id: &str) {
        self.selected_template_id = template_id.to_string();
    }
}

// 2026-03-29 CST: 这里定义 AI 建议卡片，原因是 AI 页在首版不接真实模型时，也需要稳定承载“下一步做什么”的建议语义；
// 目的：把后续 AI 建议的显示结构先标准化，方便之后接入本地规则或大模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSuggestionCard {
    pub id: &'static str,
    pub title: &'static str,
    pub reason: &'static str,
}

// 2026-03-29 CST: 这里定义 AI 页状态，原因是 AI 助手页需要同时承载上下文摘要、推荐动作和拟执行动作说明；
// 目的：先把 AI 页做成可扩展容器，后续接本地规则或模型时不需要回头改页面骨架。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiState {
    pub context_summary: String,
    pub suggestions: Vec<AiSuggestionCard>,
    pub proposed_action: String,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            context_summary: "当前还没有接入真实 AI 推理，这里会汇总当前文件、数据集、分析阶段和推荐下一步。".to_string(),
            suggestions: Vec::new(),
            proposed_action: "等待接入 AI 建议后，这里会显示拟执行动作、影响范围和确认入口。".to_string(),
        }
    }
}

// 2026-03-29 CST: 这里定义授权中心可用动作项，原因是产品化授权页需要明确呈现用户可以执行哪些授权相关动作；
// 目的：把激活、刷新、解绑等动作固化成稳定的动作列表，后续再逐步挂真实能力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseActionItem {
    pub id: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
}

// 2026-03-29 CST: 这里定义授权与设置页状态，原因是授权中心需要承载动作区、本地设置区和排障说明；
// 目的：把授权页骨架单独沉淀成状态模型，避免只是把顶部授权文字重复展示一遍。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicensePageState {
    pub available_actions: Vec<LicenseActionItem>,
    pub local_settings_hint: String,
    pub troubleshooting_notes: Vec<&'static str>,
}

impl Default for LicensePageState {
    fn default() -> Self {
        Self {
            available_actions: vec![
                LicenseActionItem {
                    id: "activate",
                    label: "激活授权",
                    summary: "输入许可证信息并绑定当前设备。",
                },
                LicenseActionItem {
                    id: "refresh",
                    label: "刷新状态",
                    summary: "重新校验本地授权和最近验证结果。",
                },
                LicenseActionItem {
                    id: "deactivate",
                    label: "解绑设备",
                    summary: "释放当前设备绑定，为更换机器做准备。",
                },
            ],
            local_settings_hint: "这里会继续接入本地运行设置、默认导出目录和授权校验策略。".to_string(),
            troubleshooting_notes: vec![
                "如果状态显示未授权，请先确认许可证是否已激活。",
                "如果设备状态异常，后续可尝试刷新状态或重新绑定。",
                "正式售卖版需要保持授权页和顶部状态使用同一份摘要来源。",
            ],
        }
    }
}

// 2026-03-29 CST: 这里定义 GUI 总状态，原因是应用壳需要统一管理导航、页面状态和顶部摘要；
// 目的：先建立稳定的状态根对象，让各个页面骨架逐步接入，而不是继续把状态散落在 app.rs。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    current_page: AppPage,
    pub dashboard: DashboardState,
    pub files_page: FilesPageState,
    pub data_processing: DataProcessingState,
    pub analysis: AnalysisState,
    pub reports: ReportsState,
    pub ai: AiState,
    pub license_page: LicensePageState,
    pub project_name: String,
    pub current_file_name: String,
    pub license_status_text: String,
    pub current_dataset_name: String,
    pub right_panel_message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page: AppPage::Dashboard,
            dashboard: DashboardState::default(),
            files_page: FilesPageState::default(),
            data_processing: DataProcessingState::default(),
            analysis: AnalysisState::default(),
            reports: ReportsState::default(),
            ai: AiState::default(),
            license_page: LicensePageState::default(),
            project_name: "未命名项目".to_string(),
            current_file_name: "未打开文件".to_string(),
            license_status_text: "未授权".to_string(),
            current_dataset_name: "暂无数据集".to_string(),
            right_panel_message: "准备开始：先打开 Excel 文件。".to_string(),
        }
    }
}

impl AppState {
    // 2026-03-29 CST: 这里提供页面切换方法，原因是导航点击和测试都需要稳定更新当前页面；
    // 目的：把页面路由收口到状态对象，避免 UI 代码直接操作内部字段。
    pub fn set_page(&mut self, page: AppPage) {
        self.current_page = page;
    }

    // 2026-03-29 CST: 这里提供当前页面读取方法，原因是 UI 渲染和测试都要读取当前路由状态；
    // 目的：用统一接口暴露当前页，避免后续直接访问内部字段。
    pub fn current_page(&self) -> AppPage {
        self.current_page
    }

    // 2026-03-29 CST: 这里暴露工作台快捷动作，原因是首页和测试都需要读取稳定入口列表；
    // 目的：避免页面层直接穿透 DashboardState 内部结构，方便后续替换数据来源。
    pub fn quick_actions(&self) -> &[DashboardQuickAction] {
        &self.dashboard.quick_actions
    }

    // 2026-03-29 CST: 这里暴露授权状态文本读取方法，原因是顶部栏和授权页测试都需要通过稳定接口读取授权文案；
    // 目的：把授权文案读取收口在状态层，避免页面层直接到处访问字段。
    pub fn license_status_text(&self) -> &str {
        &self.license_status_text
    }

    // 2026-03-29 CST: 这里允许应用壳回写授权摘要，原因是顶部栏和授权页需要共享同一状态源；
    // 目的：把授权桥接结果安全写入应用状态，而不是散落在多个局部变量里。
    pub fn apply_license_summary(
        &mut self,
        status_text: String,
        current_file_name: Option<String>,
        current_dataset_name: Option<String>,
    ) {
        self.license_status_text = status_text;
        if let Some(file_name) = current_file_name {
            self.current_file_name = file_name;
        }
        if let Some(dataset_name) = current_dataset_name {
            self.current_dataset_name = dataset_name;
        }
    }

    // 2026-03-29 CST: 这里提供文件页状态可变访问，原因是文件选择与 workbook 桥接都要回写同一块状态；
    // 目的：保持页面状态访问方式一致，避免页面层未来直接散落修改字段。
    pub fn files_page_mut(&mut self) -> &mut FilesPageState {
        &mut self.files_page
    }

    // 2026-03-29 CST: 这里提供数据处理页状态可变访问，原因是模板选择、参数提示和历史记录都会持续回写；
    // 目的：为后续真实处理动作接线预留稳定入口，避免页面层直接修改 AppState 内部细节。
    pub fn data_processing_mut(&mut self) -> &mut DataProcessingState {
        &mut self.data_processing
    }

    // 2026-03-29 CST: 这里提供分析建模页状态可变访问，原因是任务切换和后续分析结果回写都会持续使用；
    // 目的：让分析页和其他页面保持一致的状态访问模式，避免 UI 层直接穿透内部字段。
    pub fn analysis_mut(&mut self) -> &mut AnalysisState {
        &mut self.analysis
    }

    // 2026-03-29 CST: 这里提供报告导出页状态可变访问，原因是模板切换和后续导出记录回写都会使用；
    // 目的：把导出页状态更新收口在统一入口，减少页面层直接改字段的机会。
    pub fn reports_mut(&mut self) -> &mut ReportsState {
        &mut self.reports
    }

    // 2026-03-29 CST: 这里提供 AI 页状态可变访问，原因是后续 AI 建议和拟执行动作会持续更新；
    // 目的：让 AI 页和其他页面一样通过统一方法回写状态，而不是直接穿透内部结构。
    pub fn ai_mut(&mut self) -> &mut AiState {
        &mut self.ai
    }

    // 2026-03-29 CST: 这里提供授权页状态可变访问，原因是后续授权动作、设置项和排障提示会持续更新；
    // 目的：让授权页和其他页面一样通过统一方法管理状态，避免 UI 层直接改内部字段。
    pub fn license_page_mut(&mut self) -> &mut LicensePageState {
        &mut self.license_page
    }
}
