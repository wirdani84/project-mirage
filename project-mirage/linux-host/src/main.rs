use anyhow::Result;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod discovery;
mod input;
mod session;
mod capture;
mod network;
mod security;

use config::Config;
use discovery::DiscoveryService;
use input::InputManager;
use session::SessionManager;

#[derive(Parser, Debug)]
#[command(name = "mirage-host")]
#[command(about = "Project Mirage - Linux Host Daemon", long_about = None)]
struct Args {
    /// Enable discovery mode to find peer devices
    #[arg(short, long)]
    discover: bool,

    /// Configuration file path
    #[arg(short, long, default_value = "~/.config/mirage/config.toml")]
    config: String,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Node name (defaults to hostname)
    #[arg(short, long)]
    name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("mirage_host={},mirage={}", log_level, log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🌟 Project Mirage - Linux Host Daemon v{}", env!("CARGO_PKG_VERSION"));
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Load configuration
    let config = Config::load(&args.config).await?;
    info!("✓ Configuration loaded from {}", args.config);

    // Determine node name
    let node_name = args.name
        .or_else(|| config.host.name.clone())
        .unwrap_or_else(|| {
            hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "linux-host".to_string())
        });
    
    info!("✓ Node name: {}", node_name);

    // Initialize input manager (Phase 0.1 - Mouse sharing)
    info!("Initializing input manager...");
    let input_manager = InputManager::new(config.clone())?;
    info!("✓ Input manager ready");

    // Initialize session manager
    info!("Initializing session manager...");
    let session_manager = SessionManager::new(config.clone(), node_name.clone()).await?;
    info!("✓ Session manager ready");

    if args.discover {
        // Start discovery service
        info!("Starting mDNS discovery service...");
        let mut discovery = DiscoveryService::new(config.clone(), node_name.clone()).await?;
        
        info!("✓ Discovery service started");
        info!("🔍 Scanning for peer devices on local network...");
        info!("   Press Ctrl+C to exit");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Start discovery
        discovery.start().await?;

        // Wait for Ctrl+C
        tokio::signal::ctrl_c().await?;
        
        info!("\n🛑 Shutting down...");
        discovery.stop().await?;
    } else {
        // Normal daemon mode
        info!("Starting Mirage Host Daemon in normal mode...");
        info!("✓ Daemon ready");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Run the main event loop
        run_daemon(input_manager, session_manager).await?;
    }

    info!("✓ Mirage Host Daemon stopped");
    Ok(())
}

async fn run_daemon(
    input_manager: InputManager,
    session_manager: SessionManager,
) -> Result<()> {
    // Main daemon event loop
    // This will handle:
    // - Mouse event capture and forwarding
    // - Incoming control messages
    // - Window capture and streaming (Phase 0.2+)
    
    info!("Daemon running. Press Ctrl+C to exit.");
    
    // Spawn input monitoring task
    let input_handle = tokio::spawn(async move {
        if let Err(e) = input_manager.run().await {
            error!("Input manager error: {}", e);
        }
    });

    // Spawn session management task
    let session_handle = tokio::spawn(async move {
        if let Err(e) = session_manager.run().await {
            error!("Session manager error: {}", e);
        }
    });

    // Wait for Ctrl+C or task completion
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = input_handle => {
            error!("Input manager task terminated");
        }
        _ = session_handle => {
            error!("Session manager task terminated");
        }
    }

    Ok(())
}
