// src/ui/components/sidebar.rs
use dioxus::prelude::*;
use crate::ui::{
    theme::{JINNIE_THEME, button_styles, card_styles},
    state::ui_state::{UIState, Agent},
};

pub fn HistorySidebar(cx: Scope) -> Element {
    let ui_state = use_shared_state::<UIState>(cx)?;
    let is_collapsed = ui_state.read().sidebar_collapsed;
    
    cx.render(rsx! {
        div {
            class: "history-sidebar",
            style: "
                width: {if is_collapsed { \"60px\" } else { \"280px\" }};
                background: {JINNIE_THEME.surface};
                border-right: 1px solid {JINNIE_THEME.border};
                display: flex;
                flex-direction: column;
                transition: width 0.3s ease;
                flex-shrink: 0;
            ",
            
            // Header
            div {
                style: "
                    padding: 1rem;
                    border-bottom: 1px solid {JINNIE_THEME.border};
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    height: 64px;
                    box-sizing: border-box;
                ",
                
                if !is_collapsed {
                    rsx! {
                        h3 {
                            style: "
                                font-size: 1rem;
                                font-weight: 600;
                                color: {JINNIE_THEME.text_primary};
                                margin: 0;
                            ",
                            "Chat History"
                        }
                    }
                }
                
                button {
                    style: "{button_styles(\"ghost\")} width: 32px; height: 32px; padding: 0; display: flex; align-items: center; justify-content: center;",
                    onclick: move |_| {
                        ui_state.write().toggle_sidebar();
                    },
                    title: if is_collapsed { "Expand sidebar" } else { "Collapse sidebar" },
                    
                    if is_collapsed { "→" } else { "←" }
                }
            }
            
            // Chat list
            div {
                style: "
                    flex: 1;
                    overflow-y: auto;
                    padding: {if is_collapsed { \"0.5rem\" } else { \"1rem\" }};
                ",
                
                if is_collapsed {
                    rsx! {
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            
                            for chat in ui_state.read().chats.iter().take(5) {
                                div {
                                    key: "{chat.id}",
                                    style: "
                                        width: 32px;
                                        height: 32px;
                                        background: {if Some(&chat.id) == ui_state.read().current_chat_id.as_ref() { JINNIE_THEME.primary } else { JINNIE_THEME.bg_secondary }};
                                        border-radius: 8px;
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                        cursor: pointer;
                                        transition: all 0.2s;
                                        color: {if Some(&chat.id) == ui_state.read().current_chat_id.as_ref() { \"white\" } else { JINNIE_THEME.text_muted }};
                                        font-size: 0.75rem;
                                        font-weight: 600;
                                    ",
                                    title: "{chat.title}",
                                    
                                    "{chat.title.chars().next().unwrap_or('C')}"
                                }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.75rem;",
                            
                            if ui_state.read().chats.is_empty() {
                                rsx! {
                                    div {
                                        style: "
                                            text-align: center;
                                            padding: 2rem 1rem;
                                            color: {JINNIE_THEME.text_muted};
                                            font-size: 0.875rem;
                                        ",
                                        
                                        div {
                                            style: "font-size: 2rem; margin-bottom: 1rem;",
                                            "💬"
                                        }
                                        
                                        "No chat history yet"
                                        br {}
                                        "Start a conversation!"
                                    }
                                }
                            } else {
                                rsx! {
                                    for chat in ui_state.read().chats.iter() {
                                        div {
                                            key: "{chat.id}",
                                            class: "chat-item",
                                            style: "
                                                padding: 0.75rem;
                                                border-radius: 0.5rem;
                                                cursor: pointer;
                                                transition: all 0.2s;
                                                background: {if Some(&chat.id) == ui_state.read().current_chat_id.as_ref() { JINNIE_THEME.primary } else { \"transparent\" }};
                                                color: {if Some(&chat.id) == ui_state.read().current_chat_id.as_ref() { \"white\" } else { JINNIE_THEME.text_primary }};
                                                border: 1px solid {if Some(&chat.id) == ui_state.read().current_chat_id.as_ref() { JINNIE_THEME.primary } else { \"transparent\" }};
                                                
                                                &:hover {{
                                                    background: {if Some(&chat.id) == ui_state.read().current_chat_id.as_ref() { JINNIE_THEME.primary_hover } else { JINNIE_THEME.bg_secondary }};
                                                }}
                                            ",
                                            
                                            onclick: move |_| {
                                                ui_state.write().current_chat_id = Some(chat.id.clone());
                                            },
                                            
                                            div {
                                                style: "
                                                    font-weight: 500;
                                                    font-size: 0.875rem;
                                                    margin-bottom: 0.25rem;
                                                    white-space: nowrap;
                                                    overflow: hidden;
                                                    text-overflow: ellipsis;
                                                ",
                                                "{chat.title}"
                                            }
                                            
                                            div {
                                                style: "
                                                    font-size: 0.75rem;
                                                    opacity: 0.8;
                                                ",
                                                "Today" // TODO: Format timestamp
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn AgentSidebar(cx: Scope) -> Element {
    let ui_state = use_shared_state::<UIState>(cx)?;
    let is_collapsed = ui_state.read().agent_sidebar_collapsed;
    
    cx.render(rsx! {
        div {
            class: "agent-sidebar",
            style: "
                width: {if is_collapsed { \"60px\" } else { \"320px\" }};
                background: {JINNIE_THEME.surface};
                border-left: 1px solid {JINNIE_THEME.border};
                display: flex;
                flex-direction: column;
                transition: width 0.3s ease;
                flex-shrink: 0;
            ",
            
            // Header
            div {
                style: "
                    padding: 1rem;
                    border-bottom: 1px solid {JINNIE_THEME.border};
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    height: 64px;
                    box-sizing: border-box;
                ",
                
                button {
                    style: "{button_styles(\"ghost\")} width: 32px; height: 32px; padding: 0; display: flex; align-items: center; justify-content: center;",
                    onclick: move |_| {
                        ui_state.write().toggle_agent_sidebar();
                    },
                    title: if is_collapsed { "Expand agents" } else { "Collapse agents" },
                    
                    if is_collapsed { "←" } else { "→" }
                }
                
                if !is_collapsed {
                    rsx! {
                        h3 {
                            style: "
                                font-size: 1rem;
                                font-weight: 600;
                                color: {JINNIE_THEME.text_primary};
                                margin: 0;
                            ",
                            "AI Agents"
                        }
                    }
                }
            }
            
            // Agents list
            div {
                style: "
                    flex: 1;
                    overflow-y: auto;
                    padding: {if is_collapsed { \"0.5rem\" } else { \"1rem\" }};
                ",
                
                if is_collapsed {
                    rsx! {
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            
                            for agent in ui_state.read().agents.iter() {
                                div {
                                    key: "{agent.id}",
                                    style: "
                                        width: 32px;
                                        height: 32px;
                                        background: {if agent.is_active { JINNIE_THEME.primary } else { JINNIE_THEME.bg_secondary }};
                                        border-radius: 8px;
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                        cursor: pointer;
                                        transition: all 0.2s;
                                        color: {if agent.is_active { \"white\" } else { JINNIE_THEME.text_muted }};
                                        font-size: 0.75rem;
                                        font-weight: 600;
                                    ",
                                    title: "{agent.name}",
                                    onclick: move |_| {
                                        ui_state.write().switch_agent(agent.id.clone());
                                    },
                                    
                                    "🤖"
                                }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div {
                            style: "display: flex; flex-direction: column; gap: 1rem;",
                            
                            for agent in ui_state.read().agents.iter() {
                                AgentCard {
                                    key: "{agent.id}",
                                    agent: agent.clone(),
                                }
                            }
                            
                            // Create new agent button
                            div {
                                style: "
                                    padding: 2rem;
                                    border: 2px dashed {JINNIE_THEME.border};
                                    border-radius: 0.75rem;
                                    display: flex;
                                    flex-direction: column;
                                    align-items: center;
                                    justify-content: center;
                                    cursor: pointer;
                                    transition: all 0.2s;
                                    text-align: center;
                                    
                                    &:hover {{
                                        border-color: {JINNIE_THEME.primary};
                                        background: {JINNIE_THEME.primary}11;
                                    }}
                                ",
                                
                                div {
                                    style: "font-size: 2rem; color: {JINNIE_THEME.text_muted}; margin-bottom: 0.5rem;",
                                    "+"
                                }
                                
                                div {
                                    style: "font-size: 0.875rem; color: {JINNIE_THEME.text_secondary}; font-weight: 500;",
                                    "Create New Agent"
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Props)]
pub struct AgentCardProps {
    agent: Agent,
}

pub fn AgentCard(cx: Scope<AgentCardProps>) -> Element {
    let ui_state = use_shared_state::<UIState>(cx)?;
    let agent = &cx.props.agent;
    
    cx.render(rsx! {
        div {
            class: "agent-card",
            style: "
                {card_styles()};
                cursor: pointer;
                border-color: {if agent.is_active { JINNIE_THEME.primary } else { JINNIE_THEME.border }};
                background: {if agent.is_active { JINNIE_THEME.primary }11 else { JINNIE_THEME.surface }};
                
                &:hover {{
                    border-color: {JINNIE_THEME.primary};
                    transform: translateY(-2px);
                }}
            ",
            
            onclick: move |_| {
                ui_state.write().switch_agent(agent.id.clone());
            },
            
            // Header
            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem;",
                
                div {
                    style: "display: flex; align-items: center; gap: 0.75rem;",
                    
                    div {
                        style: "
                            width: 40px;
                            height: 40px;
                            background: {if agent.is_active { JINNIE_THEME.primary } else { JINNIE_THEME.bg_secondary }};
                            border-radius: 50%;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            font-size: 1.25rem;
                        ",
                        "🤖"
                    }
                    
                    div {
                        h4 {
                            style: "
                                font-size: 1rem;
                                font-weight: 600;
                                color: {JINNIE_THEME.text_primary};
                                margin: 0 0 0.25rem 0;
                            ",
                            "{agent.name}"
                        }
                        
                        div {
                            style: "
                                font-size: 0.75rem;
                                color: {JINNIE_THEME.text_muted};
                                background: {JINNIE_THEME.primary}22;
                                padding: 0.125rem 0.5rem;
                                border-radius: 0.375rem;
                                display: inline-block;
                            ",
                            "{agent.model}"
                        }
                    }
                }
                
                if agent.is_active {
                    rsx! {
                        div {
                            style: "
                                width: 8px;
                                height: 8px;
                                background: {JINNIE_THEME.success};
                                border-radius: 50%;
                            ",
                        }
                    }
                }
            }
            
            // Description
            p {
                style: "
                    font-size: 0.875rem;
                    color: {JINNIE_THEME.text_secondary};
                    line-height: 1.4;
                    margin: 0;
                ",
                "{agent.description}"
            }
        }
    })
}