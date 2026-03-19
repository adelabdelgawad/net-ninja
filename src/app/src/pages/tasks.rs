use leptos::prelude::*;
use crate::components::layout::AppShell;
use crate::components::ui::dialog::ConfirmDialog;
use crate::server_fns::tasks::{get_tasks, delete_task, trigger_task, get_task_executions};
use crate::server_fns::operations::get_execution_status;

#[component]
pub fn TasksPage() -> impl IntoView {
    let tasks_resource = Resource::new(|| (), |_| async { get_tasks().await });
    let delete_id = RwSignal::new(Option::<i64>::None);

    let delete_action = Action::new(move |id: &i64| {
        let id = *id;
        async move { delete_task(id).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            delete_id.set(None);
            tasks_resource.refetch();
        }
    });

    view! {
        <AppShell>
            <div class="max-w-6xl mx-auto">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-xl font-semibold text-slate-100">"Scheduled Tasks"</h2>
                </div>

                <Suspense fallback=|| view! { <p class="text-slate-400">"Loading…"</p> }>
                    {move || tasks_resource.get().map(|result| match result {
                        Ok(json) => {
                            let tasks: Vec<serde_json::Value> = serde_json::from_str(&json)
                                .unwrap_or_default();
                            let is_empty = tasks.is_empty();
                            view! {
                                <div class="space-y-4">
                                    {tasks.into_iter().map(|task| {
                                        let id = task.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                                        let name = task.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let run_mode = task.get("runMode").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let is_active = task.get("isActive").and_then(|v| v.as_bool()).unwrap_or(false);
                                        let status = task.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let task_types: Vec<String> = task.get("taskTypes")
                                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                                            .unwrap_or_default();
                                        let task_types_str = task_types.join(", ");
                                        let lines_count = task.get("lineIds").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);

                                        view! {
                                            <TaskCard
                                                id=id
                                                name=name
                                                run_mode=run_mode
                                                is_active=is_active
                                                status=status
                                                task_types_str=task_types_str
                                                lines_count=lines_count
                                                on_delete=move || delete_id.set(Some(id))
                                                on_refresh=move || tasks_resource.refetch()
                                            />
                                        }
                                    }).collect::<Vec<_>>()}
                                    {if is_empty {
                                        view! {
                                            <div class="bg-slate-900 border border-slate-700 rounded-xl p-8 text-center">
                                                <p class="text-slate-400">"No tasks configured yet."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div/> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <p class="text-red-400">"Failed to load tasks."</p>
                        }.into_any(),
                    })}
                </Suspense>
            </div>

            <ConfirmDialog
                title="Delete Task".to_string()
                message="Are you sure you want to delete this task? All execution history will also be deleted.".to_string()
                open=Signal::derive(move || delete_id.get().is_some())
                on_confirm=Callback::new(move |_| {
                    if let Some(id) = delete_id.get_untracked() {
                        delete_action.dispatch(id);
                    }
                })
                on_cancel=Callback::new(move |_| delete_id.set(None))
            />
        </AppShell>
    }
}

#[component]
fn TaskCard(
    id: i64,
    name: String,
    run_mode: String,
    is_active: bool,
    status: String,
    task_types_str: String,
    lines_count: usize,
    on_delete: impl Fn() + 'static,
    #[allow(unused_variables)]
    on_refresh: impl Fn() + 'static,
) -> impl IntoView {
    let show_history = RwSignal::new(false);
    let trigger_state = RwSignal::new(Option::<String>::None); // None=idle, Some("running")/Some("done")/Some("error:...")

    let history_resource = Resource::new(
        move || (show_history.get(), id),
        move |(show, tid)| async move {
            if show {
                get_task_executions(tid).await.ok()
            } else {
                None
            }
        },
    );

    let trigger_poll_handle: RwSignal<Option<leptos::prelude::IntervalHandle>> = RwSignal::new(None);

    let trigger_action = move |_| {
        trigger_state.set(Some("running".to_string()));
        leptos::task::spawn_local(async move {
            match trigger_task(id).await {
                Ok(exec_id) => {
                    let handle = leptos::prelude::set_interval_with_handle(
                        move || {
                            leptos::task::spawn_local(async move {
                                match get_execution_status(exec_id).await {
                                    Ok(json) => {
                                        let v = serde_json::from_str::<serde_json::Value>(&json).ok();
                                        let s = v.as_ref().and_then(|v| v.get("status")).and_then(|s| s.as_str()).unwrap_or("").to_string();
                                        if s == "completed" {
                                            trigger_state.set(Some("done".to_string()));
                                            if let Some(h) = trigger_poll_handle.get_untracked() { h.clear(); }
                                            trigger_poll_handle.set(None);
                                        } else if s == "failed" {
                                            let err = v.as_ref().and_then(|v| v.get("error_message")).and_then(|e| e.as_str()).unwrap_or("Unknown error").to_string();
                                            trigger_state.set(Some(format!("error:{err}")));
                                            if let Some(h) = trigger_poll_handle.get_untracked() { h.clear(); }
                                            trigger_poll_handle.set(None);
                                        }
                                    }
                                    Err(_) => {
                                        trigger_state.set(Some("error:Status check failed".to_string()));
                                        if let Some(h) = trigger_poll_handle.get_untracked() { h.clear(); }
                                        trigger_poll_handle.set(None);
                                    }
                                }
                            });
                        },
                        std::time::Duration::from_secs(3),
                    );
                    if let Ok(h) = handle {
                        trigger_poll_handle.set(Some(h));
                    }
                }
                Err(e) => {
                    trigger_state.set(Some(format!("error:{e}")));
                }
            }
        });
    };

    view! {
        <div class="bg-slate-900 border border-slate-700 rounded-xl overflow-hidden">
            <div class="p-4 flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                        <h3 class="text-sm font-semibold text-slate-100 truncate">{name}</h3>
                        {if is_active {
                            view! { <span class="px-2 py-0.5 bg-green-900/50 text-green-400 text-xs rounded-full">"Active"</span> }.into_any()
                        } else {
                            view! { <span class="px-2 py-0.5 bg-slate-700 text-slate-400 text-xs rounded-full">"Inactive"</span> }.into_any()
                        }}
                        <span class="text-xs text-slate-500">{run_mode}</span>
                    </div>
                    <p class="text-xs text-slate-500 mt-1">
                        {format!("{task_types_str} · {lines_count} line(s) · {status}")}
                    </p>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                    // Trigger status
                    {move || {
                        let s = trigger_state.get();
                        match s.as_deref() {
                            Some("running") => view! {
                                <span class="text-xs text-slate-400 flex items-center gap-1">
                                    <span class="animate-spin inline-block w-3 h-3 border border-slate-400 border-t-indigo-400 rounded-full"/>
                                    "Running…"
                                </span>
                            }.into_any(),
                            Some("done") => view! {
                                <span class="text-xs text-green-400">"Completed"</span>
                            }.into_any(),
                            Some(e) if e.starts_with("error:") => view! {
                                <span class="text-xs text-red-400">"Failed"</span>
                            }.into_any(),
                            _ => view! { <span/> }.into_any(),
                        }
                    }}
                    <button
                        class="px-3 py-1 text-xs bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white rounded transition-colors"
                        disabled=move || trigger_state.get().as_deref() == Some("running")
                        on:click=trigger_action
                    >"Run Now"</button>
                    <button
                        class="px-3 py-1 text-xs bg-slate-700 hover:bg-slate-600 text-slate-300 rounded transition-colors"
                        on:click=move |_| show_history.update(|v| *v = !*v)
                    >"History"</button>
                    <button
                        class="px-3 py-1 text-xs bg-red-900/50 hover:bg-red-800/70 text-red-400 rounded transition-colors"
                        on:click=move |_| on_delete()
                    >"Delete"</button>
                </div>
            </div>

            // Execution history panel
            <Show when=move || show_history.get()>
                <div class="border-t border-slate-700 bg-slate-950/50">
                    <Suspense fallback=|| view! { <p class="text-slate-400 text-xs p-3">"Loading history…"</p> }>
                        {move || history_resource.get().map(|opt| {
                            let executions: Vec<serde_json::Value> = opt
                                .and_then(|json| serde_json::from_str(&json).ok())
                                .unwrap_or_default();
                            let is_empty = executions.is_empty();
                            view! {
                                <div class="divide-y divide-slate-800">
                                    {executions.into_iter().map(|ex| {
                                        let status = ex.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let started = ex.get("startedAt").and_then(|v| v.as_str()).unwrap_or("").chars().take(19).collect::<String>();
                                        let duration = ex.get("durationMs").and_then(|v| v.as_i64()).map(|ms| format!("{:.1}s", ms as f64 / 1000.0)).unwrap_or_else(|| "—".to_string());
                                        let err = ex.get("errorMessage").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let status_color = match status.as_str() {
                                            "completed" => "text-green-400",
                                            "failed" => "text-red-400",
                                            "running" => "text-indigo-400",
                                            _ => "text-slate-400",
                                        };
                                        view! {
                                            <div class="flex items-center gap-3 px-4 py-2 text-xs">
                                                <span class={format!("font-medium {status_color}")}>{status}</span>
                                                <span class="text-slate-500">{started}</span>
                                                <span class="text-slate-500">{duration}</span>
                                                {if !err.is_empty() {
                                                    view! { <span class="text-red-400 truncate">{err}</span> }.into_any()
                                                } else {
                                                    view! { <span/> }.into_any()
                                                }}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                    {if is_empty {
                                        view! { <p class="text-slate-500 text-xs p-3">"No executions yet."</p> }.into_any()
                                    } else {
                                        view! { <div/> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </div>
            </Show>
        </div>
    }
}
