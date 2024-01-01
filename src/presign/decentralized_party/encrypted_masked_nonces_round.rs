use std::collections::HashMap;

use crypto_bigint::{rand_core::CryptoRngCore, Encoding, Uint};
use serde::Serialize;

use crate::{
    ahe,
    ahe::GroupsPublicParametersAccessors,
    commitments,
    commitments::{GroupsPublicParametersAccessors as _, Pedersen},
    group,
    group::{
        additive_group_of_integers_modulu_n::power_of_two_moduli, GroupElement as _,
        PrimeGroupElement, Samplable,
    },
    proofs,
    proofs::{
        range, schnorr,
        schnorr::{
            encryption_of_discrete_log, encryption_of_tuple,
            enhanced::{EnhanceableLanguage, EnhancedLanguage, EnhancedPublicParameters},
            language::{
                encryption_of_discrete_log::StatementAccessors as _,
                encryption_of_tuple::StatementAccessors as _,
            },
        },
    },
    AdditivelyHomomorphicEncryptionKey, Commitment, PartyID,
};

#[cfg_attr(feature = "benchmarking", derive(Clone))]
pub struct Party<
    const SCALAR_LIMBS: usize,
    const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
    const RANGE_CLAIMS_PER_SCALAR: usize,
    const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
    GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
    EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
    UnboundedEncDHWitness: group::GroupElement + Samplable,
    RangeProof: proofs::RangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
    ProtocolContext: Clone + Serialize,
> {
    pub(super) party_id: PartyID,
    pub(super) threshold: PartyID,
    pub(super) number_of_parties: PartyID,
    // TODO: should we get this like that?
    pub(super) protocol_context: ProtocolContext,
    pub(super) group_public_parameters: GroupElement::PublicParameters,
    pub(super) scalar_group_public_parameters: group::PublicParameters<GroupElement::Scalar>,
    pub(super) encryption_scheme_public_parameters: EncryptionKey::PublicParameters,
    pub(super) commitment_scheme_public_parameters: commitments::PublicParameters<
        SCALAR_LIMBS,
        Pedersen<1, SCALAR_LIMBS, GroupElement::Scalar, GroupElement>,
    >,
    pub(super) unbounded_encdh_witness_public_parameters: UnboundedEncDHWitness::PublicParameters,
    pub(super) range_proof_public_parameters: RangeProof::PublicParameters<RANGE_CLAIMS_PER_SCALAR>,
    pub(super) public_key_share: GroupElement,
    pub(super) public_key: GroupElement,
    pub(super) encrypted_secret_key_share: EncryptionKey::CiphertextSpaceGroupElement,
    pub(super) centralized_party_public_key_share: GroupElement,
    pub(super) shares_of_signature_nonce_shares_witnesses:
        Vec<EncryptionKey::PlaintextSpaceGroupElement>,
    pub(super) shares_of_signature_nonce_shares_encryption_randomness:
        Vec<EncryptionKey::RandomnessSpaceGroupElement>,
}

impl<
        const SCALAR_LIMBS: usize,
        const COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS: usize,
        const RANGE_CLAIMS_PER_SCALAR: usize,
        const PLAINTEXT_SPACE_SCALAR_LIMBS: usize,
        GroupElement: PrimeGroupElement<SCALAR_LIMBS>,
        EncryptionKey: AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS>,
        UnboundedEncDHWitness: group::GroupElement + Samplable,
        RangeProof: proofs::RangeProof<COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS>,
        ProtocolContext: Clone + Serialize,
    >
    Party<
        SCALAR_LIMBS,
        COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
        RANGE_CLAIMS_PER_SCALAR,
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        GroupElement,
        EncryptionKey,
        UnboundedEncDHWitness,
        RangeProof,
        ProtocolContext,
    >
