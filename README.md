# Dextra

A widget hub for Windows, written in Rust with iced.

## Run

    cargo run --release

The first build takes a few minutes. To rebuild, quit Dextra first (tray icon, then Quit), since Windows will not overwrite a running program.

## The window

Tabs along the top group the widgets:

- **Home**: everything at a glance.
- **Time**: clock and weather.
- **Utility**: files and photos.
- **Media**: now playing.
- **Social**: Discord and Snapchat shortcuts.
- **Settings**: pick desktop widgets, choose the layer, set the weather city, start with Windows.

## Desktop widgets

Press **Pin to desktop** on any card and the widget appears on your desktop as its own window. Drag it anywhere. Right-click it to remove it. Positions are saved to `%APPDATA%\Dextra\layout.json` and restored on the next start.

## Background

Closing the window hides it and your widgets keep running. Dextra stays in the system tray: click the tray icon (or launch Dextra again) to bring the window back, right-click it for Open and Quit. If the tray icon cannot be created, the close button minimizes the window instead.

If Dextra ever stops unexpectedly, the reason is saved to `%APPDATA%\Dextra\crash.log`.

Release builds open no console window.

Turn on **Start with Windows** in Settings and Dextra starts hidden at login with your widgets in place. It registers the program's current location, so switch it off and on again if you move the file.

## Widgets

- **Clock**: time and date.
- **Weather**: Open-Meteo, refreshed every 15 minutes. Set the city in Settings (plain city name, for example `Toronto`).
- **Now playing**: whatever is playing through Windows media controls, with play/pause, skip and album art.
- **Photos**: the newest images from your Pictures folder.
- **Files**: browse from your home folder and open files.
- **Discord** and **Snapchat**: launcher shortcuts.
