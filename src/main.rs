use axum::{
    http::{HeaderMap, Uri},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use axum_live_view::{
    event_data::EventData,
    html,
    live_view::{Updated, ViewHandle},
    Html, LiveView, LiveViewUpgrade,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let (tx, _) = broadcast::channel::<BroadcastPing>(10);

    let app = Router::new()
        .route("/", get(root))
        // Use a precompiled and minified build of axum-live-view's JavaScript.
        // This is the easiest way to get started. Integration with bundlers
        // is of course also possible.
        .route("/assets/live-view.js", axum_live_view::precompiled_js())
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(SharedState::default()))
                .layer(AddExtensionLayer::new(tx))
                .into_inner(),
        );

    Ok(app.into())
}

type SharedState = Arc<RwLock<State>>;
type BroadcastSender = broadcast::Sender<BroadcastPing>;

#[derive(Clone, Copy, Debug)]
enum BroadcastPing {
    UpdatedCounter,
}

#[derive(Default)]
struct State {
    pub global_count: u64,
}

// Our handler function for `GET /`
async fn root(
    // `LiveViewUpgrade` is an extractor that accepts both regular requests and
    // WebSocket upgrade requests. If it receives a regular request it will
    // render your live view's HTML and return a regular static response. This
    // leads to good SEO and fast first paint.
    //
    // axum-live-view's JavaScript client will then call this endpoint a second
    // time to establish a WebSocket connection at which point your view will be
    // spawned in an async task. Events from the browser and HTML diffs from
    // your view will then be sent over the WebSocket connection.
    //
    // If the WebSocket connection breaks (or your view crashes) the JavaScript
    // client will call this endpoint again to establish a new connection and
    // a new instance of your view is created.
    //
    // The task running the old view automatically stops when the WebSocket is
    // closed.
    live: LiveViewUpgrade,
    Extension(state): Extension<SharedState>,
    Extension(tx): Extension<BroadcastSender>,
) -> impl IntoResponse {
    // `Counter` is our live view and we initialize it with the default values.
    let counter = Counter::new(state, tx);

    live.response(|embed_live_view| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                </head>
                <body>
                    // Embed our live view into the HTML template. This will render the
                    // view and include the HTML in the response, leading to good SEO
                    // and fast first paint.
                    { embed_live_view.embed(counter) }

                    // Load the JavaScript. This will automatically initialize live view
                    // connections.
                    <script src="/assets/live-view.js"></script>
                </body>
            </html>
        }
    })
}

// Our live view is just a regular Rust struct...
struct Counter {
    shared_state: SharedState,
    tx: BroadcastSender,
    count: u64,
    msg: String,
}

impl Counter {
    pub fn new(shared_state: SharedState, tx: BroadcastSender) -> Self {
        Self {
            shared_state,
            tx,
            count: 0,
            msg: "".to_string(),
        }
    }
}

// ...that implements the `LiveView` trait.
impl LiveView for Counter {
    // This is the type of update messages our HTML contains. They will be sent
    // to the view in the `update` method
    type Message = Msg;

    fn mount(&mut self, _: Uri, _: &HeaderMap, handle: ViewHandle<Self::Message>) {
        let mut rx = self.tx.subscribe();
        tokio::spawn(async move {
            while let Ok(BroadcastPing::UpdatedCounter) = rx.recv().await {
                if handle.send(Msg::Update).await.is_err() {
                    break;
                }
            }
        });
    }

    // Update the view based on which message it receives.
    //
    // `EventData` contains data from the event that happened in the
    // browser. This might be values of input fields or which key was pressed in
    // a keyboard event.
    fn update(mut self, msg: Msg, data: Option<EventData>) -> Updated<Self> {
        dbg!(&data);

        match msg {
            Msg::Increment => {
                self.count += 1;
                self.shared_state.write().unwrap().global_count += 1;
                self.tx.send(BroadcastPing::UpdatedCounter).unwrap();
            }
            Msg::Decrement => {
                if self.count > 0 {
                    self.count -= 1;
                    self.shared_state.write().unwrap().global_count -= 1;
                    self.tx.send(BroadcastPing::UpdatedCounter).unwrap();
                }
            }
            Msg::Submit => {
                self.msg = data
                    .unwrap()
                    .as_form()
                    .unwrap()
                    .deserialize::<FormSubmit>()
                    .unwrap()
                    .name;
            }
            Msg::Update => {}
        }

        Updated::new(self)
    }

    // Render the live view into an HTML template. This function is called during
    // the initial render in `LiveViewManager::embed` and for each subsequent
    // update.
    //
    // The HTML is diff'ed on the server and only minimal deltas are sent over
    // the wire. The browser then builds the full HTML template and efficiently
    // updates the DOM.
    fn render(&self) -> Html<Self::Message> {
        let global_count = self.shared_state.read().unwrap().global_count;
        html! {
            <div>
                "Counter value: "
                // Embed dynamic Rust values into the HTML.
                //
                // `if`, `for`, and `match` are also supported.
                { self.count }
                { &self.msg }
            </div>

            <div>
                "Global counter value: "
                {global_count}
            </div>

            <div>
                // Elements with the `axm-click` attribute will send an update message
                // to the view which calls `update` after which the view is
                // re-rendered.
                <button axm-click={ Msg::Increment }>"+"</button>
                <button axm-click={ Msg::Decrement }>"-"</button>
                <form axm-submit={ Msg::Submit }>
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

    // The `LiveView` trait also has a `mount` method that is called when a new
    // WebSocket connects. This can be used to perform auth, load data that
    // isn't needed for the first response, and spawn a task that can send
    // messages to the view itself from other parts of the application.
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Msg {
    Increment,
    Decrement,
    Submit,
    Update,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FormSubmit {
    name: String,
}
