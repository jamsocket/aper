use std::cmp::Ordering;

#[derive(Clone, PartialEq, Eq, Debug)]
struct OpaqueIndex(Vec<u8>);

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
            // self is at least as long as other.
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
                if self.0[other.0.len()] == 1{
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
        }
    }
}

impl OpaqueIndex {
    pub fn new_after(other: &OpaqueIndex) -> OpaqueIndex {
        let mut v = other.0.clone();
        v.push(1);
        OpaqueIndex(
            v
        )
    }

    pub fn new_before(other: &OpaqueIndex) -> OpaqueIndex {
        let mut v = other.0.clone();
        v.push(0);
        OpaqueIndex(
            v
        )
    }

    pub fn new_between(lower_bound: &OpaqueIndex, upper_bound: &OpaqueIndex) -> OpaqueIndex {
        let v1 = &lower_bound.0;
        let v2 = &upper_bound.0;

        for i in 0..((v1.len()).min(v2.len())) {
            if v1[i] != v2[i] {
                let n: Vec<u8> = v1[0..i].iter().cloned().collect();
                return OpaqueIndex(n)
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