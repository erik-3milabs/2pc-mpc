// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

use core::array;
use std::{marker::PhantomData, ops::Mul};

use crypto_bigint::{rand_core::CryptoRngCore, Uint, U128};
use merlin::Transcript;
use serde::{Deserialize, Serialize};
use tiresias::secret_sharing::shamir::Polynomial;

use crate::{
    ahe, commitments,
    commitments::{
        pedersen, GroupsPublicParametersAccessors as _, HomomorphicCommitmentScheme, Pedersen,
    },
    group,
    group::{
        additive_group_of_integers_modulu_n::power_of_two_moduli, direct_product,
        direct_product::ThreeWayPublicParameters, paillier, self_product, BoundedGroupElement,
        GroupElement as _, GroupElement, KnownOrderScalar, Samplable, SamplableWithin,
    },
    helpers::flat_map_results,
    proofs,
    proofs::{
        range,
        range::{
            CommitmentPublicParametersAccessor, CommitmentScheme,
            CommitmentSchemeCommitmentSpaceGroupElement,
            CommitmentSchemeCommitmentSpacePublicParameters,
            CommitmentSchemeMessageSpaceGroupElement, CommitmentSchemeMessageSpacePublicParameters,
            CommitmentSchemePublicParameters, CommitmentSchemeRandomnessSpaceGroupElement,
            CommitmentSchemeRandomnessSpacePublicParameters,
        },
        schnorr,
        schnorr::{
            language,
            language::{GroupsPublicParameters, GroupsPublicParametersAccessors as _},
        },
    },
    ComputationalSecuritySizedNumber, StatisticalSecuritySizedNumber,
};

// TODO: don't even expose this, just the proof.
/// An Enhanced Schnorr Zero-Knowledge Proof Language.
/// Can be generically used to generate a batched Schnorr zero-knowledge `Proof` with range claims.
/// As defined in Appendix B. Schnorr Protocols in the paper.
#[derive(Clone, PartialEq)]
pub struct EnhancedLanguage<
    const REPETITIONS: usize,
    const NUM_RANGE_CLAIMS: usize,
    const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    RangeProof,
    UnboundedWitnessSpaceGroupElement,
    Language,
> {
    _unbounded_witness_choice: PhantomData<UnboundedWitnessSpaceGroupElement>,
    _language_choice: PhantomData<Language>,
    _range_proof_choice: PhantomData<RangeProof>,
}

// TODO: use this code in protocols. Or maybe the other compose/decompose.
pub trait EnhanceableLanguage<
    const REPETITIONS: usize,
    const NUM_RANGE_CLAIMS: usize,
    const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    UnboundedWitnessSpaceGroupElement: group::GroupElement + Samplable,
>: schnorr::Language<REPETITIONS>
{
    // TODO: solve all these refs & clones, here and in accessors. Perhaps partial move is ok.
    fn compose_witness(
        decomposed_witness: &[Uint<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>; NUM_RANGE_CLAIMS],
        unbounded_witness: &UnboundedWitnessSpaceGroupElement,
        language_public_parameters: &Self::PublicParameters,
    ) -> proofs::Result<Self::WitnessSpaceGroupElement>;

    fn decompose_witness(
        witness: &Self::WitnessSpaceGroupElement,
        language_public_parameters: &Self::PublicParameters,
    ) -> proofs::Result<(
        [Uint<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>; NUM_RANGE_CLAIMS],
        UnboundedWitnessSpaceGroupElement,
    )>;
}

