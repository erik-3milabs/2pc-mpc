// Author: dWallet Labs, LTD.
// SPDX-License-Identifier: Apache-2.0

use crypto_bigint::{NonZero, Uint};
pub use group::paillier::{
    CiphertextSpaceGroupElement, CiphertextSpacePublicParameters, PlaintextSpaceGroupElement,
    RandomnessSpaceGroupElement, RandomnessSpacePublicParameters,
};
use serde::{Deserialize, Serialize};
use tiresias::{LargeBiPrimeSizedNumber, PaillierModulusSizedNumber};

use crate::{
    ahe,
    ahe::GroupsPublicParametersAccessors as _,
    group,
    group::{
        additive_group_of_integers_modulu_n::odd_moduli, paillier::PlaintextSpacePublicParameters,
        GroupElement,
    },
    AdditivelyHomomorphicDecryptionKey, AdditivelyHomomorphicEncryptionKey,
};

/// An Encryption Key of the Paillier Additively Homomorphic Encryption Scheme.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Eq)]
pub struct EncryptionKey(tiresias::EncryptionKey);

/// An Decryption Key of the Paillier Additively Homomorphic Encryption Scheme.
#[derive(PartialEq, Clone)]
pub struct DecryptionKey(tiresias::DecryptionKey);

pub const PLAINTEXT_SPACE_SCALAR_LIMBS: usize = LargeBiPrimeSizedNumber::LIMBS;
pub const RANDOMNESS_SPACE_SCALAR_LIMBS: usize = LargeBiPrimeSizedNumber::LIMBS;
pub const CIPHERTEXT_SPACE_SCALAR_LIMBS: usize = PaillierModulusSizedNumber::LIMBS;

/// Emulate a circuit-privacy conserving additively homomorphic encryption with
/// `PlaintextSpaceGroupElement` as the plaintext group using the Paillier encryption scheme.
impl AdditivelyHomomorphicEncryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS> for EncryptionKey {
    type PlaintextSpaceGroupElement = PlaintextSpaceGroupElement;
    type RandomnessSpaceGroupElement = RandomnessSpaceGroupElement;
    type CiphertextSpaceGroupElement = CiphertextSpaceGroupElement;
    type PublicParameters = PublicParameters;

    fn new(encryption_scheme_public_parameters: &Self::PublicParameters) -> super::Result<Self> {
        // todo: checck modulus
        // TODO: this actually now should always succeed and the check should be in evaluate()
        Ok(Self(tiresias::EncryptionKey::new(
            *encryption_scheme_public_parameters
                .plaintext_space_public_parameters()
                .modulus,
        )))
        // // In order to assure circuit-privacy, the computation in
        // // [`Self::evaluate_linear_combination_with_randomness()`] must not overflow the Paillier
        // // message space modulus.
        // //
        // // This computation is $\Enc(pk, \omega q; \eta) \bigoplus_{i=1}^\ell \left(  a_i \odot
        // // \ct_i \right)$, where $\omega$ is uniformly chosen from $[0,\ellq 2^s)$.
        // //
        // // Thus, with the bound on $q$ being `PLAINTEXT_SPACE_SCALAR_LIMBS`,
        // // on the dimension $\ell$ being U64::LIMBS (as `DIMENSION` is of type `usize`),
        // // the bound on $\omega$ is therefore `(PLAINTEXT_SPACE_SCALAR_LIMBS +
        // // StatisticalSecuritySizedNumber::LIMBS + U64::LIMBS)`.
        // //
        // // Multiplying $\omega$ by $q$ thus adds an additional `PLAINTEXT_SPACE_SCALAR_LIMBS` to
        // the // bound on $\omega$ (hence the multiplication by 2).
        // //
        // // Now, we have $\ell$ more additions,
        // // each bounded to $q^2$ (as both the coefficients and the encrypted messaged of the
        // // ciphertexts are bounded by $q$) which at most adds $log(\ell)$ bits, which we can
        // bound // again by a `U64`.
        // //
        // // All of this must be `< LargeBiPrimeSizedNumber::LIMBS`.
        // if let Some(evaluation_upper_bound) = PLAINTEXT_SPACE_SCALAR_LIMBS
        //     .checked_mul(2)
        //     .and_then(|x| x.checked_add(StatisticalSecuritySizedNumber::LIMBS))
        //     .and_then(|x| x.checked_add(U64::LIMBS))
        //     .and_then(|x| x.checked_add(U64::LIMBS))
        // {
        //     if evaluation_upper_bound < LargeBiPrimeSizedNumber::LIMBS {
        //         return Ok(Self(tiresias::EncryptionKey::new(
        //             *plaintext_group_public_parameters.modulus,
        //         )));
        //     }
        // }
        // // TODO: this is a wrong computation, we need to check it is smaller than the modulus N.
        // Err(super::Error::UnsafePublicParameters)
    }

