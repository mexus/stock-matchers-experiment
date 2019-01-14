//! An order book.
use crate::{
    bids::{Bid, BidProcessingType, BuyingBid, SellingBid},
    pool::Pool,
};

/// Bids queues.
#[derive(Default)]
pub struct OrderBook {
    pub(crate) sellers: Pool<SellingBid>,
    pub(crate) buyers: Pool<BuyingBid>,
}

impl OrderBook {
    /// Initializes an empty order book.
    pub fn empty() -> Self {
        OrderBook::default()
    }

    /// Processes a selling bid.
    pub fn process_selling(&mut self, bid: Bid<SellingBid>, bid_type: BidProcessingType) {
        if let Some(rest_of_the_bid) = self.buyers.process_bid(bid, bid_type) {
            self.sellers.push(rest_of_the_bid);
        }
    }

    /// Processes a buying bid.
    pub fn process_buying(&mut self, bid: Bid<BuyingBid>, bid_type: BidProcessingType) {
        if let Some(rest_of_the_bid) = self.sellers.process_bid(bid, bid_type) {
            self.buyers.push(rest_of_the_bid);
        }
    }
}
