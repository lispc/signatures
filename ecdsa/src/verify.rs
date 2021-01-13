//! ECDSA verification key (i.e. public key). Generic over elliptic curves.
//!
//! Requires an [`elliptic_curve::ProjectiveArithmetic`] impl on the curve, and a
//! [`VerifyPrimitive`] impl on its associated `AffinePoint` type.

use crate::{
    hazmat::{DigestPrimitive, VerifyPrimitive},
    Error, Signature, SignatureSize,
};
use core::{fmt::Debug, ops::Add};
use elliptic_curve::{
    consts::U1,
    ff::PrimeField,
    generic_array::ArrayLength,
    point::{AffinePoint, ProjectivePoint},
    sec1::{
        EncodedPoint, FromEncodedPoint, ToEncodedPoint, UncompressedPointSize, UntaggedPointSize,
    },
    weierstrass::{point, Curve},
    FieldBytes, FromDigest, ProjectiveArithmetic, PublicKey, Scalar,
};
use signature::{digest::Digest, DigestVerifier};

#[cfg(feature = "pkcs8")]
use crate::{
    elliptic_curve::AlgorithmParameters,
    pkcs8::{self, FromPublicKey},
};

#[cfg(feature = "pem")]
use core::str::FromStr;

/// ECDSA verify key
#[derive(Copy, Clone, Debug)]
pub struct VerifyingKey<C>
where
    C: Curve + ProjectiveArithmetic,
    AffinePoint<C>: Copy + Clone + Debug,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
{
    pub(crate) inner: PublicKey<C>,
}

impl<C> VerifyingKey<C>
where
    C: Curve + ProjectiveArithmetic,
    AffinePoint<C>: Copy + Clone + Debug + Default + FromEncodedPoint<C> + ToEncodedPoint<C>,
    ProjectivePoint<C>: From<AffinePoint<C>>,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
    UntaggedPointSize<C>: Add<U1> + ArrayLength<u8>,
    UncompressedPointSize<C>: ArrayLength<u8>,
{
    /// Initialize [`VerifyingKey`] from a SEC1-encoded public key.
    pub fn from_sec1_bytes(bytes: &[u8]) -> Result<Self, Error> {
        PublicKey::from_sec1_bytes(bytes)
            .map(|pk| Self { inner: pk })
            .map_err(|_| Error::new())
    }

    /// Initialize [`VerifyingKey`] from an [`EncodedPoint`].
    pub fn from_encoded_point(public_key: &EncodedPoint<C>) -> Result<Self, Error> {
        PublicKey::<C>::from_encoded_point(public_key)
            .map(|public_key| Self { inner: public_key })
            .ok_or_else(Error::new)
    }

    /// Serialize this [`VerifyingKey`] as a SEC1 [`EncodedPoint`], optionally
    /// applying point compression.
    pub fn to_encoded_point(&self, compress: bool) -> EncodedPoint<C> {
        self.inner.to_encoded_point(compress)
    }
}

impl<C, D> DigestVerifier<D, Signature<C>> for VerifyingKey<C>
where
    C: Curve + ProjectiveArithmetic,
    D: Digest<OutputSize = C::FieldSize>,
    AffinePoint<C>: Copy + Clone + Debug + VerifyPrimitive<C>,
    ProjectivePoint<C>: From<AffinePoint<C>>,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>> + FromDigest<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn verify_digest(&self, digest: D, signature: &Signature<C>) -> Result<(), Error> {
        self.inner
            .as_affine()
            .verify_prehashed(&Scalar::<C>::from_digest(digest), signature)
    }
}

impl<C> signature::Verifier<Signature<C>> for VerifyingKey<C>
where
    C: Curve + ProjectiveArithmetic + DigestPrimitive,
    C::Digest: Digest<OutputSize = C::FieldSize>,
    AffinePoint<C>: Copy + Clone + Debug + VerifyPrimitive<C>,
    ProjectivePoint<C>: From<AffinePoint<C>>,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>> + FromDigest<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn verify(&self, msg: &[u8], signature: &Signature<C>) -> Result<(), Error> {
        self.verify_digest(C::Digest::new().chain(msg), signature)
    }
}

