use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, PartialEq, Eq, Debug)]
struct OpaqueIndex(Vec<u8>);

/// The largest byte which is _left_ of the magic byte.
const MAGIC_FLOOR: u8 = 0b01111111; // =127
const MAGIC_CEIL: u8 = 0b10000000; // =128

/// A [FractionByte] is the “conceptual” representation of a digit
/// of a [ZenoIndex]. A [ZenoIndex] is conceptually a finite number
/// of [FractionByte::Byte] instances followed by an infinite number
/// of [FractionByte::Magic] instances. Since we only need to store
/// the “regular” bytes, the underlying representation stores just the
/// raw `u8` values of the regular bytes. When it is indexed, it wraps
/// the result in a `Byte` if it exists and falls back on `Magic` if
/// not, which makes the comparison logic easier.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum FractionByte {
    /// A “special” byte which compares as if it were equal to 127.5.
    /// I.e., Byte(x) < Magic if x <= 127, otherwise Byte(x) > Magic.
    /// Byte(x) is never equal to Magic, but Magic = Magic.
    ///
    /// Th value 127.5 comes from the fact that the infinite sum
    /// of 127.5 * (1/256)^i over i=1..infinity equals 0.5, which is
    /// our desired default value. So a sequence of zero “regular”
    /// bytes followed by infinite “magic” bytes represents the
    /// fraction 0.5.
    Magic,

    /// A not-very-special byte.
    Byte(u8),
}

impl FractionByte {
    fn new_between_bytes(lhs: u8, rhs: u8) -> Option<FractionByte> {
        if lhs < rhs - 1 {
            Some(FractionByte::Byte((rhs - lhs) / 2 + lhs))
        } else {
            None
        }
    }

    fn new_between(lower_bound: FractionByte, upper_bound: FractionByte) -> Option<FractionByte> {
        match (lower_bound, upper_bound) {
            (FractionByte::Byte(lhs), FractionByte::Byte(rhs)) => {
                if lhs <= MAGIC_FLOOR && rhs >= MAGIC_CEIL {
                    Some(FractionByte::Magic)
                } else {
                    FractionByte::new_between_bytes(lhs, rhs)
                }
            }

            (FractionByte::Byte(lhs), FractionByte::Magic) => {
                FractionByte::new_between_bytes(lhs, MAGIC_CEIL)
            }

            (FractionByte::Magic, FractionByte::Byte(rhs)) => {
                FractionByte::new_between_bytes(MAGIC_FLOOR, rhs)
            }

            _ => None,
        }
    }
}

impl Default for FractionByte {
    fn default() -> Self {
        FractionByte::Magic
    }
}

