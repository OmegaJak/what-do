use super::AppPage;
use crate::room_state::RoomState;
use axum_live_view::html;
use std::sync::{Arc, RwLock};

pub struct VetoPage {
    pub room_code: String,
    pub room_state: Arc<RwLock<RoomState>>,
}

impl AppPage for VetoPage {
    fn update(
        &mut self,
        msg: crate::app::AppMsg,
        data: Option<axum_live_view::event_data::EventData>,
        server_shared_state: &mut crate::ServerwideSharedState,
        broadcaster: &mut crate::ServerwideBroadcastSender,
    ) -> Option<Box<dyn AppPage + Send + Sync>> {
        None
    }

    fn render(&self) -> axum_live_view::Html<crate::app::AppMsg> {
        html! {
            <div>
                <h1>{format!("Room: {}", self.room_code)}</h1>
                <h2>"It's veto time, baby!"</h2>
            </div>
        }
    }
}
