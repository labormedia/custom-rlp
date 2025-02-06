use std::fmt::Debug;
use std::collections::VecDeque;
use crate::{
    traits::{
        self,
        EndianWrite,
    },
    error,
    macros::nest,
    macros::nested_list,
};

// RLPItem is the basic unit from which the algorith begins or ends its encoding/Decoding process
#[derive(Clone, Debug, PartialEq)]
pub enum RLPItem {
    Bytes(Box<[u8]>),
    List(Box<[RLPItem]>),
}

impl From<Box<[RLPItem]>> for RLPItem {
    fn from(value: Box<[RLPItem]>) -> Self {
        Self::List( value )
    }
}

impl From<&[RLPItem]> for RLPItem {
    fn from(value: &[RLPItem]) -> Self {
        let boxed: Box<[RLPItem]> = value.into();
        Self::List( boxed )
    }
}

impl<const C: usize> From<&[RLPItem; C]> for RLPItem {
    fn from(value: &[RLPItem; C]) -> Self {
        let boxed: Box<[RLPItem]> = value.clone().into();
        Self::List( boxed )
    }
}

impl<const C: usize> From<[RLPItem; C]> for RLPItem {
    fn from(value: [RLPItem; C]) -> Self {
        let boxed: Box<[RLPItem]> = value.into();
        Self::List( boxed )
    }
}

impl From<&[u8]> for RLPItem {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(Box::from(value))
    }
}

impl<const C: usize> From<[u8; C]> for RLPItem {
    fn from(value: [u8; C]) -> Self {
        Self::Bytes(Box::from(value))
    }
}

impl<const C: usize> TryFrom<[u32; C]> for RLPItem {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: [u32; C]) -> Result<Self, Self::Error> {
        let value_bytes: Vec<u8> = value
            .into_iter()
            .fold(Ok(Vec::new()), |acc, x| {
                match acc {
                    Ok(mut bytes) => {
                        match u8::try_from(x) {
                            Err(err) => Err(err),
                            Ok(byte) => {
                                bytes.push(byte);
                                Ok(bytes)
                            }
                        }
                    },
                    Err(err) => Err(err),
                }
            })?;
        Ok(Self::Bytes(Box::from(value_bytes)))
    }
}

impl From<&str> for RLPItem {
    fn from(value: &str) -> Self {
        Self::Bytes(Box::from(value.as_bytes()))
    }
}

impl From<&[&str]> for RLPItem {
    fn from(value: &[&str]) -> Self {
        let wrapped_values: Vec<RLPItem> = value
            .into_iter()
            .map(|x| {
                RLPItem::Bytes(Box::from(x.as_bytes()))
            })
            .collect();
        Self::List(Box::from(wrapped_values))
    }
}

