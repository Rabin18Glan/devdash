use ratatui::style::Color;

pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub muted: Color,
    pub highlight_bg: Color,
    pub border: Color,
    pub border_active: Color,
    pub text: Color,
    pub text_dim: Color,
}

pub const THEME: Theme = Theme {
    primary: Color::Rgb(125, 207, 255),       // Soft cyan
    secondary: Color::Rgb(187, 154, 247),     // Lavender purple
    accent: Color::Rgb(115, 218, 202),        // Mint teal
    success: Color::Rgb(158, 206, 106),       // Emerald green
    warning: Color::Rgb(255, 158, 100),       // Warm amber / orange
    danger: Color::Rgb(247, 118, 142),        // Coral neon red
    muted: Color::Rgb(86, 95, 137),           // Slate gray
    highlight_bg: Color::Rgb(36, 40, 59),     // Deep selection bg
    border: Color::Rgb(65, 72, 104),          // Subtle border
    border_active: Color::Rgb(125, 207, 255), // Active cyan border
    text: Color::Rgb(192, 202, 245),          // Bright text
    text_dim: Color::Rgb(120, 130, 160),      // Subdued text
};
