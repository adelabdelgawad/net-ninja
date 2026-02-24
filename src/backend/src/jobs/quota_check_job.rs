use std::time::Duration;

use chrono::NaiveDate;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::clients::WebDriverClient;
use crate::config::Settings;
use crate::db::create_pool;
use crate::errors::AppResult;
use crate::models::{CreateQuotaResultRequest, Line};
use crate::services::{LineService, LogService, QuotaCheckService};

const LOGIN_URL: &str = "https://my.te.eg/user/login";
const OVERVIEW_URL: &str = "https://my.te.eg/offering/overview";
const RENEWAL_URL: &str = "https://my.te.eg/echannel/#/overview";

// CSS Selectors for the ISP portal
const BALANCE_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div:nth-child(2) > div > div > div > div > div:nth-child(3) > div:nth-child(1)";
const USED_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div.ant-col.ant-col-24 > div > div > div.ant-row.ec_accountoverview_primaryBtn_Qyg-Vp > div:nth-child(2) > div > div > div.slick-list > div > div.slick-slide.slick-active.slick-current > div > div > div > div > div:nth-child(2) > div:nth-child(2) > span:nth-child(1)";
const REMAINING_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div.ant-col.ant-col-24 > div > div > div.ant-row.ec_accountoverview_primaryBtn_Qyg-Vp > div:nth-child(2) > div > div > div.slick-list > div > div.slick-slide.slick-active.slick-current > div > div > div > div > div:nth-child(2) > div:nth-child(1) > span:nth-child(1)";
const RENEWAL_COST_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div.ant-col.ant-col-xs-24.ant-col-sm-24.ant-col-md-14.ant-col-lg-14.ant-col-xl-14 > div > div > div > div > div:nth-child(3) > div > span:nth-child(2) > div > div:nth-child(1)";
const RENEWAL_DATE_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div.ant-col.ant-col-xs-24.ant-col-sm-24.ant-col-md-14.ant-col-lg-14.ant-col-xl-14 > div > div > div > div > div:nth-child(4) > div > span";

// Login form selectors
const USERNAME_INPUT: &str = "#login_loginid_input_01";
const PASSWORD_INPUT: &str = "#login_password_input_01";
const LOGIN_TYPE_SELECTOR: &str = "#login_input_type_01";
const LOGIN_TYPE_OPTION: &str = ".ant-select-item-option-active .ant-space-item:nth-child(2) > span";
const LOGIN_BUTTON: &str = "#login-withecare";

#[derive(Debug, Default, Clone)]
struct QuotaData {
    balance: Option<f64>,
    used_quota: Option<f64>,
    remaining_quota: Option<f64>,
    #[allow(dead_code)]
    total_quota: Option<f64>,
    renewal_date: Option<NaiveDate>,
    renewal_cost: Option<f64>,
}

