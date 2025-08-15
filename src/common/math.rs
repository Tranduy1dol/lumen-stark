use crate::common::field::FieldElement;
use crate::common::polynomial::Polynomial;

pub(crate) fn extended_gcd(x: u128, y: u128) -> (u128, u128, u128) {
    let (mut old_r, mut r) = (x, y);
    let (mut old_s, mut s) = (1, 0);
    let (mut old_t, mut t) = (0, 1);

    while r != 0 {
        let quotient = old_r / r;
        (old_r, r) = (r, old_r - quotient * r);
        (old_s, s) = (s, old_s - quotient * s);
        (old_t, t) = (t, old_t - quotient * t);
    }

    (old_s, old_t, old_r)
}

pub(crate) fn poly_long_division(
    numerator: Polynomial,
    denominator: Polynomial,
) -> anyhow::Result<(Polynomial, Polynomial)> {
    if denominator.degree() == -1 {
        return Err(anyhow::anyhow!("Division by zero"));
    }
    if numerator.degree() < denominator.degree() {
        return Ok((Polynomial::default(), numerator));
    }

    let field = denominator.coeffs[0].field;
    let mut remainder = numerator.clone();
    let mut quotient_coeffs =
        vec![field.zero(); (numerator.degree() + denominator.degree()) as usize + 1];

    for i in 0..=numerator.degree() - denominator.degree() + 1 {
        if remainder.degree() < denominator.degree() {
            break;
        }
        let coeff = remainder.leading_coeff().div(denominator.leading_coeff());
        let shift = remainder.degree() - denominator.degree();

        let subtractee =
            Polynomial::new(vec![field.zero().add(coeff); shift as usize]).mul(denominator.clone());
        quotient_coeffs[shift as usize] = coeff;
        remainder = remainder.sub(subtractee);
    }

    Ok((Polynomial::new(quotient_coeffs), remainder))
}

pub(crate) fn interpolate_domain(
    domain: Vec<FieldElement>,
    values: Vec<FieldElement>,
) -> Polynomial {
    assert_eq!(domain.len(), values.len());
    assert!(domain.len() > 1);

    let field = domain[0].field;
    let x = Polynomial::new(vec![field.zero(), field.one()]);
    let mut acc = Polynomial::default();

    for i in 0..domain.len() {
        let mut prod = Polynomial::new(vec![values[i]]);
        for j in 0..domain.len() {
            if i != j {
                prod = prod
                    .mul(x.sub(Polynomial::new(vec![domain[j]])))
                    .mul(Polynomial::new(vec![domain[i].sub(domain[j]).inv()]));
            }
            acc = acc.add(prod.clone());
        }
    }

    acc
}

pub(crate) fn zerofier_domain(domain: Vec<FieldElement>) -> Polynomial {
    let field = domain[0].field;
    let x = Polynomial::new(vec![field.zero(), field.one()]);
    let mut acc = Polynomial::new(vec![field.zero()]);
    for d in domain {
        acc = acc.mul(x.sub(Polynomial::new(vec![d])));
    }
    acc
}

pub(crate) fn test_colinearity(points: Vec<(FieldElement, FieldElement)>) -> bool {
    let domain = points.iter().map(|p| p.0).collect::<Vec<_>>();
    let values = points.iter().map(|p| p.1).collect::<Vec<_>>();
    let polynomial = interpolate_domain(domain, values);
    polynomial.degree() <= 1
}