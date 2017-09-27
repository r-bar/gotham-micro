extern crate gotham_micro;
extern crate gotham;
extern crate dotenv;
use gotham_micro::{router, set_logging};

fn main() {
    dotenv::dotenv().ok();
    set_logging();

    gotham::start("127.0.0.1:7878", router());
}
