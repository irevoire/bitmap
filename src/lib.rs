use core::fmt;

type Word = u64;

#[derive(Clone)]
pub struct Bitmap {
    len: usize,
    store: [Word; Self::BITMAP_SIZE],
}

impl Bitmap {
    const BITMAP_SIZE: usize = (u16::MAX as usize + 1) / Word::BITS as usize;

    #[inline]
    pub const fn new() -> Self {
        Bitmap {
            len: 0,
            store: [0; Self::BITMAP_SIZE],
        }
    }

    #[inline]
    pub const fn full() -> Self {
        Bitmap {
            len: u16::MAX as usize + 1,
            store: [Word::MAX; Self::BITMAP_SIZE],
        }
    }

    #[inline]
    pub fn internal_store(&self) -> &[Word; Self::BITMAP_SIZE] {
        &self.store
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn key(index: u16) -> usize {
        index as usize / Word::BITS as usize
    }

    #[inline]
    fn bit(index: u16) -> usize {
        index as usize % Word::BITS as usize
    }

    /// Returns `true` if the value was already present in the bitmap.
    #[inline]
    pub fn insert(&mut self, value: u16) -> bool {
        let (key, bit) = (Self::key(value), Self::bit(value));
        let old_w = self.store[key];
        let new_w = old_w | 1 << bit;
        let inserted = (old_w ^ new_w) >> bit;
        self.store[key] = new_w;
        self.len += inserted as usize;
        inserted != 0
    }

    /// Returns `true` if the value was already present in the bitmap.
    #[inline]
    pub fn remove(&mut self, value: u16) -> bool {
        let (key, bit) = (Self::key(value), Self::bit(value));
        let old_w = self.store[key];
        let new_w = old_w & !(1 << bit);
        let removed = (old_w ^ new_w) >> bit;
        self.store[key] = new_w;
        self.len -= removed as usize;
        removed != 0
    }

    /// Returns `true` if the value was present in the bitmap.
    #[inline]
    pub fn contains(&self, index: u16) -> bool {
        self.store[Self::key(index)] & (1 << Self::bit(index)) != 0
    }

    #[inline]
    pub fn intersection(&mut self, other: &Self) {
        let mut count = 0;
        for index in 0..self.store.len() {
            self.store[index] &= other.store[index];
            count += self.store[index].count_ones();
        }
        self.len = count as usize;
    }

    #[inline]
    pub fn intersection_simd(&mut self, other: &Self) {
        use core::arch::aarch64::*;

        let mut left = self.store.as_mut_ptr();
        let mut right = other.store.as_ptr();
        let mut count = 0;

        unsafe {
            for _ in 0..(Self::BITMAP_SIZE / 2) {
                // load the data into the register
                let left_lane = vld1q_u64(left);
                let right_lane = vld1q_u64(right);

                let ret = vandq_u64(left_lane, right_lane);
                vst1q_u64(left, ret);

                // update the count
                let p8_count = vcntq_u8(vreinterpretq_u8_u64(ret));
                let p8_count = vaddvq_u8(p8_count);
                count += p8_count as usize;

                // increase the ptr
                left = left.add(2);
                right = right.add(2);
            }
        }

        self.len = count;
    }

    pub fn to_vec(&self) -> Vec<u16> {
        let mut ret = Vec::with_capacity(self.len);
        let mut word = Vec::with_capacity(Word::BITS as usize);
        let mut current_idx = 0_u16;

        for mut current in self.store {
            if current.count_ones() != 0 {
                word.clear();
                for _ in (0..Word::BITS).rev() {
                    if current & 1 == 1 {
                        word.push(current_idx);
                    }
                    current >>= 1;
                    // When reaching the last byte this is going to overflow
                    // but it's probably not an issue since we're at the end
                    current_idx = current_idx.saturating_add(1);
                }
                ret.extend_from_slice(&word);
            } else {
                // this would panic if it was executed on the last word of the store
                // but we should always enter either in the previous if, or the
                // next one in the previous iteration of the loop.
                current_idx += Word::BITS as u16;
            }
            if ret.len() == self.len {
                break;
            }
        }
        ret
    }
}

impl FromIterator<u16> for Bitmap {
    fn from_iter<T: IntoIterator<Item = u16>>(iter: T) -> Self {
        let mut bitmap = Bitmap::new();
        iter.into_iter().for_each(|value| {
            bitmap.insert(value);
        });
        bitmap
    }
}

impl<'a> FromIterator<&'a u16> for Bitmap {
    fn from_iter<T: IntoIterator<Item = &'a u16>>(iter: T) -> Self {
        let mut bitmap = Bitmap::new();
        iter.into_iter().copied().for_each(|value| {
            bitmap.insert(value);
        });
        bitmap
    }
}

