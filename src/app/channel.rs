use super::app_error::{AppError, ErrorTemplate};
use leptos::*;

#[server]
async fn validate_path(path: String) -> Result<(), ServerFnError> {
    use crate::state::ssr::{auth, rooms_manager};

    let rooms_manager = rooms_manager()?;
    let auth = auth()?;

    if !auth.is_authenticated() {
        return Err(ServerFnError::new(
            "User isn't authenticated to see the channel",
        ));
    }

    if path.starts_with("/channel/") {
        let room_uuid = path
            .strip_prefix("/channel/")
            .expect("Valid uuid is needed")
            .to_string();

        rooms_manager
            .validate_uuid(room_uuid)
            .map_err(|err| ServerFnError::new(format!("{:?}", err)))
    } else {
        Ok(())
    }
}

#[server(PublishMsg)]
async fn publish_msg(text: String, room_uuid: String) -> Result<(), ServerFnError> {
    use crate::models::message_model::{Msg, MsgData};
    use crate::state::rooms_manager::PubSubClient;
    use crate::state::ssr::{auth, pool, rooms_manager};

    let auth = auth()?;
    let pool = pool()?;
    let rooms_manager = rooms_manager()?;

    let user = auth
        .current_user
        .ok_or_else(|| ServerFnError::new("Auth does not contain user"))?;

    rooms_manager
        .init(PubSubClient::Publisher)
        .await
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))?;

    let msg = Msg::Text(text);

    logging::log!(
        "[msg] > on channel: {}:\n      > {} published: {:?}\n",
        &room_uuid,
        &user.user_name,
        &msg
    );

    let msg_data = MsgData::new(room_uuid.clone(), user, msg.clone());

    match rooms_manager
        .publish_msg(room_uuid, msg)
        .await
        .map_err(|err| ServerFnError::new(format!("{:?}", err)))
    {
        Ok(_) => msg_data
            .insert_into_db(&pool)
            .await
            .map_err(|err| ServerFnError::new(format!("{:?}", err))),
        Err(err) => Err(ServerFnError::new(format!("{:?}", err))),
    }
}

#[component]
pub fn Channel() -> impl IntoView {
    let path = leptos_router::use_location().pathname;

    let path_resource = create_resource(move || path.get(), validate_path);

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                match path_resource.get().unwrap_or(Ok(())) {
                    Ok(_) => {
                        let message_ref = create_node_ref::<html::Input>();
                        let publish_msg = create_server_action::<PublishMsg>();
                        let send = move |ev: ev::SubmitEvent| {
                            ev.prevent_default();
                            let path = path.get();
                            let room_uuid = path
                                .strip_prefix("/channel/")
                                .expect("Provide valid uuid!")
                                .to_string();
                            let text = message_ref
                                .get()
                                .expect("input element doesn't exist")
                                .value();
                            publish_msg.dispatch(PublishMsg { text, room_uuid });
                            message_ref.get().expect("input element doesn't exist").set_value("");
                        };
                        view! {
                            <div
                                class="h-full bg-transparent grow flex flex-col"
                                id="chat-interface"
                            >
                                <div class="grow bg-transparent px-4" id="chat-log"></div>
                                <form
                                    on:submit=send
                                    class="bg-transparent px-4 h-32 flex flex-row items-center"
                                >
                                    <input
                                        id="input"
                                        name="message"
                                        _ref=message_ref
                                        placeholder="Type your message..."
                                        class="grow rounded-md h-12 text-white font-sans pl-2 bg-white/20 hover:bg-white/10 focus:bg-white/10 focus:outline-none border-0 w-auto text-base"
                                    />
                                </form>
                            </div>
                        }
                            .into_view()
                    }
                    Err(_) => {
                        let mut outside_errors = Errors::default();
                        outside_errors.insert_with_default_key(AppError::RoomDoesNotExist);
                        view! {
                            <div class="flex h-full bg-transparent grow items-center justify-center">
                                <ErrorTemplate outside_errors/>
                            </div>
                        }.into_view()
                    }
                }
            }}

        </Transition>
    }
}
