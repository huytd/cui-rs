use crate::opts::*;
use crate::sys;
use crate::types::*;
use crate::ui::cstr;
use crate::Ui;
use core::cell::Cell;
use core::ops::Range;

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
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
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
    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
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

/// Feedback, data display, tables, charts, images, code and markdown.
impl<'f> Ui<'f> {
    // ---- feedback ----

    /// `value` is 0..1.
    pub fn progress(&mut self, value: f32) {
        self.progress_with(value, ProgressOpts::new())
    }
    pub fn progress_with(&mut self, value: f32, opts: ProgressOpts) {
        let o = opts.raw(self.arena);
        unsafe { sys::cui_progress_ex(self.ctx, value, &o) }
    }
    /// Circular progress with the value in the middle.
    pub fn progress_ring(&mut self, label: &str, value: f32, size: f32) {
        let l = self.s(label);
        unsafe { sys::cui_progress_ring(self.ctx, l, value, size) }
    }
    pub fn spinner(&mut self, size: f32) {
        unsafe { sys::cui_spinner(self.ctx, size) }
    }
    /// A loading placeholder.
    pub fn skeleton(&mut self, width: impl Into<Size>, height: f32) {
        unsafe { sys::cui_skeleton(self.ctx, width.into().into(), height) }
    }
    pub fn badge(&mut self, text: &str) {
        self.badge_with(text, BadgeOpts::new())
    }
    pub fn badge_with(&mut self, text: &str, opts: BadgeOpts) {
        let t = self.s(text);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_badge_ex(self.ctx, t, &o) }
    }
    /// Initials (or an image) in a circle.
    pub fn avatar(&mut self, name: &str) {
        self.avatar_with(name, AvatarOpts::new())
    }
    pub fn avatar_with(&mut self, name: &str, opts: AvatarOpts) {
        let n = self.s(name);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_avatar_ex(self.ctx, n, &o) }
    }
    pub fn avatar_group(&mut self, names: &[&str], max_visible: usize, size: f32) {
        let n = self.strs(names);
        unsafe { sys::cui_avatar_group(self.ctx, n, names.len() as i32, max_visible as i32, size) }
    }
    /// Empty state. Returns true when its action button is clicked.
    pub fn empty_state(&mut self, title: &str) -> bool {
        self.empty_state_with(title, EmptyOpts::new())
    }
    pub fn empty_state_with(&mut self, title: &str, opts: EmptyOpts) -> bool {
        let t = self.s(title);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_empty_state_ex(self.ctx, t, &o) }
    }
    /// A KPI tile: a label, a big value, an optional delta and sparkline.
    pub fn metric(&mut self, label: &str, value: &str) {
        self.metric_with(label, value, MetricOpts::new())
    }
    pub fn metric_with(&mut self, label: &str, value: &str, opts: MetricOpts) {
        let l = self.s(label);
        let v = self.s(value);
        let mut o = opts.raw(self.arena);
        if let Some(spark) = opts.spark {
            o.spark = spark.as_ptr();
            o.spark_count = spark.len() as i32;
        }
        unsafe { sys::cui_metric_ex(self.ctx, l, v, &o) }
    }
    pub fn meter(&mut self, label: &str, used: f32, total: f32, unit: &str) {
        let l = self.s(label);
        let u = self.s(unit);
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
    /// A tree node; children go in the closure. Check [`Ui::tree_clicked`]
    /// right after for selection.
    pub fn tree<R>(&mut self, label: &str, open: &mut bool, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        self.tree_with(label, open, TreeOpts::new(), f)
    }
    pub fn tree_with<R>(&mut self, label: &str, open: &mut bool, opts: TreeOpts, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        if !unsafe { sys::cui_tree_begin(self.ctx, l, open, &o) } {
            return None;
        }
        let r = f(self);
        unsafe { sys::cui_tree_end(self.ctx) };
        Some(r)
    }
    /// A tree node without children. Returns true when clicked.
    pub fn tree_leaf(&mut self, label: &str, opts: TreeOpts) -> bool {
        let l = self.s(label);
        let o = opts.leaf(true).raw(self.arena);
        if unsafe { sys::cui_tree_begin(self.ctx, l, core::ptr::null_mut(), &o) } {
            unsafe { sys::cui_tree_end(self.ctx) };
        }
        unsafe { sys::cui_tree_clicked(self.ctx) }
    }
    /// The node made by the last [`Ui::tree`] was clicked.
    pub fn tree_clicked(&mut self) -> bool {
        unsafe { sys::cui_tree_clicked(self.ctx) }
    }
    /// A selectable row with optional icon, subtitle and trailing text.
    pub fn list_item(&mut self, title: &str) -> bool {
        self.list_item_with(title, ListItemOpts::new())
    }
    pub fn list_item_with(&mut self, title: &str, opts: ListItemOpts) -> bool {
        let t = self.s(title);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_list_item_ex(self.ctx, t, &o) }
    }
    /// Only the visible rows are built: the closure gets their index range.
    pub fn virtual_list<R>(
        &mut self,
        id: &str,
        count: usize,
        row_height: f32,
        height: impl Into<Size>,
        f: impl FnOnce(&mut Self, Range<usize>) -> R,
    ) -> R {
        let id = self.s(id);
        let (mut first, mut last) = (0i32, -1i32);
        unsafe {
            sys::cui_virtual_list_begin(self.ctx, id, count as i32, row_height, height.into().into(), &mut first, &mut last)
        };
        let range = first.max(0) as usize..(last + 1).max(first).max(0) as usize;
        let r = f(self, range);
        unsafe { sys::cui_virtual_list_end(self.ctx) };
        r
    }

    /// A table. Inside, start rows with [`Ui::table_row`] and cells with
    /// [`Ui::table_cell`] or [`Ui::table_cell_text`].
    pub fn table<R>(&mut self, id: &str, columns: &[Column], f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        self.table_with(id, columns, TableOpts::new(), None, f)
    }
    pub fn table_with<R>(
        &mut self,
        id: &str,
        columns: &[Column],
        opts: TableOpts,
        sort: Option<&mut TableSort>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> Option<R> {
        let id = self.s(id);
        let cols: Vec<sys::cui_table_column> = columns
            .iter()
            .map(|c| sys::cui_table_column {
                title: self.arena.str(c.title),
                width: if c.width == Size::Fit { Size::FILL.into() } else { c.width.into() },
                align: c.align.to_raw(),
                sortable: c.sortable,
            })
            .collect();
        let mut o = opts.raw(self.arena);
        // CUI keeps the sort pointer until cui_table_end.
        let sort_cell = sort.as_ref().map(|s| Cell::new(sys::cui_table_sort { column: s.column, descending: s.descending }));
        if let Some(c) = &sort_cell {
            o.sort = c.as_ptr();
        }
        let shown = unsafe { sys::cui_table_begin_ex(self.ctx, id, cols.as_ptr(), cols.len() as i32, &o) };
        let r = if shown {
            let r = f(self);
            unsafe { sys::cui_table_end(self.ctx) };
            Some(r)
        } else {
            None
        };
        if let (Some(s), Some(c)) = (sort, sort_cell) {
            let v = c.get();
            *s = TableSort { column: v.column, descending: v.descending };
        }
        r
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

    pub fn line_chart(&mut self, id: &str, series: &[Series]) {
        self.line_chart_with(id, series, ChartOpts::new())
    }
    pub fn line_chart_with(&mut self, id: &str, series: &[Series], opts: ChartOpts) {
        let id = self.s(id);
        let s = self.series(series);
        let o = self.chart_opts(&opts);
        unsafe { sys::cui_line_chart_ex(self.ctx, id, s.as_ptr(), s.len() as i32, &o) }
    }
    pub fn bar_chart(&mut self, id: &str, series: &[Series]) {
        self.bar_chart_with(id, series, ChartOpts::new())
    }
    pub fn bar_chart_with(&mut self, id: &str, series: &[Series], opts: ChartOpts) {
        let id = self.s(id);
        let s = self.series(series);
        let o = self.chart_opts(&opts);
        unsafe { sys::cui_bar_chart_ex(self.ctx, id, s.as_ptr(), s.len() as i32, &o) }
    }
    /// `values` and `labels` have one entry per slice.
    pub fn donut_chart(&mut self, id: &str, values: &[f32], labels: &[&str]) {
        self.donut_chart_with(id, values, labels, ChartOpts::new())
    }
    pub fn donut_chart_with(&mut self, id: &str, values: &[f32], labels: &[&str], opts: ChartOpts) {
        let n = values.len().min(labels.len());
        let id = self.s(id);
        let l = self.strs(&labels[..n]);
        let o = self.chart_opts(&opts);
        unsafe { sys::cui_donut_chart_ex(self.ctx, id, values.as_ptr(), l, n as i32, &o) }
    }
    pub fn sparkline(&mut self, values: &[f32], width: impl Into<Size>, height: f32, color: Color) {
        unsafe {
            sys::cui_sparkline(self.ctx, values.as_ptr(), values.len() as i32, width.into().into(), height, color.into())
        }
    }
    pub fn gauge(&mut self, label: &str, value: f32, size: f32) {
        let l = self.s(label);
        unsafe { sys::cui_gauge(self.ctx, l, value, size) }
    }

    fn series(&mut self, series: &[Series]) -> Vec<sys::cui_series> {
        series
            .iter()
            .map(|s| sys::cui_series {
                name: self.arena.str(s.name),
                values: s.values.as_ptr(),
                count: s.values.len() as i32,
                color: s.color.into(),
            })
            .collect()
    }

    fn chart_opts(&mut self, opts: &ChartOpts) -> sys::cui_chart_opts {
        let mut o = opts.raw(self.arena);
        if let Some(labels) = opts.labels {
            o.labels = self.strs(labels);
            o.label_count = labels.len() as i32;
        }
        o
    }

    // ---- images ----

    pub fn image(&mut self, tex: Texture) {
        self.image_with(tex, ImageOpts::new())
    }
    pub fn image_with(&mut self, tex: Texture, opts: ImageOpts) {
        let o = opts.raw(self.arena);
        unsafe { sys::cui_image_ex(self.ctx, tex.0, &o) }
    }

    // ---- code and markdown ----

    /// Code with syntax highlighting, line numbers and a copy button.
    pub fn code_block(&mut self, id: &str, code: &str) {
        self.code_block_with(id, code, CodeOpts::new())
    }
    pub fn code_block_with(&mut self, id: &str, code: &str, opts: CodeOpts) {
        let id = self.s(id);
        let c = self.s(code);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_code_block_ex(self.ctx, id, c, &o) }
    }
    /// A collapsible JSON tree. Returns false when `json` is not valid.
    pub fn json_view(&mut self, id: &str, json: &str) -> bool {
        let id = self.s(id);
        let j = self.s(json);
        let o = sys::cui_json_opts::default();
        unsafe { sys::cui_json_view_ex(self.ctx, id, j, &o) }
    }
    /// Renders markdown as native content.
    pub fn markdown(&mut self, id: &str, text: &str) -> MarkdownResponse {
        self.markdown_with(id, text, MarkdownOpts::new())
    }
    pub fn markdown_with(&mut self, id: &str, text: &str, opts: MarkdownOpts) -> MarkdownResponse {
        let id = self.s(id);
        let mut o = opts.raw(self.arena);
        // A length lets CUI read the text in place, without a copy.
        let t = if text.is_empty() { self.s("") } else { text.as_ptr().cast() };
        o.length = text.len() as i32;
        let r = unsafe { sys::cui_markdown_ex(self.ctx, id, t, &o) };
        MarkdownResponse {
            link: unsafe { cstr(r.link) },
            hovered_link: unsafe { cstr(r.hovered_link) },
            task: usize::try_from(r.task_offset).ok().map(|off| (off, r.task_checked)),
            height: r.height,
        }
    }
}
