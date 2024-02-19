use leptos::*;
use leptos_router::ActionForm;

#[server(UserLogout)]
async fn logout() -> Result<(), ServerFnError> {
    use crate::auth_model::ssr::auth;

    let auth = auth()?;
    auth.logout_user();
    leptos_axum::redirect("/login");

    Ok(())
}

#[component]
pub fn ChatPage() -> impl IntoView {
    view! {
        <div class="size-11/12 flex flex-row mx-4 my-4 bg-slate-800/[.65] rounded-xl">
            <div class="h-full w-[70px] bg-slate-950/[.65] rounded-l-xl" id="main-navigation"></div>
            <div class="h-full w-[300px] bg-transparent rounded-l-xl flex flex-col">
                <div class="h-20 w-full bg-transparent pl-4">
                    <LogoutButton/>
                </div>
                <div class="grow w-full bg-slate-800/[.65] rounded-bl-xl"></div>
            </div>
            <div class="h-full bg-transparent grow flex flex-col" id="chat">
                <div class="grow bg-transparent px-4" id="chat-log">
                    <p class="text-white font-sans">"test"</p>
                </div>
                <form class="bg-transparent px-4 h-32 flex flex-row items-center">
                    <input
                        class="grow rounded-l-full h-12 text-white font-sans pl-2 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto text-base"
                    />
                    <button class="border-none h-12 w-fit px-2 bg-sky-500 hover:bg-green-300 hover:text-black rounded-r-full border-0 text-white font-sans">
                        <strong>"SEND"</strong>
                    </button>
                </form>
            </div>
        </div>
    }
}

#[component]
fn LogoutButton() -> impl IntoView {
    let logout = create_server_action::<UserLogout>();
    view! {
        <ActionForm action=logout>
            <button class="font-sans text-white hover:text-black border-none h-12 w-fit px-2 bg-transparent">
                "Log out"
            </button>
        </ActionForm>
    }
}
