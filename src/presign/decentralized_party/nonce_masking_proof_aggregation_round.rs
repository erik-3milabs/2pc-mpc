// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

use core::array;
use std::collections::HashMap;

use crypto_bigint::{rand_core::CryptoRngCore, Encoding, Uint};
use serde::Serialize;

use crate::{
    ahe, commitments,
    commitments::GroupsPublicParametersAccessors as _,
    dkg::decentralized_party::decommitment_round,
    group,
    group::{
        additive_group_of_integers_modulu_n::power_of_two_moduli, GroupElement as _,
        PrimeGroupElement, Samplable,
    },
    presign::decentralized_party::{
        nonce_masking_proof_share_round, nonce_sharing_and_key_share_masking_decommitment_round,
        nonce_sharing_and_key_share_masking_proof_share_round,
    },
    proofs,
    proofs::schnorr::{
        encryption_of_discrete_log, encryption_of_tuple,
        knowledge_of_decommitment::LanguageCommitmentScheme,
        language::{enhanced, enhanced::EnhancedLanguageStatementAccessors},
    },
    AdditivelyHomomorphicEncryptionKey, Commitment, PartyID,
};

#[cfg_attr(feature = "benchmarking", derive(Clone))]
pub struct Party<
    const SCALAR_LIMBS: usize,
    const RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    const RANGE_CLAIMS_PER_SCALAR: usize,
    const RANGE_CLAIM_LIMBS: usize,
    const WITNESS_MASK_LIMBS: usize,
    const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
    GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
    EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
    RangeProof: proofs::RangeProof<RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS, RANGE_CLAIM_LIMBS>,
    CommitmentScheme: LanguageCommitmentScheme<SCALAR_LIMBS, 1, GroupElement::Scalar, GroupElement>,
    ProtocolContext: Clone + Serialize,
> where
    Uint<RANGE_CLAIM_LIMBS>: Encoding,
    Uint<WITNESS_MASK_LIMBS>: Encoding,
    group::ScalarValue<SCALAR_LIMBS, GroupElement>: From<Uint<SCALAR_LIMBS>>,
    range::CommitmentSchemeMessageSpaceValue<
        RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIM_LIMBS,
        RangeProof,
    >: From<enhanced::ConstrainedWitnessValue<RANGE_CLAIMS_PER_SCALAR, WITNESS_MASK_LIMBS>>,
{
    pub(super) party_id: PartyID,
    pub(super) threshold: PartyID,
    pub(super) number_of_parties: PartyID,
    // TODO: should we get this like that?
    pub(super) protocol_context: ProtocolContext,
    pub(super) group_public_parameters: GroupElement::PublicParameters,
    pub(super) scalar_group_public_parameters: group::PublicParameters<GroupElement::Scalar>,
    pub(super) encryption_scheme_public_parameters: EncryptionKey::PublicParameters,
    pub(super) commitment_scheme_public_parameters: CommitmentScheme::PublicParameters,
    pub(super) range_proof_public_parameters: RangeProof::PublicParameters<RANGE_CLAIMS_PER_SCALAR>,
    pub(super) public_key_share: GroupElement,
    pub(super) public_key: GroupElement,
    pub(super) encryption_of_secret_key_share: EncryptionKey::CiphertextSpaceGroupElement,
    pub(super) centralized_party_public_key_share: GroupElement,
    pub(super) nonce_public_shares: Vec<GroupElement>,
    pub(super) masks_encryptions: Vec<EncryptionKey::CiphertextSpaceGroupElement>,
    pub(super) masked_key_share_encryptions: Vec<EncryptionKey::CiphertextSpaceGroupElement>,
    pub(super) nonce_sharing_proof: encryption_of_discrete_log::Proof<
        SCALAR_LIMBS,
        RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIM_LIMBS,
        WITNESS_MASK_LIMBS,
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        GroupElement::Scalar,
        GroupElement,
        EncryptionKey,
        RangeProof,
        ProtocolContext,
    >,
    pub(super) key_share_masking_proof: encryption_of_tuple::Proof<
        SCALAR_LIMBS,
        RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIM_LIMBS,
        WITNESS_MASK_LIMBS,
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        GroupElement::Scalar,
        GroupElement,
        EncryptionKey,
        RangeProof,
        ProtocolContext,
    >,
    pub(super) nonce_masking_proof_aggregation_round_party:
        encryption_of_tuple::ProofAggregationProofAggregationRoundParty<
            SCALAR_LIMBS,
            RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RANGE_CLAIMS_PER_SCALAR,
            RANGE_CLAIM_LIMBS,
            WITNESS_MASK_LIMBS,
            PLAINTEXT_SPACE_SCALAR_LIMBS,
            GroupElement::Scalar,
            GroupElement,
            EncryptionKey,
            RangeProof,
            ProtocolContext,
        >,
}

// TODO: derive serialize for all these structs.
#[derive(Clone)]
pub struct Output<
    const SCALAR_LIMBS: usize,
    const RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    const RANGE_CLAIMS_PER_SCALAR: usize,
    const RANGE_CLAIM_LIMBS: usize,
    const WITNESS_MASK_LIMBS: usize,
    const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
    GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
    EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
    RangeProof: proofs::RangeProof<RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS, RANGE_CLAIM_LIMBS>,
    ProtocolContext: Clone + Serialize,