impl<
        const REPETITIONS: usize,
        const NUM_RANGE_CLAIMS: usize,
        const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        RangeProof: range::RangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
        UnboundedWitnessSpaceGroupElement: group::GroupElement + SamplableWithin,
        Language: EnhanceableLanguage<
            REPETITIONS,
            NUM_RANGE_CLAIMS,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            UnboundedWitnessSpaceGroupElement,
        >,
    > schnorr::Language<REPETITIONS>
    for EnhancedLanguage<
        REPETITIONS,
        NUM_RANGE_CLAIMS,
        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RangeProof,
        UnboundedWitnessSpaceGroupElement,
        Language,
    >
{
    type WitnessSpaceGroupElement = direct_product::ThreeWayGroupElement<
        CommitmentSchemeMessageSpaceGroupElement<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >,
        CommitmentSchemeRandomnessSpaceGroupElement<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >,
        UnboundedWitnessSpaceGroupElement,
    >;

    type StatementSpaceGroupElement = direct_product::GroupElement<
        CommitmentSchemeCommitmentSpaceGroupElement<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >,
        Language::StatementSpaceGroupElement,
    >;

    type PublicParameters = PublicParameters<
        REPETITIONS,
        NUM_RANGE_CLAIMS,
        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        CommitmentSchemeMessageSpacePublicParameters<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >,
        CommitmentSchemeRandomnessSpacePublicParameters<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >,
        CommitmentSchemeCommitmentSpacePublicParameters<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >,
        RangeProof::PublicParameters<NUM_RANGE_CLAIMS>,
        UnboundedWitnessSpaceGroupElement::PublicParameters,
        group::PublicParameters<Language::StatementSpaceGroupElement>,
        Language::PublicParameters,
    >;

    const NAME: &'static str = Language::NAME;

    fn randomizer_subrange(
        enhanced_language_public_parameters: &Self::PublicParameters,
    ) -> proofs::Result<(
        Self::WitnessSpaceGroupElement,
        Self::WitnessSpaceGroupElement,
    )> {
        todo!()
        // // TODO
        // // let sampling_bit_size: usize = RangeProof::RANGE_CLAIM_BITS
        // // + ComputationalSecuritySizedNumber::BITS
        // // + StatisticalSecuritySizedNumber::BITS;
        //
        // // TODO: check that this is < SCALAR_LIMBS?
        //
        // // TODO: formula + challenge : in lightning its 1, in bp 128
        // let sampling_bit_size: usize = U128::BITS + StatisticalSecuritySizedNumber::BITS;
        //
        // // TODO: this becomes a problem, as now I don't know how to construct the subrange.
        // // One option is to have the sample get a bit size, not sure how much we wish for that,
        // but // it could help also with the random mod issue.
        // let lower_bound =
        //     ([Uint::<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>::ZERO.into();
        // NUM_RANGE_CLAIMS]).into();
        //
        // let upper_bound = ([(Uint::<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>::ONE <<
        // sampling_bit_size)     .wrapping_sub(&
        // Uint::<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>::ONE)     .into();
        // NUM_RANGE_CLAIMS])     .into();
        //
        // let lower_bound = (
        //     lower_bound,
        //     CommitmentScheme::RandomnessSpaceGroupElement::lower_bound(
        //         enhanced_language_public_parameters
        //             .commitment_scheme_public_parameters
        //             .randomness_space_public_parameters(),
        //     )?,
        //     UnboundedWitnessSpaceGroupElement::lower_bound(
        //         enhanced_language_public_parameters.unbounded_witness_public_parameters(),
        //     )?,
        // )
        //     .into();
        //
        // let upper_bound = (
        //     upper_bound,
        //     CommitmentScheme::RandomnessSpaceGroupElement::upper_bound(
        //         enhanced_language_public_parameters
        //             .commitment_scheme_public_parameters
        //             .randomness_space_public_parameters(),
        //     )?,
        //     UnboundedWitnessSpaceGroupElement::upper_bound(
        //         enhanced_language_public_parameters.unbounded_witness_public_parameters(),
        //     )?,
        // )
        //     .into();
        //
        // Ok((lower_bound, upper_bound))
    }

    fn group_homomorphism(
        witness: &Self::WitnessSpaceGroupElement,
        enhanced_language_public_parameters: &Self::PublicParameters,
    ) -> crate::proofs::Result<Self::StatementSpaceGroupElement> {
        let decomposed_witness: [_; NUM_RANGE_CLAIMS] =
            witness.range_proof_commitment_message().clone().into();

        let decomposed_witness = decomposed_witness
            .map(Into::<Uint<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>>::into);

        let language_witness = Language::compose_witness(
            &decomposed_witness,
            witness.unbounded_witness(),
            &enhanced_language_public_parameters.language_public_parameters,
        )?;

        let language_statement = Language::group_homomorphism(
            &language_witness,
            &enhanced_language_public_parameters.language_public_parameters,
        )?;

        let commitment_scheme = RangeProof::CommitmentScheme::new(
            enhanced_language_public_parameters
                .range_proof_public_parameters
                .commitment_scheme_public_parameters(),
        )?;

        let commitment_message_value =
            <[_; NUM_RANGE_CLAIMS]>::from(witness.range_proof_commitment_message().value()).into();

        let commitment_message = CommitmentSchemeMessageSpaceGroupElement::<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >::new(
            commitment_message_value,
            enhanced_language_public_parameters
                .range_proof_public_parameters
                .commitment_scheme_public_parameters()
                .message_space_public_parameters(),
        )?;

        let range_proof_commitment = commitment_scheme.commit(
            &commitment_message,
            witness.range_proof_commitment_randomness(),
        );

        Ok((range_proof_commitment, language_statement).into())
    }
}

