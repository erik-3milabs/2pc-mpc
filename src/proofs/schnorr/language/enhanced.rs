// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

// TODO: move to lightning.

use core::array;
use std::ops::Mul;

use crypto_bigint::{Encoding, Uint};
use tiresias::secret_sharing::shamir::Polynomial;

use crate::{
    commitments::{GroupsPublicParametersAccessors as _, Pedersen},
    group,
    group::{
        additive_group_of_integers_modulu_n::power_of_two_moduli, direct_product, self_product,
        BoundedGroupElement, KnownOrderScalar, Samplable,
    },
    proofs,
    proofs::schnorr::language::GroupsPublicParametersAccessors as _,
};

pub mod committed_linear_evaluation;
pub mod encryption_of_discrete_log;
pub mod encryption_of_tuple;

pub const REPETITIONS: usize = 1;

pub type ConstrainedWitnessGroupElement<const NUM_RANGE_CLAIMS: usize, const SCALAR_LIMBS: usize> =
    self_product::GroupElement<NUM_RANGE_CLAIMS, power_of_two_moduli::GroupElement<SCALAR_LIMBS>>;
pub type ConstrainedWitnessValue<const NUM_RANGE_CLAIMS: usize, const SCALAR_LIMBS: usize> =
    group::Value<
        self_product::GroupElement<
            NUM_RANGE_CLAIMS,
            power_of_two_moduli::GroupElement<SCALAR_LIMBS>,
        >,
    >;
pub type ConstrainedWitnessPublicParameters<
    const NUM_RANGE_CLAIMS: usize,
    const SCALAR_LIMBS: usize,
> = group::PublicParameters<
    self_product::GroupElement<NUM_RANGE_CLAIMS, power_of_two_moduli::GroupElement<SCALAR_LIMBS>>,
>;

pub type EnhancedLanguageWitness<
    const NUM_RANGE_CLAIMS: usize,
    const SCALAR_LIMBS: usize,
    Scalar: BoundedGroupElement<SCALAR_LIMBS>,
    GroupElement: BoundedGroupElement<SCALAR_LIMBS>,
    L,
> = direct_product::ThreeWayGroupElement<
    ConstrainedWitnessGroupElement<NUM_RANGE_CLAIMS, SCALAR_LIMBS>,
    Scalar,
    <L as EnhancedLanguage<NUM_RANGE_CLAIMS, SCALAR_LIMBS, Scalar, GroupElement>>::UnboundedWitnessSpaceGroupElement,
>;

pub type EnhancedLanguageStatement<
    const NUM_RANGE_CLAIMS: usize,
    const SCALAR_LIMBS: usize,
    Scalar: BoundedGroupElement<SCALAR_LIMBS>,
    GroupElement: BoundedGroupElement<SCALAR_LIMBS>,
    L,
> = direct_product::GroupElement<GroupElement, <L as EnhancedLanguage<NUM_RANGE_CLAIMS, SCALAR_LIMBS, Scalar, GroupElement>>::RemainingStatementSpaceGroupElement>;

// TODO: instead of this being a trait, have it as a struct with trait impl.

/// An Enhacned Schnorr Zero-Knowledge Proof Language.
/// Can be generically used to generate a batched Schnorr zero-knowledge `Proof` with range claims.
/// As defined in Appendix B. Schnorr Protocols in the paper.
pub trait EnhancedLanguage<
    const NUM_RANGE_CLAIMS: usize,
    const SCALAR_LIMBS: usize,
    Scalar: BoundedGroupElement<SCALAR_LIMBS>,
    GroupElement: BoundedGroupElement<SCALAR_LIMBS>,
>:
    super::Language<
    REPETITIONS,
    WitnessSpaceGroupElement = EnhancedLanguageWitness<
        NUM_RANGE_CLAIMS,
        SCALAR_LIMBS,
        Scalar,
        GroupElement,
        Self,
    >,
    StatementSpaceGroupElement = EnhancedLanguageStatement<
        NUM_RANGE_CLAIMS,
        SCALAR_LIMBS,
        Scalar,
        GroupElement,
        Self,
    >,
