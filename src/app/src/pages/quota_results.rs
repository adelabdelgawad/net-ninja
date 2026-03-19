use leptos::prelude::*;
use crate::components::layout::AppShell;
use crate::server_fns::history::get_quota_results;

#[component]
pub fn QuotaResultsPage() -> impl IntoView {
    let page = RwSignal::new(1u32);
    let line_filter = RwSignal::new(Option::<i32>::None);

    let results = Resource::new(
        move || (line_filter.get(), page.get()),
        move |(lid, p)| async move { get_quota_results(lid, Some(p)).await },
    );

    view! {
        <AppShell>
            <div class="max-w-6xl mx-auto">
                <h2 class="text-xl font-semibold text-slate-100 mb-6">"Quota Results"</h2>
                <Suspense fallback=|| view! { <p class="text-slate-400">"Loading…"</p> }>
                    {move || results.get().map(|r| match r {
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
                                                <th class="px-4 py-3 text-left font-medium">"Line ID"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Date"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Used GB"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Total GB"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Remaining GB"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Usage %"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Status"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-800">
                                            {items.into_iter().map(|item| {
                                                let line_id = item.get("line_id").and_then(|v| v.as_i64()).unwrap_or(0);
                                                let created = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").chars().take(10).collect::<String>();
                                                let used = item.get("used_quota").and_then(|v| v.as_f64()).map(|v| format!("{:.1}", v)).unwrap_or("—".into());
                                                let total = item.get("total_quota").and_then(|v| v.as_f64()).map(|v| format!("{:.1}", v)).unwrap_or("—".into());
                                                let remaining = item.get("remaining_quota").and_then(|v| v.as_f64()).map(|v| format!("{:.1}", v)).unwrap_or("—".into());
                                                let pct = item.get("quota_percentage").and_then(|v| v.as_f64()).map(|v| format!("{:.1}%", v)).unwrap_or("—".into());
                                                let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                view! {
                                                    <tr class="text-slate-300 hover:bg-slate-800/50">
                                                        <td class="px-4 py-3">{line_id}</td>
                                                        <td class="px-4 py-3 text-slate-400">{created}</td>
                                                        <td class="px-4 py-3">{used}</td>
                                                        <td class="px-4 py-3">{total}</td>
                                                        <td class="px-4 py-3">{remaining}</td>
                                                        <td class="px-4 py-3">{pct}</td>
                                                        <td class="px-4 py-3 text-slate-400">{status}</td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                    {if is_empty {
                                        view! { <p class="text-center text-slate-500 py-8">"No quota results yet."</p> }.into_any()
                                    } else {
                                        view! { <div/> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! { <p class="text-red-400">"Failed to load quota results."</p> }.into_any(),
                    })}
                </Suspense>
            </div>
        </AppShell>
    }
}
