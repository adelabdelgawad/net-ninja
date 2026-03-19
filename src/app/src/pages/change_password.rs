use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::server_fns::auth::change_password;
use crate::components::layout::AppShell;

#[component]
pub fn ChangePasswordPage() -> impl IntoView {
    let current = RwSignal::new(String::new());
    let new_pw = RwSignal::new(String::new());
    let confirm_pw = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);
    let success = RwSignal::new(false);

    let navigate = use_navigate();

    let change_action = Action::new(move |(c, n): &(String, String)| {
        let c = c.clone();
        let n = n.clone();
        async move { change_password(c, n).await }
    });

    Effect::new(move |_| {
        if let Some(result) = change_action.value().get() {
            match result {
                Ok(()) => {
                    success.set(true);
                    navigate("/dashboard", Default::default());
                }
                Err(ServerFnError::ServerError(ref msg)) => {
                    let display = match msg.as_str() {
                        "wrong_current_password" => "Current password is incorrect.",
                        _ => "Failed to change password. Please try again.",
                    };
                    error_msg.set(Some(display.to_string()));
                }
                Err(_) => {
                    error_msg.set(Some("Service unavailable. Please try again later.".to_string()));
                }
            }
        }
    });

    let pending = change_action.pending();

    view! {
        <AppShell>
            <div class="max-w-md mx-auto mt-8">
                <h2 class="text-xl font-semibold text-slate-100 mb-6">"Change Password"</h2>
                <div class="bg-slate-900 border border-slate-700 rounded-xl p-6">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-slate-300 mb-1">"Current Password"</label>
                            <input
                                type="password"
                                autocomplete="current-password"
                                class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                prop:value=move || current.get()
                                on:input=move |ev| current.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-300 mb-1">"New Password"</label>
                            <input
                                type="password"
                                autocomplete="new-password"
                                class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                prop:value=move || new_pw.get()
                                on:input=move |ev| new_pw.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-300 mb-1">"Confirm New Password"</label>
                            <input
                                type="password"
                                autocomplete="new-password"
                                class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                                prop:value=move || confirm_pw.get()
                                on:input=move |ev| confirm_pw.set(event_target_value(&ev))
                            />
                        </div>
                        <Show when=move || error_msg.get().is_some()>
                            <p class="text-sm text-red-400 bg-red-950/40 border border-red-800 rounded-md px-3 py-2">
                                {move || error_msg.get().unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || success.get()>
                            <p class="text-sm text-green-400 bg-green-950/40 border border-green-800 rounded-md px-3 py-2">
                                "Password changed successfully."
                            </p>
                        </Show>
                        <div class="flex gap-3 pt-2">
                            <button
                                type="button"
                                disabled=move || pending.get()
                                class="flex-1 py-2 px-4 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white font-medium rounded-md transition-colors"
                                on:click=move |_| {
                                    let c = current.get_untracked();
                                    let n = new_pw.get_untracked();
                                    let cf = confirm_pw.get_untracked();
                                    if n != cf {
                                        error_msg.set(Some("New passwords do not match.".to_string()));
                                        return;
                                    }
                                    if n.len() < 8 {
                                        error_msg.set(Some("New password must be at least 8 characters.".to_string()));
                                        return;
                                    }
                                    error_msg.set(None);
                                    change_action.dispatch((c, n));
                                }
                            >
                                {move || if pending.get() { "Saving…" } else { "Change Password" }}
                            </button>
                            <a
                                href="/dashboard"
                                class="px-4 py-2 text-sm text-slate-400 hover:text-slate-100 rounded-md bg-slate-800 hover:bg-slate-700 transition-colors"
                            >
                                "Cancel"
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </AppShell>
    }
}
