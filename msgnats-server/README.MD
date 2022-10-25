
# 从零实现消息中间件

> 通过学习rnats练习rust 参考博主链接:https://stevenbai.top/rust/rnats
> 学习过程:先学习文档业务再根据业务逻辑组织代码,文档代码作为参考
 [toc]


##  Typora常用语法
> https://www.cnblogs.com/luckforefforts/p/13642696.html  
###  一 parser模块
#### 1. 消息格式
> **服务器和客户端来往的消息只有三种,分别是订阅(SUB),发布(PUB),推送消息(MSG). 其中前两种是从客户端向服务端推送,最后一种则是服务端向客户端推送.**

#### 2. 服务端需要解析的消息格式
  * pub
   ```rust 
   PUB <subject> <size>\r\n
   <message>\r\n
   ```
  * sub 
  ```rust
   SUB <subject> <sid>\r\n
   SUB <subject> <queue> <sid>\r\n
  ```
#### 3. 客户端需要解析的消息格式
  * MSG
   ```rust
    MSG <subject> <sid> <size>\r\n
    <message>\r\n 
   ```
#### 4. 消息数据格式解析思路分析
> **出于性能考虑,以下几个问题需要注意**
   1. 尽可能避免内存分配
   2. 尽可能避免内存复制(zero copy)
   3. 不要使用正则表达式去匹配   
   总结:针对服务端采用状态机模式,灵活的方式调整实现上述三个目标
#### 5. 具体实现步骤
1. 自定义系统统一错误定义
   > 预先定义系统可能发生的错误类型
      ```rust
        pub const ERROR_PARSE: i32 = 1;
        pub const ERROR_MESSAGE_SIZE_TOO_LARGE: i32 = 2;
        pub const ERROR_INVALID_SUBJECT: i32 = 3;
        pub const ERROR_SUBSCRIBTION_NOT_FOUND: i32 = 4;
        pub const ERROR_CONNECTION_CLOSED: i32 = 5;
        pub const ERROR_UNKOWN_ERROR: i32 = 1000;
        #[derive(Debug)]
        pub struct NError {
            pub err_code: i32,
        }
        impl NError {
            pub fn new(err_code: i32) -> Self {
                Self { err_code }
            }
            pub fn error_description(&self) -> &'static str {
                match self.err_code {
                    ERROR_PARSE => return "parse error",
                    ...
                    _ => return "unkown error",
                }
            }
        }
    ```
2. 状态定义
   >  采用的是逐个byte解析的方式. 只处理pub和sub两种消息. 其中sub支持可选的queue来做负载均衡
    ``` rust
        #[derive(Debug, Clone)]
        enum ParseState {
            OpStart,
            OpS,
            OpSu,
            OpSub,
            OPSubSpace,
            OpSubArg,
            OpP,
            OpPu,
            OpPub, //pub argument
            OpPubSpace,
            OpPubArg,
            OpMsg, //pub message
            OpMsgFull,
        }
    ```
3. Parser解析器以及parse结果数据结构定义   
   > **大佬们建议写rust代码之前最好先定义清楚数据结构.**
     * 返回结果数据结构定义
     > parse的结果不外乎四种情况 1. 出错了 2. 到目前为止还没有收到完整的消息 比如只收到了SUB SUBJECT ,消息不完整,当然不能处理 3. 一条PUB消息 4. 一条SUB消息
     ```rust
     #[derive(Debug, PartialEq)]
     pub struct SubArg<'a> {
        pub subject: &'a str, //为什么是str而不是String,就是为了避免内存分配,
        pub sid: &'a str,
        pub queue: Option<&'a str>,
     }
     #[derive(Debug, PartialEq)]
     pub struct PubArg<'a> {
        pub subject: &'a str,
        pub size_buf: &'a str, // 1024 字符串形式,避免后续再次转换
        pub size: usize,       //1024 整数形式
        pub msg: &'a [u8],
     }
     #[derive(Debug, PartialEq)]
     pub enum ParseResult<'a> {
        NoMsg, //buf="sub top.stevenbai.blog" sub消息不完整,我肯定不能处理
        Sub(SubArg<'a>),
        Pub(PubArg<'a>),
     }
     ```
    * Parser数据结构定义
    > Parser的定义这个版本我们尽量去满足上述三个原则,但是考虑到第二条zero-copy会让代码中 到处都是if-else,所以暂时先不考虑
    ```rust
    /*
    这个长度很有关系,必须能够将一个完整的主题以及参数放进去,
    所以要限制subject的长度
    */
    const BUF_LEN: usize = 512;
    pub struct Parser {
        state: ParseState,
        buf: [u8; BUF_LEN], //消息解析缓冲区,如果消息体+消息头不超过512,直接用这个,超过了就必须另分配
        arg_len: usize,
        msg_buf: Option<Vec<u8>>,
        //解析过程中受到新消息,那么 新消息的总长度是msg_total_len,已收到部分应该是msg_len
        msg_total_len: usize,
        msg_len: usize,
        debug: bool,
    }
    ```
4.  消息解析核心业务逻辑
   * parse 函数的定义
   * parse函数的使用
   * 完整parse的实现



