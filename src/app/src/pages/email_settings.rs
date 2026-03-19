use leptos::prelude::*;
use crate::components::layout::AppShell;
use crate::components::ui::dialog::ConfirmDialog;
use crate::server_fns::email::{get_smtp_configs, create_smtp_config, delete_smtp_config};

#[derive(Clone, Default)]
struct SmtpForm {
    #[allow(dead_code)]
    id: Option<i32>,
    name: String,
    host: String,
    port: String,
    from_email: String,
    username: String,
    password: String,
    use_tls: bool,
    is_default: bool,
}

#[component]
pub fn EmailSettingsPage() -> impl IntoView {
    let configs_resource = Resource::new(|| (), |_| async { get_smtp_configs().await });
    let show_form = RwSignal::new(false);
    let form = RwSignal::new(SmtpForm::default());
    let form_error = RwSignal::new(Option::<String>::None);
    let delete_id = RwSignal::new(Option::<i32>::None);

    let create_action = Action::new(move |f: &SmtpForm| {
        let f = f.clone();
        async move {
            let port: i32 = f.port.parse().unwrap_or(587);
            create_smtp_config(
                f.name, f.host, port, f.from_email,
                if f.username.is_empty() { None } else { Some(f.username) },
                if f.password.is_empty() { None } else { Some(f.password) },
                f.use_tls, f.is_default,
            ).await
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = create_action.value().get() {
            show_form.set(false);
            form.set(SmtpForm::default());
            form_error.set(None);
            configs_resource.refetch();
        }
        if let Some(Err(ServerFnError::ServerError(ref msg))) = create_action.value().get() {
            form_error.set(Some(msg.clone()));
        }
    });

    let delete_action = Action::new(move |id: &i32| {
        let id = *id;
        async move { delete_smtp_config(id).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            delete_id.set(None);
            configs_resource.refetch();
        }
    });

    view! {
        <AppShell>
            <div class="max-w-4xl mx-auto">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-xl font-semibold text-slate-100">"Email Settings"</h2>
                    <button
                        class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                        on:click=move |_| {
                            form.set(SmtpForm { use_tls: true, port: "587".to_string(), ..Default::default() });
                            form_error.set(None);
                            show_form.set(true);
                        }
                    >"+ Add SMTP Config"</button>
                </div>

                <Suspense fallback=|| view! { <p class="text-slate-400">"Loading…"</p> }>
                    {move || configs_resource.get().map(|result| match result {
                        Ok(json) => {
                            let configs: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap_or_default();
                            let is_empty = configs.is_empty();
                            view! {
                                <div class="bg-slate-900 border border-slate-700 rounded-xl overflow-hidden">
                                    <table class="w-full text-sm">
                                        <thead class="bg-slate-800 text-slate-400">
                                            <tr>
                                                <th class="px-4 py-3 text-left font-medium">"Name"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Host"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Port"</th>
                                                <th class="px-4 py-3 text-left font-medium">"From"</th>
                                                <th class="px-4 py-3 text-left font-medium">"TLS"</th>
                                                <th class="px-4 py-3 text-left font-medium">"Default"</th>
                                                <th class="px-4 py-3 text-right font-medium">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-800">
                                            {configs.into_iter().map(|cfg| {
                                                let id = cfg.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                                let name = cfg.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let host = cfg.get("host").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let port = cfg.get("port").and_then(|v| v.as_i64()).unwrap_or(0);
                                                let from_email = cfg.get("senderEmail").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let use_tls = cfg.get("useTls").and_then(|v| v.as_bool()).unwrap_or(false);
                                                let is_default = cfg.get("isDefault").and_then(|v| v.as_bool()).unwrap_or(false);
                                                view! {
                                                    <tr class="text-slate-300 hover:bg-slate-800/50">
                                                        <td class="px-4 py-3 font-medium">{name}</td>
                                                        <td class="px-4 py-3 text-slate-400">{host}</td>
                                                        <td class="px-4 py-3 text-slate-400">{port}</td>
                                                        <td class="px-4 py-3 text-slate-400">{from_email}</td>
                                                        <td class="px-4 py-3">
                                                            {if use_tls {
                                                                view! { <span class="text-green-400">"Yes"</span> }.into_any()
                                                            } else {
                                                                view! { <span class="text-slate-500">"No"</span> }.into_any()
                                                            }}
                                                        </td>
                                                        <td class="px-4 py-3">
                                                            {if is_default {
                                                                view! { <span class="text-indigo-400">"Default"</span> }.into_any()
                                                            } else {
                                                                view! { <span class="text-slate-600">"—"</span> }.into_any()
                                                            }}
                                                        </td>
                                                        <td class="px-4 py-3 text-right">
                                                            <button
                                                                class="px-3 py-1 text-xs bg-red-900/50 hover:bg-red-800/70 text-red-400 rounded transition-colors"
                                                                on:click=move |_| delete_id.set(Some(id))
                                                            >"Delete"</button>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                    {if is_empty {
                                        view! {
                                            <p class="text-center text-slate-500 py-8">"No SMTP configurations yet."</p>
                                        }.into_any()
                                    } else {
                                        view! { <div/> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! { <p class="text-red-400">"Failed to load SMTP configs."</p> }.into_any(),
                    })}
                </Suspense>
            </div>

            // Create form modal
            <Show when=move || show_form.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center">
                    <div class="absolute inset-0 bg-black/60" on:click=move |_| show_form.set(false)/>
                    <div class="relative z-10 bg-slate-900 border border-slate-700 rounded-xl p-6 w-full max-w-lg shadow-2xl max-h-screen overflow-y-auto">
                        <h3 class="text-lg font-semibold text-slate-100 mb-4">"Add SMTP Configuration"</h3>
                        <div class="space-y-3">
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">"Name *"</label>
                                <input type="text" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || form.get().name
                                    on:input=move |ev| form.update(|f| f.name = event_target_value(&ev))/>
                            </div>
                            <div class="grid grid-cols-2 gap-3">
                                <div>
                                    <label class="block text-xs font-medium text-slate-400 mb-1">"SMTP Host *"</label>
                                    <input type="text" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                        prop:value=move || form.get().host
                                        on:input=move |ev| form.update(|f| f.host = event_target_value(&ev))/>
                                </div>
                                <div>
                                    <label class="block text-xs font-medium text-slate-400 mb-1">"Port"</label>
                                    <input type="number" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                        prop:value=move || form.get().port
                                        on:input=move |ev| form.update(|f| f.port = event_target_value(&ev))/>
                                </div>
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">"From Email *"</label>
                                <input type="email" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || form.get().from_email
                                    on:input=move |ev| form.update(|f| f.from_email = event_target_value(&ev))/>
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">"Username"</label>
                                <input type="text" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || form.get().username
                                    on:input=move |ev| form.update(|f| f.username = event_target_value(&ev))/>
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-slate-400 mb-1">"Password"</label>
                                <input type="password" class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                    prop:value=move || form.get().password
                                    on:input=move |ev| form.update(|f| f.password = event_target_value(&ev))/>
                            </div>
                            <div class="flex gap-4">
                                <label class="flex items-center gap-2 cursor-pointer">
                                    <input type="checkbox" class="w-4 h-4 rounded"
                                        prop:checked=move || form.get().use_tls
                                        on:change=move |ev| form.update(|f| f.use_tls = event_target_checked(&ev))/>
                                    <span class="text-sm text-slate-300">"Use TLS"</span>
                                </label>
                                <label class="flex items-center gap-2 cursor-pointer">
                                    <input type="checkbox" class="w-4 h-4 rounded"
                                        prop:checked=move || form.get().is_default
                                        on:change=move |ev| form.update(|f| f.is_default = event_target_checked(&ev))/>
                                    <span class="text-sm text-slate-300">"Set as Default"</span>
                                </label>
                            </div>
                        </div>
                        <Show when=move || form_error.get().is_some()>
                            <p class="mt-3 text-sm text-red-400 bg-red-950/40 border border-red-800 rounded-md px-3 py-2">
                                {move || form_error.get().unwrap_or_default()}
                            </p>
                        </Show>
                        <div class="flex gap-3 mt-5">
                            <button
                                class="flex-1 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors disabled:opacity-50"
                                disabled=move || create_action.pending().get()
                                on:click=move |_| {
                                    let f = form.get_untracked();
                                    if f.name.is_empty() || f.host.is_empty() || f.from_email.is_empty() {
                                        form_error.set(Some("Name, host and from email are required.".into()));
                                        return;
                                    }
                                    form_error.set(None);
                                    create_action.dispatch(f);
                                }
                            >
                                {move || if create_action.pending().get() { "Saving…" } else { "Save" }}
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
                title="Delete SMTP Config".to_string()
                message="Are you sure you want to delete this SMTP configuration?".to_string()
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
