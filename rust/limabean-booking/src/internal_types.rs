use hashbrown::{HashMap, hash_map::Entry};
use std::{fmt::Debug, hash::Hash, ops::Deref};

use super::{BookingTypes, CostSpec, Interpolated, Number, PostingSpec, PriceSpec};

#[derive(Debug)]
pub(crate) struct HashMapOfVec<K, V>(HashMap<K, Vec<V>>);

impl<K, V> HashMapOfVec<K, V> {
    pub(crate) fn push_or_insert(&mut self, k: K, v: V)
    where
        K: Eq + Hash,
    {
        use Entry::*;

        match self.0.entry(k) {
            Occupied(mut occupied) => {
                occupied.get_mut().push(v);
            }
            Vacant(vacant) => {
                vacant.insert(vec![v]);
            }
        }
    }
}

impl<K, V> Default for HashMapOfVec<K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, V> IntoIterator for HashMapOfVec<K, V> {
    type Item = (K, Vec<V>);
    type IntoIter = hashbrown::hash_map::IntoIter<K, Vec<V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, V> Deref for HashMapOfVec<K, V> {
    type Target = HashMap<K, Vec<V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AnnotatedPosting<P, C>
where
    C: Clone,
{
    pub(crate) posting: P,
    pub(crate) idx: usize,
    pub(crate) currency: Option<C>,
    pub(crate) cost_currency: Option<C>,
    pub(crate) price_currency: Option<C>,
}

impl<P, C> AnnotatedPosting<P, C>
where
    P: PostingSpec,
    C: Clone,
{
    // the bucket is the currency used for balancing weights during inference, not the currency booked to
    pub(crate) fn bucket(&self) -> Option<C>
    where
        C: Clone,
    {
        self.cost_currency
            .as_ref()
            .cloned()
            .or(self.price_currency.as_ref().cloned())
            .or_else(|| {
                // use the posting currency as the bucket only if there's neither cost nor price
                if self.posting.cost().is_none() && self.posting.price().is_none() {
                    self.currency.as_ref().cloned()
                } else {
                    None
                }
            })
    }
}

#[derive(Clone, Debug)]
pub(crate) enum BookedOrUnbookedPosting<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    Booked(Interpolated<B, P>),
    Unbooked(AnnotatedPosting<P, B::Currency>),
}

impl<B, P> BookedOrUnbookedPosting<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    // determine the weight of a posting
    // https://beancount.github.io/docs/beancount_language_syntax.html#balancing-rule-the-weight-of-postings
    pub(crate) fn weight(&self) -> Option<B::Number> {
        use BookedOrUnbookedPosting::*;

        match self {
            Booked(booked) => Some(booked.units),
            Unbooked(unbooked) => {
                let p = &unbooked.posting;

                if let Some(cost_spec) = p.cost() {
                    match (cost_spec.total(), cost_spec.per_unit(), p.units()) {
                        (Some(cost_total), _, _) => Some(cost_total),
                        (None, Some(cost_per_unit), Some(units)) => {
                            let weight = (cost_per_unit * units).rescaled(units.scale());
                            Some(weight)
                        }
                        _ => None,
                    }
                } else if let Some(price_spec) = p.price() {
                    match (price_spec.total(), price_spec.per_unit(), p.units()) {
                        (Some(price_total), _, _) => Some(price_total),
                        (None, Some(price_per_unit), Some(units)) => {
                            let weight = (price_per_unit * units).rescaled(units.scale());
                            Some(weight)
                        }
                        _ => None,
                    }
                } else {
                    p.units()
                }
            }
        }
    }
}
