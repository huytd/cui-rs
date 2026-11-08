//! Opens a real (hidden) window. Needs a display, so it is ignored by
//! default: `cargo test -- --ignored`.
#![cfg(feature = "app")]

use cui::*;

#[test]
#[ignore = "needs a display"]
fn hidden_window_renders_and_quits() {
    let desc = AppDesc::new("cui-rs test").size(480, 320).hidden(true).continuous(true);
    let mut app = App::new(&desc).expect("window");
    let mut frames = 0;
    while let Some(mut ui) = app.frame() {
        ui.column_with(BoxOpts::new().pad(24.0).fill().bg(Color::BACKGROUND), |ui| {
            ui.heading("Window", 2);
            ui.button_with("Primary", ButtonOpts::new().primary());
        });
        frames += 1;
        if frames == 5 {
            ui.quit();
        }
    }
    assert!(frames >= 5);
    let (w, h, px) = app.screenshot().expect("screenshot");
    assert!(w > 0 && h > 0);
    assert!(px.chunks(4).any(|p| p != &px[..4]), "blank frame");
    println!("backend={} software={} {w}x{h}", app.backend(), app.is_software());
}
