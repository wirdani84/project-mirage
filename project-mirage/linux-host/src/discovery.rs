use anyhow::{Context, Result};
use mdns_sd::{ServiceDaemon, ServiceInfo, ServiceEvent};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug, error};
use uuid::Uuid;

use crate::config::Config;

const SERVICE_TYPE: &str = "_mirage._tcp.local.";

#[derive(Debug, Clone)]
pub struct PeerDevice {
    pub node_id: String,
    pub node_name: String,
    pub os_type: String,
    pub ip_address: IpAddr,
    pub control_port: u16,
    pub capabilities: PeerCapabilities,
    pub last_seen: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct PeerCapabilities {
    pub can_host_mouse: bool,
    pub can_capture_windows: bool,
    pub can_render_streams: bool,
    pub video_codecs: Vec<String>,
}

pub struct DiscoveryService {
    config: Config,
    node_id: String,
    node_name: String,
    daemon: ServiceDaemon,
    peers: Arc<RwLock<HashMap<String, PeerDevice>>>,
    event_tx: mpsc::Sender<DiscoveryEvent>,
    event_rx: mpsc::Receiver<DiscoveryEvent>,
}

#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    PeerDiscovered(PeerDevice),
    PeerUpdated(PeerDevice),
    PeerLost(String),
}

impl DiscoveryService {
    pub async fn new(config: Config, node_name: String) -> Result<Self> {
        let node_id = Uuid::new_v4().to_string();
        let daemon = ServiceDaemon::new().context("Failed to create mDNS daemon")?;
        let (event_tx, event_rx) = mpsc::channel(100);

        Ok(Self {
            config,
            node_id,
            node_name,
            daemon,
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        // Register our service
        self.register_service().await?;
        
        // Start browsing for peers
        self.browse_services().await?;
        
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        // Unregister service
        self.daemon.shutdown().context("Failed to shutdown mDNS daemon")?;
        Ok(())
    }

    async fn register_service(&self) -> Result<()> {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "linux-host".to_string());

        let service_name = format!("{}._mirage", self.node_name);
        let port = self.config.network.control_port;

        // Get local IP address
        let local_ip = get_local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        let mut properties = HashMap::new();
        properties.insert("node_id".to_string(), self.node_id.clone());
        properties.insert("os_type".to_string(), "linux".to_string());
        properties.insert("can_host_mouse".to_string(), "true".to_string());
        properties.insert("can_capture_windows".to_string(), "true".to_string());
        properties.insert("can_render_streams".to_string(), "true".to_string());
        properties.insert("video_codecs".to_string(), "h264,h265".to_string());

        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &service_name,
            &hostname,
            local_ip,
            port,
            Some(properties),
        )?;

        self.daemon.register(service_info)
            .context("Failed to register mDNS service")?;

        info!("✓ Registered service: {} at {}:{}", service_name, local_ip, port);
        Ok(())
    }

    async fn browse_services(&mut self) -> Result<()> {
        let receiver = self.daemon.browse(SERVICE_TYPE)
            .context("Failed to browse for mDNS services")?;

        let peers = Arc::clone(&self.peers);
        let event_tx = self.event_tx.clone();
        let node_id = self.node_id.clone();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        debug!("Service resolved: {:?}", info);
                        
                        if let Some(peer) = Self::parse_service_info(&info, &node_id) {
                            info!("🔍 Discovered peer: {} ({}) at {}:{}", 
                                peer.node_name, peer.os_type, peer.ip_address, peer.control_port);
                            
                            let mut peers_lock = peers.write().await;
                            let is_new = !peers_lock.contains_key(&peer.node_id);
                            peers_lock.insert(peer.node_id.clone(), peer.clone());
                            drop(peers_lock);

                            let event = if is_new {
                                DiscoveryEvent::PeerDiscovered(peer)
                            } else {
                                DiscoveryEvent::PeerUpdated(peer)
                            };
                            
                            let _ = event_tx.send(event).await;
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        debug!("Service removed: {}", fullname);
                        
                        let mut peers_lock = peers.write().await;
                        if let Some((node_id, peer)) = peers_lock.iter()
                            .find(|(_, p)| fullname.contains(&p.node_name))
                            .map(|(k, v)| (k.clone(), v.clone()))
                        {
                            info!("👋 Peer lost: {} ({})", peer.node_name, peer.os_type);
                            peers_lock.remove(&node_id);
                            let _ = event_tx.send(DiscoveryEvent::PeerLost(node_id)).await;
                        }
                    }
                    ServiceEvent::SearchStarted(_) => {
                        debug!("Search started");
                    }
                    ServiceEvent::SearchStopped(_) => {
                        debug!("Search stopped");
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    fn parse_service_info(info: &ServiceInfo, our_node_id: &str) -> Option<PeerDevice> {
        let properties = info.get_properties();
        
        let node_id = properties.get("node_id")?.to_string();
        
        // Don't discover ourselves
        if node_id == our_node_id {
            return None;
        }

        let node_name = info.get_fullname()
            .split('.')
            .next()?
            .trim_start_matches('_')
            .to_string();

        let os_type = properties.get("os_type")?.to_string();
        let ip_address = *info.get_addresses().iter().next()?;
        let control_port = info.get_port();

        let can_host_mouse = properties.get("can_host_mouse")
            .map(|v| v == "true")
            .unwrap_or(false);
        
        let can_capture_windows = properties.get("can_capture_windows")
            .map(|v| v == "true")
            .unwrap_or(false);
        
        let can_render_streams = properties.get("can_render_streams")
            .map(|v| v == "true")
            .unwrap_or(false);

        let video_codecs = properties.get("video_codecs")
            .map(|v| v.split(',').map(String::from).collect())
            .unwrap_or_default();

        Some(PeerDevice {
            node_id,
            node_name,
            os_type,
            ip_address,
            control_port,
            capabilities: PeerCapabilities {
                can_host_mouse,
                can_capture_windows,
                can_render_streams,
                video_codecs,
            },
            last_seen: std::time::Instant::now(),
        })
    }

    pub async fn get_peers(&self) -> Vec<PeerDevice> {
        self.peers.read().await.values().cloned().collect()
    }

    pub async fn get_peer(&self, node_id: &str) -> Option<PeerDevice> {
        self.peers.read().await.get(node_id).cloned()
    }
}

fn get_local_ip() -> Option<IpAddr> {
    // Try to get a non-loopback IP address
    local_ip_address::local_ip().ok()
}
