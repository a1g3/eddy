#![allow(unreachable_code)]

use std::{sync::Mutex};

#[macro_use]
extern crate rouille;
#[macro_use] extern crate serde_derive;
extern crate serde;


#[derive(Serialize)]
struct Relay {
    id: usize,
    active: bool
}

#[derive(Serialize)]
struct SerializedRelayList {
    outlets: Vec<Relay>
}

fn init_gpio() -> Vec<Relay> {
    let num_relays: usize = 4;
    let mut relays = Vec::with_capacity(num_relays);
    for i in 1..num_relays {
        relays.push(Relay {
            id: i,
            active: false
        });
    }

    return relays;
}

fn main() {
    println!("Now listening on localhost:8000");

    let gpios = Mutex::new(init_gpio());

    // The `start_server` starts listening forever on the given address.
    rouille::start_server("localhost:8000", move |request| {
        // The closure passed to `start_server` will be called once for each client request. It
        // will be called multiple times concurrently when there are multiple clients.
        // Here starts the real handler for the request.
        //
        // The `router!` macro is very similar to a `match` expression in core Rust. The macro
        // takes the request as parameter and will jump to the first block that matches the
        // request.
        //
        // Each of the possible blocks builds a `Response` object. Just like most things in Rust,
        // the `router!` macro is an expression whose value is the `Response` built by the block
        // that was called. Since `router!` is the last piece of code of this closure, the
        // `Response` is then passed back to the `start_server` function and sent to the client.
        router!(request,
            (GET) (/) => {
                let mut lgpios = gpios.lock().unwrap();
                lgpios.remove(3);
                rouille::Response::redirect_302("/hello/world")
            },

            (GET) (/status) => {
                let lgpios = gpios.lock().unwrap();
                let mut local_gpios = Vec::new();

                for gpio in lgpios.iter() {
                    local_gpios.push(Relay {
                        id: gpio.id,
                        active: gpio.active
                    })
                }
                // Builds a `Response` object that contains the "hello world" text.
                rouille::Response::json(&SerializedRelayList { outlets: local_gpios })
            },

            (GET) (/off) => {
                let mut lgpios = gpios.lock().unwrap();
                let mut found = false;
                let mut relay_num = 0;


                match request.get_param("id") {
                    Some(index) => relay_num = index.parse::<usize>().unwrap(),
                    None => return rouille::Response::text("Invalid id!"),
                }

                for gpio in lgpios.iter_mut() {
                    if relay_num == gpio.id {
                        (*gpio).active = false;
                        found = true;
                    }
                }

                if !found {
                    return rouille::Response::text("Invalid id!")
                } else {

                    // For the same of the example we return an empty response with a 400 status code.
                    return rouille::Response::text("Success!")
                }

            },

            (GET) (/on) => {
                let mut lgpios = gpios.lock().unwrap();
                let mut found = false;
                let mut relay_num = 0;


                match request.get_param("id") {
                    Some(index) => relay_num = index.parse::<usize>().unwrap(),
                    None => return rouille::Response::text("Invalid id!"),
                }

                for gpio in lgpios.iter_mut() {
                    if relay_num == gpio.id {
                        (*gpio).active = true;
                        found = true;
                    }
                }

                if !found {
                    return rouille::Response::text("Invalid id!")
                } else {

                    // For the same of the example we return an empty response with a 400 status code.
                    return rouille::Response::text("Success!")
                }

            },

            // The code block is called if none of the other blocks matches the request.
            // We return an empty response with a 404 status code.
            _ => rouille::Response::empty_404()
        )
    });
}