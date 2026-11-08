use crate::builder::{IntoOpt, Run, TextTarget, Widget};
use crate::opts::*;
use crate::sys;
use crate::types::*;
use crate::Ui;
use core::ffi::c_char;
use core::fmt;

type CStr = *const c_char;
type CStrs = *const *const c_char;

/// Body text (`cui_text`).
pub struct Text(CStr);

/// A button (`cui_button`).
pub struct Button(CStr);

/// A button with only an icon (`cui_icon_button`).
pub struct IconButton(CStr);

/// A segmented control (`cui_segmented`).
pub struct Segmented<'v> {
    id: CStr,
    items: CStrs,
    count: i32,
    index: &'v mut usize,
}

/// A checkbox or switch.
pub struct Check<'v> {
    label: CStr,
    value: &'v mut bool,
    switch: bool,
}

/// A radio button for one option.
pub struct Radio<'v, T> {
    label: CStr,
    value: &'v mut T,
    option: T,
}

/// A slider (`cui_slider`).
pub struct Slider<'v> {
    label: CStr,
    value: &'v mut f32,
    min: f32,
    max: f32,
}

/// A toggleable chip (`cui_chip`).
pub struct Chip<'v> {
    label: CStr,
    selected: &'v mut bool,
}

#[derive(Clone, Copy)]
enum FieldKind {
    Input,
    TextArea,
    Search,
}

/// A text field, text area or search field.
pub struct TextField<'v, T: ?Sized> {
    id: CStr,
    text: &'v mut T,
    kind: FieldKind,
}

/// Tags stored comma separated (`cui_tag_input`).
pub struct TagInput<'v, T: ?Sized> {
    id: CStr,
    text: &'v mut T,
}

/// A number field with steppers (`cui_number`).
pub struct Number<'v> {
    id: CStr,
    value: &'v mut f64,
    min: f64,
    max: f64,
}

/// A dropdown (`cui_select`).
pub struct Select<'v> {
    id: CStr,
    items: CStrs,
    count: i32,
    index: &'v mut Option<usize>,
}

/// A date field with a calendar popup (`cui_date_picker`).
pub struct DatePicker<'v> {
    id: CStr,
    value: &'v mut Date,
}

/// Tabs (`cui_tabs`).
pub struct Tabs<'v> {
    id: CStr,
    items: CStrs,
    count: i32,
    index: &'v mut usize,
}

/// A sidebar item (`cui_nav_item`).
pub struct NavItem(CStr);

/// Text, buttons, selection controls, text input, pickers and navigation.
impl<'f> Ui<'f> {
    // ---- text ----

