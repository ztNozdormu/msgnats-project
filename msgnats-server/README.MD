
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
   ```rust
    /**
    对收到的字节序列进行解析,解析完毕后得到pub或者sub消息,
    同时有可能没有消息或者缓冲区里面还有其他消息
    返回结果中的usize指的是消耗了缓冲区中多少字节
    */
    pub fn parse(&mut self, buf: &[u8]) -> Result<(ParseResult, usize)>
   ```
   * parse函数的使用
   ```rust
   fn test_sub2() {
        let mut p = Parser::new();
        let mut buf = "SUB subject 1\r\nSUB subject2 2\r\n".as_bytes();
        loop {
            let r = p.parse(buf);
            assert!(!r.is_err());
            let r = r.unwrap();
            buf = &buf[r.1..];
            match r.0 {
                ParseResult::Sub(sub) => {
                    println!("sub.subect={}", sub.subject);
                }
                _ => panic!(),
            }
            if buf.len() == 0 {
                break;
            }
        }
    }
   ```
   * 完整parse的实现
   ```rust
   impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParseState::OpStart,
            buf: [0; BUF_LEN],
            arg_len: 0,
            msg_buf: None,
            msg_total_len: 0,
            msg_len: 0,
            debug: true,
        }
    }
    /**
    对收到的字节序列进行解析,解析完毕后得到pub或者sub消息,
    同时有可能没有消息或者缓冲区里面还有其他消息
    */
    pub fn parse(&mut self, buf: &[u8]) -> Result<(ParseResult, usize)> {
        let mut b;
        let mut i = 0;
        if self.debug {
            println!(
                "parse string:{},state={:?}",
                unsafe { std::str::from_utf8_unchecked(buf) },
                self.state
            );
        }
        while i < buf.len() {
            use ParseState::*;
            b = buf[i] as char;
            //            println!("state={:?},b={}", self.state, b);
            match self.state {
                OpStart => match b {
                    'S' => self.state = OpS,
                    'P' => self.state = OpP,
                    _ => parse_error!(),
                },
                OpS => match b {
                    'U' => self.state = OpSu,
                    _ => parse_error!(),
                },
                OpSu => match b {
                    'B' => self.state = OpSub,
                    _ => parse_error!(),
                },
                OpSub => match b {
                    //sub stevenbai.top 3 是ok的,但是substevenbai.top 3就不允许
                    ' ' | '\t' => self.state = OPSubSpace,
                    _ => parse_error!(),
                },
                OPSubSpace => match b {
                    ' ' | '\t' => {}
                    _ => {
                        self.state = OpSubArg;
                        self.arg_len = 0;
                        continue;
                    }
                },
                OpSubArg => match b {
                    '\r' => {}
                    '\n' => {
                        self.state = OpStart;
                        let r = self.process_sub()?;
                        return Ok((r, i + 1));
                    }
                    _ => {
                        self.add_arg(b as u8)?;
                    }
                },
                OpP => match b {
                    'U' => self.state = OpPu,
                    _ => parse_error!(),
                },
                OpPu => match b {
                    'B' => self.state = OpPub,
                    _ => parse_error!(),
                },
                OpPub => match b {
                    ' ' | '\t' => self.state = OpPubSpace,
                    _ => parse_error!(),
                },
                OpPubSpace => match b {
                    ' ' | '\t' => {}
                    _ => {
                        self.state = OpPubArg;
                        self.arg_len = 0;
                        continue;
                    }
                },
                OpPubArg => match b {
                    '\r' => {}
                    '\n' => {
                        //PUB top.stevenbai 5\r\n
                        self.state = OpMsg;
                        let size = self.get_message_size()?;
                        if size == 0 || size > 1 * 1024 * 1024 {
                            //消息体长度不应该超过1M,防止Dos攻击
                            return Err(NError::new(ERROR_MESSAGE_SIZE_TOO_LARGE));
                        }
                        if size + self.arg_len > BUF_LEN {
                            self.msg_buf = Some(Vec::with_capacity(size));
                        }
                        self.msg_total_len = size;
                    }
                    _ => {
                        self.add_arg(b as u8)?;
                    }
                },
                OpMsg => {
                    //涉及消息长度
                    if self.msg_len < self.msg_total_len {
                        self.add_msg(b as u8);
                    } else {
                        self.state = OpMsgFull;
                    }
                }
                OpMsgFull => match b {
                    '\r' => {}
                    '\n' => {
                        self.state = OpStart;
                        let r = self.process_msg()?;
                        return Ok((r, i + 1));
                    }
                    _ => {
                        parse_error!();
                    }
                },
                //                _ => panic!("unkown state {:?}", self.state),
            }
            i += 1;
        }
        Ok((ParseResult::NoMsg, 0))
    }
    //一种是消息体比较短,可以直接放在buf中,无需另外分配内存
    //另一种是消息体很长,无法放在buf中,额外分配了msg_buf空间
    fn add_msg(&mut self, b: u8) {
        if let Some(buf) = self.msg_buf.as_mut() {
            buf.push(b);
        } else {
            //消息体比较短的情况
            if self.arg_len + self.msg_total_len > BUF_LEN {
                panic!("message should allocate space");
            }
            self.buf[self.arg_len + self.msg_len] = b;
        }
        self.msg_len += 1;
    }
    fn add_arg(&mut self, b: u8) -> Result<()> {
        //太长的subject
        if self.arg_len >= self.buf.len() {
            parse_error!();
        }
        self.buf[self.arg_len] = b;
        self.arg_len += 1;
        Ok(())
    }
    //解析缓冲区中的形如stevenbai.top queue 3
    fn process_sub(&self) -> Result<ParseResult> {
        let buf = &self.buf[0..self.arg_len];
        //有可能客户端恶意发送一些无效的utf8字符,这会导致错误.
        let ss = unsafe { std::str::from_utf8_unchecked(buf) };
        let mut arg_buf = [""; 3]; //如果没有queue,长度就是2,否则长度是3
        let mut arg_len = 0;
        for s in ss.split(' ') {
            if s.len() == 0 {
                continue;
            }
            if arg_len >= 3 {
                parse_error!();
            }
            arg_buf[arg_len] = s;
            arg_len += 1;
        }
        let mut sub_arg = SubArg {
            subject: "",
            sid: "",
            queue: None,
        };
        sub_arg.subject = arg_buf[0];
        //长度为2时不包含queue,为3包含queue,其他都说明格式错误
        match arg_len {
            2 => {
                sub_arg.sid = arg_buf[1];
            }
            3 => {
                sub_arg.sid = arg_buf[2];
                sub_arg.queue = Some(arg_buf[1]);
            }
            _ => parse_error!(),
        }
        Ok(ParseResult::Sub(sub_arg))
    }
    //解析缓冲区中以及msg_buf中的形如stevenbai.top 5hello
    fn process_msg(&self) -> Result<ParseResult> {
        let msg = if self.msg_buf.is_some() {
            self.msg_buf.as_ref().unwrap().as_slice()
        } else {
            &self.buf[self.arg_len..self.arg_len + self.msg_total_len]
        };
        let mut arg_buf = [""; 2];
        let mut arg_len = 0;
        let ss = unsafe { std::str::from_utf8_unchecked(&self.buf[0..self.arg_len]) };
        for s in ss.split(' ') {
            if s.len() == 0 {
                continue;
            }
            if arg_len >= 2 {
                parse_error!()
            }
            arg_buf[arg_len] = s;
            arg_len += 1;
        }
        let pub_arg = PubArg {
            subject: arg_buf[0],
            size_buf: arg_buf[1],
            size: self.msg_total_len,
            msg,
        };
        Ok(ParseResult::Pub(pub_arg))
    }
    pub fn clear_msg_buf(&mut self) {
        self.msg_buf = None;
        self.msg_len = 0;
        self.msg_total_len = 0;
    }
    //从接收到的pub消息中提前解析出来消息的长度
    fn get_message_size(&self) -> Result<usize> {
        //缓冲区中形如top.stevenbai.top 5
        let arg_buf = &self.buf[0..self.arg_len];
        let pos = arg_buf
            .iter()
            .rev()
            .position(|b| *b == ' ' as u8 || *b == '\t' as u8);
        if pos.is_none() {
            parse_error!();
        }
        let pos = pos.unwrap();
        let size_buf = &arg_buf[arg_buf.len() - pos..];
        let szb = unsafe { std::str::from_utf8_unchecked(size_buf) };
        szb.parse::<usize>().map_err(|_| NError::new(ERROR_PARSE))
    }
}
```



