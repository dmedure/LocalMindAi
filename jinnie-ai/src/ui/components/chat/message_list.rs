use dioxus::prelude::*;
use crate::ui::{
    state::ui_state::UIState,
};
use super::message_item::MessageItem;

pub fn MessageList(cx: Scope) -> Element {
    let ui_state = use_shared_state::<UIState>(cx)?;
    
    cx.render(rsx! {
        div {
            class: "message-list",
            style: "
                display: flex;
                flex-direction: column;
                gap: 1.5rem;
                width: 100%;
                max-width: 1000px;
                margin: 0 auto;
            ",
            
            for message in ui_state.read().messages.iter() {
                MessageItem {
                    key: "{message.id}",
                    message: message.clone(),
                }
            }
        }
    })
}