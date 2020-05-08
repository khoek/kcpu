use itertools::iproduct;

pub fn accumulate_vecs<T, E>(it: impl Iterator<Item = Result<Vec<T>, E>>) -> Result<Vec<T>, E> {
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

pub fn slice_pairwise_unordered<T>(v: &[T]) -> impl Iterator<Item = (&T, &T)> {
    iproduct!(0..v.len(), 0..v.len())
        .filter(|(i, j)| i != j)
        .map(move |(i, j)| (&v[i], &v[j]))
}

pub fn slice_pairwise_ordered<T>(v: &[T]) -> impl Iterator<Item = (&T, &T)> {
    iproduct!(0..v.len(), 0..v.len())
        .filter(|(i, j)| j > i)
        .map(move |(i, j)| (&v[i], &v[j]))
}

pub fn unwrap_at_most_one<T>(mut it: impl Iterator<Item = T>) -> Option<T> {
    let t = it.next();
    assert!(it.next().is_none());
    t
}

pub fn unwrap_singleton<T>(it: impl Iterator<Item = T>) -> T {
    unwrap_at_most_one(it).unwrap()
}

pub fn find_is_unique<T, F>(it: impl Iterator<Item = T>, mut f: F) -> Option<(T, bool)>
where
    F: FnMut(&T) -> bool,
{
    let (candidate, found_multiple) = it.fold(
        (None, false),
        |(mut candidate, mut found_multiple), next| {
            if f(&next) {
                found_multiple = candidate.is_some();
                candidate = Some(next);
            }
            (candidate, found_multiple)
        },
    );

    candidate.map(|cand| (cand, !found_multiple))
}
