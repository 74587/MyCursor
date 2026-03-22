/// 用量统计领域模型
///
/// 包含聚合用量、模型用量、用户分析等纯数据结构。
use serde::{Deserialize, Serialize};

/// 聚合用量数据
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AggregatedUsageData {
    pub aggregations: Vec<ModelUsage>,
    pub total_input_tokens: String,
    pub total_output_tokens: String,
    pub total_cache_write_tokens: String,
    pub total_cache_read_tokens: String,
    pub total_cost_cents: f64,
}

/// 单个模型的用量数据
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ModelUsage {
    pub model_intent: String,
    pub input_tokens: String,
    pub output_tokens: String,
    pub cache_write_tokens: String,
    pub cache_read_tokens: String,
    pub total_cents: f64,
}

/// 用量请求参数
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[allow(dead_code)]
pub struct UsageRequest {
    pub start_date: u64,
    pub end_date: u64,
    pub team_id: i32,
}

/// 用户分析数据
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UserAnalyticsData {
    #[serde(rename = "dailyMetrics", default)]
    pub daily_metrics: Vec<DailyMetric>,
    pub period: Period,
    #[serde(rename = "totalMembersInTeam", default)]
    pub total_members_in_team: i32,
    #[serde(rename = "applyLinesRank", default)]
    pub apply_lines_rank: i32,
    #[serde(rename = "tabsAcceptedRank", default)]
    pub tabs_accepted_rank: i32,
    #[serde(rename = "totalTeamMembers", default)]
    pub total_team_members: i32,
    #[serde(rename = "totalApplyLines", default)]
    pub total_apply_lines: i32,
    #[serde(rename = "teamAverageApplyLines", default)]
    pub team_average_apply_lines: i32,
    #[serde(rename = "totalTabsAccepted", default)]
    pub total_tabs_accepted: i32,
    #[serde(rename = "teamAverageTabsAccepted", default)]
    pub team_average_tabs_accepted: i32,
}

/// 每日指标
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DailyMetric {
    pub date: String,
    #[serde(rename = "activeUsers", default)]
    pub active_users: Option<i32>,
    #[serde(rename = "acceptedLinesAdded", default)]
    pub accepted_lines_added: Option<i32>,
    #[serde(rename = "acceptedLinesDeleted", default)]
    pub accepted_lines_deleted: Option<i32>,
    #[serde(rename = "totalApplies", default)]
    pub total_applies: Option<i32>,
    #[serde(rename = "totalAccepts", default)]
    pub total_accepts: Option<i32>,
    #[serde(rename = "totalTabsShown", default)]
    pub total_tabs_shown: Option<i32>,
    #[serde(rename = "totalTabsAccepted", default)]
    pub total_tabs_accepted: Option<i32>,
    #[serde(rename = "composerRequests", default)]
    pub composer_requests: Option<i32>,
    #[serde(rename = "agentRequests", default)]
    pub agent_requests: Option<i32>,
    #[serde(rename = "subscriptionIncludedReqs", default)]
    pub subscription_included_reqs: Option<i32>,
    #[serde(rename = "modelUsage", default)]
    pub model_usage: Option<Vec<ModelCount>>,
    #[serde(rename = "extensionUsage", default)]
    pub extension_usage: Option<Vec<NameCount>>,
    #[serde(rename = "tabExtensionUsage", default)]
    pub tab_extension_usage: Option<Vec<NameCount>>,
    #[serde(rename = "clientVersionUsage", default)]
    pub client_version_usage: Option<Vec<NameCount>>,
}

/// 时间段
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Period {
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
}

/// 模型调用计数
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ModelCount {
    pub name: String,
    pub count: i32,
}

/// 通用名称计数
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NameCount {
    pub name: String,
    pub count: i32,
}

/// 过滤的使用事件数据
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FilteredUsageEventsData {
    #[serde(rename = "totalUsageEventsCount")]
    pub total_usage_events_count: i32,
    #[serde(rename = "usageEventsDisplay")]
    pub usage_events_display: Vec<UsageEventDisplay>,
}

/// 单条使用事件展示数据
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UsageEventDisplay {
    pub timestamp: String,
    pub model: String,
    pub kind: String,
    #[serde(rename = "requestsCosts", default)]
    pub requests_costs: Option<f64>,
    #[serde(rename = "usageBasedCosts")]
    pub usage_based_costs: String,
    #[serde(rename = "isTokenBasedCall")]
    pub is_token_based_call: bool,
    #[serde(rename = "tokenUsage", default)]
    pub token_usage: Option<TokenUsageDetail>,
    #[serde(rename = "owningUser")]
    pub owning_user: String,
}

/// Token 使用详情
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct TokenUsageDetail {
    #[serde(rename = "inputTokens")]
    pub input_tokens: Option<i32>,
    #[serde(rename = "outputTokens")]
    pub output_tokens: Option<i32>,
    #[serde(rename = "cacheWriteTokens")]
    pub cache_write_tokens: Option<i32>,
    #[serde(rename = "cacheReadTokens")]
    pub cache_read_tokens: Option<i32>,
    #[serde(rename = "totalCents")]
    pub total_cents: Option<f64>,
}

/// 过滤使用事件请求
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FilteredUsageRequest {
    #[serde(rename = "teamId")]
    pub team_id: i32,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
    pub page: i32,
    #[serde(rename = "pageSize")]
    pub page_size: i32,
}

/// 用户分析请求
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UserAnalyticsRequest {
    #[serde(rename = "teamId")]
    pub team_id: i32,
    #[serde(rename = "userId")]
    pub user_id: i32,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
}
