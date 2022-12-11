use std::sync::Arc;

use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::server::ServerState;
use crate::simple_sublist::SubListTrait;

/**
 * 定义client
 */
#[derive(Debug, Default)]
pub struct Client<T: SubListTrait> {
    sub_list: T,
}

impl<T: SubListTrait + Send + 'static> Client<T> {
    pub fn process_connection(
        cid: u64,
        serv_state: Arc<Mutex<ServerState<T>>>,
        conn: TcpStream,
    ) -> Arc<Mutex<ClientMessageSender>> {
        todo!()
    }
}

#[derive(Debug)]
pub struct ClientMessageSender {
    writer: Option<WriteHalf<TcpStream>>,
    msg_buf: Option<Vec<u8>>,
}

impl ClientMessageSender {
    pub fn new(writer: WriteHalf<TcpStream>) -> Self {
        Self {
            writer: Some(writer),
            msg_buf: Some(Vec::with_capacity(512)), // 初始缓冲区大小 512
        }
    }

    pub async fn send_all(&mut self) -> std::io::Result<()> {
        if let Some(ref mut writer) = self.writer {
            let r = writer
                .write_all(self.msg_buf.as_ref().unwrap().as_slice())
                .await;
            self.msg_buf.as_mut().unwrap().clear(); //清空数据
            r
        } else {
            Ok(())
        }
    }
}
