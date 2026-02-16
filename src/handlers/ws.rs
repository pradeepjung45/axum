use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;

use crate::routes::auth_routes::AppState;
use crate::middleware::auth::get_user_from_cookie;

/// WebSocket handler - upgrades HTTP to WebSocket
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    cookies: axum_extra::extract::CookieJar,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    // Extract user from cookie
    let user_id = match get_user_from_cookie(&cookies, &state.jwt_secret) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                axum::http::StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ));
        }
    };

    // Upgrade the connection
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

/// Handle the WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState, user_id: uuid::Uuid) {
    let (mut sender, mut receiver) = socket.split();
    
    // Create a channel for this client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    
    // Register this client
    state.notification_service.add_client(user_id, tx).await;

    // Task to send messages to the client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task to receive messages from the client (mostly just keep-alive pings)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, axum::extract::ws::Message::Close(_)) {
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Clean up: remove client from the map
    state.notification_service.remove_client(&user_id).await;
}
