//! Options for components, the Rust form of CUI's named arguments.
//!
//! C:    `cui_button(ui, "Save", .variant = CUI_BUTTON_PRIMARY, .icon = CUI_ICON_CHECK)`
//! Rust: `ui.button("Save").variant(ButtonVariant::Primary).icon(icons::CHECK)`
//!
//! Every field has a setter with the C field's name, on the component's
//! builder and on the options struct itself (for presets you pass with
//! `.opts(...)`). Anything left out keeps CUI's default, like the C macros.

use crate::arena::Arena;
use crate::builder::{Container, IntoOpt, Run, Widget};
use crate::sys;
#[allow(unused_imports)]
use crate::types::*;
use core::marker::PhantomData;

/// Defines an options struct whose fields map one to one onto a C options
/// struct, with setters on it and on the builders that carry it. Fields
/// after `;` get setters but are filled in by the component (slices that
/// become a pointer and a count).
macro_rules! options {
    (
        $(#[$m:meta])*
        $name:ident => $c:ident {
            $( $(#[$fm:meta])* $f:ident : $t:ty ),* $(,)?
            $( ; $( $(#[$em:meta])* $ef:ident : $et:ty ),* $(,)? )?
        }
    ) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name<'a> {
            $( pub(crate) $f: $t, )*
            $( $( pub(crate) $ef: $et, )* )?
            _p: PhantomData<&'a ()>,
        }

        impl<'a> $name<'a> {
            pub fn new() -> Self {
                Self::default()
            }

            #[allow(unused_variables)]
            pub(crate) fn raw(&self, a: &mut Arena) -> sys::$c {
                sys::$c {
                    $( $f: crate::types::ToC::to_c(&self.$f, a), )*
                    ..Default::default()
                }
            }
        }

        setters!(opts ['a] $name<'a>; { $( $(#[$fm])* $f: $t, )* $( $( $(#[$em])* $ef: $et, )* )? });
        setters!(ui ['u, 'f, 'a, C: Run<$name<'a>>] Widget<'u, 'f, $name<'a>, C>;
            { $( $(#[$fm])* $f: $t, )* $( $( $(#[$em])* $ef: $et, )* )? });
        setters!(ui ['u, 'f, 'a, C] Container<'u, 'f, $name<'a>, C>;
            { $( $(#[$fm])* $f: $t, )* $( $( $(#[$em])* $ef: $et, )* )? });
    };
}

macro_rules! setters {
    (opts [$($g:tt)*] $ty:ty; { $( $(#[$fm:meta])* $f:ident : $t:ty, )* }) => {
        impl<$($g)*> $ty {
            $(
                $(#[$fm])*
                pub fn $f(mut self, v: impl IntoOpt<$t>) -> Self {
                    self.$f = v.into_opt();
                    self
                }
            )*
        }
    };
    (ui [$($g:tt)*] $ty:ty; { $( $(#[$fm:meta])* $f:ident : $t:ty, )* }) => {
        impl<$($g)*> $ty {
            $(
                $(#[$fm])*
                pub fn $f(mut self, v: impl IntoOpt<$t>) -> Self {
                    self.opts.$f = v.into_opt();
                    self
                }
            )*
        }
    };
}

options! {
    /// Options for every container (`cui_box`).
    BoxOpts => cui_box {
        /// Identity for scroll state etc. (defaults to an automatic id).
        id: Option<&'a str>,
        dir: Dir,
        width: Size,
        height: Size,
        min_width: f32, max_width: f32, min_height: f32, max_height: f32,
        pad: f32, pad_x: f32, pad_y: f32, pad_top: f32, pad_right: f32, pad_bottom: f32, pad_left: f32,
        /// Space between children (and between wrapped lines).
        gap: f32,
        /// `Dir::Grid`: column count (with `min_column_width`: the most columns).
        columns: i32,
        /// `Dir::Grid`: as many columns as fit at this width, at least one.
        min_column_width: f32,
        /// `Dir::Row`: children that do not fit move to a new line.
        wrap: bool,
        align: Align,
        justify: Justify,
        bg: Color,
        border_color: Color,
        border: f32,
        radius: f32, radius_tl: f32, radius_tr: f32, radius_br: f32, radius_bl: f32,
        elevation: Elevation,
        scroll_x: bool,
        scroll_y: bool,
        clip: bool,
        /// 0 means fully opaque.
        opacity: f32,
    }
}

options! {
    CardOpts => cui_card_opts {
        id: Option<&'a str>,
        width: Size, height: Size,
        min_width: f32, max_width: f32,
        pad: f32, pad_x: f32, pad_y: f32, gap: f32,
        dir: Dir,
        align: Align,
        wrap: bool,
        /// Lifts slightly on hover and reports clicks.
        hoverable: bool,
        /// Muted surface, no border.
        muted: bool,
        /// Accent border.
        selected: bool,
    }
}

options! {
    ScrollOpts => cui_scroll_opts {
        id: Option<&'a str>,
        width: Size, height: Size,
        horizontal: bool,
        pad: f32, gap: f32,
        fade_edges: bool,
        fade_color: Color,
    }
}

options! {
    SplitOpts => cui_split_opts {
        vertical: bool,
        min_first: f32, min_second: f32,
        width: Size, height: Size,
    }
}

options! {
    TextOpts => cui_text_opts {
        font: Font,
        size: f32,
        color: Color,
        align: TextAlign,
        /// Single line, ellipsis when it does not fit.
        nowrap: bool,
        /// Tabular numbers.
        tabular: bool,
        width: Size,
        /// Multiplier, default 1.45.
        line_height: f32,
    }
}

options! {
    ButtonOpts => cui_button_opts {
        variant: ButtonVariant,
        size: ControlSize,
        /// Leading icon, e.g. `icons::PLUS`.
        icon: Option<&'a str>,
        icon_right: Option<&'a str>,
        disabled: bool,
        /// Spinner; ignores clicks.
        loading: bool,
        full_width: bool,
        /// Toggled look.
        selected: bool,
        tooltip: Option<&'a str>,
        /// Shown as a hint in the tooltip, e.g. "Ctrl S".
        shortcut: Option<&'a str>,
        width: Size,
        color: Color,
    }
}

options! {
    SegmentedOpts => cui_segmented_opts {
        size: ControlSize,
        full_width: bool,
        icons: Option<&'a [&'a str]>,
    }
}

options! {
    CheckOpts => cui_check_opts {
        description: Option<&'a str>,
        disabled: bool,
        indeterminate: bool,
    }
}

options! {
    SliderOpts => cui_slider_opts {
        /// 0 = continuous.
        step: f32,
        show_value: bool,
        /// printf format for the value, default "%.0f".
        format: Option<&'a str>,
        disabled: bool,
        width: Size,
    }
}

options! {
    ChipOpts => cui_chip_opts {
        icon: Option<&'a str>,
        disabled: bool,
    }
}

options! {
    InputOpts => cui_input_opts {
        /// Label above the field.
        label: Option<&'a str>,
        placeholder: Option<&'a str>,
        icon: Option<&'a str>,
        /// Helper text below.
        hint: Option<&'a str>,
        /// Error text below (red).
        error: Option<&'a str>,
        password: bool,
        disabled: bool,
        /// An x button when not empty.
        clearable: bool,
        autofocus: bool,
        read_only: bool,
        size: ControlSize,
        width: Size,
        /// Text area: height in points; negative fills the space.
        height: f32,
        /// Muted text before the value, e.g. "$".
        prefix: Option<&'a str>,
        /// Muted text after the value, e.g. "kg".
        suffix: Option<&'a str>,
    }
}

options! {
    TagOpts => cui_tag_opts {
        label: Option<&'a str>,
        placeholder: Option<&'a str>,
        max_tags: i32,
        width: Size,
    }
}

options! {
    NumberOpts => cui_number_opts {
        label: Option<&'a str>,
        step: f64,
        decimals: i32,
        disabled: bool,
        width: Size,
        suffix: Option<&'a str>,
    }
}

options! {
    SelectOpts => cui_select_opts {
        label: Option<&'a str>,
        placeholder: Option<&'a str>,
        icons: Option<&'a [&'a str]>,
        /// Combobox: filter options by typing.
        searchable: bool,
        disabled: bool,
        width: Size,
    }
}

options! {
    DateOpts => cui_date_opts {
        label: Option<&'a str>,
        placeholder: Option<&'a str>,
        width: Size,
    }
}

options! {
    TabsOpts => cui_tabs_opts {
        variant: TabsVariant,
        icons: Option<&'a [&'a str]>,
        full_width: bool,
    }
}

options! {
    NavOpts => cui_nav_opts {
        icon: Option<&'a str>,
        badge: Option<&'a str>,
        selected: bool,
        /// Items in one group share the gliding selection highlight.
        group: Option<&'a str>,
    }
}

options! {
    PopoverOpts => cui_popover_opts {
        placement: Placement,
        width: f32,
        pad: f32,
        gap: f32,
    }
}

options! {
    MenuOpts => cui_menu_opts {
        icon: Option<&'a str>,
        variant: ButtonVariant,
        placement: Placement,
        width: f32,
        icon_only: bool,
    }
}

options! {
    MenuItemOpts => cui_menu_item_opts {
        icon: Option<&'a str>,
        shortcut: Option<&'a str>,
        danger: bool,
        disabled: bool,
        checked: bool,
    }
}

options! {
    DialogOpts => cui_dialog_opts {
        description: Option<&'a str>,
        /// Default 440.
        width: f32,
        no_close_button: bool,
        /// Clicking outside or Esc does not close.
        persistent: bool,
    }
}

options! {
    SheetOpts => cui_sheet_opts {
        side: Side,
        size: f32,
        description: Option<&'a str>,
    }
}

options! {
    ToastOpts => cui_toast_opts {
        description: Option<&'a str>,
        tone: Tone,
        /// Seconds, default 4; negative stays until dismissed.
        duration: f32,
        /// Optional action button label.
        action: Option<&'a str>,
        /// A draining line shows how long the action stays (undo).
        show_timer: bool,
    }
}

options! {
    AlertOpts => cui_alert_opts {
        description: Option<&'a str>,
        tone: Tone,
        icon: Option<&'a str>,
        dismissible: bool,
    }
}

options! {
    ProgressOpts => cui_progress_opts {
        indeterminate: bool,
        tone: Tone,
        height: f32,
        label: Option<&'a str>,
        show_value: bool,
        width: Size,
    }
}

options! {
    BadgeOpts => cui_badge_opts {
        tone: Tone,
        dot: bool,
        solid: bool,
        icon: Option<&'a str>,
        size: ControlSize,
    }
}

options! {
    AvatarOpts => cui_avatar_opts {
        size: f32,
        image: Texture,
        /// "online", "away", "busy" or "offline".
        status: Option<&'a str>,
        square: bool,
    }
}

options! {
    EmptyOpts => cui_empty_opts {
        icon: Option<&'a str>,
        description: Option<&'a str>,
        /// Label of an action button.
        action: Option<&'a str>,
    }
}

options! {
    MetricOpts => cui_metric_opts {
        delta: f32,
        delta_label: Option<&'a str>,
        icon: Option<&'a str>,
        width: Size,
        /// A fall is good news (build times, errors), shown in green.
        lower_is_better: bool,
        ;
        /// Values for a sparkline.
        spark: Option<&'a [f32]>,
    }
}

options! {
    TreeOpts => cui_tree_opts {
        icon: Option<&'a str>,
        leaf: bool,
        selected: bool,
    }
}

options! {
    ListItemOpts => cui_list_item_opts {
        icon: Option<&'a str>,
        subtitle: Option<&'a str>,
        trailing: Option<&'a str>,
        selected: bool,
        /// Name for an initials avatar.
        avatar: Option<&'a str>,
    }
}

options! {
    TableOpts => cui_table_opts {
        /// Scrolls when set.
        height: Size,
        striped: bool,
        compact: bool,
        selectable: bool,
    }
}

options! {
    ChartOpts => cui_chart_opts {
        width: Size,
        /// Default 220.
        height: f32,
        /// Fill under lines.
        area: bool,
        no_grid: bool,
        no_legend: bool,
        /// Bar chart.
        stacked: bool,
        /// Curved lines.
        smooth: bool,
        /// Value format for axis and tooltip, default "%.0f".
        format: Option<&'a str>,
        /// Fixed y range when min != max.
        min: f32,
        max: f32,
        ;
        /// X axis labels.
        labels: Option<&'a [&'a str]>,
    }
}

options! {
    ImageOpts => cui_image_opts {
        width: Size,
        height: Size,
        radius: f32,
        tint: Color,
    }
}

options! {
    CodeOpts => cui_code_opts {
        /// "c", "cpp", "js", "ts", "python", "rust", "go", "json", "shell".
        lang: Option<&'a str>,
        /// E.g. a file name.
        title: Option<&'a str>,
        line_numbers: bool,
        /// 1 based line range to emphasize.
        highlight_from: i32,
        highlight_to: i32,
        /// Scrolls beyond this, default none.
        max_height: f32,
        /// Font size, default 13.
        size: f32,
        no_header: bool,
    }
}

options! {
    MarkdownOpts => cui_markdown_opts {
        /// Default fill.
        width: Size,
        /// As wide as the content needs, up to the space there is (chat bubbles).
        fit_content: bool,
        /// Body text size.
        size: f32,
        color: Color,
        /// Text is still arriving.
        streaming: bool,
        /// Smaller headings and tighter spacing.
        compact: bool,
    }
}
