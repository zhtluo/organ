use rug::Integer;
use std::mem::{transmute, MaybeUninit};

pub fn fmpz_to_int(p: *const flint_sys::fmpz::fmpz) -> rug::Integer {
    unsafe {
        let mut res = rug::Integer::new();
        let mpz_t_ptr: *mut gmp_mpfr_sys::gmp::mpz_t = res.as_raw_mut();
        let stub_ptr: *mut flint_sys::deps::__mpz_struct = transmute(mpz_t_ptr);
        flint_sys::fmpz::fmpz_get_mpz(stub_ptr, p);
        res
    }
}

pub fn int_to_fmpz(i: &rug::Integer) -> flint_sys::fmpz::fmpz {
    unsafe {
        let mut out = flint_sys::fmpz::fmpz::default();
        flint_sys::fmpz::fmpz_init(&mut out);
        let mpz_t_ptr: *const gmp_mpfr_sys::gmp::mpz_t = i.as_raw();
        let stub_ptr: *const flint_sys::deps::__mpz_struct = transmute(mpz_t_ptr);
        flint_sys::fmpz::fmpz_set_mpz(&mut out, stub_ptr);
        out
    }
}

pub fn solve_impl(p: &Integer, sums: &Vec<Integer>) -> Vec<Integer> {
    unsafe {
        let n: i64 = sums.len() as i64;

        let mut ctx = MaybeUninit::<flint_sys::fmpz_mod::fmpz_mod_ctx_struct>::uninit();
        flint_sys::fmpz_mod::fmpz_mod_ctx_init(ctx.as_mut_ptr(), &int_to_fmpz(p));
        let mut ctx = ctx.assume_init();

        let mut poly = MaybeUninit::<flint_sys::fmpz_mod_poly::fmpz_mod_poly_struct>::uninit();
        flint_sys::fmpz_mod_poly::fmpz_mod_poly_init(poly.as_mut_ptr(), &mut ctx);
        let mut poly = poly.assume_init();

        let mut factors =
            MaybeUninit::<flint_sys::fmpz_mod_poly_factor::fmpz_mod_poly_factor_struct>::uninit();
        flint_sys::fmpz_mod_poly_factor::fmpz_mod_poly_factor_init(factors.as_mut_ptr(), &mut ctx);
        let mut factors = factors.assume_init();

        flint_sys::fmpz_mod_poly_factor::fmpz_mod_poly_factor_fit_length(&mut factors, n, &mut ctx);

        let mut coeff = vec![Integer::from(0); sums.len()];
        flint_sys::fmpz_mod_poly::fmpz_mod_poly_set_coeff_fmpz(
            &mut poly,
            n,
            &mut int_to_fmpz(&Integer::from(1)),
            &mut ctx,
        );

        let mut inv;
        for i in 0..sums.len() {
            coeff[i] = sums[i].clone();
            let mut k = 0;
            // for j = i-1, ..., 0
            for j in (0..i).rev() {
                let mult = Integer::from(&coeff[k] * &sums[j]);
                coeff[i] += mult;
                k += 1;
            }
            inv = Integer::from(i);
            inv = -(inv + Integer::from(1));
            inv = inv.invert(p).unwrap();
            coeff[i] *= inv;
            flint_sys::fmpz_mod_poly::fmpz_mod_poly_set_coeff_fmpz(
                &mut poly,
                n - (i as i64) - 1,
                &mut int_to_fmpz(&coeff[i]),
                &mut ctx,
            );
        }

        // Factor
        flint_sys::fmpz_mod_poly_factor::fmpz_mod_poly_factor_kaltofen_shoup(
            &mut factors,
            &mut poly,
            &mut ctx,
        );

        let mut messages = Vec::<Integer>::new();
        let exp = std::slice::from_raw_parts(factors.exp, factors.num as usize);
        let poly = std::slice::from_raw_parts_mut(factors.poly, factors.num as usize);
        for i in 0..factors.num as usize {
            let mut x = int_to_fmpz(&Integer::from(0));
            flint_sys::fmpz_mod_poly::fmpz_mod_poly_get_coeff_fmpz(
                &mut x,
                &mut poly[i],
                0,
                &mut ctx,
            );
            let mut x = fmpz_to_int(&x);
            if x > 0 {
                x = p - x;
            }
            for _ in 0..exp[i as usize] {
                messages.push(x.clone());
            }
        }

        messages
    }
}
