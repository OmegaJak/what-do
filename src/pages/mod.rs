use crate::{app::AppMsg, ServerState, ServerwideBroadcastSender, ServerwideSharedState};
use axum_live_view::{event_data::EventData, Html};

pub mod room_choice_page;
pub mod veto_page;

pub trait AppPage {
    fn update(
        &mut self,
        msg: AppMsg,
        data: Option<EventData>,
        server_shared_state: &mut ServerwideSharedState,
        broadcaster: &mut ServerwideBroadcastSender,
    ) -> Option<Box<dyn AppPage + Send + Sync>>;
    fn render(&self) -> Html<AppMsg>;
}
