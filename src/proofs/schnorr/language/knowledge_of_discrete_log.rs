// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0
use std::{marker::PhantomData, ops::Mul};

#[cfg(feature = "benchmarking")]
pub(crate) use benches::benchmark;
pub use language::aliases::knowledge_of_discrete_log::*;
use serde::{Deserialize, Serialize};

use super::GroupsPublicParameters;
use crate::{
    group,
    group::{CyclicGroupElement, Samplable},
    proofs,
    proofs::{
        schnorr,
        schnorr::{aggregation, language},
    },
};

pub(crate) const REPETITIONS: usize = 1;

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
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Language<Scalar, GroupElement> {
    _scalar_choice: PhantomData<Scalar>,
    _group_element_choice: PhantomData<GroupElement>,
}

impl<Scalar, GroupElement> schnorr::Language<REPETITIONS> for Language<Scalar, GroupElement>
where
    Scalar: group::GroupElement
        + Samplable
        + Mul<GroupElement, Output = GroupElement>
        + for<'r> Mul<&'r GroupElement, Output = GroupElement>
        + Copy,
    GroupElement: group::GroupElement,
{
    type WitnessSpaceGroupElement = Scalar;
    type StatementSpaceGroupElement = GroupElement;

    type PublicParameters = PublicParameters<
        Scalar::PublicParameters,
        GroupElement::PublicParameters,
        group::Value<GroupElement>,
    >;

    const NAME: &'static str = "Knowledge of the Discrete Log";

    fn group_homomorphism(
        witness: &Self::WitnessSpaceGroupElement,
        language_public_parameters: &Self::PublicParameters,
    ) -> proofs::Result<Self::StatementSpaceGroupElement> {
        let generator = GroupElement::new(
            language_public_parameters.generator,
            &language_public_parameters
                .groups_public_parameters
                .statement_space_public_parameters,
        )?;

        Ok(*witness * generator)
    }
}

/// The Public Parameters of the Knowledge of Discrete Log Schnorr Language.
#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PublicParameters<ScalarPublicParameters, GroupElementPublicParameters, GroupElementValue>
{
    pub groups_public_parameters:
        GroupsPublicParameters<ScalarPublicParameters, GroupElementPublicParameters>,
    pub generator: GroupElementValue,
}

impl<ScalarPublicParameters, GroupElementPublicParameters, GroupElementValue>
    PublicParameters<ScalarPublicParameters, GroupElementPublicParameters, GroupElementValue>
{
    pub fn new<Scalar, GroupElement>(
        scalar_group_public_parameters: Scalar::PublicParameters,
        group_public_parameters: GroupElement::PublicParameters,
    ) -> Self
    where
        Scalar: group::GroupElement<PublicParameters = ScalarPublicParameters>
            + Samplable
            + Mul<GroupElement, Output = GroupElement>
            + for<'r> Mul<&'r GroupElement, Output = GroupElement>
            + Copy,
        GroupElement: group::GroupElement<
                Value = GroupElementValue,
                PublicParameters = GroupElementPublicParameters,
            > + CyclicGroupElement,
    {
        // TODO: maybe we don't want the generator all the time?
        let generator = GroupElement::generator_from_public_parameters(&group_public_parameters);
        Self {
            groups_public_parameters: GroupsPublicParameters {
                witness_space_public_parameters: scalar_group_public_parameters,
                statement_space_public_parameters: group_public_parameters,
            },
            generator,
        }
    }
}

impl<ScalarPublicParameters, GroupElementPublicParameters, GroupElementValue>
    AsRef<GroupsPublicParameters<ScalarPublicParameters, GroupElementPublicParameters>>
    for PublicParameters<ScalarPublicParameters, GroupElementPublicParameters, GroupElementValue>
{
    fn as_ref(
        &self,
    ) -> &GroupsPublicParameters<ScalarPublicParameters, GroupElementPublicParameters> {
        &self.groups_public_parameters
    }
}

#[cfg(any(test, feature = "benchmarking"))]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{
        group::secp256k1,
        proofs::schnorr::{aggregation, language},
    };

    pub(crate) type Lang = Language<secp256k1::Scalar, secp256k1::GroupElement>;

    pub(crate) fn language_public_parameters() -> language::PublicParameters<REPETITIONS, Lang> {
        let secp256k1_scalar_public_parameters = secp256k1::scalar::PublicParameters::default();

        let secp256k1_group_public_parameters =
            secp256k1::group_element::PublicParameters::default();

        PublicParameters::new::<secp256k1::Scalar, secp256k1::GroupElement>(
            secp256k1_scalar_public_parameters,
            secp256k1_group_public_parameters,
        )
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    fn valid_proof_verifies(#[case] batch_size: usize) {
        let language_public_parameters = language_public_parameters();

        language::tests::valid_proof_verifies::<REPETITIONS, Lang>(
            language_public_parameters,
            batch_size,
        )
    }

    #[rstest]
    #[case(1, 1)]
    #[case(1, 2)]
    #[case(2, 1)]
    #[case(2, 3)]
    #[case(5, 2)]
    fn aggregates(#[case] number_of_parties: usize, #[case] batch_size: usize) {
        let language_public_parameters = language_public_parameters();
        let witnesses = language::tests::generate_witnesses_for_aggregation::<REPETITIONS, Lang>(
            &language_public_parameters,
            number_of_parties,
            batch_size,
        );

        aggregation::tests::aggregates::<REPETITIONS, Lang>(&language_public_parameters, witnesses)
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    fn invalid_proof_fails_verification(#[case] batch_size: usize) {
        let language_public_parameters = language_public_parameters();

        // No invalid values as secp256k1 statically defines group,
        // `k256::AffinePoint` assures deserialized values are on curve,
        // and `Value` can only be instantiated through deserialization
        language::tests::invalid_proof_fails_verification::<REPETITIONS, Lang>(
            None,
            None,
            language_public_parameters,
            batch_size,
        )
    }
}

#[cfg(feature = "benchmarking")]
mod benches {
    use criterion::Criterion;

    use super::*;
    use crate::{
        group::secp256k1,
        proofs::schnorr::{
            aggregation, language,
            language::knowledge_of_discrete_log::tests::{language_public_parameters, Lang},
        },
    };

    pub(crate) fn benchmark(c: &mut Criterion) {
        let language_public_parameters = language_public_parameters();

        language::benchmark::<REPETITIONS, Lang>(language_public_parameters.clone(), c);

        aggregation::benchmark::<REPETITIONS, Lang>(language_public_parameters, c);
    }
}
