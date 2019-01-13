use crate::bids::{BuyingBid, SellingBid};
use std::{cmp::Ordering, marker::PhantomData};

#[derive(PartialEq, Eq, Debug)]
pub struct PoolKey<BidKind> {
    pub id: usize,
    price: u64,
    _p: PhantomData<BidKind>,
}

impl<BidKind> Copy for PoolKey<BidKind> {}
impl<BidKind> Clone for PoolKey<BidKind> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<BidKind> PoolKey<BidKind> {
    pub fn new(id: usize, price: u64) -> Self {
        PoolKey {
            id,
            price,
            _p: PhantomData,
        }
    }
}

impl<BidKind> PartialOrd for PoolKey<BidKind>
where
    PoolKey<BidKind>: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PoolKey<BuyingBid> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price
            .cmp(&other.price)
            .reverse()
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl Ord for PoolKey<SellingBid> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price
            .cmp(&other.price)
            .then_with(|| self.id.cmp(&other.id))
    }
}
