use std::collections::HashMap;

pub fn get_mut_or_put<'m, K, V, F>(map: &'m mut HashMap<K, V>, k: K, f: F) -> &'m mut V
where
    F: FnOnce() -> V,
    K: Eq + std::hash::Hash + Copy,
{
    if !map.contains_key(&k) {
        map.insert(k, f());
    }
    map.get_mut(&k).unwrap()
}