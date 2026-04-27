// ══════════════════════════════════════════════════════════════════
//  main.rs — AgentMemoryEngine RS Unified Entry Point
//  Rat_Heaven Configuration
// ══════════════════════════════════════════════════════════════════

mod cli;
mod engine;
mod identity;
mod models;
mod persistence;
mod ruview;
mod store;
mod spawner; // <--- The Ghost Factory

use cli::Cli;
use engine::{AgentMemoryEngine, EngineConfig};
use models::DollValue;
use spawner::GhostFactory;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let demo_mode  = args.iter().any(|a| a == "--demo");
    let restore    = args.iter().any(|a| a == "--restore");
    let no_ruview  = args.iter().any(|a| a == "--no-ruview");

    let rdb_path = flag_value(&args, "--rdb").unwrap_or("memory.rdb".to_string());
    let aof_path = flag_value(&args, "--aof").unwrap_or("memory.aof".to_string());
    let snap_interval: u64 = flag_value(&args, "--snapshot-interval")
    .and_then(|v| v.parse().ok())
    .unwrap_or(300);

    print_banner();

    let config = EngineConfig {
        rdb_path,
        aof_path,
        snapshot_interval_secs: snap_interval,
        ruview_enabled: !no_ruview,
        ..Default::default()
    };

    let engine = AgentMemoryEngine::new(config);

    // Explicit restore loop fix
    if restore {
        println!("[info] Attempting to restore state from disk...");
        if let Err(e) = engine.restore() {
            eprintln!("[error] Failed to restore state: {e}");
        } else {
            println!("  ✓ State restored successfully.");
        }
    }

    // Initialize the Ghost Factory if RuView is online
    if !no_ruview {
        if let Some(ruview_arc) = engine.get_ruview() {
            let mut factory = GhostFactory::new(Arc::clone(&engine), ruview_arc);
            tokio::spawn(async move {
                factory.start_spawning_loop().await;
            });
        }
    }

    if demo_mode {
        run_demo(Arc::clone(&engine));
    } else {
        let mut cli = Cli::new(Arc::clone(&engine));
        if let Err(e) = cli.run() {
            eprintln!("CLI error: {e}");
        }
    }
}

// ... [Keep your existing run_demo, print_banner, and flag_value functions here exactly as they were in main2.rs] ...