pub trait DecomposableWitness<
    const RANGE_CLAIMS_PER_SCALAR: usize,
    const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    const WITNESS_LIMBS: usize,
>: KnownOrderScalar<WITNESS_LIMBS> where
    Self::Value: From<Uint<WITNESS_LIMBS>>,
{
    fn decompose(
        self,
        range_claim_bits: usize,
    ) -> [Uint<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>; RANGE_CLAIMS_PER_SCALAR] {
        // TODO: sanity checks, return result?
        let witness: Uint<WITNESS_LIMBS> = self.into();

        array::from_fn(|i| {
            Uint::<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>::from(
                &((witness >> (i * range_claim_bits))
                    & ((Uint::<WITNESS_LIMBS>::ONE << range_claim_bits)
                        .wrapping_sub(&Uint::<WITNESS_LIMBS>::ONE))),
            )
        })
    }

    fn compose(
        decomposed_witness: &[Uint<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>;
             RANGE_CLAIMS_PER_SCALAR],
        public_parameters: &Self::PublicParameters,
        range_claim_bits: usize, // TODO:  ???
    ) -> proofs::Result<Self> {
        // TODO: perform all the checks here, checking add - also check that no modulation occursin
        // // LIMBS for the entire computation

        // TODO: COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS < WITNESS_LIMBS
        let delta: Uint<WITNESS_LIMBS> = Uint::<WITNESS_LIMBS>::ONE << range_claim_bits;

        let delta = Self::new(delta.into(), public_parameters)?;

        // TODO: WITNESS_LIMBS < PLAINTEXT_SPACE_SCALAR_LIMBS ?
        let decomposed_witness = decomposed_witness
            .into_iter()
            .map(|witness| {
                Self::new(
                    Uint::<WITNESS_LIMBS>::from(&Uint::<
                        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                    >::from(witness))
                    .into(),
                    public_parameters,
                )
            })
            .collect::<group::Result<Vec<_>>>()?;

        let polynomial = Polynomial::try_from(decomposed_witness)
            .map_err(|_| proofs::Error::InvalidParameters)?;

        Ok(polynomial.evaluate(&delta))
    }
}

impl<
        const RANGE_CLAIMS_PER_SCALAR: usize,
        const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        const WITNESS_LIMBS: usize,
        Witness: KnownOrderScalar<WITNESS_LIMBS>,
    >
    DecomposableWitness<
        RANGE_CLAIMS_PER_SCALAR,
        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        WITNESS_LIMBS,
    > for Witness
where
    Self::Value: From<Uint<WITNESS_LIMBS>>,
{
}

