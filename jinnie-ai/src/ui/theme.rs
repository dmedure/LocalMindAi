use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct Theme {
    // Primary burnt orange colors
    pub primary: &'static str,
    pub primary_hover: &'static str,
    pub primary_light: &'static str,
    pub primary_dark: &'static str,
    
    // Secondary colors
    pub secondary: &'static str,
    pub accent: &'static str,
    
    // Background colors
    pub bg_primary: &'static str,
    pub bg_secondary: &'static str,
    pub bg_tertiary: &'static str,
    
    // Surface colors
    pub surface: &'static str,
    pub surface_hover: &'static str,
    
    // Text colors
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    
    // Status colors
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,
    
    // Border colors
    pub border: &'static str,
    pub border_hover: &'static str,
    pub border_focus: &'static str,
}

pub const JINNIE_THEME: Theme = Theme {
    // Burnt orange primary palette
    primary: "#CC5500",          // Burnt orange
    primary_hover: "#B84C00",    // Darker burnt orange
    primary_light: "#E6692D",    // Lighter burnt orange
    primary_dark: "#994000",     // Deep burnt orange
    
    // Complementary colors
    secondary: "#4A5568",        // Cool gray
    accent: "#D97706",           // Amber accent
    
    // Rust-inspired backgrounds
    bg_primary: "#1A1414",       // Very dark brown-black
    bg_secondary: "#221A1A",     // Dark rust brown
    bg_tertiary: "#2D2222",      // Slightly lighter rust
    
    // Surface colors with rust tint
    surface: "#332626",          // Rust-tinted surface
    surface_hover: "#3D2F2F",    // Hover state
    
    // Text colors
    text_primary: "#F5F5F5",     // Almost white
    text_secondary: "#D4D4D4",   // Light gray
    text_muted: "#A8A8A8",       // Muted gray
    
    // Status colors
    success: "#10B981",
    warning: "#F59E0B",
    error: "#EF4444",
    info: "#3B82F6",
    
    // Border colors with rust tint
    border: "#443333",
    border_hover: "#554444",
    border_focus: "#CC5500",
};

pub fn global_styles() -> String {
    format!(
        r#"
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            background: {};
            color: {};
            line-height: 1.5;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }}
        
        ::-webkit-scrollbar {{
            width: 8px;
            height: 8px;
        }}
        
        ::-webkit-scrollbar-track {{
            background: {};
        }}
        
        ::-webkit-scrollbar-thumb {{
            background: {};
            border-radius: 4px;
        }}
        
        ::-webkit-scrollbar-thumb:hover {{
            background: {};
        }}
        
        .fade-in {{
            animation: fadeIn 0.3s ease-in-out;
        }}
        
        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(10px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
        
        .slide-in-left {{
            animation: slideInLeft 0.3s ease-in-out;
        }}
        
        @keyframes slideInLeft {{
            from {{ opacity: 0; transform: translateX(-20px); }}
            to {{ opacity: 1; transform: translateX(0); }}
        }}
        
        .typing-animation {{
            animation: typing 1.5s infinite;
        }}
        
        @keyframes typing {{
            0%, 60%, 100% {{ opacity: 1; }}
            30% {{ opacity: 0.7; }}
        }}
        "#,
        JINNIE_THEME.bg_primary,
        JINNIE_THEME.text_primary,
        JINNIE_THEME.bg_secondary,
        JINNIE_THEME.border,
        JINNIE_THEME.primary
    )
}

// Utility functions for common styles
pub fn button_styles(variant: &str) -> String {
    match variant {
        "primary" => format!(
            "background: {}; color: white; border: none; padding: 0.75rem 1.5rem; border-radius: 0.5rem; cursor: pointer; font-weight: 500; transition: background 0.2s; font-size: 0.875rem;",
            JINNIE_THEME.primary
        ),
        "secondary" => format!(
            "background: {}; color: {}; border: 1px solid {}; padding: 0.75rem 1.5rem; border-radius: 0.5rem; cursor: pointer; font-weight: 500; transition: all 0.2s; font-size: 0.875rem;",
            JINNIE_THEME.surface,
            JINNIE_THEME.text_primary,
            JINNIE_THEME.border
        ),
        "ghost" => format!(
            "background: transparent; color: {}; border: none; padding: 0.75rem 1.5rem; border-radius: 0.5rem; cursor: pointer; font-weight: 500; transition: background 0.2s; font-size: 0.875rem;",
            JINNIE_THEME.text_secondary
        ),
        _ => String::new(),
    }
}

pub fn card_styles() -> String {
    format!(
        "background: {}; border: 1px solid {}; border-radius: 0.75rem; padding: 1.5rem; transition: all 0.2s;",
        JINNIE_THEME.surface,
        JINNIE_THEME.border
    )
}

pub fn input_styles() -> String {
    format!(
        "background: {}; border: 1px solid {}; border-radius: 0.5rem; padding: 0.75rem 1rem; color: {}; font-size: 0.875rem; transition: all 0.2s; outline: none;",
        JINNIE_THEME.bg_secondary,
        JINNIE_THEME.border,
        JINNIE_THEME.text_primary
    )
}