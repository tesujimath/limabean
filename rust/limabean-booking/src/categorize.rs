use hashbrown::{HashMap, HashSet};
use std::fmt::Debug;

use super::{
    AnnotatedPosting, BookingError, BookingTypes, CostSpec, HashMapOfVec, Positions,
    PostingBookingError, PostingSpec, PriceSpec, TransactionBookingError,
};

pub(crate) struct CategorizedByCurrency<B, P>(
    pub(crate) HashMapOfVec<B::Currency, AnnotatedPosting<P, B::Currency>>,
)
where
    B: BookingTypes,
    P: PostingSpec<Types = B>;

// See OG Beancount function of the same name
pub(crate) fn categorize_by_currency<'a, 'b, B, P, I>(
    postings: &'b [P],
    inventory: I,
) -> Result<CategorizedByCurrency<B, P>, BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    I: Fn(B::Account) -> Option<&'a Positions<B>> + Copy,
{
    let mut currency_groups = HashMapOfVec::default();
    let mut auto_postings =
        HashMap::<Option<B::Currency>, AnnotatedPosting<P, B::Currency>>::default();
    let mut unknown = Vec::default();

    categorize_with_auto_postings_and_unknowns(
        postings,
        &mut currency_groups,
        &mut auto_postings,
        &mut unknown,
    )?;

    // if we have a single unknown posting and all others are of the same currency,
    // infer that for the unknown
    if unknown.len() == 1 && currency_groups.len() == 1 {
        infer_unknown_from_single_currency_group(
            unknown.drain(..).next().unwrap(),
            &mut currency_groups,
        );
    }

    // infer all other unknown postings from account inference
    infer_unknowns_from_account_inference(unknown, inventory, &mut currency_groups)?;

    categorize_auto_postings(auto_postings, &mut currency_groups)?;

    Ok(CategorizedByCurrency(currency_groups))
}

pub(crate) fn categorize_with_auto_postings_and_unknowns<B, P>(
    postings: &[P],
    currency_groups: &mut HashMapOfVec<B::Currency, AnnotatedPosting<P, B::Currency>>,
    auto_postings: &mut HashMap<Option<B::Currency>, AnnotatedPosting<P, B::Currency>>,
    unknown: &mut Vec<AnnotatedPosting<P, B::Currency>>,
) -> Result<(), BookingError>
where
    B: BookingTypes,
    P: PostingSpec<Types = B> + Debug,
{
    for (idx, posting) in postings.iter().enumerate() {
        let annotated = annotate(posting, idx);

        let bucket = annotated.bucket();

        if posting.units().is_none() && posting.currency().is_none() {
            if auto_postings.contains_key(&bucket) {
                return Err(BookingError::Posting(
                    idx,
                    PostingBookingError::AmbiguousAutoPost,
                ));
            }

            auto_postings.insert(bucket, annotated);
        } else if let Some(bucket) = bucket {
            currency_groups.push_or_insert(bucket, annotated);
        } else {
            unknown.push(annotated);
        }
    }

    Ok(())
}

// annotate a posting along with its index in the list of postings
fn annotate<B, P>(posting: &P, idx: usize) -> AnnotatedPosting<P, B::Currency>
where
    B: BookingTypes,
    P: PostingSpec<Types = B> + Debug,
{
    let currency = posting.currency();
    let posting_cost_currency = posting.cost().and_then(|cost_spec| cost_spec.currency());
    let posting_price_currency = posting.price().and_then(|price_spec| price_spec.currency());
    let cost_currency = posting_cost_currency
        .as_ref()
        .cloned()
        .or(posting_price_currency.as_ref().cloned());
    let price_currency = posting_price_currency
        .as_ref()
        .cloned()
        .or(posting_cost_currency);

    AnnotatedPosting {
        posting: posting.clone(),
        idx,
        currency,
        cost_currency,
        price_currency,
    }
}

