use leptos::*;
use leptos_router::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="flex flex-col space-y-3 bg-transparent">
            <p class="font-sans text-center text-[160px] text-black">"Welcome to HEY!"</p>
            <div class="flex flex-row justify-center space-x-7 bg-transparent">
                <A
                    class="font-sans text-[42px] tracking-wide text-white hover:text-cyan-400"
                    href="/register"
                >
                    "Register"
                </A>
                <A
                    class="font-sans text-[42px] tracking-wide text-white hover:text-cyan-400"
                    href="/login"
                >
                    "Login"
                </A>
            </div>
        </div>
    }
}