    /// Body text that wraps to the available width. For formatted text use
    /// [`text!`](crate::text).
    pub fn text<'a>(&mut self, text: &str) -> Widget<'_, 'f, TextOpts<'a>, Text> {
        let t = self.s(text);
        Widget::new(self, TextOpts::new(), Text(t))
    }
    /// Formatted text without an intermediate `String`; [`text!`](crate::text) calls this.
    pub fn text_fmt<'a>(&mut self, args: fmt::Arguments<'_>) -> Widget<'_, 'f, TextOpts<'a>, Text> {
        let t = self.arena.fmt(args);
        Widget::new(self, TextOpts::new(), Text(t))
    }
    /// Level 1..=4.
    pub fn heading(&mut self, text: &str, level: i32) {
        let t = self.s(text);
        unsafe { sys::cui_heading(self.ctx, t, level) }
    }
    /// Medium weight text.
    pub fn label(&mut self, text: &str) {
        let t = self.s(text);
        unsafe { sys::cui_label(self.ctx, t) }
    }
    /// Text in the secondary color.
    pub fn secondary(&mut self, text: &str) {
        let t = self.s(text);
        unsafe { sys::cui_secondary(self.ctx, t) }
    }
    /// Small, muted text.
    pub fn muted(&mut self, text: &str) {
        let t = self.s(text);
        unsafe { sys::cui_muted(self.ctx, t) }
    }
    /// Inline code.
    pub fn code(&mut self, text: &str) {
        let t = self.s(text);
        unsafe { sys::cui_code(self.ctx, t) }
    }
    /// A keycap, e.g. "Ctrl K".
    pub fn kbd(&mut self, keys: &str) {
        let t = self.s(keys);
        unsafe { sys::cui_kbd(self.ctx, t) }
    }
    /// Returns true when clicked.
    pub fn link(&mut self, text: &str) -> bool {
        let t = self.s(text);
        unsafe { sys::cui_link(self.ctx, t) }
    }
    /// An icon from [`crate::icons`]; a zero color means the foreground.
    pub fn icon(&mut self, icon: &str, size: impl IntoOpt<f32>, color: impl IntoOpt<Color>) {
        let t = self.s(icon);
        unsafe { sys::cui_icon(self.ctx, t, size.into_opt(), color.into_opt().into()) }
    }

    // ---- buttons ----

    /// A button. `.clicked()` draws it and tells whether it was clicked.
    pub fn button<'a>(&mut self, label: &str) -> Widget<'_, 'f, ButtonOpts<'a>, Button> {
        let l = self.s(label);
        Widget::new(self, ButtonOpts::new(), Button(l))
    }
    /// A square button with only an icon; give it a `.tooltip(...)` so it has a name.
    pub fn icon_button<'a>(&mut self, icon: &str) -> Widget<'_, 'f, ButtonOpts<'a>, IconButton> {
        let i = self.s(icon);
        Widget::new(self, ButtonOpts::new(), IconButton(i))
    }
    /// A row of options with a gliding highlight.
    pub fn segmented<'a, 'v>(
        &mut self,
        id: &str,
        items: &[&str],
        index: &'v mut usize,
    ) -> Widget<'_, 'f, SegmentedOpts<'a>, Segmented<'v>> {
        let (id, it) = (self.s(id), self.strs(items));
        Widget::new(self, SegmentedOpts::new(), Segmented { id, items: it, count: items.len() as i32, index })
    }
    /// Connected buttons: put buttons inside.
    pub fn button_group<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        unsafe { sys::cui_button_group_begin(self.ctx) };
        let r = f(self);
        unsafe { sys::cui_button_group_end(self.ctx) };
        r
    }
    /// Copies `text` to the clipboard and confirms with a check.
    pub fn copy_button(&mut self, id: &str, text: &str) -> bool {
        let (id, t) = (self.s(id), self.s(text));
        unsafe { sys::cui_copy_button(self.ctx, id, t) }
    }
    /// Sun and moon button that toggles the theme.
    pub fn theme_toggle(&mut self) -> bool {
        unsafe { sys::cui_theme_toggle(self.ctx) }
    }

    // ---- selection controls ----

    pub fn checkbox<'a, 'v>(&mut self, label: &str, value: &'v mut bool) -> Widget<'_, 'f, CheckOpts<'a>, Check<'v>> {
        let l = self.s(label);
        Widget::new(self, CheckOpts::new(), Check { label: l, value, switch: false })
    }
    pub fn switch<'a, 'v>(&mut self, label: &str, value: &'v mut bool) -> Widget<'_, 'f, CheckOpts<'a>, Check<'v>> {
        let l = self.s(label);
        Widget::new(self, CheckOpts::new(), Check { label: l, value, switch: true })
    }
    /// One radio button: selected when `*value == option`, and clicking sets
    /// `*value = option`. Works with any comparable value, an enum say.
    pub fn radio<'a, 'v, T: PartialEq + Copy>(
        &mut self,
        label: &str,
        value: &'v mut T,
        option: T,
    ) -> Widget<'_, 'f, CheckOpts<'a>, Radio<'v, T>> {
        let l = self.s(label);
        Widget::new(self, CheckOpts::new(), Radio { label: l, value, option })
    }
    pub fn radio_group(&mut self, id: &str, items: &[&str], index: &mut usize) -> bool {
        let (id, it) = (self.s(id), self.strs(items));
        with_index(index, |i| unsafe { sys::cui_radio_group(self.ctx, id, it, items.len() as i32, i) })
    }
    pub fn slider<'a, 'v>(
        &mut self,
        label: &str,
        value: &'v mut f32,
        min: impl IntoOpt<f32>,
        max: impl IntoOpt<f32>,
    ) -> Widget<'_, 'f, SliderOpts<'a>, Slider<'v>> {
        let l = self.s(label);
        Widget::new(self, SliderOpts::new(), Slider { label: l, value, min: min.into_opt(), max: max.into_opt() })
    }
    pub fn range_slider(&mut self, label: &str, lo: &mut f32, hi: &mut f32, min: impl IntoOpt<f32>, max: impl IntoOpt<f32>) -> bool {
        let l = self.s(label);
        unsafe { sys::cui_range_slider(self.ctx, l, lo, hi, min.into_opt(), max.into_opt()) }
    }
    /// Stars, 0..=max.
    pub fn rating(&mut self, id: &str, value: &mut i32, max: i32) -> bool {
        let id = self.s(id);
        unsafe { sys::cui_rating(self.ctx, id, value, max) }
    }
    pub fn chip<'a, 'v>(&mut self, label: &str, selected: &'v mut bool) -> Widget<'_, 'f, ChipOpts<'a>, Chip<'v>> {
        let l = self.s(label);
        Widget::new(self, ChipOpts::new(), Chip { label: l, selected })
    }

    // ---- text input ----

    /// Single line text field, editing a `String` or a [`crate::TextBuffer`].
    pub fn input<'a, 'v, T: TextTarget + ?Sized>(&mut self, id: &str, text: &'v mut T) -> Widget<'_, 'f, InputOpts<'a>, TextField<'v, T>> {
        self.field(id, text, FieldKind::Input)
    }
    /// Multi-line text area with word wrap.
    pub fn textarea<'a, 'v, T: TextTarget + ?Sized>(&mut self, id: &str, text: &'v mut T) -> Widget<'_, 'f, InputOpts<'a>, TextField<'v, T>> {
        self.field(id, text, FieldKind::TextArea)
    }
    /// Search field with a search icon and a clear button.
    pub fn search<'a, 'v, T: TextTarget + ?Sized>(&mut self, id: &str, text: &'v mut T) -> Widget<'_, 'f, InputOpts<'a>, TextField<'v, T>> {
        self.field(id, text, FieldKind::Search)
    }
    fn field<'a, 'v, T: TextTarget + ?Sized>(&mut self, id: &str, text: &'v mut T, kind: FieldKind) -> Widget<'_, 'f, InputOpts<'a>, TextField<'v, T>> {
        let id = self.s(id);
        Widget::new(self, InputOpts::new(), TextField { id, text, kind })
    }
    /// Tags stored comma separated in `text`.
    pub fn tag_input<'a, 'v, T: TextTarget + ?Sized>(&mut self, id: &str, text: &'v mut T) -> Widget<'_, 'f, TagOpts<'a>, TagInput<'v, T>> {
        let id = self.s(id);
        Widget::new(self, TagOpts::new(), TagInput { id, text })
    }
    /// Number field with steppers.
    pub fn number<'a, 'v>(
        &mut self,
        id: &str,
        value: &'v mut f64,
        min: impl IntoOpt<f64>,
        max: impl IntoOpt<f64>,
    ) -> Widget<'_, 'f, NumberOpts<'a>, Number<'v>> {
        let id = self.s(id);
        Widget::new(self, NumberOpts::new(), Number { id, value, min: min.into_opt(), max: max.into_opt() })
    }

    // ---- pickers ----

    /// Dropdown. `None` shows the placeholder.
    pub fn select<'a, 'v>(&mut self, id: &str, items: &[&str], index: &'v mut Option<usize>) -> Widget<'_, 'f, SelectOpts<'a>, Select<'v>> {
        let (id, it) = (self.s(id), self.strs(items));
        Widget::new(self, SelectOpts::new(), Select { id, items: it, count: items.len() as i32, index })
    }
    pub fn calendar(&mut self, id: &str, value: &mut Date) -> bool {
        let id = self.s(id);
        let mut d = (*value).into();
        let changed = unsafe { sys::cui_calendar(self.ctx, id, &mut d) };
        *value = d.into();
        changed
    }
    pub fn date_picker<'a, 'v>(&mut self, id: &str, value: &'v mut Date) -> Widget<'_, 'f, DateOpts<'a>, DatePicker<'v>> {
        let id = self.s(id);
        Widget::new(self, DateOpts::new(), DatePicker { id, value })
    }
    pub fn color_picker(&mut self, id: &str, value: &mut Color) -> bool {
        let id = self.s(id);
        let mut c = (*value).into();
        let changed = unsafe { sys::cui_color_picker(self.ctx, id, &mut c) };
        *value = c.into();
        changed
    }

    // ---- navigation ----

    pub fn tabs<'a, 'v>(&mut self, id: &str, items: &[&str], index: &'v mut usize) -> Widget<'_, 'f, TabsOpts<'a>, Tabs<'v>> {
        let (id, it) = (self.s(id), self.strs(items));
        Widget::new(self, TabsOpts::new(), Tabs { id, items: it, count: items.len() as i32, index })
    }
    /// Sidebar item. `.clicked()` tells whether it was clicked.
    pub fn nav_item<'a>(&mut self, label: &str) -> Widget<'_, 'f, NavOpts<'a>, NavItem> {
        let l = self.s(label);
        Widget::new(self, NavOpts::new(), NavItem(l))
    }
    pub fn nav_section(&mut self, title: &str) {
        let t = self.s(title);
        unsafe { sys::cui_nav_section(self.ctx, t) }
    }
    /// Returns the clicked segment.
    pub fn breadcrumb(&mut self, items: &[&str]) -> Option<usize> {
        let it = self.strs(items);
        usize::try_from(unsafe { sys::cui_breadcrumb(self.ctx, it, items.len() as i32) }).ok()
    }
    /// `page` is 0 based.
    pub fn pagination(&mut self, id: &str, page: &mut usize, page_count: usize) -> bool {
        let id = self.s(id);
        with_index(page, |p| unsafe { sys::cui_pagination(self.ctx, id, p, page_count as i32) })
    }
    pub fn stepper(&mut self, steps: &[&str], current: usize) {
        let st = self.strs(steps);
        unsafe { sys::cui_stepper(self.ctx, st, steps.len() as i32, current as i32) }
    }
}

