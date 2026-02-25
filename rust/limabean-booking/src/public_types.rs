use hashbrown::HashMap;
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
    iter::{Sum, repeat},
    ops::{Add, AddAssign, Deref, Div, Mul, Neg, Sub, SubAssign},
};
use strum_macros::Display;

pub trait BookingTypes {
    type Account: Eq + Hash + Clone + Display + Debug;
    type Date: Eq + Ord + Copy + Display + Debug;
    type Currency: Eq + Hash + Ord + Clone + Display + Debug;
    type Number: Number + Display + Debug;
    type Label: Eq + Ord + Clone + Display + Debug;
}

pub trait PostingSpec: BookingTypes + Clone {
    type CostSpec: CostSpec<
            Date = Self::Date,
            Currency = Self::Currency,
            Number = Self::Number,
            Label = Self::Label,
        > + Clone
        + Debug;
    type PriceSpec: PriceSpec<Currency = Self::Currency, Number = Self::Number> + Clone + Debug;

    fn account(&self) -> Self::Account;
    fn currency(&self) -> Option<Self::Currency>;
    fn units(&self) -> Option<Self::Number>;
    fn cost(&self) -> Option<Self::CostSpec>;
    fn price(&self) -> Option<Self::PriceSpec>;
}

pub trait Posting: BookingTypes + Clone {
    fn account(&self) -> Self::Account;
    fn currency(&self) -> Self::Currency;
    fn units(&self) -> Self::Number;
    fn cost(&self) -> Option<PostingCosts<Self::Date, Self::Number, Self::Currency, Self::Label>>;
    fn price(&self) -> Option<Price<Self::Number, Self::Currency>>;
}

pub trait CostSpec: BookingTypes + Clone {
    fn date(&self) -> Option<Self::Date>;
    fn per_unit(&self) -> Option<Self::Number>;
    fn total(&self) -> Option<Self::Number>;
    fn currency(&self) -> Option<Self::Currency>;
    fn label(&self) -> Option<Self::Label>;
    fn merge(&self) -> bool;
}

pub trait PriceSpec: BookingTypes + Clone {
    fn currency(&self) -> Option<Self::Currency>;
    fn per_unit(&self) -> Option<Self::Number>;
    fn total(&self) -> Option<Self::Number>;
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Position<D, N, C, L>
where
    D: Copy,
    N: Copy,
    C: Clone,
    L: Clone,
{
    pub units: N,
    pub currency: C,
    pub cost: Option<Cost<D, N, C, L>>,
}

impl<D, N, C, L> Display for Position<D, N, C, L>
where
    D: Copy + Display,
    N: Copy + Display,
    C: Clone + Display,
    L: Clone + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.currency, self.units)?;
        if let Some(cost) = self.cost.as_ref() {
            write!(f, " {cost}")?;
        }
        Ok(())
    }
}

impl<D, N, C, L> From<(N, C)> for Position<D, N, C, L>
where
    D: Copy + Display,
    N: Copy + Display,
    C: Clone + Display,
    L: Clone + Display,
{
    fn from(value: (N, C)) -> Self {
        Self {
            currency: value.1,
            units: value.0,
            cost: None,
        }
    }
}

impl<D, N, C, L> Position<D, N, C, L>
where
    D: Copy,
    N: Copy,
    C: Clone,
    L: Clone,
{
    pub(crate) fn with_accumulated(&self, units: N) -> Self
    where
        C: Clone,
        N: Add<Output = N> + Copy,
    {
        let cost = self.cost.as_ref().cloned();
        Position {
            currency: self.currency.clone(),
            units: self.units + units,
            cost,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cost<D, N, C, L>
where
    D: Copy,
    N: Copy,
    C: Clone,
    L: Clone,
{
    pub date: D,
    pub per_unit: N,
    pub currency: C,
    pub label: Option<L>,
    pub merge: bool,
}

impl<D, N, C, L> Display for Cost<D, N, C, L>
where
    D: Copy + Display,
    N: Copy + Display,
    C: Clone + Display,
    L: Clone + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}, {} {}", &self.date, &self.per_unit, &self.currency)?;

        if let Some(label) = &self.label {
            write!(f, ", \"{label}\"")?;
        }

        if self.merge {
            write!(f, ", *",)?;
        }

        f.write_str("}")
    }
}

impl<D, N, C, L> Ord for Cost<D, N, C, L>
where
    D: Ord + Copy,
    N: Ord + Copy,
    C: Ord + Clone,
    L: Ord + Clone,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.date.cmp(&other.date) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match self.currency.cmp(&other.currency) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match self.per_unit.cmp(&other.per_unit) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match self.label.cmp(&other.label) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        self.merge.cmp(&other.merge)
    }
}

impl<D, N, C, L> PartialOrd for Cost<D, N, C, L>
where
    D: Ord + Copy,
    N: Ord + Copy,
    C: Ord + Clone,
    L: Ord + Clone,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
