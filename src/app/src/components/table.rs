use leptos::prelude::*;

/// Generic data table columns config
#[derive(Clone)]
pub struct Column {
    pub header: &'static str,
    pub key: &'static str,
}

#[allow(unused_variables)]
#[component]
pub fn DataTable(
    columns: Vec<Column>,
    #[prop(into)] rows: Signal<Vec<Vec<String>>>,
    #[prop(default = 1)] page: u32,
    #[prop(default = 0)] total_pages: u32,
    #[prop(optional)] on_prev: Option<Callback<()>>,
    #[prop(optional)] on_next: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto rounded-lg border border-slate-800">
            <table class="w-full text-sm text-left text-slate-300">
                <thead class="text-xs uppercase bg-slate-900 text-slate-400">
                    <tr>
                        {columns.iter().map(|col| view! {
                            <th class="px-4 py-3">{col.header}</th>
                        }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || rows.get().iter().map(|row| view! {
                        <tr class="border-t border-slate-800 hover:bg-slate-900/50">
                            {row.iter().map(|cell| view! {
                                <td class="px-4 py-3">{cell.clone()}</td>
                            }).collect_view()}
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
            {if total_pages > 1 {
                view! {
                    <div class="flex items-center justify-between px-4 py-3 border-t border-slate-800">
                        <span class="text-sm text-slate-400">"Page " {page} " of " {total_pages}</span>
                        <div class="flex gap-2">
                            {on_prev.map(|f| view! {
                                <button
                                    class="px-3 py-1 text-sm bg-slate-800 rounded hover:bg-slate-700 disabled:opacity-50"
                                    disabled=move || page <= 1
                                    on:click=move |_| f.run(())
                                >"Previous"</button>
                            })}
                            {on_next.map(|f| {
                                let _ = f; // f is Copy; used below in move closure
                                view! {
                                    <button
                                        class="px-3 py-1 text-sm bg-slate-800 rounded hover:bg-slate-700 disabled:opacity-50"
                                        disabled=move || page >= total_pages
                                        on:click=move |_| f.run(())
                                    >"Next"</button>
                                }
                            })}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div/> }.into_any()
            }}
        </div>
    }
}
