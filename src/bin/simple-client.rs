extern crate env_logger;
extern crate ws;

use ws::connect;

fn main() {
    if let Err(error) = connect("ws://192.168.11.52:3012", |out| {
        out.send("Hello");

        move |msg| {
            println!("Got message `{}`", msg);
            out.close(ws::CloseCode::Normal)
        }
    }) {}
}
