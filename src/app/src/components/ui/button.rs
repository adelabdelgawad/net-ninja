use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Destructive,
    Ghost,
}

impl ButtonVariant {
    fn classes(&self) -> &'static str {
        match self {
            Self::Primary => "bg-blue-600 hover:bg-blue-700 text-white",
            Self::Secondary => "bg-slate-800 hover:bg-slate-700 text-slate-100 border border-slate-700",
            Self::Destructive => "bg-red-700 hover:bg-red-800 text-white",
            Self::Ghost => "hover:bg-slate-800 text-slate-400 hover:text-slate-100",
        }
    }
}

#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let base = "inline-flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 disabled:pointer-events-none disabled:opacity-50";
    let variant_classes = variant.classes();

    view! {
        <button
            class=format!("{} {}", base, variant_classes)
            disabled=disabled
            on:click=move |_| { if let Some(f) = on_click { f.run(()); } }
        >
            {children()}
        </button>
    }
}
