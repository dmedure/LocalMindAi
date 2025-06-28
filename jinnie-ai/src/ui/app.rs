use dioxus::prelude::*;
use crate::ui::{
    components::{
        header::Header,
        sidebar::{HistorySidebar, AgentSidebar},
        chat::ChatContainer,
    },
    theme::{global_styles, JINNIE_THEME},
    state::ui_state::UIState,
};

pub fn App() -> Element {
    // Initialize UI state as a context signal
    use_context_provider(|| Signal::new(UIState::new()));
    
    rsx! {
        style { "{global_styles()}" }
        
        div {
            class: "app",
            style: "
                display: flex; 
                height: 100vh; 
                overflow: hidden; 
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
                background: {JINNIE_THEME.bg_primary};
            ",
            
            // History Sidebar (left)
            HistorySidebar {}
            
            // Main Content Area (center)
            div {
                class: "main-content",
                style: "
                    flex: 1; 
                    display: flex; 
                    flex-direction: column; 
                    background: {JINNIE_THEME.bg_primary}; 
                    min-width: 0;
                ",
                
                // Header
                Header {}
                
                // Chat Container
                ChatContainer {}
            }
            
            // Agent Sidebar (right)
            AgentSidebar {}
        }
    }
}