/// Runs a C call that edits an `int` index through a `usize`.
pub(crate) fn with_index(index: &mut usize, f: impl FnOnce(&mut i32) -> bool) -> bool {
    let mut i = *index as i32;
    let r = f(&mut i);
    if let Ok(v) = usize::try_from(i) {
        *index = v;
    }
    r
}

impl Run<TextOpts<'_>> for Text {
    type Output = ();
    fn run(self, ui: &mut Ui<'_>, opts: &TextOpts<'_>) {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_text_ex(ui.ctx, self.0, &o) }
    }
}

impl Run<ButtonOpts<'_>> for Button {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &ButtonOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_button_ex(ui.ctx, self.0, &o) }
    }
}

impl Run<ButtonOpts<'_>> for IconButton {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &ButtonOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_icon_button_ex(ui.ctx, self.0, &o) }
    }
}

impl Run<SegmentedOpts<'_>> for Segmented<'_> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &SegmentedOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        with_index(self.index, |i| unsafe { sys::cui_segmented_ex(ui.ctx, self.id, self.items, self.count, i, &o) })
    }
}

impl Run<CheckOpts<'_>> for Check<'_> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &CheckOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        if self.switch {
            unsafe { sys::cui_switch_ex(ui.ctx, self.label, self.value, &o) }
        } else {
            unsafe { sys::cui_checkbox_ex(ui.ctx, self.label, self.value, &o) }
        }
    }
}