> where
    Uint<SCALAR_LIMBS>: Encoding,
{
    /// The unbounded part of the witness group element.
    type UnboundedWitnessSpaceGroupElement: group::GroupElement + Samplable;

    /// An element in the associated statement space, that will be the image of the homomorphism
    /// alongside the range proof commitment.
    type RemainingStatementSpaceGroupElement: group::GroupElement;
}

impl<WitnessSpacePublicParameters, StatementSpacePublicParameters>
    super::GroupsPublicParameters<WitnessSpacePublicParameters, StatementSpacePublicParameters>
{
    pub fn new<
        const NUM_RANGE_CLAIMS: usize,
        const SCALAR_LIMBS: usize,
        Scalar: BoundedGroupElement<SCALAR_LIMBS>,
        GroupElement: BoundedGroupElement<SCALAR_LIMBS>,
        Lang: EnhancedLanguage<NUM_RANGE_CLAIMS, SCALAR_LIMBS, Scalar, GroupElement>,
    >(
        pedersen_public_parameters: pedersen::PublicParameters<
            NUM_RANGE_CLAIMS,
            GroupElement::Value,
            Scalar::PublicParameters,
            GroupElement::PublicParameters,
        >,
        unbounded_witness_space_public_parameters: group::PublicParameters<
            Lang::UnboundedWitnessSpaceGroupElement,
        >,
        remaining_statement_space_public_parameters: group::PublicParameters<
            Lang::RemainingStatementSpaceGroupElement,
        >,
        sampling_bit_size: usize,
    ) -> super::GroupsPublicParameters<
        super::WitnessSpacePublicParameters<REPETITIONS, Lang>,
        super::StatementSpacePublicParameters<REPETITIONS, Lang>,
    >
    where
        Uint<SCALAR_LIMBS>: Encoding,
    {
        // TODO: what to do with `sampling_bit_size`? can it be consistent
        // to all claims or do we need to get it individually?
        let constrained_witness_public_parameters =
            self_product::PublicParameters::<
                NUM_RANGE_CLAIMS,
                power_of_two_moduli::PublicParameters<SCALAR_LIMBS>,
            >::new(power_of_two_moduli::PublicParameters { sampling_bit_size });

        let range_proof_commitment_randomness_space_public_parameters = pedersen_public_parameters
            .randomness_space_public_parameters()
            .clone();

        let range_proof_commitment_space_public_parameters = pedersen_public_parameters
            .commitment_space_public_parameters()
            .clone();

        super::GroupsPublicParameters {
            witness_space_public_parameters: (
                constrained_witness_public_parameters,
                range_proof_commitment_randomness_space_public_parameters,
                unbounded_witness_space_public_parameters,
            )
                .into(),
            statement_space_public_parameters: (
                range_proof_commitment_space_public_parameters,
                remaining_statement_space_public_parameters,
            )
                .into(),
        }
    }
}

pub trait EnhancedLanguageWitnessAccessors<
    const NUM_RANGE_CLAIMS: usize,
    const SCALAR_LIMBS: usize,
    Scalar: BoundedGroupElement<SCALAR_LIMBS>,
    UnboundedWitnessSpaceGroupElement: group::GroupElement,
>
{
    fn constrained_witness(
        &self,
    ) -> &ConstrainedWitnessGroupElement<NUM_RANGE_CLAIMS, SCALAR_LIMBS>;

    fn range_proof_commitment_randomness(&self) -> &Scalar;

    fn unbounded_witness(&self) -> &UnboundedWitnessSpaceGroupElement;
}

