use dioxus::prelude::*;
use crate::ui::{
    theme::{JINNIE_THEME, card_styles},
    state::ui_state::{UIState, Agent},
};

#[derive(Props, Clone, PartialEq)]
pub struct AgentCardProps {
    pub agent: Agent,
    pub on_select: Option<EventHandler<String>>,
    pub on_edit: Option<EventHandler<String>>,
    pub on_delete: Option<EventHandler<String>>,
}

pub fn AgentCard(props: AgentCardProps) -> Element {
    let mut ui_state = use_context::<Signal<UIState>>();
    let agent = &props.agent;
    
    let handle_select = move |_| {
        ui_state.write().switch_agent(agent.id.clone());
        if let Some(on_select) = &props.on_select {
            on_select.call(agent.id.clone());
        }
    };

    let handle_edit = move |_| {
        if let Some(on_edit) = &props.on_edit {
            on_edit.call(agent.id.clone());
        }
    };

    let handle_delete = move |_| {
        if let Some(on_delete) = &props.on_delete {
            on_delete.call(agent.id.clone());
        }
    };

    rsx! {
        div {
            class: "agent-card",
            style: "
                {card_styles()};
                cursor: pointer;
                border-color: {if agent.is_active { JINNIE_THEME.primary } else { JINNIE_THEME.border }};
                background: {if agent.is_active { JINNIE_THEME.primary }11 else { JINNIE_THEME.surface }};
                position: relative;
                transition: all 0.2s;
                
                &:hover {{
                    border-color: {JINNIE_THEME.primary};
                    transform: translateY(-2px);
                    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
                }}
            ",
            
            onclick: handle_select,
            
            // Status indicator
            if agent.is_active {
                div {
                    style: "
                        position: absolute;
                        top: 0.75rem;
                        right: 0.75rem;
                        width: 8px;
                        height: 8px;
                        background: {JINNIE_THEME.success};
                        border-radius: 50%;
                        box-shadow: 0 0 0 2px {JINNIE_THEME.surface};
                    ",
                }
            }
            
            // Header
            div {
                style: "display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1rem;",
                
                // Avatar
                div {
                    style: "
                        width: 48px;
                        height: 48px;
                        background: {if agent.is_active { JINNIE_THEME.primary } else { JINNIE_THEME.bg_secondary }};
                        border-radius: 50%;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 1.5rem;
                        flex-shrink: 0;
                    ",
                    "🤖"
                }
                
                // Agent info
                div {
                    style: "flex: 1; min-width: 0;",
                    
                    h4 {
                        style: "
                            font-size: 1rem;
                            font-weight: 600;
                            color: {JINNIE_THEME.text_primary};
                            margin: 0 0 0.25rem 0;
                            white-space: nowrap;
                            overflow: hidden;
                            text-overflow: ellipsis;
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
            
            // Description
            p {
                style: "
                    font-size: 0.875rem;
                    color: {JINNIE_THEME.text_secondary};
                    line-height: 1.4;
                    margin: 0 0 1rem 0;
                    display: -webkit-box;
                    -webkit-line-clamp: 3;
                    -webkit-box-orient: vertical;
                    overflow: hidden;
                ",
                "{agent.description}"
            }
            
            // Actions
            div {
                style: "
                    display: flex;
                    gap: 0.5rem;
                    justify-content: flex-end;
                ",
                
                button {
                    style: "
                        padding: 0.375rem 0.75rem;
                        background: transparent;
                        border: 1px solid {JINNIE_THEME.border};
                        border-radius: 0.375rem;
                        color: {JINNIE_THEME.text_secondary};
                        cursor: pointer;
                        font-size: 0.75rem;
                        transition: all 0.2s;
                        
                        &:hover {{
                            background: {JINNIE_THEME.bg_secondary};
                            border-color: {JINNIE_THEME.border_hover};
                        }}
                    ",
                    onclick: move |e| {
                        e.stop_propagation();
                        handle_edit(());
                    },
                    "Edit"
                }
                
                button {
                    style: "
                        padding: 0.375rem 0.75rem;
                        background: transparent;
                        border: 1px solid {JINNIE_THEME.error}44;
                        border-radius: 0.375rem;
                        color: {JINNIE_THEME.error};
                        cursor: pointer;
                        font-size: 0.75rem;
                        transition: all 0.2s;
                        
                        &:hover {{
                            background: {JINNIE_THEME.error}11;
                            border-color: {JINNIE_THEME.error};
                        }}
                    ",
                    onclick: move |e| {
                        e.stop_propagation();
                        handle_delete(());
                    },
                    "Delete"
                }
            }
        }
    }
}