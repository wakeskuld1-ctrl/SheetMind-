use eframe::egui;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::gui::bridge::license_bridge::{
    LicenseRefreshResult, LicenseSummary, load_license_summary, refresh_license_summary,
};
use crate::gui::pages::{ai, analysis, dashboard, data_processing, files, license, reports};
use crate::gui::state::{AppPage, AppState};
use crate::gui::theme::{APP_BACKGROUND, APP_PRIMARY, APP_TEXT};

// 2026-03-29 CST: 这里定义桌面 GUI 应用壳，原因是 `sheetmind_app` 需要一个稳定的应用状态承载体。
// 目的：把顶部栏、导航、中间页和右侧上下文统一挂到同一个应用对象上，后续页面继续按既定架构往下扩展。
pub struct SheetMindApp {
    pub state: AppState,
    // 2026-03-29 CST: 这里把授权摘要保留在应用壳上，原因是顶部状态和“授权与设置”页要共用同一份摘要来源。
    // 目的：避免授权页重复读取授权服务，并为后续刷新授权状态预留统一落点。
    license_summary: LicenseSummary,
    // 2026-03-30 CST: 这里保留后台授权刷新接收端，原因是“刷新中”必须跨帧可见，不能继续在 UI 线程里同步阻塞执行。
    // 目的：让授权页刷新动作可以后台执行，并在后续帧里统一轮询结果、落地摘要和错误提示。
    license_refresh_rx: Option<Receiver<Result<LicenseRefreshResult, String>>>,
}

impl Default for SheetMindApp {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            license_summary: LicenseSummary::default(),
            license_refresh_rx: None,
        }
    }
}

impl SheetMindApp {
    // 2026-03-29 CST: 这里保留独立构造函数，原因是 GUI 启动时需要注入授权摘要与运行时信息。
    // 目的：让二进制入口保持简洁，避免在启动文件里堆叠状态拼装细节。
    pub fn new() -> Self {
        let mut app = Self::default();
        app.store_license_summary(load_license_summary());
        app
    }

    // 2026-03-29 CST: 这里提供授权摘要只读访问器，原因是测试和授权页都要确认应用壳持有统一授权状态。
    // 目的：让顶部栏和授权页围绕同一份摘要工作，减少后续接入刷新动作时的改动面。
    pub fn license_summary(&self) -> &LicenseSummary {
        &self.license_summary
    }

    // 2026-03-29 CST: 这里提供真实授权刷新入口，原因是授权页后续需要从“启动时快照”升级为“可主动刷新”的状态流。
    // 目的：把刷新动作统一收口到应用壳，确保授权摘要和顶部状态文本始终同步更新。
    pub fn refresh_license_summary(&mut self) {
        self.store_license_summary(load_license_summary());
    }

    // 2026-03-29 CST: 这里提供可注入的刷新入口，原因是 TDD 需要在不依赖真实授权服务的情况下验证同步回写逻辑。
    // 目的：让测试可以稳定构造刷新后的授权摘要，同时复用和正式刷新一致的状态落点。
    pub fn refresh_license_summary_with<F>(&mut self, loader: F)
    where
        F: FnOnce() -> LicenseSummary,
    {
        self.store_license_summary(loader());
    }

    // 2026-03-30 CST: 这里补充授权刷新启动入口，原因是真实“刷新状态”动作需要后台执行，才能让加载态在 GUI 上真正可见。
    // 目的：把刷新线程创建、页面 loading 标记和结果通道收口到应用壳，避免页面层直接触碰异步细节。
    pub fn start_license_refresh(&mut self) {
        self.start_license_refresh_with(refresh_license_summary);
    }

    // 2026-03-30 CST: 这里提供可注入加载器的异步刷新入口，原因是测试需要稳定构造后台刷新结果而不依赖真实授权服务。
    // 目的：让“开始刷新”这一步既能服务正式 GUI，也能为 TDD 提供可控入口。
    pub fn start_license_refresh_with<F>(&mut self, loader: F)
    where
        F: FnOnce() -> Result<LicenseRefreshResult, String> + Send + 'static,
    {
        if self.state.license_page.refresh_in_progress {
            return;
        }

        self.state.license_page.begin_refresh();
        let (sender, receiver) = mpsc::channel();
        self.license_refresh_rx = Some(receiver);

        thread::spawn(move || {
            let _ = sender.send(loader());
        });
    }

