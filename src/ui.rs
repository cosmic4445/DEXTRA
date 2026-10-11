use iced::widget::image as picture;
use iced::widget::svg::Svg;
use iced::widget::text::Wrapping;
use iced::widget::{
    button, column, container, horizontal_space, mouse_area, progress_bar, row, text,
    vertical_space, Row, Space,
};
use iced::{Alignment, Background, Border, Color, ContentFit, Element, Length, Padding, Theme};

use crate::{colors, icons, media, photos, Message, Tab};

fn block(width: f32, height: f32, color: Color) -> Element<'static, Message> {
    container(Space::new(width, height))
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            ..container::Style::default()
        })
        .into()
}

pub fn logo(scale: f32, color: Color) -> Element<'static, Message> {
    let gap = 2.0 * scale;
    row![
        column![
            block(10.0 * scale, 14.0 * scale, color),
            block(9.0 * scale, 6.0 * scale, color)
        ]
        .spacing(gap),
        column![
            block(10.0 * scale, 6.0 * scale, color),
            block(9.0 * scale, 14.0 * scale, color)
        ]
        .spacing(gap),
    ]
    .spacing(gap)
    .into()
}

pub fn tile<'a>(
    title: &'static str,
    meta: impl Into<String>,
    height: Length,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    let label = row![
        text(title).size(13).color(colors::muted()),
        horizontal_space(),
        text(meta.into()).size(13).color(colors::muted()),
    ];

    container(column![label, body].spacing(12).height(Length::Fill))
        .padding(20)
        .width(Length::Fill)
        .height(height)
        .style(tile_style)
        .into()
}

pub fn launcher<'a>(
    title: &'static str,
    note: &'static str,
    label: &'static str,
    target: &'static str,
    height: Length,
) -> Element<'a, Message> {
    let body = column![
        text(note).size(15).color(colors::muted()),
        vertical_space(),
        button(text(label).size(14))
            .padding([12, 20])
            .style(primary)
            .on_press(Message::Launch(target)),
    ]
    .height(Length::Fill);

    tile(title, "Launcher", height, body.into())
}

pub fn entry<'a>(name: &'a str, is_dir: bool, message: Message) -> Element<'a, Message> {
    let icon = if is_dir { icons::folder() } else { icons::file() };

    button(
        row![
            icon.width(20).height(20),
            text(name).size(14).wrapping(Wrapping::None)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(10)
    .style(row_style)
    .on_press(message)
    .into()
}

pub fn photo(item: &photos::Photo) -> Element<'_, Message> {
    button(
        picture(item.handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Cover),
    )
    .padding(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(flat)
    .on_press(Message::Open(item.path.clone()))
    .into()
}

pub fn play_button(playing: bool) -> Element<'static, Message> {
    let glyph = if playing { icons::pause() } else { icons::play() };

    button(container(glyph.width(22).height(22)).center(Length::Fill))
        .width(52)
        .height(52)
        .style(circle)
        .on_press(Message::MediaControl(media::Control::Toggle))
        .into()
}

pub fn icon_button(icon: Svg<'static>, message: Message) -> Element<'static, Message> {
    button(container(icon.width(22).height(22)).center(Length::Fill))
        .width(44)
        .height(44)
        .style(ghost)
        .on_press(message)
        .into()
}

pub fn title_bar(active: Tab) -> Element<'static, Message> {
    let tabs = Tab::ALL.iter().fold(Row::new().spacing(4), |tabs, &tab| {
        tabs.push(
            button(text(tab.label()).size(13))
                .padding([8, 16])
                .style(tab_style(tab == active))
                .on_press(Message::Select(tab)),
        )
    });

    let controls = row![
        control("–", Message::Minimize, false),
        control("□", Message::ToggleMaximize, false),
        control("×", Message::Close, true),
    ];

    let bar = row![
        row![logo(1.0, colors::text()), text("Dextra").size(17)]
            .spacing(10)
            .align_y(Alignment::Center),
        horizontal_space(),
        tabs,
        horizontal_space(),
        controls,
    ]
    .align_y(Alignment::Center)
    .height(64)
    .padding(Padding {
        top: 0.0,
        right: 12.0,
        bottom: 0.0,
        left: 24.0,
    });

    mouse_area(bar).on_press(Message::Drag).into()
}

fn control(symbol: &'static str, message: Message, close: bool) -> Element<'static, Message> {
    button(container(text(symbol).size(16)).center(Length::Fill))
        .width(44)
        .height(36)
        .style(move |_, status| {
            let background = match (hovered(status), close) {
                (true, true) => Some(colors::close()),
                (true, false) => Some(colors::surface_hover()),
                _ => None,
            };
            look(background, colors::text(), 8.0)
        })
        .on_press(message)
        .into()
}

fn hovered(status: button::Status) -> bool {
    matches!(status, button::Status::Hovered | button::Status::Pressed)
}

fn look(background: Option<Color>, text_color: Color, radius: f32) -> button::Style {
    button::Style {
        background: background.map(Background::Color),
        text_color,
        border: Border {
            radius: radius.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

pub fn tile_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::surface())),
        border: Border {
            color: colors::border(),
            width: 1.0,
            radius: 20.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn art_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::light())),
        border: Border {
            radius: 14.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub fn progress(_: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(colors::track()),
        bar: Background::Color(colors::text()),
        border: Border {
            radius: 2.0.into(),
            ..Border::default()
        },
    }
}

fn tab_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, _| {
        if active {
            look(Some(colors::text()), colors::bg(), 999.0)
        } else {
            look(None, colors::muted(), 999.0)
        }
    }
}

fn primary(_: &Theme, status: button::Status) -> button::Style {
    let shade = if hovered(status) { 0.9 } else { 1.0 };
    look(Some(Color::from_rgb(shade, shade, shade)), colors::bg(), 999.0)
}

fn circle(_: &Theme, status: button::Status) -> button::Style {
    let shade = if hovered(status) { 0.9 } else { 1.0 };
    look(Some(Color::from_rgb(shade, shade, shade)), colors::bg(), 26.0)
}

fn ghost(_: &Theme, status: button::Status) -> button::Style {
    let background = hovered(status).then(colors::surface_hover);
    look(background, colors::text(), 22.0)
}

fn row_style(_: &Theme, status: button::Status) -> button::Style {
    let background = hovered(status).then(colors::surface_hover);
    look(background, colors::text(), 12.0)
}

fn flat(_: &Theme, _: button::Status) -> button::Style {
    look(None, colors::text(), 12.0)
}
