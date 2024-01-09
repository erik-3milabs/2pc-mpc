// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

use crypto_bigint::rand_core::CryptoRngCore;
use serde::Serialize;

use super::{centralized_party::PublicNonceEncryptedPartialSignatureAndProof, DIMENSION};
use crate::{
    ahe,
    ahe::{AdditivelyHomomorphicDecryptionKeyShare, GroupsPublicParametersAccessors as _},
    commitments,
    commitments::{GroupsPublicParametersAccessors as _, MultiPedersen, Pedersen},
    group,
    group::{AffineXCoordinate, GroupElement, PrimeGroupElement, Samplable},
    helpers::flat_map_results,
    proofs,
    proofs::{
        range,
        range::PublicParametersAccessors,
        schnorr,
        schnorr::{
            committed_linear_evaluation, committment_of_discrete_log,
            discrete_log_ratio_of_committed_values,
            enhanced::EnhanceableLanguage,
            language::{
                committed_linear_evaluation::StatementAccessors as _,
                discrete_log_ratio_of_committed_values::StatementAccessors as _,
            },
        },
    },
    sign::decentralized_party::schnorr::enhanced::EnhancedPublicParameters,
    AdditivelyHomomorphicEncryptionKey,
};

#[cfg_attr(feature = "benchmarking", derive(Clone))]
pub struct Party<
    const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
    const SCALAR_LIMBS: usize,
    const RANGE_CLAIMS_PER_SCALAR: usize,
    const RANGE_CLAIMS_PER_MASK: usize,
    const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    const NUM_RANGE_CLAIMS: usize,
    GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
    EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
    DecryptionKeyShare: AdditivelyHomomorphicDecryptionKeyShare<PLAINTEXT_SPACE_SCALAR_LIMBS, EncryptionKey>,
    UnboundedDComEvalWitness: group::GroupElement + Samplable,
    RangeProof: proofs::RangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
    ProtocolContext: Clone + Serialize,
> {
    pub decryption_key_share: DecryptionKeyShare,
    // TODO: should we get this like that? is it the same for both the centralized & decentralized
    // party (and all their parties?)
    pub protocol_context: ProtocolContext,
    pub scalar_group_public_parameters: group::PublicParameters<GroupElement::Scalar>,
    pub group_public_parameters: GroupElement::PublicParameters,
    pub encryption_scheme_public_parameters: EncryptionKey::PublicParameters,
    // TODO: generate pedersen public parameters instead of getting them
    pub commitment_scheme_public_parameters: commitments::PublicParameters<
        SCALAR_LIMBS,
        Pedersen<1, SCALAR_LIMBS, GroupElement::Scalar, GroupElement>,
    >,
    pub unbounded_dcom_eval_witness_public_parameters: UnboundedDComEvalWitness::PublicParameters,
    pub range_proof_public_parameters: RangeProof::PublicParameters<NUM_RANGE_CLAIMS>,
    pub public_key_share: GroupElement,
    pub nonce_public_share: GroupElement,
    pub encrypted_mask: EncryptionKey::CiphertextSpaceGroupElement,
    pub encrypted_masked_key_share: EncryptionKey::CiphertextSpaceGroupElement,
    pub encrypted_masked_nonce: EncryptionKey::CiphertextSpaceGroupElement,
    pub centralized_party_public_key_share: GroupElement,
    pub centralized_party_nonce_shares_commitment: GroupElement,
}

impl<
        const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
        const SCALAR_LIMBS: usize,
        const RANGE_CLAIMS_PER_SCALAR: usize,
        const RANGE_CLAIMS_PER_MASK: usize,
        const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        const NUM_RANGE_CLAIMS: usize,
        GroupElement: PrimeGroupElement<SCALAR_LIMBS> + AffineXCoordinate<SCALAR_LIMBS>,
        EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
        DecryptionKeyShare: AdditivelyHomomorphicDecryptionKeyShare<PLAINTEXT_SPACE_SCALAR_LIMBS, EncryptionKey>,
        UnboundedDComEvalWitness: group::GroupElement + Samplable,
        RangeProof: proofs::RangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
        ProtocolContext: Clone + Serialize,
    >
    Party<
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIMS_PER_MASK,
        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        NUM_RANGE_CLAIMS,
        GroupElement,
        EncryptionKey,
        DecryptionKeyShare,
        UnboundedDComEvalWitness,
        RangeProof,
        ProtocolContext,
    >
