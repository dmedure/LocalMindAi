use dioxus::prelude::*;
use crate::{
    ui::{
        theme::{JINNIE_THEME, input_styles, button_styles},
        state::ui_state::{UIState, MessageRole},
    },
    use_app_state, AppState,
};

pub mod message_list;
pub mod message_item;
pub mod typing_indicator;

use message_list::MessageList;
use typing_indicator::TypingIndicator;

pub fn ChatContainer() -> Element {
    let mut ui_state = use_context::<Signal<UIState>>();
    let app_state = use_app_state();
    let mut input_value = use_signal(|| String::new());
    
    // Initialize agents on first load
    use_effect(move || {
        let app_state = app_state.clone();
        let mut ui_state = ui_state.clone();
        
        async move {
            ui_state.write().set_loading_agents(true);
            
            match app_state.get_agents().await {
                Ok(agents) => {
                    ui_state.write().load_agents(agents);
                    ui_state.write().set_error(None);
                }
                Err(e) => {
                    ui_state.write().set_error(Some(format!("Failed to load agents: {}", e)));
                    log::error!("Failed to load agents: {}", e);
                }
            }
            
            ui_state.write().set_loading_agents(false);
        }
    });

    // Handle sending messages
    let send_message = move |_| {
        let message_content = input_value.read().trim().to_string();
        if message_content.is_empty() {
            return;
        }

        let current_agent_id = ui_state.read().current_agent_id.clone();
        if current_agent_id.is_empty() {
            ui_state.write().set_error(Some("No agent selected".to_string()));
            return;
        }

        // Add user message immediately
        ui_state.write().add_message(message_content.clone(), MessageRole::User);
        
        // Clear input
        input_value.set(String::new());
        
        // Set AI typing indicator
        ui_state.write().set_typing(true);
        ui_state.write().set_error(None);

        // Send message to backend
        let app_state = app_state.clone();
        let mut ui_state = ui_state.clone();
        
        app_state.spawn(async move {
            match app_state.send_message_to_agent(current_agent_id, message_content).await {
                Ok(response) => {
                    // Add AI response
                    ui_state.write().add_message(response, MessageRole::Assistant);
                }
                Err(e) => {
                    ui_state.write().set_error(Some(format!("Failed to send message: {}", e)));
                    log::error!("Failed to send message: {}", e);
                }
            }
            
            // Stop typing indicator
            ui_state.write().set_typing(false);
        });
    };

    // Handle Enter key press
    let handle_keypress = move |event: KeyboardEvent| {
        if event.key() == Key::Enter && !event.shift_key() {
            event.prevent_default();
            send_message(());
        }
    };

    // Get current state values
    let messages = ui_state.read().messages.clone();
    let is_typing = ui_state.read().is_ai_typing;
    let error_message = ui_state.read().error_message.clone();
    let current_input = input_value.read().clone();

    rsx! {
        div {
            class: "chat-container",
            style: "
                flex: 1;
                display: flex;
                flex-direction: column;
                height: calc(100vh - 64px);
                background: {JINNIE_THEME.bg_primary};
            ",
            
            // Error message banner
            if let Some(error) = error_message {
                div {
                    style: "
                        background: {JINNIE_THEME.error};
                        color: white;
                        padding: 0.75rem 1.5rem;
                        font-size: 0.875rem;
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        border-bottom: 1px solid {JINNIE_THEME.border};
                    ",
                    
                    span { "{error}" }
                    
                    button {
                        style: "
                            background: none;
                            border: none;
                            color: white;
                            cursor: pointer;
                            font-size: 1.25rem;
                            padding: 0;
                            margin-left: 1rem;
                        ",
                        onclick: move |_| {
                            ui_state.write().set_error(None);
                        },
                        "×"
                    }
                }
            }
            
            // Messages area
            div {
                style: "
                    flex: 1;
                    overflow-y: auto;
                    padding: 1.5rem;
                    display: flex;
                    flex-direction: column;
                ",
                
                if messages.is_empty() {
                    WelcomeScreen {}
                } else {
                    rsx! {
                        MessageList {}
                        
                        if is_typing {
                            TypingIndicator {}
                        }
                    }
                }
            }
            
            // Input area
            div {
                style: "
                    padding: 1.5rem;
                    border-top: 1px solid {JINNIE_THEME.border};
                    background: {JINNIE_THEME.surface};
                ",
                
                div {
                    style: "
                        display: flex;
                        gap: 1rem;
                        align-items: flex-end;
                        max-width: 1000px;
                        margin: 0 auto;
                    ",
                    
                    textarea {
                        style: "
                            {input_styles()};
                            flex: 1;
                            min-height: 44px;
                            max-height: 120px;
                            resize: none;
                            font-family: inherit;
                        ",
                        placeholder: "Type your message... (Shift + Enter for new line)",
                        value: "{current_input}",
                        oninput: move |event| {
                            input_value.set(event.value());
                        },
                        onkeydown: handle_keypress,
                    }
                    
                    button {
                        style: "
                            {button_styles(\"primary\")};
                            height: 44px;
                            padding: 0 1.5rem;
                            display: flex;
                            align-items: center;
                            gap: 0.5rem;
                            opacity: {if current_input.trim().is_empty() || is_typing { \"0.5\" } else { \"1\" }};
                            cursor: {if current_input.trim().is_empty() || is_typing { \"not-allowed\" } else { \"pointer\" }};
                        ",
                        disabled: current_input.trim().is_empty() || is_typing,
                        onclick: send_message,
                        
                        if is_typing {
                            span { "Sending..." }
                        } else {
                            rsx! {
                                span { "Send" }
                                span { "↗" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn WelcomeScreen() -> Element {
    rsx! {
        div {
            style: "
                flex: 1;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                text-align: center;
                gap: 2rem;
            ",
            
            // Welcome section
            div {
                style: "
                    max-width: 600px;
                    display: flex;
                    flex-direction: column;
                    gap: 1rem;
                ",
                
                div {
                    style: "
                        font-size: 4rem;
                        margin-bottom: 1rem;
                    ",
                    "🦀"
                }
                
                h2 {
                    style: "
                        font-size: 2rem;
                        font-weight: 600;
                        color: {JINNIE_THEME.text_primary};
                        margin: 0;
                        background: linear-gradient(135deg, {JINNIE_THEME.primary}, {JINNIE_THEME.accent});
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                        background-clip: text;
                    ",
                    "Welcome to Jinnie.ai"
                }
                
                p {
                    style: "
                        font-size: 1.125rem;
                        color: {JINNIE_THEME.text_secondary};
                        margin: 0;
                        line-height: 1.6;
                    ",
                    "Your 100% Rust-powered AI assistant. Start a conversation below!"
                }
            }
            
            // Feature highlights
            div {
                style: "
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 1.5rem;
                    max-width: 800px;
                    width: 100%;
                ",
                
                FeatureCard {
                    icon: "⚡",
                    title: "Lightning Fast",
                    description: "Native Rust performance"
                }
                
                FeatureCard {
                    icon: "🔒",
                    title: "Private & Secure",
                    description: "Local processing"
                }
                
                FeatureCard {
                    icon: "🤖",
                    title: "Multiple Agents",
                    description: "Specialized AI assistants"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FeatureCardProps {
    icon: String,
    title: String,
    description: String,
}

fn FeatureCard(props: FeatureCardProps) -> Element {
    rsx! {
        div {
            style: "
                background: {JINNIE_THEME.surface};
                border: 1px solid {JINNIE_THEME.border};
                border-radius: 0.75rem;
                padding: 1.5rem;
                text-align: center;
                transition: all 0.2s;
                
                &:hover {{
                    border-color: {JINNIE_THEME.primary};
                    transform: translateY(-2px);
                }}
            ",
            
            div {
                style: "font-size: 2rem; margin-bottom: 0.5rem;",
                "{props.icon}"
            }
            
            h3 {
                style: "
                    font-size: 1rem;
                    font-weight: 600;
                    color: {JINNIE_THEME.text_primary};
                    margin: 0 0 0.5rem 0;
                ",
                "{props.title}"
            }
            
            p {
                style: "
                    font-size: 0.875rem;
                    color: {JINNIE_THEME.text_muted};
                    margin: 0;
                ",
                "{props.description}"
            }
        }
    }
}