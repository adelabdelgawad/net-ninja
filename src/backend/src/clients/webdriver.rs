//! WebDriver client using chaser-oxide crate.
//!
//! This module provides an async API for browser automation using
//! the chaser-oxide crate with built-in anti-detection features.
//! All methods are async and can be called directly in async contexts.

use std::time::Duration;

use chaser_oxide::browser::{Browser, BrowserConfig};
use chaser_oxide::page::{Page, ScreenshotParams};
use chaser_oxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use futures_util::StreamExt;
use rand::Rng;
use uuid::Uuid;

use super::chrome_installer::ensure_chrome_available;
use crate::config::WebDriverSettings;
use crate::errors::{AppError, AppResult};

/// Maximum age for stale Chrome profiles (24 hours)
#[cfg(all(windows, feature = "service"))]
const PROFILE_MAX_AGE_SECS: u64 = 24 * 60 * 60;

// Global mutex to serialize browser launches
// This prevents Chrome's "Opening in existing browser session" conflicts
static BROWSER_LAUNCH_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// WebDriver client wrapper for chaser-oxide.
///
/// This client uses the Chrome DevTools Protocol (CDP) for browser automation
/// with advanced anti-detection features including built-in stealth mode.
pub struct WebDriverClient {
    browser: Browser,
    page: Page,
    user_data_dir: Option<std::path::PathBuf>,
    /// Unique identifier for this browser instance (for logging/debugging)
    browser_id: Uuid,
}

impl WebDriverClient {
    /// Create a new headless WebDriver client with anti-detection.
    pub async fn new_headless(settings: &WebDriverSettings) -> AppResult<Self> {
        // Generate unique browser ID for this instance (used in all logging)
        let browser_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();

        tracing::info!(
            "[Browser {}] [t=0ms] Starting browser creation",
            browser_id
        );

        // Get Chrome path (auto-install if needed)
        let chrome_path = ensure_chrome_available(settings)?;

        // CRITICAL: Create unique temp directory for this browser instance
        // This ensures multiple concurrent browsers don't interfere with each other
        let temp_dir = std::env::temp_dir();
        let user_data_dir = temp_dir.join(format!("chrome-session-{}", browser_id));

        // Create the directory
        std::fs::create_dir_all(&user_data_dir)
            .map_err(|e| AppError::WebDriver(format!("Failed to create temp profile dir: {}", e)))?;

        tracing::debug!("Created temporary Chrome profile: {:?}", user_data_dir);

        // Build browser configuration
        let mut config_builder = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .request_timeout(Duration::from_secs(30));

        // CRITICAL: Generate a unique port for this browser instance
        // This is the key to avoiding "Opening in existing browser session"
        let debug_port = {
            // Find an available port by binding to port 0
            let listener = std::net::TcpListener::bind("127.0.0.1:0")
                .map_err(|e| AppError::WebDriver(format!("Failed to find available port: {}", e)))?;
            let port = listener.local_addr()
                .map_err(|e| AppError::WebDriver(format!("Failed to get port: {}", e)))?
                .port();
            drop(listener); // Release the port so Chrome can use it
            port
        };

        tracing::info!(
            "[Browser {}] Using unique debug port: {}",
            browser_id, debug_port
        );

        // CRITICAL: Use the builder's .user_data_dir() method, NOT .arg("--user-data-dir=...")
        // Passing as raw .arg() creates a malformed argument (the entire string becomes the key)
        // and the library also adds its own --user-data-dir, causing Chrome to ignore ours.
        config_builder = config_builder
            .user_data_dir(&user_data_dir)
            .port(debug_port);

        // Set headless mode using library's built-in method (not command-line args)
        // The library defaults to HeadlessMode::False, so we must explicitly set it
        // Using new_headless_mode() which sets --headless=new (Chrome 109+)
        if settings.headless {
            config_builder = config_builder
                .new_headless_mode()
                .no_sandbox();
            tracing::info!("Browser configured for headless mode (new headless)");
        } else {
            config_builder = config_builder.with_head();
            tracing::info!("Browser configured for headed (visible) mode");
        }

        let config = config_builder
            .build()
            .map_err(|e| AppError::WebDriver(format!("Failed to build browser config: {}", e)))?;

        // CRITICAL: Extend mutex to cover ENTIRE browser initialization
        // Chrome's "Opening in existing browser session" detection can happen at any point
        // during initialization, not just during launch. By holding the mutex until the
        // browser is FULLY ready (page created, stealth enabled), we guarantee isolation.
        // This means browsers initialize sequentially, but once ready they run in parallel.
        tracing::info!(
            "[Browser {}] [t={}ms] Waiting for browser launch mutex",
            browser_id,
            start_time.elapsed().as_millis()
        );

        let (browser, page) = {
            let _lock = BROWSER_LAUNCH_MUTEX.lock().await;
            let mutex_acquired_at = start_time.elapsed().as_millis();
            tracing::info!(
                "[Browser {}] [t={}ms] Acquired browser launch mutex, launching Chrome",
                browser_id,
                mutex_acquired_at
            );

            let launch_start = std::time::Instant::now();
            let (browser, mut handler) = Browser::launch(config)
                .await
                .map_err(|e| AppError::WebDriver(format!("Failed to launch browser: {}", e)))?;

            tracing::info!(
                "[Browser {}] [t={}ms] Browser::launch() completed in {}ms",
                browser_id,
                start_time.elapsed().as_millis(),
                launch_start.elapsed().as_millis()
            );

            // Spawn handler to process CDP events in background
            let handler_browser_id = browser_id;
            tokio::spawn(async move {
                tracing::debug!("[Browser {}] CDP event handler started", handler_browser_id);
                while handler.next().await.is_some() {}
                tracing::debug!("[Browser {}] CDP event handler ended", handler_browser_id);
            });

            // Wait for handler to start and CDP connection to stabilize
            tokio::time::sleep(Duration::from_millis(300)).await;

            // Create new page (about:blank by default)
            tracing::info!(
                "[Browser {}] [t={}ms] Creating new page",
                browser_id,
                start_time.elapsed().as_millis()
            );
            let page_start = std::time::Instant::now();
            let page = browser
                .new_page("about:blank")
                .await
                .map_err(|e| AppError::WebDriver(format!("Failed to create new page: {}", e)))?;
            tracing::info!(
                "[Browser {}] [t={}ms] Page created in {}ms",
                browser_id,
                start_time.elapsed().as_millis(),
                page_start.elapsed().as_millis()
            );

            // Enable built-in stealth mode for anti-detection
            tracing::info!(
                "[Browser {}] [t={}ms] Enabling stealth mode",
                browser_id,
                start_time.elapsed().as_millis()
            );
            let stealth_start = std::time::Instant::now();
            page.enable_stealth_mode()
                .await
                .map_err(|e| AppError::WebDriver(format!("Failed to enable stealth mode: {}", e)))?;
            tracing::info!(
                "[Browser {}] [t={}ms] Stealth mode enabled in {}ms",
                browser_id,
                start_time.elapsed().as_millis(),
                stealth_start.elapsed().as_millis()
            );

            // Final stabilization delay before releasing mutex
            tokio::time::sleep(Duration::from_millis(500)).await;
            tracing::info!(
                "[Browser {}] [t={}ms] Browser fully initialized, releasing mutex",
                browser_id,
                start_time.elapsed().as_millis()
            );

            (browser, page)
        }; // Mutex released here - browser is FULLY ready

        tracing::info!(
            "[Browser {}] [t={}ms] Browser fully initialized and ready (total: {}ms)",
            browser_id,
            start_time.elapsed().as_millis(),
            start_time.elapsed().as_millis()
        );

        Ok(Self {
            browser,
            page,
            user_data_dir: Some(user_data_dir),
            browser_id,
        })
    }

