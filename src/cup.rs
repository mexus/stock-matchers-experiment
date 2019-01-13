use crate::{
    bids::{Bid, BidProcessingType, BuyingBid, SellingBid},
    pool::Pool,
};

#[derive(Default)]
pub struct BidsCup {
    pub sellers: Pool<SellingBid>,
    pub buyers: Pool<BuyingBid>,
}

impl BidsCup {
    pub fn new() -> Self {
        BidsCup::default()
    }

    pub fn process_selling(&mut self, bid: Bid<SellingBid>, bid_type: BidProcessingType) {
        if let Some(rest_of_the_bid) = self.buyers.process_bid(bid, bid_type) {
            self.sellers.push(rest_of_the_bid)
        }
    }

    pub fn process_buying(&mut self, bid: Bid<BuyingBid>, bid_type: BidProcessingType) {
        if let Some(rest_of_the_bid) = self.sellers.process_bid(bid, bid_type) {
            self.buyers.push(rest_of_the_bid)
        }
    }
}
