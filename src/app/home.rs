use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="flex flex-col space-y-3 bg-transparent">
            <p class="font-sans text-[160px] text-black">"Welcome to HEY!"</p>
            <div class="flex flex-row justify-center space-x-7 bg-transparent">
                <a
                    class="font-sans text-[42px] tracking-wide text-white hover:text-cyan-400"
                    href="/register"
                >
                    "Register"
                </a>
                <a
                    class="font-sans text-[42px] tracking-wide text-white hover:text-cyan-400"
                    href="/login"
                >
                    "Login"
                </a>
            </div>
        </div>
    }
}
