use super::{deserialize_form, ranking_page::RankingPage, AppPage};
use crate::{app::AppMsg, room_state::RoomState, BroadcastMsg};
use axum_live_view::html;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

pub struct VetoPage {
    pub room_code: String,
    pub room_state: Arc<RwLock<RoomState>>,
}
impl VetoPage {
    pub fn new(room_code: String, room_state: Arc<RwLock<RoomState>>) -> Self {
        Self {
            room_code,
            room_state,
        }
    }

    fn get_ranking_page(&mut self) -> Option<Box<dyn AppPage + Send + Sync>> {
        Some(Box::new(RankingPage::new(
            self.room_code.clone(),
            self.room_state.clone(),
        )))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum VetoMsg {
    VetoOption(String),
    VetosUpdated,
    ResetAllVetos,
    FinishVetoing,
    OtherUserFinishedVetoing,
    AddOption,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddOptionFormSubmit {
    option: String,
}

impl AppPage for VetoPage {
    fn update(
        &mut self,
        msg: crate::app::AppMsg,
        data: Option<axum_live_view::event_data::EventData>,
        _server_shared_state: &mut crate::ServerwideSharedState,
        broadcaster: &mut crate::ServerwideBroadcastSender,
    ) -> Option<Box<dyn AppPage + Send + Sync>> {
        if let AppMsg::VetoMsg(msg) = msg {
            match msg {
                VetoMsg::VetoOption(option_to_veto) => {
                    self.room_state.write().unwrap().veto_all(&option_to_veto);
                    broadcaster.send(BroadcastMsg::UpdatedVetos).unwrap();
                }
                VetoMsg::VetosUpdated => (),
                VetoMsg::ResetAllVetos => {
                    self.room_state.write().unwrap().reset_all_vetos();
                    broadcaster.send(BroadcastMsg::UpdatedVetos).unwrap();
                }
                VetoMsg::AddOption => {
                    let option = deserialize_form::<AddOptionFormSubmit>(data)
                        .unwrap()
                        .option;
                    self.room_state.write().unwrap().add_option(option);
                    broadcaster.send(BroadcastMsg::UpdatedVetos).unwrap();
                }
                VetoMsg::FinishVetoing => {
                    self.room_state.write().unwrap().finish_vetoing();
                    broadcaster.send(BroadcastMsg::FinishedVetoing).unwrap();
                    return self.get_ranking_page();
                }
                VetoMsg::OtherUserFinishedVetoing => {
                    return self.get_ranking_page();
                }
            }
        }

        None
    }

    fn render(&self) -> axum_live_view::Html<crate::app::AppMsg> {
        const BUTTON_STYLE: &str = "padding:4px;";
        const BUTTON_TEXT: &str = "X";
        const BUTTON_SPACE: &str = "  ";
        let room_state = self.room_state.read().unwrap();
        html! {
            <div>
                <h1>"It's veto time, baby!"</h1>
                <h3>{format!("Room: {}", self.room_code)}</h3>
                <p>"Any voter can veto any number of options. When the veto phase is finished, all voters proceed to the ranking phase together and can no longer veto. Voters can also add new options here, because why not."</p>
                <h4>"Options"</h4>
                <div>
                    <ol>
                        for option in room_state.options.iter() {
                            <li>
                                if option.vetoed {
                                    // Re-enabling by resetting vetoes doesn't allow re-vetoing unless I include the axm-click here, even though it's not necessary (since the button's disabled)
                                    <s>{option.text.clone()}</s>{BUTTON_SPACE}<button style={BUTTON_STYLE} disabled axm-click={ AppMsg::VetoMsg(VetoMsg::VetoOption(option.text.clone())) }>{BUTTON_TEXT}</button>
                                } else {
                                    {option.text.clone()}{BUTTON_SPACE}<button style={BUTTON_STYLE} axm-click={ AppMsg::VetoMsg(VetoMsg::VetoOption(option.text.clone())) }>{BUTTON_TEXT}</button>
                                }
                            </li>
                        }
                    </ol>

                    <form axm-submit={ AppMsg::VetoMsg(VetoMsg::AddOption) }>
                        <input
                            type="text"
                            name="option"
                            placeholder="New option"
                        />

                        <input type="submit" value="Add Option"/>
                    </form>

                    // This button seemingly has to be beneath the options, otherwise, the options don't get rendered...
                    <button style="font-size:0.75rem;" axm-click={AppMsg::VetoMsg(VetoMsg::ResetAllVetos)}>"Reset all vetos"</button>
                    " "
                    <button axm-click={AppMsg::VetoMsg(VetoMsg::FinishVetoing)}>"Finish Vetoing"</button>
                </div>
            </div>
        }
    }
}
