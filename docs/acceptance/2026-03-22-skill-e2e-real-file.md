# 技能端到端实测记录（真实文件）

## 测试说明

- 测试日期：2026-03-22
- 测试方式：人工驱动的 Skill 端到端验收；Skill 回复按 `D:\Rust\Excel_Skill\skills\table-processing-v1\SKILL.md` 的路由原则编写，Tool 请求与 Tool 响应来自真实执行。
- 测试对象：`D:\Excel测试\新疆客户\2026文旅体台账.xlsx`
- Skill 文档：`D:\Rust\Excel_Skill\skills\table-processing-v1\SKILL.md`
- 原始 JSON 工件目录：`D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-skill-e2e-real-file`

## 测试目标

- 验证 Skill 在真实 Excel 上能否按 V1 路由规则工作。
- 验证单表起手是否先 `normalize_table`，遇到 `needs_confirmation` 是否会先停下来追问。
- 验证用户确认后是否能继续调用 `apply_header_schema` 进入确认状态。
- 验证异常 sheet 是否会如实暴露 Tool 错误，而不是被 Skill 层虚构。

## 场景 1：打开工作簿并识别 Sheet

### 我问了什么

请先打开这个 Excel 工作簿，告诉我里面有哪些 sheet：`D:\\Excel测试\\新疆客户\\2026文旅体台账.xlsx`

### Skill 怎么回

我先调用 `open_workbook` 打开工作簿，确认可用 sheet 列表，然后再决定后续单表路由。

### Tool 请求

```json
{
  "tool": "open_workbook",
  "args": {
    "path": "D:\\Excel测试\\新疆客户\\2026文旅体台账.xlsx"
  }
}
```

### Tool 响应

```json
{
  "status": "ok",
  "data": {
    "path": "D:\\Excel测试\\新疆客户\\2026文旅体台账.xlsx",
    "sheet_names": [
      "旅责险",
      "团意险",
      "咨询费",
      "总部分入-线上"
    ]
  }
}
```

### 结论

工作簿可以正常打开，识别出 4 个 sheet：`旅责险`、`团意险`、`咨询费`、`总部分入-线上`。这说明中文路径 + 中文 sheet 名的 Tool 层打开链路是通的。

## 场景 2：对 `旅责险` 做表头识别

### 我问了什么

我想先看 `旅责险` 这张表能不能直接进入表处理，先帮我识别一下表头。

### Skill 怎么回

单表场景先调用 `normalize_table`。如果 Tool 返回 `needs_confirmation`，我就停在表头确认这一步，不继续做后续计算。

### Tool 请求

```json
{
  "tool": "normalize_table",
  "args": {
    "path": "D:\\Excel测试\\新疆客户\\2026文旅体台账.xlsx",
    "sheet": "旅责险"
  }
}
```

### Tool 响应

