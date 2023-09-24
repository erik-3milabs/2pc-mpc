// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

use crypto_bigint::{rand_core::CryptoRngCore, Encoding, Uint};

use crate::{
    commitments::HomomorphicCommitmentScheme,
    group::{additive_group_of_integers_modulu_n::power_of_two_moduli, self_product},
    proofs::Result,
};

pub trait RangeProof<
    // The number of witnesses with range claims
    const NUM_RANGE_CLAIMS: usize,
    // An upper bound over the range claims
    const RANGE_CLAIM_LIMBS: usize,
>: PartialEq + Clone where
    Uint<RANGE_CLAIM_LIMBS>: Encoding,
{
    /// The commitment scheme used for the range proof
    type CommitmentScheme: HomomorphicCommitmentScheme<
    MessageSpaceGroupElement = self_product::GroupElement<
    NUM_RANGE_CLAIMS,
    power_of_two_moduli::GroupElement<RANGE_CLAIM_LIMBS>,
    >,
    >;
    // TODO: protocol context? needed or not?

    /// Proves in zero-knowledge that all witnesses committed in `commitment` are bounded by their corresponding
    /// range upper bound in range_claims.
    fn prove(
        witnesses_and_range_claims: Vec<[(power_of_two_moduli::GroupElement<RANGE_CLAIM_LIMBS>, Uint<RANGE_CLAIM_LIMBS>); NUM_RANGE_CLAIMS]>,
        commitment_randomness: &Self::CommitmentScheme::RandomnessSpaceGroupElement, // TODO: one for all?
        commitment: &Self::CommitmentScheme::CommitmentSpaceGroupElement, // TODO: one for all?
        rng: &mut impl CryptoRngCore,
    ) -> Result<Self>;

    /// Verifies that all witnesses committed in `commitment` are bounded by their corresponding
    /// range upper bound in range_claims.
    fn verify(
        &self,
        range_claims: Vec<[Uint<RANGE_CLAIM_LIMBS>; NUM_RANGE_CLAIMS]>,
        commitment: &Self::CommitmentScheme::CommitmentSpaceGroupElement,
        rng: &mut impl CryptoRngCore,
    ) -> Result<()>;
}