impl PartialEq for Bitmap {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.internal_store() == other.internal_store()
    }
}

impl std::ops::BitOr<&Bitmap> for Bitmap {
    type Output = Bitmap;

    fn bitor(mut self, rhs: &Self) -> Self::Output {
        let mut count = 0;
        for index in 0..self.store.len() {
            self.store[index] |= rhs.store[index];
            count += self.store[index].count_ones();
        }
        self.len = count as usize;
        self
    }
}

impl std::ops::BitOr for Bitmap {
    type Output = Bitmap;

    fn bitor(self, rhs: Self) -> Self::Output {
        self | &rhs
    }
}

impl std::ops::BitAnd<&Bitmap> for Bitmap {
    type Output = Bitmap;

    fn bitand(mut self, rhs: &Self) -> Self::Output {
        self.intersection(rhs);
        self
    }
}

impl std::ops::BitAnd for Bitmap {
    type Output = Bitmap;

    fn bitand(self, rhs: Self) -> Self::Output {
        self & &rhs
    }
}

impl Default for Bitmap {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(&self.to_vec()).finish()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use proptest::prelude::*;

    #[test]
    fn insert() {
        let mut bitmap = Bitmap::new();
        bitmap.insert(32);
        bitmap.insert(33);
        bitmap.insert(36);

        insta::assert_debug_snapshot!(bitmap.len(), @"3");
        insta::assert_debug_snapshot!(bitmap, @r###"
        {
            32,
            33,
            36,
        }
        "###);
    }

    #[test]
    fn insert_zero() {
        let mut bitmap = Bitmap::new();
        bitmap.insert(0);
        bitmap.insert(33);
        bitmap.insert(36);

        insta::assert_debug_snapshot!(bitmap.len(), @"3");
        insta::assert_debug_snapshot!(bitmap, @r###"
        {
            0,
            33,
            36,
        }
        "###);
    }

    #[test]
    fn insert_max() {
        let mut bitmap = Bitmap::new();
        bitmap.insert(u16::MAX);
        bitmap.insert(33);
        bitmap.insert(36);

        insta::assert_debug_snapshot!(bitmap.len(), @"3");
        insta::assert_debug_snapshot!(bitmap, @r###"
        {
            33,
            36,
            65535,
        }
        "###);
    }

    #[test]
    fn full_size() {
        let orig = Bitmap::full();
        assert_eq!(orig.len(), u16::MAX as usize + 1);
        let mut other = Bitmap::full();
        other.intersection(&orig);
        assert_eq!(orig, other);

        other.intersection_simd(&orig);
        assert_eq!(orig, other);
    }

    #[test]
    fn contains() {
        let mut bitmap = Bitmap::new();
        bitmap.insert(32);
        bitmap.insert(33);
        bitmap.insert(34);

        insta::assert_debug_snapshot!(bitmap.len(), @"3");
        insta::assert_debug_snapshot!(bitmap.contains(33), @"true");
        insta::assert_debug_snapshot!(bitmap.contains(3100), @"false");
    }

    #[test]
    fn and() {
        let left = Bitmap::from_iter((0..10).step_by(2).chain(10..15));
        let right = Bitmap::from_iter((1..10).step_by(2).chain(10..15));
        let ret = left.clone() & &right;

        insta::assert_debug_snapshot!(ret.len(), @"5");
        insta::assert_debug_snapshot!(ret, @r###"
        {
            10,
            11,
            12,
            13,
            14,
        }
        "###);

        let mut simd = left.clone();
        simd.intersection_simd(&right);
        assert_eq!(ret.store, simd.store);
        insta::assert_debug_snapshot!(simd.len(), @"5");
        insta::assert_debug_snapshot!(simd, @r###"
        {
            10,
            11,
            12,
            13,
            14,
        }
        "###);
    }

    #[test]
    fn and_max() {
        let left = Bitmap::full();
        let right = Bitmap::full();
        let ret = left.clone() & &right;

        let mut simd = left.clone();
        simd.intersection_simd(&right);
        assert_eq!(ret.len, simd.len);
        assert_eq!(ret.store, simd.store);
    }

    #[test]
    fn bug_1() {
        let left = Bitmap::from_iter(Some(128));
        let right = Bitmap::from_iter(Some(0));
        let ret = left.clone() & &right;

        insta::assert_debug_snapshot!(ret.len(), @"0");
        insta::assert_debug_snapshot!(ret, @"{}");

        let mut simd = left.clone();
        simd.intersection_simd(&right);
        insta::assert_debug_snapshot!(simd.len(), @"0");
        insta::assert_debug_snapshot!(simd, @"{}");

        // Check the actual store without going through the Debug implementation
        assert_eq!(ret.store, simd.store);
    }

    #[test]
    fn or() {
        let left = Bitmap::from_iter((0..10).step_by(2).chain(10..15));
        let right = Bitmap::from_iter((1..10).step_by(2).chain(10..15));
        let ret = left | right;

        insta::assert_debug_snapshot!(ret.len(), @"15");
        insta::assert_debug_snapshot!(ret, @r###"
        {
            0,
            1,
            2,
            3,
            4,
            5,
            6,
            7,
            8,
            9,
            10,
            11,
            12,
            13,
            14,
        }
        "###);
    }

    proptest! {
        #[test]
        fn from_iter_and_insert_are_equivalent(indexes in prop::collection::vec(0..=u16::MAX, 1..150)) {
            let mut left = Bitmap::new();
            for i in &indexes {
                left.insert(*i);
            }
            let right = Bitmap::from_iter(&indexes);
            assert_eq!(left, right);
        }

        #[test]
        fn prop_and(left in prop::collection::vec(0..=u16::MAX, 1..150), right in prop::collection::vec(0..=u16::MAX, 1..150)) {
            let bleft = Bitmap::from_iter(&left);
            let bright = Bitmap::from_iter(&right);
            let bitmap = bleft & bright;

            let hleft: HashSet<&u16> = HashSet::from_iter(&left);
            let hright = HashSet::from_iter(&right);
            let mut hashset: Vec<_> = hleft.intersection(&hright).copied().copied().collect();
            hashset.sort_unstable();

            assert_eq!(bitmap.to_vec(), hashset);
        }

        #[test]
        fn prop_simd_and(left in prop::collection::vec(0..=u16::MAX, 1..150), right in prop::collection::vec(0..=u16::MAX, 1..150)) {
            let bleft = Bitmap::from_iter(&left);
            let bright = Bitmap::from_iter(&right);
            let classic = bleft.clone() & &bright;
            let mut simd = bleft.clone();
            simd.intersection_simd(&bright);

            assert_eq!(classic.len(), simd.len());
            assert_eq!(classic, simd, "\nclassic:\n{classic:?}\nsimd:\n{simd:?}");
        }

        #[test]
        fn prop_or(left in prop::collection::vec(0..=u16::MAX, 1..150), right in prop::collection::vec(0..=u16::MAX, 1..150)) {
            let bleft = Bitmap::from_iter(&left);
            let bright = Bitmap::from_iter(&right);
            let bitmap = bleft | bright;

            let hleft: HashSet<&u16> = HashSet::from_iter(&left);
            let hright = HashSet::from_iter(&right);
            let mut hashset: Vec<_> = hleft.union(&hright).copied().copied().collect();
            hashset.sort_unstable();

            assert_eq!(bitmap.to_vec(), hashset);
        }

    }

    // These tests are too slow to be ran multiple times. But even by executing them only once, if there is a bug they'll end up by find it over time.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1))]

        #[test]
        fn insert_contains_delete(mut indexes in prop::collection::vec(0..=u16::MAX, 1..150)) {
            let mut bitmap = Bitmap::new();
            for i in &indexes {
                bitmap.insert(*i);
            }
            indexes.sort_unstable();
            indexes.dedup();
            assert_eq!(bitmap.len(), indexes.len());
            for i in 0..indexes.len() as u16 {
                let contain = bitmap.contains(i);
                assert_eq!(contain, indexes.contains(&i));
                assert_eq!(bitmap.remove(i), contain);
            }
        }
    }
}
