use std::path::{Path, PathBuf};
use std::time::SystemTime;

use iced::widget::image::Handle;

#[derive(Debug, Clone)]
pub struct Photo {
    pub path: PathBuf,
    pub handle: Handle,
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp"
            )
        })
        .unwrap_or(false)
}

fn thumbnail(path: PathBuf) -> Option<Photo> {
    let picture = ::image::open(&path)
        .ok()?
        .resize_to_fill(400, 300, ::image::imageops::FilterType::Triangle)
        .to_rgba8();
    let (width, height) = picture.dimensions();
    Some(Photo {
        handle: Handle::from_rgba(width, height, picture.into_raw()),
        path,
    })
}

fn scan(count: usize) -> Vec<Photo> {
    let Some(dir) = dirs::picture_dir() else {
        return Vec::new();
    };

    let mut files: Vec<(SystemTime, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|item| is_image(&item.path()))
        .filter_map(|item| Some((item.metadata().ok()?.modified().ok()?, item.path())))
        .collect();

    files.sort_by(|a, b| b.0.cmp(&a.0));

    files
        .into_iter()
        .filter_map(|(_, path)| thumbnail(path))
        .take(count)
        .collect()
}

pub async fn load(count: usize) -> Vec<Photo> {
    tokio::task::spawn_blocking(move || scan(count))
        .await
        .unwrap_or_default()
}