```json
{
  "status": "needs_confirmation",
  "data": {
    "columns": [
      {
        "canonical_name": "column_1",
        "header_path": [
          "经营单位"
        ]
      },
      {
        "canonical_name": "column_2",
        "header_path": [
          "业务部门"
        ]
      },
      {
        "canonical_name": "column_3",
        "header_path": [
          "委托合同编号"
        ]
      },
      {
        "canonical_name": "column_4",
        "header_path": [
          "客户名称"
        ]
      },
      {
        "canonical_name": "column_5",
        "header_path": [
          "一级业务类别"
        ]
      },
      {
        "canonical_name": "column_6",
        "header_path": [
          "二级业务类别"
        ]
      },
      {
        "canonical_name": "column_7",
        "header_path": [
          "三级业务类别"
        ]
      },
      {
        "canonical_name": "column_8",
        "header_path": [
          "四级业务类别"
        ]
      },
      {
        "canonical_name": "column_9",
        "header_path": [
          "渠道/板块"
        ]
      },
      {
        "canonical_name": "column_10",
        "header_path": [
          "是否全面营销"
        ]
      },
      {
        "canonical_name": "column_11",
        "header_path": [
          "保险公司名称"
        ]
      },
      {
        "canonical_name": "column_12",
        "header_path": [
          "投保人"
        ]
      },
      {
        "canonical_name": "column_13",
        "header_path": [
          "被保险人"
        ]
      },
      {
        "canonical_name": "column_14",
        "header_path": [
          "项目名称"
        ]
      },
      {
        "canonical_name": "column_15",
        "header_path": [
          "签发日期"
        ]
      },
      {
        "canonical_name": "column_16",
        "header_path": [
          "保险起期"
        ]
      },
      {
        "canonical_name": "column_17",
        "header_path": [
          "保险止期"
        ]
      },
      {
        "canonical_name": "column_18",
        "header_path": [
          "险类"
        ]
      },
      {
        "canonical_name": "column_19",
        "header_path": [
          "险种"
        ]
      },
      {
        "canonical_name": "column_20",
        "header_path": [
          "保单性质"
        ]
      },
      {
        "canonical_name": "column_21",
        "header_path": [
          "币种"
        ]
      },
      {
        "canonical_name": "column_22",
        "header_path": [
          "保险金额（原币）"
        ]
      },
      {
        "canonical_name": "column_23",
        "header_path": [
          "保险金额（人民币）"
        ]
      },
      {
        "canonical_name": "column_24",
        "header_path": [
          "总保费（原币）"
        ]
      },
      {
        "canonical_name": "column_25",
        "header_path": [
          "总保费（人民币）"
        ]
      },
      {
        "canonical_name": "column_26",
        "header_path": [
          "保费"
        ]
      },
      {
        "canonical_name": "column_27",
        "header_path": [
          "增值税"
        ]
      },
      {
        "canonical_name": "column_28",
        "header_path": [
          "价税合计"
        ]
      },
      {
        "canonical_name": "column_29",
        "header_path": [
          "承保比例（%）"
        ]
      },
      {
        "canonical_name": "column_30",
        "header_path": [
          "总期次"
        ]
      },
      {
        "canonical_name": "column_31",
        "header_path": [
          "期次"
        ]
      },
      {
        "canonical_name": "column_32",
        "header_path": [
          "保单或批单"
        ]
      },
      {
        "canonical_name": "column_33",
        "header_path": [
          "件数"
        ]
      },
      {
        "canonical_name": "column_34",
        "header_path": [
          "是否含税"
        ]
      },
      {
        "canonical_name": "column_35",
        "header_path": [
          "佣金率(%)"
        ]
      },
      {
        "canonical_name": "column_36",
        "header_path": [
          "佣金（原币）"
        ]
      },
      {
        "canonical_name": "column_37",
        "header_path": [
          "业务收入（人民币）"
        ]
      },
      {
        "canonical_name": "column_38",
        "header_path": [
          "销项税额"
        ]
      },
      {
        "canonical_name": "column_39",
        "header_path": [
          "佣金（人民币）"
        ]
      },
      {
        "canonical_name": "column_40",
        "header_path": [
          "收入明细表编码"
        ]
      },
      {
        "canonical_name": "column_41",
        "header_path": [
          "准凭证号"
        ]
      },
      {
        "canonical_name": "column_42",
        "header_path": [
          "凭证号"
        ]
      },
      {
        "canonical_name": "column_43",
        "header_path": [
          "凭证状态"
        ]
      },
      {
        "canonical_name": "column_44",
        "header_path": [
          "结算单号"
        ]
      },
      {
        "canonical_name": "column_45",
        "header_path": [
          "结算状态"
        ]
      },
      {
        "canonical_name": "column_46",
        "header_path": [
          "到账金额"
        ]
      },
      {
        "canonical_name": "column_47",
        "header_path": [
          "发票号"
        ]
      },
      {
        "canonical_name": "column_48",
        "header_path": [
          "冲正/批单"
        ]
      },
      {
        "canonical_name": "column_49",
        "header_path": [
          "会计月度"
        ]
      },
      {
        "canonical_name": "column_50",
        "header_path": [
          "成功（已审核）日期"
        ]
      },
      {
        "canonical_name": "column_51",
        "header_path": [
          "系统来源"
        ]
      },
      {
        "canonical_name": "column_52",
        "header_path": [
          "医疗机构级别/客户行业/学校学制"
        ]
      },
      {
        "canonical_name": "column_53",
        "header_path": [
          "出单市"
        ]
      },
      {
        "canonical_name": "column_54",
        "header_path": [
          "出单县"
        ]
      },
      {
        "canonical_name": "column_55",
        "header_path": [
          "从业人员工号"
        ]
      },
      {
        "canonical_name": "column_56",
        "header_path": [
          "从业人员姓名"
        ]
      },
      {
        "canonical_name": "column_57",
        "header_path": [
          "执业证书编号"
        ]
      },
      {
        "canonical_name": "column_58",
        "header_path": [
          "录单人员"
        ]
      },
      {
        "canonical_name": "column_59",
        "header_path": [
          "是否代收保费"
        ]
      },
      {
        "canonical_name": "column_60",
        "header_path": [
          "保险产品协议编号"
        ]
      },
      {
        "canonical_name": "column_61",
        "header_path": [
          "协议起期"
        ]
      },
      {
        "canonical_name": "column_62",
        "header_path": [
          "协议止期"
        ]
      },
      {
        "canonical_name": "column_63",
        "header_path": [
          "产品编号"
        ]
      },
      {
        "canonical_name": "column_64",
        "header_path": [
          "产品名称"
        ]
      },
      {
        "canonical_name": "column_65",
        "header_path": [
          "产品类型"
        ]
      },
      {
        "canonical_name": "column_66",
        "header_path": [
          "批次号"
        ]
      },
      {
        "canonical_name": "column_67",
        "header_path": [
          "收入确认时点"
        ]
      },
      {
        "canonical_name": "column_68",
        "header_path": [
          "承保公司"
        ]
      },
      {
        "canonical_name": "column_69",
        "header_path": [
          "承保比例"
        ]
      },
      {
        "canonical_name": "column_70",
        "header_path": [
          "关联业务"
        ]
      },
      {
        "canonical_name": "column_71",
        "header_path": [
          "关联三级分类"
        ]
      },
      {
        "canonical_name": "column_72",
        "header_path": [
          "关联四级分类"
        ]
      },
      {
        "canonical_name": "column_73",
        "header_path": [
          "三类业务"
        ]
      },
      {
        "canonical_name": "column_74",
        "header_path": [
          "应收标记"
        ]
      },
      {
        "canonical_name": "column_75",
        "header_path": [
          "新增/续保"
        ]
      },
      {
        "canonical_name": "column_76",
        "header_path": [
          "备注说明"
        ]
      },
      {
        "canonical_name": "column_77",
        "header_path": [
          "档案移交"
        ]
      },
      {
        "canonical_name": "column_78",
        "header_path": [
          "其他"
        ]
      },
      {
        "canonical_name": "column_79",
        "header_path": [
          "业务人员"
        ]
      }
    ],
    "confidence": "medium",
    "header_row_count": 1,
    "schema_state": "pending",
    "sheet": "旅责险"
  }
}
```

