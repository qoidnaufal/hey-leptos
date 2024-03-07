use crate::{app::CtxProvider, user_model::User};
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateRoomPayLoad {
    room_name: String,
    user: User,
}

impl CreateRoomPayLoad {
    fn new(room_name: String, user: User) -> Self {
        Self { room_name, user }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JoinRoomPayload {
    room_uuid: String,
    user: User,
}

impl JoinRoomPayload {
    fn new(room_uuid: String, user: User) -> Self {
        Self { room_uuid, user }
    }
}

// ----

#[server(CreateNewRoom)]
async fn create_new_room(payload: CreateRoomPayLoad) -> Result<(), ServerFnError> {
    use crate::{
        state::{pool, rooms_manager},
        user_model::UserData,
    };

    let pool = pool()?;
    let user = payload.user.clone();
    let user_data = UserData::get_from_uuid(&user.uuid, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("User does not exist"))?;

    let rooms_manager = rooms_manager()?;

    match rooms_manager.new_room(payload.room_name, user) {
        Ok(room_uuid) => {
            user_data
                .add_channel(room_uuid.clone(), &pool)
                .await
                .map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

            Ok(leptos_axum::redirect(&room_uuid))
        }
        Err(err) => Err(ServerFnError::new(format!("{:?}", err))),
    }
}

#[server(JoinRoom)]
async fn join_room(payload: JoinRoomPayload) -> Result<(), ServerFnError> {
    use crate::{
        state::{pool, rooms_manager},
        user_model::UserData,
    };

    logging::log!("Received payload is: {:?}", payload);

    let pool = pool()?;
    let user = payload.user.clone();
    let user_data = UserData::get_from_uuid(&user.uuid, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("User does not exist"))?;

    user_data
        .add_channel(payload.room_uuid.clone(), &pool)
        .await
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

    let rooms_manager = rooms_manager()?;

    match rooms_manager.join_room(payload.room_uuid.clone(), user) {
        Ok(_) => Ok(leptos_axum::redirect(&payload.room_uuid)),
        Err(err) => Err(ServerFnError::new(format!("{:?}", err))),
    }
}

#[component]
pub fn PopUpRoomForm(display: ReadSignal<&'static str>) -> impl IntoView {
    let user = create_memo(move |_| expect_context::<CtxProvider>().user);
    logging::log!("User is: {:?}\n", user.get_untracked());

    let create_new_room =
        create_action(|payload: &CreateRoomPayLoad| create_new_room(payload.clone()));
    let join_room = create_action(|payload: &JoinRoomPayload| join_room(payload.clone()));

    // ----

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

    // ----

    let cr_node = create_node_ref::<html::Input>();
    let join_node = create_node_ref::<html::Input>();

    let cnr = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let room_name = cr_node.get().expect("input element does not exist").value();
        let payload = CreateRoomPayLoad::new(room_name, user.get());
        logging::log!("create new room payload: {:?}", payload);
        create_new_room.dispatch(payload);
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
        let payload = JoinRoomPayload::new(room_uuid, user.get());
        join_room.dispatch(payload);
        join_node
            .get()
            .expect("input element does not exist")
            .set_value("");
    };

    view! {
        <div class=move || display.get()>
        <form
            on:submit=cnr
            class="flex flex-col px-4">
            <div class="cursor-pointer bg-indigo-500/[.65] justify-center w-auto h-14 flex flex-col rounded-xl" on:click=show_create_form>
                <h1 class="cursor-pointer text-white text-center text-xl">"Create New Room"</h1>
            </div>
            <div class=move || show_cr.get()>
                <input
                    _ref=cr_node
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
            <div class="cursor-pointer bg-indigo-500/[.65] mt-4 justify-center w-auto h-14 flex flex-col rounded-xl" on:click=show_join_form>
                <h1 class="cursor-pointer text-white text-center text-xl">"Join Room"</h1>
            </div>
            <div class=move || show_join.get()>
                <input
                    _ref=join_node
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

#[component]
pub fn CreateOrJoinRoomButton(
    read_sig: ReadSignal<&'static str>,
    write_sig: WriteSignal<&'static str>,
) -> impl IntoView {
    let (rotate, set_rotate) = create_signal("transition duration-150 border-none pb-1 h-12 w-12 bg-sky-500 hover:bg-green-300 hover:text-black rounded-xl text-white font-sans text-2xl text-center");
    let popup = move |_| {
        if read_sig.get() == "hidden" {
            write_sig.set(
                "block absolute top-1/3 left-1/3 flex flex-col bg-slate-800/[.45] w-[300px] h-fit rounded-xl py-4"
            );
            set_rotate.set("transition duration-150 border-none pb-1 rotate-45 h-12 w-12 bg-green-300 text-black rounded-full font-sans text-2xl text-center");
        } else {
            write_sig.set("hidden");
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
