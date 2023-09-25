use axum::{extract::ws::WebSocketUpgrade, response::Html, routing::get, Router};
use dioxus::prelude::*;

fn app(cx: Scope) -> Element {
    let mut num = use_state(cx, || 0);

    cx.render(rsx! {
        div {
            "hello axum! {num}"
            button { onclick: move |_| num += 1, "Increment" }
        }
    })
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let websocket_addr: String = if cfg!(debug_assertions) {
        "ws://127.0.0.1:8000/ws".into()
    } else {
        "wss://group-activity-decider-test.shuttleapp.rs/ws".into()
    };

    let view = dioxus_liveview::LiveViewPool::new();

    let app = Router::new()
        .route(
            "/",
            get(move || async move {
                Html(format!(
                    r#"
            <!DOCTYPE html>
            <html>
                <head> <title>Dioxus LiveView with axum</title>  </head>
                <body> <div id="main"></div> </body>
                {glue}
            </html>
            "#,
                    glue = dioxus_liveview::interpreter_glue(&websocket_addr)
                ))
            }),
        )
        .route(
            "/ws",
            get(move |ws: WebSocketUpgrade| async move {
                ws.on_upgrade(move |socket| async move {
                    _ = view.launch(dioxus_liveview::axum_socket(socket), app).await;
                })
            }),
        );

    // println!("Listening on http://{addr}");

    // axum::Server::bind(&addr.to_string().parse().unwrap())
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();

    Ok(app.into())
}