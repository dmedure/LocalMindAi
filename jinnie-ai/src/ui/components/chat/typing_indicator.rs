use dioxus::prelude::*;
use crate::ui::theme::JINNIE_THEME;

pub fn TypingIndicator(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "typing-indicator fade-in",
            style: "
                display: flex;
                gap: 1rem;
                align-items: flex-start;
                max-width: 1000px;
                margin: 0 auto;
            ",
            
            // Avatar
            div {
                style: "
                    width: 40px;
                    height: 40px;
                    border-radius: 50%;
                    background: {JINNIE_THEME.surface};
                    color: {JINNIE_THEME.text_primary};
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-weight: 600;
                    font-size: 1.125rem;
                    flex-shrink: 0;
                    border: 1px solid {JINNIE_THEME.border};
                ",
                "🦀"
            }
            
            // Typing animation
            div {
                style: "
                    background: {JINNIE_THEME.surface};
                    border: 1px solid {JINNIE_THEME.border};
                    border-radius: 1rem;
                    border-bottom-left-radius: 0.25rem;
                    padding: 1rem 1.25rem;
                    display: flex;
                    align-items: center;
                    gap: 0.25rem;
                ",
                
                for i in 0..3 {
                    div {
                        key: "{i}",
                        style: "
                            width: 8px;
                            height: 8px;
                            background: {JINNIE_THEME.text_muted};
                            border-radius: 50%;
                            animation: typing 1.4s infinite ease-in-out;
                            animation-delay: {i * 0.2}s;
                        ",
                    }
                }
            }
        }
        
        style {
            r#"
            @keyframes typing {
                0%, 60%, 100% { opacity: 0.3; transform: scale(0.8); }
                30% { opacity: 1; transform: scale(1); }
            }
            "#
        }
    })
}