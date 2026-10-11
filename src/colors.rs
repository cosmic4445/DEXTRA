use iced::Color;

fn hex(value: u32) -> Color {
    Color::from_rgb8((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

pub fn bg() -> Color {
    hex(0x2a2a2a)
}

pub fn surface() -> Color {
    hex(0x3c3c3c)
}

pub fn surface_hover() -> Color {
    hex(0x444444)
}

pub fn border() -> Color {
    hex(0x4a4a4a)
}

pub fn text() -> Color {
    hex(0xffffff)
}

pub fn muted() -> Color {
    hex(0xb3b3b3)
}

pub fn track() -> Color {
    hex(0x5a5a5a)
}

pub fn light() -> Color {
    hex(0xe8e8e8)
}

pub fn close() -> Color {
    hex(0xc42b1c)
}