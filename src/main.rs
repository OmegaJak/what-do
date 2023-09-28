use app::App;
use axum::{extract::Path, response::IntoResponse, routing::get, Extension, Router};
use axum_live_view::{html, LiveViewUpgrade};
use pages::{room_choice_page::RoomChoicePage, AppPage};
use server_state::ServerState;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use tracing::info;
use tracing_panic::panic_hook;

pub mod app;
pub mod pages;
pub mod room_state;
pub mod server_state;

pub type ServerwideSharedState = Arc<RwLock<ServerState>>;
pub type ServerwideBroadcastSender = broadcast::Sender<BroadcastMsg>;

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    std::panic::set_hook(Box::new(panic_hook));
    info!("Starting server!");
    let (tx, _) = broadcast::channel::<BroadcastMsg>(10);

    let app = Router::new()
        .route("/", get(root))
        .route("/room/:room_code", get(room))
        .route("/room/:room_code/results", get(room_results))
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
    UpdatedVetos,
    FinishedVetoing,
    UpdatedVotes,
}

async fn root(
    live: LiveViewUpgrade,
    Extension(state): Extension<ServerwideSharedState>,
    Extension(tx): Extension<ServerwideBroadcastSender>,
) -> impl IntoResponse {
    let app = App::new(state, tx, Box::new(RoomChoicePage::default()));
    live_view_response(live, app)
}

async fn room(
    Path(room_code): Path<String>,
    live: LiveViewUpgrade,
    Extension(state): Extension<ServerwideSharedState>,
    Extension(tx): Extension<ServerwideBroadcastSender>,
) -> impl IntoResponse {
    live_view_response(
        live,
        get_app_starting_on_room_page(room_code, state, tx, ServerState::get_room_voting_page),
    )
}

async fn room_results(
    Path(room_code): Path<String>,
    live: LiveViewUpgrade,
    Extension(state): Extension<ServerwideSharedState>,
    Extension(tx): Extension<ServerwideBroadcastSender>,
) -> impl IntoResponse {
    live_view_response(
        live,
        get_app_starting_on_room_page(room_code, state, tx, ServerState::get_room_results_page),
    )
}

fn get_app_starting_on_room_page(
    room_code: String,
    state: ServerwideSharedState,
    tx: ServerwideBroadcastSender,
    get_room_page: impl FnOnce(&ServerState, &str) -> Result<Box<dyn AppPage + Send + Sync>, String>,
) -> App {
    let starting_page = get_room_page(&state.read().unwrap(), &room_code)
        .unwrap_or_else(|e| Box::new(RoomChoicePage::new(Some(e))));
    App::new(state, tx, starting_page)
}

fn live_view_response(live: LiveViewUpgrade, app: App) -> impl IntoResponse {
    live.response(|embed_live_view| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <link rel="stylesheet" href="https://cdn.simplecss.org/simple.min.css"/>
                    <style>
                        """body {
                            font-family: monospace,system-ui;
                        }
                        """
                    </style>
                </head>
                <body>
                    <script src="https://cdn.jsdelivr.net/npm/sortablejs@latest/Sortable.min.js"></script>
                    <script type="text/javascript">
                    {include_str!("../assets/main.js")}
                    </script>
                    { embed_live_view.embed(app) }
                    <script src="/assets/live-view.js"></script>
                </body>
            </html>
        }
    })
}
