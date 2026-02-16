use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Service to manage active WebSocket connections
#[derive(Clone)]
pub struct NotificationService {
    // Map of user_id -> sender channel
    clients: Arc<Mutex<HashMap<Uuid, mpsc::UnboundedSender<String>>>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a new client connection
    pub async fn add_client(&self, user_id: Uuid, sender: mpsc::UnboundedSender<String>) {
        let mut clients = self.clients.lock().await;
        clients.insert(user_id, sender);
        tracing::info!("✅ User {} connected to WebSocket", user_id);
    }

    /// Remove a client connection
    pub async fn remove_client(&self, user_id: &Uuid) {
        let mut clients = self.clients.lock().await;
        clients.remove(user_id);
        tracing::info!("❌ User {} disconnected from WebSocket", user_id);
    }

    /// Send a message to a specific user (if they're online)
    pub async fn send_to_user(&self, user_id: &Uuid, message: String) {
        let clients = self.clients.lock().await;
        if let Some(sender) = clients.get(user_id) {
            if sender.send(message.clone()).is_ok() {
                tracing::info!("📨 Sent notification to user {}", user_id);
            } else {
                tracing::warn!("⚠️  Failed to send to user {}", user_id);
            }
        } else {
            tracing::debug!("User {} is offline, skipping notification", user_id);
        }
    }
}