pub async fn run(settings: &Settings) -> AppResult<()> {
    let process_id = Uuid::new_v4();

    // Create database pool for this job run
    let pool = create_pool().await?;

    LogService::info(&pool, process_id, "quota_check_job::run", "Starting quota check job").await?;

    // Get all lines with credentials
    let all_lines = LineService::get_all_with_credentials(&pool).await?;
    let total_lines = all_lines.len();

    // Filter out lines without portal credentials
    let lines: Vec<Line> = all_lines
        .into_iter()
        .filter(|line| !line.username.is_empty() && !line.password.is_empty())
        .collect();

    let skipped_count = total_lines - lines.len();
    if skipped_count > 0 {
        LogService::info(
            &pool,
            process_id,
            "quota_check_job::run",
            &format!("Skipped {} lines without portal credentials", skipped_count),
        )
        .await?;
    }

    LogService::info(
        &pool,
        process_id,
        "quota_check_job::run",
        &format!("Found {} lines to check", lines.len()),
    )
    .await?;

    // Insert an empty pending record for every line before scraping begins.
    // This clears the "latest result" shown in the UI so stale data from the
    // previous run is not displayed while the new check is in progress.
    for line in &lines {
        let pending = CreateQuotaResultRequest {
            line_id: line.id,
            process_id,
            balance: None,
            quota_percentage: None,
            used_quota: None,
            remaining_quota: None,
            total_quota: None,
            renewal_date: None,
            renewal_cost: None,
            extra_quota: None,
            status: Some("pending".to_string()),
            message: Some("Quota check in progress".to_string()),
        };
        if let Err(e) = QuotaCheckService::create(&pool, pending).await {
            tracing::warn!("Failed to insert pending record for line {}: {}", line.id, e);
        }
    }

    // Process lines sequentially — one browser at a time
    let mut success_count = 0;
    let mut error_count = 0;

    for line in &lines {
        match check_quota_for_line(&pool, settings, line).await {
            Ok(()) => success_count += 1,
            Err(e) => {
                tracing::error!("Quota check failed for line {}: {:?}", line.name, e);
                error_count += 1;
            }
        }
    }

    LogService::info(
        &pool,
        process_id,
        "quota_check_job::run",
        &format!(
            "Quota check job completed: {} success, {} errors",
            success_count, error_count
        ),
    )
    .await?;

    Ok(())
}

async fn check_quota_for_line(pool: &SqlitePool, settings: &Settings, line: &Line) -> AppResult<()> {
    let process_id = Uuid::new_v4();

    LogService::info_for_line(
        pool,
        process_id,
        line.id,
        "check_quota_for_line",
        &format!("Starting quota check for line: {}", line.name),
    )
    .await?;

    // Run browser operations (async with chaser-oxide)
    let quota_data = scrape_quota_data(settings, line).await?;

    // Store the result — total_quota and quota_percentage are derived fields,
    // computed on read from used_quota and remaining_quota.
    let request = CreateQuotaResultRequest {
        line_id: line.id,
        process_id,
        balance: quota_data.balance,
        quota_percentage: None,
        used_quota: quota_data.used_quota,
        remaining_quota: quota_data.remaining_quota,
        total_quota: None,
        renewal_date: quota_data.renewal_date,
        renewal_cost: quota_data.renewal_cost,
        extra_quota: None,
        status: Some("success".to_string()),
        message: Some("Quota check completed successfully".to_string()),
    };

    QuotaCheckService::create(pool, request).await?;

    LogService::info_for_line(
        pool,
        process_id,
        line.id,
        "check_quota_for_line",
        &format!(
            "Quota check completed for {}: Balance={:?}, Used={:?}, Remaining={:?}",
            line.name, quota_data.balance, quota_data.used_quota, quota_data.remaining_quota
        ),
    )
    .await?;

    Ok(())
}

