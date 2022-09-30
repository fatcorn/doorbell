use std::future;
use std::thread::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() {
    loop {
        let value = hello_world();
    }

}


async fn hello_world() -> String {
    sleep(Duration::from_secs(1));
    format!("hello world!")
}