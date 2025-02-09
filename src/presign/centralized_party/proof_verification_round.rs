// Author: dWallet Labs, Ltd.
// SPDX-License-Identifier: BSD-3-Clause-Clear

#![allow(clippy::type_complexity)]

use commitment::GroupsPublicParametersAccessors;
use crypto_bigint::{rand_core::CryptoRngCore, Encoding, Uint};
use enhanced_maurer::{
    encryption_of_discrete_log, encryption_of_tuple, language::composed_witness_upper_bound,
    EnhanceableLanguage, EnhancedPublicParameters,
};
use group::{GroupElement as _, PrimeGroupElement, Samplable};
use homomorphic_encryption::{
    AdditivelyHomomorphicEncryptionKey, GroupsPublicParametersAccessors as _,
};
use maurer::SOUND_PROOFS_REPETITIONS;
use proof::{range::PublicParametersAccessors, AggregatableRangeProof};
use serde::Serialize;

use crate::{
    dkg,
    presign::{centralized_party::Presign, decentralized_party},
    Error, ProtocolPublicParameters,
};
#[cfg_attr(feature = "benchmarking", derive(Clone))]
pub struct Party<
    const SCALAR_LIMBS: usize,
    const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    const RANGE_CLAIMS_PER_SCALAR: usize,
    const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
    GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
    EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
    RangeProof: AggregatableRangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
    UnboundedEncDLWitness: group::GroupElement + Samplable,
    UnboundedEncDHWitness: group::GroupElement + Samplable,
    ProtocolContext: Clone + Serialize,
> {
    pub(super) protocol_context: ProtocolContext,
    pub(super) scalar_group_public_parameters: group::PublicParameters<GroupElement::Scalar>,
    pub(super) group_public_parameters: GroupElement::PublicParameters,
    pub(super) encryption_scheme_public_parameters: EncryptionKey::PublicParameters,
    pub(super) unbounded_encdl_witness_public_parameters: UnboundedEncDLWitness::PublicParameters,
    pub(super) unbounded_encdh_witness_public_parameters: UnboundedEncDHWitness::PublicParameters,
    pub(super) range_proof_public_parameters: RangeProof::PublicParameters<RANGE_CLAIMS_PER_SCALAR>,
    pub signature_nonce_shares_and_commitment_randomnesses:
        Vec<(GroupElement::Scalar, GroupElement::Scalar)>,
    pub(super) encrypted_decentralized_party_secret_key_share:
        EncryptionKey::CiphertextSpaceGroupElement,
}

impl<
        const SCALAR_LIMBS: usize,
        const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        const RANGE_CLAIMS_PER_SCALAR: usize,
        const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
        GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
        EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
        RangeProof: AggregatableRangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
        UnboundedEncDLWitness: group::GroupElement + Samplable,
        UnboundedEncDHWitness: group::GroupElement + Samplable,
        ProtocolContext: Clone + Serialize,
    >
    Party<
        SCALAR_LIMBS,
        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        GroupElement,
        EncryptionKey,
        RangeProof,
        UnboundedEncDLWitness,
        UnboundedEncDHWitness,
        ProtocolContext,
    >