### 结论

`旅责险` 被识别为 `needs_confirmation`，说明 V1 能先判断“结构大致对，但还需用户确认”，而不会跳过表头确认直接计算。

## 场景 3：对 `旅责险` 应用表头确认

### 我问了什么

我确认 `旅责险` 这张表的表头识别可以接受，请继续应用这份 schema。

### Skill 怎么回

表头已确认，我调用 `apply_header_schema` 把这张表置为可继续处理的确认状态。

### Tool 请求

```json
{
  "tool": "apply_header_schema",
  "args": {
    "path": "D:\\Excel测试\\新疆客户\\2026文旅体台账.xlsx",
    "sheet": "旅责险"
  }
}
```

### Tool 响应

```json
{
  "status": "ok",
  "data": {
    "columns": [
      {
        "canonical_name": "column_1",
        "header_path": [
          "经营单位"
        ]
      },
      {
        "canonical_name": "column_2",
        "header_path": [
          "业务部门"
        ]
      },
      {
        "canonical_name": "column_3",
        "header_path": [
          "委托合同编号"
        ]
      },
      {
        "canonical_name": "column_4",
        "header_path": [
          "客户名称"
        ]
      },
      {
        "canonical_name": "column_5",
        "header_path": [
          "一级业务类别"
        ]
      },
      {
        "canonical_name": "column_6",
        "header_path": [
          "二级业务类别"
        ]
      },
      {
        "canonical_name": "column_7",
        "header_path": [
          "三级业务类别"
        ]
      },
      {
        "canonical_name": "column_8",
        "header_path": [
          "四级业务类别"
        ]
      },
      {
        "canonical_name": "column_9",
        "header_path": [
          "渠道/板块"
        ]
      },
      {
        "canonical_name": "column_10",
        "header_path": [
          "是否全面营销"
        ]
      },
      {
        "canonical_name": "column_11",
        "header_path": [
          "保险公司名称"
        ]
      },
      {
        "canonical_name": "column_12",
        "header_path": [
          "投保人"
        ]
      },
      {
        "canonical_name": "column_13",
        "header_path": [
          "被保险人"
        ]
      },
      {
        "canonical_name": "column_14",
        "header_path": [
          "项目名称"
        ]
      },
      {
        "canonical_name": "column_15",
        "header_path": [
          "签发日期"
        ]
      },
      {
        "canonical_name": "column_16",
        "header_path": [
          "保险起期"
        ]
      },
      {
        "canonical_name": "column_17",
        "header_path": [
          "保险止期"
        ]
      },
      {
        "canonical_name": "column_18",
        "header_path": [
          "险类"
        ]
      },
      {
        "canonical_name": "column_19",
        "header_path": [
          "险种"
        ]
      },
      {
        "canonical_name": "column_20",
        "header_path": [
          "保单性质"
        ]
      },
      {
        "canonical_name": "column_21",
        "header_path": [
          "币种"
        ]
      },
      {
        "canonical_name": "column_22",
        "header_path": [
          "保险金额（原币）"
        ]
      },
      {
        "canonical_name": "column_23",
        "header_path": [
          "保险金额（人民币）"
        ]
      },
      {
        "canonical_name": "column_24",
        "header_path": [
          "总保费（原币）"
        ]
      },
      {
        "canonical_name": "column_25",
        "header_path": [
          "总保费（人民币）"
        ]
      },
      {
        "canonical_name": "column_26",
        "header_path": [
          "保费"
        ]
      },
      {
        "canonical_name": "column_27",
        "header_path": [
          "增值税"
        ]
      },
      {
        "canonical_name": "column_28",
        "header_path": [
          "价税合计"
        ]
      },
      {
        "canonical_name": "column_29",
        "header_path": [
          "承保比例（%）"
        ]
      },
      {
        "canonical_name": "column_30",
        "header_path": [
          "总期次"
        ]
      },
      {
        "canonical_name": "column_31",
        "header_path": [
          "期次"
        ]
      },
      {
        "canonical_name": "column_32",
        "header_path": [
          "保单或批单"
        ]
      },
      {
        "canonical_name": "column_33",
        "header_path": [
          "件数"
        ]
      },
      {
        "canonical_name": "column_34",
        "header_path": [
          "是否含税"
        ]
      },
      {
        "canonical_name": "column_35",
        "header_path": [
          "佣金率(%)"
        ]
      },
      {
        "canonical_name": "column_36",
        "header_path": [
          "佣金（原币）"
        ]
      },
      {
        "canonical_name": "column_37",
        "header_path": [
          "业务收入（人民币）"
        ]
      },
      {
        "canonical_name": "column_38",
        "header_path": [
          "销项税额"
        ]
      },
      {
        "canonical_name": "column_39",
        "header_path": [
          "佣金（人民币）"
        ]
      },
      {
        "canonical_name": "column_40",
        "header_path": [
          "收入明细表编码"
        ]
      },
      {
        "canonical_name": "column_41",
        "header_path": [
          "准凭证号"
        ]
      },
      {
        "canonical_name": "column_42",
        "header_path": [
          "凭证号"
        ]
      },
      {
        "canonical_name": "column_43",
        "header_path": [
          "凭证状态"
        ]
      },
      {
        "canonical_name": "column_44",
        "header_path": [
          "结算单号"
        ]
      },
      {
        "canonical_name": "column_45",
        "header_path": [
          "结算状态"
        ]
      },
      {
        "canonical_name": "column_46",
        "header_path": [
          "到账金额"
        ]
      },
      {
        "canonical_name": "column_47",
        "header_path": [
          "发票号"
        ]
      },
      {
        "canonical_name": "column_48",
        "header_path": [
          "冲正/批单"
        ]
      },
      {
        "canonical_name": "column_49",
        "header_path": [
          "会计月度"
        ]
      },
      {
        "canonical_name": "column_50",
        "header_path": [
          "成功（已审核）日期"
        ]
      },
      {
        "canonical_name": "column_51",
        "header_path": [
          "系统来源"
        ]
      },
      {
        "canonical_name": "column_52",
        "header_path": [
          "医疗机构级别/客户行业/学校学制"
        ]
      },
      {
        "canonical_name": "column_53",
        "header_path": [
          "出单市"
        ]
      },
      {
        "canonical_name": "column_54",
        "header_path": [
          "出单县"
        ]
      },
      {
        "canonical_name": "column_55",
        "header_path": [
          "从业人员工号"
        ]
      },
      {
        "canonical_name": "column_56",
        "header_path": [
          "从业人员姓名"
        ]
      },
      {
        "canonical_name": "column_57",
        "header_path": [
          "执业证书编号"
        ]
      },
      {
        "canonical_name": "column_58",
        "header_path": [
          "录单人员"
        ]
      },
      {
        "canonical_name": "column_59",
        "header_path": [
          "是否代收保费"
        ]
      },
      {
        "canonical_name": "column_60",
        "header_path": [
          "保险产品协议编号"
        ]
      },
      {
        "canonical_name": "column_61",
        "header_path": [
          "协议起期"
        ]
      },
      {
        "canonical_name": "column_62",
        "header_path": [
          "协议止期"
        ]
      },
      {
        "canonical_name": "column_63",
        "header_path": [
          "产品编号"
        ]
      },
      {
        "canonical_name": "column_64",
        "header_path": [
          "产品名称"
        ]
      },
      {
        "canonical_name": "column_65",
        "header_path": [
          "产品类型"
        ]
      },
      {
        "canonical_name": "column_66",
        "header_path": [
          "批次号"
        ]
      },
      {
        "canonical_name": "column_67",
        "header_path": [
          "收入确认时点"
        ]
      },
      {
        "canonical_name": "column_68",
        "header_path": [
          "承保公司"
        ]
      },
      {
        "canonical_name": "column_69",
        "header_path": [
          "承保比例"
        ]
      },
      {
        "canonical_name": "column_70",
        "header_path": [
          "关联业务"
        ]
      },
      {
        "canonical_name": "column_71",
        "header_path": [
          "关联三级分类"
        ]
      },
      {
        "canonical_name": "column_72",
        "header_path": [
          "关联四级分类"
        ]
      },
      {
        "canonical_name": "column_73",
        "header_path": [
          "三类业务"
        ]
      },
      {
        "canonical_name": "column_74",
        "header_path": [
          "应收标记"
        ]
      },
      {
        "canonical_name": "column_75",
        "header_path": [
          "新增/续保"
        ]
      },
      {
        "canonical_name": "column_76",
        "header_path": [
          "备注说明"
        ]
      },
      {
        "canonical_name": "column_77",
        "header_path": [
          "档案移交"
        ]
      },
      {
        "canonical_name": "column_78",
        "header_path": [
          "其他"
        ]
      },
      {
        "canonical_name": "column_79",
        "header_path": [
          "业务人员"
        ]
      }
    ],
    "row_count": 683,
    "schema_state": "confirmed",
    "sheet": "旅责险",
    "table_id": "table_1"
  }
}
```

