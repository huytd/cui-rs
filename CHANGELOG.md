# Changelog

## 0.1.0

- `cui-sys`: bindings to all of CUI 1.1.0 (`cui.h`, `cui_app.h`, `cui_render.h`), C build from upstream's CMake source list.
- `cui-rs`: safe API for the app loop, layout, text, buttons, selection controls, text input, pickers, navigation, menus, popovers, dialogs, sheets, toasts, alerts, feedback, trees, lists, virtual lists, tables, charts, images, code blocks, JSON and markdown; `Headless` software rendering with synthetic input; generated `icons`.
- `cargo xtask update | bindgen | check | coverage`, and a daily workflow that opens a PR when CUI changes.
