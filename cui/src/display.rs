use crate::builder::{Container, IntoOpt, Run, Widget};
use crate::opts::*;
use crate::sys;
use crate::types::*;
use crate::ui::cstr;
use crate::widgets::bool_result;
use crate::Ui;
use core::cell::Cell;
use core::ffi::c_char;
use core::ops::Range;

type CStr = *const c_char;

/// A data series for line and bar charts.
#[derive(Clone, Copy, Debug)]
pub struct Series<'a> {
    pub name: &'a str,
    pub values: &'a [f32],
    /// Zero picks the next series palette color.
    pub color: Color,
}

impl<'a> Series<'a> {
    pub fn new(name: &'a str, values: &'a [f32]) -> Self {
        Series { name, values, color: Color::default() }
    }
    pub fn color(mut self, color: impl IntoOpt<Color>) -> Self {
        self.color = color.into_opt();
        self
    }
}

/// A table column.
#[derive(Clone, Copy, Debug, Default)]
pub struct Column<'a> {
    pub title: &'a str,
    /// Default: fill.
    pub width: Size,
    pub align: TextAlign,
    pub sortable: bool,
}

impl<'a> Column<'a> {
    pub fn new(title: &'a str) -> Self {
        Column { title, ..Default::default() }
    }
    pub fn width(mut self, width: impl IntoOpt<Size>) -> Self {
        self.width = width.into_opt();
        self
    }
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
}

/// The sort a table header shows; updated when a sortable header is clicked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableSort {
    /// Column index, or -1 for none.
    pub column: i32,
    pub descending: bool,
}

/// What a markdown view reported this frame.
#[derive(Clone, Debug, Default)]
pub struct MarkdownResponse {
    /// URL of the link clicked (or activated with Enter) this frame.
    pub link: Option<String>,
    /// URL of the link under the pointer.
    pub hovered_link: Option<String>,
    /// A task box was toggled: byte offset in the text of the character
    /// between its brackets, and the state it was toggled to.
    pub task: Option<(usize, bool)>,
    /// Height of the view in the latest layout.
    pub height: f32,
}

/// A progress bar (`cui_progress`).
pub struct Progress(f32);

/// A badge (`cui_badge`).
pub struct Badge(CStr);

/// An avatar (`cui_avatar`).
pub struct Avatar(CStr);

/// An empty state (`cui_empty_state`).
pub struct EmptyState(CStr);

/// A KPI tile (`cui_metric`).
pub struct Metric {
    label: CStr,
    value: CStr,
}

/// A tree node with children (`cui_tree`).
pub struct Tree<'v> {
    label: CStr,
    open: &'v mut bool,
}

/// A tree node without children.
pub struct TreeLeaf(CStr);

/// A list row (`cui_list_item`).
pub struct ListItem(CStr);

/// A table (`cui_table`).
pub struct Table<'s> {
    id: CStr,
    columns: Vec<sys::cui_table_column>,
    sort: Option<&'s mut TableSort>,
}

#[derive(Clone, Copy)]
enum ChartKind {
    Line,
    Bar,
}

/// A line or bar chart.
pub struct Chart<'d> {
    id: CStr,
    series: &'d [Series<'d>],
    kind: ChartKind,
}

/// A donut chart (`cui_donut_chart`).
pub struct Donut<'d> {
    id: CStr,
    values: &'d [f32],
    labels: *const *const c_char,
}

/// An image (`cui_image`).
pub struct Image(Texture);

/// A code block (`cui_code_block`).
pub struct CodeBlock {
    id: CStr,
    code: CStr,
}

/// A markdown view (`cui_markdown`).
pub struct Markdown<'t> {
    id: CStr,
    text: &'t str,
}

/// Feedback, data display, tables, charts, images, code and markdown.
impl<'f> Ui<'f> {
    // ---- feedback ----