    /// Create a new WebDriver client optimized for Windows Service context.
    ///
    /// This constructor is specifically designed for running under the SYSTEM account
    /// in Windows Service mode. It:
    /// - Forces headless mode (no display available in Session 0)
    /// - Adds --disable-gpu flag (no GPU access in SYSTEM context)
    /// - Adds --no-sandbox flag (required for SYSTEM account)
    /// - Uses ProgramData-based chrome profiles for persistence
    /// - Creates one unique profile per job/worker
    ///
    /// # Arguments
    /// * `settings` - WebDriver configuration settings
    /// * `worker_id` - Unique identifier for the worker/job (used for profile isolation)
    #[cfg(all(windows, feature = "service"))]
    pub async fn new_for_service(settings: &WebDriverSettings, worker_id: &str) -> AppResult<Self> {
        // Service mode always uses headless Chrome (Session 0 has no display)
        // Visible mode is not supported in service context due to Windows session isolation
        if !settings.headless {
            tracing::warn!(
                "[Service Mode] headless=false is ignored in service mode - Chrome will run headless (Session 0 limitation)"
            );
        }

        use crate::config::paths::get_service_chrome_profiles_path;

        // Generate unique browser ID for this instance (used in all logging)
        let browser_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();

        tracing::info!(
            "[Browser {}] [Service Mode] [t=0ms] Starting browser creation for worker: {}",
            browser_id,
            worker_id
        );

        // Get Chrome path (auto-install if needed)
        let chrome_path = ensure_chrome_available(settings)?;

        // CRITICAL: Use ProgramData-based chrome profiles for service mode
        // This ensures profiles persist and are accessible by SYSTEM account
        let profiles_base = get_service_chrome_profiles_path();

        // Ensure the profiles directory exists
        std::fs::create_dir_all(&profiles_base)
            .map_err(|e| AppError::WebDriver(format!("Failed to create profiles directory: {}", e)))?;

        // Create unique profile per worker with timestamp for cleanup tracking
        let timestamp = chrono::Utc::now().timestamp();
        let user_data_dir = profiles_base.join(format!("worker-{}-{}-{}", worker_id, browser_id, timestamp));

        // Create the directory
        std::fs::create_dir_all(&user_data_dir)
            .map_err(|e| AppError::WebDriver(format!("Failed to create service profile dir: {}", e)))?;

        tracing::debug!(
            "[Browser {}] [Service Mode] Created Chrome profile: {:?}",
            browser_id,
            user_data_dir
        );

        // Build browser configuration for service context
        let mut config_builder = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .request_timeout(Duration::from_secs(30));

        // CRITICAL: Generate a unique port for this browser instance
        let debug_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")
                .map_err(|e| AppError::WebDriver(format!("Failed to find available port: {}", e)))?;
            let port = listener.local_addr()
                .map_err(|e| AppError::WebDriver(format!("Failed to get port: {}", e)))?
                .port();
            drop(listener);
            port
        };