impl<
        const NUM_RANGE_CLAIMS: usize,
        const SCALAR_LIMBS: usize,
        Scalar: BoundedGroupElement<SCALAR_LIMBS>,
        UnboundedWitnessSpaceGroupElement: group::GroupElement,
    >
    EnhancedLanguageWitnessAccessors<
        NUM_RANGE_CLAIMS,
        SCALAR_LIMBS,
        Scalar,
        UnboundedWitnessSpaceGroupElement,
    >
    for direct_product::ThreeWayGroupElement<
        ConstrainedWitnessGroupElement<NUM_RANGE_CLAIMS, SCALAR_LIMBS>,
        Scalar,
        UnboundedWitnessSpaceGroupElement,
    >
{
    fn constrained_witness(
        &self,
    ) -> &ConstrainedWitnessGroupElement<NUM_RANGE_CLAIMS, SCALAR_LIMBS> {
        let (constrained_witness, ..): (_, _, _) = self.into();

        constrained_witness
    }

    fn range_proof_commitment_randomness(&self) -> &Scalar {
        let (_, randomness, _) = self.into();

        randomness
    }

    fn unbounded_witness(&self) -> &UnboundedWitnessSpaceGroupElement {
        let (_, _, unbounded_witness) = self.into();

        unbounded_witness
    }
}

// TODO: do I need to make it for different types really? can't I do that for scalar, and transition
// to plaintext?
pub trait DecomposableWitness<const WITNESS_LIMBS: usize>: KnownOrderScalar<WITNESS_LIMBS>
where
    Self::Value: From<Uint<WITNESS_LIMBS>>,
{
    fn decompose_into_constrained_witness<
        const SCALAR_LIMBS: usize,
        const RANGE_CLAIMS_PER_SCALAR: usize,
    >(
        self,
        range_claim_bits: usize,
    ) -> ConstrainedWitnessGroupElement<RANGE_CLAIMS_PER_SCALAR, SCALAR_LIMBS>
    where
        Uint<SCALAR_LIMBS>: Encoding,
    {
        let witness: Uint<SCALAR_LIMBS> = self.into();

        let witness_in_range_claim_base: [power_of_two_moduli::GroupElement<SCALAR_LIMBS>;
            RANGE_CLAIMS_PER_SCALAR] = array::from_fn(|i| {
            Uint::<SCALAR_LIMBS>::from(
                &((witness >> (i * range_claim_bits))
                    & ((Uint::<WITNESS_LIMBS>::ONE << range_claim_bits)
                        .wrapping_sub(&Uint::<WITNESS_LIMBS>::ONE))),
            )
            .into()
        });

        witness_in_range_claim_base.into()
    }

    fn compose_from_constrained_witness<
        const SCALAR_LIMBS: usize,
        const RANGE_CLAIMS_PER_SCALAR: usize,
    >(
        constrained_witness: ConstrainedWitnessGroupElement<RANGE_CLAIMS_PER_SCALAR, SCALAR_LIMBS>,
        public_parameters: &group::PublicParameters<Self>,
        range_claim_bits: usize, // TODO:  take from RangeProof?
    ) -> proofs::Result<Self>
    where
        Uint<SCALAR_LIMBS>: Encoding,
    {
        // TODO: perform all the checks here, checking add - also check that no modulation occursin
        // // LIMBS for the entire computation

        // TODO: RANGE_CLAIM_LIMBS < SCALAR_LIMBS
        // TODO: use RANGE_CLAIM_BITS instead?
        let delta: Uint<WITNESS_LIMBS> = Uint::<WITNESS_LIMBS>::ONE << range_claim_bits;

        let delta = Self::new(delta.into(), public_parameters)?;

        let witness_in_witness_mask_base: [_; RANGE_CLAIMS_PER_SCALAR] = constrained_witness.into();

        // TODO: SCALAR_LIMBS < SCALAR_LIMBS
        let witness_in_witness_mask_base: group::Result<Vec<Self>> = witness_in_witness_mask_base
            .into_iter()
            .map(|witness| {
                Self::new(
                    Uint::<WITNESS_LIMBS>::from(&Uint::<SCALAR_LIMBS>::from(witness)).into(),
                    public_parameters,
                )
            })
            .collect();

        let polynomial = Polynomial::try_from(witness_in_witness_mask_base?)
            .map_err(|_| proofs::Error::InvalidParameters)?;

        Ok(polynomial.evaluate(&delta))
    }
}

