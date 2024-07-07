use super::{
    extract_args, validate_command, validate_multi_arg_command, CommandExecutor, SAdd, SIsMember,
};
use crate::{cmd::CommandError, RespArray, RespFrame};

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let count = backend.sadd(self.key, self.members);
        RespFrame::Integer(count as i64).into()
    }
}

impl CommandExecutor for SIsMember {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        match backend.sismember(&self.key, &self.member) {
            Some(v) => (if v { 1 } else { 0 }).into(),
            None => 0.into()
        }
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_multi_arg_command(&value, &["sadd"], 3)?;
        let mut result = SAdd {
            key: String::new(),
            members: Vec::new(),
        };
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => result.key = String::from_utf8(key.data)?,
            _ => return Err(CommandError::InvalidArgument("Invalid key".to_string())),
        };

        while let Some(RespFrame::BulkString(member)) = args.next() {
            result.members.push(String::from_utf8(member.data)?);
        }

        Ok(result)
    }
}

impl TryFrom<RespArray> for SIsMember {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(member))) => {
                Ok(SIsMember {
                    key: String::from_utf8(key.data)?,
                    member: String::from_utf8(member.data)?,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid arguments".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespDecode;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_sadd_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nsadd\r\n$3\r\nkey\r\n$7\r\nmember1\r\n$7\r\nmember2\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result : SAdd = frame.try_into()?;
        assert_eq!(result.key, "key");
        assert_eq!(result.members, vec!["member1".to_string(), "member2".to_string()]);
        Ok(())
    }

    #[test]
    fn test_sismember_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$9\r\nsismember\r\n$3\r\nkey\r\n$7\r\nmember1\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result : SIsMember = frame.try_into()?;
        assert_eq!(result.key, "key");
        assert_eq!(result.member, "member1");
        Ok(())
    }

    #[test]
    fn test_sadd() {
        let backend = crate::Backend::new();
        let cmd = SAdd {
            key: "key".to_string(),
            members: vec!["member1".to_string(), "member2".to_string()],
        };
        let frame = cmd.execute(&backend);
        assert_eq!(frame, RespFrame::Integer(2));

        let cmd = SIsMember {
            key: "key".to_string(),
            member: "member".to_string(),
        };
        let frame = cmd.execute(&backend);
        assert_eq!(frame, RespFrame::Integer(0));

        let cmd = SIsMember {
            key: "key".to_string(),
            member: "member1".to_string(),
        };
        let frame = cmd.execute(&backend);
        assert_eq!(frame, RespFrame::Integer(1));
    }
}