impl PartialOrd for FractionByte {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FractionByte {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (FractionByte::Magic, FractionByte::Magic) => Ordering::Equal,
            (FractionByte::Byte(lhs), FractionByte::Magic) => {
                if *lhs <= MAGIC_FLOOR {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            (FractionByte::Magic, FractionByte::Byte(rhs)) => {
                if *rhs <= MAGIC_FLOOR {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (FractionByte::Byte(lhs), FractionByte::Byte(rhs)) => lhs.cmp(rhs),
        }
    }
}

/// A [ZenoIndex] is a binary representation of a fraction between 0 and 1,
/// exclusive, with arbitrary precision. The only operations it supports are:
///
/// - Construction of a [ZenoIndex] representing one half.
/// - Comparison of two [ZenoIndex] values.
/// - Returning an arbitrary [ZenoIndex] strictly between two other values.
/// - Returning an arbitrary [ZenoIndex] strictly between a given value and
///   either zero or one.
///
/// Note that as a result of these restrictions:
/// - It's possible to arrive at a value infinitely close, but not equal to,
///   zero or one ([hence the name](https://plato.stanford.edu/entries/paradox-zeno/)).
/// - We only ever care about the  _relative_ value of two [ZenoIndex]es; not
///   their actual value. In fact, the only reason to think about them as fractions
///   at all is because it makes them easier to reason about.
///
/// The use of fractional indexes for real-time editing of lists is described in
/// [this post](https://www.figma.com/blog/realtime-editing-of-ordered-sequences/).
/// The specifics of the encoding used in that post differ from the one we use.
///
/// The underlying data structure used by a ZenoIndex is a vector of bytes. The
/// fraction represented by a given vector of N bytes, where z<sub>i</sub> is the
/// i<sup>th</sup> byte (1-based indexing):
///
/// 0.5 * (1/256)^N + sum<sub>i=1..N</sub> (z_i * (1/256)^i)
///
///
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct ZenoIndex(Vec<u8>);

impl ZenoIndex {
    fn digit(&self, i: usize) -> FractionByte {
        self.0
            .get(i)
            .cloned()
            .map(FractionByte::Byte)
            .unwrap_or_default()
    }
}

impl ZenoIndex {
    pub fn new_before(fs: &ZenoIndex) -> ZenoIndex {
        for i in 0..fs.0.len() {
            if fs.0[i] > u8::MIN {
                let mut bytes: Vec<u8> = fs.0[0..(i + 1)].into();
                bytes[i] -= 1;
                return ZenoIndex(bytes);
            }
        }

        let mut bytes = fs.0.clone();
        bytes.push(MAGIC_FLOOR);
        ZenoIndex(bytes)
    }

    pub fn new_after(fs: &ZenoIndex) -> ZenoIndex {
        for i in 0..fs.0.len() {
            if fs.0[i] < u8::MAX {
                let mut bytes: Vec<u8> = fs.0[0..(i + 1)].into();
                bytes[i] += 1;
                return ZenoIndex(bytes);
            }
        }

        let mut bytes = fs.0.clone();
        bytes.push(MAGIC_CEIL);
        ZenoIndex(bytes)
    }

    pub fn new_between(left: &ZenoIndex, right: &ZenoIndex) -> ZenoIndex {
        for i in 0..=left.0.len() {
            // Find the first index at which left and right values differ.
            let ld = left.digit(i);
            let rd = right.digit(i);
            if ld < rd {
                return match FractionByte::new_between(ld, rd) {
                    Some(FractionByte::Magic) => ZenoIndex(left.0[0..(i - 1)].into()),
                    Some(FractionByte::Byte(b)) => {
                        let mut bytes: Vec<u8> = left.0[0..(i - 1)].into();
                        bytes.push(b);
                        ZenoIndex(bytes)
                    }
                    None => {
                        for j in (i + 1)..(left.0.len() + 1) {
                            match left.digit(j) {
                                FractionByte::Magic => {
                                    let mut bytes: Vec<u8> = left.0[0..j].into();
                                    bytes.push(MAGIC_CEIL);
                                    return ZenoIndex(bytes);
                                }
                                FractionByte::Byte(b) => {
                                    if b < u8::MAX {
                                        let mut bytes: Vec<u8> = left.0[0..j].into();
                                        bytes.push(b + 1);
                                        return ZenoIndex(bytes);
                                    }
                                }
                            }
                        }

                        for j in (i + 1)..(right.0.len() + 1) {
                            match right.digit(j) {
                                FractionByte::Magic => {
                                    let mut bytes: Vec<u8> = right.0[0..j].into();
                                    bytes.push(MAGIC_FLOOR);
                                    return ZenoIndex(bytes);
                                }
                                FractionByte::Byte(b) => {
                                    if b > u8::MIN {
                                        let mut bytes: Vec<u8> = right.0[0..j].into();
                                        bytes.push(b - 1);
                                        return ZenoIndex(bytes);
                                    }
                                }
                            }
                        }

                        panic!("Should never get here")
                    }
                };
            } else if ld > rd {
                panic!("left should be less than right.")
            }
        }
        panic!("Can't generate between two ZenoIndex values that are equal.")
    }
}

impl PartialOrd for ZenoIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ZenoIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..=self.0.len() {
            let sd = self.digit(i);
            let od = other.digit(i);
            if sd < od {
                return Ordering::Less;
            } else if sd > od {
                return Ordering::Greater;
            }
        }
        Ordering::Equal
    }
}

impl Default for ZenoIndex {
    fn default() -> Self {
        ZenoIndex(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeno_index() {
        let mut indices: Vec<ZenoIndex> = Vec::new();

        let c = ZenoIndex::default();

        {
            let mut m = c.clone();
            let mut low = Vec::new();
            for _ in 0..20 {
                m = ZenoIndex::new_before(&m);
                low.push(m.clone())
            }

            low.reverse();
            indices.append(&mut low)
        }

        indices.push(c.clone());

        {
            let mut m = c.clone();
            let mut high = Vec::new();
            for _ in 0..20 {
                m = ZenoIndex::new_after(&m);
                high.push(m.clone())
            }

            indices.append(&mut high)
        }

        for i in 0..(indices.len() - 1) {
            assert!(indices[i] < indices[i + 1])
        }

        for _ in 0..12 {
            let mut new_indices: Vec<ZenoIndex> = Vec::new();
            for i in 0..(indices.len() - 1) {
                let cb = ZenoIndex::new_between(&indices[i], &indices[i + 1]);
                assert!(&indices[i] < &cb);
                assert!(&cb < &indices[i + 1]);
                new_indices.push(cb);
                new_indices.push(indices[i + 1].clone());
            }

            indices = new_indices;
        }
    }

    fn byte(v: u8) -> FractionByte {
        FractionByte::Byte(v)
    }

    const MAGIC: FractionByte = FractionByte::Magic;

    #[test]
    fn test_fraction_byte_comparisons() {
        assert!(byte(0) < MAGIC);
        assert!(byte(255) > MAGIC);
        assert!(byte(127) < MAGIC);
        assert!(byte(128) > MAGIC);
        assert_eq!(MAGIC, MAGIC);
        assert_eq!(byte(128), byte(128));
        assert!(byte(8) < byte(9));
    }

    #[test]
    fn test_fraction_byte_new_between_bytes() {
        assert_eq!(Some(byte(8)), FractionByte::new_between_bytes(7, 9));
        assert_eq!(None, FractionByte::new_between_bytes(5, 6));
        assert_eq!(None, FractionByte::new_between_bytes(5, 5));
        assert_eq!(None, FractionByte::new_between_bytes(5, 4));
    }

    #[test]
    fn test_fraction_byte_new_between() {
        assert_eq!(Some(byte(8)), FractionByte::new_between(byte(7), byte(9)));
        assert_eq!(Some(MAGIC), FractionByte::new_between(byte(7), byte(192)));
        assert_eq!(Some(byte(67)), FractionByte::new_between(byte(7), MAGIC));
        assert_eq!(Some(byte(126)), FractionByte::new_between(byte(125), MAGIC));
        assert_eq!(Some(byte(127)), FractionByte::new_between(byte(126), MAGIC));
        assert_eq!(Some(byte(128)), FractionByte::new_between(MAGIC, byte(129)));
        assert_eq!(None, FractionByte::new_between(byte(127), MAGIC));
        assert_eq!(None, FractionByte::new_between(MAGIC, byte(128)));
        assert_eq!(Some(byte(191)), FractionByte::new_between(MAGIC, byte(255)));
        assert_eq!(None, FractionByte::new_between(MAGIC, MAGIC));
        assert_eq!(None, FractionByte::new_between(byte(191), byte(191)));
    }
}
