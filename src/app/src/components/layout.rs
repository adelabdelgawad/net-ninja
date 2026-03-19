use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::Redirect;
use crate::server_fns::auth::check_auth;

/// Authenticated shell — checks session before rendering any protected content.
/// Redirects to /login if not authenticated.
#[component]
pub fn AppShell(children: ChildrenFn) -> impl IntoView {
    let auth = Resource::new(|| (), |_| async move { check_auth().await });

    view! {
        <Suspense>
            {move || {
                let ch = children.clone();
                auth.get().map(move |r| {
                    if r.is_ok() {
                        view! {
                            <div class="flex h-screen bg-slate-950">
                                <Sidebar/>
                                <div class="flex-1 flex flex-col overflow-hidden min-w-0">
                                    <TopBar/>
                                    <main class="flex-1 overflow-auto p-4 md:p-6">
                                        {ch()}
                                    </main>
                                </div>
                            </div>
                        }
                        .into_any()
                    } else {
                        view! { <Redirect path="/login"/> }.into_any()
                    }
                })
            }}
        </Suspense>
    }
}

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <nav class="hidden md:flex w-56 lg:w-64 bg-slate-900 border-r border-slate-800 flex-col shrink-0">
            <div class="p-4 border-b border-slate-800">
                <h1 class="text-lg font-bold text-slate-100">"NetNinja"</h1>
            </div>
            <ul class="flex-1 p-2 space-y-1">
                <SidebarLink href="/dashboard" label="Dashboard"/>
                <SidebarLink href="/lines" label="Lines"/>
                <SidebarLink href="/tasks" label="Tasks"/>
                <SidebarLink href="/email-settings" label="Email Settings"/>
                <SidebarLink href="/quota-results" label="Quota Results"/>
                <SidebarLink href="/speed-results" label="Speed Results"/>
                <SidebarLink href="/logs" label="Logs"/>
            </ul>
            <div class="p-2 border-t border-slate-800">
                <LogoutButton/>
            </div>
        </nav>
    }
}

#[component]
fn SidebarLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <li>
            <a
                href=href
                class="block px-3 py-2 rounded text-sm text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
            >
                {label}
            </a>
        </li>
    }
}

#[component]
fn LogoutButton() -> impl IntoView {
    let pending = RwSignal::new(false);

    let on_logout = move |_| {
        pending.set(true);
        spawn_local(async move {
            let _ = crate::server_fns::auth::logout().await;
            // Use native browser navigation to clear JS state completely
            leptos_router::hooks::use_navigate()("/login", Default::default());
        });
    };

    view! {
        <button
            class="block w-full text-left px-3 py-2 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 text-sm transition-colors disabled:opacity-50"
            disabled=move || pending.get()
            on:click=on_logout
        >
            {move || if pending.get() { "Signing out…" } else { "Logout" }}
        </button>
    }
}

#[component]
pub fn TopBar() -> impl IntoView {
    view! {
        <header class="h-14 bg-slate-900 border-b border-slate-800 flex items-center px-4 md:px-6 shrink-0">
            <span class="text-sm text-slate-400">"NetNinja Admin"</span>
        </header>
    }
}