fn infer_unknown_from_single_currency_group<B, P>(
    unknown: AnnotatedPosting<P, B::Currency>,
    currency_groups: &mut HashMapOfVec<B::Currency, AnnotatedPosting<P, B::Currency>>,
) where
    B: BookingTypes,
    P: PostingSpec<Types = B> + Debug,
{
    let only_bucket = currency_groups
        .keys()
        .next()
        .as_ref()
        .cloned()
        .unwrap()
        .clone();

    // infer any missing currency from bucket only if there's no cost or price
    let currency = unknown.currency.or(
        if unknown.posting.price().is_none() && unknown.posting.cost().is_none() {
            Some(only_bucket.clone())
        } else {
            None
        },
    );

    let inferred = AnnotatedPosting {
        posting: unknown.posting,
        idx: unknown.idx,
        currency,
        cost_currency: unknown
            .cost_currency
            .as_ref()
            .cloned()
            .or(Some(only_bucket.clone())),
        price_currency: unknown.price_currency.or(Some(only_bucket.clone())),
    };
    currency_groups.push_or_insert(only_bucket.clone(), inferred);
}

pub(crate) fn infer_unknowns_from_account_inference<'a, 'b, B, P, I>(
    unknown: Vec<AnnotatedPosting<P, B::Currency>>,
    inventory: I,
    currency_groups: &mut HashMapOfVec<B::Currency, AnnotatedPosting<P, B::Currency>>,
) -> Result<(), BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    I: Fn(B::Account) -> Option<&'a Positions<B>> + Copy,
{
    let mut account_currency_lookup = HashMap::<B::Account, Option<B::Currency>>::default();
    for u in unknown {
        let u_account = u.posting.account();
        if let Some(bucket) = account_currency(u_account, inventory, &mut account_currency_lookup) {
            currency_groups.push_or_insert(bucket, u);
        } else {
            return Err(BookingError::Posting(
                u.idx,
                crate::PostingBookingError::CannotInferAnything,
            ));
        }
    }
    Ok(())
}
pub(crate) fn categorize_auto_postings<B, P>(
    mut auto_postings: HashMap<Option<B::Currency>, AnnotatedPosting<P, B::Currency>>,
    currency_groups: &mut HashMapOfVec<B::Currency, AnnotatedPosting<P, B::Currency>>,
) -> Result<(), BookingError>
where
    B: BookingTypes,
    P: PostingSpec<Types = B> + Debug,
{
    if let Some(auto_posting) = auto_postings.remove(&None) {
        if !auto_postings.is_empty() {
            return Err(BookingError::Posting(
                auto_posting.idx,
                PostingBookingError::AmbiguousAutoPost,
            ));
        }

        // can only have a currency-ambiguous auto-post if there's a single bucket
        let all_buckets = currency_groups.keys().cloned().collect::<Vec<_>>();
        if all_buckets.is_empty() {
            return Err(BookingError::Transaction(
                TransactionBookingError::CannotDetermineCurrencyForBalancing,
            ));
        } else if all_buckets.len() == 1 {
            let sole_bucket = all_buckets.into_iter().next().unwrap();
            currency_groups.push_or_insert(sole_bucket, auto_posting);
        } else {
            return Err(BookingError::Transaction(
                TransactionBookingError::AutoPostMultipleBuckets(
                    all_buckets
                        .into_iter()
                        .map(|cur| cur.to_string())
                        .collect::<Vec<_>>(),
                ),
            ));
        }
    } else {
        for (bucket, auto_posting) in auto_postings.into_iter() {
            let bucket = bucket.unwrap();

            currency_groups.push_or_insert(bucket, auto_posting);
        }
    }

    Ok(())
}

// lookup account currency with memoization
fn account_currency<'a, B, I>(
    account: B::Account,
    inventory: I,
    account_currency: &mut HashMap<B::Account, Option<B::Currency>>,
) -> Option<B::Currency>
where
    B: BookingTypes + 'a,
    I: Fn(B::Account) -> Option<&'a Positions<B>> + Copy,
{
    account_currency.get(&account).cloned().unwrap_or_else(|| {
        let currency = if let Some(positions) = inventory(account.clone()) {
            let currencies = positions
                .iter()
                .map(|pos| pos.currency.clone())
                .collect::<HashSet<B::Currency>>();

            if currencies.len() == 1 {
                currencies.iter().next().cloned()
            } else {
                None
            }
        } else {
            None
        };

        account_currency.insert(account.clone(), currency.clone());

        currency
    })
}
