use std::collections::BTreeMap;

use polars::prelude::DataFrame;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里定义注册后的表对象，目的是让句柄元数据和可选 DataFrame 在同一个 registry 条目内统一管理。
pub struct RegisteredTable {
    // 2026-03-21: 保存句柄元数据，目的是让上层可以先读取 schema 和来源信息而不必关心底层 DataFrame 细节。
    pub handle: TableHandle,
    // 2026-03-21: 保存可选 DataFrame，目的是兼容当前逐步演进过程中的“先有句柄，后有数据体”模式。
    pub dataframe: Option<DataFrame>,
}

// 2026-03-21: 这里定义最小表注册表，目的是先打通 table_id 分配与已确认表对象托管，再逐步升级为真正的内存表仓库。
#[derive(Default)]
pub struct TableRegistry {
    // 2026-03-21: 记录下一个可用编号，目的是为每张确认后的表分配稳定递增的临时 ID。
    next_id: usize,
    // 2026-03-21: 存放已注册表对象，目的是为后续同进程连续操作预留句柄管理基础。
    tables: BTreeMap<String, RegisteredTable>,
}

impl TableRegistry {
    // 2026-03-21: 提供空注册表构造器，目的是让测试和后续引擎上下文都能从统一初始状态开始。
    pub fn new() -> Self {
        Self {
            next_id: 1,
            tables: BTreeMap::new(),
        }
    }

    // 2026-03-21: 注册已确认表并返回 table_id，目的是把“确认 schema”正式转换成后续可引用的表身份标识。
    pub fn register(&mut self, table: TableHandle) -> String {
        let table_id = self.next_table_id();
        self.tables.insert(
            table_id.clone(),
            RegisteredTable {
                handle: table,
                dataframe: None,
            },
        );
        table_id
    }

    // 2026-03-21: 注册已加载的表对象，目的是让 DataFrame 能和 table_id 绑定，为下一步原子 Tool 铺路。
    pub fn register_loaded(&mut self, loaded: LoadedTable) -> String {
        let table_id = self.next_table_id();
        self.tables.insert(
            table_id.clone(),
            RegisteredTable {
                handle: loaded.handle,
                dataframe: Some(loaded.dataframe),
            },
        );
        table_id
    }

    // 2026-03-21: 暴露句柄查询接口，目的是为后续连续 Tool 操作和测试断言提供最小读取能力。
    pub fn get(&self, table_id: &str) -> Option<&TableHandle> {
        self.tables
            .get(table_id)
            .map(|registered| &registered.handle)
    }

    // 2026-03-21: 暴露 DataFrame 查询接口，目的是让后续选择列、预览等 Tool 能直接读取已加载表数据。
    pub fn get_dataframe(&self, table_id: &str) -> Option<&DataFrame> {
        self.tables
            .get(table_id)
            .and_then(|registered| registered.dataframe.as_ref())
    }

    // 2026-03-21: 统一生成下一个 table_id，目的是避免多个注册入口各自维护编号逻辑导致分配不一致。
    fn next_table_id(&mut self) -> String {
        let table_id = format!("table_{}", self.next_id);
        self.next_id += 1;
        table_id
    }
}