impl<C> From<&VerifyingKey<C>> for EncodedPoint<C>
where
    C: Curve + ProjectiveArithmetic + point::Compression,
    AffinePoint<C>: Copy + Clone + Debug + Default + FromEncodedPoint<C> + ToEncodedPoint<C>,
    ProjectivePoint<C>: From<AffinePoint<C>>,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
    UntaggedPointSize<C>: Add<U1> + ArrayLength<u8>,
    UncompressedPointSize<C>: ArrayLength<u8>,
{
    fn from(verify_key: &VerifyingKey<C>) -> EncodedPoint<C> {
        verify_key.to_encoded_point(C::COMPRESS_POINTS)
    }
}

impl<C> From<PublicKey<C>> for VerifyingKey<C>
where
    C: Curve + ProjectiveArithmetic,
    AffinePoint<C>: Copy + Clone + Debug,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
{
    fn from(public_key: PublicKey<C>) -> VerifyingKey<C> {
        VerifyingKey { inner: public_key }
    }
}

impl<C> From<&PublicKey<C>> for VerifyingKey<C>
where
    C: Curve + ProjectiveArithmetic,
    AffinePoint<C>: Copy + Clone + Debug,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
{
    fn from(public_key: &PublicKey<C>) -> VerifyingKey<C> {
        public_key.clone().into()
    }
}

impl<C> From<VerifyingKey<C>> for PublicKey<C>
where
    C: Curve + ProjectiveArithmetic,
    AffinePoint<C>: Copy + Clone + Debug,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
{
    fn from(verify_key: VerifyingKey<C>) -> PublicKey<C> {
        verify_key.inner
    }
}

impl<C> From<&VerifyingKey<C>> for PublicKey<C>
where
    C: Curve + ProjectiveArithmetic,
    AffinePoint<C>: Copy + Clone + Debug,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
{
    fn from(verify_key: &VerifyingKey<C>) -> PublicKey<C> {
        verify_key.clone().into()
    }
}

impl<C> Eq for VerifyingKey<C>
where
    C: Curve + ProjectiveArithmetic,
    AffinePoint<C>: Copy + Clone + Debug + Default + FromEncodedPoint<C> + ToEncodedPoint<C>,
    ProjectivePoint<C>: From<AffinePoint<C>>,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
    UntaggedPointSize<C>: Add<U1> + ArrayLength<u8>,
    UncompressedPointSize<C>: ArrayLength<u8>,
{
}

impl<C> PartialEq for VerifyingKey<C>
where
    C: Curve + ProjectiveArithmetic,
    AffinePoint<C>: Copy + Clone + Debug + Default + FromEncodedPoint<C> + ToEncodedPoint<C>,
    ProjectivePoint<C>: From<AffinePoint<C>>,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
    UntaggedPointSize<C>: Add<U1> + ArrayLength<u8>,
    UncompressedPointSize<C>: ArrayLength<u8>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

#[cfg(feature = "pkcs8")]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs8")))]
impl<C> FromPublicKey for VerifyingKey<C>
where
    C: Curve + AlgorithmParameters + ProjectiveArithmetic + point::Compression,
    AffinePoint<C>: Copy + Clone + Debug + Default + FromEncodedPoint<C> + ToEncodedPoint<C>,
    ProjectivePoint<C>: From<AffinePoint<C>>,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
    UntaggedPointSize<C>: Add<U1> + ArrayLength<u8>,
    UncompressedPointSize<C>: ArrayLength<u8>,
{
    fn from_spki(spki: pkcs8::SubjectPublicKeyInfo<'_>) -> pkcs8::Result<Self> {
        PublicKey::from_spki(spki).map(|inner| Self { inner })
    }
}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl<C> FromStr for VerifyingKey<C>
where
    C: Curve + AlgorithmParameters + ProjectiveArithmetic + point::Compression,
    AffinePoint<C>: Copy + Clone + Debug + Default + FromEncodedPoint<C> + ToEncodedPoint<C>,
    ProjectivePoint<C>: From<AffinePoint<C>>,
    Scalar<C>: PrimeField<Repr = FieldBytes<C>>,
    UntaggedPointSize<C>: Add<U1> + ArrayLength<u8>,
    UncompressedPointSize<C>: ArrayLength<u8>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::from_public_key_pem(s).map_err(|_| Error::new())
    }
}
