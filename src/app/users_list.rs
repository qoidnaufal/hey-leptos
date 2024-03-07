// use leptos::*;

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub struct OnlineUser {
//     pub uuid: String,
//     pub user_name: String,
// }

// #[server(TrackRoom)]
// async fn track_room(room_uuid: String) -> Result<Vec<OnlineUser>, ServerFnError> {
//     use crate::state::rooms_manager;

//     let rooms_manager = rooms_manager()?;

//     let rooms = rooms_manager.rooms.read().unwrap();
//     let room = rooms.get(&room_uuid).expect("Room does not exist");

//     let online = room
//         .read()
//         .expect("Error in acquiring the lock permission to read")
//         .iter()
//         .map(|(_, user)| OnlineUser {
//             uuid: user.uuid.clone(),
//             user_name: user.user_name.clone(),
//         })
//         .collect::<Vec<_>>();

//     Ok(online)
// }

// #[component]
// fn OnlineList() -> impl IntoView {
//     let _on_load = create_server_action::<TrackRoom>();
//     let resource = create_resource(
//         || window().location().pathname(),
//         |room_uuid| async move { track_room(room_uuid.unwrap()).await },
//     );
//     view! {
//         {move || match resource.get() {
//             None => view! { <p>"Loading..."</p> }.into_view(),
//             Some(data) => view! {
//                 <ul>
//                     { data.unwrap().iter().map(move |user| view! {<li>{user.user_name.clone()}</li>}).collect_view() }
//                 </ul>
//             }.into_view()
//         }}
//     }
// }
