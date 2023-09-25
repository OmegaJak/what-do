use axum::http::{HeaderMap, Uri};
use axum_live_view::{
    event_data::EventData,
    html,
    live_view::{Updated, ViewHandle},
    Html, LiveView,
};
use serde::{Deserialize, Serialize};

use crate::{ServerwideBroadcastSender, ServerwideSharedState, BroadcastMsg};

pub struct App {
    shared_state: ServerwideSharedState,
    tx: ServerwideBroadcastSender,
    count: u64,
    msg: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum AppMsg {
    Increment,
    Decrement,
    Submit,
    Update,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormSubmit {
    name: String,
}

impl App {
    pub fn new(shared_state: ServerwideSharedState, tx: ServerwideBroadcastSender) -> Self {
        Self {
            shared_state,
            tx,
            count: 0,
            msg: "".to_string(),
        }
    }
}

impl LiveView for App {
    type Message = AppMsg;

    fn mount(&mut self, _: Uri, _: &HeaderMap, handle: ViewHandle<Self::Message>) {
        let mut rx = self.tx.subscribe();
        tokio::spawn(async move {
            while let Ok(BroadcastMsg::UpdatedCounter) = rx.recv().await {
                if handle.send(AppMsg::Update).await.is_err() {
                    break;
                }
            }
        });
    }

    fn update(mut self, msg: AppMsg, data: Option<EventData>) -> Updated<Self> {
        match msg {
            AppMsg::Increment => {
                self.count = self.count.saturating_add(1);

                let mut state = self.shared_state.write().unwrap();
                state.global_count = state.global_count.saturating_add(1);

                self.tx.send(BroadcastMsg::UpdatedCounter).unwrap();
            }
            AppMsg::Decrement => {
                self.count = self.count.saturating_sub(1);

                let mut state = self.shared_state.write().unwrap();
                state.global_count = state.global_count.saturating_sub(1);

                self.tx.send(BroadcastMsg::UpdatedCounter).unwrap();
            }
            AppMsg::Submit => {
                self.msg = data
                    .unwrap()
                    .as_form()
                    .unwrap()
                    .deserialize::<FormSubmit>()
                    .unwrap()
                    .name;
            }
            AppMsg::Update => {}
        }

        Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        let global_count = self.shared_state.read().unwrap().global_count;
        html! {
            <div>
                "Counter value: "
                { self.count }
                { &self.msg }
            </div>

            <div>
                "Global counter value: "
                {global_count}
            </div>

            <div>
                <button axm-click={ AppMsg::Increment }>"+"</button>
                <button axm-click={ AppMsg::Decrement }>"-"</button>
                <form axm-submit={ AppMsg::Submit }>
                    <input
                        type="text"
                        name="name"
                        placeholder="Your name"
                    />

                    <input
                        type="submit"
                        value="Send!"
                    />
                </form>
            </div>
        }
    }
}