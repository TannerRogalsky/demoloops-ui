# Demoloops UI
A node-based workflow for making demoloops.

## Thoughts

Must be able to represent `(0..3).flat_map(|y| (0..3).map(move |x| x + 3 * y))`
https://docs.rs/syn/1.0.60/syn/struct.ItemFn.html

```Rust
fn Frame() -> u32;
fn Ratio(length: u32, frame: u32) -> f32;
fn Rectangle(x: f32, y: f32, width: f32, height: f32) -> Rectangle;
fn Count(count: u32) -> Iterator<Item = u32>;
fn Map<B, I: Iterator, F: FnMut(I::Item) -> B>(iter: I, func: F) -> Iterator<Item = B>;
```