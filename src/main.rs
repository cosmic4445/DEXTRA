mod colors;
mod files;
mod icons;
mod media;
mod photos;
mod ui;
mod weather;

use std::path::PathBuf;
use std::time::Duration;

use iced::widget::image as picture;
use iced::widget::text::Wrapping;
use iced::widget::{
    column, container, horizontal_space, progress_bar, row, scrollable, text, vertical_space,
    Column, Row, Space,
};
use iced::{
    time, window, Alignment, ContentFit, Element, Length, Padding, Size, Subscription, Task, Theme,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Home,
    Media,
    Weather,
    Photos,
    Files,
    Discord,
    Snapchat,
}

impl Tab {
    pub const ALL: [Tab; 7] = [
        Tab::Home,
        Tab::Media,
        Tab::Weather,
        Tab::Photos,
        Tab::Files,
        Tab::Discord,
        Tab::Snapchat,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Media => "Media",
            Tab::Weather => "Weather",
            Tab::Photos => "Photos",
            Tab::Files => "Files",
            Tab::Discord => "Discord",
            Tab::Snapchat => "Snapchat",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(Tab),
    Drag,
    Minimize,
    ToggleMaximize,
    Close,
    WeatherLoaded(Result<weather::Report, String>),
    RefreshWeather,
    PhotosLoaded(Vec<photos::Photo>),
    Browse(PathBuf),
    Up,
    Open(PathBuf),
    Launch(&'static str),
    MediaTick,
    MediaUpdated(Option<media::Snapshot>),
    MediaControl(media::Control),
}

enum WeatherState {
    Loading,
    Ready(weather::Report),
    Failed(String),
}

struct Dextra {
    active: Tab,
    weather: WeatherState,
    photos: Vec<photos::Photo>,
    folder: PathBuf,
    entries: Vec<files::Entry>,
    media: Option<media::Snapshot>,
}

impl Dextra {
    fn new() -> (Self, Task<Message>) {
        let folder = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let entries = files::list(&folder);

        let app = Dextra {
            active: Tab::Home,
            weather: WeatherState::Loading,
            photos: Vec::new(),
            folder,
            entries,
            media: None,
        };

        let boot = Task::batch([
            Task::perform(weather::fetch(), Message::WeatherLoaded),
            Task::perform(photos::load(12), Message::PhotosLoaded),
            Task::perform(media::read(None), Message::MediaUpdated),
        ]);

        (app, boot)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Select(tab) => {
                self.active = tab;
                Task::none()
            }
            Message::Drag => window::get_oldest().and_then(window::drag),
            Message::Minimize => window::get_oldest().and_then(|id| window::minimize(id, true)),
            Message::ToggleMaximize => window::get_oldest().and_then(window::toggle_maximize),
            Message::Close => window::get_oldest().and_then(window::close),
            Message::WeatherLoaded(result) => {
                self.weather = match result {
                    Ok(report) => WeatherState::Ready(report),
                    Err(reason) => WeatherState::Failed(reason),
                };
                Task::none()
            }
            Message::RefreshWeather => Task::perform(weather::fetch(), Message::WeatherLoaded),
            Message::PhotosLoaded(photos) => {
                self.photos = photos;
                Task::none()
            }
            Message::Browse(path) => {
                self.entries = files::list(&path);
                self.folder = path;
                Task::none()
            }
            Message::Up => match self.folder.parent().map(PathBuf::from) {
                Some(parent) => self.update(Message::Browse(parent)),
                None => Task::none(),
            },
            Message::Open(path) => {
                let _ = opener::open(path);
                Task::none()
            }
            Message::Launch(target) => {
                let _ = opener::open(target);
                Task::none()
            }
            Message::MediaTick => {
                let known = self.media.as_ref().map(|track| track.title.clone());
                Task::perform(media::read(known), Message::MediaUpdated)
            }
            Message::MediaUpdated(next) => {
                let merged = next.map(|mut track| {
                    if track.art.is_none() {
                        if let Some(previous) = &self.media {
                            if previous.title == track.title {
                                track.art = previous.art.clone();
                            }
                        }
                    }
                    track
                });
                self.media = merged;
                Task::none()
            }
            Message::MediaControl(action) => {
                Task::perform(media::control(action), |_| Message::MediaTick)
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(Duration::from_secs(900)).map(|_| Message::RefreshWeather),
            time::every(Duration::from_secs(1)).map(|_| Message::MediaTick),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let page = match self.active {
            Tab::Home => self.board(),
            Tab::Media => self.media_tile(Length::Fill, true),
            Tab::Weather => self.weather_tile(Length::Fill, true),
            Tab::Photos => self.photos_tile(Length::Fill, true),
            Tab::Files => self.files_tile(Length::Fill, true),
            Tab::Discord => ui::launcher(
                "Discord",
                "Opens the Discord desktop app.",
                "Open Discord",
                "discord://",
                Length::Fill,
            ),
            Tab::Snapchat => ui::launcher(
                "Snapchat",
                "Opens Snapchat Web in your browser.",
                "Open Snapchat",
                "https://web.snapchat.com",
                Length::Fill,
            ),
        };

        let content = column![
            ui::title_bar(self.active),
            container(page)
                .padding(Padding {
                    top: 0.0,
                    right: 24.0,
                    bottom: 24.0,
                    left: 24.0,
                })
                .height(Length::Fill),
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(colors::bg())),
                ..container::Style::default()
            })
            .into()
    }

    fn board(&self) -> Element<'_, Message> {
        row![
            column![
                self.media_tile(Length::FillPortion(2), false),
                ui::launcher(
                    "Snapchat",
                    "Opens Snapchat Web.",
                    "Open Snapchat",
                    "https://web.snapchat.com",
                    Length::FillPortion(1),
                ),
            ]
            .spacing(16)
            .width(Length::FillPortion(11))
            .height(Length::Fill),
            column![
                self.weather_tile(Length::FillPortion(1), false),
                self.photos_tile(Length::FillPortion(2), false),
            ]
            .spacing(16)
            .width(Length::FillPortion(10))
            .height(Length::Fill),
            column![
                self.files_tile(Length::FillPortion(2), false),
                ui::launcher(
                    "Discord",
                    "Opens the Discord app.",
                    "Open Discord",
                    "discord://",
                    Length::FillPortion(1),
                ),
            ]
            .spacing(16)
            .width(Length::FillPortion(10))
            .height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    }

    fn weather_tile(&self, height: Length, expanded: bool) -> Element<'_, Message> {
        let size = if expanded { 96.0 } else { 60.0 };

        match &self.weather {
            WeatherState::Ready(report) => {
                let body = column![
                    text(format!("{:.0}°", report.temperature)).size(size),
                    vertical_space(),
                    row![
                        text(report.condition).size(14),
                        horizontal_space(),
                        text(format!("H {:.0}°  L {:.0}°", report.high, report.low))
                            .size(14)
                            .color(colors::muted()),
                    ],
                ]
                .height(Length::Fill);
                ui::tile("Weather", report.city.clone(), height, body.into())
            }
            WeatherState::Loading => ui::tile(
                "Weather",
                "",
                height,
                text("Loading").color(colors::muted()).into(),
            ),
            WeatherState::Failed(reason) => ui::tile(
                "Weather",
                "",
                height,
                text(reason.as_str()).size(13).color(colors::muted()).into(),
            ),
        }
    }

