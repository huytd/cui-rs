//! A to-do list: app state in a struct, a sidebar, a list, a dialog, toasts
//! and a keyboard shortcut.

use cui::prelude::*;

struct Task {
    title: String,
    done: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    All,
    Open,
    Done,
}

struct State {
    tasks: Vec<Task>,
    new_task: String,
    filter: Filter,
    confirm_clear: bool,
    dark: bool,
}

fn main() -> Result<(), cui::Error> {
    let mut s = State {
        tasks: ["Port the settings screen", "Write the release notes", "Water the plants"]
            .into_iter()
            .map(|t| Task { title: t.into(), done: false })
            .collect(),
        new_task: String::new(),
        filter: Filter::All,
        confirm_clear: false,
        dark: false,
    };
    App::run(&AppDesc::new("Tasks").size(820, 560).accent(Accent::Violet), |ui| frame(ui, &mut s))
}

fn frame(ui: &mut Ui, s: &mut State) {
    if ui.shortcut(Mods::PRIMARY, Key::Q) {
        ui.quit();
    }
    ui.row().fill().bg(Color::BACKGROUND).show(|ui| {
        sidebar(ui, s);
        ui.separator();
        ui.column().fill().pad(28).gap(16).show(|ui| main_panel(ui, s));
    });

    ui.dialog("Clear completed tasks?", &mut s.confirm_clear)
        .description("This removes them for good.")
        .show(|ui, open| {
            ui.dialog_footer(|ui| {
                if ui.button("Cancel").clicked() {
                    *open = false;
                }
                if ui.button("Clear").danger().clicked() {
                    let before = s.tasks.len();
                    s.tasks.retain(|t| !t.done);
                    ui.toast(&format!("Removed {} tasks", before - s.tasks.len())).tone(Tone::Success);
                    *open = false;
                }
            });
        });
}

fn sidebar(ui: &mut Ui, s: &mut State) {
    let open = s.tasks.iter().filter(|t| !t.done).count().to_string();
    ui.column().width(220).fill_height().pad(16).gap(4).show(|ui| {
        ui.nav_section("Tasks");
        for (label, icon, filter, badge) in [
            ("All", icons::LIST, Filter::All, None),
            ("Open", icons::CIRCLE, Filter::Open, Some(open.as_str())),
            ("Done", icons::CIRCLE_CHECK, Filter::Done, None),
        ] {
            let item = ui.nav_item(label).icon(icon).badge(badge).selected(s.filter == filter).group("filter");
            if item.clicked() {
                s.filter = filter;
            }
        }
        ui.spacer();
        if ui.switch("Dark mode", &mut s.dark).changed() {
            ui.set_theme(if s.dark { ThemeMode::Dark } else { ThemeMode::Light });
        }
    });
}

fn main_panel(ui: &mut Ui, s: &mut State) {
    ui.heading("Tasks", 2);
    ui.row().gap(8).fill_width().show(|ui| {
        let ev = ui.input("new", &mut s.new_task).placeholder("Add a task and press Enter").icon(icons::PLUS).width(FILL).show();
        let add = ui.button("Add").primary().clicked();
        if (ev.submitted() || add) && !s.new_task.trim().is_empty() {
            s.tasks.push(Task { title: s.new_task.trim().to_owned(), done: false });
            s.new_task.clear();
        }
    });

    let visible: Vec<usize> = (0..s.tasks.len())
        .filter(|&i| match s.filter {
            Filter::All => true,
            Filter::Open => !s.tasks[i].done,
            Filter::Done => s.tasks[i].done,
        })
        .collect();

    if visible.is_empty() {
        ui.empty_state("Nothing here").icon(icons::INBOX).description("Add a task above.");
    }
    ui.scroll().fill().gap(2).show(|ui| {
        let mut remove = None;
        for i in visible {
            ui.id_scope(i, |ui| {
                ui.row().gap(8).fill_width().show(|ui| {
                    let task = &mut s.tasks[i];
                    ui.checkbox(&task.title, &mut task.done);
                    ui.spacer();
                    if ui.icon_button(icons::TRASH_2).tooltip("Delete").clicked() {
                        remove = Some(i);
                    }
                });
            });
        }
        if let Some(i) = remove {
            let t = s.tasks.remove(i);
            ui.toast(&format!("Deleted \"{}\"", t.title));
        }
    });

    ui.row().gap(8).fill_width().show(|ui| {
        let done = s.tasks.iter().filter(|t| t.done).count();
        ui.muted(&format!("{done} of {} done", s.tasks.len()));
        ui.spacer();
        if ui.button("Clear completed").ghost().icon(icons::TRASH_2).disabled(done == 0).clicked() {
            s.confirm_clear = true;
        }
    });
}
