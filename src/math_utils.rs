use crate::stdlib::ops::Shr;
use crate::types::errors::math_errors::MathError;
use felt::Felt252;
use num_bigint::{BigInt, BigUint};
use num_integer::Integer;
use num_traits::{Bounded, One, Pow, Signed, Zero};

///Returns the integer square root of the nonnegative integer n.
///This is the floor of the exact square root of n.
///Unlike math.sqrt(), this function doesn't have rounding error issues.
pub fn isqrt(n: &BigUint) -> Result<BigUint, MathError> {
    /*    # The following algorithm was copied from
    # https://stackoverflow.com/questions/15390807/integer-square-root-in-python.
    x = n
    y = (x + 1) // 2
    while y < x:
        x = y
        y = (x + n // x) // 2
    assert x**2 <= n < (x + 1) ** 2
    return x*/

    let mut x = n.clone();
    //n.shr(1) = n.div_floor(2)
    let mut y = (&x + 1_u32).shr(1_u32);

    while y < x {
        x = y;
        y = (&x + n.div_floor(&x)).shr(1_u32);
    }

    if !(&BigUint::pow(&x, 2_u32) <= n && n < &BigUint::pow(&(&x + 1_u32), 2_u32)) {
        return Err(MathError::FailedToGetSqrt(n.clone()));
    };
    Ok(x)
}

/// Performs integer division between x and y; fails if x is not divisible by y.
pub fn safe_div(x: &Felt252, y: &Felt252) -> Result<Felt252, MathError> {
    if y.is_zero() {
        return Err(MathError::DividedByZero);
    }

    let (q, r) = x.div_mod_floor(y);

    if !r.is_zero() {
        return Err(MathError::SafeDivFail(x.clone(), y.clone()));
    }

    Ok(q)
}

/// Performs integer division between x and y; fails if x is not divisible by y.
pub fn safe_div_bigint(x: &BigInt, y: &BigInt) -> Result<BigInt, MathError> {
    if y.is_zero() {
        return Err(MathError::DividedByZero);
    }

    let (q, r) = x.div_mod_floor(y);

    if !r.is_zero() {
        return Err(MathError::SafeDivFailBigInt(x.clone(), y.clone()));
    }

    Ok(q)
}

/// Performs integer division between x and y; fails if x is not divisible by y.
pub fn safe_div_usize(x: usize, y: usize) -> Result<usize, MathError> {
    if y.is_zero() {
        return Err(MathError::DividedByZero);
    }

    let (q, r) = x.div_mod_floor(&y);

    if !r.is_zero() {
        return Err(MathError::SafeDivFailUsize(x, y));
    }

    Ok(q)
}

///Returns x, y, g such that g = x*a + y*b = gcd(a, b).
fn igcdex(num_a: &BigInt, num_b: &BigInt) -> (BigInt, BigInt, BigInt) {
    match (num_a, num_b) {
        (a, b) if a.is_zero() && b.is_zero() => (BigInt::zero(), BigInt::one(), BigInt::zero()),
        (a, _) if a.is_zero() => (BigInt::zero(), num_b.signum(), num_b.abs()),
        (_, b) if b.is_zero() => (num_a.signum(), BigInt::zero(), num_a.abs()),
        _ => {
            let mut a = num_a.abs();
            let x_sign = num_a.signum();
            let mut b = num_b.abs();
            let y_sign = num_b.signum();
            let (mut x, mut y, mut r, mut s) =
                (BigInt::one(), BigInt::zero(), BigInt::zero(), BigInt::one());
            let (mut c, mut q);
            while !b.is_zero() {
                (q, c) = a.div_mod_floor(&b);
                x -= &q * &r;
                y -= &q * &s;
                (a, b, r, s, x, y) = (b, c, x, y, r, s)
            }
            (x * x_sign, y * y_sign, a)
        }
    }
}

///Finds a nonnegative integer x < p such that (m * x) % p == n.
pub fn div_mod(n: &BigInt, m: &BigInt, p: &BigInt) -> BigInt {
    let (a, _, c) = igcdex(m, p);
    debug_assert_eq!(c, BigInt::one());
    (n * a).mod_floor(p)
}

pub fn ec_add(
    point_a: (BigInt, BigInt),
    point_b: (BigInt, BigInt),
    prime: &BigInt,
) -> (BigInt, BigInt) {
    let m = line_slope(&point_a, &point_b, prime);
    let x = (m.clone() * m.clone() - point_a.0.clone() - point_b.0).mod_floor(prime);
    let y = (m * (point_a.0 - x.clone()) - point_a.1).mod_floor(prime);
    (x, y)
}

