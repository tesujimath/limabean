use hashbrown::HashMap;
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
    iter::{Sum, repeat},
    ops::{Add, AddAssign, Deref, Mul, Neg, Sub, SubAssign},
};
use strum_macros::Display;

pub trait BookingTypes: Clone + Debug {
    type Account: Eq + Hash + Clone + Display + Debug;
    type Date: Eq + Hash + Ord + Copy + Display + Debug;
    type Currency: Eq + Hash + Ord + Clone + Display + Debug;
    type Number: Number + Display + Debug;
    type Label: Eq + Hash + Ord + Clone + Display + Debug;
}

/// The interface which must be supported by a posting to be bookable.
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

/// A cost specification, which may be rather loosely specified.
///
/// After booking, the process of interpolation turns each cost spec into a [Cost].
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

/// A price specification, which may be rather loosely specified.
///
/// After booking, the process of interpolation turns each price spec into a [Price].
pub trait PriceSpec: Clone + Debug {
    type Types: BookingTypes;

    fn per_unit(&self) -> Option<PriceSpecNumber<Self>>;
    fn total(&self) -> Option<PriceSpecNumber<Self>>;
    fn currency(&self) -> Option<PriceSpecCurrency<Self>>;
}

pub type PriceSpecNumber<T> = <<T as PriceSpec>::Types as BookingTypes>::Number;
pub type PriceSpecCurrency<T> = <<T as PriceSpec>::Types as BookingTypes>::Currency;

/// A single position in a currency, optionally at given cost.
///
/// Lots held at cost are split into separate positions, each with a unique combination of cost attributes, with at most
/// one position having no cost.
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

/// A cost complete with any fields which were missing from its [CostSpec].
///
/// In addition to `per-unit` which is the natural representation, the `total`
/// is also exposed, since this may be what the user originally specified in the
/// beanfile, and ought to be preserved at its original precision.
#[derive(Clone, Debug)]
pub struct Cost<B>
where
    B: BookingTypes,
{
    pub date: B::Date,
    pub per_unit: B::Number,
    pub total: B::Number,
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

impl<B> Hash for Cost<B>
where
    B: BookingTypes,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.date.hash(state);
        self.per_unit.hash(state);
        self.total.hash(state);
        self.currency.hash(state);
        self.label.hash(state);
        self.merge.hash(state);
    }
}

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

/// The list of posting costs for an [Interpolated] posting.
///
/// Multiple different lots may be reduced by a single post,
/// but only for a single cost currency.
// (so that reductions don't violate the categorize by currency buckets)
#[derive(PartialEq, Eq, Clone, Debug)]
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

/// One of potentially a number of posting costs for an [Interpolated] posting.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PostingCost<B>
where
    B: BookingTypes,
{
    pub date: B::Date,
    pub units: B::Number,
    pub per_unit: B::Number,
    pub total: B::Number,
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
                total,
                per_unit,
                label,
                merge,
            },
        ) = value;
        Self {
            date,
            per_unit,
            total,
            currency,
            label,
            merge,
        }
    }
}

/// A price complete with any fields which were missing from its [PriceSpec].
///
/// In addition to `per-unit` which is the natural representation, the `total`
/// is also exposed, since this may be what the user originally specified in the
/// beanfile, and ought to be preserved at its original precision.
#[derive(Clone, Debug)]
pub struct Price<B>
where
    B: BookingTypes,
{
    pub per_unit: B::Number,
    pub total: Option<B::Number>,
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

impl<B> PartialEq for Price<B>
where
    B: BookingTypes,
{
    fn eq(&self, other: &Self) -> bool {
        self.per_unit == other.per_unit
            && self.total == other.total
            && self.currency == other.currency
    }
}

impl<B> Eq for Price<B> where B: BookingTypes {}

impl<B> Hash for Price<B>
where
    B: BookingTypes,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.per_unit.hash(state);
        self.total.hash(state);
        self.currency.hash(state);
    }
}
impl<B> Ord for Price<B>
where
    B: BookingTypes,
    B::Date: Ord,
    B::Currency: Ord,
    B::Number: Ord,
    B::Label: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.currency.cmp(&other.currency) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        match self.per_unit.cmp(&other.per_unit) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        self.total.cmp(&other.total)
    }
}

impl<B> PartialOrd for Price<B>
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

/// The interpolated postings and updated inventory after booking all postings in a transaction.
#[derive(Debug)]
pub struct Bookings<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    pub interpolated_postings: Vec<Interpolated<B, P>>,
    pub updated_inventory: Inventory<B>,
}

/// An interpolated posting is one complete with any fields which were missing from its [PostingSpec].
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

pub trait Tolerance: Clone + Debug {
    type Types: BookingTypes;

    /// The default tolerance for a given currency,
    /// returning the fallback value if that particular currency was not specified.
    fn inferred_tolerance_default(
        &self,
        cur: &ToleranceCurrency<Self>,
    ) -> Option<ToleranceNumber<Self>>;

    fn inferred_tolerance_multiplier(&self) -> Option<ToleranceNumber<Self>>;
}

pub type ToleranceNumber<T> = <<T as Tolerance>::Types as BookingTypes>::Number;
pub type ToleranceCurrency<T> = <<T as Tolerance>::Types as BookingTypes>::Currency;

/// The properties required for a decimal type to be usable for booking.
pub trait Number:
    Copy
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<Output = Self>
    + Sum
    + Eq
    + Hash
    + Ord
    + Sized
    + Default
{
    fn abs(&self) -> Self;

    // zero is neither positive nor negative
    fn sign(&self) -> Option<Sign>;

    fn zero() -> Self;

    fn new(m: i64, scale: u32) -> Self;

    fn checked_div(self, other: Self) -> Option<Self>;

    // Returns the scale of the decimal number, otherwise known as e.
    fn scale(&self) -> u32;

    // Returns a new number with specified scale, rounding as required.
    fn rescaled(self, scale: u32) -> Self;
}

/// Positive or negative, with zero being neither.
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

/// The list of positions for an account.
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
                position.units += units;
            }
            (Ok(i), Some(_cost)) => {
                let position = self.get_mut(i).unwrap();
                position.units += units;
            }
            (Err(i), None) => {
                let position = Position {
                    units,
                    currency,
                    cost: None,
                };
                self.insert(i, position)
            }
            (Err(i), Some(cost)) => {
                let position = Position {
                    units,
                    currency,
                    cost: Some(cost),
                };
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

/// All account positions.
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
