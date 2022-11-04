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

use crate::errors::Result;


#[derive(Debug,Clone)]
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
    OpMsg,  //pub message
    OpMsgFull,
}

// 解析结果定义
#[derive(Debug,PartialEq)]
pub struct SubArg<'a> {
    pub subject: &'a str, // 'a str 避免内存复制
    pub sid: &'a str,
    pub queue: Option<&'a str>, 
}
#[derive(Debug,PartialEq)]
pub struct PubArg<'a> {
    pub subject: &'a str,
    pub size_buf: &'a str, // str字符串切片形式避免内存复制
    pub size: usize,
    pub msg: &'a [u8],
}
#[derive(Debug,PartialEq)]
pub enum ParseResult<'a> {
    NoMsg,  // buf="sub top.stevenbai.blog" sub消息格式不完整
    SubArg(SubArg<'a>),
    PubArg(PubArg<'a>),
}

// 解析器数据结构定义
const  DEFAULT_BUF_LEN: usize= 512; // 默认解析缓冲区大小为512

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
            buf: [0;DEFAULT_BUF_LEN],
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
    pub fn parse(&mut self,buf: &[u8]) -> Result<(ParseResult,usize)> {
        // 定义字节数据接收变量
        let mut b;
        // buf字节序列循环变量
        let mut i = 0;
        // 打印 debug日志
        if self.debug {
            print!(
                "parse string:{},state:{:?}",
                unsafe {
                    std::str::from_utf8_unchecked(buf)
                },
                self.state
            )
        }
        while i< buf.len() {
            use ParseState::*;
            b = buf[i] as char;

            // 根据状态匹配处理分支
            match self.state {
                OpStart => match b {
                    'S' => self.state = OpS,
                    'P' => self.state = OpP,
                     _  => todo!(),
                },
                OpS => todo!(),
                OpSu => todo!(),
                OpSub => todo!(),
                OpSubSpace => todo!(),
                OpSubArg => todo!(),
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

}