### 结论

`apply_header_schema` 成功，`旅责险` 进入 `confirmed` 状态，行数为 `683`。这说明表头确认链路是通的。

## 场景 4：对 `总部分入-线上` 做表头识别

### 我问了什么

再看一张结构不同的表，请先识别 `总部分入-线上` 的表头。

### Skill 怎么回

我仍然先调用 `normalize_table`，确认这张表是否也需要用户确认表头。

### Tool 请求

```json
{
  "tool": "normalize_table",
  "args": {
    "path": "D:\\Excel测试\\新疆客户\\2026文旅体台账.xlsx",
    "sheet": "总部分入-线上"
  }
}
```

### Tool 响应

```json
{
  "status": "needs_confirmation",
  "data": {
    "columns": [
      {
        "canonical_name": "column_1",
        "header_path": [
          "经营单位"
        ]
      },
      {
        "canonical_name": "column_2",
        "header_path": [
          "业务部门"
        ]
      },
      {
        "canonical_name": "column_3",
        "header_path": [
          "客户"
        ]
      },
      {
        "canonical_name": "column_4",
        "header_path": [
          "项目"
        ]
      },
      {
        "canonical_name": "column_5",
        "header_path": [
          "业务分类"
        ]
      },
      {
        "canonical_name": "column_6",
        "header_path": [
          "险种"
        ]
      },
      {
        "canonical_name": "column_7",
        "header_path": [
          "保单号"
        ]
      },
      {
        "canonical_name": "column_8",
        "header_path": [
          "业务类型"
        ]
      },
      {
        "canonical_name": "column_9",
        "header_path": [
          "分出类型"
        ]
      },
      {
        "canonical_name": "column_10",
        "header_path": [
          "确认金额"
        ]
      },
      {
        "canonical_name": "column_11",
        "header_path": [
          "分出比例"
        ]
      },
      {
        "canonical_name": "column_12",
        "header_path": [
          "分入金额"
        ]
      },
      {
        "canonical_name": "column_13",
        "header_path": [
          "分入机构"
        ]
      },
      {
        "canonical_name": "column_14",
        "header_path": [
          "分入部门"
        ]
      },
      {
        "canonical_name": "column_15",
        "header_path": [
          "所属期间"
        ]
      },
      {
        "canonical_name": "column_16",
        "header_path": [
          "省"
        ]
      },
      {
        "canonical_name": "column_17",
        "header_path": [
          "市"
        ]
      },
      {
        "canonical_name": "column_18",
        "header_path": [
          "营销人员编码"
        ]
      },
      {
        "canonical_name": "column_19",
        "header_path": [
          "营销人员姓名"
        ]
      },
      {
        "canonical_name": "column_20",
        "header_path": [
          "全面营销"
        ]
      },
      {
        "canonical_name": "column_21",
        "header_path": [
          "保险公司"
        ]
      },
      {
        "canonical_name": "column_22",
        "header_path": [
          "保费"
        ]
      },
      {
        "canonical_name": "column_23",
        "header_path": [
          "含税收入"
        ]
      },
      {
        "canonical_name": "column_24",
        "header_path": [
          "佣金比例"
        ]
      },
      {
        "canonical_name": "column_25",
        "header_path": [
          "新增/续保"
        ]
      }
    ],
    "confidence": "medium",
    "header_row_count": 1,
    "schema_state": "pending",
    "sheet": "总部分入-线上"
  }
}
```

