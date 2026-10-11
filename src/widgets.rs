use iced::Size;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Widget {
    Clock,
    Weather,
    Media,
    Photos,
    Files,
    Discord,
    Snapchat,
}

impl Widget {
    pub const ALL: [Widget; 7] = [
        Widget::Clock,
        Widget::Weather,
        Widget::Media,
        Widget::Photos,
        Widget::Files,
        Widget::Discord,
        Widget::Snapchat,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Widget::Clock => "Clock",
            Widget::Weather => "Weather",
            Widget::Media => "Now playing",
            Widget::Photos => "Photos",
            Widget::Files => "Files",
            Widget::Discord => "Discord",
            Widget::Snapchat => "Snapchat",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Widget::Clock => "Time and date.",
            Widget::Weather => "Current conditions with today's high and low.",
            Widget::Media => "Whatever is playing, with controls and album art.",
            Widget::Photos => "Your newest pictures.",
            Widget::Files => "Browse folders and open files.",
            Widget::Discord => "Shortcut to the Discord app.",
            Widget::Snapchat => "Shortcut to Snapchat Web.",
        }
    }

    pub fn size(self) -> Size {
        match self {
            Widget::Clock => Size::new(300.0, 150.0),
            Widget::Weather => Size::new(300.0, 170.0),
            Widget::Media => Size::new(360.0, 410.0),
            Widget::Photos => Size::new(340.0, 430.0),
            Widget::Files => Size::new(320.0, 420.0),
            Widget::Discord | Widget::Snapchat => Size::new(270.0, 170.0),
        }
    }
}
