// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use local_ai_agent::{initialize_app, commands::*};
use tauri::Manager;

fn main() {
    // Initialize runtime
    let runtime = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime");

    // Initialize application
    let app_state = runtime.block_on(async {
        initialize_app().await
            .expect("Failed to initialize LocalMind")
    });

    // Build Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Agent commands
            get_agents,
            create_agent,
            update_agent,
            delete_agent,
            
            // Chat commands
            send_message_to_agent,
            get_agent_messages,
            clear_chat,
            
            // Memory commands
            search_memories,
            get_memory_stats,
            trigger_consolidation,
            
            // Document commands
            add_document,
            get_documents,
            delete_document,
            
            // System commands
            check_service_status,
            get_system_info,
            export_agent_knowledge,
            import_agent_knowledge,
        ])
        .setup(|app| {
            // Set up any runtime configuration
            let window = app.get_window("main").unwrap();
            
            #[cfg(debug_assertions)]
            {
                window.open_devtools();
            }
            
            Ok(())
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                // Perform cleanup before closing
                // This is where you might save state, close connections, etc.
                log::info!("Application closing, performing cleanup...");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}