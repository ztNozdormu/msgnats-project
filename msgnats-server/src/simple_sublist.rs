use std::{cmp::Ordering, env::consts, sync::Arc};

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
// 定义Arc智能指针的SubScription
pub type ArcSubscription = Arc<SubScription>;

// 订阅结果数据结构
#[derive(Debug, Default)]
pub struct SubResult {
    pub ppubs: Vec<ArcSubscription>,
    pub qpubs: Vec<Vec<ArcSubscription>>,
}

impl SubResult {
    pub fn new() -> Self {
        Self {
            ppubs: Vec::new(),
            qpubs: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ppubs.len() == 0 && self.qpubs.len() == 0
    }
}

// SimpleSubList中,保存在BTreeSeet中的存放的是ArcSubscriptionWrapper,而不是ArcSubscriptionWrapper.
// 这是有意为之的,因为我们在向BTreeSet中插入新的Sub的时候不需要关心他们真实的顺序,只是需要关心他们是否相同.
// 所以我们比较的对象是他们的地址而不是内容.
// 但是因为孤儿原则的限制,我们不能为Arc实现Ord这个trait,只能再多一次wrapper,
//  相信我们代码中有不少为孤儿原则做出的让步.

// 孤儿原则的限制,我们不能为Arc实现Ord这个trait,只能再多一次wrapper
#[derive(Debug, Clone)]
pub(crate) struct ArcSubscriptionWrapper(pub ArcSubscription);

impl std::cmp::Eq for ArcSubscriptionWrapper {}

impl std::cmp::PartialEq for ArcSubscriptionWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
// 比較類型
impl std::cmp::Ord for ArcSubscriptionWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 内存指针地址类型
        let a = self.0.as_ref() as *const SubScription as usize;
        let b = other.0.as_ref() as *const SubScription as usize;
        a.cmp(&b)
    }
}
// 比較指針地址
impl std::cmp::PartialOrd for ArcSubscriptionWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
