use std::{borrow::Borrow, collections::HashMap, hash::Hash, mem};

use anyhow::bail;

#[derive(Debug, Eq, PartialEq)]
pub enum Value<T> {
    Scalar(T),
    Vector(Vec<T>),
}

impl<T: Default> Value<T> {
    fn push(&mut self, new: Self) {
        if let Self::Vector(inner) = self {
            match new {
                Value::Scalar(new) => inner.push(new),
                Value::Vector(mut new) => inner.append(&mut new),
            }
            return;
        };

        if let Self::Scalar(inner) = mem::take(self) {
            match new {
                Value::Scalar(new) => *self = Value::Vector(vec![inner, new]),
                Value::Vector(mut new) => {
                    new.insert(0, inner);
                    *self = Value::Vector(new);
                }
            }
        }
    }
}

impl<T: Default> Default for Value<T> {
    fn default() -> Self {
        Value::Scalar(T::default())
    }
}

impl<T> From<T> for Value<T> {
    fn from(value: T) -> Self {
        Value::Scalar(value)
    }
}

impl<T> From<Vec<T>> for Value<T> {
    fn from(value: Vec<T>) -> Self {
        Value::Vector(value)
    }
}

impl<'a, T> IntoIterator for &'a Value<T> {
    type Item = &'a T;
    type IntoIter = ValueIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        ValueIter::new(self)
    }
}

pub struct ValueIter<'a, T> {
    value: &'a Value<T>,
    idx: usize,
}

impl<'a, T> ValueIter<'a, T> {
    fn new(value: &'a Value<T>) -> Self {
        ValueIter { value, idx: 0 }
    }
}

impl<'a, T> Iterator for ValueIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.value {
            Value::Scalar(scalar) => {
                if self.idx == 0 {
                    Some(scalar)
                } else {
                    None
                }
            }
            Value::Vector(vector) => vector.get(self.idx),
        };
        self.idx += 1;
        res
    }
}

#[derive(Debug)]
pub struct MultiMap<K, T>(HashMap<K, Value<T>>);

impl<K, T> MultiMap<K, T>
where
    K: Eq + Hash,
    T: Default,
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
            Some(Value::Scalar(scalar)) => Ok(Some(scalar)),
            Some(Value::Vector(_)) => bail!("not scalar"),
            None => Ok(None),
        }
    }

    pub fn get_vector<Q>(&self, key: &Q) -> anyhow::Result<Option<&[T]>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.0.get(key) {
            Some(Value::Scalar(_)) => bail!("not vector"),
            Some(Value::Vector(vector)) => Ok(Some(vector)),
            None => Ok(None),
        }
    }

    pub fn get_value_iter<'a, Q>(&'a self, key: &Q) -> Option<ValueIter<'a, T>>
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
