// Macro to construct a nested list structure
macro_rules! nest {
    // Base case: Convert a byte slice (e.g., b"cat") into an ElementBytes variant
    ($elem:expr) => {
        RLPItem::Bytes(Box::new($elem.into()))
    };

    // Special case: Convert an empty list `[]` into an empty NestedList::List
    ([]) => {
        RLPItem::List(vec![])
    };

    // Recursive case: Convert multiple elements into a NestedList::List
    ($($elems:tt),+) => {
        RLPItem::List(vec![$(nest!($elems)),+].into())
    };
}

// Macro to construct a NestedList from bracket syntax
macro_rules! nested_list {
    // Base case: Convert a byte array (e.g., `[1, 2, 3, 4]`) into an ElementBytes variant
    ([ $($elem:expr),* ]) => {
        RLPItem::Bytes(Box::new([$($elem as u8),*])) // Ensure all elements are `u8`
    };

    // Recursive case: Convert nested brackets into a List variant
    ( $($elems:tt),* ) => {
        RLPItem::List(vec![$(nested_list!($elems)),*])
    };
}

pub(crate) use nest;
pub(crate) use nested_list;