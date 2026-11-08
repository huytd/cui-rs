use crate::opts::*;
use crate::sys;
use crate::types::*;
use crate::{TextBuffer, Ui};

/// Text, buttons, selection controls, text input, pickers and navigation.
impl<'f> Ui<'f> {
    // ---- text ----

    /// Body text that wraps to the available width.
    pub fn text(&mut self, text: &str) {
        self.text_with(text, TextOpts::new())
    }
    pub fn text_with(&mut self, text: &str, opts: TextOpts) {
        let t = self.s(text);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_text_ex(self.ctx, t, &o) }
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
    pub fn icon(&mut self, icon: &str, size: f32, color: Color) {
        let t = self.s(icon);
        unsafe { sys::cui_icon(self.ctx, t, size, color.into()) }
    }

    // ---- buttons ----

    /// Returns true once, when clicked.
    pub fn button(&mut self, label: &str) -> bool {
        self.button_with(label, ButtonOpts::new())
    }
    pub fn button_with(&mut self, label: &str, opts: ButtonOpts) -> bool {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_button_ex(self.ctx, l, &o) }
    }
    /// A square button with only an icon; the tooltip gives it a name.
    pub fn icon_button(&mut self, icon: &str, tooltip: &str) -> bool {
        self.icon_button_with(icon, ButtonOpts::new().tooltip(tooltip))
    }
    pub fn icon_button_with(&mut self, icon: &str, opts: ButtonOpts) -> bool {
        let i = self.s(icon);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_icon_button_ex(self.ctx, i, &o) }
    }
    /// A row of options with a gliding highlight. Returns true when changed.
    pub fn segmented(&mut self, id: &str, items: &[&str], index: &mut usize) -> bool {
        self.segmented_with(id, items, index, SegmentedOpts::new())
    }
    pub fn segmented_with(&mut self, id: &str, items: &[&str], index: &mut usize, opts: SegmentedOpts) -> bool {
        let id = self.s(id);
        let it = self.strs(items);
        let o = opts.raw(self.arena);
        with_index(index, |i| unsafe { sys::cui_segmented_ex(self.ctx, id, it, items.len() as i32, i, &o) })
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
        let id = self.s(id);
        let t = self.s(text);
        unsafe { sys::cui_copy_button(self.ctx, id, t) }
    }
    /// Sun and moon button that toggles the theme.
    pub fn theme_toggle(&mut self) -> bool {
        unsafe { sys::cui_theme_toggle(self.ctx) }
    }

    // ---- selection controls ----

    /// Returns true when toggled.
    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> bool {
        self.checkbox_with(label, value, CheckOpts::new())
    }
    pub fn checkbox_with(&mut self, label: &str, value: &mut bool, opts: CheckOpts) -> bool {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_checkbox_ex(self.ctx, l, value, &o) }
    }
    pub fn switch(&mut self, label: &str, value: &mut bool) -> bool {
        self.switch_with(label, value, CheckOpts::new())
    }
    pub fn switch_with(&mut self, label: &str, value: &mut bool, opts: CheckOpts) -> bool {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_switch_ex(self.ctx, l, value, &o) }
    }
    /// One radio button: selected when `*value == option`, and clicking sets
    /// `*value = option`. Works with any comparable value (an enum, say).
    pub fn radio<T: PartialEq + Copy>(&mut self, label: &str, value: &mut T, option: T) -> bool {
        self.radio_with(label, value, option, CheckOpts::new())
    }
    pub fn radio_with<T: PartialEq + Copy>(&mut self, label: &str, value: &mut T, option: T, opts: CheckOpts) -> bool {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        let mut cur = (*value == option) as i32;
        let clicked = unsafe { sys::cui_radio_ex(self.ctx, l, &mut cur, 1, &o) };
        if cur == 1 {
            *value = option;
        }
        clicked
    }
    pub fn radio_group(&mut self, id: &str, items: &[&str], index: &mut usize) -> bool {
        let id = self.s(id);
        let it = self.strs(items);
        with_index(index, |i| unsafe { sys::cui_radio_group(self.ctx, id, it, items.len() as i32, i) })
    }
    /// Returns true while the value changes.
    pub fn slider(&mut self, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
        self.slider_with(label, value, min, max, SliderOpts::new())
    }
    pub fn slider_with(&mut self, label: &str, value: &mut f32, min: f32, max: f32, opts: SliderOpts) -> bool {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_slider_ex(self.ctx, l, value, min, max, &o) }
    }
    pub fn range_slider(&mut self, label: &str, lo: &mut f32, hi: &mut f32, min: f32, max: f32) -> bool {
        let l = self.s(label);
        unsafe { sys::cui_range_slider(self.ctx, l, lo, hi, min, max) }
    }
    /// Stars, 0..=max.
    pub fn rating(&mut self, id: &str, value: &mut i32, max: i32) -> bool {
        let id = self.s(id);
        unsafe { sys::cui_rating(self.ctx, id, value, max) }
    }
    pub fn chip(&mut self, label: &str, selected: &mut bool) -> bool {
        self.chip_with(label, selected, ChipOpts::new())
    }
    pub fn chip_with(&mut self, label: &str, selected: &mut bool, opts: ChipOpts) -> bool {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_chip_ex(self.ctx, l, selected, &o) }
    }

    // ---- text input ----

    /// Single line text field.
    pub fn input(&mut self, id: &str, buf: &mut TextBuffer) -> InputEvents {
        self.input_with(id, buf, InputOpts::new())
    }
    pub fn input_with(&mut self, id: &str, buf: &mut TextBuffer, opts: InputOpts) -> InputEvents {
        let id = self.s(id);
        let o = opts.raw(self.arena);
        let b = buf.as_mut_bytes();
        InputEvents(unsafe { sys::cui_input_ex(self.ctx, id, b.as_mut_ptr().cast(), b.len(), &o) })
    }
    /// Multi-line text area with word wrap.
    pub fn textarea(&mut self, id: &str, buf: &mut TextBuffer) -> InputEvents {
        self.textarea_with(id, buf, InputOpts::new())
    }
    pub fn textarea_with(&mut self, id: &str, buf: &mut TextBuffer, opts: InputOpts) -> InputEvents {
        let id = self.s(id);
        let o = opts.raw(self.arena);
        let b = buf.as_mut_bytes();
        InputEvents(unsafe { sys::cui_textarea_ex(self.ctx, id, b.as_mut_ptr().cast(), b.len(), &o) })
    }
    /// Search field with a search icon and a clear button.
    pub fn search(&mut self, id: &str, buf: &mut TextBuffer) -> InputEvents {
        self.search_with(id, buf, InputOpts::new())
    }
    pub fn search_with(&mut self, id: &str, buf: &mut TextBuffer, opts: InputOpts) -> InputEvents {
        let id = self.s(id);
        let o = opts.raw(self.arena);
        let b = buf.as_mut_bytes();
        InputEvents(unsafe { sys::cui_search_ex(self.ctx, id, b.as_mut_ptr().cast(), b.len(), &o) })
    }
    /// Tags stored comma separated in `buf`.
    pub fn tag_input(&mut self, id: &str, buf: &mut TextBuffer, label: Option<&str>) -> bool {
        let id = self.s(id);
        let o = sys::cui_tag_opts { label: self.os(label), ..Default::default() };
        let b = buf.as_mut_bytes();
        unsafe { sys::cui_tag_input_ex(self.ctx, id, b.as_mut_ptr().cast(), b.len(), &o) }
    }
    /// Number field with steppers. Returns true when changed.
    pub fn number(&mut self, id: &str, value: &mut f64, min: f64, max: f64) -> bool {
        self.number_with(id, value, min, max, NumberOpts::new())
    }
    pub fn number_with(&mut self, id: &str, value: &mut f64, min: f64, max: f64, opts: NumberOpts) -> bool {
        let id = self.s(id);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_number_ex(self.ctx, id, value, min, max, &o) }
    }

    // ---- pickers ----

    /// Dropdown. `None` shows the placeholder. Returns true when changed.
    pub fn select(&mut self, id: &str, items: &[&str], index: &mut Option<usize>) -> bool {
        self.select_with(id, items, index, SelectOpts::new())
    }
    pub fn select_with(&mut self, id: &str, items: &[&str], index: &mut Option<usize>, opts: SelectOpts) -> bool {
        let id = self.s(id);
        let it = self.strs(items);
        let o = opts.raw(self.arena);
        let mut i = index.map_or(-1, |i| i as i32);
        let changed = unsafe { sys::cui_select_ex(self.ctx, id, it, items.len() as i32, &mut i, &o) };
        *index = usize::try_from(i).ok();
        changed
    }
    pub fn calendar(&mut self, id: &str, value: &mut Date) -> bool {
        let id = self.s(id);
        let mut d = (*value).into();
        let changed = unsafe { sys::cui_calendar(self.ctx, id, &mut d) };
        *value = d.into();
        changed
    }
    pub fn date_picker(&mut self, id: &str, value: &mut Date) -> bool {
        self.date_picker_with(id, value, DateOpts::new())
    }
    pub fn date_picker_with(&mut self, id: &str, value: &mut Date, opts: DateOpts) -> bool {
        let id = self.s(id);
        let o = opts.raw(self.arena);
        let mut d = (*value).into();
        let changed = unsafe { sys::cui_date_picker_ex(self.ctx, id, &mut d, &o) };
        *value = d.into();
        changed
    }
    pub fn color_picker(&mut self, id: &str, value: &mut Color) -> bool {
        let id = self.s(id);
        let mut c = (*value).into();
        let changed = unsafe { sys::cui_color_picker(self.ctx, id, &mut c) };
        *value = c.into();
        changed
    }

    // ---- navigation ----

    pub fn tabs(&mut self, id: &str, items: &[&str], index: &mut usize) -> bool {
        self.tabs_with(id, items, index, TabsOpts::new())
    }
    pub fn tabs_with(&mut self, id: &str, items: &[&str], index: &mut usize, opts: TabsOpts) -> bool {
        let id = self.s(id);
        let it = self.strs(items);
        let o = opts.raw(self.arena);
        with_index(index, |i| unsafe { sys::cui_tabs_ex(self.ctx, id, it, items.len() as i32, i, &o) })
    }
    /// Sidebar item. Returns true when clicked.
    pub fn nav_item(&mut self, label: &str, selected: bool) -> bool {
        self.nav_item_with(label, NavOpts::new().selected(selected))
    }
    pub fn nav_item_with(&mut self, label: &str, opts: NavOpts) -> bool {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_nav_item_ex(self.ctx, l, &o) }
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
fn with_index(index: &mut usize, f: impl FnOnce(&mut i32) -> bool) -> bool {
    let mut i = *index as i32;
    let r = f(&mut i);
    if let Ok(v) = usize::try_from(i) {
        *index = v;
    }
    r
}