    /// A progress bar; `value` is 0..1.
    pub fn progress<'a>(&mut self, value: f32) -> Widget<'_, 'f, ProgressOpts<'a>, Progress> {
        Widget::new(self, ProgressOpts::new(), Progress(value))
    }
    /// Circular progress with the value in the middle.
    pub fn progress_ring(&mut self, label: &str, value: f32, size: impl IntoOpt<f32>) {
        let l = self.s(label);
        unsafe { sys::cui_progress_ring(self.ctx, l, value, size.into_opt()) }
    }
    pub fn spinner(&mut self, size: impl IntoOpt<f32>) {
        unsafe { sys::cui_spinner(self.ctx, size.into_opt()) }
    }
    /// A loading placeholder.
    pub fn skeleton(&mut self, width: impl IntoOpt<Size>, height: impl IntoOpt<f32>) {
        unsafe { sys::cui_skeleton(self.ctx, width.into_opt().into(), height.into_opt()) }
    }
    pub fn badge<'a>(&mut self, text: &str) -> Widget<'_, 'f, BadgeOpts<'a>, Badge> {
        let t = self.s(text);
        Widget::new(self, BadgeOpts::new(), Badge(t))
    }
    /// Initials (or an image) in a circle.
    pub fn avatar<'a>(&mut self, name: &str) -> Widget<'_, 'f, AvatarOpts<'a>, Avatar> {
        let n = self.s(name);
        Widget::new(self, AvatarOpts::new(), Avatar(n))
    }
    pub fn avatar_group(&mut self, names: &[&str], max_visible: usize, size: impl IntoOpt<f32>) {
        let n = self.strs(names);
        unsafe { sys::cui_avatar_group(self.ctx, n, names.len() as i32, max_visible as i32, size.into_opt()) }
    }
    /// An empty state. `.clicked()` tells whether its action button was clicked.
    pub fn empty_state<'a>(&mut self, title: &str) -> Widget<'_, 'f, EmptyOpts<'a>, EmptyState> {
        let t = self.s(title);
        Widget::new(self, EmptyOpts::new(), EmptyState(t))
    }
    /// A KPI tile: a label, a big value, an optional delta and sparkline.
    pub fn metric<'a>(&mut self, label: &str, value: &str) -> Widget<'_, 'f, MetricOpts<'a>, Metric> {
        let (label, value) = (self.s(label), self.s(value));
        Widget::new(self, MetricOpts::new(), Metric { label, value })
    }
    pub fn meter(&mut self, label: &str, used: f32, total: f32, unit: &str) {
        let (l, u) = (self.s(label), self.s(unit));
        unsafe { sys::cui_meter(self.ctx, l, used, total, u) }
    }

    // ---- data display ----

    /// A disclosure whose content animates its height. Clicking the title toggles `*open`.
    pub fn accordion<R>(&mut self, title: &str, open: &mut bool, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        let t = self.s(title);
        if !unsafe { sys::cui_accordion_begin(self.ctx, t, open) } {
            return None;
        }
        let r = f(self);
        unsafe { sys::cui_accordion_end(self.ctx) };
        Some(r)
    }
    /// A tree node; `.show(|ui| ...)` builds its children while it is open.
    /// Check [`Ui::tree_clicked`] after it for selection.
    pub fn tree<'a, 'v>(&mut self, label: &str, open: &'v mut bool) -> Container<'_, 'f, TreeOpts<'a>, Tree<'v>> {
        let l = self.s(label);
        Container::new(self, TreeOpts::new(), Tree { label: l, open })
    }
    /// A tree node without children. `.clicked()` tells whether it was clicked.
    pub fn tree_leaf<'a>(&mut self, label: &str) -> Widget<'_, 'f, TreeOpts<'a>, TreeLeaf> {
        let l = self.s(label);
        Widget::new(self, TreeOpts::new(), TreeLeaf(l))
    }
    /// The node made by the last [`Ui::tree`] was clicked.
    pub fn tree_clicked(&mut self) -> bool {
        unsafe { sys::cui_tree_clicked(self.ctx) }
    }
    /// A selectable row with optional icon, subtitle and trailing text.
    pub fn list_item<'a>(&mut self, title: &str) -> Widget<'_, 'f, ListItemOpts<'a>, ListItem> {
        let t = self.s(title);
        Widget::new(self, ListItemOpts::new(), ListItem(t))
    }
    /// Only the visible rows are built: the closure gets their index range.
    pub fn virtual_list<R>(
        &mut self,
        id: &str,
        count: usize,
        row_height: impl IntoOpt<f32>,
        height: impl IntoOpt<Size>,
        f: impl FnOnce(&mut Self, Range<usize>) -> R,
    ) -> R {
        let id = self.s(id);
        let (mut first, mut last) = (0i32, -1i32);
        unsafe {
            sys::cui_virtual_list_begin(
                self.ctx,
                id,
                count as i32,
                row_height.into_opt(),
                height.into_opt().into(),
                &mut first,
                &mut last,
            )
        };
        let range = first.max(0) as usize..(last + 1).max(first).max(0) as usize;
        let r = f(self, range);
        unsafe { sys::cui_virtual_list_end(self.ctx) };
        r
    }

