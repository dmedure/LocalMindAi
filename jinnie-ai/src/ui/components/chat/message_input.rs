use dioxus::prelude::*;
use crate::{
    ui::{
        theme::{JINNIE_THEME, input_styles, button_styles},
        state::ui_state::{UIState, MessageRole},
    },
    use_app_state,
};

pub fn MessageInput() -> Element {
    let mut ui_state = use_context::<Signal<UIState>>();
    let app_state = use_app_state();
    let mut input_value = use_signal(|| String::new());
    let mut is_sending = use_signal(|| false);
    
    // Handle sending messages
    let send_message = move |_| {
        let message_content = input_value.read().trim().to_string();
        if message_content.is_empty() || *is_sending.read() {
            return;
        }

        let current_agent_id = ui_state.read().current_agent_id.clone();
        if current_agent_id.is_empty() {
            ui_state.write().set_error(Some("No agent selected".to_string()));
            return;
        }

        // Add user message immediately
        ui_state.write().add_message(message_content.clone(), MessageRole::User);
        
        // Clear input and set sending state
        input_value.set(String::new());
        is_sending.set(true);
        ui_state.write().set_typing(true);
        ui_state.write().set_error(None);

        // Send message to backend
        let app_state = app_state.clone();
        let mut ui_state = ui_state.clone();
        let mut is_sending = is_sending.clone();
        
        spawn(async move {
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
            
            // Stop typing indicator and sending state
            ui_state.write().set_typing(false);
            is_sending.set(false);
        });
    };

    // Handle Enter key press
    let handle_keypress = move |event: KeyboardEvent| {
        if event.key() == Key::Enter {
            if event.ctrl_key() || event.meta_key() {
                // Ctrl+Enter or Cmd+Enter to send
                event.prevent_default();
                send_message(());
            } else if !event.shift_key() {
                // Just Enter to send (unless Shift+Enter for new line)
                event.prevent_default();
                send_message(());
            }
        }
    };

    // Get current values
    let current_input = input_value.read().clone();
    let is_typing = ui_state.read().is_ai_typing;
    let sending = *is_sending.read();
    let current_agent = ui_state.read().get_current_agent().cloned();

    rsx! {
        div {
            class: "message-input-container",
            style: "
                padding: 1.5rem;
                background: {JINNIE_THEME.surface};
                border-top: 1px solid {JINNIE_THEME.border};
                flex-shrink: 0;
            ",
            
            // Agent indicator (if agent selected)
            if let Some(agent) = current_agent {
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 0.5rem;
                        margin-bottom: 0.75rem;
                        padding: 0.5rem 0.75rem;
                        background: {JINNIE_THEME.bg_secondary};
                        border-radius: 0.5rem;
                        font-size: 0.875rem;
                        color: {JINNIE_THEME.text_secondary};
                    ",
                    
                    div {
                        style: "
                            width: 6px;
                            height: 6px;
                            background: {JINNIE_THEME.success};
                            border-radius: 50%;
                        ",
                    }
                    
                    span { "Chatting with {agent.name}" }
                    
                    span {
                        style: "
                            background: {JINNIE_THEME.primary}22;
                            color: {JINNIE_THEME.primary};
                            padding: 0.125rem 0.375rem;
                            border-radius: 0.25rem;
                            font-size: 0.75rem;
                            font-weight: 500;
                        ",
                        "{agent.model}"
                    }
                }
            }
            
            // Input container
            div {
                style: "
                    display: flex;
                    gap: 0.75rem;
                    align-items: flex-end;
                    max-width: 1000px;
                    margin: 0 auto;
                ",
                
                // Text input area
                div {
                    style: "
                        flex: 1;
                        position: relative;
                    ",
                    
                    textarea {
                        style: "
                            {input_styles()};
                            width: 100%;
                            min-height: 48px;
                            max-height: 120px;
                            resize: none;
                            font-family: inherit;
                            font-size: 1rem;
                            line-height: 1.5;
                            padding-right: 3rem;
                            transition: all 0.2s;
                            
                            &:focus {{
                                border-color: {JINNIE_THEME.border_focus};
                                box-shadow: 0 0 0 3px {JINNIE_THEME.primary}22;
                            }}
                        ",
                        placeholder: "Type your message... (Enter to send, Shift+Enter for new line)",
                        value: "{current_input}",
                        disabled: sending || is_typing,
                        oninput: move |event| {
                            input_value.set(event.value());
                        },
                        onkeydown: handle_keypress,
                        
                        // Auto-resize textarea
                        oninput: move |event| {
                            input_value.set(event.value());
                            // Auto-resize logic would go here if needed
                        },
                    }
                    
                    // Character counter (optional)
                    if current_input.len() > 100 {
                        div {
                            style: "
                                position: absolute;
                                bottom: 0.5rem;
                                right: 0.5rem;
                                font-size: 0.75rem;
                                color: {JINNIE_THEME.text_muted};
                                background: {JINNIE_THEME.surface};
                                padding: 0.125rem 0.375rem;
                                border-radius: 0.25rem;
                            ",
                            "{current_input.len()}"
                        }
                    }
                }
                
                // Send button
                button {
                    style: "
                        {button_styles(\"primary\")};
                        width: 48px;
                        height: 48px;
                        border-radius: 50%;
                        padding: 0;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        transition: all 0.2s;
                        flex-shrink: 0;
                        opacity: {if current_input.trim().is_empty() || sending || is_typing { \"0.5\" } else { \"1\" }};
                        cursor: {if current_input.trim().is_empty() || sending || is_typing { \"not-allowed\" } else { \"pointer\" }};
                        
                        &:hover:not(:disabled) {{
                            background: {JINNIE_THEME.primary_hover};
                            transform: scale(1.05);
                        }}
                        
                        &:active:not(:disabled) {{
                            transform: scale(0.95);
                        }}
                    ",
                    disabled: current_input.trim().is_empty() || sending || is_typing,
                    onclick: send_message,
                    title: if sending { "Sending..." } else if is_typing { "AI is responding..." } else { "Send message (Enter)" },
                    
                    if sending {
                        // Spinning loader
                        div {
                            style: "
                                width: 20px;
                                height: 20px;
                                border: 2px solid rgba(255,255,255,0.3);
                                border-top: 2px solid white;
                                border-radius: 50%;
                                animation: spin 1s linear infinite;
                            ",
                        }
                        
                        style {
                            r#"
                            @keyframes spin {
                                0% { transform: rotate(0deg); }
                                100% { transform: rotate(360deg); }
                            }
                            "#
                        }
                    } else {
                        // Send icon (arrow)
                        svg {
                            width: "20",
                            height: "20",
                            viewBox: "0 0 24 24",
                            fill: "currentColor",
                            path {
                                d: "M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"
                            }
                        }
                    }
                }
            }
            
            // Helpful hints
            div {
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-top: 0.75rem;
                    font-size: 0.75rem;
                    color: {JINNIE_THEME.text_muted};
                ",
                
                div {
                    style: "display: flex; gap: 1rem;",
                    
                    span { "💡 Press Enter to send" }
                    span { "⏎ Shift+Enter for new line" }
                }
                
                if is_typing {
                    div {
                        style: "
                            display: flex;
                            align-items: center;
                            gap: 0.5rem;
                            color: {JINNIE_THEME.primary};
                            font-weight: 500;
                        ",
                        
                        div {
                            style: "
                                width: 6px;
                                height: 6px;
                                background: {JINNIE_THEME.primary};
                                border-radius: 50%;
                                animation: pulse 1s infinite;
                            ",
                        }
                        
                        span { "AI is thinking..." }
                        
                        style {
                            r#"
                            @keyframes pulse {
                                0%, 100% { opacity: 1; }
                                50% { opacity: 0.5; }
                            }
                            "#
                        }
                    }
                }
            }
        }
    }
}