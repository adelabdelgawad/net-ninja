use leptos::prelude::*;
use crate::components::layout::AppShell;
use crate::components::ui::dialog::ConfirmDialog;
use crate::server_fns::lines::{get_lines, create_line, update_line, delete_line};

#[derive(Clone, Default)]
struct LineForm {
    id: Option<i32>,
    name: String,
    line_number: String,
    username: String,
    password: String,
    ip_address: String,
    isp: String,
    description: String,
    is_active: bool,
}

#[component]
pub fn LinesPage() -> impl IntoView {
    let lines_resource = Resource::new(
        || (),
        |_| async { get_lines(None, None).await },
    );

    let form = RwSignal::new(LineForm::default());
    let show_form = RwSignal::new(false);
    let delete_id = RwSignal::new(Option::<i32>::None);
    let form_error = RwSignal::new(Option::<String>::None);

    let submit_action = Action::new(move |f: &LineForm| {
        let f = f.clone();
        async move {
            if let Some(id) = f.id {
                update_line(
                    id,
                    Some(f.name), Some(f.line_number), Some(f.username),
                    Some(f.password),
                    if f.ip_address.is_empty() { None } else { Some(f.ip_address) },
                    if f.isp.is_empty() { None } else { Some(f.isp) },
                    if f.description.is_empty() { None } else { Some(f.description) },
                    None, Some(f.is_active),
                ).await
            } else {
                create_line(
                    f.name, f.line_number, f.username, f.password,
                    if f.ip_address.is_empty() { None } else { Some(f.ip_address) },
                    if f.isp.is_empty() { None } else { Some(f.isp) },
                    if f.description.is_empty() { None } else { Some(f.description) },
                    None, Some(f.is_active),
                ).await
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = submit_action.value().get() {
            show_form.set(false);
            form.set(LineForm::default());
            form_error.set(None);
            lines_resource.refetch();
        }
        if let Some(Err(ServerFnError::ServerError(ref msg))) = submit_action.value().get() {
            form_error.set(Some(msg.clone()));
        }
    });

    let delete_action = Action::new(move |id: &i32| {
        let id = *id;
        async move { delete_line(id).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            delete_id.set(None);
            lines_resource.refetch();
        }
    });

    view! {
        <AppShell>
            <div class="max-w-6xl mx-auto">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-xl font-semibold text-slate-100">"Internet Lines"</h2>
                    <button
                        class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                        on:click=move |_| {
                            form.set(LineForm { is_active: true, ..Default::default() });
                            form_error.set(None);
                            show_form.set(true);
                        }
                    >
                        "+ Add Line"
                    </button>
                </div>

                <Suspense fallback=|| view! { <p class="text-slate-400">"Loading…"</p> }>
                    {move || lines_resource.get().map(|result| match result {
                        Ok(json) => {
                            let lines: Vec<serde_json::Value> = serde_json::from_str::<serde_json::Value>(&json)
                                .ok()
                                .and_then(|v| v.get("items").and_then(|i| serde_json::from_value(i.clone()).ok()))
                                .unwrap_or_default();

                            let is_empty = lines.is_empty();
                            view! {
                                <div class="bg-slate-900 border border-slate-700 rounded-xl overflow-hidden">
                                    <table class="w-full text-sm">
                                        <thead class="bg-slate-800 text-slate-400">
                                            <tr>
                                                <th class="px-4 py-3 text-left font-medium">"Name"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Line #"</th>
                                                <th class="px-4 py-3 text-left font-medium">"ISP"</th>
                                                <th class="px-4 py-3 text-left font-medium">"IP Address"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Status"</th>
                                                <th class="px-4 py-3 text-right font-medium">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-800">
                                            {lines.into_iter().map(|line| {
                                                let id = line.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                                let name = line.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let line_number = line.get("line_number").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let isp = line.get("isp").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                let ip = line.get("ip_address").and_then(|v| v.as_str()).unwrap_or("—").to_string();
                                                let is_active = line.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false);
                                                let username = line.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let description = line.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();

                                                let edit_form = LineForm {
                                                    id: Some(id),
                                                    name: name.clone(),
                                                    line_number: line_number.clone(),
                                                    username,
                                                    password: String::new(),
                                                    ip_address: ip.clone(),
                                                    isp: isp.clone(),
                                                    description,
                                                    is_active,
                                                };

                                                view! {
                                                    <tr class="text-slate-300 hover:bg-slate-800/50 transition-colors">
                                                        <td class="px-4 py-3 font-medium">{name}</td>
                                                        <td class="px-4 py-3 text-slate-400">{line_number}</td>
                                                        <td class="px-4 py-3 text-slate-400">{isp}</td>
                                                        <td class="px-4 py-3 text-slate-400">{ip}</td>
                                                        <td class="px-4 py-3">
                                                            {if is_active {
                                                                view! { <span class="px-2 py-0.5 bg-green-900/50 text-green-400 text-xs rounded-full">"Active"</span> }.into_any()
                                                            } else {
                                                                view! { <span class="px-2 py-0.5 bg-slate-700 text-slate-400 text-xs rounded-full">"Inactive"</span> }.into_any()
                                                            }}
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            <div class="flex justify-end gap-2">
                                                                <button
                                                                    class="px-3 py-1 text-xs bg-slate-700 hover:bg-slate-600 text-slate-300 rounded transition-colors"
                                                                    on:click=move |_| {
                                                                        form.set(edit_form.clone());
                                                                        form_error.set(None);
                                                                        show_form.set(true);
                                                                    }
                                                                >
                                                                    "Edit"
                                                                </button>
                                                                <button
                                                                    class="px-3 py-1 text-xs bg-red-900/50 hover:bg-red-800/70 text-red-400 rounded transition-colors"
                                                                    on:click=move |_| delete_id.set(Some(id))
                                                                >
                                                                    "Delete"
                                                                </button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                    {if is_empty {
                                        view! {
                                            <p class="text-center text-slate-500 py-8">"No lines configured yet."</p>
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

            // Create/Edit Form Modal
            <Show when=move || show_form.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center">
                    <div class="absolute inset-0 bg-black/60" on:click=move |_| show_form.set(false)/>
                    <div class="relative z-10 bg-slate-900 border border-slate-700 rounded-xl p-6 w-full max-w-lg shadow-2xl max-h-screen overflow-y-auto">
                        <h3 class="text-lg font-semibold text-slate-100 mb-4">
                            {move || if form.get().id.is_some() { "Edit Line" } else { "Add Line" }}
                        </h3>
                        <div class="space-y-3">
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">"Name *"</label>
                                <input type="text" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || form.get().name
                                    on:input=move |ev| form.update(|f| f.name = event_target_value(&ev))/>
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">"Line Number *"</label>
                                <input type="text" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || form.get().line_number
                                    on:input=move |ev| form.update(|f| f.line_number = event_target_value(&ev))/>
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">"Portal Username *"</label>
                                <input type="text" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || form.get().username
                                    on:input=move |ev| form.update(|f| f.username = event_target_value(&ev))/>
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">
                                    {move || if form.get().id.is_some() { "Portal Password (leave blank to keep unchanged)" } else { "Portal Password *" }}
                                </label>
                                <input type="password" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || form.get().password
                                    on:input=move |ev| form.update(|f| f.password = event_target_value(&ev))/>
                            </div>
                            <div class="grid grid-cols-2 gap-3">
                                <div>
                                    <label class="block text-xs font-medium text-slate-400 mb-1">"ISP"</label>
                                    <input type="text" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                        prop:value=move || form.get().isp
                                        on:input=move |ev| form.update(|f| f.isp = event_target_value(&ev))/>
                                </div>
                                <div>
                                    <label class="block text-xs font-medium text-slate-400 mb-1">"IP Address"</label>
                                    <input type="text" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                        prop:value=move || form.get().ip_address
                                        on:input=move |ev| form.update(|f| f.ip_address = event_target_value(&ev))/>
                                </div>
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">"Description"</label>
                                <textarea class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 resize-none" rows="2"
                                    prop:value=move || form.get().description
                                    on:input=move |ev| form.update(|f| f.description = event_target_value(&ev))/>
                            </div>
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox"
                                    class="w-4 h-4 rounded border-slate-600 bg-slate-800 text-indigo-600"
                                    prop:checked=move || form.get().is_active
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        form.update(|f| f.is_active = checked);
                                    }/>
                                <span class="text-sm text-slate-300">"Active"</span>
                            </label>
                        </div>
                        <Show when=move || form_error.get().is_some()>
                            <p class="mt-3 text-sm text-red-400 bg-red-950/40 border border-red-800 rounded-md px-3 py-2">
                                {move || form_error.get().unwrap_or_default()}
                            </p>
                        </Show>
                        <div class="flex gap-3 mt-5">
                            <button
                                class="flex-1 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors disabled:opacity-50"
                                disabled=move || submit_action.pending().get()
                                on:click=move |_| {
                                    let f = form.get_untracked();
                                    if f.name.is_empty() || f.line_number.is_empty() || f.username.is_empty() {
                                        form_error.set(Some("Name, line number and username are required.".into()));
                                        return;
                                    }
                                    if f.id.is_none() && f.password.is_empty() {
                                        form_error.set(Some("Password is required for new lines.".into()));
                                        return;
                                    }
                                    form_error.set(None);
                                    submit_action.dispatch(f);
                                }
                            >
                                {move || if submit_action.pending().get() { "Saving…" } else { "Save" }}
                            </button>
                            <button
                                class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-slate-300 text-sm rounded-md transition-colors"
                                on:click=move |_| show_form.set(false)
                            >"Cancel"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <ConfirmDialog
                title="Delete Line".to_string()
                message="Are you sure you want to delete this line? This cannot be undone.".to_string()
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
