// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Add, AddAssign, BitAnd, Mul, MulAssign, Neg, Sub, SubAssign};

use crypto_bigint::{ConcatMixed, Uint};
use serde::{Deserialize, Serialize};
use subtle::{Choice, ConstantTimeEq};

use crate::group::GroupElement as GroupElementTrait;

/// An element of the Direct Product of the two Groups `G` and `H`.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct GroupElement<const G_SCALAR_LIMBS: usize, const H_SCALAR_LIMBS: usize, G, H>(G, H);

/// The public parameters of the Direct Product of the two Groups `G` and `H`.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct PublicParameters<
    const G_SCALAR_LIMBS: usize,
    const H_SCALAR_LIMBS: usize,
    G: GroupElementTrait<G_SCALAR_LIMBS>,
    H: GroupElementTrait<H_SCALAR_LIMBS>,
>(G::PublicParameters, H::PublicParameters);

/// The value of the Direct Product of the two Groups `G` and `H`.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Value<
    const G_SCALAR_LIMBS: usize,
    const H_SCALAR_LIMBS: usize,
    G: GroupElementTrait<G_SCALAR_LIMBS>,
    H: GroupElementTrait<H_SCALAR_LIMBS>,
>(G::Value, H::Value);

impl<
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > ConstantTimeEq for Value<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0).bitand(self.1.ct_eq(&other.1))
    }
}

impl<
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
        const SCALAR_LIMBS: usize,
    > GroupElementTrait<SCALAR_LIMBS> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
where
    // The direct product of two bounded-order groups `G` and `H` is bounded by the order of the
    // multiple of the bounds.
    Uint<G_SCALAR_LIMBS>: ConcatMixed<Uint<H_SCALAR_LIMBS>, MixedOutput = Uint<SCALAR_LIMBS>>,
{
    type Value = Value<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>;

    fn value(&self) -> Self::Value {
        Value(self.0.value(), self.1.value())
    }

    type PublicParameters = PublicParameters<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>;

    fn public_parameters(&self) -> Self::PublicParameters {
        PublicParameters(self.0.public_parameters(), self.1.public_parameters())
    }

    fn new(
        value: Self::Value,
        public_parameters: &Self::PublicParameters,
    ) -> crate::group::Result<Self> {
        Ok(Self(
            G::new(value.0, &public_parameters.0)?,
            H::new(value.1, &public_parameters.1)?,
        ))
    }

    fn neutral(&self) -> Self {
        Self(G::neutral(&self.0), H::neutral(&self.1))
    }

    fn scalar_mul<const LIMBS: usize>(&self, scalar: &Uint<LIMBS>) -> Self {
        Self(self.0.scalar_mul(scalar), self.1.scalar_mul(scalar))
    }

    fn double(&self) -> Self {
        Self(self.0.double(), self.1.double())
    }
}

impl<
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > Neg for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.neg(), self.1.neg())
    }
}

impl<
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > Add<Self> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.add(&rhs.0), self.1.add(rhs.1))
    }
}

impl<
        'r,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > Add<&'r Self> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    type Output = Self;

    fn add(self, rhs: &'r Self) -> Self::Output {
        Self(self.0.add(&rhs.0), self.1.add(&rhs.1))
    }
}

impl<
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > Sub<Self> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.sub(&rhs.0), self.1.sub(rhs.1))
    }
}

impl<
        'r,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > Sub<&'r Self> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    type Output = Self;

    fn sub(self, rhs: &'r Self) -> Self::Output {
        Self(self.0.sub(&rhs.0), self.1.sub(&rhs.1))
    }
}

impl<
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > AddAssign<Self> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    fn add_assign(&mut self, rhs: Self) {
        self.0.add_assign(&rhs.0);
        self.1.add_assign(rhs.1);
    }
}

impl<
        'r,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > AddAssign<&'r Self> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    fn add_assign(&mut self, rhs: &'r Self) {
        self.0.add_assign(&rhs.0);
        self.1.add_assign(&rhs.1);
    }
}

impl<
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > SubAssign<Self> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0.sub_assign(&rhs.0);
        self.1.sub_assign(rhs.1);
    }
}

impl<
        'r,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
    > SubAssign<&'r Self> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
{
    fn sub_assign(&mut self, rhs: &'r Self) {
        self.0.sub_assign(&rhs.0);
        self.1.sub_assign(&rhs.1);
    }
}

impl<
        const LIMBS: usize,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
        const SCALAR_LIMBS: usize,
    > Mul<Uint<LIMBS>> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
where
    Uint<G_SCALAR_LIMBS>: ConcatMixed<Uint<H_SCALAR_LIMBS>, MixedOutput = Uint<SCALAR_LIMBS>>,
{
    type Output = Self;

    fn mul(self, rhs: Uint<LIMBS>) -> Self::Output {
        self.scalar_mul(&rhs)
    }
}

impl<
        'r,
        const LIMBS: usize,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
        const SCALAR_LIMBS: usize,
    > Mul<&'r Uint<LIMBS>> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
where
    Uint<G_SCALAR_LIMBS>: ConcatMixed<Uint<H_SCALAR_LIMBS>, MixedOutput = Uint<SCALAR_LIMBS>>,
{
    type Output = Self;

    fn mul(self, rhs: &'r Uint<LIMBS>) -> Self::Output {
        self.scalar_mul(rhs)
    }
}

impl<
        'r,
        const LIMBS: usize,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
        const SCALAR_LIMBS: usize,
    > Mul<Uint<LIMBS>> for &'r GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
where
    Uint<G_SCALAR_LIMBS>: ConcatMixed<Uint<H_SCALAR_LIMBS>, MixedOutput = Uint<SCALAR_LIMBS>>,
{
    type Output = GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>;

    fn mul(self, rhs: Uint<LIMBS>) -> Self::Output {
        self.scalar_mul(&rhs)
    }
}

impl<
        'r,
        const LIMBS: usize,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
        const SCALAR_LIMBS: usize,
    > Mul<&'r Uint<LIMBS>> for &'r GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
where
    Uint<G_SCALAR_LIMBS>: ConcatMixed<Uint<H_SCALAR_LIMBS>, MixedOutput = Uint<SCALAR_LIMBS>>,
{
    type Output = GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>;

    fn mul(self, rhs: &'r Uint<LIMBS>) -> Self::Output {
        self.scalar_mul(rhs)
    }
}

impl<
        const LIMBS: usize,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
        const SCALAR_LIMBS: usize,
    > MulAssign<Uint<LIMBS>> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
where
    Uint<G_SCALAR_LIMBS>: ConcatMixed<Uint<H_SCALAR_LIMBS>, MixedOutput = Uint<SCALAR_LIMBS>>,
{
    fn mul_assign(&mut self, rhs: Uint<LIMBS>) {
        *self = self.scalar_mul(&rhs)
    }
}

impl<
        'r,
        const LIMBS: usize,
        const G_SCALAR_LIMBS: usize,
        const H_SCALAR_LIMBS: usize,
        G: GroupElementTrait<G_SCALAR_LIMBS>,
        H: GroupElementTrait<H_SCALAR_LIMBS>,
        const SCALAR_LIMBS: usize,
    > MulAssign<&'r Uint<LIMBS>> for GroupElement<G_SCALAR_LIMBS, H_SCALAR_LIMBS, G, H>
where
    Uint<G_SCALAR_LIMBS>: ConcatMixed<Uint<H_SCALAR_LIMBS>, MixedOutput = Uint<SCALAR_LIMBS>>,
{
    fn mul_assign(&mut self, rhs: &'r Uint<LIMBS>) {
        *self = self.scalar_mul(rhs)
    }
}