// Multiple different lots may be reduced by a single post,
// but only for a single cost currency.
// This is so that reductions don't violate the categorize by currency buckets.
pub struct PostingCosts<D, N, C, L>
where
    D: Copy,
    N: Copy,
    C: Clone,
    L: Clone,
{
    pub(crate) cost_currency: C,
    pub(crate) adjustments: Vec<PostingCost<D, N, L>>,
}

impl<D, N, C, L> PostingCosts<D, N, C, L>
where
    D: Copy,
    N: Copy,
    C: Clone,
    L: Clone,
{
    pub fn iter(&self) -> impl Iterator<Item = (&C, &PostingCost<D, N, L>)> {
        repeat(&self.cost_currency).zip(self.adjustments.iter())
    }

    pub fn into_currency_costs(self) -> impl Iterator<Item = (C, PostingCost<D, N, L>)> {
        repeat(self.cost_currency).zip(self.adjustments)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PostingCost<D, N, L>
where
    D: Copy,
    N: Copy,
    L: Clone,
{
    pub date: D,
    pub units: N,
    pub per_unit: N,
    pub label: Option<L>,
    pub merge: bool,
}

impl<D, N, C, L> From<(C, PostingCost<D, N, L>)> for Cost<D, N, C, L>
where
    D: Copy,
    N: Copy,
    C: Clone,
    L: Clone,
{
    fn from(value: (C, PostingCost<D, N, L>)) -> Self {
        let (
            currency,
            PostingCost {
                date,
                units: _,
                per_unit,
                label,
                merge,
            },
        ) = value;
        Self {
            date,
            per_unit,
            currency,
            label,
            merge,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Price<N, C>
where
    N: Copy,
    C: Clone,
{
    pub per_unit: N,
    pub currency: C,
}

impl<N, C> Display for Price<N, C>
where
    N: Copy + Display,
    C: Clone + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@ {} {}", &self.per_unit, &self.currency)
    }
}

#[derive(Debug)]
pub struct Bookings<P>
where
    P: PostingSpec,
{
    pub interpolated_postings: Vec<Interpolated<P>>,
    pub updated_inventory: Inventory<P>,
}

#[derive(Clone, Debug)]
pub struct Interpolated<P>
where
    P: PostingSpec,
{
    pub(crate) posting: P,
    pub(crate) idx: usize,
    pub units: P::Number,
    pub currency: P::Currency,
    pub cost: Option<PostingCosts<P::Date, P::Number, P::Currency, P::Label>>,
    pub price: Option<Price<P::Number, P::Currency>>,
}

impl<P> BookingTypes for Interpolated<P>
where
    P: PostingSpec,
{
    type Date = P::Date;
    type Account = P::Account;
    type Currency = P::Currency;
    type Number = P::Number;
    type Label = P::Label;
}

impl<P> Posting for Interpolated<P>
where
    P: PostingSpec,
{
    fn account(&self) -> Self::Account {
        self.posting.account()
    }

    fn currency(&self) -> Self::Currency {
        self.currency.clone()
    }

    fn units(&self) -> Self::Number {
        self.units
    }

    fn cost(&self) -> Option<PostingCosts<P::Date, P::Number, P::Currency, P::Label>> {
        self.cost.clone()
    }

    fn price(&self) -> Option<Price<Self::Number, Self::Currency>> {
        self.price.clone()
    }
}

pub trait Tolerance: BookingTypes {
    /// compute residual, ignoring sums which are tolerably small
    fn residual(
        &self,
        values: impl Iterator<Item = Self::Number>,
        cur: &Self::Currency,
    ) -> Option<Self::Number>;
}

pub trait Number:
    Copy
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Sum
    + Eq
    + Ord
    + Sized
    + Default
{
    fn abs(&self) -> Self;

    // zero is neither positive nor negative
    fn sign(&self) -> Option<Sign>;

    fn zero() -> Self;

    // Returns the scale of the decimal number, otherwise known as e.
    fn scale(&self) -> u32;

    // Returns a new number with specified scale, rounding as required.
    fn rescaled(self, scale: u32) -> Self;
}

#[derive(PartialEq, Eq, Clone, Copy, Display, Debug)]
pub enum Sign {
    Positive,
    Negative,
}

/// The booking method for an account.
#[derive(PartialEq, Eq, Default, Clone, Copy, Display, Debug)]
pub enum Booking {
    #[default]
    Strict,
    StrictWithSize,
    None,
    Average,
    Fifo,
    Lifo,
    Hifo,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Positions<B>(Vec<Position<B::Date, B::Number, B::Currency, B::Label>>)
where
    B: BookingTypes;

impl<B> Display for Positions<B>
where
    B: BookingTypes,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, p) in self.0.iter().enumerate() {
            write!(f, "{}{}", if i > 0 { ", " } else { "" }, p)?;
        }
        Ok(())
    }
}

impl<B> Positions<B>
where
    B: BookingTypes,
{
    // Requires that `positions` satisfy our invariants, so can't be public.
    pub(crate) fn from_previous(
        positions: Vec<Position<B::Date, B::Number, B::Currency, B::Label>>,
    ) -> Self {
        Self(positions)
    }

    pub(crate) fn get_mut(
        &mut self,
        i: usize,
    ) -> Option<&mut Position<B::Date, B::Number, B::Currency, B::Label>> {
        self.0.get_mut(i)
    }

    pub(crate) fn insert(
        &mut self,
        i: usize,
        element: Position<B::Date, B::Number, B::Currency, B::Label>,
    ) {
        self.0.insert(i, element)
    }

    pub fn units(&self) -> HashMap<&B::Currency, B::Number> {
        let mut units_by_currency = HashMap::default();
        for Position {
            currency, units, ..
        } in &self.0
        {
            if units_by_currency.contains_key(currency) {
                *units_by_currency.get_mut(currency).unwrap() += *units;
            } else {
                units_by_currency.insert(currency, *units);
            }
        }
        units_by_currency
    }

    pub fn accumulate(
        &mut self,
        units: B::Number,
        currency: B::Currency,
        cost: Option<Cost<B::Date, B::Number, B::Currency, B::Label>>,
        method: Booking,
    ) {
        use Ordering::*;

        tracing::debug!(
            "accumulate {method} {:?} {:?} {:?}",
            &units,
            &currency,
            &cost
        );

        let insertion_idx = match method {
            Booking::Strict
            | Booking::StrictWithSize
            | Booking::Fifo
            | Booking::Lifo
            | Booking::Hifo => {
                self.binary_search_by(|existing| match &existing.currency.cmp(&currency) {
                    ordering @ (Less | Greater) => *ordering,
                    Equal => match (&existing.cost, &cost) {
                        (None, None) => Equal,
                        (Some(_), None) => Greater,
                        (None, Some(_)) => Less,
                        (Some(existing_cost), Some(cost)) => {
                            existing_cost.partial_cmp(cost).unwrap_or(Equal)
                        }
                    },
                })
            }
            Booking::None => {
                self.binary_search_by(|existing| match &existing.currency.cmp(&currency) {
                    ordering @ (Less | Greater) => *ordering,
                    Equal => match (&existing.cost, &cost) {
                        (None, None) => Equal,
                        (Some(_), None) => Greater,
                        (_, Some(_)) => Less,
                    },
                })
            }
            Booking::Average => todo!("average booking method is not yet implemented"),
        };

        match (insertion_idx, cost) {
            (Ok(i), None) => {
                let position = self.get_mut(i).unwrap();
                tracing::debug!("augmenting position {:?} with {:?}", &position, units,);
                position.units += units;
            }
            (Ok(i), Some(cost)) => {
                let position = self.get_mut(i).unwrap();
                tracing::debug!(
                    "augmenting position {:?} with {:?} {:?}",
                    &position,
                    units,
                    &cost
                );
                position.units += units;
            }
            (Err(i), None) => {
                let position = Position {
                    units,
                    currency,
                    cost: None,
                };
                tracing::debug!("inserting new position {:?} at {i}", &position);
                self.insert(i, position)
            }
            (Err(i), Some(cost)) => {
                let position = Position {
                    units,
                    currency,
                    cost: Some(cost),
                };
                tracing::debug!("inserting new position {:?} at {i}", &position);
                self.insert(i, position)
            }
        }
    }
}

impl<B> Default for Positions<B>
where
    B: BookingTypes,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<B> Deref for Positions<B>
where
    B: BookingTypes,
{
    type Target = Vec<Position<B::Date, B::Number, B::Currency, B::Label>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B> IntoIterator for Positions<B>
where
    B: BookingTypes,
{
    type Item = Position<B::Date, B::Number, B::Currency, B::Label>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Inventory<B>
where
    B: BookingTypes,
{
    value: HashMap<B::Account, Positions<B>>,
}

impl<B> Default for Inventory<B>
where
    B: BookingTypes,
{
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

impl<B> From<HashMap<B::Account, Positions<B>>> for Inventory<B>
where
    B: BookingTypes,
{
    fn from(value: HashMap<B::Account, Positions<B>>) -> Self {
        Self { value }
    }
}

impl<B> Deref for Inventory<B>
where
    B: BookingTypes,
{
    type Target = HashMap<B::Account, Positions<B>>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<B> IntoIterator for Inventory<B>
where
    B: BookingTypes,
{
    type Item = (B::Account, Positions<B>);
    type IntoIter = hashbrown::hash_map::IntoIter<B::Account, Positions<B>>;

    fn into_iter(self) -> hashbrown::hash_map::IntoIter<B::Account, Positions<B>> {
        self.value.into_iter()
    }
}

impl<B> Inventory<B>
where
    B: BookingTypes,
{
    pub(crate) fn insert(&mut self, k: B::Account, v: Positions<B>) -> Option<Positions<B>> {
        self.value.insert(k, v)
    }
}
