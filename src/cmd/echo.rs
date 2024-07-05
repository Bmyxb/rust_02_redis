use super::{extract_args, validate_command, CommandExecutor, Echo};
use crate::{cmd::CommandError, BulkString, RespArray, RespFrame};

impl CommandExecutor for Echo {
    fn execute(self, _backend: &crate::Backend) -> RespFrame {
        BulkString::new(self.msg).into()
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        validate_command(&v, &["echo"], 1)?;
        let mut args = extract_args(v, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(msg)) => Ok(Echo {
                msg: String::from_utf8(msg.0)?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::RespDecode;

    use super::*;

    use bytes::BytesMut;

    #[test]
    fn test_echo() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf).unwrap();
        let echo = Echo::try_from(frame).unwrap();
        assert_eq!(echo.msg, "hello");
    }
}