    fn files_tile(&self, height: Length, _expanded: bool) -> Element<'_, Message> {
        let mut list = Column::new().spacing(2).width(Length::Fill);

        if self.folder.parent().is_some() {
            list = list.push(ui::entry("..", true, Message::Up));
        }

        for item in &self.entries {
            let message = if item.is_dir {
                Message::Browse(item.path.clone())
            } else {
                Message::Open(item.path.clone())
            };
            list = list.push(ui::entry(&item.name, item.is_dir, message));
        }

        let meta = self
            .folder
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();

        ui::tile(
            "Files",
            meta,
            height,
            scrollable(list).width(Length::Fill).height(Length::Fill).into(),
        )
    }

    fn photos_tile(&self, height: Length, expanded: bool) -> Element<'_, Message> {
        let (columns, limit) = if expanded { (4, 12) } else { (2, 6) };
        let shown: Vec<&photos::Photo> = self.photos.iter().take(limit).collect();

        let body: Element<'_, Message> = if shown.is_empty() {
            text("No photos found in Pictures")
                .color(colors::muted())
                .into()
        } else {
            shown
                .chunks(columns)
                .fold(
                    Column::new().spacing(8).height(Length::Fill),
                    |grid, chunk| {
                        let mut cells = Row::new().spacing(8).height(Length::Fill);
                        for slot in 0..columns {
                            cells = match chunk.get(slot) {
                                Some(item) => cells.push(ui::photo(item)),
                                None => cells.push(Space::new(Length::Fill, Length::Fill)),
                            };
                        }
                        grid.push(cells)
                    },
                )
                .into()
        };

        ui::tile("Photos", "Recent", height, body)
    }

    fn media_tile(&self, height: Length, expanded: bool) -> Element<'_, Message> {
        let Some(track) = &self.media else {
            return ui::tile(
                "Now playing",
                "",
                height,
                text("Nothing playing right now")
                    .color(colors::muted())
                    .into(),
            );
        };

        let art_height = if expanded { 320.0 } else { 170.0 };

        let art: Element<'_, Message> = match &track.art {
            Some(handle) => picture(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Cover)
                .into(),
            None => container(ui::logo(3.5, colors::bg()))
                .center(Length::Fill)
                .into(),
        };

        let controls = row![
            ui::icon_button(icons::previous(), Message::MediaControl(media::Control::Previous)),
            ui::play_button(track.playing),
            ui::icon_button(icons::next(), Message::MediaControl(media::Control::Next)),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let body = column![
            container(art)
                .width(Length::Fill)
                .height(art_height)
                .style(ui::art_style),
            text(track.title.clone()).size(22).wrapping(Wrapping::None),
            text(track.artist.clone())
                .color(colors::muted())
                .wrapping(Wrapping::None),
            vertical_space(),
            progress_bar(0.0..=track.duration.max(1.0), track.position)
                .height(4)
                .style(ui::progress),
            row![
                text(clock(track.position)).size(12).color(colors::muted()),
                horizontal_space(),
                text(clock(track.duration)).size(12).color(colors::muted()),
            ],
            container(controls).center_x(Length::Fill),
        ]
        .spacing(8)
        .height(Length::Fill);

        ui::tile("Now playing", "", height, body.into())
    }
}

fn clock(seconds: f32) -> String {
    let total = seconds as u32;
    format!("{}:{:02}", total / 60, total % 60)
}

fn main() -> iced::Result {
    iced::application("Dextra", Dextra::update, Dextra::view)
        .subscription(Dextra::subscription)
        .theme(|_| Theme::Dark)
        .window(window::Settings {
            size: Size::new(1120.0, 720.0),
            min_size: Some(Size::new(900.0, 600.0)),
            position: window::Position::Centered,
            decorations: false,
            ..window::Settings::default()
        })
        .run_with(Dextra::new)
}
