# Changelog

## Unreleased

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
