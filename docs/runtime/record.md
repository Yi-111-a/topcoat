The `#[record]` attribute makes a struct usable in runtime [expressions](macro.expr.html). Records group values under field names, like tuples group them by position. Expressions can build records, read their fields, and pass them to signals and procedures.

```rust
use topcoat::runtime::record;

#[record]
#[derive(Clone)]
struct Todo {
    title: String,
    done: bool,
}
```

Every field type must belong to the shared vocabulary of [`expr!`](macro.expr.html). Fields can also be other records, and records can appear inside other supported types, such as `Vec<Todo>` or `Option<Todo>`.

# Using Records In Expressions

An expression builds a record with Rust's struct literal syntax and reads a field with `.`:

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

Field values follow the expression rules for their types. For example, an unsuffixed integer literal is a `usize`, so a `u32` field needs a value such as `1u32`. Struct update syntax (`..base`) is not supported.

Reading a field of a borrowed record borrows the field, as with tuples. `clone` copies a borrowed record into an owned one if the struct implements `Clone`. [`Signal::get`] also requires `Clone`; [`Signal::read`] does not.

A record does not render on its own. Render its fields instead.

# Declaring Records

A record must be a struct with named fields and no generic parameters. Field visibility applies inside expressions as it does in Rust. Some field names are reserved because they would collide with names the browser runtime uses, such as `clone`, `then`, and `constructor`. The macro reports an error for a reserved name.

# Security

A captured record sends every field to the browser, including private fields. Do not store secrets in records that reach an expression.

Records that come back from the browser, such as procedure arguments or restored signal values, are rebuilt from client input. The browser can send any field values, even values that Rust code outside the module could not construct. Validate records from the browser like any other client input.

[`Signal::get`]: struct.Signal.html#method.get
[`Signal::read`]: struct.Signal.html#method.read
