pub mod handler;

extern crate log;

use async_std::net::TcpListener;
use futures::executor::block_on;


async fn run_serv() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        handler::handle_stream(stream).await;
    }
}

fn main() {
    env_logger::init();
    block_on(run_serv())
}