/// Async quota scraping using chaser-oxide
async fn scrape_quota_data(settings: &Settings, line: &Line) -> AppResult<QuotaData> {
    let scrape_start = std::time::Instant::now();
    tracing::info!("[scrape_quota_data] ========== STARTING SCRAPE FOR '{}' ==========", line.name);

    // Detect service mode for logging and browser selection
    #[cfg(all(windows, feature = "service"))]
    let service_mode = crate::config::paths::is_service_mode();
    #[cfg(not(all(windows, feature = "service")))]
    let service_mode = false;

    // Log execution context for diagnostics
    tracing::info!(
        "[scrape_quota_data] Execution context: service_mode={}, temp_dir={:?}",
        service_mode,
        std::env::temp_dir()
    );

    // Create WebDriver client (headless mode with anti-detection)
    let browser_start = std::time::Instant::now();
    // Detect execution context and use appropriate browser constructor
    #[cfg(all(windows, feature = "service"))]
    let driver = if service_mode {
        let worker_id = format!("quota-{}", line.id);
        WebDriverClient::new_for_service(&settings.webdriver, &worker_id).await?
    } else {
        WebDriverClient::new_headless(&settings.webdriver).await?
    };

    #[cfg(not(all(windows, feature = "service")))]
    let driver = WebDriverClient::new_headless(&settings.webdriver).await?;
    tracing::info!(
        "[scrape_quota_data] Browser created in {}ms",
        browser_start.elapsed().as_millis()
    );

    let mut data = QuotaData::default();

    // Login
    tracing::info!("[scrape_quota_data] >>> Starting login phase");
    let login_start = std::time::Instant::now();
    login(&driver, line).await?;
    tracing::info!(
        "[scrape_quota_data] <<< Login phase completed in {}ms",
        login_start.elapsed().as_millis()
    );

    // Scrape overview page (required)
    tracing::info!("[scrape_quota_data] >>> Starting overview page scrape");
    let overview_start = std::time::Instant::now();
    scrape_overview_page(&driver, &mut data).await?;
    tracing::info!(
        "[scrape_quota_data] <<< Overview page scrape completed in {}ms",
        overview_start.elapsed().as_millis()
    );

    // Scrape renewal page (required)
    tracing::info!("[scrape_quota_data] >>> Starting renewal page scrape");
    let renewal_start = std::time::Instant::now();
    scrape_renewal_page(&driver, &mut data).await?;
    tracing::info!(
        "[scrape_quota_data] <<< Renewal page scrape completed in {}ms",
        renewal_start.elapsed().as_millis()
    );

    // Browser is automatically closed when driver is dropped
    tracing::info!("[scrape_quota_data] >>> Closing browser");
    let quit_start = std::time::Instant::now();
    driver.quit().await?;
    tracing::info!(
        "[scrape_quota_data] <<< Browser closed in {}ms",
        quit_start.elapsed().as_millis()
    );

    tracing::info!(
        "[scrape_quota_data] ========== SCRAPE COMPLETED FOR '{}' in {}ms ==========",
        line.name,
        scrape_start.elapsed().as_millis()
    );

    Ok(data)
}

