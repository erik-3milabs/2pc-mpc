// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

use std::ops::Mul;

use serde::Serialize;

use crate::{
    group, proofs,
    proofs::{schnorr, schnorr::Samplable},
};

/// Knowledge of Discrete Log Schnorr Language.
#[derive(Clone)]
pub struct Language {}

/// The Public Parameters of the Knowledge of Discrete Log Schnorr Language.
#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PublicParameters<GroupElementValue> {
    pub generator: GroupElementValue,
}

/// Knowledge of Discrete Log Schnorr Language.
///
/// SECURITY NOTICE:
/// Because correctness and zero-knowledge is guaranteed for any group in this language, we choose
/// to provide a fully generic implementation.
///
/// However knowledge-soundness proofs are group dependent, and thus we can only assure security for
/// groups for which we know how to prove it.
///
/// In the paper, we have proved it for any prime known-order group; so it is safe to use with a
/// `PrimeOrderGroupElement`.
impl<Scalar, GroupElement> schnorr::Language<Scalar, GroupElement> for Language
where
    Scalar: group::GroupElement + Samplable,
    GroupElement: group::GroupElement
        + Mul<Scalar, Output = GroupElement>
        + for<'r> Mul<&'r Scalar, Output = GroupElement>,
{
    type PublicParameters = PublicParameters<GroupElement::Value>;
    const NAME: &'static str = "Knowledge of the Discrete Log";

    fn group_homomorphism(
        witness: &Scalar,
        language_public_parameters: &Self::PublicParameters,
        _witness_space_public_parameters: &Scalar::PublicParameters,
        public_value_space_public_parameters: &GroupElement::PublicParameters,
    ) -> proofs::Result<GroupElement> {
        let generator = GroupElement::new(
            language_public_parameters.generator.clone(),
            public_value_space_public_parameters,
        )?;

        Ok(generator * witness)
    }
}

/// A Knowledge of Discrete Log Schnorr Proof.
#[allow(dead_code)]
pub type Proof<S, G, ProtocolContext> = schnorr::Proof<S, G, Language, ProtocolContext>;
