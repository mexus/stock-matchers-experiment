//! Bids-related types and traits.

use serde_derive::Deserialize;
use std::marker::PhantomData;

/// Processing type of a bid.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq)]
pub enum BidProcessingType {
    /// The bid might be executed partially. The part that can not be executed immediately should be
    /// put on a queue.
    Limit,
    /// The bid should be executed either completely or not executed at all.
    FillOrKill,
    /// The bid might be executed partially. The part that can not be executed immediately should be
    /// dropped.
    ImmediateOrCancel,
}

/// A selling or a buying bid. Its kind depends on the `BidKind` generic argument.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Bid<BidKind> {
    pub price: u64,
    pub amount: u64,
    pub user_id: u64,
    _marker: PhantomData<BidKind>,
}

impl<BidKind> Bid<BidKind> {
    /// Initializes an empty bid (with zero price, zero amount and zero user id).
    pub fn empty() -> Self {
        Bid {
            price: 0,
            amount: 0,
            user_id: 0,
            _marker: PhantomData,
        }
    }

    /// Updates the price.
    pub fn price(self, price: u64) -> Self {
        Bid {
            price: price,
            ..self
        }
    }

    /// Updates the amount.
    pub fn amount(self, amount: u64) -> Self {
        Bid {
            amount: amount,
            ..self
        }
    }

    /// Updates the user id.
    pub fn user_id(self, user_id: u64) -> Self {
        Bid {
            user_id: user_id,
            ..self
        }
    }
}

/// A marker type that marks a `Bid` as a *selling* bid.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SellingBid;

/// A marker type that marks a `Bid` as a *buying* bid.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BuyingBid;

/// A helper trait that allows to match selling and buying bids in compile time.
pub trait HasOpposite: Sized {
    type Opposite: HasOpposite<Opposite = Self>;
}

impl HasOpposite for BuyingBid {
    type Opposite = SellingBid;
}

impl HasOpposite for SellingBid {
    type Opposite = BuyingBid;
}
