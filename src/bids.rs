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
    /// Price: either the highest price for a buying bid a the lowest price for a selling bid.
    pub price: u64,
    /// Amount of items to trade.
    pub amount: u64,
    /// Bid's user id.
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

/// A helper trait that allows to match selling and buying bids in compile time and provides
/// some generic methods.
pub trait GenericBid: Sized {
    /// The opposite kind of bid.
    type Opposite: GenericBid<Opposite = Self>;

    /// Verb ("bought"/"sold") and direction ("from"/"to") of the deal.
    ///
    /// Use for sentences like "User XX bought YY items from user ...".
    fn deal_verb_direction() -> (&'static str, &'static str);

    /// Literal name of the bid's kind.
    fn kind_name() -> &'static str;
}

impl GenericBid for BuyingBid {
    type Opposite = SellingBid;

    fn deal_verb_direction() -> (&'static str, &'static str) {
        ("bought", "from")
    }

    fn kind_name() -> &'static str {
        "buying bid"
    }
}

impl GenericBid for SellingBid {
    type Opposite = BuyingBid;

    fn deal_verb_direction() -> (&'static str, &'static str) {
        ("sold", "to")
    }

    fn kind_name() -> &'static str {
        "selling bid"
    }
}