    // 2026-03-30 CST: 这里统一落地授权刷新结果，原因是成功、警告和失败都要通过同一条路径更新 GUI 状态。
    // 目的：确保顶部授权文案、授权摘要和授权页反馈始终一致，不会因为刷新结果不同而分叉失控。
    pub fn apply_license_refresh_result(&mut self, result: Result<LicenseRefreshResult, String>) {
        match result {
            Ok(refresh_result) => {
                self.store_license_summary(refresh_result.summary);
                if let Some(warning_message) = refresh_result.warning_message {
                    self.state
                        .license_page
                        .finish_refresh_warning(warning_message);
                } else {
                    self.state.license_page.finish_refresh_success(None);
                }
            }
            Err(error) => {
                self.state.license_page.finish_refresh_failure(error);
            }
        }
    }

    // 2026-03-30 CST: 这里补充授权页动作统一处理入口，原因是授权页按钮已经改为返回页面事件，不能再停留在只渲染不执行的状态。
    // 目的：把“刷新状态”这类动作统一收口到应用壳，避免页面层直接依赖授权加载细节，同时满足红测对页面动作回写闭环的验证。
    pub fn handle_license_page_action(&mut self, action: license::LicensePageAction) {
        match action {
            license::LicensePageAction::RefreshStatus => self.start_license_refresh(),
        }
    }

    // 2026-03-30 CST: 这里保留可注入加载器的页面动作处理入口，原因是现有红绿测试需要在不启动后台线程轮询的情况下直接验证动作落地结果。
    // 目的：让测试继续聚焦“动作触发后的状态结果”，同时不影响真实 GUI 使用后台刷新通道。
    pub fn handle_license_page_action_with<F>(
        &mut self,
        action: license::LicensePageAction,
        loader: F,
    ) where
        F: FnOnce() -> LicenseSummary,
    {
        match action {
            license::LicensePageAction::RefreshStatus => {
                self.state.license_page.begin_refresh();
                self.apply_license_refresh_result(Ok(LicenseRefreshResult::success(loader())));
            }
        }
    }

    // 2026-03-29 CST: 这里对外暴露导航项清单，原因是 smoke 测试和后续 GUI 壳都要读取同一份一级导航定义。
    // 目的：把导航结构固定成可复用契约，避免页面标题和导航标签在不同位置各写一份。
    pub fn navigation_items() -> [(AppPage, &'static str); 7] {
        [
            (AppPage::Dashboard, "工作台"),
            (AppPage::FilesAndTables, "文件与表"),
            (AppPage::DataProcessing, "数据处理"),
            (AppPage::AnalysisModeling, "分析建模"),
            (AppPage::Reports, "报告导出"),
            (AppPage::AiAssistant, "AI 助手"),
            (AppPage::LicenseSettings, "授权与设置"),
        ]
    }

    // 2026-03-29 CST: 这里对外暴露页面标题映射，原因是页面抬头、路由 smoke 和后续文档都需要统一标题来源。
    // 目的：把“页面枚举 -> 页面标题”的映射收口到应用壳，减少后续页面扩展时的重复修改。
    pub fn page_title(page: AppPage) -> &'static str {
        Self::navigation_items()
            .into_iter()
            .find_map(|(candidate, title)| (candidate == page).then_some(title))
            .unwrap_or("SheetMind")
    }

    // 2026-03-29 CST: 这里集中处理授权摘要回写，原因是初始化和刷新都需要同时更新应用壳摘要与顶部授权状态文本。
    // 目的：避免两处分别维护授权状态同步逻辑，降低后续接入刷新按钮时的改动风险。
    fn store_license_summary(&mut self, summary: LicenseSummary) {
        self.state
            .apply_license_summary(summary.status_text.clone(), None, None);
        self.license_summary = summary;
    }

