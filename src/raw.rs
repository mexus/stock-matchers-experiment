//! Raw data processing.

use crate::{
    bids::{Bid, BidProcessingType},
    cup::BidsCup,
};
use serde_derive::Deserialize;
use std::io::Read;

#[derive(Debug, Deserialize, PartialEq)]
enum Side {
    Sell,
    Buy,
}

#[derive(Debug, Deserialize, PartialEq)]
struct RawBid {
    side: Side,
    price: u64,
    #[serde(rename = "size")]
    amount: u64,
    user_id: u64,
    #[serde(rename = "type")]
    processing_type: BidProcessingType,
}

/// Processes orders (bids) from a given reader.
///
/// The data is expected to be a list of orders (bids) in the `yaml` format with the following
/// structure:
///
/// ```norun
/// ---
/// - side: ..
///   price: ..
///   size: ..
///   user_id: ..
///   type: ..
/// - ...
/// ```
///
/// Where ...
///  * `side` could be either `Sell` or `Buy`,
///  * `price`, `size` and `user_id` are unsigned integers (`u64`),
///  * `type` is either `Limit`, `FillOrKill` or `ImmediateOrCancel`.
///
/// ```yaml
/// ---
/// - side: Sell
///   price: 100500
///   size: 999
///   user_id: 15
///   type: Limit
/// - side: Buy
///   price: 100500
///   size: 999
///   user_id: 15
///   type: ImmediateOrCancel
/// ```
pub fn process_reader(cup: &mut BidsCup, r: impl Read) -> Result<(), serde_yaml::Error> {
    let raw_bids: Vec<RawBid> = serde_yaml::from_reader(r)?;
    raw_bids.into_iter().for_each(|raw_bid| match raw_bid.side {
        Side::Sell => {
            let selling_bid = Bid::empty()
                .price(raw_bid.price)
                .amount(raw_bid.amount)
                .user_id(raw_bid.user_id);
            cup.process_selling(selling_bid, raw_bid.processing_type);
        }
        Side::Buy => {
            let buying_bid = Bid::empty()
                .price(raw_bid.price)
                .amount(raw_bid.amount)
                .user_id(raw_bid.user_id);
            cup.process_buying(buying_bid, raw_bid.processing_type);
        }
    });
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bids::Bid;

    #[test]
    fn test_deser() {
        let data = br#"---
- side: Sell
  price: 10
  size: 99
  user_id: 15
  type: Limit
- side: Buy
  price: 100500
  size: 104
  user_id: 16
  type: Limit
- side: Buy
  price: 904902491
  size: 35923852309
  user_id: 1543923349209
  type: FillOrKill
- side: Buy
  price: 0
  size: 0
  user_id: 0
  type: ImmediateOrCancel
"#;
        let data: Vec<RawBid> = serde_yaml::from_reader(&data[..]).unwrap();
        let expected = vec![
            RawBid {
                side: Side::Sell,
                price: 10,
                amount: 99,
                user_id: 15,
                processing_type: BidProcessingType::Limit,
            },
            RawBid {
                side: Side::Buy,
                price: 100500,
                amount: 104,
                user_id: 16,
                processing_type: BidProcessingType::Limit,
            },
            RawBid {
                side: Side::Buy,
                price: 904902491,
                amount: 35923852309,
                user_id: 1543923349209,
                processing_type: BidProcessingType::FillOrKill,
            },
            RawBid {
                side: Side::Buy,
                price: 0,
                amount: 0,
                user_id: 0,
                processing_type: BidProcessingType::ImmediateOrCancel,
            },
        ];
        assert_eq!(data, expected);
    }

    #[test]
    fn test_process() {
        let data = br#"---
- side: Sell
  price: 10
  size: 99
  user_id: 15
  type: Limit
- side: Buy
  price: 100500
  size: 104
  user_id: 16
  type: Limit
"#;
        let mut cup = BidsCup::default();
        process_reader(&mut cup, &data[..]).unwrap();
        let selling_bids: Vec<_> = cup.sellers.view_bids().collect();
        let buying_bids: Vec<_> = cup.buyers.view_bids().collect();
        let expected_buying = [&Bid::empty().price(100500).amount(5).user_id(16)];
        assert!(selling_bids.is_empty(), "{:?}", selling_bids);
        assert_eq!(buying_bids, expected_buying);
    }
}
