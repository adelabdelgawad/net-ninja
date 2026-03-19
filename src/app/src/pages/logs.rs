use leptos::prelude::*;
use crate::components::layout::AppShell;
use crate::server_fns::history::get_logs;

#[component]
pub fn LogsPage() -> impl IntoView {
    let page = RwSignal::new(1u32);

    let logs = Resource::new(
        move || page.get(),
        move |p| async move { get_logs(None, Some(p)).await },
    );

    view! {
        <AppShell>
            <div class="max-w-6xl mx-auto">
                <h2 class="text-xl font-semibold text-slate-100 mb-6">"Operation Logs"</h2>
                <Suspense fallback=|| view! { <p class="text-slate-400">"Loading…"</p> }>
                    {move || logs.get().map(|r| match r {
                        Ok(json) => {
                            let items: Vec<serde_json::Value> = serde_json::from_str::<serde_json::Value>(&json)
                                .ok()
                                .and_then(|v| v.get("items").and_then(|i| serde_json::from_value(i.clone()).ok()))
                                .unwrap_or_default();
                            let is_empty = items.is_empty();
                            view! {
                                <div class="bg-slate-900 border border-slate-700 rounded-xl overflow-hidden">
                                    <table class="w-full text-sm">
                                        <thead class="bg-slate-800 text-slate-400">
                                            <tr>
                                                <th class="px-4 py-3 text-left font-medium">"Time"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Level"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Function"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Line"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Message"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-800">
                                            {items.into_iter().map(|item| {
                                                let created = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").chars().take(19).collect::<String>();
                                                let level = item.get("level").and_then(|v| v.as_str()).unwrap_or("Info").to_string();
                                                let func = item.get("function_name").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                let line_id = item.get("line_id").and_then(|v| v.as_i64()).map(|v| v.to_string()).unwrap_or("—".into());
                                                let message = item.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let level_class = match level.as_str() {
                                                    "Error" => "text-red-400",
                                                    "Warning" => "text-yellow-400",
                                                    "Debug" => "text-slate-500",
                                                    _ => "text-blue-400",
                                                };
                                                view! {
                                                    <tr class="text-slate-300 hover:bg-slate-800/50">
                                                        <td class="px-4 py-2 text-slate-400 font-mono text-xs whitespace-nowrap">{created}</td>
                                                        <td class="px-4 py-2">
                                                            <span class=format!("text-xs font-medium {level_class}")>{level}</span>
                                                        </td>
                                                        <td class="px-4 py-2 text-slate-400 text-xs">{func}</td>
                                                        <td class="px-4 py-2 text-slate-400 text-xs">{line_id}</td>
                                                        <td class="px-4 py-2 text-xs">{message}</td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                    {if is_empty {
                                        view! { <p class="text-center text-slate-500 py-8">"No logs yet."</p> }.into_any()
                                    } else {
                                        view! { <div/> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! { <p class="text-red-400">"Failed to load logs."</p> }.into_any(),
                    })}
                </Suspense>
            </div>
        </AppShell>
    }
}
