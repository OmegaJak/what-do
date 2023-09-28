use super::{deserialize_form, AppPage};
use crate::{app::AppMsg, pages::veto_page::VetoPage};
use axum::http::Uri;
use axum_live_view::{html, js_command};
use serde::{Deserialize, Serialize};

#[derive(Default)]
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
    pub fn new(join_error_msg: Option<String>) -> Self {
        Self { join_error_msg }
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
                    match state.get_room_voting_page(&code) {
                        Ok(page) => {
                            return (
                                Some(page),
                                Some(vec![js_command::history_push_state(room_uri(&code))]),
                            )
                        }
                        Err(msg) => self.join_error_msg = Some(msg),
                    }
                }
                RoomChoiceMsg::CreateRoom => {
                    let options_text = deserialize_form::<CreateRoomFormSubmit>(data)
                        .unwrap()
                        .options_text;

                    let mut state = server_shared_state.write().unwrap();
                    if let Ok((room_code, room)) = state.create_room(options_text) {
                        let cmd = js_command::history_push_state(room_uri(&room_code));
                        return (
                            Some(Box::new(VetoPage::new(room_code, room.clone()))),
                            Some(vec![cmd]),
                        );
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
                <p>{self.join_error_msg.as_ref().map_or("", |s| &*s)}</p> // Doing this instead of the more intuitive `if let Some ...` approach as that causes a strange bug that turns things into <p> els on the next page

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

fn room_uri(room_code: &str) -> Uri {
    format!("/room/{}", room_code).parse().unwrap()
}