/// Computes the slope of the line connecting the two given EC points over the field GF(p).
/// Assumes the points are given in affine form (x, y) and have different x coordinates.
pub fn line_slope(
    point_a: &(BigInt, BigInt),
    point_b: &(BigInt, BigInt),
    prime: &BigInt,
) -> BigInt {
    debug_assert!(!(&point_a.0 - &point_b.0.mod_floor(prime)).is_zero());
    div_mod(
        &(&point_a.1 - &point_b.1),
        &(&point_a.0 - &point_b.0),
        prime,
    )
}

///  Doubles a point on an elliptic curve with the equation y^2 = x^3 + alpha*x + beta mod p.
/// Assumes the point is given in affine form (x, y) and has y != 0.
pub fn ec_double(point: (BigInt, BigInt), alpha: &BigInt, prime: &BigInt) -> (BigInt, BigInt) {
    let m = ec_double_slope(&point, alpha, prime);
    let x = ((&m * &m) - (2_i32 * &point.0)).mod_floor(prime);
    let y = (m * (point.0 - &x) - point.1).mod_floor(prime);
    (x, y)
}
/// Computes the slope of an elliptic curve with the equation y^2 = x^3 + alpha*x + beta mod p, at
/// the given point.
/// Assumes the point is given in affine form (x, y) and has y != 0.
pub fn ec_double_slope(point: &(BigInt, BigInt), alpha: &BigInt, prime: &BigInt) -> BigInt {
    debug_assert!(!point.1.mod_floor(prime).is_zero());
    div_mod(
        &(3_i32 * &point.0 * &point.0 + alpha),
        &(2_i32 * &point.1),
        prime,
    )
}