    /// A table. Inside `.show(|ui| ...)`, start rows with [`Ui::table_row`]
    /// and cells with [`Ui::table_cell`] or [`Ui::table_cell_text`].
    pub fn table<'a, 's>(&mut self, id: &str, columns: &[Column]) -> Container<'_, 'f, TableOpts<'a>, Table<'s>> {
        let id = self.s(id);
        let columns = columns
            .iter()
            .map(|c| sys::cui_table_column {
                title: self.arena.str(c.title),
                width: if c.width == Size::Fit { Size::FILL.into() } else { c.width.into() },
                align: c.align.to_raw(),
                sortable: c.sortable,
            })
            .collect();
        Container::new(self, TableOpts::new(), Table { id, columns, sort: None })
    }
    /// Begins a row. Returns true when the row is clicked.
    pub fn table_row(&mut self, selected: bool) -> bool {
        unsafe { sys::cui_table_row(self.ctx, selected) }
    }
    /// Moves to the next cell; build any components after it.
    pub fn table_cell(&mut self) {
        unsafe { sys::cui_table_cell(self.ctx) }
    }
    /// The next cell, holding plain text.
    pub fn table_cell_text(&mut self, text: &str) {
        let t = self.s(text);
        unsafe { sys::cui_table_cell_text(self.ctx, t) }
    }

    // ---- charts ----

