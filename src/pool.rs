//! Bids pool.

use crate::{
    bids::{Bid, BidProcessingType, GenericBid},
    key::PoolKey,
    range::MatchingRange,
};
use log::{debug, info};
use std::{cmp::Ord, collections::BTreeMap};

#[derive(Clone, Debug)]
pub struct Pool<BidKind>(BTreeMap<PoolKey<BidKind>, Bid<BidKind>>, usize);

impl<BidKind> Default for Pool<BidKind>
where
    PoolKey<BidKind>: Ord,
{
    fn default() -> Self {
        Pool(BTreeMap::new(), 0)
    }
}

impl<BidKind> Pool<BidKind>
where
    PoolKey<BidKind>: Ord,
{
    pub fn new() -> Self {
        Pool::default()
    }

    pub fn push(&mut self, bid: Bid<BidKind>) {
        self.1 += 1;
        let key = PoolKey::new(self.1, bid.price);
        self.0.insert(key, bid);
    }

    pub fn view_bids(&self) -> impl Iterator<Item = &Bid<BidKind>> {
        self.0.values()
    }
}

impl<BidKind, I> From<I> for Pool<BidKind>
where
    PoolKey<BidKind>: Ord,
    I: IntoIterator<Item = Bid<BidKind>>,
{
    fn from(data: I) -> Self {
        let map: BTreeMap<_, _> = data
            .into_iter()
            .zip(0..)
            .map(|(bid, id)| (PoolKey::new(id, bid.price), bid))
            .collect();
        let count = map.len();
        Pool(map, count)
    }
}

struct MatchingResult<BidKind> {
    keys_to_drop: Vec<PoolKey<BidKind>>,
    items_processed: u64,
}

impl<BidKind> Pool<BidKind>
where
    BidKind: GenericBid,
    Bid<BidKind::Opposite>: MatchingRange<BidKind>,
    PoolKey<BidKind>: Ord,
{
    fn get_suitable(
        &mut self,
        active_bid: &Bid<BidKind::Opposite>,
    ) -> impl Iterator<Item = (&PoolKey<BidKind>, &mut Bid<BidKind>)> {
        let active_user_id = active_bid.user_id;
        let range = active_bid.what_matches();
        let max_amount = active_bid.amount;
        self.0
            .range_mut(range)
            .filter(move |(_key, pool_bid)| pool_bid.user_id != active_user_id)
            .scan(max_amount, move |left, (key, pool_bid)| {
                if *left == 0 {
                    None
                } else {
                    let amount = pool_bid.amount;
                    if amount > *left {
                        *left = 0;
                    } else {
                        *left -= amount;
                    }
                    Some((key, pool_bid))
                }
            })
    }

    pub fn process_bid(
        &mut self,
        active_bid: Bid<BidKind::Opposite>,
        ty: BidProcessingType,
    ) -> Option<Bid<BidKind::Opposite>> {
        debug!(
            "Processing a {} from user {} (price: {}, size: {})",
            BidKind::Opposite::kind_name(),
            active_bid.user_id,
            active_bid.price,
            active_bid.amount
        );
        let suitable_bids = self.get_suitable(&active_bid);
        let bid = match ty {
            BidProcessingType::Limit => {
                let MatchingResult {
                    items_processed,
                    keys_to_drop,
                } = process_items(suitable_bids, &active_bid);
                keys_to_drop.into_iter().for_each(|key| {
                    self.0.remove(&key);
                });
                if items_processed == active_bid.amount {
                    None
                } else {
                    let mut active_bid = active_bid;
                    active_bid.amount -= items_processed;
                    Some(active_bid)
                }
            }
            BidProcessingType::FillOrKill => {
                let needed_amount = active_bid.amount;
                let available_amount: u64 = suitable_bids.map(|(_key, value)| value.amount).sum();
                if available_amount >= needed_amount {
                    let suitable_bids = self.get_suitable(&active_bid);
                    let MatchingResult {
                        items_processed, ..
                    } = process_items(suitable_bids, &active_bid);
                    debug_assert_eq!(items_processed, active_bid.amount);
                } else {
                    info!(
                        "[DROP ] Drop a {} from user {} (price: {}, size: {})",
                        BidKind::Opposite::kind_name(),
                        active_bid.user_id,
                        active_bid.price,
                        active_bid.amount
                    );
                }
                None
            }
            BidProcessingType::ImmediateOrCancel => {
                let MatchingResult {
                    keys_to_drop,
                    items_processed,
                } = process_items(suitable_bids, &active_bid);
                keys_to_drop.into_iter().for_each(|key| {
                    self.0.remove(&key);
                });
                if items_processed == 0 {
                    info!(
                        "[DROP ] Drop a {} from user {} (price: {}, size: {})",
                        BidKind::Opposite::kind_name(),
                        active_bid.user_id,
                        active_bid.price,
                        active_bid.amount
                    );
                }
                None
            }
        };
        if let Some(active_bid) = bid.as_ref() {
            info!(
                "[ ADD ] Add a {} from user {} (price: {}, size: {}) to the pool",
                BidKind::Opposite::kind_name(),
                active_bid.user_id,
                active_bid.price,
                active_bid.amount
            );
        }
        bid
    }
}

