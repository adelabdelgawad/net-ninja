use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Raw database row (all TEXT columns)
#[derive(Debug, Clone, FromRow)]
pub struct QuotaResultRow {
    pub id: i32,
    pub line_id: i32,
    pub process_id: String,
    pub balance: Option<String>,
    pub quota_percentage: Option<String>,
    pub used_quota: Option<String>,
    pub total_quota: Option<String>,
    pub remaining_quota: Option<String>,
    pub renewal_date: Option<String>,
    pub renewal_cost: Option<String>,
    pub extra_quota: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
    pub created_at: String,
}

impl QuotaResultRow {
    fn parse_f64(s: &Option<String>) -> Option<f64> {
        s.as_ref().and_then(|v| v.parse().ok())
    }

    fn parse_date(s: &Option<String>) -> Option<NaiveDate> {
        s.as_ref().and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
    }

    fn parse_datetime(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| {
                // Try alternate formats
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
                    .unwrap_or_else(|_| Utc::now())
            })
    }
}

impl From<QuotaResultRow> for QuotaResult {
    fn from(row: QuotaResultRow) -> Self {
        let used_quota = QuotaResultRow::parse_f64(&row.used_quota);
        let remaining_quota = QuotaResultRow::parse_f64(&row.remaining_quota);

        // Derive total from raw scraped values; fall back to stored value for older records.
        let total_quota = match (used_quota, remaining_quota) {
            (Some(u), Some(r)) => Some(u + r),
            _ => QuotaResultRow::parse_f64(&row.total_quota),
        };

        // Derive percentage from raw scraped values; fall back to stored value for older records.
        let quota_percentage = match (used_quota, total_quota) {
            (Some(u), Some(t)) if t > 0.0 => Some(((u / t) * 1000.0).round() / 10.0),
            _ => QuotaResultRow::parse_f64(&row.quota_percentage),
        };

        Self {
            id: row.id,
            line_id: row.line_id,
            process_id: Uuid::parse_str(&row.process_id).unwrap_or_default(),
            balance: QuotaResultRow::parse_f64(&row.balance),
            quota_percentage,
            used_quota,
            total_quota,
            remaining_quota,
            renewal_date: QuotaResultRow::parse_date(&row.renewal_date),
            renewal_cost: QuotaResultRow::parse_f64(&row.renewal_cost),
            extra_quota: QuotaResultRow::parse_f64(&row.extra_quota),
            status: row.status,
            message: row.message,
            created_at: QuotaResultRow::parse_datetime(&row.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QuotaResult {
    pub id: i32,
    #[serde(rename = "lineId")]
    pub line_id: i32,
    #[serde(rename = "processId")]
    pub process_id: Uuid,
    pub balance: Option<f64>,
    #[serde(rename = "quotaPercentage")]
    pub quota_percentage: Option<f64>,
    #[serde(rename = "usedQuota")]
    pub used_quota: Option<f64>,
    #[serde(rename = "totalQuota")]
    pub total_quota: Option<f64>,
    #[serde(rename = "remainingQuota")]
    pub remaining_quota: Option<f64>,
    #[serde(rename = "renewalDate")]
    pub renewal_date: Option<NaiveDate>,
    #[serde(rename = "renewalCost")]
    pub renewal_cost: Option<f64>,
    #[serde(rename = "extraQuota")]
    pub extra_quota: Option<f64>,
    pub status: Option<String>,
    pub message: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaResultResponse {
    pub id: i32,
    #[serde(rename = "lineId")]
    pub line_id: i32,
    #[serde(rename = "processId")]
    pub process_id: Uuid,
    pub balance: Option<f64>,
    #[serde(rename = "quotaPercentage")]
    pub quota_percentage: Option<f64>,
    #[serde(rename = "usedQuota")]
    pub used_quota: Option<f64>,
    #[serde(rename = "totalQuota")]
    pub total_quota: Option<f64>,
    #[serde(rename = "remainingQuota")]
    pub remaining_quota: Option<f64>,
    #[serde(rename = "renewalDate")]
    pub renewal_date: Option<NaiveDate>,
    #[serde(rename = "renewalCost")]
    pub renewal_cost: Option<f64>,
    #[serde(rename = "extraQuota")]
    pub extra_quota: Option<f64>,
    pub status: Option<String>,
    pub message: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

impl From<QuotaResult> for QuotaResultResponse {
    fn from(r: QuotaResult) -> Self {
        Self {
            id: r.id,
            line_id: r.line_id,
            process_id: r.process_id,
            balance: r.balance,
            quota_percentage: r.quota_percentage,
            used_quota: r.used_quota,
            total_quota: r.total_quota,
            remaining_quota: r.remaining_quota,
            renewal_date: r.renewal_date,
            renewal_cost: r.renewal_cost,
            extra_quota: r.extra_quota,
            status: r.status,
            message: r.message,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateQuotaResultRequest {
    #[serde(rename = "lineId")]
    pub line_id: i32,
    #[serde(rename = "processId")]
    pub process_id: Uuid,
    pub balance: Option<f64>,
    #[serde(rename = "quotaPercentage")]
    pub quota_percentage: Option<f64>,
    #[serde(rename = "usedQuota")]
    pub used_quota: Option<f64>,
    #[serde(rename = "totalQuota")]
    pub total_quota: Option<f64>,
    #[serde(rename = "remainingQuota")]
    pub remaining_quota: Option<f64>,
    #[serde(rename = "renewalDate")]
    pub renewal_date: Option<NaiveDate>,
    #[serde(rename = "renewalCost")]
    pub renewal_cost: Option<f64>,
    #[serde(rename = "extraQuota")]
    pub extra_quota: Option<f64>,
    pub status: Option<String>,
    pub message: Option<String>,
}
