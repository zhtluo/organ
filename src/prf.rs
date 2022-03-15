use rayon::prelude::*;
use rug::Integer;
use rug_fft::{bit_rev_radix_2_intt, bit_rev_radix_2_ntt};

pub fn compute(
    params: &crate::config::ProtocolParams,
    prf: &crate::guard::SetupValues,
) -> Vec<Integer> {
    let root_of_unity = params.ring_v.root_of_unity(params.vector_len);
    let mut hash_vector = crate::guard::compute_hash(0, params.vector_len, &params.ring_v.order);
    bit_rev_radix_2_ntt(&mut hash_vector, &params.ring_v.order, &root_of_unity);
    let mut product = prf
        .share
        .value_ntt
        .par_iter()
        .zip(hash_vector)
        .map(|(a, b)| a * b)
        .collect::<Vec<_>>();
    bit_rev_radix_2_intt(&mut product, &params.ring_v.order, &root_of_unity);
    product
        .par_iter()
        .map(|i| Integer::from(i * &params.q) / &params.ring_v.order)
        .collect()
}
