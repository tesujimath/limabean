use hashbrown::HashMap;
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
    iter::{Sum, repeat},
    ops::{Add, AddAssign, Deref, Div, Mul, Neg, Sub, SubAssign},
};
use strum_macros::Display;

pub trait BookingTypes: Clone + Debug {
    type Account: Eq + Hash + Clone + Display + Debug;
    type Date: Eq + Ord + Copy + Display + Debug;
    type Currency: Eq + Hash + Ord + Clone + Display + Debug;
    type Number: Number + Display + Debug;
    type Label: Eq + Ord + Clone + Display + Debug;
}

pub trait PostingSpec: Clone + Debug {
    type Types: BookingTypes;

    type CostSpec: CostSpec<Types = Self::Types> + Clone + Debug;
    type PriceSpec: PriceSpec<Types = Self::Types> + Clone + Debug;

    fn account(&self) -> PostingSpecAccount<Self>;
    fn units(&self) -> Option<PostingSpecNumber<Self>>;
    fn currency(&self) -> Option<PostingSpecCurrency<Self>>;
    fn cost(&self) -> Option<Self::CostSpec>;
    fn price(&self) -> Option<Self::PriceSpec>;
}

pub type PostingSpecAccount<T> = <<T as PostingSpec>::Types as BookingTypes>::Account;
pub type PostingSpecNumber<T> = <<T as PostingSpec>::Types as BookingTypes>::Number;
pub type PostingSpecCurrency<T> = <<T as PostingSpec>::Types as BookingTypes>::Currency;

pub trait Posting: Clone + Debug {
    type Types: BookingTypes;

    fn account(&self) -> PostingAccount<Self>;
    fn units(&self) -> PostingNumber<Self>;
    fn currency(&self) -> PostingCurrency<Self>;
    fn cost(&self) -> Option<PostingCosts<Self::Types>>;
    fn price(&self) -> Option<Price<Self::Types>>;
}

pub type PostingAccount<T> = <<T as Posting>::Types as BookingTypes>::Account;
pub type PostingNumber<T> = <<T as Posting>::Types as BookingTypes>::Number;
pub type PostingCurrency<T> = <<T as Posting>::Types as BookingTypes>::Currency;

pub trait CostSpec: Clone + Debug {
    type Types: BookingTypes;

    fn date(&self) -> Option<CostSpecDate<Self>>;
    fn per_unit(&self) -> Option<CostSpecNumber<Self>>;
    fn total(&self) -> Option<CostSpecNumber<Self>>;
    fn currency(&self) -> Option<CostSpecCurrency<Self>>;
    fn label(&self) -> Option<CostSpecLabel<Self>>;
    fn merge(&self) -> bool;
}

pub type CostSpecDate<T> = <<T as CostSpec>::Types as BookingTypes>::Date;
pub type CostSpecNumber<T> = <<T as CostSpec>::Types as BookingTypes>::Number;
pub type CostSpecCurrency<T> = <<T as CostSpec>::Types as BookingTypes>::Currency;
pub type CostSpecLabel<T> = <<T as CostSpec>::Types as BookingTypes>::Label;

pub trait PriceSpec: Clone + Debug {
    type Types: BookingTypes;

    fn per_unit(&self) -> Option<PriceSpecNumber<Self>>;
    fn total(&self) -> Option<PriceSpecNumber<Self>>;
    fn currency(&self) -> Option<PriceSpecCurrency<Self>>;
}

pub type PriceSpecNumber<T> = <<T as PriceSpec>::Types as BookingTypes>::Number;
pub type PriceSpecCurrency<T> = <<T as PriceSpec>::Types as BookingTypes>::Currency;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Position<B>
where
    B: BookingTypes,
{
    pub units: B::Number,
    pub currency: B::Currency,
    pub cost: Option<Cost<B>>,
}

impl<B> Display for Position<B>
where
    B: BookingTypes,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.currency, self.units)?;
        if let Some(cost) = self.cost.as_ref() {
            write!(f, " {cost}")?;
        }
        Ok(())
    }
}

impl<B> From<(B::Number, B::Currency)> for Position<B>
where
    B: BookingTypes,
{
    fn from(value: (B::Number, B::Currency)) -> Self {
        Self {
            currency: value.1,
            units: value.0,
            cost: None,
        }
    }
}

