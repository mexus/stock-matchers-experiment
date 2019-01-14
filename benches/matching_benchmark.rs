use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use simple_stock_matcher_experiment::{
    bids::{Bid, BidProcessingType, SellingBid},
    Pool,
};

fn generate_matching_bids(
    rng: &mut SmallRng,
    max_price: u64,
    how_many: usize,
    user_id: u64,
) -> Vec<Bid<SellingBid>> {
    (0..how_many)
        .map(|_| {
            let price = rng.gen_range(0, max_price + 1);
            let amount = rng.gen_range(1, 100);
            Bid::empty().price(price).amount(amount).user_id(user_id)
        })
        .collect()
}

fn generate_non_matching_bids(
    rng: &mut SmallRng,
    min_price: u64,
    how_many: usize,
    user_id: u64,
) -> Vec<Bid<SellingBid>> {
    (0..how_many)
        .map(|_| {
            let price = rng.gen_range(min_price + 1, u64::max_value());
            let amount = rng.gen_range(1, 100);
            Bid::empty().price(price).amount(amount).user_id(user_id)
        })
        .collect()
}

fn generate_test_queue(
    seed: u64,
    total_amount: usize,
    matching_amount: usize,
    price_level: u64,
    user_id: u64,
) -> (u64, Vec<Bid<SellingBid>>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let matching = generate_matching_bids(&mut rng, price_level, matching_amount, user_id);
    let matching_items = matching.iter().map(|bid| bid.amount).sum();
    let nonmatching = generate_non_matching_bids(
        &mut rng,
        price_level + 1,
        total_amount - matching_amount,
        user_id,
    );
    let mut combined: Vec<_> = matching.into_iter().chain(nonmatching).collect();
    combined.shuffle(&mut rng);
    (matching_items, combined)
}

fn match_maker(c: &mut Criterion) {
    let price = 20;
    let seed = 10;
    let (mathing_items_amount, pool) = generate_test_queue(seed, 7000, 20, price, 0);
    let pool = Pool::from(pool);
    let buying_bid = Bid::empty()
        .price(price)
        // We deliberately remove one item so that at least one bid from the pool is not fully
        // covered.
        .amount(mathing_items_amount - 1)
        .user_id(1);
    c.bench_function_over_inputs(
        "match_maker",
        move |bencher, &ty| {
            bencher.iter_with_setup(|| pool.clone(), |mut pool| pool.process_bid(buying_bid, ty))
        },
        vec![
            BidProcessingType::Limit,
            BidProcessingType::FillOrKill,
            BidProcessingType::ImmediateOrCancel,
        ],
    );
}

criterion_group!(benches, match_maker);
criterion_main!(benches);
