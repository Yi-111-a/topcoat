# Runtime

Pages that use the browser runtime. Each page shows one feature:

- `counter`: event handlers that change a signal.
- `show`: a bind attribute that follows a signal.
- `sort`: a page that runs again on the server when a signal it reads changes.
- `procedure`: an event handler that calls a function on the server.
- `shard`: search results that render again on the server when the query changes.
- `record`: an order form that keeps a `#[record]` struct in a signal and sends it to a procedure.

Run it with:

```sh
cargo topcoat dev -p runtime
```