impl<B> Position<B>
where
    B: BookingTypes,
{
    pub(crate) fn with_accumulated(&self, units: B::Number) -> Self {
        let cost = self.cost.as_ref().cloned();
        Position {
            currency: self.currency.clone(),
            units: self.units + units,
            cost,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cost<B>
where
    B: BookingTypes,
{
    pub date: B::Date,
    pub per_unit: B::Number,
    pub currency: B::Currency,
    pub label: Option<B::Label>,
    pub merge: bool,
}

impl<B> Display for Cost<B>
where
    B: BookingTypes,
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

impl<B> PartialEq for Cost<B>
where
    B: BookingTypes,
{
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
            && self.per_unit == other.per_unit
            && self.currency == other.currency
            && self.label == other.label
            && self.merge == other.merge
    }
}

impl<B> Eq for Cost<B> where B: BookingTypes {}

impl<B> Ord for Cost<B>
where
    B: BookingTypes,
    B::Date: Ord,
    B::Currency: Ord,
    B::Number: Ord,
    B::Label: Ord,
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

impl<B> PartialOrd for Cost<B>
where
    B: BookingTypes,
    B::Date: Ord,
    B::Currency: Ord,
    B::Number: Ord,
    B::Label: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
// Multiple different lots may be reduced by a single post,
// but only for a single cost currency.
// This is so that reductions don't violate the categorize by currency buckets.
pub struct PostingCosts<B>
where
    B: BookingTypes,
{
    pub(crate) cost_currency: B::Currency,
    pub(crate) adjustments: Vec<PostingCost<B>>,
}

impl<B> PostingCosts<B>
where
    B: BookingTypes,
{
    pub fn iter(&self) -> impl Iterator<Item = (&B::Currency, &PostingCost<B>)> {
        repeat(&self.cost_currency).zip(self.adjustments.iter())
    }

    pub fn into_currency_costs(self) -> impl Iterator<Item = (B::Currency, PostingCost<B>)> {
        repeat(self.cost_currency).zip(self.adjustments)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PostingCost<B>
where
    B: BookingTypes,
{
    pub date: B::Date,
    pub units: B::Number,
    pub per_unit: B::Number,
    pub label: Option<B::Label>,
    pub merge: bool,
}

impl<B> From<(B::Currency, PostingCost<B>)> for Cost<B>
where
    B: BookingTypes,
{
    fn from(value: (B::Currency, PostingCost<B>)) -> Self {
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
pub struct Price<B>
where
    B: BookingTypes,
{
    pub per_unit: B::Number,
    pub currency: B::Currency,
}

impl<B> Display for Price<B>
where
    B: BookingTypes,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@ {} {}", &self.per_unit, &self.currency)
    }
}

#[derive(Debug)]
pub struct Bookings<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    pub interpolated_postings: Vec<Interpolated<B, P>>,
    pub updated_inventory: Inventory<B>,
}

#[derive(Clone, Debug)]
pub struct Interpolated<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    pub(crate) posting: P,
    pub(crate) idx: usize,
    pub units: B::Number,
    pub currency: B::Currency,
    pub cost: Option<PostingCosts<B>>,
    pub price: Option<Price<B>>,
}

impl<B, P> Posting for Interpolated<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    type Types = B;

    fn account(&self) -> B::Account {
        self.posting.account()
    }

    fn currency(&self) -> B::Currency {
        self.currency.clone()
    }

    fn units(&self) -> B::Number {
        self.units
    }

    fn cost(&self) -> Option<PostingCosts<B>> {
        self.cost.clone()
    }

    fn price(&self) -> Option<Price<B>> {
        self.price.clone()
    }
}

pub trait Tolerance: Clone + Debug {
    type Types: BookingTypes;

    /// compute residual, ignoring sums which are tolerably small
    fn residual(
        &self,
        values: impl Iterator<Item = ToleranceNumber<Self>>,
        cur: &ToleranceCurrency<Self>,
    ) -> Option<ToleranceNumber<Self>>;
}

pub type ToleranceNumber<T> = <<T as Tolerance>::Types as BookingTypes>::Number;
pub type ToleranceCurrency<T> = <<T as Tolerance>::Types as BookingTypes>::Currency;

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
pub struct Positions<B>(Vec<Position<B>>)
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
    pub(crate) fn from_previous(positions: Vec<Position<B>>) -> Self {
        Self(positions)
    }

    pub(crate) fn get_mut(&mut self, i: usize) -> Option<&mut Position<B>> {
        self.0.get_mut(i)
    }

    pub(crate) fn insert(&mut self, i: usize, element: Position<B>) {
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
        cost: Option<Cost<B>>,
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
    type Target = Vec<Position<B>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B> IntoIterator for Positions<B>
where
    B: BookingTypes,
{
    type Item = Position<B>;
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
