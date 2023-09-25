use app::App;
use axum::{
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use axum_live_view::{LiveViewUpgrade, html};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

pub mod app;

pub type ServerwideSharedState = Arc<RwLock<ServerState>>;
pub type ServerwideBroadcastSender = broadcast::Sender<BroadcastMsg>;

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let (tx, _) = broadcast::channel::<BroadcastMsg>(10);

    let app = Router::new()
        .route("/", get(root))
        .route("/assets/live-view.js", axum_live_view::precompiled_js())
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(ServerwideSharedState::default()))
                .layer(AddExtensionLayer::new(tx))
                .into_inner(),
        );

    Ok(app.into())
}

#[derive(Clone, Copy, Debug)]
pub enum BroadcastMsg {
    UpdatedCounter,
}

#[derive(Default)]
pub struct ServerState {
    pub global_count: u64,
}

async fn root(
    live: LiveViewUpgrade,
    Extension(state): Extension<ServerwideSharedState>,
    Extension(tx): Extension<ServerwideBroadcastSender>,
) -> impl IntoResponse {
    let counter = App::new(state, tx);

    live.response(|embed_live_view| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/water.min.css"/>
                </head>
                <body>
                    { embed_live_view.embed(counter) }
                    <script src="/assets/live-view.js"></script>
                </body>
            </html>
        }
    })
}
