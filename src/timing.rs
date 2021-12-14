use rug_fft::bit_rev_radix_2_intt;
use rug::Integer;

#[derive(Debug, Clone)]
pub struct CompParameters {
    pub a: Vec<Integer>,
    pub b: Vec<Integer>,
    pub p: Integer,
    pub w: Integer,
    pub order: Integer
}

pub fn compute(param: &CompParameters) -> Vec<Integer> {
    let mut c: Vec<Integer> = (0..param.a.len())
        .map(|i| Integer::from(&param.a[i] * &param.b[i]))
        .collect();
    bit_rev_radix_2_intt(&mut c, &param.p, &param.w);
    for i in 0..c.len() {
        c[i] = Integer::from(&c[i] * &param.order) / &param.p;
    }
    c
}