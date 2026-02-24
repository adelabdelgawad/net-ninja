use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Combined result for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombinedResult {
    pub line_id: i32,
    pub line_number: String,
    pub name: String,
    pub isp: Option<String>,
    pub description: Option<String>,
    pub download: Option<f64>,
    pub upload: Option<f64>,
    pub ping: Option<f64>,
    pub data_used: Option<f64>,
    pub usage_percentage: Option<f64>,
    pub data_remaining: Option<f64>,
    pub renewal_date: Option<NaiveDate>,
    pub balance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<DateTime<Utc>>,
}
