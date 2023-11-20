use crate::constant::{CARRY_BIT, FOUR_BIT, LEAF_FLAG};
use std::cmp::min;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Nibble {
    data: Vec<u8>,
}

impl Nibble {
    pub fn from_raw(raw: Vec<u8>, is_leaf: bool) -> Nibble {
        let mut data = vec![];
        for byte in raw.into_iter() {
            let high_half = byte >> FOUR_BIT;
            let low_half = byte % CARRY_BIT;
            data.push(high_half);
            data.push(low_half);
        }
        if is_leaf {
            data.push(LEAF_FLAG as u8);
        }
        Nibble { data }
    }

    pub fn match_len(&self, other: &Nibble) -> usize {
        let len = min(self.len(), other.len());
        let mut i = 0usize;
        while i < len {
            if self.value_at(i) != other.value_at(i) {
                break;
            }
            i += 1;
        }
        i
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn value_at(&self, index: usize) -> usize {
        self.data[index] as usize
    }

    pub fn slice_from(&self, index: usize) -> Nibble {
        self.sub_slice(index, self.len())
    }

    pub fn sub_slice(&self, from: usize, to: usize) -> Nibble {
        Nibble {
            data: self.data[from..to].to_vec(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
