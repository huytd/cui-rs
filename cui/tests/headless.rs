//! Drives CUI through the software renderer: no window or GPU needed.

use cui::prelude::*;
use cui::{ButtonOpts, Headless, Rect};

const DT: f32 = 1.0 / 60.0;

/// Builds two frames so layout settles; `f` returns the rect to click.
fn settle(h: &mut Headless, f: &mut dyn FnMut(&mut Ui) -> Rect) -> Rect {
    h.frame(DT, |ui| f(ui));
    h.frame(DT, |ui| f(ui))
}

fn click(h: &mut Headless, f: &mut dyn FnMut(&mut Ui) -> Rect) {
    let r = settle(h, f);
    h.mouse_move(r.x + r.w / 2.0, r.y + r.h / 2.0);
    h.frame(DT, |ui| f(ui));
    h.mouse_button(MouseButton::Left, true);
    h.frame(DT, |ui| f(ui));
    h.mouse_button(MouseButton::Left, false);
    h.frame(DT, |ui| f(ui));
}

/// Builds `f` in a padded column and returns the rect of its last component.
fn padded(ui: &mut Ui, f: impl FnOnce(&mut Ui)) -> Rect {
    ui.column().pad(20).fill_width().show(|ui| {
        f(ui);
        ui.last_rect()
    })
}

#[test]
fn version_and_commit() {
    assert_eq!(cui::cui_version().split('.').count(), 3);
    assert_eq!(cui::CUI_COMMIT.trim().len(), 40);
}

#[test]
fn button_click_is_reported_once() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut clicks = 0;
    let r = settle(&mut h, &mut |ui| padded(ui, |ui| { ui.button("Press me"); }));
    assert!(r.w > 0.0 && r.h > 0.0, "button has no size: {r:?}");
    click(&mut h, &mut |ui| padded(ui, |ui| clicks += ui.button("Press me").primary().clicked() as i32));
    assert_eq!(clicks, 1);
}

#[test]
fn a_dropped_builder_draws_at_the_end_of_its_statement() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let build = |ui: &mut Ui| {
        ui.column().pad(20).show(|ui| {
            ui.button("Plain");
            let plain = ui.last_rect();
            ui.button("Chained").icon(icons::PLUS).size(ControlSize::Lg);
            (plain, ui.last_rect())
        })
    };
    // Rects come from the latest layout, so build twice.
    h.frame(DT, |ui| build(ui));
    let (plain, chained) = h.frame(DT, |ui| build(ui));
    assert!(plain.h > 0.0);
    assert!(chained.y > plain.y, "second button should come after the first: {plain:?} {chained:?}");
    assert!(chained.h > plain.h, "the large size option should apply");
}

#[test]
fn switch_toggles_a_bool() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut on = false;
    click(&mut h, &mut |ui| padded(ui, |ui| { ui.switch("Wi-Fi", &mut on); }));
    assert!(on);
}

#[test]
fn typing_into_a_string_field() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut name = String::from("hi ");
    let mut changed = false;
    let mut build = |ui: &mut Ui| padded(ui, |ui| changed |= ui.input("name", &mut name).placeholder("Name").changed());
    click(&mut h, &mut build);
    h.text("héllo");
    h.frame(DT, |ui| build(ui));
    assert!(changed);
    // Where the caret lands depends on the font, so only check both parts survived.
    assert!(name.contains("héllo") && name.contains("hi"), "got {name:?}");
    assert_eq!(name.len(), "hi héllo".len());
}

#[test]
fn typing_into_a_text_buffer() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut name = TextBuffer::new(32);
    let mut build = |ui: &mut Ui| padded(ui, |ui| { ui.input("name", &mut name); });
    click(&mut h, &mut build);
    h.text("abc");
    h.frame(DT, |ui| build(ui));
    assert_eq!(name.as_str(), "abc");
}

#[test]
fn radio_sets_its_option() {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Plan {
        Free,
        Pro,
    }
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut plan = Plan::Free;
    click(&mut h, &mut |ui| {
        padded(ui, |ui| {
            ui.radio("Free", &mut plan, Plan::Free);
            ui.radio("Pro", &mut plan, Plan::Pro);
        })
    });
    assert_eq!(plan, Plan::Pro);
}