// TODO: accessors

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PublicParameters<
    const REPETITIONS: usize,
    const NUM_RANGE_CLAIMS: usize,
    const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    MessageSpacePublicParameters,
    RandomnessSpacePublicParameters,
    CommitmentSpacePublicParameters,
    RangeProofPublicParameters,
    UnboundedWitnessSpacePublicParameters,
    LanguageStatementSpacePublicParameters,
    LanguagePublicParameters,
> {
    pub groups_public_parameters: GroupsPublicParameters<
        direct_product::ThreeWayPublicParameters<
            MessageSpacePublicParameters,
            RandomnessSpacePublicParameters,
            UnboundedWitnessSpacePublicParameters,
        >,
        direct_product::PublicParameters<
            CommitmentSpacePublicParameters,
            LanguageStatementSpacePublicParameters,
        >,
    >,
    pub range_proof_public_parameters: RangeProofPublicParameters,
    pub language_public_parameters: LanguagePublicParameters,
}

impl<
        const REPETITIONS: usize,
        const NUM_RANGE_CLAIMS: usize,
        const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        MessageSpacePublicParameters: Clone,
        RandomnessSpacePublicParameters: Clone,
        CommitmentSpacePublicParameters: Clone,
        RangeProofPublicParameters,
        UnboundedWitnessSpacePublicParameters,
        LanguageStatementSpacePublicParameters: Clone,
        LanguagePublicParameters,
    >
    PublicParameters<
        REPETITIONS,
        NUM_RANGE_CLAIMS,
        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        MessageSpacePublicParameters,
        RandomnessSpacePublicParameters,
        CommitmentSpacePublicParameters,
        RangeProofPublicParameters,
        UnboundedWitnessSpacePublicParameters,
        LanguageStatementSpacePublicParameters,
        LanguagePublicParameters,
    >
{
    pub fn new<RangeProof, UnboundedWitnessSpaceGroupElement, Language>(
        unbounded_witness_public_parameters: UnboundedWitnessSpacePublicParameters,
        range_proof_public_parameters: RangeProofPublicParameters,
        language_public_parameters: LanguagePublicParameters,
    ) -> Self
    where
        RangeProof: range::RangeProof<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            PublicParameters<NUM_RANGE_CLAIMS> = RangeProofPublicParameters,
        >,
        CommitmentSchemeMessageSpaceGroupElement<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >: group::GroupElement<PublicParameters = MessageSpacePublicParameters>,
        CommitmentSchemeRandomnessSpaceGroupElement<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >: group::GroupElement<PublicParameters = RandomnessSpacePublicParameters>,
        CommitmentSchemeCommitmentSpaceGroupElement<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >: group::GroupElement<PublicParameters = CommitmentSpacePublicParameters>,
        CommitmentSchemePublicParameters<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >: AsRef<
            commitments::GroupsPublicParameters<
                MessageSpacePublicParameters,
                RandomnessSpacePublicParameters,
                CommitmentSpacePublicParameters,
            >,
        >,
        RangeProofPublicParameters: AsRef<
            CommitmentSchemePublicParameters<
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                NUM_RANGE_CLAIMS,
                RangeProof,
            >,
        >,
        UnboundedWitnessSpaceGroupElement:
            group::GroupElement<PublicParameters = UnboundedWitnessSpacePublicParameters>,
        Language: language::Language<REPETITIONS, PublicParameters = LanguagePublicParameters>,
        LanguagePublicParameters: AsRef<
            GroupsPublicParameters<
                group::PublicParameters<Language::WitnessSpaceGroupElement>,
                LanguageStatementSpacePublicParameters,
            >,
        >,
    {
        Self {
            groups_public_parameters: language::GroupsPublicParameters {
                witness_space_public_parameters: (
                    range_proof_public_parameters
                        .commitment_scheme_public_parameters()
                        .message_space_public_parameters()
                        .clone(),
                    range_proof_public_parameters
                        .commitment_scheme_public_parameters()
                        .randomness_space_public_parameters()
                        .clone(),
                    unbounded_witness_public_parameters,
                )
                    .into(),
                statement_space_public_parameters: (
                    range_proof_public_parameters
                        .commitment_scheme_public_parameters()
                        .commitment_space_public_parameters()
                        .clone(),
                    language_public_parameters
                        .statement_space_public_parameters()
                        .clone(),
                )
                    .into(),
            },
            range_proof_public_parameters,
            language_public_parameters,
        }
    }

    pub fn unbounded_witness_public_parameters(&self) -> &UnboundedWitnessSpacePublicParameters {
        let (_, _, unbounded_witness_public_parameters) = (&self
            .groups_public_parameters
            .witness_space_public_parameters)
            .into();

        unbounded_witness_public_parameters
    }
}

