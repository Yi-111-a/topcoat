use std::sync::atomic::{AtomicU64, Ordering};

use topcoat::{
    Result,
    context::Cx,
    router::page,
    runtime::{Event, procedure, record, signal},
    view::{View, view},
};

// `#[record]` makes a struct usable in runtime expressions: they can build
// it, read its fields, keep it in a signal, and send it to the server.
#[record]
#[derive(Clone)]
pub struct Order {
    item: String,
    quantity: u32,
    address: Address,
}

// Records can contain other records.
#[record]
#[derive(Clone)]
pub struct Address {
    street: String,
    city: String,
}

#[record]
#[derive(Clone)]
pub struct Receipt {
    number: u64,
    summary: String,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let order = signal(cx, || Order {
        item: "coffee".to_owned(),
        quantity: 1,
        address: Address {
            street: String::new(),
            city: String::new(),
        },
    });
    let receipt = signal(cx, || None::<Result<Receipt, String>>);

    // Reading the receipt on the server makes the page run again when the
    // browser stores a new one, so plain Rust renders it below.
    let placed = receipt.get();

    Ok(view! {
        // Each handler builds a new order from the current one and stores it.
        <label>
            "item "
            <input
                :value=$(order.read().item.to_owned())
                @input=$(|e: Event| {
                    let order_ = order.get();
                    order.set(Order {
                        item: e.target.value,
                        quantity: order_.quantity,
                        address: order_.address,
                    });
                })
            >
        </label>

        <p>
            "quantity: " $(order.get().quantity) " "
            <button
                @click=$(|_e| {
                    let order_ = order.get();
                    if order_.quantity > 1u32 {
                        order.set(Order {
                            quantity: order_.quantity - 1u32,
                            item: order_.item,
                            address: order_.address,
                        });
                    }
                })
            >
                "-"
            </button>
            <button
                @click=$(|_e| {
                    let order_ = order.get();
                    order.set(Order {
                        quantity: order_.quantity + 1u32,
                        item: order_.item,
                        address: order_.address,
                    });
                })
            >
                "+"
            </button>
        </p>

        <label>
            "street "
            <input
                :value=$(order.read().address.street.to_owned())
                @input=$(|e: Event| {
                    let order_ = order.get();
                    order.set(Order {
                        address: Address { street: e.target.value, city: order_.address.city },
                        item: order_.item,
                        quantity: order_.quantity,
                    });
                })
            >
        </label>

        <label>
            "city "
            <input
                :value=$(order.read().address.city.to_owned())
                @input=$(|e: Event| {
                    let order_ = order.get();
                    order.set(Order {
                        address: Address { street: order_.address.street, city: e.target.value },
                        item: order_.item,
                        quantity: order_.quantity,
                    });
                })
            >
        </label>

        // Sends the whole order to the server and stores the answer.
        <p>
            <button
                @click=$(async |_e| {
                    let result = place_order(order.get()).await;
                    receipt.set(Some(result));
                })
            >
                "place order"
            </button>
        </p>

        match placed {
            Some(Ok(receipt)) => {
                <p>"Order #" (receipt.number) ": " (receipt.summary)</p>
            }
            Some(Err(message)) => <p>"Could not place the order: " (message)</p>,
            None => {}
        }
    })
}

static NEXT_ORDER: AtomicU64 = AtomicU64::new(1);

// The order comes from the browser, so the procedure validates it like any
// other client input. The nested `Result` lets the page show the problem.
#[procedure]
pub async fn place_order(order: Order) -> Result<Result<Receipt, String>> {
    if !(1..=10).contains(&order.quantity) {
        return Ok(Err("orders are limited to 10 items".to_owned()));
    }
    if order.address.street.trim().is_empty() || order.address.city.trim().is_empty() {
        return Ok(Err("please enter a street and city".to_owned()));
    }

    Ok(Ok(Receipt {
        number: NEXT_ORDER.fetch_add(1, Ordering::Relaxed),
        summary: format!(
            "{} x {} to {}, {}",
            order.quantity, order.item, order.address.street, order.address.city
        ),
    }))
}
