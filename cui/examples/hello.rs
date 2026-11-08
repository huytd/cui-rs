//! The smallest CUI app (a port of CUI's examples/hello.c).

use cui::{App, AppDesc, BoxOpts, ButtonOpts, Size};

fn main() -> Result<(), cui::Error> {
    let mut notifications = true;
    let mut clicks = 0;
    App::run(&AppDesc::new("Hello").size(420, 300), |ui| {
        ui.column_with(BoxOpts::new().pad(32.0).gap(16.0).width(Size::FILL), |ui| {
            ui.heading("Hello, CUI", 2);
            ui.secondary("A native window, a few components and no dependencies.");
            ui.switch("Notifications", &mut notifications);
            ui.row_with(BoxOpts::new().gap(8.0), |ui| {
                if ui.button_with("Click me", ButtonOpts::new().primary()) {
                    clicks += 1;
                }
                ui.text(&format!("Clicked {clicks} times"));
            });
        });
    })
}
