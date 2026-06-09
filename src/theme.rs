use ratatui::style::Color;

/// Available color themes for the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Default,
    Dark,
    Light,
    Solarized,
    Monokai,
}

impl Theme {
    /// Parse a theme from its name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default" => Some(Theme::Default),
            "dark" => Some(Theme::Dark),
            "light" => Some(Theme::Light),
            "solarized" => Some(Theme::Solarized),
            "monokai" => Some(Theme::Monokai),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Default => "default",
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Solarized => "solarized",
            Theme::Monokai => "monokai",
        }
    }

    /// Get the primary accent color (borders, highlights)
    pub fn primary(&self) -> Color {
        match self {
            Theme::Default => Color::Cyan,
            Theme::Dark => Color::Blue,
            Theme::Light => Color::Blue,
            Theme::Solarized => Color::Rgb(42, 161, 152), // cyan
            Theme::Monokai => Color::Rgb(102, 217, 239),  // cyan
        }
    }

    /// Get the secondary color (selected items when not focused)
    pub fn secondary(&self) -> Color {
        match self {
            Theme::Default => Color::DarkGray,
            Theme::Dark => Color::Gray,
            Theme::Light => Color::Gray,
            Theme::Solarized => Color::Rgb(88, 110, 117), // base01
            Theme::Monokai => Color::Rgb(117, 113, 94),   // comment
        }
    }

    /// Get the background color for selected items
    pub fn selection_bg(&self) -> Color {
        match self {
            Theme::Default => Color::Cyan,
            Theme::Dark => Color::Blue,
            Theme::Light => Color::LightBlue,
            Theme::Solarized => Color::Rgb(42, 161, 152), // cyan
            Theme::Monokai => Color::Rgb(73, 72, 62),     // selection
        }
    }

    /// Get the foreground color for selected items
    pub fn selection_fg(&self) -> Color {
        match self {
            Theme::Default => Color::Black,
            Theme::Dark => Color::White,
            Theme::Light => Color::Black,
            Theme::Solarized => Color::Rgb(0, 43, 54),    // base03
            Theme::Monokai => Color::Rgb(248, 248, 242),  // foreground
        }
    }

    /// Get the text color
    pub fn text(&self) -> Color {
        match self {
            Theme::Default => Color::White,
            Theme::Dark => Color::Gray,
            Theme::Light => Color::Black,
            Theme::Solarized => Color::Rgb(131, 148, 150), // base0
            Theme::Monokai => Color::Rgb(248, 248, 242),   // foreground
        }
    }

    /// Get the muted/text color for dim elements
    pub fn muted(&self) -> Color {
        match self {
            Theme::Default => Color::Gray,
            Theme::Dark => Color::DarkGray,
            Theme::Light => Color::DarkGray,
            Theme::Solarized => Color::Rgb(88, 110, 117), // base01
            Theme::Monokai => Color::Rgb(117, 113, 94),   // comment
        }
    }

    /// Get the status bar background color
    pub fn status_bg(&self) -> Color {
        match self {
            Theme::Default => Color::White,
            Theme::Dark => Color::DarkGray,
            Theme::Light => Color::Gray,
            Theme::Solarized => Color::Rgb(238, 232, 213), // base2
            Theme::Monokai => Color::Rgb(73, 72, 62),      // selection
        }
    }

    /// Get the status bar foreground color
    pub fn status_fg(&self) -> Color {
        match self {
            Theme::Default => Color::Black,
            Theme::Dark => Color::White,
            Theme::Light => Color::Black,
            Theme::Solarized => Color::Rgb(7, 54, 66),     // base02
            Theme::Monokai => Color::Rgb(248, 248, 242),   // foreground
        }
    }

    /// Get the color for info-level logs
    pub fn info_color(&self) -> Color {
        match self {
            Theme::Default => Color::Green,
            Theme::Dark => Color::Green,
            Theme::Light => Color::Green,
            Theme::Solarized => Color::Rgb(133, 153, 0),   // green
            Theme::Monokai => Color::Rgb(166, 226, 46),    // green
        }
    }

    /// Get the color for warning-level logs
    pub fn warn_color(&self) -> Color {
        match self {
            Theme::Default => Color::Yellow,
            Theme::Dark => Color::Yellow,
            Theme::Light => Color::Rgb(184, 134, 11),      // dark goldenrod
            Theme::Solarized => Color::Rgb(181, 137, 0),   // yellow
            Theme::Monokai => Color::Rgb(253, 151, 31),    // orange
        }
    }

    /// Get the color for error-level logs
    pub fn error_color(&self) -> Color {
        match self {
            Theme::Default => Color::Red,
            Theme::Dark => Color::Red,
            Theme::Light => Color::Red,
            Theme::Solarized => Color::Rgb(220, 50, 47),   // red
            Theme::Monokai => Color::Rgb(249, 38, 114),    // pink
        }
    }

    /// Get the color for fatal-level logs
    pub fn fatal_color(&self) -> Color {
        match self {
            Theme::Default => Color::Magenta,
            Theme::Dark => Color::Magenta,
            Theme::Light => Color::Magenta,
            Theme::Solarized => Color::Rgb(211, 54, 130),  // magenta
            Theme::Monokai => Color::Rgb(174, 129, 255),   // purple
        }
    }

    /// Get the color for debug-level logs
    pub fn debug_color(&self) -> Color {
        match self {
            Theme::Default => Color::Blue,
            Theme::Dark => Color::Blue,
            Theme::Light => Color::Blue,
            Theme::Solarized => Color::Rgb(38, 139, 210),  // blue
            Theme::Monokai => Color::Rgb(102, 217, 239),   // cyan
        }
    }

    /// Get the color for trace-level logs
    pub fn trace_color(&self) -> Color {
        match self {
            Theme::Default => Color::DarkGray,
            Theme::Dark => Color::DarkGray,
            Theme::Light => Color::Gray,
            Theme::Solarized => Color::Rgb(7, 54, 66),     // base02
            Theme::Monokai => Color::Rgb(117, 113, 94),    // comment
        }
    }

    /// Get the color for unknown-level logs
    pub fn unknown_color(&self) -> Color {
        match self {
            Theme::Default => Color::White,
            Theme::Dark => Color::White,
            Theme::Light => Color::Black,
            Theme::Solarized => Color::Rgb(131, 148, 150), // base0
            Theme::Monokai => Color::Rgb(248, 248, 242),   // foreground
        }
    }

    /// Get a group by field color (used for field values)
    pub fn field_color(&self) -> Color {
        match self {
            Theme::Default => Color::Yellow,
            Theme::Dark => Color::Rgb(255, 215, 0),        // gold
            Theme::Light => Color::Rgb(184, 134, 11),      // dark goldenrod
            Theme::Solarized => Color::Rgb(203, 75, 22),   // orange
            Theme::Monokai => Color::Rgb(253, 151, 31),    // orange
        }
    }

    /// Get the time bucket color
    pub fn time_color(&self) -> Color {
        match self {
            Theme::Default => Color::Green,
            Theme::Dark => Color::Rgb(50, 205, 50),        // lime green
            Theme::Light => Color::Rgb(34, 139, 34),       // forest green
            Theme::Solarized => Color::Rgb(133, 153, 0),   // green
            Theme::Monokai => Color::Rgb(166, 226, 46),    // green
        }
    }

    /// Get the total count highlight color
    pub fn count_color(&self) -> Color {
        match self {
            Theme::Default => Color::Cyan,
            Theme::Dark => Color::Rgb(0, 191, 255),        // deep sky blue
            Theme::Light => Color::Blue,
            Theme::Solarized => Color::Rgb(42, 161, 152),  // cyan
            Theme::Monokai => Color::Rgb(102, 217, 239),   // cyan
        }
    }

    /// Cycle to the next theme
    pub fn next(&self) -> Self {
        match self {
            Theme::Default => Theme::Dark,
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Solarized,
            Theme::Solarized => Theme::Monokai,
            Theme::Monokai => Theme::Default,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_name() {
        assert_eq!(Theme::from_name("default"), Some(Theme::Default));
        assert_eq!(Theme::from_name("dark"), Some(Theme::Dark));
        assert_eq!(Theme::from_name("light"), Some(Theme::Light));
        assert_eq!(Theme::from_name("solarized"), Some(Theme::Solarized));
        assert_eq!(Theme::from_name("monokai"), Some(Theme::Monokai));
        assert_eq!(Theme::from_name("unknown"), None);
    }

    #[test]
    fn test_theme_next() {
        let theme = Theme::Default;
        assert_eq!(theme.next(), Theme::Dark);
        assert_eq!(theme.next().next(), Theme::Light);
        assert_eq!(theme.next().next().next(), Theme::Solarized);
        assert_eq!(theme.next().next().next().next(), Theme::Monokai);
        assert_eq!(theme.next().next().next().next().next(), Theme::Default);
    }

    #[test]
    fn test_theme_colors_not_default() {
        // Ensure themes have distinct colors
        let default = Theme::Default;
        let dark = Theme::Dark;
        assert_ne!(default.primary(), dark.primary());
    }
}