    pub fn line_chart<'a, 'd>(&mut self, id: &str, series: &'d [Series<'d>]) -> Widget<'_, 'f, ChartOpts<'a>, Chart<'d>> {
        let id = self.s(id);
        Widget::new(self, ChartOpts::new(), Chart { id, series, kind: ChartKind::Line })
    }
    pub fn bar_chart<'a, 'd>(&mut self, id: &str, series: &'d [Series<'d>]) -> Widget<'_, 'f, ChartOpts<'a>, Chart<'d>> {
        let id = self.s(id);
        Widget::new(self, ChartOpts::new(), Chart { id, series, kind: ChartKind::Bar })
    }
    /// `values` and `labels` have one entry per slice.
    pub fn donut_chart<'a, 'd>(&mut self, id: &str, values: &'d [f32], labels: &[&str]) -> Widget<'_, 'f, ChartOpts<'a>, Donut<'d>> {
        let n = values.len().min(labels.len());
        let (id, labels) = (self.s(id), self.strs(&labels[..n]));
        Widget::new(self, ChartOpts::new(), Donut { id, values: &values[..n], labels })
    }
    pub fn sparkline(&mut self, values: &[f32], width: impl IntoOpt<Size>, height: impl IntoOpt<f32>, color: impl IntoOpt<Color>) {
        unsafe {
            sys::cui_sparkline(
                self.ctx,
                values.as_ptr(),
                values.len() as i32,
                width.into_opt().into(),
                height.into_opt(),
                color.into_opt().into(),
            )
        }
    }
    pub fn gauge(&mut self, label: &str, value: f32, size: impl IntoOpt<f32>) {
        let l = self.s(label);
        unsafe { sys::cui_gauge(self.ctx, l, value, size.into_opt()) }
    }

    // ---- images ----

    pub fn image<'a>(&mut self, tex: Texture) -> Widget<'_, 'f, ImageOpts<'a>, Image> {
        Widget::new(self, ImageOpts::new(), Image(tex))
    }

    // ---- code and markdown ----

    /// Code with syntax highlighting, line numbers and a copy button.
    pub fn code_block<'a>(&mut self, id: &str, code: &str) -> Widget<'_, 'f, CodeOpts<'a>, CodeBlock> {
        let (id, code) = (self.s(id), self.s(code));
        Widget::new(self, CodeOpts::new(), CodeBlock { id, code })
    }
    /// A collapsible JSON tree. Returns false when `json` is not valid.
    pub fn json_view(&mut self, id: &str, json: &str) -> bool {
        let (id, j) = (self.s(id), self.s(json));
        let o = sys::cui_json_opts::default();
        unsafe { sys::cui_json_view_ex(self.ctx, id, j, &o) }
    }
    /// Renders markdown as native content. `.show()` reports clicked links
    /// and toggled task boxes.
    pub fn markdown<'a, 't>(&mut self, id: &str, text: &'t str) -> Widget<'_, 'f, MarkdownOpts<'a>, Markdown<'t>> {
        let id = self.s(id);
        Widget::new(self, MarkdownOpts::new(), Markdown { id, text })
    }
}

impl<'f> Container<'_, 'f, TreeOpts<'_>, Tree<'_>> {
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> Option<R> {
        let Container { ui, opts, c } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        if !unsafe { sys::cui_tree_begin(ctx, c.label, c.open, &o) } {
            return None;
        }
        let r = f(ui);
        unsafe { sys::cui_tree_end(ctx) };
        Some(r)
    }
}

impl<'s, O> Container<'_, '_, O, Table<'s>> {
    /// The sort the header shows, updated when a sortable header is clicked.
    pub fn sort(mut self, sort: &'s mut TableSort) -> Self {
        self.c.sort = Some(sort);
        self
    }
}

impl<'f> Container<'_, 'f, TableOpts<'_>, Table<'_>> {
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> Option<R> {
        let Container { ui, opts, c } = self;
        let (ctx, mut o) = (ui.ctx, opts.raw(ui.arena));
        // CUI keeps the sort pointer until cui_table_end.
        let cell = c.sort.as_ref().map(|s| Cell::new(sys::cui_table_sort { column: s.column, descending: s.descending }));
        if let Some(cell) = &cell {
            o.sort = cell.as_ptr();
        }
        let shown = unsafe { sys::cui_table_begin_ex(ctx, c.id, c.columns.as_ptr(), c.columns.len() as i32, &o) };
        let r = shown.then(|| {
            let r = f(ui);
            unsafe { sys::cui_table_end(ctx) };
            r
        });
        if let (Some(s), Some(cell)) = (c.sort, cell) {
            let v = cell.get();
            *s = TableSort { column: v.column, descending: v.descending };
        }
        r
    }
}

impl Run<ProgressOpts<'_>> for Progress {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &ProgressOpts<'_>) {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_progress_ex(ui.ctx, self.0, &o) }
    }
}

impl Run<BadgeOpts<'_>> for Badge {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &BadgeOpts<'_>) {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_badge_ex(ui.ctx, self.0, &o) }
    }
}

impl Run<AvatarOpts<'_>> for Avatar {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &AvatarOpts<'_>) {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_avatar_ex(ui.ctx, self.0, &o) }
    }
}

