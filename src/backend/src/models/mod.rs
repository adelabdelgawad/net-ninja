pub mod types;
pub mod line;
pub mod quota_result;
pub mod speed_test_result;
pub mod email;
pub mod log;
pub mod smtp_config;
pub mod task;
pub mod task_execution;
pub mod task_notification_config;
pub mod report;

pub use line::*;
pub use quota_result::*;
pub use speed_test_result::*;
pub use email::*;
pub use log::*;
pub use smtp_config::*;
pub use task::*;
pub use task_execution::*;
pub use task_notification_config::*;
pub use report::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self.per_page();
        (page - 1) * per_page
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, params: &PaginationParams) -> Self {
        let per_page = params.per_page();
        let page = params.page.unwrap_or(1).max(1);
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}
