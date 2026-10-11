use iced::widget::svg::{Handle, Svg};

fn icon(markup: &'static str) -> Svg<'static> {
    Svg::new(Handle::from_memory(markup.as_bytes()))
}

pub fn previous() -> Svg<'static> {
    icon(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><path d="M6 5h2v14H6zM20 5v14L9 12z"/></svg>"##)
}

pub fn next() -> Svg<'static> {
    icon(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><path d="M16 5h2v14h-2zM4 5v14l11-7z"/></svg>"##)
}

pub fn play() -> Svg<'static> {
    icon(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#2a2a2a"><path d="M8 5v14l11-7z"/></svg>"##)
}

pub fn pause() -> Svg<'static> {
    icon(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#2a2a2a"><path d="M6 5h4v14H6zM14 5h4v14h-4z"/></svg>"##)
}

pub fn folder() -> Svg<'static> {
    icon(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="1.8" stroke-linejoin="round"><path d="M3 6h6l2 2h10v11H3z"/></svg>"##)
}

pub fn file() -> Svg<'static> {
    icon(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="1.8" stroke-linejoin="round"><path d="M6 3h8l4 4v14H6z"/></svg>"##)
}