where
    encryption_of_discrete_log::Language<
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        SCALAR_LIMBS,
        GroupElement,
        EncryptionKey,
    >: maurer::Language<
            SOUND_PROOFS_REPETITIONS,
            WitnessSpaceGroupElement = encryption_of_discrete_log::WitnessSpaceGroupElement<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                EncryptionKey,
            >,
            StatementSpaceGroupElement = encryption_of_discrete_log::StatementSpaceGroupElement<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >,
            PublicParameters = encryption_of_discrete_log::PublicParameters<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >,
        > + EnhanceableLanguage<
            SOUND_PROOFS_REPETITIONS,
            RANGE_CLAIMS_PER_SCALAR,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            UnboundedEncDLWitness,
        >,
    encryption_of_tuple::Language<
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        SCALAR_LIMBS,
        GroupElement,
        EncryptionKey,
    >: maurer::Language<
            SOUND_PROOFS_REPETITIONS,
            WitnessSpaceGroupElement = encryption_of_tuple::WitnessSpaceGroupElement<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                EncryptionKey,
            >,
            StatementSpaceGroupElement = encryption_of_tuple::StatementSpaceGroupElement<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                EncryptionKey,
            >,
            PublicParameters = encryption_of_tuple::PublicParameters<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >,
        > + EnhanceableLanguage<
            SOUND_PROOFS_REPETITIONS,
            RANGE_CLAIMS_PER_SCALAR,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            UnboundedEncDHWitness,
        >,
    Uint<PLAINTEXT_SPACE_SCALAR_LIMBS>: Encoding,
{
    pub fn verify_presign_output(
        self,
        output: decentralized_party::Output<
            GroupElement::Value,
            proof::range::CommitmentSchemeCommitmentSpaceValue<
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                RANGE_CLAIMS_PER_SCALAR,
                RangeProof,
            >,
            homomorphic_encryption::CiphertextSpaceValue<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                EncryptionKey,
            >,
            encryption_of_tuple::Proof<
                RANGE_CLAIMS_PER_SCALAR,
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
                RangeProof,
                UnboundedEncDHWitness,
                ProtocolContext,
            >,
            encryption_of_discrete_log::Proof<
                RANGE_CLAIMS_PER_SCALAR,
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
                RangeProof,
                UnboundedEncDLWitness,
                ProtocolContext,
            >,
        >,
        rng: &mut impl CryptoRngCore,
    ) -> crate::Result<
        Vec<
            Presign<
                GroupElement::Value,
                group::Value<GroupElement::Scalar>,
                homomorphic_encryption::CiphertextSpaceValue<
                    PLAINTEXT_SPACE_SCALAR_LIMBS,
                    EncryptionKey,
                >,
            >,
        >,
    > {
        let batch_size = self
            .signature_nonce_shares_and_commitment_randomnesses
            .len();

        if output.encrypted_masked_key_shares.len() != batch_size
            || output.encrypted_masks.len() != batch_size
            || output.nonce_public_shares.len() != batch_size
        {
            return Err(Error::InvalidParameters);
        }

        let encrypted_masks = output
            .encrypted_masks
            .clone()
            .into_iter()
            .map(|encrypted_mask| {
                EncryptionKey::CiphertextSpaceGroupElement::new(
                    encrypted_mask,
                    self.encryption_scheme_public_parameters
                        .ciphertext_space_public_parameters(),
                )
            })
            .collect::<group::Result<Vec<_>>>()?;

        let encrypted_masked_key_shares = output
            .encrypted_masked_key_shares
            .clone()
            .into_iter()
            .map(|encrypted_masked_key_share| {
                EncryptionKey::CiphertextSpaceGroupElement::new(
                    encrypted_masked_key_share,
                    self.encryption_scheme_public_parameters
                        .ciphertext_space_public_parameters(),
                )
            })
            .collect::<group::Result<Vec<_>>>()?;

        let key_share_masking_range_proof_commitments = output
            .key_share_masking_range_proof_commitments
            .into_iter()
            .map(|key_share_masking_range_proof_commitment| {
                proof::range::CommitmentSchemeCommitmentSpaceGroupElement::<
                    COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                    RANGE_CLAIMS_PER_SCALAR,
                    RangeProof,
                >::new(
                    key_share_masking_range_proof_commitment,
                    self.range_proof_public_parameters
                        .commitment_scheme_public_parameters()
                        .commitment_space_public_parameters(),
                )
            })
            .collect::<group::Result<Vec<_>>>()?;

        let statements = encrypted_masks
            .into_iter()
            .zip(encrypted_masked_key_shares)
            .zip(key_share_masking_range_proof_commitments)
            .map(
                |(
                    (encrypted_mask, encrypted_masked_key_share),
                    key_share_masking_range_proof_commitment,
                )| {
                    (
                        key_share_masking_range_proof_commitment,
                        [encrypted_mask, encrypted_masked_key_share].into(),
                    )
                        .into()
                },
            )
            .collect();

        let encrypted_secret_key_share_upper_bound = composed_witness_upper_bound::<
            RANGE_CLAIMS_PER_SCALAR,
            PLAINTEXT_SPACE_SCALAR_LIMBS,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RangeProof,
        >()?;

        let language_public_parameters = encryption_of_tuple::PublicParameters::<
            PLAINTEXT_SPACE_SCALAR_LIMBS,
            SCALAR_LIMBS,
            GroupElement,
            EncryptionKey,
        >::new::<SCALAR_LIMBS, GroupElement, EncryptionKey>(
            self.scalar_group_public_parameters.clone(),
            self.encryption_scheme_public_parameters.clone(),
            self.encrypted_decentralized_party_secret_key_share.value(),
            encrypted_secret_key_share_upper_bound,
        );

        let language_public_parameters = EnhancedPublicParameters::<
            SOUND_PROOFS_REPETITIONS,
            RANGE_CLAIMS_PER_SCALAR,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RangeProof,
            UnboundedEncDHWitness,
            encryption_of_tuple::Language<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >,
        >::new::<
            RangeProof,
            UnboundedEncDHWitness,
            encryption_of_tuple::Language<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >,
        >(
            self.unbounded_encdh_witness_public_parameters.clone(),
            self.range_proof_public_parameters.clone(),
            language_public_parameters,
        )?;

        output.masks_and_encrypted_masked_key_share_proof.verify(
            &self.protocol_context,
            &language_public_parameters,
            statements,
            rng,
        )?;

        let encrypted_nonces = output
            .encrypted_nonces
            .clone()
            .into_iter()
            .map(|encrypted_nonce| {
                EncryptionKey::CiphertextSpaceGroupElement::new(
                    encrypted_nonce,
                    self.encryption_scheme_public_parameters
                        .ciphertext_space_public_parameters(),
                )
            })
            .collect::<group::Result<Vec<_>>>()?;

        let decentralized_party_nonce_public_shares = output
            .nonce_public_shares
            .clone()
            .into_iter()
            .map(|nonce_public_share| {
                GroupElement::new(nonce_public_share, &self.group_public_parameters)
            })
            .collect::<group::Result<Vec<_>>>()?;

        let nonce_sharing_range_proof_commitments = output
            .nonce_sharing_range_proof_commitments
            .into_iter()
            .map(|nonce_sharing_range_proof_commitment| {
                proof::range::CommitmentSchemeCommitmentSpaceGroupElement::<
                    COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                    RANGE_CLAIMS_PER_SCALAR,
                    RangeProof,
                >::new(
                    nonce_sharing_range_proof_commitment,
                    self.range_proof_public_parameters
                        .commitment_scheme_public_parameters()
                        .commitment_space_public_parameters(),
                )
            })
            .collect::<group::Result<Vec<_>>>()?;

        let statements = encrypted_nonces
            .into_iter()
            .zip(decentralized_party_nonce_public_shares)
            .zip(nonce_sharing_range_proof_commitments)
            .map(
                |((encrypted_nonce, nonce_public_share), nonce_sharing_range_proof_commitment)| {
                    (
                        nonce_sharing_range_proof_commitment,
                        (encrypted_nonce, nonce_public_share).into(),
                    )
                        .into()
                },
            )
            .collect();

        let language_public_parameters =
            encryption_of_discrete_log::PublicParameters::<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >::new::<PLAINTEXT_SPACE_SCALAR_LIMBS, SCALAR_LIMBS, GroupElement, EncryptionKey>(
                self.scalar_group_public_parameters.clone(),
                self.group_public_parameters.clone(),
                self.encryption_scheme_public_parameters.clone(),
                GroupElement::generator_value_from_public_parameters(&self.group_public_parameters),
            );

        let language_public_parameters = EnhancedPublicParameters::<
            SOUND_PROOFS_REPETITIONS,
            RANGE_CLAIMS_PER_SCALAR,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RangeProof,
            UnboundedEncDLWitness,
            encryption_of_discrete_log::Language<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >,
        >::new::<
            RangeProof,
            UnboundedEncDLWitness,
            encryption_of_discrete_log::Language<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >,
        >(
            self.unbounded_encdl_witness_public_parameters.clone(),
            self.range_proof_public_parameters.clone(),
            language_public_parameters,
        )?;

        output
            .encrypted_nonce_shares_and_public_shares_proof
            .verify(
                &self.protocol_context,
                &language_public_parameters,
                statements,
                rng,
            )?;

        Ok(output
            .nonce_public_shares
            .into_iter()
            .zip(
                output.encrypted_masks.into_iter().zip(
                    output
                        .encrypted_masked_key_shares
                        .into_iter()
                        .zip(self.signature_nonce_shares_and_commitment_randomnesses),
                ),
            )
            .map(
                |(
                    decentralized_party_nonce_public_share,
                    (
                        encrypted_mask,
                        (encrypted_masked_key_share, (nonce_share, commitment_randomness)),
                    ),
                )| {
                    Presign {
                        nonce_share: nonce_share.value(),
                        decentralized_party_nonce_public_share,
                        encrypted_mask,
                        encrypted_masked_key_share,
                        commitment_randomness: commitment_randomness.value(),
                    }
                },
            )
            .collect())
    }

    pub fn new<
        const NUM_RANGE_CLAIMS: usize,
        UnboundedDComEvalWitness: group::GroupElement + Samplable,
    >(
        signature_nonce_shares_and_commitment_randomnesses: Vec<(
            GroupElement::Scalar,
            GroupElement::Scalar,
        )>,
        protocol_context: ProtocolContext,
        protocol_public_parameters: ProtocolPublicParameters<
            SCALAR_LIMBS,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            RANGE_CLAIMS_PER_SCALAR,
            NUM_RANGE_CLAIMS,
            PLAINTEXT_SPACE_SCALAR_LIMBS,
            GroupElement,
            EncryptionKey,
            RangeProof,
            UnboundedEncDLWitness,
            UnboundedEncDHWitness,
            UnboundedDComEvalWitness,
        >,
        dkg_output: dkg::centralized_party::Output<
            GroupElement::Value,
            group::Value<GroupElement::Scalar>,
            group::Value<EncryptionKey::CiphertextSpaceGroupElement>,
        >,
    ) -> crate::Result<Self> {
        let encryption_scheme_public_parameters =
            protocol_public_parameters.encryption_scheme_public_parameters;

        let encrypted_decentralized_party_secret_key_share =
            EncryptionKey::CiphertextSpaceGroupElement::new(
                dkg_output.encrypted_decentralized_party_secret_key_share,
                encryption_scheme_public_parameters.ciphertext_space_public_parameters(),
            )?;

        Ok(Self {
            protocol_context,
            scalar_group_public_parameters: protocol_public_parameters
                .scalar_group_public_parameters,
            group_public_parameters: protocol_public_parameters.group_public_parameters,
            encryption_scheme_public_parameters,
            unbounded_encdl_witness_public_parameters: protocol_public_parameters
                .unbounded_encdl_witness_public_parameters,
            unbounded_encdh_witness_public_parameters: protocol_public_parameters
                .unbounded_encdh_witness_public_parameters,
            range_proof_public_parameters: protocol_public_parameters
                .range_proof_enc_dl_public_parameters,
            signature_nonce_shares_and_commitment_randomnesses,
            encrypted_decentralized_party_secret_key_share,
        })
    }
}