### 结论

`总部分入-线上` 也被识别为 `needs_confirmation`，说明这套路由不依赖某一张表的固定样式。

## 场景 5：对 `总部分入-线上` 应用表头确认

### 我问了什么

`总部分入-线上` 这张表的表头我也确认可以，请继续应用 schema。

### Skill 怎么回

表头已确认，我调用 `apply_header_schema` 完成确认状态切换。

### Tool 请求

```json
{
  "tool": "apply_header_schema",
  "args": {
    "path": "D:\\Excel测试\\新疆客户\\2026文旅体台账.xlsx",
    "sheet": "总部分入-线上"
  }
}
```

### Tool 响应

```json
{
  "status": "ok",
  "data": {
    "columns": [
      {
        "canonical_name": "column_1",
        "header_path": [
          "经营单位"
        ]
      },
      {
        "canonical_name": "column_2",
        "header_path": [
          "业务部门"
        ]
      },
      {
        "canonical_name": "column_3",
        "header_path": [
          "客户"
        ]
      },
      {
        "canonical_name": "column_4",
        "header_path": [
          "项目"
        ]
      },
      {
        "canonical_name": "column_5",
        "header_path": [
          "业务分类"
        ]
      },
      {
        "canonical_name": "column_6",
        "header_path": [
          "险种"
        ]
      },
      {
        "canonical_name": "column_7",
        "header_path": [
          "保单号"
        ]
      },
      {
        "canonical_name": "column_8",
        "header_path": [
          "业务类型"
        ]
      },
      {
        "canonical_name": "column_9",
        "header_path": [
          "分出类型"
        ]
      },
      {
        "canonical_name": "column_10",
        "header_path": [
          "确认金额"
        ]
      },
      {
        "canonical_name": "column_11",
        "header_path": [
          "分出比例"
        ]
      },
      {
        "canonical_name": "column_12",
        "header_path": [
          "分入金额"
        ]
      },
      {
        "canonical_name": "column_13",
        "header_path": [
          "分入机构"
        ]
      },
      {
        "canonical_name": "column_14",
        "header_path": [
          "分入部门"
        ]
      },
      {
        "canonical_name": "column_15",
        "header_path": [
          "所属期间"
        ]
      },
      {
        "canonical_name": "column_16",
        "header_path": [
          "省"
        ]
      },
      {
        "canonical_name": "column_17",
        "header_path": [
          "市"
        ]
      },
      {
        "canonical_name": "column_18",
        "header_path": [
          "营销人员编码"
        ]
      },
      {
        "canonical_name": "column_19",
        "header_path": [
          "营销人员姓名"
        ]
      },
      {
        "canonical_name": "column_20",
        "header_path": [
          "全面营销"
        ]
      },
      {
        "canonical_name": "column_21",
        "header_path": [
          "保险公司"
        ]
      },
      {
        "canonical_name": "column_22",
        "header_path": [
          "保费"
        ]
      },
      {
        "canonical_name": "column_23",
        "header_path": [
          "含税收入"
        ]
      },
      {
        "canonical_name": "column_24",
        "header_path": [
          "佣金比例"
        ]
      },
      {
        "canonical_name": "column_25",
        "header_path": [
          "新增/续保"
        ]
      }
    ],
    "row_count": 139,
    "schema_state": "confirmed",
    "sheet": "总部分入-线上",
    "table_id": "table_1"
  }
}
```

