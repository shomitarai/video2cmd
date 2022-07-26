extern crate env_logger;
extern crate ws;

use ws::listen;

fn main() {
    if let Err(error) = listen("192.168.11.52", |out| {
        move |msg| {
            println!("Got Message `{}`", msg);
            out.send(msg)
        }
    }) {}
}