#[test]
fn closed_containers_skip_their_closure() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut open = false;
    let ran = h.frame(DT, |ui| ui.dialog("Hidden", &mut open).show(|_, _| ()).is_some());
    assert!(!ran);
    let mut open = true;
    let ran = h.frame(DT, |ui| {
        ui.dialog("Shown", &mut open).description("Body").show(|ui, open| {
            ui.text("hi");
            *open = false;
        })
    });
    assert!(ran.is_some());
    assert!(!open, "the closure can close the dialog");
}

#[test]
fn option_presets_apply_with_opts() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let big = ButtonOpts::new().size(ControlSize::Lg).width(200);
    let r = settle(&mut h, &mut |ui| {
        ui.column().show(|ui| {
            ui.button("Preset").opts(big);
            ui.last_rect()
        })
    });
    assert_eq!(r.w, 200.0);
}

#[test]
fn virtual_list_builds_only_visible_rows() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut seen = 0..0;
    for _ in 0..3 {
        h.frame(DT, |ui| {
            ui.virtual_list("rows", 1_000_000, 24, 200, |ui, range| {
                for i in range.clone() {
                    text!(ui, "Row {i}");
                }
                seen = range;
            });
        });
    }
    assert!(!seen.is_empty());
    assert!(seen.len() < 50, "built {} rows", seen.len());
}

