use leptos::prelude::*;

#[derive(Clone, Copy)]
pub enum AlertVariant {
    Warning,
    Info,
    Error,
}

impl AlertVariant {
    fn classes(&self) -> &'static str {
        match self {
            Self::Warning => "bg-yellow-950 border-yellow-800 text-yellow-200",
            Self::Info => "bg-blue-950 border-blue-800 text-blue-200",
            Self::Error => "bg-red-950 border-red-800 text-red-200",
        }
    }
}

#[component]
pub fn Alert(
    #[prop(default = AlertVariant::Info)] variant: AlertVariant,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!(
            "flex items-start gap-3 p-4 rounded-lg border {}",
            variant.classes()
        )>
            <div class="flex-1 text-sm">{children()}</div>
        </div>
    }
}
