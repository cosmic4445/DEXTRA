#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod background;
mod colors;
mod files;
mod icons;
mod layout;
mod media;
mod photos;
mod startup;
mod ui;
mod weather;
mod widgets;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Local};
use iced::daemon::Appearance;
use iced::futures::SinkExt;
use iced::widget::image as picture;
use iced::widget::text::Wrapping;
use iced::widget::{
    column, container, horizontal_space, mouse_area, progress_bar, row, scrollable, text,
    vertical_space, Column, Row, Space,
};
use iced::{
    time, window, Alignment, Background, ContentFit, Element, Length, Padding, Point, Size,
    Subscription, Task, Theme,
};
use tray_icon::TrayIcon;

use layout::{Layout, Placed};
use widgets::Widget;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Home,
    Time,
    Utility,
    Media,
    Social,
    Settings,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Home,
        Tab::Time,
        Tab::Utility,
        Tab::Media,
        Tab::Social,
        Tab::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Time => "Time",
            Tab::Utility => "Utility",
            Tab::Media => "Media",
            Tab::Social => "Social",
            Tab::Settings => "Settings",
        }
    }

    fn shows(self, widget: Widget) -> bool {
        match self {
            Tab::Home => widget != Widget::Clock,
            Tab::Time => matches!(widget, Widget::Clock | Widget::Weather),
            Tab::Utility => matches!(widget, Widget::Files | Widget::Photos),
            Tab::Media => widget == Widget::Media,
            Tab::Social => matches!(widget, Widget::Discord | Widget::Snapchat),
            Tab::Settings => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(Tab),
    Drag(window::Id),
    Minimize(window::Id),
    ToggleMaximize(window::Id),
    CloseHub(window::Id),
    Quit,
    Toggle(Widget),
    Remove(Widget),
    SetBehind(bool),
    SetStartup(bool),
    CityChanged(String),
    CitySubmitted,
    Signal(background::Signal),
    WindowClosed(window::Id),
    WindowEvent(window::Id, window::Event),
    SaveTick,
    ClockTick,
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

#[derive(Clone, Copy)]
enum Role {
    Hub,
    Widget(Widget),
}

enum WeatherState {
    Loading,
    Ready(weather::Report),
    Failed(String),
}

struct Dextra {
    windows: HashMap<window::Id, Role>,
    positions: HashMap<Widget, Point>,
    tab: Tab,
    hub_visible: bool,
    behind: bool,
    startup: bool,
    city: String,
    dirty: bool,
    now: DateTime<Local>,
    weather: WeatherState,
    photos_loaded: bool,
    photos: Vec<photos::Photo>,
    folder: PathBuf,
    entries: Vec<files::Entry>,
    media: Option<media::Snapshot>,
    tray: Option<TrayIcon>,
}

fn hub_settings(visible: bool) -> window::Settings {
    #[allow(unused_mut)]
    let mut settings = window::Settings {
        size: Size::new(1120.0, 720.0),
        min_size: Some(Size::new(900.0, 600.0)),
        position: window::Position::Centered,
        decorations: false,
        visible,
        exit_on_close_request: false,
        ..window::Settings::default()
    };

    #[cfg(windows)]
    {
        settings.platform_specific.undecorated_shadow = true;
    }

    settings
}

fn widget_settings(widget: Widget, position: Point, behind: bool) -> window::Settings {
    #[allow(unused_mut)]
    let mut settings = window::Settings {
        size: widget.size(),
        position: window::Position::Specific(position),
        decorations: false,
        resizable: false,
        exit_on_close_request: false,
        level: level(behind),
        ..window::Settings::default()
    };

    #[cfg(windows)]
    {
        settings.platform_specific.skip_taskbar = true;
        settings.platform_specific.undecorated_shadow = true;
    }

    settings
}

fn level(behind: bool) -> window::Level {
    if behind {
        window::Level::AlwaysOnBottom
    } else {
        window::Level::Normal
    }
}

fn signals() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(8, |mut output| async move {
        loop {
            for signal in background::poll() {
                let _ = output.send(Message::Signal(signal)).await;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
}

fn media_stream() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(4, |mut output| async move {
        let mut known: Option<String> = None;
        let mut last: Option<(String, bool, u32)> = None;

        loop {
            let snapshot = media::read(known.clone()).await;
            known = snapshot.as_ref().map(|track| track.title.clone());

            let marker = snapshot
                .as_ref()
                .map(|track| (track.title.clone(), track.playing, track.position as u32));
            let has_art = snapshot.as_ref().is_some_and(|track| track.art.is_some());

            if marker != last || has_art {
                last = marker;
                let _ = output.send(Message::MediaUpdated(snapshot)).await;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

impl Dextra {
    fn new(hidden: bool) -> (Self, Task<Message>) {
        let saved = layout::load();
        let folder = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let entries = files::list(&folder);

        let mut app = Dextra {
            windows: HashMap::new(),
            positions: HashMap::new(),
            tab: Tab::Home,
            hub_visible: !hidden,
            behind: saved.behind,
            startup: startup::enabled(),
            city: saved.city,
            dirty: false,
            now: Local::now(),
            weather: WeatherState::Loading,
            photos_loaded: false,
            photos: Vec::new(),
            folder,
            entries,
            media: None,
            tray: background::tray(),
        };

        let mut tasks = vec![Task::perform(
            weather::fetch(app.city.clone()),
            Message::WeatherLoaded,
        )];

        let (hub, open_hub) = window::open(hub_settings(!hidden));
        app.windows.insert(hub, Role::Hub);
        tasks.push(open_hub.discard());
        if !hidden {
            tasks.push(app.load_photos());
        }

        for placed in saved.widgets {
            tasks.push(app.open_widget(placed.widget, Point::new(placed.x, placed.y)));
        }

        (app, Task::batch(tasks))
    }

    fn title(&self, _: window::Id) -> String {
        "Dextra".to_string()
    }

    fn theme(&self, _: window::Id) -> Theme {
        Theme::Dark
    }

    fn style(&self, _: &Theme) -> Appearance {
        Appearance {
            background_color: colors::surface(),
            text_color: colors::text(),
        }
    }

    fn hub_id(&self) -> Option<window::Id> {
        self.windows
            .iter()
            .find(|(_, role)| matches!(role, Role::Hub))
            .map(|(id, _)| *id)
    }

    fn id_of(&self, widget: Widget) -> Option<window::Id> {
        self.windows
            .iter()
            .find(|(_, role)| matches!(role, Role::Widget(open) if *open == widget))
            .map(|(id, _)| *id)
    }

    fn is_open(&self, widget: Widget) -> bool {
        self.id_of(widget).is_some()
    }

    fn needs(&self, widget: Widget) -> bool {
        self.is_open(widget) || (self.hub_visible && self.tab.shows(widget))
    }

    fn load_photos(&mut self) -> Task<Message> {
        if self.photos_loaded {
            return Task::none();
        }
        self.photos_loaded = true;
        Task::perform(photos::load(12), Message::PhotosLoaded)
    }

    fn open_hub(&mut self) -> Task<Message> {
        let Some(id) = self.hub_id() else {
            return Task::none();
        };
        self.hub_visible = true;

        let photos = if self.tab.shows(Widget::Photos) {
            self.load_photos()
        } else {
            Task::none()
        };

        Task::batch([
            window::change_mode(id, window::Mode::Windowed),
            window::minimize(id, false),
            window::gain_focus(id),
            photos,
        ])
    }

    fn hide_hub(&mut self, id: window::Id) -> Task<Message> {
        if self.tray.is_some() {
            self.hub_visible = false;
            window::change_mode(id, window::Mode::Hidden)
        } else {
            window::minimize(id, true)
        }
    }

    fn open_widget(&mut self, widget: Widget, position: Point) -> Task<Message> {
        let (id, open) = window::open(widget_settings(widget, position, self.behind));
        self.windows.insert(id, Role::Widget(widget));
        self.positions.insert(widget, position);

        let load = match widget {
            Widget::Photos => self.load_photos(),
            Widget::Media => Task::perform(media::read(None), Message::MediaUpdated),
            Widget::Files => {
                self.entries = files::list(&self.folder);
                Task::none()
            }
            _ => Task::none(),
        };

        Task::batch([open.discard(), load])
    }

    fn remove(&mut self, widget: Widget) -> Task<Message> {
        let Some(id) = self.id_of(widget) else {
            return Task::none();
        };
        self.windows.remove(&id);
        self.positions.remove(&widget);
        self.save();
        window::close(id)
    }

    fn save(&self) {
        let widgets = self
            .windows
            .values()
            .filter_map(|role| match role {
                Role::Widget(widget) => Some(*widget),
                Role::Hub => None,
            })
            .filter_map(|widget| {
                self.positions.get(&widget).map(|point| Placed {
                    widget,
                    x: point.x,
                    y: point.y,
                })
            })
            .collect();

        layout::save(&Layout {
            widgets,
            behind: self.behind,
            city: self.city.clone(),
        });
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Select(tab) => {
                self.tab = tab;
                if tab.shows(Widget::Photos) {
                    self.load_photos()
                } else {
                    Task::none()
                }
            }
            Message::Drag(id) => window::drag(id),
            Message::Minimize(id) => window::minimize(id, true),
            Message::ToggleMaximize(id) => window::toggle_maximize(id),
            Message::CloseHub(id) => self.hide_hub(id),
            Message::Quit => iced::exit(),
            Message::Toggle(widget) => {
                if self.is_open(widget) {
                    return self.remove(widget);
                }
                let step = 36.0 * self.positions.len() as f32;
                let task = self.open_widget(widget, Point::new(60.0 + step, 60.0 + step));
                self.save();
                task
            }
            Message::Remove(widget) => self.remove(widget),
            Message::SetBehind(behind) => {
                self.behind = behind;
                self.save();
                let moves: Vec<Task<Message>> = self
                    .windows
                    .iter()
                    .filter(|(_, role)| matches!(role, Role::Widget(_)))
                    .map(|(id, _)| window::change_level(*id, level(behind)))
                    .collect();
                Task::batch(moves)
            }
            Message::SetStartup(on) => {
                startup::set(on);
                self.startup = startup::enabled();
                Task::none()
            }
            Message::CityChanged(city) => {
                self.city = city;
                Task::none()
            }
            Message::CitySubmitted => {
                self.save();
                self.weather = WeatherState::Loading;
                Task::perform(weather::fetch(self.city.clone()), Message::WeatherLoaded)
            }
            Message::Signal(signal) => match signal {
                background::Signal::Open => self.open_hub(),
                background::Signal::Quit => iced::exit(),
            },
            Message::WindowClosed(id) => {
                if let Some(Role::Widget(widget)) = self.windows.remove(&id) {
                    self.positions.remove(&widget);
                    self.save();
                }
                Task::none()
            }
            Message::WindowEvent(id, event) => {
                if matches!(event, window::Event::CloseRequested) {
                    return match self.windows.get(&id).copied() {
                        Some(Role::Hub) => self.hide_hub(id),
                        Some(Role::Widget(widget)) => self.remove(widget),
                        None => Task::none(),
                    };
                }

                let position = match event {
                    window::Event::Moved(point) => Some(point),
                    window::Event::Opened { position, .. } => position,
                    _ => None,
                };
                if let (Some(point), Some(Role::Widget(widget))) = (position, self.windows.get(&id)) {
                    self.positions.insert(*widget, point);
                    self.dirty = true;
                }
                Task::none()
            }
            Message::SaveTick => {
                self.save();
                self.dirty = false;
                Task::none()
            }
            Message::ClockTick => {
                self.now = Local::now();
                Task::none()
            }
            Message::WeatherLoaded(result) => {
                self.weather = match result {
                    Ok(report) => WeatherState::Ready(report),
                    Err(reason) => WeatherState::Failed(reason),
                };
                Task::none()
            }
            Message::RefreshWeather => {
                Task::perform(weather::fetch(self.city.clone()), Message::WeatherLoaded)
            }
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
        let mut active = vec![
            window::close_events().map(Message::WindowClosed),
            iced::event::listen_with(|event, _, id| match event {
                iced::Event::Window(
                    event @ (window::Event::Moved(_)
                    | window::Event::Opened { .. }
                    | window::Event::CloseRequested),
                ) => Some(Message::WindowEvent(id, event)),
                _ => None,
            }),
            Subscription::run(signals),
            time::every(Duration::from_secs(900)).map(|_| Message::RefreshWeather),
        ];

        if self.needs(Widget::Media) {
            active.push(Subscription::run(media_stream));
        }
        if self.needs(Widget::Clock) {
            active.push(time::every(Duration::from_secs(5)).map(|_| Message::ClockTick));
        }
        if self.dirty {
            active.push(time::every(Duration::from_secs(2)).map(|_| Message::SaveTick));
        }

        Subscription::batch(active)
    }

    fn view(&self, id: window::Id) -> Element<'_, Message> {
        match self.windows.get(&id) {
            Some(Role::Hub) => self.hub(id),
            Some(Role::Widget(widget)) => self.widget(id, *widget),
            None => Space::new(0.0, 0.0).into(),
        }
    }

    fn hub(&self, id: window::Id) -> Element<'_, Message> {
        let page: Element<'_, Message> = match self.tab {
            Tab::Home => self.board(),
            Tab::Time => row![
                self.card(Widget::Clock, Length::Fill, true),
                self.card(Widget::Weather, Length::Fill, true),
            ]
            .spacing(16)
            .height(Length::Fill)
            .into(),
            Tab::Utility => row![
                weighted(self.card(Widget::Files, Length::Fill, true), 1),
                weighted(self.card(Widget::Photos, Length::Fill, true), 2),
            ]
            .spacing(16)
            .height(Length::Fill)
            .into(),
            Tab::Media => container(
                container(self.card(Widget::Media, Length::Fill, true))
                    .width(520)
                    .height(Length::Fill),
            )
            .center_x(Length::Fill)
            .height(Length::Fill)
            .into(),
            Tab::Social => row![
                self.card(Widget::Discord, Length::Fixed(240.0), false),
                self.card(Widget::Snapchat, Length::Fixed(240.0), false),
            ]
            .spacing(16)
            .into(),
            Tab::Settings => self.settings(),
        };

        let content = column![
            ui::title_bar(id, self.tab),
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
                background: Some(Background::Color(colors::bg())),
                ..container::Style::default()
            })
            .into()
    }

    fn board(&self) -> Element<'_, Message> {
        row![
            column![
                self.card(Widget::Media, Length::FillPortion(2), false),
                self.card(Widget::Snapchat, Length::FillPortion(1), false),
            ]
            .spacing(16)
            .width(Length::FillPortion(11))
            .height(Length::Fill),
            column![
                self.card(Widget::Weather, Length::FillPortion(1), false),
                self.card(Widget::Photos, Length::FillPortion(2), false),
            ]
            .spacing(16)
            .width(Length::FillPortion(10))
            .height(Length::Fill),
            column![
                self.card(Widget::Files, Length::FillPortion(2), false),
                self.card(Widget::Discord, Length::FillPortion(1), false),
            ]
            .spacing(16)
            .width(Length::FillPortion(10))
            .height(Length::Fill),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    }

    fn settings(&self) -> Element<'_, Message> {
        let catalog = Widget::ALL
            .iter()
            .fold(Column::new().spacing(8), |list, &widget| {
                list.push(ui::catalog_row(widget, self.is_open(widget)))
            });

        let layer = row![
            text("Desktop widgets sit").size(14),
            horizontal_space(),
            ui::segment("Behind windows", self.behind, Message::SetBehind(true)),
            ui::segment("On top", !self.behind, Message::SetBehind(false)),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let city = row![
            text("Weather city").size(14),
            horizontal_space(),
            ui::input(&self.city, Message::CityChanged, Message::CitySubmitted),
        ]
        .align_y(Alignment::Center);

        let startup = row![
            text("Start with Windows").size(14),
            horizontal_space(),
            ui::segment("On", self.startup, Message::SetStartup(true)),
            ui::segment("Off", !self.startup, Message::SetStartup(false)),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let list = column![
            text("Desktop widgets").size(22),
            text("Pin widgets to your desktop here or from any tab. Drag them anywhere and right-click to remove one.")
                .size(14)
                .color(colors::muted()),
            catalog,
            layer,
            city,
            startup,
            text("Closing this window keeps your widgets running. Dextra stays in the system tray.")
                .size(13)
                .color(colors::muted()),
            ui::outline_button("Quit Dextra", Message::Quit),
        ]
        .spacing(14)
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 0.0,
        });

        container(scrollable(list))
            .max_width(720)
            .center_x(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn widget(&self, id: window::Id, widget: Widget) -> Element<'_, Message> {
        let (meta, body) = self.parts(widget, false);

        mouse_area(ui::tile(widget.label(), meta, body))
            .on_press(Message::Drag(id))
            .on_right_press(Message::Remove(widget))
            .into()
    }

    fn card(&self, widget: Widget, height: Length, expanded: bool) -> Element<'_, Message> {
        let (meta, body) = self.parts(widget, expanded);
        ui::card(widget, meta, body, height, self.is_open(widget))
    }

    fn parts(&self, widget: Widget, expanded: bool) -> (String, Element<'_, Message>) {
        match widget {
            Widget::Clock => self.clock_parts(expanded),
            Widget::Weather => self.weather_parts(expanded),
            Widget::Media => self.media_parts(expanded),
            Widget::Photos => self.photos_parts(expanded),
            Widget::Files => self.files_parts(),
            Widget::Discord => (
                "Launcher".to_string(),
                ui::launcher("Opens the Discord app.", "Open Discord", "discord://"),
            ),
            Widget::Snapchat => (
                "Launcher".to_string(),
                ui::launcher(
                    "Opens Snapchat Web.",
                    "Open Snapchat",
                    "https://web.snapchat.com",
                ),
            ),
        }
    }

    fn clock_parts(&self, expanded: bool) -> (String, Element<'_, Message>) {
        let size = if expanded { 120.0 } else { 64.0 };

        let body = column![
            text(self.now.format("%H:%M").to_string()).size(size),
            vertical_space(),
            text(self.now.format("%A, %B %-d").to_string())
                .size(14)
                .color(colors::muted()),
        ]
        .height(Length::Fill);

        (String::new(), body.into())
    }

    fn weather_parts(&self, expanded: bool) -> (String, Element<'_, Message>) {
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
                (report.city.clone(), body.into())
            }
            WeatherState::Loading => (
                String::new(),
                text("Loading").color(colors::muted()).into(),
            ),
            WeatherState::Failed(reason) => (
                String::new(),
                text(reason.as_str()).size(13).color(colors::muted()).into(),
            ),
        }
    }

    fn files_parts(&self) -> (String, Element<'_, Message>) {
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

        (
            meta,
            scrollable(list).width(Length::Fill).height(Length::Fill).into(),
        )
    }

    fn photos_parts(&self, expanded: bool) -> (String, Element<'_, Message>) {
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

        ("Recent".to_string(), body)
    }

    fn media_parts(&self, expanded: bool) -> (String, Element<'_, Message>) {
        let Some(track) = &self.media else {
            return (
                String::new(),
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

        (String::new(), body.into())
    }
}

fn weighted(content: Element<'_, Message>, portion: u16) -> Element<'_, Message> {
    container(content)
        .width(Length::FillPortion(portion))
        .height(Length::Fill)
        .into()
}

fn clock(seconds: f32) -> String {
    let total = seconds as u32;
    format!("{}:{:02}", total / 60, total % 60)
}

fn record_panics() {
    std::panic::set_hook(Box::new(|info| {
        if let Some(dir) = dirs::config_dir().map(|dir| dir.join("Dextra")) {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(dir.join("crash.log"), info.to_string());
        }
    }));
}

fn main() -> iced::Result {
    std::env::set_var("ICED_PRESENT_MODE", "no_vsync");
    record_panics();

    let hidden = std::env::args().any(|arg| arg == "--background");
    if !background::claim(hidden) {
        return Ok(());
    }

    iced::daemon(Dextra::title, Dextra::update, Dextra::view)
        .subscription(Dextra::subscription)
        .theme(Dextra::theme)
        .style(Dextra::style)
        .run_with(move || Dextra::new(hidden))
}