/// Every wrapped component in one frame, rendered to pixels.
#[test]
fn kitchen_sink_renders() {
    let mut h = Headless::with_theme(1000.0, 900.0, 1.0, ThemeMode::Dark);
    let mut text = String::from("text");
    let mut tags = TextBuffer::with_text("rust", 64);
    let (mut b, mut f, mut i, mut opt, mut idx) = (true, 0.5f32, 2i32, Some(1usize), 0usize);
    let (mut lo, mut hi, mut num, mut ratio) = (0.2f32, 0.8f32, 3.0f64, 0.4f32);
    let mut date = Date::new(2026, 11, 7);
    let mut color = Color::rgb(0x3b82f6);
    let mut sort = TableSort { column: 0, descending: false };
    for _ in 0..3 {
        h.frame(DT, |ui| {
            ui.scroll().fill().show(|ui| {
                ui.column().pad(16).gap(8).fill_width().show(|ui| {
                    ui.heading("Kitchen sink", 1);
                    ui.text("wrapped").color(Color::ACCENT).font(Font::MONO);
                    text!(ui, "{} formatted", 42).tabular(true);
                    ui.label("label");
                    ui.secondary("secondary");
                    ui.muted("muted");
                    ui.row().gap(8).wrap(true).show(|ui| {
                        ui.code("code");
                        ui.kbd("Ctrl K");
                        ui.link("link");
                        ui.icon(icons::STAR, 16, Token::Warning);
                        ui.button("Primary").primary().icon(icons::PLUS);
                        ui.icon_button(icons::SETTINGS).tooltip("Settings");
                        ui.button_group(|ui| {
                            ui.button("A");
                            ui.button("B");
                        });
                        ui.copy_button("copy", "copied");
                        ui.theme_toggle();
                        ui.badge("New").tone(Tone::Success).dot(true);
                        ui.avatar("Maya Okafor").status("online");
                        ui.avatar_group(&["A B", "C D", "E F"], 2, 24);
                        ui.spinner(16);
                    });
                    ui.segmented("seg", &["Day", "Week"], &mut idx).icons(&[icons::SUN, icons::MOON]);
                    ui.checkbox("check", &mut b).description("desc");
                    ui.switch("switch", &mut b);
                    ui.radio_group("rg", &["x", "y"], &mut idx);
                    ui.slider("slider", &mut f, 0, 1).show_value(true).format("%.2f").step(0.05);
                    ui.range_slider("range", &mut lo, &mut hi, 0, 1);
                    ui.rating("rating", &mut i, 5);
                    ui.chip("chip", &mut b).icon(icons::CHECK);
                    ui.input("in", &mut text).label("Label").placeholder("...").clearable(true);
                    ui.textarea("ta", &mut text).height(80);
                    ui.search("se", &mut text);
                    ui.tag_input("tags", &mut tags).label("Tags");
                    ui.number("num", &mut num, 0, 10).step(0.5).decimals(1);
                    ui.select("sel", &["one", "two"], &mut opt).icons(&[icons::USER, icons::MAIL]).width(240);
                    ui.date_picker("date", &mut date).label("Date");
                    ui.calendar("cal", &mut date);
                    ui.color_picker("color", &mut color);
                    ui.tabs("tabs", &["A", "B"], &mut idx).variant(TabsVariant::Pill);
                    ui.nav_section("Section");
                    ui.nav_item("Inbox").icon(icons::MAIL).badge("3").selected(true);
                    ui.breadcrumb(&["a", "b", "c"]);
                    ui.pagination("pg", &mut idx, 5);
                    ui.stepper(&["one", "two"], 1);
                    ui.menu("Menu").icon(icons::ELLIPSIS).show(|ui| {
                        ui.menu_item("item").shortcut("Ctrl I");
                    });
                    ui.button("anchor");
                    ui.tooltip("tip");
                    ui.popover("pop").width(200).show(|ui| { ui.text("popover"); });
                    ui.context_menu("ctx", |ui| ui.menu_item("ctx item").clicked());
                    ui.alert("Alert").tone(Tone::Warning).dismissible(true);
                    ui.progress(0.4).label("Progress").show_value(true);
                    ui.progress_ring("ring", 0.7, 48);
                    ui.skeleton(200, 12);
                    ui.empty_state("Empty").icon(icons::INBOX).action("Add");
                    ui.metric("Revenue", "$12k").delta(4.2).spark(&[1.0, 3.0, 2.0]);
                    ui.meter("Disk", 40.0, 100.0, "GB");
                    let mut open = true;
                    ui.accordion("Accordion", &mut open, |ui| { ui.text("inside"); });
                    ui.tree("Folder", &mut open).show(|ui| ui.tree_leaf("file.rs").icon(icons::FILE).clicked());
                    ui.list_item("Item").subtitle("sub").trailing("now");
                    let cols = [Column::new("Name").sortable(), Column::new("Size").width(80).align(TextAlign::Right)];
                    ui.table("t", &cols).striped(true).sort(&mut sort).show(|ui| {
                        for r in 0..3 {
                            ui.table_row(r == 1);
                            ui.table_cell_text(&format!("row {r}"));
                            ui.table_cell();
                            ui.badge("1 KB");
                        }
                    });
                    let v = [1.0, 4.0, 2.0, 5.0];
                    let a = [Series::new("a", &v)];
                    ui.line_chart("lc", &a).labels(&["q1", "q2", "q3", "q4"]).smooth(true);
                    let ab = [Series::new("a", &v), Series::new("b", &v).color(Color::series(3))];
                    ui.bar_chart("bc", &ab).stacked(true);
                    ui.donut_chart("dc", &v, &["a", "b", "c", "d"]);
                    ui.sparkline(&v, 100, 24, Color::ACCENT);
                    ui.gauge("gauge", 0.6, 80);
                    ui.code_block("cb", "fn main() {}").lang("rust").line_numbers(true);
                    ui.json_view("json", r#"{"a": [1, 2, {"b": null}]}"#);
                    ui.markdown("md", "# Title\n\nSome *markdown* with a [link](https://example.com).\n\n- [ ] task");
                    ui.card().hoverable(true).clicked(|ui| { ui.text("card"); });
                    ui.split("split", &mut ratio).height(80).show(|ui| { ui.text("left"); }, |ui| { ui.text("right"); });
                    ui.id_scope(7usize, |ui| ui.disabled(true, |ui| ui.button("disabled").clicked()));
                    let mut sheet = false;
                    ui.sheet("Sheet", &mut sheet).side(Side::Left).show(|_, _| ());
                    let mut pal = false;
                    ui.command_palette(&mut pal, &[Command::new("Open").icon(icons::FOLDER)]);
                    ui.toast("Toast").description("queued").show();
                });
            });
        });
    }
    let (w, hh, px) = h.render_rgba();
    assert_eq!((w, hh), (1000, 900));
    // Something other than the background got drawn.
    let first = &px[..4];
    assert!(px.chunks(4).any(|p| p != first));
    if let Ok(dir) = std::env::var("CUI_SHOTS") {
        h.save_png(&format!("{dir}/kitchen_sink.png"));
    }
}