impl<T: PartialEq + Copy> Run<CheckOpts<'_>> for Radio<'_, T> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &CheckOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        let mut cur = (*self.value == self.option) as i32;
        let clicked = unsafe { sys::cui_radio_ex(ui.ctx, self.label, &mut cur, 1, &o) };
        if cur == 1 {
            *self.value = self.option;
        }
        clicked
    }
}

impl Run<SliderOpts<'_>> for Slider<'_> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &SliderOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_slider_ex(ui.ctx, self.label, self.value, self.min, self.max, &o) }
    }
}

impl Run<ChipOpts<'_>> for Chip<'_> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &ChipOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_chip_ex(ui.ctx, self.label, self.selected, &o) }
    }
}

impl<T: TextTarget + ?Sized> Run<InputOpts<'_>> for TextField<'_, T> {
    type Output = InputEvents;
    fn run(self, ui: &mut Ui<'_>, opts: &InputOpts<'_>) -> InputEvents {
        let o = opts.raw(ui.arena);
        let f = match self.kind {
            FieldKind::Input => sys::cui_input_ex,
            FieldKind::TextArea => sys::cui_textarea_ex,
            FieldKind::Search => sys::cui_search_ex,
        };
        let (ctx, id) = (ui.ctx, self.id);
        InputEvents(self.text.edit(&mut ui.arena.text, |b| unsafe { f(ctx, id, b.as_mut_ptr().cast(), b.len(), &o) }))
    }
}

