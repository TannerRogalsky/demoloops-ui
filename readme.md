# Demoloops UI
A node-based workflow for making demoloops.

## Thoughts

Must be able to represent `(0..3).flat_map(|y| (0..3).map(move |x| x + 3 * y))`
https://docs.rs/syn/1.0.60/syn/struct.ItemFn.html

### Reducing Heap Allocations
Both `Many<T>` and the input/output traits use `Box<T>` to encapsulate the dynamic type system. If we deigned to elevate certain types to Supported to the exclusion of all others, the internals of `Many<T>` could be replaced with an enum of `Map`, `Zip`, etc. and the input/output traits could, similarly, receive an enum. The shape of the input/output enum would have to encapsulte both the elevated types are the one-many allowance in the system.

### Multidimensional Ranges
If a range had multiple inputs it could automatically expand them as if it were a multidimensional loop. This could be encapsulated into a single iterator. What would it output? Let's consider the Rust equivalent.

```Rust
let width = 2;
let height = 3;

let output1 = (0..width).flat_map(|y| (0..height).map(move |x| x + y * height)).collect::<Vec<_>>();
assert_eq!(vec![0, 1, 2, 3, 4, 5], output1);

let output2 = (0..width).flat_map(|_y| (0..height)).collect::<Vec<_>>();
assert_eq!(vec![0, 1, 2, 0, 1, 2], output2);
```

It's not immediately clear to me which of these is more useful. My inclination is that the second implementation discards important information (see: the discarding of the varying `y`). The first implementation looks like an enumeration (which could be easily constructed from the second) in this example but that's only because both ranges start from 0. If necessary, a custom iterator could be created to provide flexibility here but the actual usefulness needs to be identified first.

```Rust
fn Frame() -> u32;
fn Ratio(length: u32, frame: u32) -> f32;
fn Rectangle(x: f32, y: f32, width: f32, height: f32) -> Rectangle;
fn Count(count: u32) -> Range<u32>;
fn Map<B, I: Iterator, F: FnMut(I::Item) -> B>(iter: I, func: F) -> Map<Item = B>;
```

```rust
enum Node {
	Frame,
	Ratio,
	Multiply,
}

impl Node {
	fn op(self, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, OpError> {
		match self {
			Self::Frame => Ok(Box::new(0)),
			Self::Ratio => dyn_wrapper(ratio, args)
		}
	}
}
```

## Ideas
- "Genres" of function that perform the same operation but on different types of inputs.
- Iterators or Futures/Streams as a common interface for nodes.
- The only problem that having an explicit enum of Node types solves is the data marshalling. And that is largely secured in the NodeInput trait anyway.