use crate::config::ProtocolParams;
use crate::ecc::{add, get_g, get_h, get_order, mul, new_big_num_context, to_bytes};
use openssl::ec::EcPoint;
use rayon::prelude::*;
use rug::{Complete, Integer};
use rug_fft::{bit_rev_radix_2_intt, bit_rev_radix_2_ntt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

#[derive(Serialize, Clone, Deserialize, Default, Debug)]
pub struct SetupVector {
    pub value: Vec<Integer>,
    pub value_ntt: Vec<Integer>,
    pub product_ntt: Vec<Integer>,
    pub product: Vec<Integer>,
    pub scaled: Vec<Integer>,
    pub e: Vec<Integer>,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct SetupValues {
    pub share: SetupVector,
    pub blinding: SetupVector,
    pub e: Option<Vec<Vec<u8>>>,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct SetupRelay {
    pub values: SetupValues,
    pub qw: Option<Vec<Vec<Vec<u8>>>>,
}

#[derive(Serialize, Clone, Deserialize)]
pub enum Setup {
    SetupValues(SetupValues),
    SetupRelay(SetupRelay),
}

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

pub fn gen_setup_vector(params: &ProtocolParams, mut shares: Vec<Integer>) -> SetupVector {
    let mut result: SetupVector = SetupVector::default();
    let mut hash_vector = compute_hash(0, params.vector_len, &params.ring_v.order);
    let root_of_unity = params.ring_v.root_of_unity(params.vector_len);
    result.value = shares.clone();
    bit_rev_radix_2_ntt(&mut shares, &params.ring_v.order, &root_of_unity);
    bit_rev_radix_2_ntt(&mut hash_vector, &params.ring_v.order, &root_of_unity);
    result.value_ntt = shares.clone();
    result.product_ntt = shares
        .into_iter()
        .zip(hash_vector)
        .map(|(a, b)| a * b)
        .collect();
    result.product = result.product_ntt.clone();
    bit_rev_radix_2_intt(&mut result.product, &params.ring_v.order, &root_of_unity);
    result.scaled = result
        .product
        .iter()
        .map(|i| Integer::from(i * &params.q) / &params.ring_v.order)
        .collect();
    result.e = result
        .product
        .iter()
        .zip(result.scaled.iter())
        .map(|(w, z)| Integer::from(w * &params.q) - z * &params.ring_v.order)
        .collect();
    result
}

pub fn gen_setup_values(
    params: &ProtocolParams,
    shares: &Vec<Integer>,
    do_blame: bool,
) -> SetupValues {
    let share = gen_setup_vector(params, shares.clone());
    let blinding = gen_setup_vector(params, shares.clone());
    SetupValues {
        e: if do_blame {
            Some(
                (0..params.vector_len)
                    .map(|i| {
                        to_bytes(
                            params,
                            &add(
                                params,
                                &mul(params, &get_g(params), &share.e[i]),
                                &mul(params, &get_h(params), &blinding.e[i]),
                            ),
                        )
                    })
                    .collect(),
            )
        } else {
            None
        },
        share: share,
        blinding: blinding,
    }
}

fn compute_d(
    order: &Integer,
    tau: &Integer,
    _gamma: &Vec<Integer>,
    omega: &Vec<Integer>,
    omega_len: &Vec<Integer>,
    product_ntt: &Vec<Integer>,
    product: &Vec<Integer>,
) -> Vec<Integer> {
    assert_eq!(product_ntt.len(), product.len());
    (0..product_ntt.len())
        .into_par_iter()
        .map(|j| {
            let val = Integer::from(
                Integer::from(product_ntt.len()).invert(order).unwrap()
                //    * &gamma[j]
                    * Integer::sum(
                        (0..product_ntt.len())
                            .into_par_iter()
                            .map(|k| Integer::from(&product_ntt[k] * &omega_len[j * k / product_ntt.len()]) * &omega[j * k % product_ntt.len()])
                            .collect::<Vec<_>>()
                            .iter(),
                    )
                    .complete(),
            ) - &product[j];
            assert_eq!(Integer::from(&val % order), Integer::from(0));
            val / order % tau
        })
        .collect()
}

pub fn gen_setup_relay(
    params: &ProtocolParams,
    client_values: &Vec<SetupValues>,
    do_blame: bool,
) -> SetupRelay {
    let values = gen_setup_values(params, &vec![Integer::from(1); params.vector_len], do_blame);
    if do_blame {
        let gamma = params
            .ring_v
            .root_of_unity(params.vector_len * 2)
            .invert(&params.ring_v.order)
            .unwrap();
        let omega = params
            .ring_v
            .root_of_unity(params.vector_len)
            .invert(&params.ring_v.order)
            .unwrap();
        info!("Computing gamma...");
        let gamma_inverse: Vec<Integer> = std::iter::once(Integer::from(1))
            .chain((0..params.vector_len).scan(Integer::from(1), |acc, _| {
                *acc *= &gamma;
                *acc %= &params.ring_v.order;
                Some(acc.clone())
            }))
            .collect();
        info!("Computing omega...");
        let omega_inverse: Vec<Integer> = std::iter::once(Integer::from(1))
            .chain((0..params.vector_len).scan(Integer::from(1), |acc, _| {
                *acc *= &omega;
                *acc %= &params.ring_v.order;
                Some(acc.clone())
            }))
            .collect();
        let omega_len = Integer::from(
            omega
                .pow_mod_ref(&Integer::from(params.vector_len), &params.ring_v.order)
                .unwrap(),
        );
        let omega_len_inverse: Vec<Integer> = std::iter::once(Integer::from(1))
            .chain((0..params.vector_len).scan(Integer::from(1), |acc, _| {
                *acc *= &omega_len;
                *acc %= &params.ring_v.order;
                Some(acc.clone())
            }))
            .collect();

        let mut hash_vector = compute_hash(0, params.vector_len, &params.ring_v.order);
        let root_of_unity = params.ring_v.root_of_unity(params.vector_len);
        bit_rev_radix_2_ntt(&mut hash_vector, &params.ring_v.order, &root_of_unity);
        info!("Computing d...");
        let d: Vec<Vec<Integer>> = (0..client_values.len())
            .into_par_iter()
            .map(|i| {
                compute_d(
                    &params.ring_v.order,
                    &get_order(&params),
                    &gamma_inverse,
                    &omega_inverse,
                    &omega_len_inverse,
                    &client_values[i].share.product_ntt,
                    &client_values[i].share.product,
                )
            })
            .collect();
        debug!("d = {:?}", d);
        info!("Computing d_...");
        let d_blinding: Vec<Vec<Integer>> = (0..client_values.len())
            .into_par_iter()
            .map(|i| {
                compute_d(
                    &params.ring_v.order,
                    &get_order(&params),
                    &gamma_inverse,
                    &omega_inverse,
                    &omega_len_inverse,
                    &client_values[i].blinding.product_ntt,
                    &client_values[i].blinding.product,
                )
            })
            .collect();
        info!("Computing ab...");
        let ab: Vec<Vec<EcPoint>> = (0..client_values.len())
            .into_par_iter()
            .map(|i| {
                (0..params.vector_len)
                    .into_par_iter()
                    .map(|j| {
                        add(
                            params,
                            &mul(params, &get_g(params), &client_values[i].share.value_ntt[j]),
                            &mul(
                                params,
                                &get_h(params),
                                &client_values[i].blinding.value_ntt[j],
                            ),
                        )
                    })
                    .collect()
            })
            .collect();
        info!("Computing qw...");
        let qw: Vec<Vec<EcPoint>> = (0..client_values.len())
            .into_par_iter()
            .map(|i| {
                (0..params.vector_len)
                    .into_par_iter()
                    .map(|k| {
                        mul(
                            params,
                            &add(
                                params,
                                &add(
                                    params,
                                    &mul(
                                        params,
                                        &get_g(params),
                                        &(-Integer::from(&params.ring_v.order) * &d[i][k]),
                                    ),
                                    &mul(
                                        params,
                                        &get_h(params),
                                        &(-Integer::from(&params.ring_v.order) * &d_blinding[i][k]),
                                    ),
                                ),
                                &(0..params.vector_len)
                                    .into_par_iter()
                                    .map(|j| {
                                        mul(
                                            params,
                                            &ab[i][j],
                                            &(Integer::from(
                                                Integer::from(params.vector_len)
                                                    .invert(&params.ring_v.order)
                                                    .unwrap()
                                                    * &hash_vector[j]
                                                    * &omega_len_inverse[j * k / params.vector_len]
                                                    * &omega_inverse[j * k % params.vector_len],
                                            )),
                                        )
                                    })
                                    .reduce_with(|a, b| add(params, &a, &b))
                                    .unwrap(),
                            ),
                            &params.q,
                        )
                    })
                    .collect()
            })
            .collect();
        assert!(qw
            .iter()
            .zip(
                (0..client_values.len())
                    .into_par_iter()
                    .map(|i| {
                        (0..params.vector_len)
                            .into_par_iter()
                            .map(|j| {
                                mul(
                                    params,
                                    &add(
                                        params,
                                        &mul(
                                            params,
                                            &get_h(params),
                                            &client_values[i].blinding.product[j],
                                        ),
                                        &mul(
                                            params,
                                            &get_g(params),
                                            &client_values[i].share.product[j],
                                        ),
                                    ),
                                    &params.q,
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
                    .iter()
            )
            .all(|(a, b)| {
                a.iter().zip(b.iter()).all(|(c, d)| {
                    c.eq(
                        &params.group.as_ref().unwrap(),
                        &d,
                        &mut new_big_num_context(),
                    )
                    .unwrap()
                })
            }),);
        SetupRelay {
            values: values,
            qw: Some(
                qw.iter()
                    .map(|a| a.iter().map(|b| to_bytes(params, b)).collect::<Vec<_>>())
                    .collect(),
            ),
        }
    } else {
        SetupRelay {
            values: values,
            qw: None,
        }
    }
}