impl<
        const REPETITIONS: usize,
        const NUM_RANGE_CLAIMS: usize,
        const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        MessageSpacePublicParameters,
        RandomnessSpacePublicParameters,
        CommitmentSpacePublicParameters,
        CommitmentSchemePublicParameters,
        UnboundedWitnessSpacePublicParameters,
        LanguageStatementSpacePublicParameters,
        LanguagePublicParameters,
    >
    AsRef<
        GroupsPublicParameters<
            direct_product::ThreeWayPublicParameters<
                MessageSpacePublicParameters,
                RandomnessSpacePublicParameters,
                UnboundedWitnessSpacePublicParameters,
            >,
            direct_product::PublicParameters<
                CommitmentSpacePublicParameters,
                LanguageStatementSpacePublicParameters,
            >,
        >,
    >
    for PublicParameters<
        REPETITIONS,
        NUM_RANGE_CLAIMS,
        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        MessageSpacePublicParameters,
        RandomnessSpacePublicParameters,
        CommitmentSpacePublicParameters,
        CommitmentSchemePublicParameters,
        UnboundedWitnessSpacePublicParameters,
        LanguageStatementSpacePublicParameters,
        LanguagePublicParameters,
    >
{
    fn as_ref(
        &self,
    ) -> &GroupsPublicParameters<
        direct_product::ThreeWayPublicParameters<
            MessageSpacePublicParameters,
            RandomnessSpacePublicParameters,
            UnboundedWitnessSpacePublicParameters,
        >,
        direct_product::PublicParameters<
            CommitmentSpacePublicParameters,
            LanguageStatementSpacePublicParameters,
        >,
    > {
        &self.groups_public_parameters
    }
}

pub trait EnhancedLanguageWitnessAccessors<
    MessageSpaceGroupElement: group::GroupElement,
    RandomnessSpaceGroupElement: group::GroupElement,
    UnboundedWitnessSpaceGroupElement: group::GroupElement,
>
{
    fn range_proof_commitment_message(&self) -> &MessageSpaceGroupElement;

    fn range_proof_commitment_randomness(&self) -> &RandomnessSpaceGroupElement;

    fn unbounded_witness(&self) -> &UnboundedWitnessSpaceGroupElement;
}

impl<
        MessageSpaceGroupElement: group::GroupElement,
        RandomnessSpaceGroupElement: group::GroupElement,
        UnboundedWitnessSpaceGroupElement: group::GroupElement,
    >
    EnhancedLanguageWitnessAccessors<
        MessageSpaceGroupElement,
        RandomnessSpaceGroupElement,
        UnboundedWitnessSpaceGroupElement,
    >
    for direct_product::ThreeWayGroupElement<
        MessageSpaceGroupElement,
        RandomnessSpaceGroupElement,
        UnboundedWitnessSpaceGroupElement,
    >
{
    fn range_proof_commitment_message(&self) -> &MessageSpaceGroupElement {
        let (range_proof_commitment_message, ..): (_, _, _) = self.into();

        range_proof_commitment_message
    }

    fn range_proof_commitment_randomness(&self) -> &RandomnessSpaceGroupElement {
        let (_, randomness, _) = self.into();

        randomness
    }

    fn unbounded_witness(&self) -> &UnboundedWitnessSpaceGroupElement {
        let (_, _, unbounded_witness) = self.into();

        unbounded_witness
    }
}

