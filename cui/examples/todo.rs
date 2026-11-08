//! A to-do list: app state in a struct, a sidebar, a list, a dialog, toasts
//! and a keyboard shortcut.

use cui::*;

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
    new_task: TextBuffer,
    filter: Filter,
    confirm_clear: bool,
    dark: bool,
}

fn main() -> Result<(), Error> {
    let mut s = State {
        tasks: ["Port the settings screen", "Write the release notes", "Water the plants"]
            .into_iter()
            .map(|t| Task { title: t.into(), done: false })
            .collect(),
        new_task: TextBuffer::new(128),
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
    ui.row_with(BoxOpts::new().fill().bg(Color::BACKGROUND), |ui| {
        sidebar(ui, s);
        ui.separator();
        ui.column_with(BoxOpts::new().fill().pad(28.0).gap(16.0), |ui| main_panel(ui, s));
    });

    ui.dialog_with(
        "Clear completed tasks?",
        &mut s.confirm_clear,
        DialogOpts::new().description("This removes them for good."),
        |ui, open| {
            ui.dialog_footer(|ui| {
                if ui.button("Cancel") {
                    *open = false;
                }
                if ui.button_with("Clear", ButtonOpts::new().danger()) {
                    let before = s.tasks.len();
                    s.tasks.retain(|t| !t.done);
                    ui.toast_with(
                        &format!("Removed {} tasks", before - s.tasks.len()),
                        ToastOpts::new().tone(Tone::Success),
                    );
                    *open = false;
                }
            });
        },
    );
}

fn sidebar(ui: &mut Ui, s: &mut State) {
    let open = s.tasks.iter().filter(|t| !t.done).count().to_string();
    ui.column_with(BoxOpts::new().width(Size::Px(220.0)).height(Size::FILL).pad(16.0).gap(4.0), |ui| {
        ui.nav_section("Tasks");
        for (label, icon, filter, badge) in [
            ("All", icons::LIST, Filter::All, None),
            ("Open", icons::CIRCLE, Filter::Open, Some(open.as_str())),
            ("Done", icons::CIRCLE_CHECK, Filter::Done, None),
        ] {
            let mut opts = NavOpts::new().icon(icon).selected(s.filter == filter).group("filter");
            if let Some(b) = badge {
                opts = opts.badge(b);
            }
            if ui.nav_item_with(label, opts) {
                s.filter = filter;
            }
        }
        ui.spacer();
        if ui.switch("Dark mode", &mut s.dark) {
            ui.set_theme(if s.dark { ThemeMode::Dark } else { ThemeMode::Light });
        }
    });
}

fn main_panel(ui: &mut Ui, s: &mut State) {
    ui.heading("Tasks", 2);
    ui.row_with(BoxOpts::new().gap(8.0).width(Size::FILL), |ui| {
        let ev = ui.input_with(
            "new",
            &mut s.new_task,
            InputOpts::new().placeholder("Add a task and press Enter").icon(icons::PLUS).width(Size::FILL),
        );
        let add = ui.button_with("Add", ButtonOpts::new().primary());
        if (ev.submitted() || add) && !s.new_task.as_str().trim().is_empty() {
            s.tasks.push(Task { title: s.new_task.as_str().trim().to_owned(), done: false });
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
        ui.empty_state_with("Nothing here", EmptyOpts::new().icon(icons::INBOX).description("Add a task above."));
    }
    ui.scroll_with(ScrollOpts::new().width(Size::FILL).height(Size::FILL).gap(2.0), |ui| {
        let mut remove = None;
        for i in visible {
            ui.id_scope_int(i as i32, |ui| {
                ui.row_with(BoxOpts::new().gap(8.0).width(Size::FILL), |ui| {
                    let task = &mut s.tasks[i];
                    ui.checkbox(&task.title, &mut task.done);
                    ui.spacer();
                    if ui.icon_button(icons::TRASH_2, "Delete") {
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

    ui.row_with(BoxOpts::new().gap(8.0).width(Size::FILL), |ui| {
        let done = s.tasks.iter().filter(|t| t.done).count();
        ui.muted(&format!("{done} of {} done", s.tasks.len()));
        ui.spacer();
        let clear = ButtonOpts::new().ghost().icon(icons::TRASH_2).disabled(done == 0);
        if ui.button_with("Clear completed", clear) {
            s.confirm_clear = true;
        }
    });
}
