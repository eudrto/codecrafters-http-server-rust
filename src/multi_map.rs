use std::{borrow::Borrow, collections::HashMap, hash::Hash, slice::Iter};

use anyhow::bail;

#[derive(Debug, Eq, PartialEq)]
pub struct Value<T>(Vec<T>);
impl<T> Value<T> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, mut new: Self) {
        self.0.append(&mut new.0);
    }
}

impl<T> From<T> for Value<T> {
    fn from(scalar: T) -> Self {
        Value(vec![scalar])
    }
}

impl<T> From<Vec<T>> for Value<T> {
    fn from(vector: Vec<T>) -> Self {
        Value(vector)
    }
}

pub type ValueIter<'a, T> = Iter<'a, T>;

impl<'a, T> IntoIterator for &'a Value<T> {
    type Item = &'a T;
    type IntoIter = ValueIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug)]
pub struct MultiMap<K, T>(HashMap<K, Value<T>>);

impl<K, T> MultiMap<K, T>
where
    K: Eq + Hash,
{
    pub fn new_empty() -> Self {
        Self(HashMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, key: K, value: Value<T>) {
        if let Some(current) = self.0.get_mut(&key) {
            current.push(value);
        } else {
            self.0.insert(key, value);
        }
    }

    pub fn insert_scalar(&mut self, key: K, scalar: T) {
        self.insert(key, Value::from(scalar));
    }

    pub fn insert_vector(&mut self, key: K, vector: Vec<T>) {
        self.insert(key, Value::from(vector));
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Value<T>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.get(&key)
    }

    pub fn get_scalar<Q>(&self, key: &Q) -> anyhow::Result<Option<&T>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.0.get(key) {
            Some(value) => {
                assert_ne!(value.len(), 0);
                if value.len() == 1 {
                    Ok(Some(&value.0[0]))
                } else {
                    bail!("not scalar")
                }
            }
            None => Ok(None),
        }
    }

    pub fn get_vector<Q>(&self, key: &Q) -> anyhow::Result<Option<&[T]>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.0.get(key) {
            Some(value) => {
                assert_ne!(value.len(), 0);
                if value.len() != 1 {
                    Ok(Some(value.0.as_slice()))
                } else {
                    bail!("not vector")
                }
            }
            None => Ok(None),
        }
    }

    pub fn get_value_iter<'a, Q>(&'a self, key: &Q) -> Option<Iter<'_, T>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).map(|v| v.into_iter())
    }
}

impl<K, V> FromIterator<(K, Value<V>)> for MultiMap<K, V>
where
    K: Eq + Hash,
    V: Default,
{
    fn from_iter<T: IntoIterator<Item = (K, Value<V>)>>(iter: T) -> Self {
        let mut mm = Self::new_empty();
        for (k, v) in iter {
            mm.insert(k, v);
        }
        mm
    }
}

impl<'a, K, V> IntoIterator for &'a MultiMap<K, V> {
    type Item = <&'a HashMap<K, Value<V>> as IntoIterator>::Item;
    type IntoIter = <&'a HashMap<K, Value<V>> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
