use itertools::iproduct;
use std::marker::PhantomData;

pub fn accumulate<T, E>(it: impl Iterator<Item = Result<Vec<T>, E>>) -> Result<Vec<T>, E> {
    let mut result = Vec::new();
    for ts in it {
        result.append(&mut ts?)
    }
    Ok(result)
}

pub fn eq_ignore_case(a: &str, b: &str) -> bool {
    a.chars()
        .map(std::primitive::char::to_lowercase)
        .flatten()
        .eq(b.chars().map(std::primitive::char::to_lowercase).flatten())
}

pub fn vec_pairwise_iter<T>(v: &Vec<T>) -> impl Iterator<Item = (&T, &T)> {
    iproduct!(0..v.len(), 0..v.len())
        .filter(|(i, j)| i != j)
        .map(move |(i, j)| (&v[i], &v[j]))
}

pub fn unwrap_singleton<T>(it: &mut impl Iterator<Item = T>) -> T {
    let t = it.next().unwrap();
    assert!(it.next().is_none());
    t
}

// RUSTFIX implement and use
// RUSTFIX can't see a way to make this genertic over the `U32`.
// pub struct U32FixedWith<Limit: typenum::Unsigned> {
//     val: u32,
//     phantom: PhantomData<Limit>,
// }

// impl<Limit: typenum::Unsigned> U32FixedWith<Limit> {
//     pub fn new(val: u32) -> Self {
//         assert!(Limit::to_u32() >= val);
//         Self { val, phantom: PhantomData }
//     }
// }
