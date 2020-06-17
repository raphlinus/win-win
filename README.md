# win-win

A semi-opinionated way to create windows on Windows.

The heart of the crate is a `WindowProc` trait that is a Rust-native wrapping of the "wndproc" pattern in Windows programming.

One goal of the crate is to make it easier to reason about soundness, by providing the correct types for this trait, and documenting the soundness concerns. However, it does *not* try to wrap everything in a safe wrapper.

Another goal is to provide reasonably good documentation, including detailed links to official documentation and other resources. Many of these lessons have been hard-learned, as part of the Windows backend for druid, and other experiments.

The crate is "semi-opinionated" in that it nails down some details, especially the way threads work, but how you draw and the way you handle events is entirely up to you. It is a goal that anybody who creates a HWND from Rust should use this crate. If there's some reason it doesn't work for your use case, I'm curious why, so please file an issue.

There is an optional `kb` feature, which does the rather tricky and fiddly job of converting platform keyboard messages into `KeyboardEvent` structs from the [keyboard-types] crate, based firmly on W3C specs. It's possible that more such features will be added (dpi handling is a strong possibility).

[keyboard-types]: https://crates.io/crates/keyboard-types
