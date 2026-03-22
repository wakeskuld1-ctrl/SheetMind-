use chrono::{Datelike, NaiveDate};

// 2026-03-21: 这里集中维护列语义识别与轻量日期/时间解析，目的是让 analyze/stat_summary 等上层能力共用同一套基础规则。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParsedDate {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

impl ParsedDate {
    // 2026-03-21: 这里统一输出 ISO 日期文本，目的是让观察文案和测试断言都保持稳定口径。
    pub fn to_iso_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    // 2026-03-21: 这里统一输出年月键，目的是支撑“日期集中在某个月”的保守观察。
    pub fn to_year_month(&self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedTime {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

// 2026-03-21: 这里维护日期列名启发式，目的是把语义识别先收敛在少量稳定词典上，避免误报。
pub fn looks_like_date_column_name(column: &str) -> bool {
    let normalized = normalize_column_name(column);
    normalized.contains("日期")
        || normalized.contains("date")
        || normalized.ends_with("day")
        || normalized.contains("_day")
}

// 2026-03-21: 这里维护时间列名启发式，目的是优先识别最常见的业务命名而不做激进推断。
pub fn looks_like_time_column_name(column: &str) -> bool {
    let normalized = normalize_column_name(column);
    normalized.contains("时间")
        || normalized.contains("time")
        || normalized.ends_with("hour")
        || normalized.contains("_hour")
}

// 2026-03-21: 这里维护金额列名启发式，目的是让常见金额/费用/价格字段能进入专门观察通道。
pub fn looks_like_amount_column_name(column: &str) -> bool {
    let normalized = normalize_column_name(column);
    [
        "金额", "amount", "amt", "price", "fee", "total", "实付", "应收", "收入", "支出",
    ]
    .iter()
    .any(|keyword| normalized.contains(keyword))
}

// 2026-03-22: 这里改用真实日历校验日期文本，目的是拦住 2 月 30 日、闰年边界等“格式合法但日期不存在”的脏值。
pub fn parse_date_value(value: &str) -> Option<ParsedDate> {
    let candidate = value.trim();
    if candidate.is_empty() {
        return None;
    }

    let date_fragment = candidate
        .split([' ', 'T'])
        .next()
        .unwrap_or(candidate)
        .replace('/', "-");
    let parts = date_fragment.split('-').collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }

    let year = parts[0].parse::<i32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let day = parts[2].parse::<u32>().ok()?;
    let validated = NaiveDate::from_ymd_opt(year, month, day)?;

    Some(ParsedDate {
        year: validated.year() as u32,
        month: validated.month(),
        day: validated.day(),
    })
}

// 2026-03-21: 这里解析时间文本，目的是让纯时间和日期时间字段都能进入时段观察通道。
pub fn parse_time_value(value: &str) -> Option<ParsedTime> {
    let candidate = value.trim();
    if candidate.is_empty() {
        return None;
    }

    let time_fragment = candidate.split([' ', 'T']).last().unwrap_or(candidate);
    let parts = time_fragment.split(':').collect::<Vec<_>>();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }

    let hour = parts[0].parse::<u32>().ok()?;
    let minute = parts[1].parse::<u32>().ok()?;
    let second = if parts.len() == 3 {
        parts[2].parse::<u32>().ok()?
    } else {
        0
    };

    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    Some(ParsedTime {
        hour,
        minute,
        second,
    })
}

// 2026-03-21: 这里根据小时段给出中文分组，目的是让观察文案更贴近非技术用户的日常理解。
pub fn classify_time_period(hour: u32) -> &'static str {
    match hour {
        0..=5 => "凌晨",
        6..=11 => "上午",
        12..=17 => "下午",
        _ => "晚上",
    }
}

// 2026-03-21: 这里统一列名标准化，目的是让中英文、下划线和大小写命名都能走同一套启发式判断。
fn normalize_column_name(column: &str) -> String {
    column.trim().to_lowercase().replace(' ', "_")
}

// 2026-03-21: 这里补充标识列命名识别，目的是让 analyze 与多表关系建议都能复用同一套保守规则。
pub fn looks_like_identifier_column_name(column: &str) -> bool {
    let normalized = normalize_column_name(column);
    let compact = normalized.replace(['_', '-'], "");
    let tokens = normalized
        .split('_')
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    normalized.contains("编号")
        || normalized.contains("编码")
        || normalized.contains("代码")
        || tokens.iter().any(|token| matches!(*token, "id" | "code" | "no"))
        || normalized.ends_with("_id")
        || normalized.ends_with("_code")
        || normalized.ends_with("_no")
        // 2026-03-21: 这里保留对紧凑英文标识列的覆盖，目的是兼容 Excel 里常见的 userid/orderid 一类命名。
        || matches!(
            compact.as_str(),
            "uid" | "userid" | "orderid" | "customerid" | "clientid" | "memberid"
        )
}
