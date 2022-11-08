// /**
// * ## pub
//  * ```
//  * PUB <subject> <size>\r\n
//  * <message>\r\n
//  * ```
//  * ## sub
//  * ```
//  * SUB <subject> <sid>\r\n
//  * SUB <subject> <queue> <sid>\r\n
//  * ```
//  * ## MSG
//  * ```
//  * MSG <subject> <sid> <size>\r\n
//  * <message>\r\n
// *
// **/
use std::string::ParseError;

use crate::errors::{NError, Result, ERROR_PARSE};

// 定义错误宏
#[macro_export]
macro_rules! parse_error {
    () => {
        return Err(NError::new(ERROR_PARSE))
    };
}

#[derive(Debug, Clone)]
enum ParseState {
    OpStart,
    OpS,
    OpSu,
    OpSub,
    OpSubSpace,
    OpSubArg,
    OpP,
    OpPu,
    OpPub, //pub argument
    OpPubSpace,
    OpPubArg,
    OpMsg, //pub message
    OpMsgFull,
}

// 解析结果定义
#[derive(Debug, PartialEq)]
pub struct SubArg<'a> {
    pub subject: &'a str, // 'a str 避免内存复制
    pub sid: &'a str,
    pub queue: Option<&'a str>,
}
#[derive(Debug, PartialEq)]
pub struct PubArg<'a> {
    pub subject: &'a str,
    pub size_buf: &'a str, // str字符串切片形式避免内存复制
    pub size: usize,
    pub msg: &'a [u8],
}
#[derive(Debug, PartialEq)]
pub enum ParseResult<'a> {
    NoMsg, // buf="sub top.stevenbai.blog" sub消息格式不完整
    SubArg(SubArg<'a>),
    PubArg(PubArg<'a>),
}

// 解析器数据结构定义
const DEFAULT_BUF_LEN: usize = 512; // 默认解析缓冲区大小为512

pub struct Parser {
    state: ParseState,
    buf: [u8; DEFAULT_BUF_LEN], // 默认缓冲区大小512 超过另行分配使用msg_buf
    arg_len: usize,
    msg_buf: Option<Vec<u8>>,
    //解析过程中收到新消息,那么 新消息的总长度是msg_total_len,已收到部分应该是msg_len
    msg_total_len: usize,
    msg_len: usize,
    debug: bool,
}

// TODO：解析器实现

impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParseState::OpStart,
            buf: [0; DEFAULT_BUF_LEN],
            arg_len: 0,
            msg_buf: None,
            msg_total_len: 0,
            msg_len: 0,
            debug: false,
        }
    }

    // /**
    //  * 解析业务逻辑实现
    //  * 对收到的字节序列进行解析,解析完毕后得到pub或者sub消息,
    //  * 同时有可能没有消息或者缓冲区里面还有其他消息
    //  */
    pub fn parse(&mut self, buf: &[u8]) -> Result<(ParseResult, usize)> {
        // 定义字节数据接收变量
        let mut b;
        // buf字节序列循环变量
        let mut i = 0;
        // 打印 debug日志
        if self.debug {
            print!(
                "parse string:{},state:{:?}",
                unsafe { std::str::from_utf8_unchecked(buf) },
                self.state
            )
        }
        while i < buf.len() {
            use ParseState::*;
            b = buf[i] as char;

            // 根据状态匹配处理分支
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
                    ' ' | '\t' => self.state = OpSubSpace,
                    _ => parse_error!(),
                },
                OpSubSpace => match b {
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
                        //PUB top.stevenbai 5\r\n
                    }
                    _ => {
                        todo!()
                        // self.add_arg(b as u8)?;
                    }
                },
                OpP => todo!(),
                OpPu => todo!(),
                OpPub => todo!(),
                OpPubSpace => todo!(),
                OpPubArg => todo!(),
                OpMsg => todo!(),
                OpMsgFull => todo!(),
            }
        }

        todo!()
    }
    //一种是消息体比较短,可以直接放在buf中,无需另外分配内存
    //另一种是消息体很长,无法放在buf中,额外分配了msg_buf空间
    fn add_msg(&mut self, b: u8) {
        if let Some(buf) = self.msg_buf.as_mut() {
            buf.push(b);
        } else {
            // 如果消息体比较短
            if self.arg_len + self.msg_total_len > DEFAULT_BUF_LEN {
                panic!("message should allocate space");
            }
            self.buf[self.arg_len + self.msg_len] = b;
        }
        self.msg_len += 1;
    }

    fn add_arg(&mut self, b: u8) -> Result<()> {
        // 太长的subject
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
        Ok(ParseResult::SubArg(sub_arg))
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
        let sie_buf = &arg_buf[arg_buf.len() - pos..];
        let szb = unsafe { std::str::from_utf8_unchecked(sie_buf) };
        szb.parse::<usize>().map_err(|_| NError::new(ERROR_PARSE))
    }
}

// 自定义Parser的迭代器
pub struct ParseIter<'a> {
    parser: *mut Parser,
    buf: &'a [u8],
}

impl<'a> Iterator for ParseIter<'a> {
    type Item = Result<ParseResult<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() == 0 {
            return None;
        }
        /*
        对于外部使用这类来说,这里使用unsafe是安全的.
        首先,ParseIter<'a>的生命周期一定是小于self.parser,也就是说parser这个指针一定是有效的.
        其次,ParseIter的构造只能通过Parser.iter来构造,所以parser一定是mutable的
        所以不存在内存安全问题.
        */
        let parser = unsafe { &mut *self.parser };

        let r: Result<(ParseResult<'a>, usize)> = parser.parse(self.buf);

        return Some(r.map(|r| {
            self.buf = &self.buf[r.1..];
            r.0
        }));
    }
}