use std::error::Error;

use simple_sublist::SimpleSubList;

use crate::server::Server;
pub mod client;
pub mod errors;
pub mod parser;
pub mod server;
pub mod simple_sublist;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("server is begin start......");
    let server: Server<SimpleSubList> = Server::default();
    server.start().await?;
    Ok(())
}
