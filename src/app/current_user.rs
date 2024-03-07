use leptos::*;

#[component]
pub fn CurrentUser(mut name: String) -> impl IntoView {
    name.truncate(1);

    view! {
        <div class="flex justify-center items-center pb-1 cursor-pointer h-12 w-12 bg-sky-500 hover:bg-green-300 hover:text-black rounded-full text-white uppercase font-sans text-2xl text-center">
            { name }
        </div>
    }
}
