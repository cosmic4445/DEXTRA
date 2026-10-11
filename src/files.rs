use std::fs::DirEntry;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[cfg(windows)]
fn hidden(item: &DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;
    item.metadata()
        .map(|meta| meta.file_attributes() & 0x2 != 0)
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn hidden(_: &DirEntry) -> bool {
    false
}

pub fn list(dir: &Path) -> Vec<Entry> {
    let mut entries: Vec<Entry> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|item| {
            let name = item.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || hidden(&item) {
                return None;
            }
            let is_dir = item.file_type().ok()?.is_dir();
            Some(Entry {
                name,
                path: item.path(),
                is_dir,
            })
        })
        .collect();

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}