where
    // TODO: I'd love to solve this huge restriction, which seems completely useless to me and is
    // required because Rust.
    committed_linear_evaluation::Language<
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        RANGE_CLAIMS_PER_MASK,
        DIMENSION,
        GroupElement,
        EncryptionKey,
    >: schnorr::Language<
            { committed_linear_evaluation::REPETITIONS },
            WitnessSpaceGroupElement = committed_linear_evaluation::WitnessSpaceGroupElement<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                DIMENSION,
                GroupElement,
                EncryptionKey,
            >,
            StatementSpaceGroupElement = committed_linear_evaluation::StatementSpaceGroupElement<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                DIMENSION,
                GroupElement,
                EncryptionKey,
            >,
            PublicParameters = committed_linear_evaluation::PublicParameters<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                DIMENSION,
                GroupElement,
                EncryptionKey,
            >,
        > + EnhanceableLanguage<
            { committed_linear_evaluation::REPETITIONS },
            NUM_RANGE_CLAIMS,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            UnboundedDComEvalWitness,
        >,
{
    pub fn partially_decrypt_encrypted_signature_parts(
        self,
        message: GroupElement::Scalar,
        public_nonce_encrypted_partial_signature_and_proof: PublicNonceEncryptedPartialSignatureAndProof<
            GroupElement::Value,
            range::CommitmentSchemeCommitmentSpaceValue<
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                NUM_RANGE_CLAIMS,
                RangeProof,
            >,
            ahe::CiphertextSpaceValue<PLAINTEXT_SPACE_SCALAR_LIMBS, EncryptionKey>,
            schnorr::Proof<
                { committment_of_discrete_log::REPETITIONS },
                committment_of_discrete_log::Language<
                    SCALAR_LIMBS,
                    GroupElement::Scalar,
                    GroupElement,
                    Pedersen<1, SCALAR_LIMBS, GroupElement::Scalar, GroupElement>,
                >,
                ProtocolContext,
            >,
            schnorr::Proof<
                { discrete_log_ratio_of_committed_values::REPETITIONS },
                discrete_log_ratio_of_committed_values::Language<
                    SCALAR_LIMBS,
                    GroupElement::Scalar,
                    GroupElement,
                >,
                ProtocolContext,
            >,
            committed_linear_evaluation::EnhancedProof<
                NUM_RANGE_CLAIMS,
                RANGE_CLAIMS_PER_SCALAR,
                RANGE_CLAIMS_PER_MASK,
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                DIMENSION,
                GroupElement,
                EncryptionKey,
                UnboundedDComEvalWitness,
                RangeProof,
                ProtocolContext,
            >,
        >,
        rng: &mut impl CryptoRngCore,
    ) -> crate::Result<(
        DecryptionKeyShare::DecryptionShare,
        DecryptionKeyShare::DecryptionShare,
    )> {
        let language_public_parameters = committment_of_discrete_log::PublicParameters::new::<
            SCALAR_LIMBS,
            GroupElement::Scalar,
            GroupElement,
            Pedersen<1, SCALAR_LIMBS, GroupElement::Scalar, GroupElement>,
        >(
            self.scalar_group_public_parameters.clone(),
            self.group_public_parameters.clone(),
            self.commitment_scheme_public_parameters.clone(),
            public_nonce_encrypted_partial_signature_and_proof.public_nonce,
        );

        public_nonce_encrypted_partial_signature_and_proof
            .public_nonce_proof
            .verify(
                None,
                &self.protocol_context,
                &language_public_parameters,
                vec![[
                    self.nonce_public_share,
                    self.centralized_party_nonce_shares_commitment.clone(),
                ]
                .into()],
            )?;

        let language_public_parameters =
            discrete_log_ratio_of_committed_values::PublicParameters::new::<
                SCALAR_LIMBS,
                GroupElement::Scalar,
                GroupElement,
            >(
                self.scalar_group_public_parameters.clone(),
                self.group_public_parameters.clone(),
                self.commitment_scheme_public_parameters.clone(),
                self.centralized_party_public_key_share,
            );

        let nonce_share_by_key_share_commitment = GroupElement::new(
            public_nonce_encrypted_partial_signature_and_proof.nonce_share_by_key_share_commitment,
            &self.group_public_parameters,
        )?;

        public_nonce_encrypted_partial_signature_and_proof
            .nonce_share_by_key_share_proof
            .verify(
                None,
                &self.protocol_context,
                &language_public_parameters,
                vec![[
                    self.centralized_party_nonce_shares_commitment.clone(),
                    nonce_share_by_key_share_commitment.clone(),
                ]
                .into()],
            )?;

        let ciphertexts =
            [self.encrypted_mask, self.encrypted_masked_key_share].map(|ct| ct.value());

        let coefficient_commitments = flat_map_results(
            [
                public_nonce_encrypted_partial_signature_and_proof.first_coefficient_commitment,
                public_nonce_encrypted_partial_signature_and_proof.second_coefficient_commitment,
            ]
            .map(|value| {
                GroupElement::new(
                    public_nonce_encrypted_partial_signature_and_proof
                        .nonce_share_by_key_share_commitment,
                    &self.group_public_parameters,
                )
            }),
        )?;

        // TODO: From.
        let commitment_scheme_public_parameters =
            commitments::PublicParameters::<
                SCALAR_LIMBS,
                MultiPedersen<DIMENSION, SCALAR_LIMBS, GroupElement::Scalar, GroupElement>,
            >::new::<SCALAR_LIMBS, GroupElement::Scalar, GroupElement>(
                self.scalar_group_public_parameters.clone(),
                self.group_public_parameters.clone(),
                self.commitment_scheme_public_parameters.message_generators[0],
                self.commitment_scheme_public_parameters
                    .randomness_generator,
            );

        let language_public_parameters =
            committed_linear_evaluation::PublicParameters::<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                DIMENSION,
                GroupElement,
                EncryptionKey,
            >::new::<PLAINTEXT_SPACE_SCALAR_LIMBS, SCALAR_LIMBS, GroupElement, EncryptionKey>(
                self.scalar_group_public_parameters.clone(),
                self.group_public_parameters.clone(),
                self.encryption_scheme_public_parameters.clone(),
                commitment_scheme_public_parameters,
                ciphertexts,
            );

        let language_public_parameters = EnhancedPublicParameters::<
            { committed_linear_evaluation::REPETITIONS },
            NUM_RANGE_CLAIMS,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RangeProof,
            UnboundedDComEvalWitness,
            committed_linear_evaluation::Language<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                RANGE_CLAIMS_PER_SCALAR,
                RANGE_CLAIMS_PER_MASK,
                DIMENSION,
                GroupElement,
                EncryptionKey,
            >,
        >::new::<
            RangeProof,
            UnboundedDComEvalWitness,
            committed_linear_evaluation::Language<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                RANGE_CLAIMS_PER_SCALAR,
                RANGE_CLAIMS_PER_MASK,
                DIMENSION,
                GroupElement,
                EncryptionKey,
            >,
        >(
            self.unbounded_dcom_eval_witness_public_parameters.clone(),
            self.range_proof_public_parameters.clone(),
            language_public_parameters,
        );

        let encrypted_partial_signature = EncryptionKey::CiphertextSpaceGroupElement::new(
            public_nonce_encrypted_partial_signature_and_proof.encrypted_partial_signature,
            self.encryption_scheme_public_parameters
                .ciphertext_space_public_parameters(),
        )?;

        let range_proof_commitment = range::CommitmentSchemeCommitmentSpaceGroupElement::<
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            NUM_RANGE_CLAIMS,
            RangeProof,
        >::new(
            public_nonce_encrypted_partial_signature_and_proof
                .encrypted_partial_signature_range_proof_commitment,
            &self
                .range_proof_public_parameters
                .commitment_scheme_public_parameters()
                .commitment_space_public_parameters(),
        )?;

        public_nonce_encrypted_partial_signature_and_proof
            .encrypted_partial_signature_proof
            .verify(
                // TODO: there actually are `n` parties, but we don't know how many, so what to do
                // here?
                None,
                &self.protocol_context,
                &language_public_parameters,
                vec![(
                    range_proof_commitment,
                    (
                        encrypted_partial_signature.clone(),
                        coefficient_commitments.clone().into(),
                    )
                        .into(),
                )
                    .into()],
                rng,
            )?;

        // TODO: "verifies that the values used in the proofs are consistent with values obtained
        // previously" - did I cover this already by taking values from the party struct, or do I
        // need to do it explicitly as stated in the paper, where you seek "records" holding more
        // info?

        let public_nonce = GroupElement::new(
            public_nonce_encrypted_partial_signature_and_proof.public_nonce,
            &self.group_public_parameters,
        )?; // $R$

        let nonce_x_coordinate = public_nonce.x(); // $r$

        if coefficient_commitments[0]
            != ((nonce_x_coordinate * nonce_share_by_key_share_commitment)
                + (message * &self.centralized_party_nonce_shares_commitment))
            || coefficient_commitments[1]
                != (nonce_x_coordinate * &self.centralized_party_nonce_shares_commitment)
        {
            return Err(crate::Error::CommitmentsHomomorphicEvaluation);
        }

        let partial_signature_decryption_share = self
            .decryption_key_share
            .generate_decryption_share_semi_honest(&encrypted_partial_signature)?;

        let masked_nonce_decryption_share = self
            .decryption_key_share
            .generate_decryption_share_semi_honest(&self.encrypted_masked_nonce)?;

        Ok((
            partial_signature_decryption_share,
            masked_nonce_decryption_share,
        ))
    }
}
