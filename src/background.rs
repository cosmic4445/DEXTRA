use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

const ADDRESS: &str = "127.0.0.1:47213";

static OPEN_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy)]
pub enum Signal {
    Open,
    Quit,
}

pub fn claim(background: bool) -> bool {
    match TcpListener::bind(ADDRESS) {
        Ok(listener) => {
            std::thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    drop(stream);
                    OPEN_REQUESTED.store(true, Ordering::SeqCst);
                }
            });
            true
        }
        Err(_) => {
            if !background {
                let _ = TcpStream::connect(ADDRESS);
            }
            false
        }
    }
}

pub fn poll() -> Vec<Signal> {
    let mut signals = Vec::new();

    if OPEN_REQUESTED.swap(false, Ordering::SeqCst) {
        signals.push(Signal::Open);
    }

    while let Ok(event) = MenuEvent::receiver().try_recv() {
        match event.id.as_ref() {
            "open" => signals.push(Signal::Open),
            "quit" => signals.push(Signal::Quit),
            _ => {}
        }
    }

    while let Ok(event) = TrayIconEvent::receiver().try_recv() {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            signals.push(Signal::Open);
        }
    }

    signals
}

fn glyph() -> Option<Icon> {
    const SIZE: u32 = 32;
    let blocks = [
        (1.0, 1.0, 10.0, 14.0),
        (1.0, 17.0, 9.0, 6.0),
        (13.0, 1.0, 10.0, 6.0),
        (13.0, 9.0, 9.0, 14.0),
    ];
    let scale = SIZE as f32 / 24.0;

    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (px, py) = (x as f32 / scale, y as f32 / scale);
            let lit = blocks
                .iter()
                .any(|&(bx, by, w, h)| px >= bx && px < bx + w && py >= by && py < by + h);
            let shade = if lit { 255 } else { 0x2a };
            pixels.extend_from_slice(&[shade, shade, shade, 255]);
        }
    }

    Icon::from_rgba(pixels, SIZE, SIZE).ok()
}

pub fn tray() -> Option<TrayIcon> {
    let open = MenuItem::with_id("open", "Open Dextra", true, None);
    let quit = MenuItem::with_id("quit", "Quit", true, None);
    let menu = Menu::new();
    menu.append_items(&[&open, &quit]).ok()?;

    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip("Dextra")
        .with_icon(glyph()?)
        .build()
        .ok()
}
