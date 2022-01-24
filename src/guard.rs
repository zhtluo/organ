use crate::config::ProtocolParams;
use rug::Integer;
use rug_fft::{bit_rev_radix_2_intt, bit_rev_radix_2_ntt};
use sha2::{Digest, Sha512};

pub fn generate_sum_shares(n: usize, modulus: &Integer, sum: &Integer) -> Vec<Integer> {
    let mut rand = rug::rand::RandState::new();
    let mut shares: Vec<Integer> =
        std::iter::repeat_with(|| Integer::from(modulus.random_below_ref(&mut rand)))
            .take(n - 1)
            .collect();
    shares.push(
        shares
            .iter()
            .fold(sum.clone(), |acc, x| (acc + modulus - x) % modulus),
    );
    shares
}

pub fn compute_hash(slot_number: usize, vec_length: usize, ring_v: &Integer) -> Vec<Integer> {
    (0..vec_length)
        .map(|i| {
            let mut hasher = Sha512::new();
            hasher.update(slot_number.to_string());
            hasher.update(i.to_string());
            Integer::from_digits(&hasher.finalize(), rug::integer::Order::Lsf) % ring_v
        })
        .collect()
}

pub fn message_gen(params: &ProtocolParams, mut shares: Vec<Integer>) -> Vec<Integer> {
    let mut hash_vector = compute_hash(0, params.vector_len, &params.ring_v.order);
    let root_of_unity = params.ring_v.root_of_unity(params.vector_len);
    bit_rev_radix_2_ntt(&mut shares, &params.ring_v.order, &root_of_unity);
    bit_rev_radix_2_ntt(&mut hash_vector, &params.ring_v.order, &root_of_unity);
    let mut comp_result: Vec<Integer> = shares
        .into_iter()
        .zip(hash_vector)
        .map(|(a, b)| a * b)
        .collect();
    bit_rev_radix_2_intt(&mut comp_result, &params.ring_v.order, &root_of_unity);
    comp_result
        .iter()
        .map(|i| Integer::from(i * &params.q) / &params.ring_v.order)
        .collect()
}