async fn login(driver: &WebDriverClient, line: &Line) -> AppResult<()> {
    let phase_start = std::time::Instant::now();
    tracing::info!("[login] Starting login for '{}'", line.name);

    tracing::info!("[login] Navigating to login page");
    driver.navigate(LOGIN_URL).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    tracing::info!(
        "[login] Login page navigation completed in {}ms",
        phase_start.elapsed().as_millis()
    );

    // Wait for login form
    let wait_start = std::time::Instant::now();
    tracing::info!("[login] Waiting for USERNAME_INPUT element");
    driver.wait_for_element(USERNAME_INPUT, 10).await?;
    tracing::info!(
        "[login] USERNAME_INPUT element found in {}ms",
        wait_start.elapsed().as_millis()
    );

    // Enter username — type naturally first (anti-detection), then force React state sync
    let type_start = std::time::Instant::now();
    driver.click_and_type(USERNAME_INPUT, &line.username).await?;
    tracing::info!(
        "[login] Username entered in {}ms",
        type_start.elapsed().as_millis()
    );

    // Force React state sync for username (CDP type_str may not trigger React onChange)
    if let Err(e) = driver.set_react_input_value(USERNAME_INPUT, &line.username).await {
        tracing::warn!("[login] React value sync failed for username (continuing): {}", e);
    }

    // Conditionally select login type (WE serves different page versions)
    let login_type_script = format!(
        "document.querySelector('{}') !== null",
        LOGIN_TYPE_SELECTOR.replace('\'', "\\'")
    );
    let has_login_type = match driver.execute_script(&login_type_script).await {
        Ok(val) => val.as_bool().unwrap_or(false),
        Err(_) => false,
    };
    if has_login_type {
        tracing::info!("[login] Login type dropdown found — selecting type");
        let type_start = std::time::Instant::now();
        driver.click(LOGIN_TYPE_SELECTOR).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        driver.click(LOGIN_TYPE_OPTION).await?;
        tracing::info!("[login] Login type selected in {}ms", type_start.elapsed().as_millis());
    } else {
        tracing::info!("[login] Login type dropdown absent — skipping");
    }

    // Enter password — type naturally first, then force React state sync
    let pass_start = std::time::Instant::now();
    driver.click_and_type(PASSWORD_INPUT, &line.password).await?;
    tracing::info!(
        "[login] Password entered in {}ms",
        pass_start.elapsed().as_millis()
    );

    // Force React state sync for password
    if let Err(e) = driver.set_react_input_value(PASSWORD_INPUT, &line.password).await {
        tracing::warn!("[login] React value sync failed for password (continuing): {}", e);
    }

    // Wait for login button to become enabled (form validation must pass first)
    match driver.wait_for_element_enabled(LOGIN_BUTTON, 5).await {
        Ok(_) => tracing::info!("[login] Login button is enabled"),
        Err(e) => tracing::warn!("[login] Login button still disabled after waiting: {} — clicking anyway", e),
    }

    // Click login button with human-like behavior
    tracing::info!("[login] Clicking login button");
    let click_start = std::time::Instant::now();
    driver.click_human(LOGIN_BUTTON).await?;

    // Poll for URL change instead of a fixed sleep — exits as soon as redirect happens
    let redirect_start = std::time::Instant::now();
    let redirected = loop {
        tokio::time::sleep(Duration::from_millis(300)).await;
        match driver.get_current_url().await {
            Ok(url) if !url.contains("#/login") => break true,
            _ => {}
        }
        if redirect_start.elapsed().as_secs() >= 10 {
            break false;
        }
    };

    tracing::info!(
        "[login] Login button clicked, redirect detected={}  after {}ms, total login time: {}ms",
        redirected,
        click_start.elapsed().as_millis(),
        phase_start.elapsed().as_millis()
    );

    if !redirected {
        if let Ok(url) = driver.get_current_url().await {
            tracing::error!("[login] Login verification FAILED — still on login page: {}", url);
            return Err(crate::errors::AppError::WebDriver(
                format!("Login failed for '{}' — page stayed at login URL: {}", line.name, url)
            ));
        }
    } else if let Ok(url) = driver.get_current_url().await {
        tracing::info!("[login] Login verified — redirected to: {}", url);
    }

    tracing::info!("[login] Login successful for '{}'", line.name);

    Ok(())
}

async fn scrape_overview_page(driver: &WebDriverClient, data: &mut QuotaData) -> AppResult<()> {
    let phase_start = std::time::Instant::now();
    tracing::info!("[scrape_overview_page] Starting overview page scrape");

    tracing::info!("[scrape_overview_page] Navigating to overview page");
    let nav_start = std::time::Instant::now();
    driver.navigate(OVERVIEW_URL).await?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    tracing::info!(
        "[scrape_overview_page] Navigation completed in {}ms",
        nav_start.elapsed().as_millis()
    );

    // Log current URL to verify we're on the right page
    if let Ok(url) = driver.get_current_url().await {
        tracing::info!("[scrape_overview_page] Current URL after navigation: {}", url);
    }

    // Wait for balance element (longer timeout for headless)
    let wait_start = std::time::Instant::now();
    tracing::info!(
        "[scrape_overview_page] About to wait for BALANCE_SELECTOR (pre-wait elapsed: {}ms)",
        phase_start.elapsed().as_millis()
    );
    match driver.wait_for_element(BALANCE_SELECTOR, 30).await {
        Ok(_) => {
            tracing::info!(
                "[scrape_overview_page] BALANCE_SELECTOR wait completed in {}ms",
                wait_start.elapsed().as_millis()
            );
        }
        Err(e) => {
            tracing::error!(
                "[scrape_overview_page] BALANCE_SELECTOR wait FAILED after {}ms",
                wait_start.elapsed().as_millis()
            );
            // Save screenshot for debugging
            if let Ok(screenshot) = driver.take_screenshot().await {
                let path = "/tmp/quota_check_debug.png";
                if std::fs::write(path, &screenshot).is_ok() {
                    tracing::error!("Saved debug screenshot to {}", path);
                }
            }
            // Log page title
            if let Ok(title) = driver.get_title().await {
                tracing::error!("Page title: {}", title);
            }
            return Err(e);
        }
    }

    let extraction_start = std::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Extract balance
    if let Ok(balance_text) = driver.get_text(BALANCE_SELECTOR).await {
        data.balance = parse_number(&balance_text);
    }

    // Extract used quota
    if let Ok(used_text) = driver.get_text(USED_SELECTOR).await {
        data.used_quota = parse_number(&used_text);
    }

    // Extract remaining quota
    if let Ok(remaining_text) = driver.get_text(REMAINING_SELECTOR).await {
        data.remaining_quota = parse_number(&remaining_text);
    }

    tracing::info!(
        "[scrape_overview_page] Extraction completed in {}ms, total phase time: {}ms - Balance={:?}, Used={:?}, Remaining={:?}",
        extraction_start.elapsed().as_millis(),
        phase_start.elapsed().as_millis(),
        data.balance,
        data.used_quota,
        data.remaining_quota
    );

    Ok(())
}

