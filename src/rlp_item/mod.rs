use crate::traits;

// RLPItem is the basic unit from which the algorith begins or ends its encoding/Decoding process
pub enum RLPItem {
    Bytes(Box<[u8]>),
    List(Box<[RLPItem]>),
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
                    let len_len = len.to_be_bytes().len();
                    value.extend_from_slice(&(0xb7+len_len).to_be_bytes());
                    value.extend_from_slice(&len.to_be_bytes());
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