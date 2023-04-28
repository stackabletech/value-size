# value-size

***Experimental*** helper library for memory-profiling Rust code.

A fork of k8s-openapi that supports this library is available at https://github.com/stackabletech/k8s-openapi/tree/spike/value-size.

## Usage

```rust
use value_size::Size;

#[derive(Size)]
struct MyThingToAnalyze {
    primitive_value: u8,
    dynamic_value: String,
}

// An empty string does not allocate anything, so we only store data on the stack.
let empty = MyThingToAnalyze {
    primitive_value: 0,
    dynamic_value: String::new(),
};
assert_eq!(empty.full_size(), std::mem::size_of::<MyThingToAnalyze>());

// Strings are allocated on the heap, so we also need to count its size to get an accurate understanding
// of our memory consumption.
let with_content = MyThingToAnalyze {
    primitive_value: 0,
    dynamic_value: "content!".to_string(),
};
assert_eq!(
    with_content.full_size(),
    std::mem::size_of::<MyThingToAnalyze>()
        + with_content.dynamic_value.capacity(),
);

// Boxes are also hidden from `std::mem::size_of`, but we can see them!
#[derive(Size)]
struct Boxed {
    value: u8,
}
#[derive(Size)]
struct HasBox {
    the_box: Box<Boxed>,
}
assert_eq!(
    HasBox {
        the_box: Box::new(Boxed { value: 0 }),
    }.full_size(),
    std::mem::size_of::<HasBox>() + std::mem::size_of::<Boxed>(),
);
```
