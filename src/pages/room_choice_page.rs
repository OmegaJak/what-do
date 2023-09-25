use axum_live_view::html;
use serde::{Deserialize, Serialize};

use crate::{app::AppMsg, pages::veto_page::VetoPage};

use super::AppPage;

pub struct RoomChoicePage {
    join_error_msg: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RoomChoiceMsg {
    JoinRoom,
    CreateRoom,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinRoomFormSubmit {
    room_code: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRoomFormSubmit {
    options_text: String,
}

impl RoomChoicePage {
    pub fn new() -> Self {
        Self {
            join_error_msg: None,
        }
    }
}

impl AppPage for RoomChoicePage {
    fn update(
        &mut self,
        msg: AppMsg,
        data: Option<axum_live_view::event_data::EventData>,
        server_shared_state: &mut crate::ServerwideSharedState,
        broadcaster: &mut crate::ServerwideBroadcastSender,
    ) -> Option<Box<dyn AppPage + Send + Sync>> {
        if let AppMsg::RoomChoiceMsg(msg) = msg {
            match msg {
                RoomChoiceMsg::JoinRoom => {
                    let code = data
                        .unwrap()
                        .as_form()
                        .unwrap()
                        .deserialize::<JoinRoomFormSubmit>()
                        .unwrap()
                        .room_code
                        .to_ascii_lowercase();

                    let state = server_shared_state.read().unwrap();
                    if let Some(room) = state.rooms.get(&code) {
                        return Some(Box::new(VetoPage {
                            room_code: code,
                            room_state: room.clone(),
                        }));
                    } else {
                        self.join_error_msg = Some(format!("Room \"{}\" not found", code));
                    }
                }
                RoomChoiceMsg::CreateRoom => {
                    let options_text = data
                        .unwrap()
                        .as_form()
                        .unwrap()
                        .deserialize::<CreateRoomFormSubmit>()
                        .unwrap()
                        .options_text;

                    let mut state = server_shared_state.write().unwrap();
                    if let Ok((room_code, room)) = state.create_room(options_text) {
                        return Some(Box::new(VetoPage {
                            room_code,
                            room_state: room.clone(),
                        }));
                    }
                }
            }
        }

        None
    }

    fn render(&self) -> axum_live_view::Html<AppMsg> {
        html! {
            <div>
                <h1>"Join Room"</h1>
                <form axm-submit={ AppMsg::RoomChoiceMsg(RoomChoiceMsg::JoinRoom) }>
                    <input
                        type="text"
                        name="room_code"
                        maxlength="4"
                        placeholder="Room Code"
                    />

                    <input type="submit" value="Join"/>
                </form>
                if let Some(error) = self.join_error_msg.as_ref() {
                    <p>{error}</p>
                }

                <h1>"Create Room"</h1>
                <form axm-submit={ AppMsg::RoomChoiceMsg(RoomChoiceMsg::CreateRoom) }>
                    <textarea name="options_text">
                    </textarea>
                    <input type="submit" value="Create Room"/>
                </form>
            </div>
        }
    }
}
