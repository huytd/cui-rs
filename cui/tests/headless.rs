//! Drives CUI through the software renderer: no window or GPU needed.

use cui::*;

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

fn padded(ui: &mut Ui, f: impl FnOnce(&mut Ui)) -> Rect {
    ui.column_with(BoxOpts::new().pad(20.0).width(Size::FILL), |ui| {
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
    click(&mut h, &mut |ui| padded(ui, |ui| clicks += ui.button("Press me") as i32));
    assert_eq!(clicks, 1);
}

#[test]
fn switch_toggles_a_bool() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut on = false;
    click(&mut h, &mut |ui| padded(ui, |ui| { ui.switch("Wi-Fi", &mut on); }));
    assert!(on);
}

#[test]
fn typing_into_an_input_fills_the_buffer() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut name = TextBuffer::new(32);
    let mut changed = false;
    let mut build = |ui: &mut Ui| padded(ui, |ui| changed |= ui.input("name", &mut name).changed());
    click(&mut h, &mut build);
    h.text("héllo");
    h.frame(DT, |ui| build(ui));
    assert!(changed);
    assert_eq!(name.as_str(), "héllo");
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
    let ran = h.frame(DT, |ui| ui.dialog("Hidden", &mut open, |_, _| ()).is_some());
    assert!(!ran);
    let mut open = true;
    let ran = h.frame(DT, |ui| ui.dialog("Shown", &mut open, |ui, _| ui.text("hi")).is_some());
    assert!(ran);
}