impl<const SCALAR_LIMBS: usize, Scalar: KnownOrderScalar<SCALAR_LIMBS>>
    DecomposableWitness<SCALAR_LIMBS> for Scalar
where
    Self::Value: From<Uint<SCALAR_LIMBS>>,
{
}

pub trait EnhancedLanguageStatementAccessors<
    RangeProofCommitmentSchemeCommitmentSpaceGroupElement: group::GroupElement,
    RemainingStatementSpaceGroupElement: group::GroupElement,
>
{
    fn range_proof_commitment(&self) -> &RangeProofCommitmentSchemeCommitmentSpaceGroupElement;

    fn remaining_statement(&self) -> &RemainingStatementSpaceGroupElement;
}

impl<
        GroupElement: group::GroupElement,
        RemainingStatementSpaceGroupElement: group::GroupElement,
    > EnhancedLanguageStatementAccessors<GroupElement, RemainingStatementSpaceGroupElement>
    for direct_product::GroupElement<GroupElement, RemainingStatementSpaceGroupElement>
{
    fn range_proof_commitment(&self) -> &GroupElement {
        let (range_proof_commitment, _) = self.into();

        range_proof_commitment
    }

    fn remaining_statement(&self) -> &RemainingStatementSpaceGroupElement {
        let (_, remaining_statement) = self.into();

        remaining_statement
    }
}

pub type GroupsPublicParameters<
    const NUM_RANGE_CLAIMS: usize,
    const SCALAR_LIMBS: usize,
    ScalarPublicParameters,
    GroupElementPublicParameters,
    UnboundedWitnessSpacePublicParameters,
    RemainingStatementSpacePublicParameters,
> = super::GroupsPublicParameters<
    direct_product::ThreeWayPublicParameters<
        ConstrainedWitnessPublicParameters<NUM_RANGE_CLAIMS, SCALAR_LIMBS>,
        ScalarPublicParameters,
        UnboundedWitnessSpacePublicParameters,
    >,
    direct_product::PublicParameters<
        GroupElementPublicParameters,
        RemainingStatementSpacePublicParameters,
    >,
>;

#[cfg(any(test, feature = "benchmarking"))]
pub(crate) mod tests {
    use std::{array, iter, marker::PhantomData};

    use crypto_bigint::{Random, Wrapping, U128, U256};
    use rand_core::OsRng;

    use super::*;
    use crate::{
        group::{ristretto, secp256k1},
        proofs::{
            range,
            range::RangeProof,
            schnorr::{enhanced, language},
        },
        ComputationalSecuritySizedNumber, StatisticalSecuritySizedNumber,
    };

    // TODO: hardcode repetitions to 1 for enhanced languages.

    // TODO: import these constants from `lightning`
    pub(crate) const SCALAR_LIMBS: usize = U256::LIMBS;

    pub(crate) const RANGE_CLAIMS_PER_SCALAR: usize = 2;

