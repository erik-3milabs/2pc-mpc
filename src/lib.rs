// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: BSD-3-Clause-Clear
pub use homomorphic_encryption::{AdditivelyHomomorphicDecryptionKey, AdditivelyHomomorphicEncryptionKey};
#[cfg(feature = "benchmarking")]
use criterion::criterion_group;
use crypto_bigint::{Concat, U128, U64};
use merlin::Transcript;
use proofs::transcript_protocol::TranscriptProtocol as _;
use serde::{Deserialize, Serialize};

pub mod homomorphic_encryption;
pub mod commitment;
pub mod dkg;
pub mod group;
pub(crate) mod helpers;
pub mod presign;
pub mod proofs;
mod protocol_context;
pub mod sign;
mod traits;

/// Represents an unsigned integer sized based on the computation security parameter, denoted as
/// $\kappa$.
pub type ComputationalSecuritySizedNumber = U128;

/// Represents an unsigned integer sized based on the statistical security parameter, denoted as
/// $s$. Configured for 64-bit statistical security using U64.
pub type StatisticalSecuritySizedNumber = U64;

/// Represents an unsigned integer sized based on the commitment size that matches security
/// parameter, which is double in size, as collisions can be found in the root of the space.
pub type CommitmentSizedNumber = <ComputationalSecuritySizedNumber as Concat>::Output;

#[derive(PartialEq, Debug, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct Commitment(CommitmentSizedNumber);

impl Commitment {
    pub fn commit_transcript(
        transcript: &mut Transcript,
        commitment_randomness: &ComputationalSecuritySizedNumber,
    ) -> Self {
        transcript.append_uint(
            b"maurer proof aggregation commitment round commitment randomness",
            commitment_randomness,
        );

        Commitment(transcript.challenge(b"maurer proof aggregation commitment round commitment"))
    }
}

/// A unique identifier of a party in a MPC protocol.
pub type PartyID = u16;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid parameters")]
    InvalidParameters,

    #[error("an internal error that should never have happened and signifies a bug")]
    InternalError,

    #[error("group error")]
    Group(#[from] group::Error),

    #[error("proofs error")]
    Proofs(#[from] proofs::Error),

    #[error("error in homomorphic encryption related operations")]
    HomomorphicEncryption(#[from] homomorphic_encryption::Error),

    #[error("the other party maliciously attempted to bypass the commitment round by sending decommitment which does not match its commitment")]
    WrongDecommitment,

    #[error("the other party maliciously attempted to bypass validity checks by sending commitment whose homomorphic evaluation did not equal expected values")]
    CommitmentsHomomorphicEvaluation,
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "benchmarking")]
criterion_group!(
    benches,
    // group::benchmark_scalar_mul_bounded,
    // proofs::transcript_protocol::benchmark,
    // proofs::maurer::knowledge_of_discrete_log::benchmark,
    // proofs::maurer::knowledge_of_decommitment::benchmark_zero_knowledge,
    // proofs::maurer::knowledge_of_decommitment::benchmark_lightningproofs_single_message,
    // proofs::maurer::knowledge_of_decommitment::benchmark_lightningproofs_encdl,
    // proofs::maurer::knowledge_of_decommitment::benchmark_lightningproofs_dcom_eval,
    // proofs::maurer::committment_of_discrete_log::benchmark,
    // proofs::maurer::discrete_log_ratio_of_commited_values::benchmark,
    // proofs::maurer::encryption_of_discrete_log::benchmark,
    // proofs::maurer::encryption_of_tuple::benchmark,
    // proofs::maurer::committed_linear_evaluation::benchmark,
    sign::benchmark,
);