async fn scrape_renewal_page(driver: &WebDriverClient, data: &mut QuotaData) -> AppResult<()> {
    let phase_start = std::time::Instant::now();
    tracing::info!("[scrape_renewal_page] Starting renewal page scrape");

    tracing::info!("[scrape_renewal_page] Navigating to renewal page (SPA)");
    let nav_start = std::time::Instant::now();
    driver.navigate_spa(RENEWAL_URL).await?;
    tracing::info!(
        "[scrape_renewal_page] SPA navigation completed in {}ms",
        nav_start.elapsed().as_millis()
    );

    tokio::time::sleep(Duration::from_millis(300)).await; // Wait for SPA to update
    tracing::info!(
        "[scrape_renewal_page] Post-navigation sleep completed, total elapsed: {}ms",
        phase_start.elapsed().as_millis()
    );

    // Wait for renewal cost element (longer timeout for headless)
    let wait_start = std::time::Instant::now();
    tracing::info!(
        "[scrape_renewal_page] About to wait for RENEWAL_COST_SELECTOR (pre-wait elapsed: {}ms)",
        phase_start.elapsed().as_millis()
    );
    driver.wait_for_element(RENEWAL_COST_SELECTOR, 30).await?;
    tracing::info!(
        "[scrape_renewal_page] RENEWAL_COST_SELECTOR wait completed in {}ms",
        wait_start.elapsed().as_millis()
    );

    let extraction_start = std::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Extract renewal cost
    if let Ok(cost_text) = driver.get_text(RENEWAL_COST_SELECTOR).await {
        data.renewal_cost = parse_number(&cost_text);
    }

    // Extract renewal date
    if let Ok(date_text) = driver.get_text(RENEWAL_DATE_SELECTOR).await {
        data.renewal_date = parse_renewal_date(&date_text);
    }

    tracing::info!(
        "[scrape_renewal_page] Extraction completed in {}ms, total phase time: {}ms - Renewal Cost={:?}, Renewal Date={:?}",
        extraction_start.elapsed().as_millis(),
        phase_start.elapsed().as_millis(),
        data.renewal_cost,
        data.renewal_date
    );

    Ok(())
}

fn parse_number(text: &str) -> Option<f64> {
    let cleaned = text.replace(',', "").trim().to_string();
    cleaned.parse().ok()
}

fn parse_renewal_date(text: &str) -> Option<NaiveDate> {
    // Format: "Renewal Date: DD-MM-YYYY, X Remaining Days"
    // Simple parsing without regex to avoid adding dependency
    if let Some(start) = text.find("Renewal Date: ") {
        let start = start + "Renewal Date: ".len();
        if text.len() >= start + 10 {
            let date_str = &text[start..start + 10];
            return NaiveDate::parse_from_str(date_str, "%d-%m-%Y").ok();
        }
    }
    None
}