### 结论

`apply_header_schema` 成功，`总部分入-线上` 行数为 `139`。这说明不同结构的单表也能走通“先识别 -> 再确认”的链路。

## 场景 6：异常场景：`咨询费` 无可识别数据

### 我问了什么

最后再测一个异常场景，请对 `咨询费` 做一次表头识别。

### Skill 怎么回

我先调用 `normalize_table`。如果 Tool 无法识别出可用表格，就原样保留异常响应，不在 Skill 层编造结果。

### Tool 请求

```json
{
  "tool": "normalize_table",
  "args": {
    "path": "D:\\Excel测试\\新疆客户\\2026文旅体台账.xlsx",
    "sheet": "咨询费"
  }
}
```

### Tool 响应

```json
{
  "status": "error",
  "data": {},
  "error": "????????????"
}
```

### 结论

`咨询费` 返回 `error`。本次实测里，Tool 的 `error` 字段实际输出就是 `????????????`，文档已按实际响应保留。这说明异常路径可以走到，但错误信息的中文可读性还需继续收口。

## 总结

- `open_workbook` 在这个真实中文路径 Excel 上可正常打开。
- `旅责险` 和 `总部分入-线上` 都走通了 `normalize_table -> needs_confirmation -> apply_header_schema` 链路。
- `咨询费` 暴露了异常路径：Skill 没有乱猜结果，而是原样保留 Tool 错误。
- 当前仍有一个明确收口项：`咨询费` 错误信息的中文可读性不足，实际输出仍为 `????????????`。

## 后续建议

1. 把“确认后 schema 怎么继续传递到 `preview_table` / `stat_summary` 等 Tool”补成完整验收链路。
2. 把 `needs_confirmation` 里的 `column_n + header_path` 组装成更适合非 IT 用户的确认问法。
3. 单独收口 `咨询费` 这类“无可识别数据”的中文错误输出。
