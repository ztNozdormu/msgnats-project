use std::sync::Arc;

use futures::lock::Mutex;

use crate::client::ClientMessageSender;

/**
考虑到Trie的实现以及Cache的实现都是很琐碎,
我这里专门实现一个简单的订阅关系查找,不支持*和>这两种模糊匹配.
这样就是简单的字符串查找了. 使用map即可.
但是为了后续的扩展性呢,我会定义SubListTrait,这样方便后续实现Trie树
*/
//  订阅消息描述结构体
#[derive(Debug)]
pub struct SubScription {
    pub msg_sender: Arc<Mutex<ClientMessageSender>>,
    pub subject: String,
    pub queue: Option<String>,
    pub sid: String,
}

impl SubScription {
    fn new(
        msg_sender: Arc<Mutex<ClientMessageSender>>,
        subject: &str,
        queue: Option<&str>,
        sid: &str,
    ) -> Self {
        Self {
            msg_sender,
            subject: subject.to_string(),
            queue: queue.map(|q| q.to_string()),
            sid: sid.to_string(),
        }
    }
}