        tracing::info!(
            "[Browser {}] [Service Mode] Using unique debug port: {}",
            browser_id, debug_port
        );

        // CRITICAL: Use the builder's .user_data_dir() method, NOT .arg("--user-data-dir=...")
        // Passing as raw .arg() creates a malformed argument (the entire string becomes the key)
        // and the library also adds its own --user-data-dir, causing Chrome to ignore ours.
        config_builder = config_builder
            .user_data_dir(&user_data_dir)
            .port(debug_port);

        // SERVICE-SPECIFIC FLAGS: Required for SYSTEM account operation
        config_builder = config_builder
            // CRITICAL: Disable GPU - SYSTEM account has no GPU access
            .arg("disable-gpu")
            // Disable software rasterizer since we're headless
            .arg("disable-software-rasterizer");

        // FORCE headless mode - service has no display
        config_builder = config_builder
            .new_headless_mode()
            .no_sandbox();
        tracing::info!(
            "[Browser {}] [Service Mode] Browser configured for headless mode (forced)",
            browser_id
        );

        let config = config_builder
            .build()
            .map_err(|e| AppError::WebDriver(format!("Failed to build browser config: {}", e)))?;

        // Serialize browser launches to prevent conflicts
        tracing::info!(
            "[Browser {}] [Service Mode] [t={}ms] Waiting for browser launch mutex",
            browser_id,
            start_time.elapsed().as_millis()
        );

        let (browser, page) = {
            let _lock = BROWSER_LAUNCH_MUTEX.lock().await;
            let mutex_acquired_at = start_time.elapsed().as_millis();
            tracing::info!(
                "[Browser {}] [Service Mode] [t={}ms] Acquired browser launch mutex, launching Chrome",
                browser_id,
                mutex_acquired_at
            );

            let launch_start = std::time::Instant::now();
            let (browser, mut handler) = Browser::launch(config)
                .await
                .map_err(|e| AppError::WebDriver(format!("Failed to launch browser in service mode: {}", e)))?;

            tracing::info!(
                "[Browser {}] [Service Mode] [t={}ms] Browser::launch() completed in {}ms",
                browser_id,
                start_time.elapsed().as_millis(),
                launch_start.elapsed().as_millis()
            );

            // Spawn handler to process CDP events in background
            let handler_browser_id = browser_id;
            tokio::spawn(async move {
                tracing::debug!("[Browser {}] [Service Mode] CDP event handler started", handler_browser_id);
                while handler.next().await.is_some() {}
                tracing::debug!("[Browser {}] [Service Mode] CDP event handler ended", handler_browser_id);
            });

            // Wait for handler to start and CDP connection to stabilize
            tokio::time::sleep(Duration::from_millis(300)).await;

            // Create new page
            tracing::info!(
                "[Browser {}] [Service Mode] [t={}ms] Creating new page",
                browser_id,
                start_time.elapsed().as_millis()
            );
            let page_start = std::time::Instant::now();
            let page = browser
                .new_page("about:blank")
                .await
                .map_err(|e| AppError::WebDriver(format!("Failed to create new page: {}", e)))?;
            tracing::info!(
                "[Browser {}] [Service Mode] [t={}ms] Page created in {}ms",
                browser_id,
                start_time.elapsed().as_millis(),
                page_start.elapsed().as_millis()
            );

            // Enable stealth mode
            tracing::info!(
                "[Browser {}] [Service Mode] [t={}ms] Enabling stealth mode",
                browser_id,
                start_time.elapsed().as_millis()
            );
            let stealth_start = std::time::Instant::now();
            page.enable_stealth_mode()
                .await
                .map_err(|e| AppError::WebDriver(format!("Failed to enable stealth mode: {}", e)))?;
            tracing::info!(
                "[Browser {}] [Service Mode] [t={}ms] Stealth mode enabled in {}ms",
                browser_id,
                start_time.elapsed().as_millis(),
                stealth_start.elapsed().as_millis()
            );

            // Final stabilization delay
            tokio::time::sleep(Duration::from_millis(500)).await;
            tracing::info!(
                "[Browser {}] [Service Mode] [t={}ms] Browser fully initialized, releasing mutex",
                browser_id,
                start_time.elapsed().as_millis()
            );

            (browser, page)
        }; // Mutex released here

