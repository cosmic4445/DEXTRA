use iced::widget::image::Handle;

#[derive(Debug, Clone, Copy)]
pub enum Control {
    Previous,
    Toggle,
    Next,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub position: f32,
    pub duration: f32,
    pub art: Option<Handle>,
}

pub async fn read(known_title: Option<String>) -> Option<Snapshot> {
    tokio::task::spawn_blocking(move || platform::read(known_title))
        .await
        .ok()
        .flatten()
}

pub async fn control(action: Control) {
    let _ = tokio::task::spawn_blocking(move || platform::control(action)).await;
}

#[cfg(windows)]
mod platform {
    use super::{Control, Snapshot};
    use iced::widget::image::Handle;
    use windows::core::Result;
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionMediaProperties as Properties,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };
    use windows::Storage::Streams::DataReader;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    fn session() -> Result<Session> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        Manager::RequestAsync()?.get()?.GetCurrentSession()
    }

    fn artwork(properties: &Properties) -> Result<Vec<u8>> {
        let stream = properties.Thumbnail()?.OpenReadAsync()?.get()?;
        let size = stream.Size()? as u32;
        let reader = DataReader::CreateDataReader(&stream)?;
        reader.LoadAsync(size)?.get()?;
        let mut bytes = vec![0u8; size as usize];
        reader.ReadBytes(&mut bytes)?;
        Ok(bytes)
    }

    fn snapshot(known_title: Option<String>) -> Result<Snapshot> {
        let session = session()?;
        let properties = session.TryGetMediaPropertiesAsync()?.get()?;
        let title = properties.Title()?.to_string();
        let artist = properties.Artist()?.to_string();
        let playing = session.GetPlaybackInfo()?.PlaybackStatus()? == Status::Playing;

        let timeline = session.GetTimelineProperties()?;
        let start = timeline.StartTime()?.Duration;
        let position = (timeline.Position()?.Duration - start) as f32 / 10_000_000.0;
        let duration = (timeline.EndTime()?.Duration - start) as f32 / 10_000_000.0;

        let art = if known_title.as_deref() == Some(title.as_str()) {
            None
        } else {
            artwork(&properties).ok().map(Handle::from_bytes)
        };

        Ok(Snapshot {
            title,
            artist,
            playing,
            position: position.max(0.0),
            duration: duration.max(0.0),
            art,
        })
    }

    fn send(action: Control) -> Result<()> {
        let session = session()?;
        match action {
            Control::Previous => session.TrySkipPreviousAsync()?.get()?,
            Control::Toggle => session.TryTogglePlayPauseAsync()?.get()?,
            Control::Next => session.TrySkipNextAsync()?.get()?,
        };
        Ok(())
    }

    pub fn read(known_title: Option<String>) -> Option<Snapshot> {
        snapshot(known_title).ok()
    }

    pub fn control(action: Control) {
        let _ = send(action);
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{Control, Snapshot};

    pub fn read(_: Option<String>) -> Option<Snapshot> {
        None
    }

    pub fn control(_: Control) {}
}
