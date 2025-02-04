use crate::{
    traits,
    error,
};

// RLPItem is the basic unit from which the algorith begins or ends its encoding/Decoding process
pub enum RLPItem<T: traits::EndianWrite> {
    Bytes(Box<[u8]>),
    List(Box<[T]>),
}

impl<T: traits::EndianWrite> From<&[u8]> for RLPItem<T> {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(Box::from(value))
    }
}

impl<T: traits::EndianWrite> From<&str> for RLPItem<T> {
    fn from(value: &str) -> Self {
        Self::Bytes(Box::from(value.as_bytes()))
    }
}

impl<T: traits::EndianWrite> TryInto<Box<[u8]>> for RLPItem<T> {
    type Error = error::Error;
    fn try_into(self) -> Result<Box<[u8]>, Self::Error> {
        match self {
            Self::Bytes(value) => Ok(value),
            Self::List(value) => Err(error::Error::WrongType),
        }
    }
}

impl traits::EndianWrite for &str {
    type Output = Box<[u8]>;
    fn to_le_bytes(&self) -> Self::Output {
        unimplemented!()
    }
    fn to_be_bytes(&self) -> Self::Output {
        Box::from(self.as_bytes())
    }
}

impl<const C: usize> traits::EndianWrite for [u8;C] {
    type Output = Box<[u8]>;
    fn to_le_bytes(&self) -> Self::Output {
        unimplemented!()
    }
    fn to_be_bytes(&self) -> Self::Output {
        Box::from(*self)
    }
}

impl<T: traits::EndianWrite<Output = Box<[u8]>>> traits::EndianWrite for RLPItem<T> {
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
            RLPItem::List(list) => {
                let len = list.len();
                if len <= 55 {
                    let mut value = Vec::new();
                    value.push(0xc0+len as u8);
                    let items = list.into_iter().fold( Vec::new(), |mut acc, x| { 
                        acc.extend_from_slice(&*x.to_be_bytes());
                        acc
                    });
                    value.extend_from_slice(&items);
                    value                    
                } else {
                    let mut value = Vec::new();
                    let len_list = len
                        .to_be_bytes()
                        .into_iter()
                        .skip_while(|&byte| byte == 0)
                        .collect::<Vec<u8>>();
                    let len_list_len = len_list.len();
                    value.extend_from_slice(&[0xf7+len_list_len as u8]);
                    value.extend_from_slice(&len_list);
                    let items = list.into_iter().fold( Vec::new(), |mut acc, x| { 
                        acc.extend_from_slice(&*x.to_be_bytes());
                        acc
                    });
                    value.extend_from_slice(&items);
                    value                    
                }
            }
        };
        Box::from(to_box.as_slice())
    }
}

#[test]
fn endianwrite_basic_case_empty_list() {
    use crate::traits::EndianWrite;
    use hex::ToHex;
    let value: &str = "";
    let sized_1: Box<[u8]> = RLPItem::<&str>::from(value).to_be_bytes().into();
    assert_eq!(sized_1.encode_hex::<String>(), "c0");
}

#[test]
fn endianwrite_basic_case_dog() {
    use crate::traits::EndianWrite;
    use hex::ToHex;
    let value = "dog";
    let sized: Box<[u8]> = RLPItem::<&str>::from(value).to_be_bytes().into();
    assert_eq!(sized.encode_hex::<String>(), "83646f67");
}

#[test]
fn endianwrite_basic_case_lorem_ipsum() {
    use crate::traits::EndianWrite;
    use hex::ToHex;
    let value =  "Lorem ipsum dolor sit amet, consectetur adipisicing elit";
    let sized: Box<[u8]> = RLPItem::<&str>::from(value).to_be_bytes().into();
    let first_three_bytes: &[u8] = &sized[0..=2];
    let last_byte: &u8 = &sized.last().unwrap();
    assert_eq!(first_three_bytes.encode_hex::<String>(), "b8384c");
    assert_eq!(last_byte, &b't');
}

#[test]
fn endianwrite_basic_case_1024() {
    use crate::traits::EndianWrite;
    use hex::ToHex;
    
    let sized_1024: Box<[u8]> = RLPItem::<[u8; 1024]>::from(&[0; 1024][..]).to_be_bytes().into();
    let first_three_bytes: &[u8] = &sized_1024[0..=2];
    assert_eq!(first_three_bytes.encode_hex::<String>(), "b90400");
}