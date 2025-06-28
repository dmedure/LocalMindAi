use dioxus::prelude::*;
use crate::ui::{
    theme::{JINNIE_THEME, input_styles, button_styles},
    state::ui_state::{UIState, Agent},
};

#[derive(Props, Clone, PartialEq)]
pub struct AgentCreatorProps {
    pub on_close: EventHandler<()>,
}

pub fn AgentCreator(props: AgentCreatorProps) -> Element {
    let mut ui_state = use_context::<Signal<UIState>>();
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut system_prompt = use_signal(|| String::new());
    let mut selected_model = use_signal(|| "TinyLlama".to_string());
    let mut is_creating = use_signal(|| false);

    let handle_create = move |_| {
        let name_val = name.read().trim().to_string();
        let desc_val = description.read().trim().to_string();
        let prompt_val = system_prompt.read().trim().to_string();
        let model_val = selected_model.read().clone();

        if name_val.is_empty() {
            return;
        }

        is_creating.set(true);

        // Create new agent
        let new_agent = Agent {
            id: uuid::Uuid::new_v4().to_string(),
            name: name_val,
            model: model_val,
            description: if desc_val.is_empty() { "A helpful AI assistant".to_string() } else { desc_val },
            system_prompt: if prompt_val.is_empty() { "You are a helpful AI assistant.".to_string() } else { prompt_val },
            is_active: false,
        };

        // Add to state
        ui_state.write().agents.push(new_agent);
        
        // Close modal
        props.on_close.call(());
    };

    let handle_close = move |_| {
        props.on_close.call(());
    };

    rsx! {
        // Modal backdrop
        div {
            style: "
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.6);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 1000;
                padding: 1rem;
            ",
            onclick: move |e| {
                if e.target() == e.current_target() {
                    handle_close(());
                }
            },
            
            // Modal content
            div {
                style: "
                    background: {JINNIE_THEME.surface};
                    border-radius: 1rem;
                    padding: 2rem;
                    width: 100%;
                    max-width: 600px;
                    max-height: 90vh;
                    overflow-y: auto;
                    border: 1px solid {JINNIE_THEME.border};
                ",
                
                // Header
                div {
                    style: "
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        margin-bottom: 2rem;
                        padding-bottom: 1rem;
                        border-bottom: 1px solid {JINNIE_THEME.border};
                    ",
                    
                    h2 {
                        style: "
                            font-size: 1.5rem;
                            font-weight: 600;
                            color: {JINNIE_THEME.text_primary};
                            margin: 0;
                        ",
                        "🤖 Create New Agent"
                    }
                    
                    button {
                        style: "
                            background: none;
                            border: none;
                            color: {JINNIE_THEME.text_muted};
                            cursor: pointer;
                            font-size: 1.5rem;
                            padding: 0.25rem;
                            border-radius: 0.25rem;
                            transition: all 0.2s;
                            
                            &:hover {{
                                background: {JINNIE_THEME.bg_secondary};
                                color: {JINNIE_THEME.text_primary};
                            }}
                        ",
                        onclick: handle_close,
                        "×"
                    }
                }
                
                // Form
                div {
                    style: "display: flex; flex-direction: column; gap: 1.5rem;",
                    
                    // Name input
                    div {
                        label {
                            style: "
                                display: block;
                                font-size: 0.875rem;
                                font-weight: 500;
                                color: {JINNIE_THEME.text_primary};
                                margin-bottom: 0.5rem;
                            ",
                            "Agent Name *"
                        }
                        
                        input {
                            style: "{input_styles()}; width: 100%;",
                            r#type: "text",
                            placeholder: "e.g., Code Expert, Writing Assistant, Data Analyst",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    
                    // Model selection
                    div {
                        label {
                            style: "
                                display: block;
                                font-size: 0.875rem;
                                font-weight: 500;
                                color: {JINNIE_THEME.text_primary};
                                margin-bottom: 0.5rem;
                            ",
                            "AI Model"
                        }
                        
                        select {
                            style: "{input_styles()}; width: 100%;",
                            value: "{selected_model}",
                            onchange: move |e| selected_model.set(e.value()),
                            
                            option { value: "TinyLlama", "TinyLlama (Fast, Lightweight)" }
                            option { value: "Mistral 7B", "Mistral 7B (Balanced)" }
                            option { value: "CodeLlama", "CodeLlama (Code Specialist)" }
                        }
                    }
                    
                    // Description
                    div {
                        label {
                            style: "
                                display: block;
                                font-size: 0.875rem;
                                font-weight: 500;
                                color: {JINNIE_THEME.text_primary};
                                margin-bottom: 0.5rem;
                            ",
                            "Description"
                        }
                        
                        textarea {
                            style: "{input_styles()}; width: 100%; min-height: 80px; resize: vertical;",
                            placeholder: "Describe what this agent specializes in and how it can help users...",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                        }
                    }
                    
                    // System prompt
                    div {
                        label {
                            style: "
                                display: block;
                                font-size: 0.875rem;
                                font-weight: 500;
                                color: {JINNIE_THEME.text_primary};
                                margin-bottom: 0.5rem;
                            ",
                            "System Prompt"
                        }
                        
                        textarea {
                            style: "{input_styles()}; width: 100%; min-height: 120px; resize: vertical;",
                            placeholder: "You are a helpful AI assistant that specializes in...",
                            value: "{system_prompt}",
                            oninput: move |e| system_prompt.set(e.value()),
                        }
                        
                        div {
                            style: "
                                font-size: 0.75rem;
                                color: {JINNIE_THEME.text_muted};
                                margin-top: 0.5rem;
                            ",
                            "💡 This defines how the agent behaves and responds to users"
                        }
                    }
                }
                
                // Actions
                div {
                    style: "
                        display: flex;
                        gap: 1rem;
                        justify-content: flex-end;
                        margin-top: 2rem;
                        padding-top: 1rem;
                        border-top: 1px solid {JINNIE_THEME.border};
                    ",
                    
                    button {
                        style: "{button_styles(\"secondary\")}",
                        onclick: handle_close,
                        "Cancel"
                    }
                    
                    button {
                        style: "
                            {button_styles(\"primary\")};
                            opacity: {if name.read().trim().is_empty() || *is_creating.read() { \"0.5\" } else { \"1\" }};
                            cursor: {if name.read().trim().is_empty() || *is_creating.read() { \"not-allowed\" } else { \"pointer\" }};
                        ",
                        disabled: name.read().trim().is_empty() || *is_creating.read(),
                        onclick: handle_create,
                        
                        if *is_creating.read() {
                            "Creating..."
                        } else {
                            "Create Agent"
                        }
                    }
                }
            }
        }
    }
}