impl Run<EmptyOpts<'_>> for EmptyState {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &EmptyOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_empty_state_ex(ui.ctx, self.0, &o) }
    }
}

impl Run<MetricOpts<'_>> for Metric {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &MetricOpts<'_>) {
        let mut o = opts.raw(ui.arena);
        if let Some(spark) = opts.spark {
            o.spark = spark.as_ptr();
            o.spark_count = spark.len() as i32;
        }
        unsafe { sys::cui_metric_ex(ui.ctx, self.label, self.value, &o) }
    }
}

impl Run<TreeOpts<'_>> for TreeLeaf {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &TreeOpts<'_>) -> bool {
        let o = opts.leaf(true).raw(ui.arena);
        if unsafe { sys::cui_tree_begin(ui.ctx, self.0, core::ptr::null_mut(), &o) } {
            unsafe { sys::cui_tree_end(ui.ctx) };
        }
        unsafe { sys::cui_tree_clicked(ui.ctx) }
    }
}

impl Run<ListItemOpts<'_>> for ListItem {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &ListItemOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_list_item_ex(ui.ctx, self.0, &o) }
    }
}

fn chart_opts(ui: &mut Ui<'_>, opts: &ChartOpts<'_>) -> sys::cui_chart_opts {
    let mut o = opts.raw(ui.arena);
    if let Some(labels) = opts.labels {
        o.labels = ui.strs(labels);
        o.label_count = labels.len() as i32;
    }
    o
}

impl Run<ChartOpts<'_>> for Chart<'_> {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &ChartOpts<'_>) {
        let o = chart_opts(ui, opts);
        let series: Vec<sys::cui_series> = self
            .series
            .iter()
            .map(|s| sys::cui_series {
                name: ui.arena.str(s.name),
                values: s.values.as_ptr(),
                count: s.values.len() as i32,
                color: s.color.into(),
            })
            .collect();
        let f = match self.kind {
            ChartKind::Line => sys::cui_line_chart_ex,
            ChartKind::Bar => sys::cui_bar_chart_ex,
        };
        unsafe { f(ui.ctx, self.id, series.as_ptr(), series.len() as i32, &o) }
    }
}

impl Run<ChartOpts<'_>> for Donut<'_> {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &ChartOpts<'_>) {
        let o = chart_opts(ui, opts);
        unsafe { sys::cui_donut_chart_ex(ui.ctx, self.id, self.values.as_ptr(), self.labels, self.values.len() as i32, &o) }
    }
}

impl Run<ImageOpts<'_>> for Image {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &ImageOpts<'_>) {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_image_ex(ui.ctx, self.0 .0, &o) }
    }
}

impl Run<CodeOpts<'_>> for CodeBlock {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &CodeOpts<'_>) {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_code_block_ex(ui.ctx, self.id, self.code, &o) }
    }
}

impl Run<MarkdownOpts<'_>> for Markdown<'_> {
    type Output = MarkdownResponse;
    fn run(self, ui: &mut Ui<'_>, opts: &MarkdownOpts<'_>) -> MarkdownResponse {
        let mut o = opts.raw(ui.arena);
        // A length lets CUI read the text in place, without a copy.
        let t = if self.text.is_empty() { ui.s("") } else { self.text.as_ptr().cast() };
        o.length = self.text.len() as i32;
        let r = unsafe { sys::cui_markdown_ex(ui.ctx, self.id, t, &o) };
        MarkdownResponse {
            link: unsafe { cstr(r.link) },
            hovered_link: unsafe { cstr(r.hovered_link) },
            task: usize::try_from(r.task_offset).ok().map(|off| (off, r.task_checked)),
            height: r.height,
        }
    }
}

bool_result!(clicked, "Draws it; true when clicked.":
    EmptyState => EmptyOpts, TreeLeaf => TreeOpts, ListItem => ListItemOpts);