impl<const C: usize> From<[&str; C]> for RLPItem {
    fn from(value: [&str; C]) -> Self {
        let wrapped_values: Vec<RLPItem> = value
            .into_iter()
            .map(|x| {
                RLPItem::Bytes(Box::from(x.as_bytes()))
            })
            .collect();
        Self::List(Box::from(wrapped_values))
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

impl RLPItem {
    fn to_be_bytes_with_total_traversal_length<'a>(&self, total_size_acc: &'a mut usize) -> (Box<[u8]>, &'a mut usize) {
        let to_box: Vec<u8> = match self {
            RLPItem::Bytes(bytes) => {
                let len = bytes.len();
                if len == 1 {
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
                        let items = list.into_iter().fold( Vec::new(), |mut acc: Vec<u8>, x: &RLPItem| { 
                        let (bytes, size) = x.to_be_bytes_with_total_traversal_length(total_size_acc); // this folding accumulates the total length of the encoding
                        acc.extend_from_slice(&*bytes); 
                        acc
                    });
                

                if len <= 55 {
                    let mut value = Vec::new();
                    *total_size_acc += len;
                    value.push(0xc0+*total_size_acc as u8);
                    value.extend_from_slice(&items);
                    value.into()                    
                } else {
                    let mut value = Vec::new();
                    let len_list = len
                        .to_be_bytes()
                        .into_iter()
                        .skip_while(|&byte| byte == 0)
                        .collect::<Vec<u8>>();
                    let len_list_len = len_list.len();
                    *total_size_acc += len_list_len;
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
        (Box::from(to_box.as_slice()), total_size_acc)
    }
}

impl traits::EndianWrite for RLPItem {
    type Output = Box<[u8]>;
    fn to_le_bytes(&self) -> Self::Output {
        unimplemented!()
    }
    fn to_be_bytes(&self) -> Self::Output {
        let mut total_size = 0;
        let (output, total_size) = self.to_be_bytes_with_total_traversal_length(&mut total_size);
        println!("Total size: {}", total_size);
        output
    }
}

#[test]
fn endianwrite_basic_case_empty_list() {
    use crate::traits::EndianWrite;
    use crate::RLPItem;
    use hex::ToHex;
    let sized_1: Box<[u8]> = RLPItem::from(&[]).to_be_bytes().into();
    assert_eq!(sized_1.encode_hex::<String>(), "c0");
}

#[test]
fn endianwrite_basic_case_empty_string() {
    use crate::traits::EndianWrite;
    use crate::RLPItem;
    use hex::ToHex;
    let value: &str = "";
    let sized: Box<[u8]> = RLPItem::from(value).to_be_bytes().into();
    assert_eq!(sized.encode_hex::<String>(), "80");
}

#[test]
fn endianwrite_basic_case_hello_world() {
    use crate::traits::EndianWrite;
    use crate::RLPItem;
    use hex::ToHex;
    let value: &str = "hello world";
    let sized: Box<[u8]> = RLPItem::from(value).to_be_bytes().into();
    assert_eq!(sized.encode_hex::<String>(), "8b68656c6c6f20776f726c64");
}

#[test]
fn endianwrite_basic_case_dog() {
    use crate::traits::EndianWrite;
    use hex::ToHex;
    let value = "dog";
    let sized: Box<[u8]> = RLPItem::from(value).to_be_bytes().into();
    assert_eq!(sized.encode_hex::<String>(), "83646f67");
}

#[test]
fn endianwrite_basic_case_lorem_ipsum() {
    use crate::traits::EndianWrite;
    use hex::ToHex;
    let value =  "Lorem ipsum dolor sit amet, consectetur adipisicing elit";
    let sized: Box<[u8]> = RLPItem::from(value).to_be_bytes().into();
    let first_three_bytes: &[u8] = &sized[0..=2];
    let last_byte: &[u8; 3] = sized.last_chunk::<3>().unwrap();
    assert_eq!(first_three_bytes.encode_hex::<String>(), "b8384c");
    assert_eq!(last_byte, b"lit");
}

#[test]
fn endianwrite_basic_case_1024() {
    use crate::traits::EndianWrite;
    use hex::ToHex;
    
    let sized_1024: Box<[u8]> = RLPItem::from(&[0; 1024][..]).to_be_bytes().into();
    let first_three_bytes: &[u8] = &sized_1024[0..=2];
    assert_eq!(first_three_bytes.encode_hex::<String>(), "b90400");
}

#[test]
fn from_nested_basic() {
    type R = RLPItem;
    let empty_item: [RLPItem; 0] = [];
    let nested_data: RLPItem = R::from([
        R::from(empty_item),
        R::from(["cat"]),
        "cat".into(),
        R::from([
            R::from([5_u8, 6_u8, 7_u8, 8_u8]),
            R::from([40, 50, 60]),
            R::from([
                R::from([100, 200, 255]),
                R::from([0, 1, 2, 3]),
            ]),
            R::from("squirrel"),
            R::from(["squirrel", "dog", "cat"])
        ]),
    ]);
    // at least one element explicitly converted without type inference is needed to progress with all Into instances
    let same_nested_data: RLPItem = [
        (&[]).into(),
        (["cat"]).into(),
        "cat".into(),
        R::from([
            [5_u8, 6_u8, 7_u8, 8_u8].into(),
            [40, 50, 60].into(),
            [
                [100, 200, 255].into(),
                R::from([0, 1, 2, 3]),
            ].into(),
            R::from("squirrel"),
            ["squirrel", "dog", "cat"].into(),
        ]),
    ].into();
    assert_eq!(nested_data, same_nested_data);
}

#[test]
fn set_theoretical_three_encoding() {
    use hex::ToHex;
    use crate::traits::EndianWrite;
    // Using a closure as a simple builder instead of cloning.
    let empty = || RLPItem::from(&[]);
    let value: RLPItem = [ 
        empty(), 
        [
            empty()
        ].into(), 
        [ 
            empty(), 
            [
                empty()
            ].into() 
        ].into() 
    ].into();
    //assert_eq!(value.to_be_bytes(), [ 0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0 ].into());
    assert_eq!(value.to_be_bytes().encode_hex::<String>(), "c7c0c1c0c3c0c1c0");
}