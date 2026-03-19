use leptos::prelude::*;

#[derive(Clone, Copy)]
pub enum BadgeVariant {
    Success,
    Failure,
    Running,
    Warning,
    Info,
}

impl BadgeVariant {
    fn classes(&self) -> &'static str {
        match self {
            Self::Success => "bg-green-900 text-green-300 border-green-800",
            Self::Failure => "bg-red-900 text-red-300 border-red-800",
            Self::Running => "bg-blue-900 text-blue-300 border-blue-800",
            Self::Warning => "bg-yellow-900 text-yellow-300 border-yellow-800",
            Self::Info => "bg-slate-800 text-slate-300 border-slate-700",
        }
    }
}

#[component]
pub fn Badge(
    #[prop(default = BadgeVariant::Info)] variant: BadgeVariant,
    children: Children,
) -> impl IntoView {
    view! {
        <span class=format!(
            "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border {}",
            variant.classes()
        )>
            {children()}
        </span>
    }
}

/// Convenience: convert a status string to a Badge
#[component]
pub fn StatusBadge(status: String) -> impl IntoView {
    let variant = match status.as_str() {
        "success" | "completed" => BadgeVariant::Success,
        "failed" | "error" => BadgeVariant::Failure,
        "running" => BadgeVariant::Running,
        "warning" => BadgeVariant::Warning,
        _ => BadgeVariant::Info,
    };
    view! {
        <Badge variant=variant>{status}</Badge>
    }
}
