use leptos::prelude::*;

#[component]
pub fn ConfirmDialog(
    title: String,
    message: String,
    #[prop(into)] open: Signal<bool>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
    #[prop(default = "Confirm")] confirm_label: &'static str,
    #[prop(default = "Cancel")] cancel_label: &'static str,
) -> impl IntoView {
    // StoredValue is Copy, so it can be captured by Fn closures
    let title = StoredValue::new(title);
    let message = StoredValue::new(message);
    let on_cancel_overlay = on_cancel;
    let on_cancel_btn = on_cancel;

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center">
                <div class="absolute inset-0 bg-black/60" on:click=move |_| on_cancel_overlay.run(())/>
                <div class="relative z-10 bg-slate-900 border border-slate-700 rounded-lg p-6 w-full max-w-sm shadow-xl">
                    <h3 class="text-lg font-semibold text-slate-100 mb-2">
                        {move || title.get_value()}
                    </h3>
                    <p class="text-sm text-slate-400 mb-6">
                        {move || message.get_value()}
                    </p>
                    <div class="flex justify-end gap-3">
                        <button
                            class="px-4 py-2 text-sm rounded-md bg-slate-800 text-slate-300 hover:bg-slate-700"
                            on:click=move |_| on_cancel_btn.run(())
                        >{cancel_label}</button>
                        <button
                            class="px-4 py-2 text-sm rounded-md bg-red-700 text-white hover:bg-red-800"
                            on:click=move |_| on_confirm.run(())
                        >{confirm_label}</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