pub fn sqrt(n: &Felt252) -> Felt252 {
    // Based on Tonelli-Shanks' algorithm for finding square roots
    // and sympy's library implementation of said algorithm.
    if n.is_zero() || n.is_one() {
        return n.clone();
    }

    let max_felt = Felt252::max_value();
    let trailing_prime = Felt252::max_value() >> 192; // 0x800000000000011
    let a = n.pow(&trailing_prime);
    let d = (&Felt252::new(3_i32)).pow(&trailing_prime);
    let mut m = Felt252::zero();
    let mut exponent = Felt252::one() << 191_u32;
    let mut adm;
    for i in 0..192_u32 {
        adm = &a * &(&d).pow(&m);
        adm = (&adm).pow(&exponent);
        exponent >>= 1;
        // if adm ≡ -1 (mod CAIRO_PRIME)
        if adm == max_felt {
            m += Felt252::one() << i;
        }
    }
    let root_1 = n.pow(&((trailing_prime + 1_u32) >> 1)) * (&d).pow(&(m >> 1));
    let root_2 = &max_felt - &root_1 + 1_usize;
    if root_1 < root_2 {
        root_1
    } else {
        root_2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::*;
    use crate::utils::CAIRO_PRIME;
    use assert_matches::assert_matches;
    use num_traits::Num;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_divmod_a() {
        let a = bigint_str!(
            "11260647941622813594563746375280766662237311019551239924981511729608487775604310196863705127454617186486639011517352066501847110680463498585797912894788"
        );
        let b = bigint_str!(
            "4020711254448367604954374443741161860304516084891705811279711044808359405970"
        );
        assert_eq!(
            bigint_str!(
                "2904750555256547440469454488220756360634457312540595732507835416669695939476"
            ),
            div_mod(
                &a,
                &b,
                &BigInt::from_str_radix(&felt::PRIME_STR[2..], 16).expect("Couldn't parse prime")
            )
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_divmod_b() {
        let a = bigint_str!(
            "29642372811668969595956851264770043260610851505766181624574941701711520154703788233010819515917136995474951116158286220089597404329949295479559895970988"
        );
        let b = bigint_str!(
            "3443173965374276972000139705137775968422921151703548011275075734291405722262"
        );
        assert_eq!(
            bigint_str!(
                "3601388548860259779932034493250169083811722919049731683411013070523752439691"
            ),
            div_mod(
                &a,
                &b,
                &BigInt::from_str_radix(&felt::PRIME_STR[2..], 16).expect("Couldn't parse prime")
            )
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_divmod_c() {
        let a = bigint_str!(
            "1208267356464811040667664150251401430616174694388968865551115897173431833224432165394286799069453655049199580362994484548890574931604445970825506916876"
        );
        let b = bigint_str!(
            "1809792356889571967986805709823554331258072667897598829955472663737669990418"
        );
        assert_eq!(
            bigint_str!(
                "1545825591488572374291664030703937603499513742109806697511239542787093258962"
            ),
            div_mod(
                &a,
                &b,
                &BigInt::from_str_radix(&felt::PRIME_STR[2..], 16).expect("Couldn't parse prime")
            )
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_safe_div() {
        let x = Felt252::new(26);
        let y = Felt252::new(13);
        assert_matches!(safe_div(&x, &y), Ok(i) if i == Felt252::new(2));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_safe_div_non_divisor() {
        let x = Felt252::new(25);
        let y = Felt252::new(4);
        let result = safe_div(&x, &y);
        assert_matches!(
            result,
            Err(MathError::SafeDivFail(
                i, j
            )) if i == Felt252::new(25) && j == Felt252::new(4));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_safe_div_by_zero() {
        let x = Felt252::new(25);
        let y = Felt252::zero();
        let result = safe_div(&x, &y);
        assert_matches!(result, Err(MathError::DividedByZero));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_safe_div_usize() {
        assert_matches!(safe_div_usize(26, 13), Ok(2));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_safe_div_usize_non_divisor() {
        assert_matches!(
            safe_div_usize(25, 4),
            Err(MathError::SafeDivFailUsize(25, 4))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_safe_div_usize_by_zero() {
        assert_matches!(safe_div_usize(25, 0), Err(MathError::DividedByZero));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_line_slope_for_valid_points() {
        let point_a = (
            bigint_str!(
                "3139037544796708144595053687182055617920475701120786241351436619796497072089"
            ),
            bigint_str!(
                "2119589567875935397690285099786081818522144748339117565577200220779667999801"
            ),
        );
        let point_b = (
            bigint_str!(
                "3324833730090626974525872402899302150520188025637965566623476530814354734325"
            ),
            bigint_str!(
                "3147007486456030910661996439995670279305852583596209647900952752170983517249"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        assert_eq!(
            bigint_str!(
                "992545364708437554384321881954558327331693627531977596999212637460266617010"
            ),
            line_slope(&point_a, &point_b, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_double_slope_for_valid_point_a() {
        let point = (
            bigint_str!(
                "3143372541908290873737380228370996772020829254218248561772745122290262847573"
            ),
            bigint_str!(
                "1721586982687138486000069852568887984211460575851774005637537867145702861131"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        let alpha = bigint!(1);
        assert_eq!(
            bigint_str!(
                "3601388548860259779932034493250169083811722919049731683411013070523752439691"
            ),
            ec_double_slope(&point, &alpha, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn compute_double_slope_for_valid_point_b() {
        let point = (
            bigint_str!(
                "1937407885261715145522756206040455121546447384489085099828343908348117672673"
            ),
            bigint_str!(
                "2010355627224183802477187221870580930152258042445852905639855522404179702985"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        let alpha = bigint!(1);
        assert_eq!(
            bigint_str!(
                "2904750555256547440469454488220756360634457312540595732507835416669695939476"
            ),
            ec_double_slope(&point, &alpha, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_ec_double_for_valid_point_a() {
        let point = (
            bigint_str!(
                "1937407885261715145522756206040455121546447384489085099828343908348117672673"
            ),
            bigint_str!(
                "2010355627224183802477187221870580930152258042445852905639855522404179702985"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        let alpha = bigint!(1);
        assert_eq!(
            (
                bigint_str!(
                    "58460926014232092148191979591712815229424797874927791614218178721848875644"
                ),
                bigint_str!(
                    "1065613861227134732854284722490492186040898336012372352512913425790457998694"
                )
            ),
            ec_double(point, &alpha, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_ec_double_for_valid_point_b() {
        let point = (
            bigint_str!(
                "3143372541908290873737380228370996772020829254218248561772745122290262847573"
            ),
            bigint_str!(
                "1721586982687138486000069852568887984211460575851774005637537867145702861131"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        let alpha = bigint!(1);
        assert_eq!(
            (
                bigint_str!(
                    "1937407885261715145522756206040455121546447384489085099828343908348117672673"
                ),
                bigint_str!(
                    "2010355627224183802477187221870580930152258042445852905639855522404179702985"
                )
            ),
            ec_double(point, &alpha, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_ec_double_for_valid_point_c() {
        let point = (
            bigint_str!(
                "634630432210960355305430036410971013200846091773294855689580772209984122075"
            ),
            bigint_str!(
                "904896178444785983993402854911777165629036333948799414977736331868834995209"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        let alpha = bigint!(1);
        assert_eq!(
            (
                bigint_str!(
                    "3143372541908290873737380228370996772020829254218248561772745122290262847573"
                ),
                bigint_str!(
                    "1721586982687138486000069852568887984211460575851774005637537867145702861131"
                )
            ),
            ec_double(point, &alpha, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_ec_add_for_valid_points_a() {
        let point_a = (
            bigint_str!(
                "1183418161532233795704555250127335895546712857142554564893196731153957537489"
            ),
            bigint_str!(
                "1938007580204102038458825306058547644691739966277761828724036384003180924526"
            ),
        );
        let point_b = (
            bigint_str!(
                "1977703130303461992863803129734853218488251484396280000763960303272760326570"
            ),
            bigint_str!(
                "2565191853811572867032277464238286011368568368717965689023024980325333517459"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        assert_eq!(
            (
                bigint_str!(
                    "1977874238339000383330315148209250828062304908491266318460063803060754089297"
                ),
                bigint_str!(
                    "2969386888251099938335087541720168257053975603483053253007176033556822156706"
                )
            ),
            ec_add(point_a, point_b, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_ec_add_for_valid_points_b() {
        let point_a = (
            bigint_str!(
                "3139037544796708144595053687182055617920475701120786241351436619796497072089"
            ),
            bigint_str!(
                "2119589567875935397690285099786081818522144748339117565577200220779667999801"
            ),
        );
        let point_b = (
            bigint_str!(
                "3324833730090626974525872402899302150520188025637965566623476530814354734325"
            ),
            bigint_str!(
                "3147007486456030910661996439995670279305852583596209647900952752170983517249"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        assert_eq!(
            (
                bigint_str!(
                    "1183418161532233795704555250127335895546712857142554564893196731153957537489"
                ),
                bigint_str!(
                    "1938007580204102038458825306058547644691739966277761828724036384003180924526"
                )
            ),
            ec_add(point_a, point_b, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_ec_add_for_valid_points_c() {
        let point_a = (
            bigint_str!(
                "1183418161532233795704555250127335895546712857142554564893196731153957537489"
            ),
            bigint_str!(
                "1938007580204102038458825306058547644691739966277761828724036384003180924526"
            ),
        );
        let point_b = (
            bigint_str!(
                "1977703130303461992863803129734853218488251484396280000763960303272760326570"
            ),
            bigint_str!(
                "2565191853811572867032277464238286011368568368717965689023024980325333517459"
            ),
        );
        let prime = (*CAIRO_PRIME).clone().into();
        assert_eq!(
            (
                bigint_str!(
                    "1977874238339000383330315148209250828062304908491266318460063803060754089297"
                ),
                bigint_str!(
                    "2969386888251099938335087541720168257053975603483053253007176033556822156706"
                )
            ),
            ec_add(point_a, point_b, &prime)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_isqrt_a() {
        let n = biguint!(81);
        assert_matches!(isqrt(&n), Ok(x) if x == biguint!(9));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_isqrt_b() {
        let n = biguint_str!("4573659632505831259480");
        assert_matches!(isqrt(&BigUint::pow(&n, 2_u32)), Ok(num) if num == n);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_isqrt_c() {
        let n = biguint_str!(
            "3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_matches!(isqrt(&BigUint::pow(&n, 2_u32)), Ok(inner) if inner == n);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn calculate_isqrt_zero() {
        let n = BigUint::zero();
        assert_matches!(isqrt(&n), Ok(inner) if inner.is_zero());
    }

    #[test]
    fn safe_div_bigint_by_zero() {
        let x = BigInt::one();
        let y = BigInt::zero();
        assert_matches!(safe_div_bigint(&x, &y), Err(MathError::DividedByZero))
    }

    #[test]
    fn test_sqrt() {
        let n = Felt252::from_str_radix(
            "99957092485221722822822221624080199277265330641980989815386842231144616633668",
            10,
        )
        .unwrap();
        let expected_sqrt = Felt252::from_str_radix(
            "205857351767627712295703269674687767888261140702556021834663354704341414042",
            10,
        )
        .unwrap();
        assert_eq!(sqrt(&n), expected_sqrt);
    }

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
    #[test]
         // Test for sqrt of a quadratic residue. Result should be the minimum root.
         fn sqrt_felt_test(ref x in "([1-9][0-9]*)") {
             println!("{x}");
             let x = &Felt252::parse_bytes(x.as_bytes(), 10).unwrap();
             let x_sq = x * x;
             let sqrt = x_sq.sqrt();

             if &sqrt != x {
                 assert_eq!(Felt252::max_value() - sqrt + 1_usize, *x);
             } else {
                 assert_eq!(&sqrt, x);
             }
         }
    }
}
