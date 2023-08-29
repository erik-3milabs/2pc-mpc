// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Borrow;

use crypto_bigint::rand_core::CryptoRngCore;
use merlin::Transcript;
use serde::{Deserialize, Serialize};

use super::Result;
use crate::{group, group::GroupElement};

/// A Schnorr Zero-Knowledge Proof Language
/// Can be generically used to generate a batched Schnorr zero-knowledge `Proof`
/// As defined in Appendix B. Schnorr Protocols in the paper
pub trait Language<
    // The upper bound for the scalar size of the witness group
    const WITNESS_SCALAR_LIMBS: usize,
    // The upper bound for the scalar size of the associated public-value space group
    const PUBLIC_VALUE_SCALAR_LIMBS: usize,
    // An element of the witness space $(\HH_\pp, +)$
    WitnessSpaceGroupElement: GroupElement<WITNESS_SCALAR_LIMBS>,
    // An element in the associated public-value space $(\GG_\pp, \cdot)$,
    PublicValueSpaceGroupElement: GroupElement<PUBLIC_VALUE_SCALAR_LIMBS>,
>
{
    /// Public parameters for a language family $\pp \gets \Setup(1^\kappa)$
    ///
    /// Used for language-specific parameters (e.g., the public parameters of the commitment scheme
    /// used for proving knowledge of decommitment - the bases $g$, $h$ in the case of Pedersen).
    ///
    /// Group public parameters are encoded separately in
    /// `WitnessSpaceGroupElement::PublicParameters` and
    /// `PublicValueSpaceGroupElement::PublicParameters`.
    type PublicParameters: Serialize + PartialEq;

    /// A unique string representing the name of this language; will be inserted to the Fiat-Shamir
    /// transcript.
    const NAME: &'static str;

    /// A group homomorphism $\phi:\HH\to\GG$  from $(\HH_\pp, +)$, the witness space,
    /// to $(\GG_\pp,\cdot)$, the statement space.
    fn group_homomorphism(
        witness: &WitnessSpaceGroupElement,
        language_public_parameters: &Self::PublicParameters,
        witness_space_public_parameters: &WitnessSpaceGroupElement::PublicParameters,
        public_value_space_public_parameters: &PublicValueSpaceGroupElement::PublicParameters,
    ) -> group::Result<PublicValueSpaceGroupElement>;
}

/// An Enhanced Batched Schnorr Zero-Knowledge Proof.
/// Implements Appendix B. Schnorr Protocols in the paper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof<
    const WITNESS_SCALAR_LIMBS: usize,
    const PUBLIC_VALUE_SCALAR_LIMBS: usize,
    WitnessSpaceGroupElement: GroupElement<WITNESS_SCALAR_LIMBS>,
    PublicValueSpaceGroupElement: GroupElement<PUBLIC_VALUE_SCALAR_LIMBS>,
    Lang,
    // A struct used by the protocol using this proof,
    // used to provide extra necessary context that will parameterize the proof (and thus verifier
    // code) and be inserted to the Fiat-Shamir transcript
    ProtocolContext,
> {
    statement_mask: PublicValueSpaceGroupElement::Value,
    response: WitnessSpaceGroupElement::Value,

    _language_choice: PhantomData<Lang>,
    _protocol_context_choice: PhantomData<ProtocolContext>,
}

impl<
        const WITNESS_SCALAR_LIMBS: usize,
        const PUBLIC_VALUE_SCALAR_LIMBS: usize,
        WitnessSpaceGroupElement: GroupElement<WITNESS_SCALAR_LIMBS>,
        PublicValueSpaceGroupElement: GroupElement<PUBLIC_VALUE_SCALAR_LIMBS>,
        L: Language<
            WITNESS_SCALAR_LIMBS,
            PUBLIC_VALUE_SCALAR_LIMBS,
            WitnessSpaceGroupElement,
            PublicValueSpaceGroupElement,
        >,
        ProtocolContext: Serialize,
    >
    Proof<
        WITNESS_SCALAR_LIMBS,
        PUBLIC_VALUE_SCALAR_LIMBS,
        WitnessSpaceGroupElement,
        PublicValueSpaceGroupElement,
        L,
        ProtocolContext,
    >
{
    #[allow(dead_code)]
    fn new(
        statement_mask: PublicValueSpaceGroupElement,
        response: WitnessSpaceGroupElement,
    ) -> Self {
        Self {
            statement_mask: statement_mask.value(),
            response: response.value(),
            _language_choice: PhantomData,
            _protocol_context_choice: PhantomData,
        }
    }

    /// Prove an enhanced batched Schnorr zero-knowledge claim.
    /// Returns the zero-knowledge proof.
    pub fn prove(
        _protocol_context: ProtocolContext,
        _language_public_parameters: &L::PublicParameters,
        _witness_space_public_parameters: &WitnessSpaceGroupElement::PublicParameters,
        _public_value_space_public_parameters: &PublicValueSpaceGroupElement::PublicParameters,
        _witnesses_and_statements: Vec<(WitnessSpaceGroupElement, PublicValueSpaceGroupElement)>,
        _rng: &mut impl CryptoRngCore,
    ) -> Result<Self> {
        todo!()
    }

    /// Verify an enhanced batched Schnorr zero-knowledge proof
    pub fn verify(
        &self,
        _protocol_context: ProtocolContext,
        _language_public_parameters: &L::PublicParameters,
        _witness_space_public_parameters: &WitnessSpaceGroupElement::PublicParameters,
        _public_value_space_public_parameters: &PublicValueSpaceGroupElement::PublicParameters,
        _statements: Vec<PublicValueSpaceGroupElement>,
    ) -> Result<()> {
        todo!()
    }

    #[allow(dead_code)]
    fn setup_protocol(
        protocol_context: &ProtocolContext,
        public_parameters: &L::PublicParameters,
        statements: Vec<PublicValueSpaceGroupElement>,
    ) -> Result<Transcript> {
        let mut transcript = Transcript::new(L::NAME.as_bytes());

        // TODO: should we add anything on the challenge space E? Even though it's hardcoded U128?

        transcript
            .serialize_to_transcript_as_json(b"protocol context", protocol_context)
            .map_err(|_e| Error::InvalidParameters())?;

        transcript
            .serialize_to_transcript_as_json(b"public parameters", public_parameters)
            .map_err(|_e| Error::InvalidParameters())?;

        if statements
            .iter()
            .map(|statement| {
                transcript.serialize_to_transcript_as_json(b"statement value", &statement.value())
            })
            .any(|res| res.is_err())
        {
            return Err(Error::InvalidParameters());
        }

        Ok(transcript)
    }

    #[allow(dead_code)]
    fn compute_challenges(
        statement_mask_value: &PublicValueSpaceGroupElement::Value,
        batch_size: usize,
        transcript: &mut Transcript,
    ) -> Result<Vec<ComputationalSecuritySizedNumber>> {
        transcript
            .serialize_to_transcript_as_json(b"randomizer public value", statement_mask_value)
            .map_err(|_e| Error::InvalidParameters())?;

        Ok((1..=batch_size)
            .map(|_| {
                // The `.challenge` method mutates `transcript` by adding the label to it.
                // Although the same label is used for all values,
                // each value will be a digest of different values
                // (i.e. it will hold different `multiple` of the label inside the digest),
                // and will therefore be unique.
                transcript.challenge(b"challenge")

                // TODO: should we also add the challenge to the transcript?
            })
            .collect())
    }
}