fn process_items<'a, BidKind: 'a>(
    items: impl IntoIterator<Item = (&'a PoolKey<BidKind>, &'a mut Bid<BidKind>)>,
    active_bid: &Bid<BidKind::Opposite>,
) -> MatchingResult<BidKind>
where
    BidKind: GenericBid,
    Bid<BidKind::Opposite>: MatchingRange<BidKind>,
    PoolKey<BidKind>: Ord,
{
    let amount_needed = active_bid.amount;
    let mut keys_to_drop = Vec::new();
    let mut items_left = amount_needed;
    items.into_iter().for_each(|(key, pool_bid)| {
        let current_items = pool_bid.amount;
        if current_items <= items_left {
            items_left -= current_items;
            keys_to_drop.push(*key);
            let (verb, direction) = BidKind::Opposite::deal_verb_direction();
            info!(
                "[TRADE] User {} {} {} items {} user {} for price {}",
                active_bid.user_id,
                verb,
                current_items,
                direction,
                pool_bid.user_id,
                pool_bid.price,
            );
        } else {
            pool_bid.amount -= items_left;
            items_left = 0;
        }
    });
    MatchingResult {
        keys_to_drop,
        items_processed: amount_needed - items_left,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bids::{BuyingBid, SellingBid};

    #[test]
    fn test_sorting_buy() {
        let pool: Pool<BuyingBid> = vec![
            Bid::empty().price(100).amount(4).user_id(1),
            Bid::empty().price(150).amount(2).user_id(1),
            Bid::empty().price(90).amount(5).user_id(1),
            Bid::empty().price(100).amount(5).user_id(0),
            Bid::empty().price(100).amount(6).user_id(1),
            Bid::empty().price(101).amount(5).user_id(1),
            Bid::empty().price(100).amount(2).user_id(1),
        ]
        .into();
        let sorted = vec![
            (1, Bid::empty().price(150).amount(2).user_id(1)),
            (5, Bid::empty().price(101).amount(5).user_id(1)),
            (0, Bid::empty().price(100).amount(4).user_id(1)),
            (3, Bid::empty().price(100).amount(5).user_id(0)),
            (4, Bid::empty().price(100).amount(6).user_id(1)),
            (6, Bid::empty().price(100).amount(2).user_id(1)),
            (2, Bid::empty().price(90).amount(5).user_id(1)),
        ];
        assert_eq!(
            sorted,
            pool.0
                .iter()
                .map(|(key, value)| (key.id, *value))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn range_test_buying_pool() {
        let selling_bid = Bid::empty().price(100).amount(15).user_id(0);
        let pool: Pool<BuyingBid> = vec![
            Bid::empty().price(100).amount(4).user_id(1),
            Bid::empty().price(150).amount(2).user_id(1),
            Bid::empty().price(90).amount(5).user_id(1),
            Bid::empty().price(100).amount(5).user_id(0),
            Bid::empty().price(100).amount(6).user_id(1),
            Bid::empty().price(101).amount(5).user_id(1),
            Bid::empty().price(99).amount(2).user_id(1),
        ]
        .into();
        let rng = selling_bid.what_matches();
        let reference = vec![
            (1, Bid::empty().price(150).amount(2).user_id(1)),
            (5, Bid::empty().price(101).amount(5).user_id(1)),
            (0, Bid::empty().price(100).amount(4).user_id(1)),
            (3, Bid::empty().price(100).amount(5).user_id(0)),
            (4, Bid::empty().price(100).amount(6).user_id(1)),
        ];
        let matched: Vec<_> = pool
            .0
            .range(rng)
            .map(|(key, value)| (key.id, *value))
            .collect();
        assert_eq!(reference, matched);
    }

    #[test]
    fn test_sorting_sell() {
        let pool: Pool<SellingBid> = vec![
            Bid::empty().price(100).amount(4).user_id(1),
            Bid::empty().price(150).amount(2).user_id(1),
            Bid::empty().price(90).amount(5).user_id(1),
            Bid::empty().price(70).amount(5).user_id(0),
            Bid::empty().price(100).amount(6).user_id(1),
            Bid::empty().price(101).amount(5).user_id(1),
            Bid::empty().price(99).amount(2).user_id(1),
        ]
        .into();
        let sorted = vec![
            (3, Bid::empty().price(70).amount(5).user_id(0)),
            (2, Bid::empty().price(90).amount(5).user_id(1)),
            (6, Bid::empty().price(99).amount(2).user_id(1)),
            (0, Bid::empty().price(100).amount(4).user_id(1)),
            (4, Bid::empty().price(100).amount(6).user_id(1)),
            (5, Bid::empty().price(101).amount(5).user_id(1)),
            (1, Bid::empty().price(150).amount(2).user_id(1)),
        ];
        assert_eq!(
            sorted,
            pool.0
                .iter()
                .map(|(key, value)| (key.id, *value))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn range_test_selling_pool() {
        let buying_bid = Bid::empty().price(100).amount(15).user_id(0);
        let pool: Pool<SellingBid> = vec![
            Bid::empty().price(100).amount(4).user_id(1),
            Bid::empty().price(150).amount(2).user_id(1),
            Bid::empty().price(90).amount(5).user_id(1),
            Bid::empty().price(70).amount(5).user_id(0),
            Bid::empty().price(100).amount(6).user_id(1),
            Bid::empty().price(101).amount(5).user_id(1),
            Bid::empty().price(100).amount(2).user_id(1),
        ]
        .into();
        let rng = buying_bid.what_matches();
        let reference = vec![
            (3, Bid::empty().price(70).amount(5).user_id(0)),
            (2, Bid::empty().price(90).amount(5).user_id(1)),
            (0, Bid::empty().price(100).amount(4).user_id(1)),
            (4, Bid::empty().price(100).amount(6).user_id(1)),
            (6, Bid::empty().price(100).amount(2).user_id(1)),
        ];
        let matched: Vec<_> = pool
            .0
            .range(rng)
            .map(|(key, value)| (key.id, *value))
            .collect();
        assert_eq!(reference, matched);
    }

    #[test]
    fn test_suitable_buying_pool() {
        let selling_bid = Bid::empty().price(100).amount(15).user_id(0);
        let mut pool: Pool<BuyingBid> = vec![
            Bid::empty().price(100).amount(4).user_id(1),
            Bid::empty().price(150).amount(2).user_id(1),
            Bid::empty().price(90).amount(5).user_id(1),
            Bid::empty().price(100).amount(5).user_id(0),
            Bid::empty().price(100).amount(6).user_id(1),
            Bid::empty().price(101).amount(5).user_id(1),
            Bid::empty().price(100).amount(2).user_id(1),
        ]
        .into();
        let check: Vec<_> = pool
            .get_suitable(&selling_bid)
            .map(|(key, value)| (key.id, *value))
            .collect();
        let expected = vec![
            (1, Bid::empty().price(150).amount(2).user_id(1)),
            (5, Bid::empty().price(101).amount(5).user_id(1)),
            (0, Bid::empty().price(100).amount(4).user_id(1)),
            (4, Bid::empty().price(100).amount(6).user_id(1)),
        ];
        assert_eq!(expected, check);
    }

    #[test]
    fn test_suitable_selling_pool() {
        let buying_bid = Bid::empty().price(100).amount(15).user_id(0);
        let mut pool: Pool<SellingBid> = vec![
            Bid::empty().price(100).amount(4).user_id(1),
            Bid::empty().price(150).amount(2).user_id(1),
            Bid::empty().price(90).amount(5).user_id(1),
            Bid::empty().price(70).amount(5).user_id(0),
            Bid::empty().price(100).amount(6).user_id(1),
            Bid::empty().price(101).amount(5).user_id(1),
            Bid::empty().price(100).amount(2).user_id(1),
        ]
        .into();
        let reference = vec![
            (2, Bid::empty().price(90).amount(5).user_id(1)),
            (0, Bid::empty().price(100).amount(4).user_id(1)),
            (4, Bid::empty().price(100).amount(6).user_id(1)),
        ];
        let check: Vec<_> = pool
            .get_suitable(&buying_bid)
            .map(|(key, value)| (key.id, *value))
            .collect();
        assert_eq!(reference, check);
    }
}
