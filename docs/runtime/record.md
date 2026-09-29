Use `#[record]` to work with your own structs in runtime [expressions](macro.expr.html). An expression can create a record, access its fields by name, store it in a signal, or pass it to a procedure.

```rust
use topcoat::runtime::record;

#[record]
#[derive(Clone)]
struct Todo {
    title: String,
    done: bool,
}
```

Declare a struct with named fields and add the attribute above it. Each field must use a type supported by [`expr!`](macro.expr.html), which includes other records. Supported containers can hold records too, for example `Vec<Todo>` and `Option<Todo>`.

# Reading and updating a record

Use a struct literal to create a record and dot syntax to access a field. This example stores a todo in a signal, displays its title, and replaces it with a completed todo when the button is clicked:

```rust
# use topcoat::{Result, context::Cx, runtime::{record, signal}, view::*};
# #[record]
# #[derive(Clone)]
# struct Todo { title: String, done: bool }
# #[component]
# async fn example(cx: &Cx) -> Result<impl View> {
let todo = signal(cx, || Todo {
    title: "Write docs".to_owned(),
    done: false,
});

Ok(view! {
    <p>$(todo.read().title.to_owned())</p>
    <button
        @click=$(|_e| {
            let title = todo.get().title;
            todo.set(Todo { title, done: true });
        })
    >
        "Done"
    </button>
})
# }
```

Provide every field when constructing a record. Expressions do not support struct update syntax (`..base`). The usual expression type rules apply to field values. An integer literal without a suffix has type `usize`, so use a suffix such as `1u32` when a field expects `u32`.

Accessing a field through a borrowed record gives you a borrow of that field. To obtain an owned record from a borrow, call `clone` on a record that implements `Clone`. For a signal containing a record, [`Signal::read`] borrows the value without requiring `Clone`, while [`Signal::get`] clones it.

Render the individual fields you want to display. Records cannot be rendered directly or compared in expressions.

# Declaration limits

Records cannot have generic parameters. Expressions respect Rust's field visibility rules, so a private field is accessible only where Rust permits it. The macro also rejects field names reserved by the browser runtime, including `clone`, `then`, and `constructor`.

# Data sent to and from the browser

Capturing a record in an expression exposes all of its fields to the browser. Private fields are included, so keep secrets out of any record you capture.

Treat records received from the browser as client input. This includes procedure arguments and restored signal values. Rust's visibility rules do not restrict the values a client can submit for private fields. Validate incoming records before using them.

[`Signal::get`]: struct.Signal.html#method.get
[`Signal::read`]: struct.Signal.html#method.read