        tracing::info!(
            "[Browser {}] [Service Mode] [t={}ms] Browser fully initialized and ready (total: {}ms)",
            browser_id,
            start_time.elapsed().as_millis(),
            start_time.elapsed().as_millis()
        );

        Ok(Self {
            browser,
            page,
            user_data_dir: Some(user_data_dir),
            browser_id,
        })
    }

    /// Get the unique browser ID for this instance (for logging/debugging)
    pub fn browser_id(&self) -> Uuid {
        self.browser_id
    }

    /// Navigate to a URL and wait for page load.
    pub async fn navigate(&self, url: &str) -> AppResult<()> {
        let nav_start = std::time::Instant::now();
        tracing::info!(
            "[Browser {}] Starting navigation to: {}",
            self.browser_id,
            url
        );

        self.page
            .goto(url)
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to navigate to {}: {}", url, e)))?;

        tracing::info!(
            "[Browser {}] goto() completed in {}ms, waiting for page load",
            self.browser_id,
            nav_start.elapsed().as_millis()
        );

        // WORKAROUND: Use fixed delay instead of wait_for_navigation()
        // wait_for_navigation() may have internal serialization in chaser-oxide
        // causing browsers to navigate sequentially instead of in parallel
        // Fixed delay allows parallel navigation at the cost of potential timing issues
        tokio::time::sleep(Duration::from_millis(1000)).await;

        tracing::info!(
            "[Browser {}] Navigation complete in {}ms",
            self.browser_id,
            nav_start.elapsed().as_millis()
        );

        Ok(())
    }

    /// Navigate with explicit wait_for_navigation (original behavior)
    /// Use this if you need to ensure page is fully loaded and can tolerate serialization
    #[allow(dead_code)]
    pub async fn navigate_with_wait(&self, url: &str) -> AppResult<()> {
        self.page
            .goto(url)
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to navigate to {}: {}", url, e)))?;

        self.page
            .wait_for_navigation()
            .await
            .map_err(|e| AppError::WebDriver(format!("Navigation timeout for {}: {}", url, e)))?;

        Ok(())
    }

    /// Navigate without waiting for full page load (for SPAs).
    pub async fn navigate_spa(&self, url: &str) -> AppResult<()> {
        self.page
            .goto(url)
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to navigate to {}: {}", url, e)))?;

        // Small delay for SPA to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(())
    }

    /// Find an element with retry logic for stale node errors (CDP -32000).
    ///
    /// Some JS-heavy pages re-render elements between find and interact,
    /// causing "Could not find node with given id". This retries with a
    /// short delay to let the DOM stabilize.
    async fn find_element_with_retry(
        &self,
        selector: &str,
        max_attempts: u32,
    ) -> AppResult<chaser_oxide::Element> {
        let mut last_err = None;
        for attempt in 1..=max_attempts {
            match self.page.find_element(selector).await {
                Ok(element) => return Ok(element),
                Err(e) => {
                    let err_str = format!("{}", e);
                    last_err = Some(err_str.clone());
                    if attempt < max_attempts {
                        tracing::warn!(
                            "[Browser {}] find_element('{}') failed (attempt {}/{}): {}, retrying...",
                            self.browser_id, selector, attempt, max_attempts, err_str
                        );
                        tokio::time::sleep(Duration::from_millis(300 * attempt as u64)).await;
                    }
                }
            }
        }
        Err(AppError::WebDriver(format!(
            "Element not found '{}' after {} attempts: {}",
            selector,
            max_attempts,
            last_err.unwrap_or_default()
        )))
    }

    /// Get text content of an element.
    pub async fn get_text(&self, selector: &str) -> AppResult<String> {
        let element = self
            .find_element_with_retry(selector, 3)
            .await?;

        element
            .inner_text()
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to get text: {}", e)))?
            .ok_or_else(|| AppError::WebDriver(format!("Element '{}' has no text content", selector)))
    }

    /// Click an element.
    pub async fn click(&self, selector: &str) -> AppResult<()> {
        let element = self
            .find_element_with_retry(selector, 3)
            .await?;

        element
            .scroll_into_view()
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to scroll into view: {}", e)))?;

        element
            .click()
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to click: {}", e)))?;

        Ok(())
    }

    /// Click an element with human-like behavior (delays).
    pub async fn click_human(&self, selector: &str) -> AppResult<()> {
        // Generate random delays before any await (thread_rng is not Send)
        let delay_before = {
            let mut rng = rand::thread_rng();
            rng.gen_range(100..300) // 100-300ms
        };
        let delay_after = {
            let mut rng = rand::thread_rng();
            rng.gen_range(50..200) // 50-200ms
        };

        // Small random delay before action
        tokio::time::sleep(Duration::from_millis(delay_before)).await;

        // Use regular click (chaser-oxide doesn't have click_human)
        self.click(selector).await?;

        // Small random delay after action
        tokio::time::sleep(Duration::from_millis(delay_after)).await;

        Ok(())
    }

    /// Type text into an element with human-like typing speed.
    pub async fn type_text(&self, selector: &str, text: &str) -> AppResult<()> {
        let element = self
            .find_element_with_retry(selector, 3)
            .await?;

        // Generate all random delays upfront (thread_rng is not Send)
        let delays: Vec<u64> = {
            let mut rng = rand::thread_rng();
            (0..text.len()).map(|_| rng.gen_range(20..60)).collect()
        };

        // Type with human-like delays between keystrokes
        for (i, ch) in text.chars().enumerate() {
            element
                .type_str(&ch.to_string())
                .await
                .map_err(|e| AppError::WebDriver(format!("Failed to type character: {}", e)))?;

            // Random delay between keystrokes
            if let Some(&delay_ms) = delays.get(i) {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        Ok(())
    }

    /// Click an element and type text using a single element lookup.
    ///
    /// This avoids the stale node race condition that occurs when `click()` and
    /// `type_text()` each do separate `find_element` calls — the page may re-render
    /// between the two lookups, invalidating the node ID (CDP error -32000).
    pub async fn click_and_type(&self, selector: &str, text: &str) -> AppResult<()> {
        let element = self
            .find_element_with_retry(selector, 3)
            .await?;

        element
            .scroll_into_view()
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to scroll into view: {}", e)))?;

        element
            .click()
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to click '{}': {}", selector, e)))?;

        // Small stabilization delay after click for JS frameworks to settle
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Generate all random delays upfront (thread_rng is not Send)
        let delays: Vec<u64> = {
            let mut rng = rand::thread_rng();
            (0..text.len()).map(|_| rng.gen_range(20..60)).collect()
        };

        for (i, ch) in text.chars().enumerate() {
            element
                .type_str(&ch.to_string())
                .await
                .map_err(|e| AppError::WebDriver(format!("Failed to type character: {}", e)))?;

            if let Some(&delay_ms) = delays.get(i) {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        Ok(())
    }

    /// Set an input value using JavaScript's native setter to work with React controlled inputs.
    ///
    /// React/Ant Design controlled inputs intercept the `value` property via a custom setter.
    /// CDP's `type_str` sends keystroke events, but React may not update its internal state.
    /// This method bypasses React's setter by using the browser's native
    /// `HTMLInputElement.prototype.value` setter, then dispatches `input` and `change`
    /// events so React re-syncs its state from the DOM value.
    ///
    /// Typical usage: call `click_and_type()` first (for anti-detection / natural typing),
    /// then call this method to guarantee React state is in sync.
    pub async fn set_react_input_value(&self, selector: &str, value: &str) -> AppResult<()> {
        // Escape the value for safe embedding in JavaScript
        let escaped_value = value
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n")
            .replace('\r', "\\r");
        let escaped_selector = selector.replace('\\', "\\\\").replace('\'', "\\'");

        let script = format!(
            r#"(function() {{
                var el = document.querySelector('{}');
                if (!el) return 'NOT_FOUND';
                el.focus();
                var nativeSetter = Object.getOwnPropertyDescriptor(
                    window.HTMLInputElement.prototype, 'value'
                ).set;
                nativeSetter.call(el, '{}');
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return el.value;
            }})()"#,
            escaped_selector, escaped_value
        );

        let result = self
            .execute_script(&script)
            .await
            .map_err(|e| {
                AppError::WebDriver(format!(
                    "Failed to set React input value for '{}': {}",
                    selector, e
                ))
            })?;

        // Verify the value was actually set
        let actual = result.as_str().unwrap_or("");
        if actual == "NOT_FOUND" {
            return Err(AppError::WebDriver(format!(
                "Element '{}' not found when setting React input value",
                selector
            )));
        }

        tracing::debug!(
            "[Browser {}] set_react_input_value('{}') done, value length={}",
            self.browser_id,
            selector,
            actual.len()
        );

        Ok(())
    }

    /// Wait for an element to become enabled (not disabled), with polling.
    pub async fn wait_for_element_enabled(
        &self,
        selector: &str,
        timeout_secs: u64,
    ) -> AppResult<()> {
        let start = tokio::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let escaped_selector = selector.replace('\\', "\\\\").replace('\'', "\\'");

        let script = format!(
            r#"(function() {{
                var el = document.querySelector('{}');
                if (!el) return 'NOT_FOUND';
                return el.disabled ? 'DISABLED' : 'ENABLED';
            }})()"#,
            escaped_selector
        );

        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self.execute_script(&script).await {
                Ok(val) => {
                    let status = val.as_str().unwrap_or("UNKNOWN");
                    match status {
                        "ENABLED" => {
                            tracing::info!(
                                "[Browser {}] Element '{}' is enabled (attempt #{}, {}ms)",
                                self.browser_id,
                                selector,
                                attempt,
                                start.elapsed().as_millis()
                            );
                            return Ok(());
                        }
                        "NOT_FOUND" => {
                            if start.elapsed() >= timeout {
                                return Err(AppError::WebDriver(format!(
                                    "Element '{}' not found after {} seconds",
                                    selector, timeout_secs
                                )));
                            }
                        }
                        _ => {
                            // DISABLED or unknown — keep polling
                            if start.elapsed() >= timeout {
                                return Err(AppError::WebDriver(format!(
                                    "Element '{}' still disabled after {} seconds",
                                    selector, timeout_secs
                                )));
                            }
                        }
                    }
                }
                Err(_) if start.elapsed() < timeout => {}
                Err(e) => {
                    return Err(AppError::WebDriver(format!(
                        "Failed to check enabled state of '{}': {}",
                        selector, e
                    )));
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    /// Clear an element and type new text.
    pub async fn clear_and_type(&self, selector: &str, text: &str) -> AppResult<()> {
        let element = self
            .find_element_with_retry(selector, 3)
            .await?;

        // Clear using JavaScript via call_js_fn
        element
            .call_js_fn("function() { this.value = ''; }", false)
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to clear: {}", e)))?;

        // Type the new text with human-like typing
        self.type_text(selector, text).await?;

        Ok(())
    }

    /// Wait for an element to appear (with polling).
    pub async fn wait_for_element(&self, selector: &str, timeout_secs: u64) -> AppResult<()> {
        let start = tokio::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        tracing::info!(
            "[Browser {}] [wait_for_element] Starting to wait for element '{}' (timeout: {}s)",
            self.browser_id,
            selector,
            timeout_secs
        );

        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self.page.find_element(selector).await {
                Ok(_) => {
                    let elapsed = start.elapsed();
                    tracing::info!(
                        "[Browser {}] [wait_for_element] Element '{}' found in {}ms (attempt #{})",
                        self.browser_id,
                        selector,
                        elapsed.as_millis(),
                        attempt
                    );
                    return Ok(());
                }
                Err(_) if start.elapsed() < timeout => {
                    tracing::debug!(
                        "[Browser {}] [wait_for_element] Element '{}' not found on attempt #{}, elapsed: {}ms, retrying...",
                        self.browser_id,
                        selector,
                        attempt,
                        start.elapsed().as_millis()
                    );
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                Err(_) => {
                    let elapsed = start.elapsed();
                    tracing::error!(
                        "[Browser {}] [wait_for_element] Element '{}' NOT FOUND after {}ms (timeout: {}s, total attempts: {})",
                        self.browser_id,
                        selector,
                        elapsed.as_millis(),
                        timeout_secs,
                        attempt
                    );
                    return Err(AppError::WebDriver(format!(
                        "Element '{}' not found after {} seconds (waited {}ms, {} attempts)",
                        selector, timeout_secs, elapsed.as_millis(), attempt
                    )));
                }
            }
        }
    }

    /// Wait for an element to be visible.
    pub async fn wait_for_element_visible(
        &self,
        selector: &str,
        timeout_secs: u64,
    ) -> AppResult<()> {
        self.wait_for_element(selector, timeout_secs).await?;

        // Additional visibility check via JavaScript
        let script = format!(
            r#"
            (function() {{
                var el = document.querySelector('{}');
                if (!el) return false;
                var style = window.getComputedStyle(el);
                return style.display !== 'none' &&
                       style.visibility !== 'hidden' &&
                       style.opacity !== '0';
            }})()
            "#,
            selector.replace('\'', "\\'")
        );

        let result = self
            .page
            .evaluate(script.as_str())
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to check visibility: {}", e)))?;

        let is_visible = result
            .into_value()
            .ok()
            .and_then(|v: serde_json::Value| v.as_bool())
            .unwrap_or(false);

        if !is_visible {
            return Err(AppError::WebDriver(format!(
                "Element '{}' exists but is not visible",
                selector
            )));
        }

        Ok(())
    }

    /// Execute JavaScript in the page context.
    pub async fn execute_script(&self, script: &str) -> AppResult<serde_json::Value> {
        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to execute script: {}", e)))?;

        result
            .into_value()
            .map_err(|e| AppError::WebDriver(format!("Failed to parse script result: {}", e)))
    }

    /// Take a screenshot of the page.
    pub async fn take_screenshot(&self) -> AppResult<Vec<u8>> {
        let params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(false)
            .omit_background(false)
            .build();

        self.page
            .screenshot(params)
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to take screenshot: {}", e)))
    }

    /// Get the current URL.
    pub async fn get_current_url(&self) -> AppResult<String> {
        self.page
            .url()
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to get current URL: {}", e)))?
            .ok_or_else(|| AppError::WebDriver("No URL available".into()))
    }

    /// Get the page title.
    pub async fn get_title(&self) -> AppResult<String> {
        self.page
            .get_title()
            .await
            .map_err(|e| AppError::WebDriver(format!("Failed to get title: {}", e)))?
            .ok_or_else(|| AppError::WebDriver("No title available".into()))
    }

    /// Close the browser and wait for process termination.
    pub async fn quit(mut self) -> AppResult<()> {
        tracing::info!("[Browser {}] Closing browser via CDP", self.browser_id);

        // Step 1: Send CDP close command
        let close_ok = match self.browser.close().await {
            Ok(_) => {
                tracing::info!("[Browser {}] Browser close command sent successfully", self.browser_id);
                true
            }
            Err(e) => {
                tracing::warn!(
                    "[Browser {}] Browser close failed: {}, will force-kill",
                    self.browser_id, e
                );
                false
            }
        };

        // Step 2: Wait for the Chrome process to actually exit (with timeout)
        // This is CRITICAL on Windows: without waiting, the next browser launch may
        // connect to the still-running Chrome instance instead of starting a fresh one,
        // causing it to inherit the previous session's signed-in state.
        if close_ok {
            let wait_result = tokio::time::timeout(
                Duration::from_secs(5),
                self.browser.wait()
            ).await;

            match wait_result {
                Ok(Ok(_)) => {
                    tracing::info!("[Browser {}] Chrome process exited cleanly", self.browser_id);
                }
                Ok(Err(e)) => {
                    tracing::warn!(
                        "[Browser {}] Error waiting for Chrome process: {}, will force-kill",
                        self.browser_id, e
                    );
                    if let Some(Err(ke)) = self.browser.kill().await {
                        tracing::warn!("[Browser {}] Force-kill also failed: {}", self.browser_id, ke);
                    }
                }
                Err(_) => {
                    tracing::warn!(
                        "[Browser {}] Chrome process did not exit within 5s, force-killing",
                        self.browser_id
                    );
                    if let Some(Err(ke)) = self.browser.kill().await {
                        tracing::warn!("[Browser {}] Force-kill failed: {}", self.browser_id, ke);
                    }
                }
            }
        } else {
            // close() failed, go straight to kill
            tracing::info!("[Browser {}] Force-killing Chrome process", self.browser_id);
            if let Some(Err(ke)) = self.browser.kill().await {
                tracing::warn!("[Browser {}] Force-kill failed: {}", self.browser_id, ke);
            }
        }

        // Step 3: Small grace period for Windows to release file locks on the profile dir
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Step 4: Clean up temporary user data directory
        if let Some(dir) = self.user_data_dir.take() {
            tracing::debug!("[Browser {}] Cleaning up temporary profile: {:?}", self.browser_id, dir);
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                tracing::warn!(
                    "[Browser {}] Failed to remove temporary profile directory {:?}: {}",
                    self.browser_id, dir, e
                );
            } else {
                tracing::debug!("[Browser {}] Successfully removed temporary profile", self.browser_id);
            }
        }

        Ok(())
    }
}