impl<T: TextTarget + ?Sized> Run<TagOpts<'_>> for TagInput<'_, T> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &TagOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        let (ctx, id) = (ui.ctx, self.id);
        self.text.edit(&mut ui.arena.text, |b| unsafe { sys::cui_tag_input_ex(ctx, id, b.as_mut_ptr().cast(), b.len(), &o) })
    }
}

impl Run<NumberOpts<'_>> for Number<'_> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &NumberOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_number_ex(ui.ctx, self.id, self.value, self.min, self.max, &o) }
    }
}

impl Run<SelectOpts<'_>> for Select<'_> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &SelectOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        let mut i = self.index.map_or(-1, |i| i as i32);
        let changed = unsafe { sys::cui_select_ex(ui.ctx, self.id, self.items, self.count, &mut i, &o) };
        *self.index = usize::try_from(i).ok();
        changed
    }
}

impl Run<DateOpts<'_>> for DatePicker<'_> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &DateOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        let mut d = (*self.value).into();
        let changed = unsafe { sys::cui_date_picker_ex(ui.ctx, self.id, &mut d, &o) };
        *self.value = d.into();
        changed
    }
}

impl Run<TabsOpts<'_>> for Tabs<'_> {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &TabsOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        with_index(self.index, |i| unsafe { sys::cui_tabs_ex(ui.ctx, self.id, self.items, self.count, i, &o) })
    }
}

impl Run<NavOpts<'_>> for NavItem {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &NavOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_nav_item_ex(ui.ctx, self.0, &o) }
    }
}

// ---- result shortcuts ----

/// `.clicked()` / `.changed()` on widgets whose result is a bool.
macro_rules! bool_result {
    ($method:ident, $doc:literal: $($c:ty => $o:ident),* $(,)?) => {$(
        impl<'a> Widget<'_, '_, $o<'a>, $c> {
            #[doc = $doc]
            pub fn $method(self) -> bool {
                self.show()
            }
        }
    )*};
}
pub(crate) use bool_result;

bool_result!(clicked, "Draws it; true once, when clicked.":
    Button => ButtonOpts, IconButton => ButtonOpts, NavItem => NavOpts);
bool_result!(changed, "Draws it; true when the user changed the value.":
    Segmented<'_> => SegmentedOpts, Check<'_> => CheckOpts, Slider<'_> => SliderOpts, Chip<'_> => ChipOpts,
    Number<'_> => NumberOpts, Select<'_> => SelectOpts, DatePicker<'_> => DateOpts, Tabs<'_> => TabsOpts,
    TagInput<'_, String> => TagOpts, TagInput<'_, crate::TextBuffer> => TagOpts);

impl<'a, T: PartialEq + Copy> Widget<'_, '_, CheckOpts<'a>, Radio<'_, T>> {
    /// Draws it; true when clicked.
    pub fn clicked(self) -> bool {
        self.show()
    }
}

impl<'a, C: Run<InputOpts<'a>, Output = InputEvents>> Widget<'_, '_, InputOpts<'a>, C> {
    /// Draws it; true when the text changed.
    pub fn changed(self) -> bool {
        self.show().changed()
    }
    /// Draws it; true when Enter was pressed.
    pub fn submitted(self) -> bool {
        self.show().submitted()
    }
}

impl<'a, C: Run<ButtonOpts<'a>>> Widget<'_, '_, ButtonOpts<'a>, C> {
    /// The one main action.
    pub fn primary(self) -> Self {
        self.variant(ButtonVariant::Primary)
    }
    /// Neutral filled.
    pub fn secondary(self) -> Self {
        self.variant(ButtonVariant::Secondary)
    }
    pub fn outline(self) -> Self {
        self.variant(ButtonVariant::Outline)
    }
    pub fn ghost(self) -> Self {
        self.variant(ButtonVariant::Ghost)
    }
    pub fn danger(self) -> Self {
        self.variant(ButtonVariant::Danger)
    }
}
