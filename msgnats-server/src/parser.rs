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
// **/  TODO
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
