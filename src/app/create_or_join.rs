use leptos::*;
// use leptos_router::ActionForm;

#[server(CreateNewRoom)]
pub async fn create_new_room(room_name: String) -> Result<(), ServerFnError> {
    use super::AppPath;
    use crate::{
        models::user_model::UserData,
        state::{auth, pool, rooms_manager},
    };
    use chrono::Utc;

    let auth = auth()?;
    let pool = pool()?;
    let rooms_manager = rooms_manager()?;

    let user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("Auth does not contain user"))?;
    let user_data = UserData::get_from_uuid(&user.uuid, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("User does not exist"))?;

    let created_at = Utc::now();

    match rooms_manager
        .new_room(room_name.clone(), user, &pool, created_at)
        .await
    {
        Ok(room_uuid) => {
            user_data
                .add_channel(room_uuid.clone(), &pool)
                .await
                .map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

            Ok(leptos_axum::redirect(
                &AppPath::Channel(Some(room_uuid)).to_string(),
            ))
        }
        Err(err) => Err(ServerFnError::new(format!("{:?}", err))),
    }
}

#[server(JoinRoom)]
pub async fn join_room(room_uuid: String) -> Result<(), ServerFnError> {
    use super::AppPath;
    use crate::{
        models::user_model::UserData,
        state::{auth, pool, rooms_manager},
    };

    let auth = auth()?;
    let pool = pool()?;
    let rooms_manager = rooms_manager()?;

    let user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("Auth does not contain user"))?;

    let user_data = UserData::get_from_uuid(&user.uuid, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("User does not exist"))?;

    user_data
        .add_channel(room_uuid.clone(), &pool)
        .await
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

    match rooms_manager.join_room(&room_uuid, user, &pool).await {
        Ok(_) => Ok(leptos_axum::redirect(
            &AppPath::Channel(Some(room_uuid)).to_string(),
        )),
        Err(err) => Err(ServerFnError::new(format!("{:?}", err))),
    }
}

#[component]
pub fn PopUpRoomForm(
    display_room_form: ReadSignal<bool>,
    create_room_action: Action<CreateNewRoom, Result<(), ServerFnError>>,
    join_room_action: Action<JoinRoom, Result<(), ServerFnError>>,
) -> impl IntoView {
    let (show_cr, set_show_cr) = create_signal("hidden");
    let (show_join, set_show_join) = create_signal("hidden");

    let show_create_form = move |_| {
        if show_cr.get() == "hidden" {
            set_show_cr.set("block flex flex-col");
            set_show_join.set("hidden");
        } else {
            set_show_cr.set("hidden")
        }
    };

    let show_join_form = move |_| {
        if show_join.get() == "hidden" {
            set_show_join.set("block flex flex-col");
            set_show_cr.set("hidden");
        } else {
            set_show_join.set("hidden")
        }
    };

    view! {
        <Show when=move || display_room_form.get()>
            {
                let cr_node = create_node_ref::<html::Input>();
                let join_node = create_node_ref::<html::Input>();

                let cnr = move |ev: ev::SubmitEvent| {
                    ev.prevent_default();
                    let room_name = cr_node.get().expect("input element does not exist").value();
                    create_room_action.dispatch(CreateNewRoom { room_name });
                    cr_node
                        .get()
                        .expect("input element does not exist")
                        .set_value("");
                };

                let jtr = move |ev: ev::SubmitEvent| {
                    ev.prevent_default();
                    let room_uuid = join_node
                        .get()
                        .expect("input element does not exist")
                        .value();
                    join_room_action.dispatch(JoinRoom { room_uuid });
                    join_node
                        .get()
                        .expect("input element does not exist")
                        .set_value("");
                };

                view! {
                    <div class="block absolute m-auto left-0 right-0 top-0 bottom-0 flex flex-col bg-slate-800 size-[25rem] rounded-xl py-4">
                        <form
                            on:submit=cnr
                            class="flex flex-col px-4">
                            <div
                                class="cursor-pointer bg-indigo-500/[.65] justify-center w-auto h-14 flex flex-col rounded-xl"
                                on:click=show_create_form
                            >
                                <h1 class="cursor-pointer text-white text-center text-xl">"Create New Room"</h1>
                            </div>
                            <div class=move || show_cr.get()>
                                <input
                                    _ref=cr_node
                                    required
                                    class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-4 text-base h-10"
                                    placeholder="Enter new room name..."
                                    name="room_name"/>
                                <button class="text-white hover:text-black mt-2 w-full bg-sky-500 hover:bg-green-300 rounded-lg border-0 w-fit py-1 px-1">
                                    "create"
                                </button>
                            </div>
                        </form>
                        <form
                            on:submit=jtr
                            class="flex flex-col px-4">
                            <div
                                class="cursor-pointer bg-indigo-500/[.65] mt-4 justify-center w-auto h-14 flex flex-col rounded-xl"
                                on:click=show_join_form
                            >
                                <h1 class="cursor-pointer text-white text-center text-xl">"Join Room"</h1>
                            </div>
                            <div class=move || show_join.get()>
                                <input
                                    _ref=join_node
                                    required
                                    class="text-white pl-1 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto mt-4 text-base h-10"
                                    placeholder="Enter room id..."
                                    name="room_uuid"/>
                                <button class="text-white hover:text-black mt-2 w-full bg-sky-500 hover:bg-green-300 rounded-lg border-0 w-fit py-1 px-1">
                                    "join"
                                </button>
                            </div>
                        </form>
                    </div>
                }
            }
        </Show>
    }
}

#[component]
pub fn CreateOrJoinRoomButton(
    display_room_form: ReadSignal<bool>,
    set_display_room_form: WriteSignal<bool>,
) -> impl IntoView {
    let (rotate, set_rotate) = create_signal("transition duration-150 border-none pb-1 h-12 w-12 bg-sky-500 hover:bg-green-300 hover:text-black rounded-xl text-white font-sans text-2xl text-center");

    let popup = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        if !display_room_form.get() {
            set_display_room_form.set(true);
            set_rotate.set("transition duration-150 border-none pb-1 rotate-45 h-12 w-12 bg-green-300 text-black rounded-full font-sans text-2xl text-center");
        } else {
            set_display_room_form.set(false);
            set_rotate.set("transition duration-150 border-none pb-1 h-12 w-12 bg-sky-500 hover:bg-green-300 hover:text-black rounded-xl text-white font-sans text-2xl text-center");
        }
    };
    view! {
        <button
            on:click=popup
            class=move || rotate.get()
        >
            "+"
        </button>
    }
}
