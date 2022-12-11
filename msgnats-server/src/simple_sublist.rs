use crate::client::ClientMessageSender;
use crate::errors::Result;
use futures::lock::Mutex;
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

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

// 定义ArcSubResult类型

pub type ArcSubResult = Arc<SubResult>;
// SubListTrait是他对外提供的服务接口,主要是
// 1. 新增订阅 这个是当一个Client 发送sub消息到服务端的时候要处理的
// 2. 删除订阅 这个是当一个Client发送 unsub消息到服务端的时候要处理的,不过因为我们不支持unsub,那就是连接断开的时候处理的.
// 3. 查找相关订阅 这个是当一个client发送pub消息到服务端后,服务端要查找所有相关的订阅,然后把消息逐一转发给他们.
pub trait SubListTrait {
    fn insert(&mut self, sub: ArcSubscription) -> Result<()>;
    fn remove(&mut self, sub: ArcSubscription) -> Result<()>;
    fn match_subject(&self, subject: &str) -> Result<ArcSubResult>;
}
// 订阅列表 SimpleSubList中,BTreeSeet中的存放的是ArcSubscriptionWrapper,而不是ArcSubscriptionWrapper.
// 这是有意为之的,因为我们在向BTreeSet中插入新的Sub的时候不需要关心他们真实的顺序,只是需要关心他们是否相同. 所以我们比较的对象是他们的地址而不是内容.
// 但是因为孤儿原则的限制,我们不能为Arc实现Ord这个trait,只能再多一次wrapper, 相信我们代码中有不少为孤儿原则做出的让步.
#[derive(Debug, Default)]
pub struct SimpleSubList {
    subs: HashMap<String, BTreeSet<ArcSubscriptionWrapper>>,
    qsubs: HashMap<String, HashMap<String, BTreeSet<ArcSubscriptionWrapper>>>,
}

impl SubListTrait for SimpleSubList {
    /**
     * 向subList中插入SubScription，通过地址来判断唯一性
     */
    fn insert(&mut self, sub: Arc<SubScription>) -> Result<()> {
        if let Some(ref q) = sub.queue {
            let qsubs = self
                .qsubs
                .entry(sub.subject.clone())
                .or_insert(Default::default());
            let subs = qsubs.entry(q.clone()).or_insert(Default::default());
            subs.insert(ArcSubscriptionWrapper(sub));
        } else {
            let subs = self
                .subs
                .entry(sub.subject.clone())
                .or_insert(Default::default());
            // 零成本抽象
            subs.insert(ArcSubscriptionWrapper(sub));
        }
        Ok(())
    }
    /**
     * 当一个client断开链接的时候，删除所有相关的订阅
     */
    fn remove(&mut self, sub: Arc<SubScription>) -> Result<()> {
        if let Some(ref q) = sub.queue {
            if let Some(qsubs) = self.qsubs.get_mut(&sub.subject) {
                if let Some(subs) = qsubs.get_mut(q) {
                    if subs.remove(&ArcSubscriptionWrapper(sub.clone())) {
                        if subs.is_empty() {
                            qsubs.remove(q);
                        }
                    }
                }
                if qsubs.is_empty() {
                    // 不存在值 清空
                    self.qsubs.remove(&sub.subject);
                }
            }
        } else {
            if let Some(subs) = self.subs.get_mut(&sub.subject) {
                if subs.remove(&ArcSubscriptionWrapper(sub.clone())) {
                    if subs.is_empty() {
                        // 不存在值 清空
                        self.subs.remove(&sub.subject);
                    }
                }
            }
        }
        Ok(())
    }
    /**
     * 当一个client pub一个消息的时候需要查找相关的订阅者
     */
    fn match_subject(&self, subject: &str) -> Result<ArcSubResult> {
        let mut r: SubResult = Default::default();

        if let Some(subs) = self.subs.get(subject) {
            for sub in subs.iter() {
                r.ppubs.push(sub.0.clone());
            }
        }

        if let Some(qsubs) = self.qsubs.get(subject) {
            for (_, subs) in qsubs.iter() {
                let mut vec: Vec<ArcSubscription> = vec![];
                for sub in subs {
                    vec.push(sub.0.clone());
                }
                r.qpubs.push(vec);
            }
        }
        Ok(Arc::new(r))
    }
}