#[test]
fn virtual_list_builds_only_visible_rows() {
    let mut h = Headless::new(400.0, 300.0, 1.0);
    let mut seen = 0..0;
    for _ in 0..3 {
        h.frame(DT, |ui| {
            ui.virtual_list("rows", 1_000_000, 24.0, Size::Px(200.0), |ui, range| {
                for i in range.clone() {
                    ui.text(&format!("Row {i}"));
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
    let mut text = TextBuffer::with_text("text", 64);
    let (mut b, mut f, mut i, mut opt, mut idx) = (true, 0.5f32, 2i32, Some(1usize), 0usize);
    let (mut lo, mut hi, mut num, mut ratio) = (0.2f32, 0.8f32, 3.0f64, 0.4f32);
    let mut date = Date::new(2026, 11, 7);
    let mut color = Color::rgb(0x3b82f6);
    let mut sort = TableSort { column: 0, descending: false };
    for _ in 0..3 {
        h.frame(DT, |ui| {
            ui.scroll_with(ScrollOpts::new().width(Size::FILL).height(Size::FILL), |ui| {
                ui.column_with(BoxOpts::new().pad(16.0).gap(8.0).width(Size::FILL), |ui| {
                    ui.heading("Kitchen sink", 1);
                    ui.text_with("wrapped", TextOpts::new().color(Color::ACCENT).font(Font::MONO));
                    ui.label("label");
                    ui.secondary("secondary");
                    ui.muted("muted");
                    ui.row_with(BoxOpts::new().gap(8.0).wrap(true), |ui| {
                        ui.code("code");
                        ui.kbd("Ctrl K");
                        ui.link("link");
                        ui.icon(icons::STAR, 16.0, Color::WARNING);
                        ui.button_with("Primary", ButtonOpts::new().primary().icon(icons::PLUS));
                        ui.icon_button(icons::SETTINGS, "Settings");
                        ui.button_group(|ui| {
                            ui.button("A");
                            ui.button("B");
                        });
                        ui.copy_button("copy", "copied");
                        ui.theme_toggle();
                        ui.badge_with("New", BadgeOpts::new().tone(Tone::Success).dot(true));
                        ui.avatar("Maya Okafor");
                        ui.avatar_group(&["A B", "C D", "E F"], 2, 24.0);
                        ui.spinner(16.0);
                    });
                    ui.segmented("seg", &["Day", "Week"], &mut idx);
                    ui.checkbox_with("check", &mut b, CheckOpts::new().description("desc"));
                    ui.switch("switch", &mut b);
                    ui.radio_group("rg", &["x", "y"], &mut idx);
                    ui.slider_with("slider", &mut f, 0.0, 1.0, SliderOpts::new().show_value(true).format("%.2f"));
                    ui.range_slider("range", &mut lo, &mut hi, 0.0, 1.0);
                    ui.rating("rating", &mut i, 5);
                    ui.chip("chip", &mut b);
                    ui.input_with("in", &mut text, InputOpts::new().label("Label").placeholder("..."));
                    ui.textarea("ta", &mut text);
                    ui.search("se", &mut text);
                    ui.tag_input("tags", &mut text, Some("Tags"));
                    ui.number("num", &mut num, 0.0, 10.0);
                    ui.select_with("sel", &["one", "two"], &mut opt, SelectOpts::new().icons(&[icons::USER, icons::MAIL][..]));
                    ui.date_picker("date", &mut date);
                    ui.calendar("cal", &mut date);
                    ui.color_picker("color", &mut color);
                    ui.tabs("tabs", &["A", "B"], &mut idx);
                    ui.nav_section("Section");
                    ui.nav_item_with("Inbox", NavOpts::new().icon(icons::MAIL).badge("3"));
                    ui.breadcrumb(&["a", "b", "c"]);
                    ui.pagination("pg", &mut idx, 5);
                    ui.stepper(&["one", "two"], 1);
                    ui.menu("Menu", |ui| {
                        ui.menu_item("item");
                    });
                    ui.button("anchor");
                    ui.tooltip("tip");
                    ui.popover("pop", |ui| ui.text("popover"));
                    ui.context_menu("ctx", |ui| ui.menu_item("ctx item"));
                    ui.alert_with("Alert", AlertOpts::new().tone(Tone::Warning).dismissible(true));
                    ui.progress_with(0.4, ProgressOpts::new().label("Progress").show_value(true));
                    ui.progress_ring("ring", 0.7, 48.0);
                    ui.skeleton(Size::Px(200.0), 12.0);
                    ui.empty_state_with("Empty", EmptyOpts::new().icon(icons::INBOX).action("Add"));
                    ui.metric_with("Revenue", "$12k", MetricOpts::new().delta(4.2).spark(&[1.0, 3.0, 2.0][..]));
                    ui.meter("Disk", 40.0, 100.0, "GB");
                    let mut open = true;
                    ui.accordion("Accordion", &mut open, |ui| ui.text("inside"));
                    ui.tree("Folder", &mut open, |ui| ui.tree_leaf("file.rs", TreeOpts::new()));
                    ui.list_item_with("Item", ListItemOpts::new().subtitle("sub").trailing("now"));
                    let cols = [Column::new("Name").sortable(), Column::new("Size").width(Size::Px(80.0)).align(TextAlign::Right)];
                    ui.table_with("t", &cols, TableOpts::new().striped(true), Some(&mut sort), |ui| {
                        for r in 0..3 {
                            ui.table_row(r == 1);
                            ui.table_cell_text(&format!("row {r}"));
                            ui.table_cell();
                            ui.badge("1 KB");
                        }
                    });
                    let v = [1.0, 4.0, 2.0, 5.0];
                    ui.line_chart_with("lc", &[Series::new("a", &v)], ChartOpts::new().labels(&["q1", "q2", "q3", "q4"][..]).smooth(true));
                    ui.bar_chart("bc", &[Series::new("a", &v), Series::new("b", &v).color(Color::series(3))]);
                    ui.donut_chart("dc", &v, &["a", "b", "c", "d"]);
                    ui.sparkline(&v, Size::Px(100.0), 24.0, Color::ACCENT);
                    ui.gauge("gauge", 0.6, 80.0);
                    ui.code_block_with("cb", "fn main() {}", CodeOpts::new().lang("rust").line_numbers(true));
                    ui.json_view("json", r#"{"a": [1, 2, {"b": null}]}"#);
                    ui.markdown("md", "# Title\n\nSome *markdown* with a [link](https://example.com).\n\n- [ ] task");
                    ui.card_with(CardOpts::new().hoverable(true), |ui| ui.text("card"));
                    ui.split("split", &mut ratio, SplitOpts::new().height(Size::Px(80.0)), |ui| ui.text("left"), |ui| ui.text("right"));
                    ui.id_scope_int(7, |ui| ui.disabled(true, |ui| ui.button("disabled")));
                    let mut sheet = false;
                    ui.sheet("Sheet", &mut sheet, |_, _| ());
                    let mut pal = false;
                    ui.command_palette(&mut pal, &[Command::new("Open").icon(icons::FOLDER)]);
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