    fn encrypt_with_randomness(
        &self,
        plaintext: &PlaintextSpaceGroupElement,
        randomness: &RandomnessSpaceGroupElement,
    ) -> CiphertextSpaceGroupElement {
        // safe to `unwrap()` here, as encryption always returns a valid element in the
        // ciphertext group

        // TODO: maybe have encrypt_with_randomness

        let ciphertext = self.0.encrypt_with_randomness_inner(
            &(&<&PlaintextSpaceGroupElement as Into<Uint<PLAINTEXT_SPACE_SCALAR_LIMBS>>>::into(
                plaintext,
            ))
                .into(),
            &randomness.into(),
        );
        CiphertextSpaceGroupElement::new(
            ciphertext.into(),
            &CiphertextSpacePublicParameters {
                params: *ciphertext.params(),
            },
        )
        .unwrap()
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PublicParameters(
    ahe::GroupsPublicParameters<
        PlaintextSpacePublicParameters,
        RandomnessSpacePublicParameters,
        CiphertextSpacePublicParameters,
    >,
);

impl PublicParameters {
    pub fn new(
        paillier_associate_bi_prime: Uint<PLAINTEXT_SPACE_SCALAR_LIMBS>,
    ) -> group::Result<Self> {
        Ok(Self(ahe::GroupsPublicParameters {
            plaintext_space_public_parameters: PlaintextSpacePublicParameters {
                modulus: NonZero::new(paillier_associate_bi_prime).unwrap(),
            },
            randomness_space_public_parameters: RandomnessSpacePublicParameters::new(
                paillier_associate_bi_prime,
            )?,
            ciphertext_space_public_parameters: CiphertextSpacePublicParameters::new(
                paillier_associate_bi_prime.square(),
            )?,
        }))
    }
}

impl
    AsRef<
        ahe::GroupsPublicParameters<
            PlaintextSpacePublicParameters,
            RandomnessSpacePublicParameters,
            CiphertextSpacePublicParameters,
        >,
    > for PublicParameters
{
    fn as_ref(
        &self,
    ) -> &ahe::GroupsPublicParameters<
        PlaintextSpacePublicParameters,
        RandomnessSpacePublicParameters,
        CiphertextSpacePublicParameters,
    > {
        &self.0
    }
}

impl From<EncryptionKey> for PublicParameters {
    fn from(value: EncryptionKey) -> Self {
        Self::new(value.0.n).unwrap()
    }
}

impl AdditivelyHomomorphicDecryptionKey<PLAINTEXT_SPACE_SCALAR_LIMBS, EncryptionKey>
    for DecryptionKey
{
    type SecretKey = PaillierModulusSizedNumber;

    fn new(
        encryption_scheme_public_parameters: &ahe::PublicParameters<
            PLAINTEXT_SPACE_SCALAR_LIMBS,
            EncryptionKey,
        >,
        secret_key: Self::SecretKey,
    ) -> super::Result<Self> {
        let encryption_key = EncryptionKey::new(encryption_scheme_public_parameters)?;

        Ok(Self(tiresias::DecryptionKey::new(
            encryption_key.0,
            secret_key,
        )))
    }

    // todo: new()
    fn decrypt(&self, ciphertext: &CiphertextSpaceGroupElement) -> PlaintextSpaceGroupElement {
        PlaintextSpaceGroupElement::new(
            self.0.decrypt_inner(ciphertext.into()),
            &odd_moduli::PublicParameters {
                modulus: NonZero::new(self.0.encryption_key.n).unwrap(),
            },
        )
        .unwrap()
    }
}

impl From<tiresias::DecryptionKey> for DecryptionKey {
    fn from(value: tiresias::DecryptionKey) -> Self {
        Self(value)
    }
}

impl Into<EncryptionKey> for DecryptionKey {
    fn into(self) -> EncryptionKey {
        EncryptionKey(self.0.encryption_key)
    }
}

#[cfg(any(test, feature = "benchmarking"))]
pub(crate) mod tests {
    use crypto_bigint::{U256, U384};

    use super::*;
    use crate::{
        ahe,
        group::{multiplicative_group_of_integers_modulu_n, secp256k1},
    };

    // TODO: modulation checks and so forth
    const MASK_LIMBS: usize = U384::LIMBS;

    pub(crate) const N: LargeBiPrimeSizedNumber = LargeBiPrimeSizedNumber::from_be_hex("97431848911c007fa3a15b718ae97da192e68a4928c0259f2d19ab58ed01f1aa930e6aeb81f0d4429ac2f037def9508b91b45875c11668cea5dc3d4941abd8fbb2d6c8750e88a69727f982e633051f60252ad96ba2e9c9204f4c766c1c97bc096bb526e4b7621ec18766738010375829657c77a23faf50e3a31cb471f72c7abecdec61bdf45b2c73c666aa3729add2d01d7d96172353380c10011e1db3c47199b72da6ae769690c883e9799563d6605e0670a911a57ab5efc69a8c5611f158f1ae6e0b1b6434bafc21238921dc0b98a294195e4e88c173c8dab6334b207636774daad6f35138b9802c1784f334a82cbff480bb78976b22bb0fb41e78fdcb8095");

    pub(crate) const SECRET_KEY: PaillierModulusSizedNumber = PaillierModulusSizedNumber::from_be_hex("19d698592b9ccb2890fb84be46cd2b18c360153b740aeccb606cf4168ee2de399f05273182bf468978508a5f4869cb867b340e144838dfaf4ca9bfd38cd55dc2837688aed2dbd76d95091640c47b2037d3d0ca854ffb4c84970b86f905cef24e876ddc8ab9e04f2a5f171b9c7146776c469f0d90908aa436b710cf4489afc73cd3ee38bb81e80a22d5d9228b843f435c48c5eb40088623a14a12b44e2721b56625da5d56d257bb27662c6975630d51e8f5b930d05fc5ba461a0e158cbda0f3266408c9bf60ff617e39ae49e707cbb40958adc512f3b4b69a5c3dc8b6d34cf45bc9597840057438598623fb65254869a165a6030ec6bec12fd59e192b3c1eefd33ef5d9336e0666aa8f36c6bd2749f86ea82290488ee31bf7498c2c77a8900bae00efcff418b62d41eb93502a245236b89c241ad6272724858122a2ebe1ae7ec4684b29048ba25b3a516c281a93043d58844cf3fa0c6f1f73db5db7ecba179652349dea8df5454e0205e910e0206736051ac4b7c707c3013e190423532e907af2e85e5bb6f6f0b9b58257ca1ec8b0318dd197f30352a96472a5307333f0e6b83f4f775fb302c1e10f21e1fcbfff17e3a4aa8bb6f553d9c6ebc2c884ae9b140dd66f21afc8610418e9f0ba2d14ecfa51ff08744a3470ebe4bb21bd6d65b58ac154630b8331ea620673ffbabb179a971a6577c407a076654a629c7733836c250000");

    const SECP256K1_ORDER_LIMBS: usize = U256::LIMBS;

    #[test]
    fn encrypt_decrypts() {
        let decryption_key = DecryptionKey::from(tiresias::DecryptionKey::new(
            tiresias::EncryptionKey::new(N),
            SECRET_KEY,
        ));

        let public_parameters = PublicParameters::new(N).unwrap();

        ahe::tests::encrypt_decrypts::<PLAINTEXT_SPACE_SCALAR_LIMBS, EncryptionKey, DecryptionKey>(
            decryption_key,
            public_parameters,
        )
    }

    #[test]
    fn evaluates() {
        let decryption_key = DecryptionKey::from(tiresias::DecryptionKey::new(
            tiresias::EncryptionKey::new(N),
            SECRET_KEY,
        ));

        let public_parameters = PublicParameters::new(N).unwrap();

        ahe::tests::evaluates::<
            MASK_LIMBS,
            SECP256K1_ORDER_LIMBS,
            PLAINTEXT_SPACE_SCALAR_LIMBS,
            secp256k1::Scalar,
            EncryptionKey,
            DecryptionKey,
        >(
            decryption_key,
            secp256k1::scalar::PublicParameters::default(),
            public_parameters,
        )
    }
}
