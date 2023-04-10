#![allow(unreachable_code)]

use std::{sync::Mutex};
use rppal::{gpio::Gpio};

#[macro_use]
extern crate rouille;
#[macro_use] extern crate serde_derive;
extern crate serde;


#[derive(Serialize)]
struct Relay {
    id: usize,
    pin: u8,
    active: bool
}

#[derive(Serialize)]
struct SerializedRelayList {
    outlets: Vec<Relay>
}

fn init_gpio() -> Vec<Relay> {
    let mut relaysnums = Vec::new();
    relaysnums.push(7);
    relaysnums.push(3);
    relaysnums.push(22);
    relaysnums.push(25);

    let mut relays = Vec::with_capacity(relaysnums.len());
    for (i, el) in relaysnums.iter().enumerate() {
        relays.push(Relay {
            id: i,
            pin: *el,
            active: false
        });
    }

    return relays;
}

fn main() {
    println!("Now listening on 0.0.0.0:8000");

    let gpios = Mutex::new(init_gpio());

    // The `start_server` starts listening forever on the given address.
    rouille::start_server("0.0.0.0:8000", move |request| {

        router!(request,
            (GET) (/) => {
                rouille::Response::redirect_302("/status")
            },

            (GET) (/status) => {
                let lgpios = gpios.lock().unwrap();
                let mut local_gpios = Vec::new();

                for gpio in lgpios.iter() {
                    local_gpios.push(Relay {
                        id: gpio.id,
                        pin: gpio.pin,
                        active: gpio.active
                    })
                }
                rouille::Response::json(&SerializedRelayList { outlets: local_gpios })
            },

            (GET) (/off) => {
                let mut lgpios = gpios.lock().unwrap();
                let mut found = false;
                let relay_num;

				let gpio = Gpio::new().unwrap();

                match request.get_param("id") {
                    Some(index) => relay_num = index.parse::<usize>().unwrap(),
                    None => return rouille::Response::text("Invalid id!"),
                }

                for relay in lgpios.iter_mut() {
                    if relay_num == relay.id {
                        (*relay).active = false;
						let mut pin = gpio.get((*relay).pin).unwrap().into_output();
						pin.set_low();
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
                let relay_num;

				let gpio = Gpio::new().unwrap();

                match request.get_param("id") {
                    Some(index) => relay_num = index.parse::<usize>().unwrap(),
                    None => return rouille::Response::text("Invalid id!"),
                }

                for relay in lgpios.iter_mut() {
                    if relay_num == relay.id {
						let mut pin = gpio.get((*relay).pin).unwrap().into_output();
						pin.set_high();
                        (*relay).active = true;
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