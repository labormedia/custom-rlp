use crate::{
    traits,
    error,
};

// RLPItem is the basic unit from which the algorith begins or ends its encoding/Decoding process
pub enum RLPItem {
    Bytes(Box<[u8]>),
    List(Box<[RLPItem]>),
}

impl From<&[u8]> for RLPItem {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(Box::from(value))
    }
}

impl TryInto<Box<[u8]>> for RLPItem {
    type Error = error::Error;
    fn try_into(self) -> Result<Box<[u8]>, Self::Error> {
        match self {
            Self::Bytes(value) => Ok(value),
            Self::List(value) => Err(error::Error::WrongType),
        }
    }
}

impl traits::EndianWrite for RLPItem {
    type Output = Box<[u8]>;
    fn to_le_bytes(&self) -> Self::Output {
        unimplemented!()
    }
    fn to_be_bytes(&self) -> Self::Output {
        let to_box: Vec<u8> = match self {
            RLPItem::Bytes(bytes) => {
                let len = bytes.len();
                if len == 0 {
                    [].into()
                } else if len == 1 {
                    bytes.to_vec()
                } else if len <= 55 {
                    let mut value = Vec::new();
                    value.push(0x80+len as u8);
                    value.extend_from_slice(&bytes);
                    value
                } else {
                    let mut value = Vec::new();
                    let len_bytes = len
                        .to_be_bytes()
                        .into_iter()
                        .skip_while(|&byte| byte == 0)
                        .collect::<Vec<u8>>();
                    let len_bytes_len = len_bytes.len();
                    value.extend_from_slice(&[0xb7+len_bytes_len as u8]);
                    value.extend_from_slice(&len_bytes);
                    value.extend_from_slice(&bytes);
                    value
                }
            },
            RLPItem::List(_) => {
                [].into()
            }
        };
        Box::from(to_box.as_slice())
    }
}

#[test]
fn endianwrite_basic_case_1024() {
    use crate::traits::EndianWrite;
    use hex::ToHex;
    
    let sized_1024: Box<[u8]> = RLPItem::from(&[0; 1024][..]).to_be_bytes().into();
    let first_three_bytes: &[u8] = &sized_1024[0..=2];
    assert_eq!(first_three_bytes.encode_hex::<String>(), "b90400");
}