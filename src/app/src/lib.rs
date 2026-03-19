pub mod components;
pub mod pages;
pub mod server_fns;

#[cfg(feature = "ssr")]
pub mod state;
#[cfg(feature = "ssr")]
pub mod startup;
#[cfg(feature = "ssr")]
pub mod auth;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes, Redirect},
    path,
};

pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/netninja-web.css"/>
        <Title text="NetNinja"/>
        <Router>
            <Routes fallback=|| view! { <p class="p-8 text-slate-400">"404 — Page not found"</p> }>
                <Route path=path!("/login") view=pages::login::LoginPage/>
                <Route path=path!("/change-password") view=pages::change_password::ChangePasswordPage/>
                <Route path=path!("/dashboard") view=pages::dashboard::DashboardPage/>
                <Route path=path!("/lines") view=pages::lines::LinesPage/>
                <Route path=path!("/tasks") view=pages::tasks::TasksPage/>
                <Route path=path!("/email-settings") view=pages::email_settings::EmailSettingsPage/>
                <Route path=path!("/quota-results") view=pages::quota_results::QuotaResultsPage/>
                <Route path=path!("/speed-results") view=pages::speed_results::SpeedResultsPage/>
                <Route path=path!("/logs") view=pages::logs::LogsPage/>
                <Route path=path!("/") view=|| view! { <Redirect path="/dashboard"/> }/>
            </Routes>
        </Router>
    }
}