where
    // TODO: I'd love to solve this huge restriction, which seems completely useless to me and is
    // required because Rust.
    encryption_of_tuple::Language<
        PLAINTEXT_SPACE_SCALAR_LIMBS,
        SCALAR_LIMBS,
        GroupElement,
        EncryptionKey,
    >: schnorr::Language<
            { encryption_of_tuple::REPETITIONS },
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
            { encryption_of_tuple::REPETITIONS },
            RANGE_CLAIMS_PER_SCALAR,
            COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
            UnboundedEncDHWitness,
        >,
{
    pub fn initialize_proof_aggregation(
        self,
        masks_and_encrypted_masked_key_share: Vec<
            encryption_of_tuple::StatementSpaceGroupElement<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                EncryptionKey,
            >,
        >,
        encrypted_nonce_shares_and_public_shares: Vec<
            encryption_of_discrete_log::StatementSpaceGroupElement<
                PLAINTEXT_SPACE_SCALAR_LIMBS,
                SCALAR_LIMBS,
                GroupElement,
                EncryptionKey,
            >,
        >,
        rng: &mut impl CryptoRngCore,
    ) -> proofs::Result<
        Vec<
            range::CommitmentRoundParty<
                { encryption_of_tuple::REPETITIONS },
                RANGE_CLAIMS_PER_SCALAR,
                COMMITMENT_SCHEME_MESSAGE_SPACE_SCALAR_LIMBS,
                UnboundedEncDHWitness,
                RangeProof,
                encryption_of_tuple::Language<
                    PLAINTEXT_SPACE_SCALAR_LIMBS,
                    SCALAR_LIMBS,
                    GroupElement,
                    EncryptionKey,
                >,
                ProtocolContext,
            >,
        >,
    > {
        // TODO: do we need to make sure the vectors are same size?
        let batch_size = encrypted_nonce_shares_and_public_shares.len();

        let encrypted_masks: Vec<_> = masks_and_encrypted_masked_key_share
            .iter()
            .map(|statement| statement.encryption_of_multiplicand().clone())
            .collect();

        // TODO: we're not sampling new encryption randomness here for the encryption of the nonce
        // share, this is intended, just making sure.

        let masked_nonce_encryption_randomness =
            EncryptionKey::RandomnessSpaceGroupElement::sample_batch(
                &self
                    .encryption_scheme_public_parameters
                    .randomness_space_public_parameters(),
                batch_size,
                rng,
            )?;

        encrypted_masks
            .into_iter()
            .zip(
                self.shares_of_signature_nonce_shares_witnesses
                    .into_iter()
                    .zip(
                        self.shares_of_signature_nonce_shares_encryption_randomness
                            .into_iter()
                            .zip(masked_nonce_encryption_randomness.clone().into_iter()),
                    ),
            )
            .map(
                |(
                    encrypted_mask,
                    (nonce, (nonces_encryption_randomness, masked_nonces_encryption_randomness)),
                )| {
                    let language_public_parameters = encryption_of_tuple::PublicParameters::<
                        PLAINTEXT_SPACE_SCALAR_LIMBS,
                        SCALAR_LIMBS,
                        GroupElement,
                        EncryptionKey,
                    >::new::<
                        PLAINTEXT_SPACE_SCALAR_LIMBS,
                        SCALAR_LIMBS,
                        GroupElement,
                        EncryptionKey,
                    >(
                        self.scalar_group_public_parameters.clone(),
                        self.encryption_scheme_public_parameters.clone(),
                        encrypted_mask.value(),
                    );

                    let language_public_parameters = EnhancedPublicParameters::<
                        { encryption_of_tuple::REPETITIONS },
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
                    );

                    EnhancedLanguage::<
                        { encryption_of_tuple::REPETITIONS },
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
                    >::generate_witness(
                        (
                            nonce,
                            nonces_encryption_randomness,
                            masked_nonces_encryption_randomness,
                        )
                            .into(),
                        &language_public_parameters,
                        rng,
                    )
                    .map(|witness| {
                        RangeProof::new_enhanced_session::<
                            { encryption_of_tuple::REPETITIONS },
                            RANGE_CLAIMS_PER_SCALAR,
                            UnboundedEncDHWitness,
                            encryption_of_tuple::Language<
                                PLAINTEXT_SPACE_SCALAR_LIMBS,
                                SCALAR_LIMBS,
                                GroupElement,
                                EncryptionKey,
                            >,
                            ProtocolContext,
                        >(
                            self.party_id,
                            self.threshold,
                            self.number_of_parties,
                            language_public_parameters,
                            self.protocol_context.clone(),
                            vec![witness],
                        )
                    })
                },
            )
            .collect()
    }
}