pub trait EnhancedLanguageStatementAccessors<
    CommitmentSpaceGroupElement: group::GroupElement,
    LanguageStatementSpaceGroupElement: group::GroupElement,
>
{
    fn range_proof_commitment(&self) -> &CommitmentSpaceGroupElement;

    fn language_statement(&self) -> &LanguageStatementSpaceGroupElement;
}

impl<
        CommitmentSpaceGroupElement: group::GroupElement,
        LanguageStatementSpaceGroupElement: group::GroupElement,
    >
    EnhancedLanguageStatementAccessors<
        CommitmentSpaceGroupElement,
        LanguageStatementSpaceGroupElement,
    >
    for direct_product::GroupElement<
        CommitmentSpaceGroupElement,
        LanguageStatementSpaceGroupElement,
    >
{
    fn range_proof_commitment(&self) -> &CommitmentSpaceGroupElement {
        let (range_proof_commitment, _) = self.into();

        range_proof_commitment
    }

    fn language_statement(&self) -> &LanguageStatementSpaceGroupElement {
        let (_, language_statement) = self.into();

        language_statement
    }
}

#[cfg(any(test, feature = "benchmarking"))]
pub(crate) mod tests {
    use ahe::paillier::tests::N;
    use crypto_bigint::U256;
    use rand_core::OsRng;

    use super::*;
    use crate::{ahe::GroupsPublicParametersAccessors, group::secp256k1};

    pub const RANGE_CLAIMS_PER_SCALAR: usize = { secp256k1::SCALAR_LIMBS / U128::LIMBS }; // TODO: proper range claims bits

    pub const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize = { U256::LIMBS };

    type EnhancedLang<
        const REPETITIONS: usize,
        const NUM_RANGE_CLAIMS: usize,
        UnboundedWitnessSpaceGroupElement,
        Lang,
    > = EnhancedLanguage<
        REPETITIONS,
        NUM_RANGE_CLAIMS,
        { secp256k1::SCALAR_LIMBS },
        Pedersen<
            NUM_RANGE_CLAIMS,
            { secp256k1::SCALAR_LIMBS },
            secp256k1::Scalar,
            secp256k1::GroupElement,
        >,
        UnboundedWitnessSpaceGroupElement,
        Lang,
    >;

    pub fn scalar_lower_bound() -> paillier::PlaintextSpaceGroupElement {
        let paillier_public_parameters = ahe::paillier::PublicParameters::new(N).unwrap();

        paillier::PlaintextSpaceGroupElement::new(
            Uint::<{ paillier::PLAINTEXT_SPACE_SCALAR_LIMBS }>::ZERO,
            paillier_public_parameters.plaintext_space_public_parameters(),
        )
        .unwrap()
    }

    pub fn scalar_upper_bound() -> paillier::PlaintextSpaceGroupElement {
        let paillier_public_parameters = ahe::paillier::PublicParameters::new(N).unwrap();

        paillier::PlaintextSpaceGroupElement::new(
            (&secp256k1::ORDER.wrapping_sub(&U256::ONE)).into(),
            paillier_public_parameters.plaintext_space_public_parameters(),
        )
        .unwrap()
    }

