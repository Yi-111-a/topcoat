use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::{Event, procedure, signal},
    view::{View, view},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let input = signal(cx, String::new);

    Ok(view! {
        // `:value` renders the signal, `@change` writes back to it.
        <input :value=$(input.get()) @change=$(|e: Event| input.set(e.target.value))>

        // Calls the procedure on the server and puts its return value back
        // into the signal.
        <button
            @click=$(async |_e| {
                let server_response = print_on_server(input.get()).await;
                input.set(server_response);
            })
        >
            "Print on server"
        </button>
    })
}

// The arguments come from the client, so a real application would validate
// them here.
#[procedure]
pub async fn print_on_server(input: String) -> Result<String> {
    println!("{input}");
    Ok(format!("message received: {input}"))
}
