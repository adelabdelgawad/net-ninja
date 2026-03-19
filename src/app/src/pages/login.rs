use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::server_fns::auth::login;

#[component]
pub fn LoginPage() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error_msg = RwSignal::new(Option::<String>::None);

    let navigate = use_navigate();

    let login_action = Action::new(move |(u, p): &(String, String)| {
        let u = u.clone();
        let p = p.clone();
        async move { login(u, p).await }
    });

    // React to action result
    Effect::new(move |_| {
        if let Some(result) = login_action.value().get() {
            match result {
                Ok(()) => navigate("/dashboard", Default::default()),
                Err(ServerFnError::ServerError(ref msg)) => {
                    let display = match msg.as_str() {
                        "invalid_credentials" => "Invalid username or password.",
                        "rate_limited" => "Too many login attempts. Please wait and try again.",
                        _ => "Service unavailable. Please try again later.",
                    };
                    error_msg.set(Some(display.to_string()));
                }
                Err(_) => {
                    error_msg.set(Some("Service unavailable. Please try again later.".to_string()));
                }
            }
        }
    });

    let pending = login_action.pending();

    view! {
        <div class="min-h-screen flex items-center justify-center bg-slate-950 px-4">
            <div class="w-full max-w-sm">
                <div class="mb-8 text-center">
                    <h1 class="text-3xl font-bold text-slate-100">"NetNinja"</h1>
                    <p class="mt-1 text-sm text-slate-400">"Network monitoring dashboard"</p>
                </div>
                <div class="bg-slate-900 border border-slate-700 rounded-xl p-6 shadow-2xl">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-slate-300 mb-1">"Username"</label>
                            <input
                                type="text"
                                autocomplete="username"
                                class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                                placeholder="admin"
                                prop:value=move || username.get()
                                on:input=move |ev| username.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-300 mb-1">"Password"</label>
                            <input
                                type="password"
                                autocomplete="current-password"
                                class="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded-md text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                                placeholder="••••••••"
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </div>
                        <Show when=move || error_msg.get().is_some()>
                            <p class="text-sm text-red-400 bg-red-950/40 border border-red-800 rounded-md px-3 py-2">
                                {move || error_msg.get().unwrap_or_default()}
                            </p>
                        </Show>
                        <button
                            type="button"
                            disabled=move || pending.get()
                            class="w-full py-2 px-4 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium rounded-md transition-colors"
                            on:click=move |_| {
                                error_msg.set(None);
                                login_action.dispatch((username.get_untracked(), password.get_untracked()));
                            }
                        >
                            {move || if pending.get() { "Signing in…" } else { "Sign in" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