    pub(crate) fn generate_witnesses<
        const NUM_RANGE_CLAIMS: usize,
        Scalar: BoundedGroupElement<SCALAR_LIMBS>,
        GroupElement: BoundedGroupElement<SCALAR_LIMBS>,
        Lang: EnhancedLanguage<NUM_RANGE_CLAIMS, SCALAR_LIMBS, Scalar, GroupElement>,
    >(
        language_public_parameters: &Lang::PublicParameters,
        batch_size: usize,
    ) -> Vec<Lang::WitnessSpaceGroupElement>
    where
        Uint<SCALAR_LIMBS>: Encoding,
    {
        iter::repeat_with(|| {
            let (_, commitment_randomness, unbounded_witness) =
                Lang::WitnessSpaceGroupElement::sample(
                    &mut OsRng,
                    &language_public_parameters.witness_space_public_parameters(),
                )
                .unwrap()
                .into();

            let (constrained_witness_public_paramaters, ..): (&_, &_, &_) =
                (language_public_parameters.witness_space_public_parameters()).into();

            // TODO: replace this by introducing a `ComposedWitness` type into the language, and
            // using `DecomposeWitness` trait?
            (
                array::from_fn(|_| {
                    let mask = Uint::<SCALAR_LIMBS>::MAX
                        >> (Uint::<SCALAR_LIMBS>::BITS
                            - <range::bulletproofs::RangeProof as RangeProof<
                                { ristretto::SCALAR_LIMBS },
                                { range::bulletproofs::RANGE_CLAIM_LIMBS },
                            >>::RANGE_CLAIM_BITS);

                    let value = Uint::<{ SCALAR_LIMBS }>::random(&mut OsRng) & mask;

                    power_of_two_moduli::GroupElement::new(
                        value,
                        &constrained_witness_public_paramaters.public_parameters,
                    )
                    .unwrap()
                })
                .into(),
                commitment_randomness,
                unbounded_witness,
            )
                .into()
        })
        .take(batch_size)
        .collect()
    }

    pub(crate) fn generate_witnesses_for_aggregation<
        const NUM_RANGE_CLAIMS: usize,
        Scalar: BoundedGroupElement<SCALAR_LIMBS>,
        GroupElement: BoundedGroupElement<SCALAR_LIMBS>,
        Lang: EnhancedLanguage<NUM_RANGE_CLAIMS, SCALAR_LIMBS, Scalar, GroupElement>,
    >(
        language_public_parameters: &Lang::PublicParameters,
        number_of_parties: usize,
        batch_size: usize,
    ) -> Vec<Vec<Lang::WitnessSpaceGroupElement>>
    where
        Uint<SCALAR_LIMBS>: Encoding,
    {
        iter::repeat_with(|| {
            generate_witnesses::<NUM_RANGE_CLAIMS, SCALAR_LIMBS, Scalar, GroupElement, Lang>(
                language_public_parameters,
                batch_size,
            )
        })
        .take(number_of_parties)
        .collect()
    }

