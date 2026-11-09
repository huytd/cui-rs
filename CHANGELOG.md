# Changelog

## Unreleased

- CUI 1.3.0 (from 1.1.0). `cui-sys` covers everything new: right-to-left
  layout, the developer tools (trace viewer, network waterfall, dependency
  graph, commit graph, pipeline, problems panel, key-value editor, query
  builder, schedule picker, keymap, test explorer, stack trace, blame,
  coverage, regex tester, HTTP request bar, API explorer, feature flags,
  uptime bars, time histogram), `cui_set_strings`, `cui_state_on_free` and
  the rest. The new components are available raw through `cui::sys`.
- `Direction` and a `direction` option on containers and text, for
  right-to-left layout (`ui.row().direction(Direction::Rtl)`).
- `TextAlign` gains `Start` and `End`, and defaults to `Start`, which follows
  the reading direction. CUI renumbered `CUI_TEXT_LEFT`; `Left` and `Right`
  now stay put under right-to-left layout.
- `Ui::request_wakeup(seconds)`: a frame within that time with no input, for
  content that changes on a timer.
- Builder API: components with options return a builder you chain options
  onto (`ui.button("Save").primary().clicked()`), drawn when the statement
  ends or when you ask for the result. Containers end with `.show(|ui| ...)`.
  Replaces the `*_with(..., XxxOpts::new()...)` methods.
- Option setters accept integers for floats, plain numbers for sizes, `&str`
  for optional strings and `Token`s for colors.
- `text!(ui, "...")` formats straight into frame memory.
- Text fields edit a `String` as well as a `TextBuffer`.
- `id_scope` takes a string or any integer (replaces `id_scope_int`).
- `cui::prelude`, `cui::FILL`, `.opts(preset)` on every builder, and a
  `tag_input` with options.

## 0.1.0

- `cui-sys`: bindings to all of CUI 1.1.0 (`cui.h`, `cui_app.h`, `cui_render.h`), C build from upstream's CMake source list.
- `cui-rs`: safe API for the app loop, layout, text, buttons, selection controls, text input, pickers, navigation, menus, popovers, dialogs, sheets, toasts, alerts, feedback, trees, lists, virtual lists, tables, charts, images, code blocks, JSON and markdown; `Headless` software rendering with synthetic input; generated `icons`.
- `cargo xtask update | bindgen | check | coverage`, and a daily workflow that opens a PR when CUI changes.
