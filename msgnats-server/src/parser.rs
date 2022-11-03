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
