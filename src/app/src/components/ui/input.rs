use leptos::prelude::*;

#[component]
pub fn Input(
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    #[prop(default = "text")] input_type: &'static str,
    #[prop(into)] value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="space-y-1">
            {label.map(|l| view! {
                <label class="block text-sm font-medium text-slate-300">{l}</label>
            })}
            <input
                type=input_type
                class="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-md text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
                placeholder=placeholder.unwrap_or_default()
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
            />
            {error.map(|e| view! {
                <p class="text-sm text-red-400">{e}</p>
            })}
        </div>
    }
}
