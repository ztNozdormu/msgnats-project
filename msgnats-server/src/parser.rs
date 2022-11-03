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
