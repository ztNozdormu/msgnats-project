use std::{collections::HashMap, error::Error, sync::Arc};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::{
    client::{Client, ClientMessageSender},
    simple_sublist::SubListTrait,
};

/**
 * 服务端数据结构定义
 */
#[derive(Debug, Default)]
pub struct Server<T: SubListTrait> {
    state: Arc<Mutex<ServerState<T>>>,
}

#[derive(Debug, Default)]
pub struct ServerState<T: SubListTrait> {
    pub clients: HashMap<u64, Arc<Mutex<ClientMessageSender>>>, // 服务端维护的客户端集合
    pub sub_list: T,                                            // 订阅管理列表
    pub gen_cid: u64,                                           // 服务端维护全局客户端ID
}

/**
 * 为服务端实现启动方法和客户端创建方法
 * send 多线程特征 static 静态生命周期特性
 *
 */
impl<T: SubListTrait + Send + 'static> Server<T> {
    // 服务端启动方法
    pub async fn start(self) -> Result<(), Box<dyn Error>> {
        let addr = "127.0.0.1:18888";
        let listener = TcpListener::bind(addr).await?;

        tokio::spawn(async move {
            loop {
                let rc = listener.accept().await;
                if rc.is_err() {
                    print!("accecpt conn is error:{}", rc.err().unwrap()); // rc.unwrap_err()
                    return;
                }
                //  let r = rc.ok().unwrap();// rc.unwrap();
                let (conn, _) = rc.unwrap();
                self.new_client(conn).await;
            }
        });

        Ok(())
    }
    // 客户端创建方法  服务器私有
    async fn new_client(&self, conn: TcpStream) {
        let server_state = self.state.clone();
        let cid = {
            let mut state = server_state.lock().await;
            state.gen_cid += 1;
            state.gen_cid
        };
        let client_message_sender = Client::process_connection(cid, server_state, conn);

        self.state
            .lock()
            .await
            .clients
            .insert(cid, client_message_sender);
    }
}