    pub(crate) fn enhanced_language_public_parameters<
        const REPETITIONS: usize,
        const NUM_RANGE_CLAIMS: usize,
        RangeProof: range::RangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
        UnboundedWitnessSpaceGroupElement: group::GroupElement + SamplableWithin,
        Lang: EnhanceableLanguage<
            REPETITIONS,
            NUM_RANGE_CLAIMS,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RangeProof,
            UnboundedWitnessSpaceGroupElement,
        >,
    >(
        unbounded_witness_public_parameters: UnboundedWitnessSpaceGroupElement::PublicParameters,
        language_public_parameters: Lang::PublicParameters,
    ) -> language::PublicParameters<
        REPETITIONS,
        EnhancedLang<REPETITIONS, NUM_RANGE_CLAIMS, UnboundedWitnessSpaceGroupElement, Lang>,
    > {
        let secp256k1_scalar_public_parameters = secp256k1::scalar::PublicParameters::default();

        let secp256k1_group_public_parameters =
            secp256k1::group_element::PublicParameters::default();

        // TODO: move this shared logic somewhere e.g. DRY
        let generator = secp256k1::GroupElement::new(
            secp256k1_group_public_parameters.generator,
            &secp256k1_group_public_parameters,
        )
        .unwrap();

        let message_generators = array::from_fn(|_| {
            let generator =
                secp256k1::Scalar::sample(&secp256k1_scalar_public_parameters, &mut OsRng).unwrap()
                    * generator;

            generator.value()
        });

        let randomness_generator =
            secp256k1::Scalar::sample(&secp256k1_scalar_public_parameters, &mut OsRng).unwrap()
                * generator;

        // TODO: this is not safe; we need a proper way to derive generators
        let pedersen_public_parameters = pedersen::PublicParameters::new::<
            { secp256k1::SCALAR_LIMBS },
            secp256k1::Scalar,
            secp256k1::GroupElement,
        >(
            secp256k1_scalar_public_parameters.clone(),
            secp256k1_group_public_parameters.clone(),
            message_generators,
            randomness_generator.value(),
        );

        schnorr::enhanced::PublicParameters::new::<
            Pedersen<
                NUM_RANGE_CLAIMS,
                { secp256k1::SCALAR_LIMBS },
                secp256k1::Scalar,
                secp256k1::GroupElement,
            >,
            UnboundedWitnessSpaceGroupElement,
            Lang,
        >(
            unbounded_witness_public_parameters,
            pedersen_public_parameters,
            language_public_parameters,
        )
    }

    pub(crate) fn generate_witnesses<
        const REPETITIONS: usize,
        const NUM_RANGE_CLAIMS: usize,
        const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        CommitmentScheme: HomomorphicCommitmentScheme<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
        RangeProof: range::RangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
        UnboundedWitnessSpaceGroupElement: group::GroupElement + SamplableWithin,
        Lang: EnhanceableLanguage<
            REPETITIONS,
            NUM_RANGE_CLAIMS,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RangeProof,
            UnboundedWitnessSpaceGroupElement,
        >,
    >(
        witnesses: Vec<Lang::WitnessSpaceGroupElement>,
        enhanced_language_public_parameters: &language::PublicParameters<
            REPETITIONS,
            EnhancedLanguage<
                REPETITIONS,
                NUM_RANGE_CLAIMS,
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                CommitmentScheme,
                UnboundedWitnessSpaceGroupElement,
                Lang,
            >,
        >,
    ) -> Vec<
        language::WitnessSpaceGroupElement<
            REPETITIONS,
            EnhancedLanguage<
                REPETITIONS,
                NUM_RANGE_CLAIMS,
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                CommitmentScheme,
                UnboundedWitnessSpaceGroupElement,
                Lang,
            >,
        >,
    > {
        witnesses
            .into_iter()
            .map(|witness| {
                let (range_proof_commitment_message, unbounded_element) = Lang::decompose_witness(
                    &witness,
                    &enhanced_language_public_parameters.language_public_parameters,
                )
                .unwrap();

                let commitment_randomness = CommitmentScheme::RandomnessSpaceGroupElement::sample(
                    enhanced_language_public_parameters
                        .commitment_scheme_public_parameters
                        .randomness_space_public_parameters(),
                    &mut OsRng,
                )
                .unwrap();

                (
                    range_proof_commitment_message,
                    commitment_randomness,
                    unbounded_element,
                )
                    .into()
            })
            .collect()
    }
}
