use leptos::prelude::*;
use crate::components::layout::AppShell;
use crate::server_fns::lines::get_lines;
use crate::server_fns::operations::{trigger_quota_check, trigger_speed_test, get_execution_status};

#[derive(Clone, Debug, PartialEq)]
enum OpState {
    Idle,
    Running(i64), // execution_id
    Done(String), // status message
    Error(String),
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let lines_resource = Resource::new(
        || (),
        |_| async { get_lines(None, None).await },
    );

    view! {
        <AppShell>
            <div class="max-w-6xl mx-auto">
                <DefaultPasswordWarning/>
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-xl font-semibold text-slate-100">"Dashboard"</h2>
                </div>

                <Suspense fallback=|| view! { <p class="text-slate-400">"Loading lines…"</p> }>
                    {move || lines_resource.get().map(|result| match result {
                        Ok(json) => {
                            let lines: Vec<serde_json::Value> = serde_json::from_str::<serde_json::Value>(&json)
                                .ok()
                                .and_then(|v| v.get("items").and_then(|i| serde_json::from_value(i.clone()).ok()))
                                .unwrap_or_default();
                            let is_empty = lines.is_empty();
                            view! {
                                <div class="space-y-4">
                                    {lines.into_iter().map(|line| {
                                        let id = line.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                                        let name = line.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let isp = line.get("isp").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                        let is_active = line.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false);
                                        view! { <LineCard id=id name=name isp=isp is_active=is_active/> }
                                    }).collect::<Vec<_>>()}
                                    {if is_empty {
                                        view! {
                                            <div class="bg-slate-900 border border-slate-700 rounded-xl p-8 text-center">
                                                <p class="text-slate-400">"No lines configured yet."</p>
                                                <a href="/lines" class="mt-2 inline-block text-indigo-400 hover:text-indigo-300 text-sm">
                                                    "Go to Lines to add one"
                                                </a>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div/> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <p class="text-red-400">"Failed to load lines."</p>
                        }.into_any(),
                    })}
                </Suspense>
            </div>
        </AppShell>
    }
}

/// Warning banner shown when the admin is still using the default password.
#[component]
fn DefaultPasswordWarning() -> impl IntoView {
    let default_pw_check = Resource::new(
        || (),
        |_| async { check_default_password().await },
    );

    view! {
        <Suspense fallback=|| ()>
            {move || default_pw_check.get().map(|r| {
                if r.unwrap_or(false) {
                    view! {
                        <div class="mb-4 flex items-center gap-3 px-4 py-3 bg-amber-900/40 border border-amber-700 rounded-xl text-amber-300 text-sm">
                            <span class="text-amber-400 font-bold">"⚠"</span>
                            <span>
                                "You are using the default password. "
                                <a href="/change-password" class="underline hover:text-amber-100">"Change it now"</a>
                                " to secure your account."
                            </span>
                        </div>
                    }.into_any()
                } else {
                    view! { <div/> }.into_any()
                }
            })}
        </Suspense>
    }
}

/// Server function: returns true if the admin is still on the default "admin" password.
#[server]
async fn check_default_password() -> Result<bool, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;
    use crate::startup::verify_password;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Ok(false);
    };
    let Some(pool) = state.pool.clone() else {
        return Ok(false);
    };

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT password_hash FROM web_admin_credentials LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    Ok(row.map(|(hash,)| verify_password("admin", &hash)).unwrap_or(false))
}

/// Per-line card with quota check and speed test trigger buttons.
#[component]
fn LineCard(id: i64, name: String, isp: String, is_active: bool) -> impl IntoView {
    let quota_state: RwSignal<OpState> = RwSignal::new(OpState::Idle);
    let speed_state: RwSignal<OpState> = RwSignal::new(OpState::Idle);

    // Polling interval handle (stored as signal for cleanup)
    let quota_poll = RwSignal::new(Option::<leptos::prelude::IntervalHandle>::None);
    let speed_poll = RwSignal::new(Option::<leptos::prelude::IntervalHandle>::None);

    let start_polling = move |exec_id: i64, state_sig: RwSignal<OpState>, poll_sig: RwSignal<Option<leptos::prelude::IntervalHandle>>| {
        let handle = leptos::prelude::set_interval_with_handle(
            move || {
                leptos::task::spawn_local(async move {
                    match get_execution_status(exec_id).await {
                        Ok(json) => {
                            let status = serde_json::from_str::<serde_json::Value>(&json)
                                .ok()
                                .and_then(|v| v.get("status").and_then(|s| s.as_str()).map(|s| s.to_string()))
                                .unwrap_or_default();
                            if status == "completed" || status == "failed" {
                                if status == "failed" {
                                    let err = serde_json::from_str::<serde_json::Value>(&json)
                                        .ok()
                                        .and_then(|v| v.get("error_message").and_then(|e| e.as_str()).map(|e| e.to_string()))
                                        .unwrap_or_else(|| "Unknown error".to_string());
                                    state_sig.set(OpState::Error(err));
                                } else {
                                    state_sig.set(OpState::Done("Completed".to_string()));
                                }
                                // Clear poll handle
                                if let Some(h) = poll_sig.get_untracked() {
                                    h.clear();
                                }
                                poll_sig.set(None);
                            }
                        }
                        Err(_) => {
                            state_sig.set(OpState::Error("Status check failed".to_string()));
                            if let Some(h) = poll_sig.get_untracked() {
                                h.clear();
                            }
                            poll_sig.set(None);
                        }
                    }
                });
            },
            std::time::Duration::from_secs(3),
        );
        if let Ok(h) = handle {
            poll_sig.set(Some(h));
        }
    };

    let trigger_quota = move |_| {
        let id = id;
        quota_state.set(OpState::Running(0));
        leptos::task::spawn_local(async move {
            match trigger_quota_check(id).await {
                Ok(exec_id) => {
                    quota_state.set(OpState::Running(exec_id));
                    start_polling(exec_id, quota_state, quota_poll);
                }
                Err(ServerFnError::ServerError(ref msg)) if msg == "busy" => {
                    quota_state.set(OpState::Error("Already running".to_string()));
                }
                Err(e) => {
                    quota_state.set(OpState::Error(e.to_string()));
                }
            }
        });
    };

    let trigger_speed = move |_| {
        let id = id;
        speed_state.set(OpState::Running(0));
        leptos::task::spawn_local(async move {
            match trigger_speed_test(id).await {
                Ok(exec_id) => {
                    speed_state.set(OpState::Running(exec_id));
                    start_polling(exec_id, speed_state, speed_poll);
                }
                Err(ServerFnError::ServerError(ref msg)) if msg == "busy" => {
                    speed_state.set(OpState::Error("Already running".to_string()));
                }
                Err(e) => {
                    speed_state.set(OpState::Error(e.to_string()));
                }
            }
        });
    };

    view! {
        <div class="bg-slate-900 border border-slate-700 rounded-xl p-4">
            <div class="flex items-start justify-between">
                <div>
                    <div class="flex items-center gap-2">
                        <h3 class="text-sm font-semibold text-slate-100">{name}</h3>
                        {if is_active {
                            view! { <span class="px-2 py-0.5 bg-green-900/50 text-green-400 text-xs rounded-full">"Active"</span> }.into_any()
                        } else {
                            view! { <span class="px-2 py-0.5 bg-slate-700 text-slate-400 text-xs rounded-full">"Inactive"</span> }.into_any()
                        }}
                    </div>
                    <p class="text-xs text-slate-500 mt-0.5">{isp}</p>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0 ml-4">
                    // Quota check button + status
                    <div class="flex items-center gap-2">
                        {move || match quota_state.get() {
                            OpState::Running(_) => view! {
                                <span class="text-xs text-slate-400 flex items-center gap-1">
                                    <span class="animate-spin inline-block w-3 h-3 border border-slate-400 border-t-indigo-400 rounded-full"/>
                                    "Checking…"
                                </span>
                            }.into_any(),
                            OpState::Done(msg) => view! {
                                <span class="text-xs text-green-400">{msg}</span>
                            }.into_any(),
                            OpState::Error(e) => view! {
                                <span class="text-xs text-red-400" title={e.clone()}>"Failed"</span>
                            }.into_any(),
                            OpState::Idle => view! { <span/> }.into_any(),
                        }}
                        <button
                            class="px-3 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white rounded-md transition-colors"
                            disabled=move || matches!(quota_state.get(), OpState::Running(_))
                            on:click=trigger_quota
                        >
                            "Quota Check"
                        </button>
                    </div>
                    // Speed test button + status
                    <div class="flex items-center gap-2">
                        {move || match speed_state.get() {
                            OpState::Running(_) => view! {
                                <span class="text-xs text-slate-400 flex items-center gap-1">
                                    <span class="animate-spin inline-block w-3 h-3 border border-slate-400 border-t-indigo-400 rounded-full"/>
                                    "Testing…"
                                </span>
                            }.into_any(),
                            OpState::Done(msg) => view! {
                                <span class="text-xs text-green-400">{msg}</span>
                            }.into_any(),
                            OpState::Error(e) => view! {
                                <span class="text-xs text-red-400" title={e.clone()}>"Failed"</span>
                            }.into_any(),
                            OpState::Idle => view! { <span/> }.into_any(),
                        }}
                        <button
                            class="px-3 py-1.5 text-xs bg-slate-700 hover:bg-slate-600 disabled:opacity-50 text-slate-300 rounded-md transition-colors"
                            disabled=move || matches!(speed_state.get(), OpState::Running(_))
                            on:click=trigger_speed
                        >
                            "Speed Test"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