> where
    Uint<RANGE_CLAIM_LIMBS>: Encoding,
    Uint<WITNESS_MASK_LIMBS>: Encoding,
    group::ScalarValue<SCALAR_LIMBS, GroupElement>: From<Uint<SCALAR_LIMBS>>,
    range::CommitmentSchemeMessageSpaceValue<
        RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIM_LIMBS,
        RangeProof,
    >: From<enhanced::ConstrainedWitnessValue<RANGE_CLAIMS_PER_SCALAR, WITNESS_MASK_LIMBS>>,
{
    pub nonce_public_shares: Vec<GroupElement>,
    pub masks_encryptions: Vec<EncryptionKey::CiphertextSpaceGroupElement>,
    pub masked_key_share_encryptions: Vec<EncryptionKey::CiphertextSpaceGroupElement>,
    pub(super) nonce_sharing_proof: encryption_of_discrete_log::Proof<
        SCALAR_LIMBS,
        RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIM_LIMBS,
        WITNESS_MASK_LIMBS,
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        GroupElement::Scalar,
        GroupElement,
        EncryptionKey,
        RangeProof,
        ProtocolContext,
    >,
    pub(super) key_share_masking_proof: encryption_of_tuple::Proof<
        SCALAR_LIMBS,
        RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIM_LIMBS,
        WITNESS_MASK_LIMBS,
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        GroupElement::Scalar,
        GroupElement,
        EncryptionKey,
        RangeProof,
        ProtocolContext,
    >,
}

#[derive(Clone)]
pub struct Presign<
    const SCALAR_LIMBS: usize,
    const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
    GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
    EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
> {
    pub(crate) nonce_public_share: GroupElement,
    pub(crate) mask_encryption: EncryptionKey::CiphertextSpaceGroupElement,
    pub(crate) masked_key_share_encryption: EncryptionKey::CiphertextSpaceGroupElement,
    pub(crate) masked_nonce_encryption: EncryptionKey::CiphertextSpaceGroupElement,
}

impl<
        const SCALAR_LIMBS: usize,
        const RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        const RANGE_CLAIMS_PER_SCALAR: usize,
        const RANGE_CLAIM_LIMBS: usize,
        const WITNESS_MASK_LIMBS: usize,
        const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
        GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
        EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
        RangeProof: proofs::RangeProof<
            RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RANGE_CLAIM_LIMBS,
        >,
        CommitmentScheme: LanguageCommitmentScheme<SCALAR_LIMBS, 1, GroupElement::Scalar, GroupElement>,
        ProtocolContext: Clone + Serialize,
    >
    Party<
        SCALAR_LIMBS,
        RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIM_LIMBS,
        WITNESS_MASK_LIMBS,
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        GroupElement,
        EncryptionKey,
        RangeProof,
        CommitmentScheme,
        ProtocolContext,
    >
where
    Uint<RANGE_CLAIM_LIMBS>: Encoding,
    Uint<WITNESS_MASK_LIMBS>: Encoding,
    group::ScalarValue<SCALAR_LIMBS, GroupElement>: From<Uint<SCALAR_LIMBS>>,
    range::CommitmentSchemeMessageSpaceValue<
        RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIM_LIMBS,
        RangeProof,
    >: From<enhanced::ConstrainedWitnessValue<RANGE_CLAIMS_PER_SCALAR, WITNESS_MASK_LIMBS>>,
{
    pub fn aggregate_proof_shares(
        self,
        proof_shares: HashMap<
            PartyID,
            encryption_of_tuple::ProofShare<
                SCALAR_LIMBS,
                RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                RANGE_CLAIMS_PER_SCALAR,
                RANGE_CLAIM_LIMBS,
                WITNESS_MASK_LIMBS,
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                GroupElement::Scalar,
                GroupElement,
                EncryptionKey,
                RangeProof,
            >,
        >,
    ) -> crate::Result<(
        Vec<Presign<SCALAR_LIMBS, PLAINTEXT_SPACE_SCALAR_LIMBS, GroupElement, EncryptionKey>>,
        Output<
            SCALAR_LIMBS,
            RANGE_PROOF_COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RANGE_CLAIMS_PER_SCALAR,
            RANGE_CLAIM_LIMBS,
            WITNESS_MASK_LIMBS,
            PLAINTEXT_SPACE_SCALAR_LIMBS,
            GroupElement,
            EncryptionKey,
            RangeProof,
            ProtocolContext,
        >,
    )> {
        let (nonce_masking_proof, statements) = self
            .nonce_masking_proof_aggregation_round_party
            .aggregate_proof_shares(proof_shares)?;

        let masked_nonces_encryptions: Vec<_> = statements
            .into_iter()
            .map(|statement| {
                let as_array: &[_; 2] = statement.remaining_statement().into();

                as_array[1].clone()
            })
            .collect();

        let presigns = self
            .nonce_public_shares
            .clone()
            .into_iter()
            .zip(
                self.masks_encryptions.clone().into_iter().zip(
                    self.masked_key_share_encryptions
                        .clone()
                        .into_iter()
                        .zip(masked_nonces_encryptions.clone().into_iter()),
                ),
            )
            .map(
                |(
                    nonce_public_share,
                    (mask_encryption, (masked_key_share_encryption, masked_nonce_encryption)),
                )| Presign {
                    nonce_public_share,
                    mask_encryption,
                    masked_key_share_encryption,
                    masked_nonce_encryption,
                },
            )
            .collect();

        // TODO: maybe move this to a first-half of the protocol.
        let output = Output {
            nonce_public_shares: self.nonce_public_shares,
            masks_encryptions: self.masks_encryptions,
            masked_key_share_encryptions: self.masked_key_share_encryptions,
            nonce_sharing_proof: self.nonce_sharing_proof,
            key_share_masking_proof: self.key_share_masking_proof,
        };

        // TODO: create output struct, save statements.
        Ok((presigns, output))
    }
}
