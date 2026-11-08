//! The smallest CUI app (a port of CUI's examples/hello.c).

use cui::prelude::*;

fn main() -> Result<(), cui::Error> {
    let mut notifications = true;
    let mut clicks = 0;
    App::run(&AppDesc::new("Hello").size(420, 300), |ui| {
        ui.column().pad(32).gap(16).fill_width().show(|ui| {
            ui.heading("Hello, CUI", 2);
            ui.secondary("A native window, a few components and no dependencies.");
            ui.switch("Notifications", &mut notifications);
            ui.row().gap(8).show(|ui| {
                if ui.button("Click me").primary().clicked() {
                    clicks += 1;
                }
                text!(ui, "Clicked {clicks} times");
            });
        });
    })
}