    // pub(crate) fn generate_valid_proof<
    //     const NUM_RANGE_CLAIMS: usize,
    //     Scalar: BoundedGroupElement<SCALAR_LIMBS>,
    //     GroupElement: BoundedGroupElement<SCALAR_LIMBS>,
    //     Lang: EnhancedLanguage<NUM_RANGE_CLAIMS, SCALAR_LIMBS, Scalar, GroupElement>,
    // >(
    //     language_public_parameters: &Lang::PublicParameters,
    //     range_proof_public_parameters: &language::enhanced::RangeProofPublicParameters<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //         Lang,
    //     >,
    //     witnesses: Vec<Lang::WitnessSpaceGroupElement>,
    // ) -> (
    //     enhanced::Proof<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //         Lang,
    //         PhantomData<()>,
    //     >,
    //     Vec<Lang::StatementSpaceGroupElement>,
    // )
    // where
    //     Uint<RANGE_CLAIM_LIMBS>: Encoding,
    //     Uint<SCALAR_LIMBS>: Encoding,
    // {
    //     enhanced::Proof::prove(
    //         &PhantomData,
    //         language_public_parameters,
    //         range_proof_public_parameters,
    //         witnesses,
    //         &mut OsRng,
    //     )
    //     .unwrap()
    // }
    //
    // #[allow(dead_code)]
    // pub(crate) fn valid_proof_verifies<
    //     const REPETITIONS: usize,
    //     const RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    //     const NUM_RANGE_CLAIMS: usize,
    //     const RANGE_CLAIM_LIMBS: usize,
    //     const SCALAR_LIMBS: usize,
    //     Lang: EnhancedLanguage<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //     >,
    // >(
    //     language_public_parameters: &Lang::PublicParameters,
    //     range_proof_public_parameters: &RangeProofPublicParameters<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //         Lang,
    //     >,
    //     batch_size: usize,
    // ) where
    //     Uint<RANGE_CLAIM_LIMBS>: Encoding,
    //     Uint<SCALAR_LIMBS>: Encoding,
    // {
    //     let witnesses = generate_witnesses::<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //         Lang,
    //     >(language_public_parameters, batch_size);
    //
    //     let (proof, statements) = generate_valid_proof::<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //         Lang,
    //     >(
    //         language_public_parameters,
    //         range_proof_public_parameters,
    //         witnesses.clone(),
    //     );
    //
    //     let res = proof.verify(
    //         None,
    //         &PhantomData,
    //         language_public_parameters,
    //         range_proof_public_parameters,
    //         statements,
    //         &mut OsRng,
    //     );
    //
    //     assert!(
    //         res.is_ok(),
    //         "valid enhanced proofs should verify, got error: {:?}",
    //         res.err().unwrap()
    //     );
    // }
    //
    // #[allow(dead_code)]
    // pub(crate) fn proof_with_out_of_range_witness_fails<
    //     const REPETITIONS: usize,
    //     const RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    //     const NUM_RANGE_CLAIMS: usize,
    //     const RANGE_CLAIM_LIMBS: usize,
    //     const SCALAR_LIMBS: usize,
    //     Lang: EnhancedLanguage<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //     >,
    // >(
    //     language_public_parameters: &Lang::PublicParameters,
    //     range_proof_public_parameters: &RangeProofPublicParameters<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //         Lang,
    //     >,
    //     batch_size: usize,
    // ) where
    //     Uint<RANGE_CLAIM_LIMBS>: Encoding,
    //     Uint<SCALAR_LIMBS>: Encoding,
    // {
    //     let mut witnesses = generate_witnesses::<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //         Lang,
    //     >(language_public_parameters, batch_size);
    //
    //     let (constrained_witnesses, commitment_randomness, unbounded_witness) =
    //         witnesses.first().unwrap().clone().into();
    //     let mut constrained_witnesses: [power_of_two_moduli::GroupElement<SCALAR_LIMBS>;
    //         NUM_RANGE_CLAIMS] = constrained_witnesses.into();
    //
    //     // just out of range by 1
    //     constrained_witnesses[0] = power_of_two_moduli::GroupElement::new(
    //         (Uint::<SCALAR_LIMBS>::MAX
    //             >> (Uint::<SCALAR_LIMBS>::BITS
    //                 - <range::bulletproofs::RangeProof as RangeProof< { ristretto::SCALAR_LIMBS
    //                   }, { range::bulletproofs::RANGE_CLAIM_LIMBS },
    //                 >>::RANGE_CLAIM_BITS))
    //             .wrapping_add(&Uint::<SCALAR_LIMBS>::ONE),
    //         &constrained_witnesses[0].public_parameters(),
    //     )
    //     .unwrap();
    //
    //     let out_of_range_witness = (
    //         constrained_witnesses.into(),
    //         commitment_randomness,
    //         unbounded_witness,
    //     )
    //         .into();
    //
    //     witnesses[0] = out_of_range_witness;
    //
    //     let (proof, statements) = generate_valid_proof::<
    //         REPETITIONS,
    //         RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
    //         NUM_RANGE_CLAIMS,
    //         RANGE_CLAIM_LIMBS,
    //         SCALAR_LIMBS,
    //         Lang,
    //     >(
    //         language_public_parameters,
    //         range_proof_public_parameters,
    //         witnesses.clone(),
    //     );
    //
    //     assert!(
    //         matches!(
    //             proof
    //                 .verify(
    //                     None,
    //                     &PhantomData,
    //                     language_public_parameters,
    //                     range_proof_public_parameters,
    //                     statements,
    //                     &mut OsRng,
    //                 )
    //                 .err()
    //                 .unwrap(),
    //             proofs::Error::Bulletproofs(bulletproofs::ProofError::VerificationError)
    //         ),
    //         "out of range error should fail on range verification"
    //     );
    // }
}
