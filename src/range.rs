use crate::{
    bids::{Bid, BuyingBid, SellingBid},
    key::PoolKey,
};
use std::ops::RangeTo;

pub trait MatchingRange<Against>: Sized {
    /// Returns a range that should match (by price) current `self`.
    fn what_matches(&self) -> RangeTo<PoolKey<Against>>;
}

impl MatchingRange<SellingBid> for Bid<BuyingBid> {
    fn what_matches(&self) -> RangeTo<PoolKey<SellingBid>> {
        let maximum_buying_price = self.price;
        ..PoolKey::new(usize::max_value(), maximum_buying_price)
    }
}

impl MatchingRange<BuyingBid> for Bid<SellingBid> {
    fn what_matches(&self) -> RangeTo<PoolKey<BuyingBid>> {
        let minimum_selling_price = self.price;
        ..PoolKey::new(usize::max_value(), minimum_selling_price)
    }
}
