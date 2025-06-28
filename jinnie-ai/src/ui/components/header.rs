use dioxus::prelude::*;
use crate::ui::{
    theme::{JINNIE_THEME, button_styles},
    state::ui_state::UIState,
};

pub fn Header(cx: Scope) -> Element {
    let ui_state = use_shared_state::<UIState>(cx)?;
    
    cx.render(rsx! {
        header {
            style: "
                height: 64px;
                background: {JINNIE_THEME.surface};
                border-bottom: 1px solid {JINNIE_THEME.border};
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 0 1.5rem;
                flex-shrink: 0;
            ",
            
            // Left section - Logo and current agent
            div {
                style: "display: flex; align-items: center; gap: 1.5rem;",
                
                // Logo and title
                div {
                    style: "display: flex; align-items: center; gap: 0.75rem;",
                    
                    // Rust logo/icon
                    div {
                        style: "
                            width: 32px;
                            height: 32px;
                            background: linear-gradient(135deg, {JINNIE_THEME.primary}, {JINNIE_THEME.accent});
                            border-radius: 8px;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            font-weight: bold;
                            color: white;
                            font-size: 18px;
                        ",
                        "🦀"
                    }
                    
                    h1 {
                        style: "
                            font-size: 1.25rem;
                            font-weight: 600;
                            color: {JINNIE_THEME.text_primary};
                            margin: 0;
                        ",
                        "Jinnie.ai"
                    }
                }
                
                // Current agent indicator
                if let Some(agent) = ui_state.read().get_current_agent() {
                    rsx! {
                        div {
                            style: "
                                display: flex;
                                align-items: center;
                                gap: 0.5rem;
                                padding: 0.5rem 1rem;
                                background: {JINNIE_THEME.bg_secondary};
                                border-radius: 1rem;
                                border: 1px solid {JINNIE_THEME.border};
                            ",
                            
                            div {
                                style: "
                                    width: 8px;
                                    height: 8px;
                                    background: {JINNIE_THEME.success};
                                    border-radius: 50%;
                                ",
                            }
                            
                            span {
                                style: "
                                    font-size: 0.875rem;
                                    color: {JINNIE_THEME.text_secondary};
                                    font-weight: 500;
                                ",
                                "{agent.name}"
                            }
                            
                            span {
                                style: "
                                    font-size: 0.75rem;
                                    color: {JINNIE_THEME.text_muted};
                                    background: {JINNIE_THEME.primary}22;
                                    padding: 0.125rem 0.5rem;
                                    border-radius: 0.375rem;
                                ",
                                "{agent.model}"
                            }
                        }
                    }
                }
            }
            
            // Center section - Chat title
            div {
                style: "flex: 1; display: flex; justify-content: center;",
                
                if let Some(chat) = ui_state.read().get_current_chat() {
                    rsx! {
                        h2 {
                            style: "
                                font-size: 1rem;
                                font-weight: 500;
                                color: {JINNIE_THEME.text_secondary};
                                margin: 0;
                                text-align: center;
                            ",
                            "{chat.title}"
                        }
                    }
                } else {
                    rsx! {
                        span {
                            style: "
                                font-size: 1rem;
                                color: {JINNIE_THEME.text_muted};
                                text-align: center;
                            ",
                            "Start a new conversation"
                        }
                    }
                }
            }
            
            // Right section - Actions
            div {
                style: "display: flex; align-items: center; gap: 0.75rem;",
                
                // New chat button
                button {
                    style: "{button_styles(\"primary\")}",
                    onclick: move |_| {
                        ui_state.write().create_new_chat("New Chat".to_string());
                    },
                    
                    span { "+" }
                    span {
                        style: "margin-left: 0.5rem;",
                        "New Chat"
                    }
                }
                
                // Settings button
                button {
                    style: "{button_styles(\"ghost\")}",
                    title: "Settings",
                    
                    "⚙️"
                }
                
                // Toggle sidebars
                button {
                    style: "{button_styles(\"ghost\")}",
                    title: "Toggle Sidebar",
                    onclick: move |_| {
                        ui_state.write().toggle_sidebar();
                    },
                    
                    "☰"
                }
            }
        }
    })
}