    // 2026-03-30 CST: 这里轮询后台授权刷新结果，原因是后台线程完成后必须在 UI 线程里安全回写状态。
    // 目的：让刷新动作真正具备跨帧 loading / warning / error 闭环，而不是点击后静默阻塞。
    fn poll_license_refresh(&mut self) {
        let Some(receiver) = self.license_refresh_rx.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => self.apply_license_refresh_result(result),
            Err(TryRecvError::Empty) => {
                self.license_refresh_rx = Some(receiver);
            }
            Err(TryRecvError::Disconnected) => {
                self.state
                    .license_page
                    .finish_refresh_failure("授权刷新任务已中断".to_string());
            }
        }
    }

    // 2026-03-29 CST: 这里渲染顶部栏，原因是项目、文件和授权状态属于全局上下文。
    // 目的：让用户进入 GUI 后第一时间看到当前工作对象，而不是空白页面。
    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("sheetmind_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("SheetMind");
                ui.separator();
                ui.label(format!("项目：{}", self.state.project_name));
                ui.separator();
                ui.label(format!("文件：{}", self.state.current_file_name));
                ui.separator();
                ui.label(format!("授权：{}", self.state.license_status_text()));
            });
        });
    }

    // 2026-03-29 CST: 这里渲染左侧导航，原因是首发版本已经固定为七个一级页面。
    // 目的：先把页面切换路径稳定下来，再逐步替换中心区具体内容。
    fn render_left_nav(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sheetmind_left_nav")
            .resizable(false)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.heading("导航");
                ui.separator();

                for (page, label) in Self::navigation_items() {
                    let selected = self.state.current_page() == page;
                    if ui.selectable_label(selected, label).clicked() {
                        self.state.set_page(page);
                    }
                }
            });
    }

    // 2026-03-29 CST: 这里渲染右侧上下文区，原因是参数说明、下一步建议和数据集摘要需要固定落点。
    // 目的：保持整体 GUI 的三栏信息架构稳定，后续页面扩展时不再反复调整布局。
    fn render_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("sheetmind_right_panel")
            .resizable(true)
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.heading("上下文");
                ui.separator();
                ui.label(&self.state.right_panel_message);
                ui.separator();
                ui.label(format!("当前数据集：{}", self.state.current_dataset_name));
            });
    }

    // 2026-03-29 CST: 这里渲染中心区，原因是应用壳需要把当前页面委托给对应页面模块。
    // 目的：维持 app 只负责路由与布局，不把页面细节重新堆回应应用壳。
    fn render_center_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_current_page(ui, frame);
        });
    }

    // 2026-03-29 CST: 这里抽出当前页渲染方法，原因是中心区渲染逻辑需要在 update 里保持简洁。
    // 目的：让页面路由更易读，也方便后续继续拆分到更多页面模块。
    fn render_current_page(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        <Self as eframe::App>::ui(self, ui, frame);
    }
}

impl eframe::App for SheetMindApp {
    // 2026-03-29 CST: 这里实现中心区页面路由，原因是 eframe 需要应用对象提供具体 UI 内容。
    // 目的：根据当前页面状态切换到对应页面模块，同时保留既定导航结构和页面职责边界。
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(
            APP_TEXT[0],
            APP_TEXT[1],
            APP_TEXT[2],
        ));

        ui.heading(Self::page_title(self.state.current_page()));
        ui.separator();

        match self.state.current_page() {
            AppPage::Dashboard => {
                dashboard::render(ui, &self.state);
            }
            AppPage::FilesAndTables => {
                files::render(ui, &mut self.state);
            }
            AppPage::DataProcessing => {
                data_processing::render(ui, &mut self.state);
            }
            AppPage::AnalysisModeling => {
                analysis::render(ui, &mut self.state);
            }
            AppPage::Reports => {
                reports::render(ui, &mut self.state);
            }
            AppPage::AiAssistant => {
                ai::render(ui, &mut self.state);
            }
            AppPage::LicenseSettings => {
                // 2026-03-29 CST: 这里把授权页路由切到独立页面模块，原因是 Task 10 已经具备产品化授权中心骨架。
                // 目的：用真实授权页替换占位文案，并让页面复用应用壳里的统一授权摘要。
                let license_summary = self.license_summary.clone();
                // 2026-03-30 CST: 这里把授权页返回动作正式接回应用壳，原因是“刷新状态”按钮现在已经具备页面事件语义。
                // 目的：让授权页点击可以真实驱动授权摘要刷新，并保持顶部状态文本与授权页摘要同步更新。
                if let Some(action) = license::render(ui, &mut self.state, &license_summary) {
                    self.handle_license_page_action(action);
                }
            }
        }
    }

    // 2026-03-29 CST: 这里实现应用总更新入口，原因是完整 GUI 骨架需要统一调度顶部栏、导航、中间页和右栏。
    // 目的：让二进制入口从“最小窗口”升级为真正可持续扩展的桌面应用壳。
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::light();
        visuals.panel_fill =
            egui::Color32::from_rgb(APP_BACKGROUND[0], APP_BACKGROUND[1], APP_BACKGROUND[2]);
        visuals.selection.bg_fill =
            egui::Color32::from_rgb(APP_PRIMARY[0], APP_PRIMARY[1], APP_PRIMARY[2]);
        ctx.set_visuals(visuals);

        // 2026-03-30 CST: 这里在每帧开头轮询授权刷新结果，原因是后台刷新完成后必须尽快回写到应用壳和授权页。
        // 目的：让 loading 状态、警告提示和错误提示都能随着下一帧稳定落地。
        self.poll_license_refresh();
        if self.state.license_page.refresh_in_progress {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        self.render_top_bar(ctx);
        self.render_left_nav(ctx);
        self.render_right_panel(ctx);
        self.render_center_panel(ctx, frame);
    }
}
