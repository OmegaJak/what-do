use axum_live_view::html;

use super::AppPage;

pub struct ErrorPage {
    error: anyhow::Error,
}

impl ErrorPage {
    pub fn new(error: anyhow::Error) -> Self {
        Self { error }
    }
}

impl AppPage for ErrorPage {
    fn update(
        &mut self,
        _msg: crate::app::AppMsg,
        _data: Option<axum_live_view::event_data::EventData>,
        _server_shared_state: &mut crate::ServerwideSharedState,
        _broadcast_rx_tx: &mut crate::BroadcastReceiverSender,
    ) -> anyhow::Result<super::AppUpdateResponse> {
        Ok((None, None).into())
    }

    fn render(&self) -> axum_live_view::Html<crate::app::AppMsg> {
        html! {
            <div>
                <p>{ammonia::clean_text(&format!("Encountered an error: {}", self.error))}</p>
            </div>
        }
    }
}
