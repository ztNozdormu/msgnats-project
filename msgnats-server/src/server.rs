use std::{collections::HashMap, error::Error, sync::Arc};

use tokio::{net::TcpStream, sync::Mutex};

use crate::{client::ClientMessageSender, simple_sublist::SubListTrait};
#[derive(Debug, Default)]
pub struct Server<T: SubListTrait> {
    state: Arc<Mutex<ServerState<T>>>,
}

#[derive(Debug, Default)]
pub struct ServerState<T: SubListTrait> {
    clients: HashMap<u64, Arc<Mutex<ClientMessageSender>>>, // 服务端维护的客户端集合
    sub_list: T,                                            // 订阅管理列表
    gen_cid: u64,                                           // 服务端维护全局客户端ID
}

/**
 * 为服务端实现启动方法和客户端创建方法
 * send 多线程特征 static 静态生命周期特性
 *
 */
impl<T: SubListTrait + Send + 'static> Server<T> {
    // 服务端启动方法
    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    // 客户端创建方法  服务器私有
    async fn new_client(&self, conn: TcpStream) {}
}
