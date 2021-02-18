use std::cmp::Ordering;

#[derive(Clone, PartialEq, Eq, Debug)]
struct OpaqueIndex(Vec<u8>);

const MIDPOINT: u8 = 0b10000000;

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
#[derive(PartialEq, Eq)]
struct ZenoIndex(Vec<u8>);

impl ZenoIndex {
    fn get_val(&self, i: usize) -> u8 {
        if i < self.0.len() {
            self[i]
        } else {
            MIDPOINT
        }
    }
}

impl ZenoIndex {
    fn new_before(fs: &ZenoIndex) -> ZenoIndex {
        unimplemented!()
    }
}

impl PartialOrd for ZenoIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ZenoIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..self.0.len() {
            if self.0[i] < other.0[i] {
                Ordering::Less
            } else if self.0[i] > other.0[i] {
                Ordering::Greater
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
    fn test_frac_string() {

    }
}

/*
const LEFT_ROOT: u8 = 0b01000000;
const RIGHT_ROOT: u8 = 0b11000000;
const RIGHT_MASK: u8 = 0b10000000;

impl PartialOrd for OpaqueIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OpaqueIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        if other.0.len() > self.0.len() {
            other.cmp(&self).reverse()
        } else {
            // Within this block, we are guaranteed that
            // self.0 is at least as long as other.0.
            for i in 0..other.0.len() {
                if self.0[i] > other.0[i] {
                    return Ordering::Greater
                } else if self.0[i] < other.0[i] {
                    return Ordering::Less
                }
            }
            if self.0.len() == other.0.len() {
                return Ordering::Equal
            } else {
                if self.0[other.0.len()] == 1 {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
        }
    }
}

impl OpaqueIndex {
    pub fn new_before(other: &OpaqueIndex) -> OpaqueIndex {
        let mut v = other.0.clone();
        if *v.last().unwrap() > u8::MIN {
            *v.last_mut() -= 1;
        } else {
            v.push(LEFT_ROOT);
        }
        OpaqueIndex(v)
    }

    pub fn new_after(other: &OpaqueIndex) -> OpaqueIndex {
        let mut v = other.0.clone();
        if *v.last().unwrap() > u8::MAX {
            *v.last_mut() += 1;
        } else {
            v.push(RIGHT_ROOT);
        }
        OpaqueIndex(v)
    }

    pub fn new_between(lower_bound: &OpaqueIndex, upper_bound: &OpaqueIndex) -> OpaqueIndex {
        let v1 = &lower_bound.0;
        let v2 = &upper_bound.0;

        // Try to find a common root.
        for i in 0..((v1.len()).min(v2.len())) {
            if v1[i] != v2[i] {
                if v2[i] > v1[i] + 1 {
                    // Find the middle.
                } else {
                    // Nodes are adjacent; extend.

                }
            }
        }

        if v1.len() < v2.len() {
            let mut v = v1.clone();
            v.push(1);
            return OpaqueIndex(v);
        } else {
            let mut v = v2.clone();
            v.push(0);
            return OpaqueIndex(v);
        }
    }
}

impl Default for OpaqueIndex {
    fn default() -> Self {
        OpaqueIndex(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order() {
        let p5 = OpaqueIndex::default();
        println!("p5 {:?}", &p5);

        let p10 = OpaqueIndex::new_after(&p5);

        let p1 = OpaqueIndex::new_before(&p5);
        println!("p1 {:?}", &p1);

        let p8 = OpaqueIndex::new_between(&p5, &p10);

        assert!(p1 < p5);
        assert!(p5 < p10);
    }
}
 */