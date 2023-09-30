use crate::{app::AppMsg, ServerwideSharedState};
use anyhow::anyhow;
use anyhow::Result;
use axum_live_view::{event_data::EventData, Html};

pub mod error_page;
pub mod ranking_page;
pub mod results_page;
pub mod room_choice_page;
pub mod veto_page;

pub trait AppPage {
    fn update(
        &mut self,
        msg: AppMsg,
        data: Option<EventData>,
        server_shared_state: &mut ServerwideSharedState,
        broadcast_rx_tx: &mut crate::BroadcastReceiverSender,
    ) -> Result<AppUpdateResponse>;
    fn render(&self) -> Html<AppMsg>;
}

pub struct AppUpdateResponse {
    pub next_page: Option<Box<dyn AppPage + Send + Sync>>,
    pub js_commands: Option<Vec<axum_live_view::js_command::JsCommand>>,
}

impl
    From<(
        Option<Box<dyn AppPage + Send + Sync>>,
        Option<Vec<axum_live_view::js_command::JsCommand>>,
    )> for AppUpdateResponse
{
    fn from(
        (next_page, js_commands): (
            Option<Box<dyn AppPage + Send + Sync>>,
            Option<Vec<axum_live_view::js_command::JsCommand>>,
        ),
    ) -> Self {
        Self {
            next_page,
            js_commands,
        }
    }
}

pub fn deserialize_form<T>(data: Option<axum_live_view::event_data::EventData>) -> Result<T>
where
    for<'de> T: serde::Deserialize<'de>,
{
    Ok(data
        .ok_or(anyhow!("Missing event data"))?
        .as_form()
        .ok_or(anyhow!("Event data was not a form"))?
        .deserialize::<T>()?)
}