/// Clean up stale Chrome profiles from the service profiles directory.
///
/// This function scans the ProgramData-based chrome-profiles directory and removes
/// profiles older than 24 hours. It handles crashes and forced shutdowns gracefully
/// by checking profile modification times.
///
/// This should be called periodically (e.g., on service startup) to prevent
/// disk space accumulation from orphaned profiles.
///
/// Returns the number of profiles removed.
#[cfg(all(windows, feature = "service"))]
pub fn cleanup_stale_profiles() -> AppResult<usize> {
    use crate::config::paths::get_service_chrome_profiles_path;
    use std::time::SystemTime;

    let profiles_dir = get_service_chrome_profiles_path();

    tracing::info!(
        "[Profile Cleanup] Starting cleanup of stale Chrome profiles in {:?}",
        profiles_dir
    );

    // If the profiles directory doesn't exist, nothing to clean
    if !profiles_dir.exists() {
        tracing::debug!("[Profile Cleanup] Profiles directory does not exist, skipping");
        return Ok(0);
    }

    let now = SystemTime::now();
    let max_age = std::time::Duration::from_secs(PROFILE_MAX_AGE_SECS);
    let mut removed_count = 0;
    let mut error_count = 0;

    // Read directory entries
    let entries = std::fs::read_dir(&profiles_dir)
        .map_err(|e| AppError::WebDriver(format!("Failed to read profiles directory: {}", e)))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("[Profile Cleanup] Failed to read directory entry: {}", e);
                error_count += 1;
                continue;
            }
        };

        let path = entry.path();

        // Only process directories that look like our worker profiles
        // Format: worker-{worker_id}-{browser_id}-{timestamp}
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Check if it matches our worker profile naming pattern
        if !dir_name.starts_with("worker-") {
            tracing::debug!(
                "[Profile Cleanup] Skipping non-worker directory: {}",
                dir_name
            );
            continue;
        }

        // Check the directory's modification time
        let metadata = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    "[Profile Cleanup] Failed to get metadata for {:?}: {}",
                    path, e
                );
                error_count += 1;
                continue;
            }
        };

        let modified = match metadata.modified() {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(
                    "[Profile Cleanup] Failed to get modification time for {:?}: {}",
                    path, e
                );
                // If we can't get the modification time, try to extract timestamp from name
                // Format: worker-{worker_id}-{browser_id}-{timestamp}
                if let Some(timestamp_str) = dir_name.rsplit('-').next() {
                    if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                        let profile_time = chrono::DateTime::from_timestamp(timestamp, 0)
                            .map(|dt| SystemTime::from(dt));
                        if let Some(t) = profile_time {
                            t
                        } else {
                            error_count += 1;
                            continue;
                        }
                    } else {
                        error_count += 1;
                        continue;
                    }
                } else {
                    error_count += 1;
                    continue;
                }
            }
        };

        // Calculate age of the profile
        let age = match now.duration_since(modified) {
            Ok(d) => d,
            Err(_) => {
                // Profile has a future timestamp, skip it
                tracing::debug!(
                    "[Profile Cleanup] Profile {:?} has future timestamp, skipping",
                    path
                );
                continue;
            }
        };

        // Remove if older than max age
        if age > max_age {
            tracing::info!(
                "[Profile Cleanup] Removing stale profile {:?} (age: {} hours)",
                path,
                age.as_secs() / 3600
            );

            match std::fs::remove_dir_all(&path) {
                Ok(_) => {
                    removed_count += 1;
                    tracing::debug!(
                        "[Profile Cleanup] Successfully removed stale profile: {:?}",
                        path
                    );
                }
                Err(e) => {
                    // Handle case where browser might still be using it
                    tracing::warn!(
                        "[Profile Cleanup] Failed to remove profile {:?}: {} (may still be in use)",
                        path, e
                    );
                    error_count += 1;
                }
            }
        } else {
            tracing::debug!(
                "[Profile Cleanup] Profile {:?} is recent (age: {} min), keeping",
                path,
                age.as_secs() / 60
            );
        }
    }

    tracing::info!(
        "[Profile Cleanup] Cleanup complete. Removed: {}, Errors: {}",
        removed_count,
        error_count
    );

    Ok(removed_count)
}

/// Async wrapper for cleanup_stale_profiles that runs in a blocking task.
///
/// Use this when calling from async code to avoid blocking the async runtime.
#[cfg(all(windows, feature = "service"))]
pub async fn cleanup_stale_profiles_async() -> AppResult<usize> {
    tokio::task::spawn_blocking(|| cleanup_stale_profiles())
        .await
        .map_err(|e| AppError::WebDriver(format!("Profile cleanup task panicked: {}", e)))?
}
