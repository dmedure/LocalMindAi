use dioxus::prelude::*;
use crate::ui::{
    theme::JINNIE_THEME,
    state::ui_state::{Message, MessageRole},
};

#[derive(Props)]
pub struct MessageItemProps {
    message: Message,
}

pub fn MessageItem(cx: Scope<MessageItemProps>) -> Element {
    let is_user = matches!(cx.props.message.role, MessageRole::User);
    
    cx.render(rsx! {
        div {
            class: "message-item fade-in",
            style: "
                display: flex;
                gap: 1rem;
                align-items: flex-start;
                {if is_user { \"flex-direction: row-reverse;\" } else { \"\" }}
            ",
            
            // Avatar
            div {
                style: "
                    width: 40px;
                    height: 40px;
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-weight: 600;
                    font-size: 1.125rem;
                    flex-shrink: 0;
                    {if is_user {
                        format!(\"background: {}; color: white;\", JINNIE_THEME.primary)
                    } else {
                        format!(\"background: {}; color: {};\", JINNIE_THEME.surface, JINNIE_THEME.text_primary)
                    }}
                ",
                
                if is_user { "You" } else { "🦀" }
            }
            
            // Message content
            div {
                style: "
                    flex: 1;
                    max-width: calc(100% - 80px);
                ",
                
                div {
                    style: "
                        background: {if is_user { JINNIE_THEME.primary } else { JINNIE_THEME.surface }};
                        color: {if is_user { \"white\" } else { JINNIE_THEME.text_primary }};
                        padding: 1rem 1.25rem;
                        border-radius: 1rem;
                        {if is_user {
                            \"border-bottom-right-radius: 0.25rem;\"
                        } else {
                            \"border-bottom-left-radius: 0.25rem; border: 1px solid \".to_owned() + JINNIE_THEME.border + \";\"
                        }}
                        line-height: 1.5;
                        word-wrap: break-word;
                        white-space: pre-wrap;
                    ",
                    
                    "{cx.props.message.content}"
                }
                
                // Timestamp
                div {
                    style: "
                        font-size: 0.75rem;
                        color: {JINNIE_THEME.text_muted};
                        margin-top: 0.5rem;
                        {if is_user { \"text-align: right;\" } else { \"\" }}
                    ",
                    
                    "Just now"
                }
            }
        }
    })
}