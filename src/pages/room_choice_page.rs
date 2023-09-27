use axum_live_view::html;
use serde::{Deserialize, Serialize};

use crate::{app::AppMsg, pages::veto_page::VetoPage, room_state::VotingStage};

use super::{deserialize_form, ranking_page::RankingPage, AppPage};

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
        _broadcaster: &mut crate::ServerwideBroadcastSender,
    ) -> (
        Option<Box<dyn AppPage + Send + Sync>>,
        Option<Vec<axum_live_view::js_command::JsCommand>>,
    ) {
        if let AppMsg::RoomChoiceMsg(msg) = msg {
            match msg {
                RoomChoiceMsg::JoinRoom => {
                    let code = deserialize_form::<JoinRoomFormSubmit>(data)
                        .unwrap()
                        .room_code
                        .to_ascii_lowercase();

                    let state = server_shared_state.read().unwrap();
                    if let Some(room) = state.rooms.get(&code) {
                        return match room.read().unwrap().voting_stage() {
                            VotingStage::Vetoing => {
                                (Some(Box::new(VetoPage::new(code, room.clone()))), None)
                            }
                            VotingStage::Ranking => {
                                (Some(Box::new(RankingPage::new(code, room.clone()))), None)
                            }
                        };
                    } else {
                        self.join_error_msg = Some(format!("Room \"{}\" not found", code));
                    }
                }
                RoomChoiceMsg::CreateRoom => {
                    let options_text = deserialize_form::<CreateRoomFormSubmit>(data)
                        .unwrap()
                        .options_text;

                    let mut state = server_shared_state.write().unwrap();
                    if let Ok((room_code, room)) = state.create_room(options_text) {
                        return (Some(Box::new(VetoPage::new(room_code, room.clone()))), None);
                    }
                }
            }
        }

        (None, None)
    }

    fn render(&self) -> axum_live_view::Html<AppMsg> {
        html! {
            <div>
                <h1>"Join Room"</h1>
                <p>"Join an existing room by entering its 4-letter code below."</p>
                <form axm-submit={ AppMsg::RoomChoiceMsg(RoomChoiceMsg::JoinRoom) }>
                    <input
                        type="text"
                        name="room_code"
                        maxlength="4"
                        spellcheck="false"
                        autocorrect="off"
                        placeholder="Room Code"
                    />

                    <input type="submit" value="Join"/>
                </form>
                if let Some(error) = self.join_error_msg.as_ref() {
                    <p>{error}</p>
                }

                <h1>"Create Room"</h1>
                <p>"Create a new room by entering the options below, each on its own line."</p>
                <form axm-submit={ AppMsg::RoomChoiceMsg(RoomChoiceMsg::CreateRoom) }>
                    <textarea name="options_text" rows="10">
                    </textarea>
                    <input type="submit" value="Create Room"/>
                </form>
            </div>
        }
    }
}
