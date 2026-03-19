use leptos::prelude::*;
use crate::components::layout::AppShell;
use crate::server_fns::history::get_speed_results;

#[component]
pub fn SpeedResultsPage() -> impl IntoView {
    let page = RwSignal::new(1u32);
    let line_filter = RwSignal::new(Option::<i32>::None);

    let results = Resource::new(
        move || (line_filter.get(), page.get()),
        move |(lid, p)| async move { get_speed_results(lid, Some(p)).await },
    );

    view! {
        <AppShell>
            <div class="max-w-6xl mx-auto">
                <h2 class="text-xl font-semibold text-slate-100 mb-6">"Speed Test Results"</h2>
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
                                                <th class="px-4 py-3 text-left font-medium">"Download Mbps"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Upload Mbps"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Ping ms"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Server"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Status"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-800">
                                            {items.into_iter().map(|item| {
                                                let line_id = item.get("line_id").and_then(|v| v.as_i64()).unwrap_or(0);
                                                let created = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").chars().take(10).collect::<String>();
                                                let dl = item.get("download_speed").and_then(|v| v.as_f64()).map(|v| format!("{:.1}", v)).unwrap_or("—".into());
                                                let ul = item.get("upload_speed").and_then(|v| v.as_f64()).map(|v| format!("{:.1}", v)).unwrap_or("—".into());
                                                let ping = item.get("ping").and_then(|v| v.as_f64()).map(|v| format!("{:.0}", v)).unwrap_or("—".into());
                                                let server = item.get("server_name").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                view! {
                                                    <tr class="text-slate-300 hover:bg-slate-800/50">
                                                        <td class="px-4 py-3">{line_id}</td>
                                                        <td class="px-4 py-3 text-slate-400">{created}</td>
                                                        <td class="px-4 py-3">{dl}</td>
                                                        <td class="px-4 py-3">{ul}</td>
                                                        <td class="px-4 py-3">{ping}</td>
                                                        <td class="px-4 py-3 text-slate-400">{server}</td>
                                                        <td class="px-4 py-3 text-slate-400">{status}</td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                    {if is_empty {
                                        view! { <p class="text-center text-slate-500 py-8">"No speed test results yet."</p> }.into_any()
                                    } else {
                                        view! { <div/> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! { <p class="text-red-400">"Failed to load speed results."</p> }.into_any(),
                    })}
                </Suspense>
            </div>
        </AppShell>
    }
}
