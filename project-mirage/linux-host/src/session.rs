use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub peer_node_id: String,
    pub peer_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub mouse_owner: MouseOwner,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseOwner {
    Local,
    Remote,
}

pub struct SessionManager {
    config: Config,
    node_name: String,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionManager {
    pub async fn new(config: Config, node_name: String) -> Result<Self> {
        Ok(Self {
            config,
            node_name,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(self) -> Result<()> {
        info!("Session manager running...");
        
        // Main session management loop
        // This will handle:
        // - Session lifecycle management
        // - Heartbeat monitoring
        // - Mouse ownership transfers
        // - Stream coordination (Phase 0.2+)
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            // Clean up expired sessions
            let mut sessions = self.sessions.write().await;
            let now = chrono::Utc::now();
            let timeout = chrono::Duration::minutes(self.config.security.session_timeout_minutes as i64);
            
            sessions.retain(|_, session| {
                let elapsed = now - session.last_activity;
                if elapsed > timeout {
                    debug!("Session {} timed out", session.session_id);
                    false
                } else {
                    true
                }
            });
        }
    }

    pub async fn create_session(&self, peer_node_id: String, peer_name: String) -> Result<Session> {
        let session = Session {
            session_id: Uuid::new_v4().to_string(),
            peer_node_id: peer_node_id.clone(),
            peer_name: peer_name.clone(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            mouse_owner: MouseOwner::Local,
        };

        info!("Created session {} with peer {}", session.session_id, peer_name);
        
        self.sessions.write().await.insert(session.session_id.clone(), session.clone());
        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }

    pub async fn update_activity(&self, session_id: &str) {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.last_activity = chrono::Utc::now();
        }
    }

    pub async fn transfer_mouse(&self, session_id: &str, owner: MouseOwner) -> Result<()> {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.mouse_owner = owner;
            info!("Mouse ownership transferred to {:?} for session {}", owner, session_id);
        }
        Ok(())
    }

    pub async fn close_session(&self, session_id: &str) {
        if let Some(session) = self.sessions.write().await.remove(session_id) {
            info!("Closed session {} with peer {}", session.session_id, session.peer_name);
        }
    }
}
