use dioxus::prelude::*;
use crate::ui::{
    theme::JINNIE_THEME,
    state::ui_state::UIState,
};
use super::{AgentCard, AgentCreator};

pub fn AgentList() -> Element {
    let ui_state = use_context::<Signal<UIState>>();
    let mut show_creator = use_signal(|| false);
    
    let handle_create_agent = move |_| {
        show_creator.set(true);
    };

    let handle_close_creator = move |_| {
        show_creator.set(false);
    };

    let agents = ui_state.read().agents.clone();

    rsx! {
        div {
            class: "agent-list",
            style: "
                display: flex;
                flex-direction: column;
                gap: 1rem;
                height: 100%;
            ",
            
            // Header
            div {
                style: "
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    padding-bottom: 1rem;
                    border-bottom: 1px solid {JINNIE_THEME.border};
                ",
                
                h3 {
                    style: "
                        font-size: 1.125rem;
                        font-weight: 600;
                        color: {JINNIE_THEME.text_primary};
                        margin: 0;
                    ",
                    "AI Agents ({agents.len()})"
                }
                
                button {
                    style: "
                        padding: 0.5rem 1rem;
                        background: {JINNIE_THEME.primary};
                        border: none;
                        border-radius: 0.5rem;
                        color: white;
                        cursor: pointer;
                        font-size: 0.875rem;
                        font-weight: 500;
                        transition: all 0.2s;
                        
                        &:hover {{
                            background: {JINNIE_THEME.primary_hover};
                        }}
                    ",
                    onclick: handle_create_agent,
                    "+ New Agent"
                }
            }
            
            // Agent grid
            div {
                style: "
                    flex: 1;
                    overflow-y: auto;
                    display: grid;
                    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
                    gap: 1rem;
                    padding: 1rem 0;
                ",
                
                for agent in agents {
                    AgentCard {
                        key: "{agent.id}",
                        agent: agent.clone(),
                    }
                }
                
                // Create new agent card
                div {
                    style: "
                        border: 2px dashed {JINNIE_THEME.border};
                        border-radius: 0.75rem;
                        padding: 2rem;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        justify-content: center;
                        cursor: pointer;
                        transition: all 0.2s;
                        text-align: center;
                        min-height: 200px;
                        
                        &:hover {{
                            border-color: {JINNIE_THEME.primary};
                            background: {JINNIE_THEME.primary}08;
                        }}
                    ",
                    onclick: handle_create_agent,
                    
                    div {
                        style: "
                            font-size: 3rem;
                            color: {JINNIE_THEME.text_muted};
                            margin-bottom: 1rem;
                        ",
                        "+"
                    }
                    
                    h4 {
                        style: "
                            font-size: 1rem;
                            font-weight: 600;
                            color: {JINNIE_THEME.text_primary};
                            margin: 0 0 0.5rem 0;
                        ",
                        "Create New Agent"
                    }
                    
                    p {
                        style: "
                            font-size: 0.875rem;
                            color: {JINNIE_THEME.text_muted};
                            margin: 0;
                            line-height: 1.4;
                        ",
                        "Design a specialized AI assistant for your specific needs"
                    }
                }
            }
        }
        
        // Agent creator modal
        if *show_creator.read() {
            AgentCreator {
                on_close: handle_close_creator,
            }
        }
    }
}