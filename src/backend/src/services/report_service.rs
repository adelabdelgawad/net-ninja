use sqlx::SqlitePool;

use crate::errors::AppResult;
use crate::models::CombinedResult;
use crate::services::{LineService, QuotaCheckService, SpeedTestService};

pub struct ReportService;

impl ReportService {
    pub async fn get_latest_report(pool: &SqlitePool) -> AppResult<Vec<CombinedResult>> {
        let lines = LineService::get_all(pool).await?;
        let quotas = QuotaCheckService::get_latest_for_all_lines(pool).await?;
        let speed_tests = SpeedTestService::get_latest_for_all_lines(pool).await?;

        let results: Vec<CombinedResult> = lines
            .into_iter()
            .map(|line| {
                let quota = quotas.iter().find(|q| q.line_id == line.id);
                let speed_test = speed_tests.iter().find(|s| s.line_id == line.id);

                CombinedResult {
                    line_id: line.id,
                    line_number: line.line_number,
                    name: line.name,
                    isp: line.isp,
                    description: line.description,
                    download: speed_test.and_then(|s| s.download_speed),
                    upload: speed_test.and_then(|s| s.upload_speed),
                    ping: speed_test.and_then(|s| s.ping),
                    data_used: quota.and_then(|q| q.used_quota),
                    usage_percentage: quota.and_then(|q| q.quota_percentage),
                    data_remaining: quota.and_then(|q| q.remaining_quota),
                    renewal_date: quota.and_then(|q| q.renewal_date),
                    balance: quota.and_then(|q| q.balance),
                    last_updated: quota
                        .map(|q| q.created_at)
                        .or_else(|| speed_test.map(|s| s.created_at)),
                }
            })
            .collect();

        Ok(results)
    }
}
