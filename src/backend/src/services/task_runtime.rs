use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// Global registry for managing task cancellation tokens
static TASK_REGISTRY: OnceLock<RwLock<HashMap<i64, CancellationToken>>> = OnceLock::new();

fn registry() -> &'static RwLock<HashMap<i64, CancellationToken>> {
    TASK_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Register a new task and return its cancellation token
pub async fn register(task_id: i64) -> CancellationToken {
    let token = CancellationToken::new();
    let mut registry = registry().write().await;
    registry.insert(task_id, token.clone());
    tracing::debug!("[task_runtime] Registered task_id={}", task_id);
    token
}

/// Cancel a running task by its ID
/// Returns true if the task was found and cancelled, false otherwise
pub async fn cancel(task_id: i64) -> bool {
    let registry = registry().read().await;
    if let Some(token) = registry.get(&task_id) {
        token.cancel();
        tracing::info!("[task_runtime] Cancelled task_id={}", task_id);
        true
    } else {
        tracing::warn!("[task_runtime] Task not found in registry: task_id={}", task_id);
        false
    }
}

/// Remove a task from the registry (cleanup after execution)
pub async fn remove(task_id: i64) {
    let mut registry = registry().write().await;
    registry.remove(&task_id);
    tracing::debug!("[task_runtime] Removed task_id={}", task_id);
}

/// Check if a task is currently registered (running)
pub async fn is_running(task_id: i64) -> bool {
    let registry = registry().read().await;
    registry.contains_key(&task_id)
}
