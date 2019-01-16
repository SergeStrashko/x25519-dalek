// -*- mode: rust; -*-
//
// This file is part of x25519-dalek.
// Copyright (c) 2017-2019 isis lovecruft
// Copyright (c) 2019 DebugSteven
// See LICENSE for licensing information.
//
// Authors:
// - isis agora lovecruft <isis@patternsinthevoid.net>
// - DebugSteven <debugsteven@gmail.com>

//! x25519 Diffie-Hellman key exchange
//!
//! This implements x25519 key exchange as specified by Mike Hamburg
//! and Adam Langley in [RFC7748](https://tools.ietf.org/html/rfc7748).

use clear_on_drop::clear::Clear;

use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
use curve25519_dalek::montgomery::MontgomeryPoint;
use curve25519_dalek::scalar::Scalar;

use rand_core::RngCore;
use rand_core::CryptoRng;

/// A DH ephemeral public key.
pub struct EphemeralPublic(pub (crate) MontgomeryPoint);

impl From<[u8; 32]> for EphemeralPublic {
    /// Given a byte array, construct an x25519 `EphemeralPublic` key
    fn from(bytes: [u8; 32]) -> EphemeralPublic {
        EphemeralPublic(MontgomeryPoint(bytes))
    }

}

/// A DH ephemeral secret key.
pub struct EphemeralSecret(pub (crate) Scalar);

/// Overwrite ephemeral secret key material with null bytes when it goes out of scope.
impl Drop for EphemeralSecret {
    fn drop(&mut self) {
        self.0.clear();
    }
}

impl EphemeralSecret {
    /// Utility function to make it easier to call `x25519()` with
    /// an ephemeral secret key and montegomery point as input and
    /// a shared secret as the output.
    pub fn diffie_hellman(self, their_public: &EphemeralPublic) -> SharedSecret {
        SharedSecret(self.0 * their_public.0)
    }

    /// Generate an x25519 `EphemeralSecret` key.
    pub fn new<T>(csprng: &mut T) -> Self
        where T: RngCore + CryptoRng
    {
        let mut bytes = [0u8; 32];

        csprng.fill_bytes(&mut bytes);

        EphemeralSecret(clamp_scalar(bytes))
    }

}

impl<'a> From<&'a EphemeralSecret> for EphemeralPublic {
    /// Given an x25519 `EphemeralSecret` key, compute its corresponding
    /// `EphemeralPublic` key.
    fn from(secret: &'a EphemeralSecret) -> EphemeralPublic {
        EphemeralPublic((&ED25519_BASEPOINT_TABLE * &secret.0).to_montgomery())
    }

}

/// A DH SharedSecret
pub struct SharedSecret(pub (crate) MontgomeryPoint);

/// Overwrite shared secret material with null bytes when it goes out of scope.
impl Drop for SharedSecret {
    fn drop(&mut self) {
        self.0.clear();
    }
}

impl SharedSecret {

    /// View this shared secret key as a byte array.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0.as_bytes()
    }
}

/// "Decode" a scalar from a 32-byte array.
///
/// By "decode" here, what is really meant is applying key clamping by twiddling
/// some bits.
///
/// # Returns
///
/// A `Scalar`.
fn clamp_scalar(scalar: [u8; 32]) -> Scalar {
    let mut s: [u8; 32] = scalar.clone();

    s[0]  &= 248;
    s[31] &= 127;
    s[31] |= 64;

    Scalar::from_bits(s)
}

/// The x25519 function, as specified in RFC7748.
pub fn x25519(k: [u8; 32], u: [u8; 32]) -> [u8; 32] {
    (clamp_scalar(k) * MontgomeryPoint(u)).to_bytes()
}

#[cfg(test)]
mod test {
    use super::*;

    fn do_rfc7748_ladder_test1(input_scalar: [u8; 32],
                               input_point: [u8; 32],
                               expected: [u8; 32]) {
        let result = x25519(input_scalar, input_point);

        assert_eq!(result, expected);
    }

    #[test]
    fn rfc7748_ladder_test1_vectorset1() {
        let input_scalar: [u8; 32] = [
            0xa5, 0x46, 0xe3, 0x6b, 0xf0, 0x52, 0x7c, 0x9d,
            0x3b, 0x16, 0x15, 0x4b, 0x82, 0x46, 0x5e, 0xdd,
            0x62, 0x14, 0x4c, 0x0a, 0xc1, 0xfc, 0x5a, 0x18,
            0x50, 0x6a, 0x22, 0x44, 0xba, 0x44, 0x9a, 0xc4, ];
        let input_point: [u8; 32] = [
            0xe6, 0xdb, 0x68, 0x67, 0x58, 0x30, 0x30, 0xdb,
            0x35, 0x94, 0xc1, 0xa4, 0x24, 0xb1, 0x5f, 0x7c,
            0x72, 0x66, 0x24, 0xec, 0x26, 0xb3, 0x35, 0x3b,
            0x10, 0xa9, 0x03, 0xa6, 0xd0, 0xab, 0x1c, 0x4c, ];
        let expected: [u8; 32] = [
            0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90,
            0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f,
            0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7,
            0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52, ];

        do_rfc7748_ladder_test1(input_scalar, input_point, expected);
    }

    #[test]
    fn rfc7748_ladder_test1_vectorset2() {
        let input_scalar: [u8; 32] = [
            0x4b, 0x66, 0xe9, 0xd4, 0xd1, 0xb4, 0x67, 0x3c,
            0x5a, 0xd2, 0x26, 0x91, 0x95, 0x7d, 0x6a, 0xf5,
            0xc1, 0x1b, 0x64, 0x21, 0xe0, 0xea, 0x01, 0xd4,
            0x2c, 0xa4, 0x16, 0x9e, 0x79, 0x18, 0xba, 0x0d, ];
        let input_point: [u8; 32] = [
            0xe5, 0x21, 0x0f, 0x12, 0x78, 0x68, 0x11, 0xd3,
            0xf4, 0xb7, 0x95, 0x9d, 0x05, 0x38, 0xae, 0x2c,
            0x31, 0xdb, 0xe7, 0x10, 0x6f, 0xc0, 0x3c, 0x3e,
            0xfc, 0x4c, 0xd5, 0x49, 0xc7, 0x15, 0xa4, 0x93, ];
        let expected: [u8; 32] = [
            0x95, 0xcb, 0xde, 0x94, 0x76, 0xe8, 0x90, 0x7d,
            0x7a, 0xad, 0xe4, 0x5c, 0xb4, 0xb8, 0x73, 0xf8,
            0x8b, 0x59, 0x5a, 0x68, 0x79, 0x9f, 0xa1, 0x52,
            0xe6, 0xf8, 0xf7, 0x64, 0x7a, 0xac, 0x79, 0x57, ];

        do_rfc7748_ladder_test1(input_scalar, input_point, expected);
    }

    #[test]
    #[ignore] // Run only if you want to burn a lot of CPU doing 1,000,000 DH operations
    fn rfc7748_ladder_test2() {
        use curve25519_dalek::constants::X25519_BASEPOINT;

        let mut k: [u8; 32] = X25519_BASEPOINT.0;
        let mut u: [u8; 32] = X25519_BASEPOINT.0;
        let mut result: [u8; 32];

        macro_rules! do_iterations {
            ($n:expr) => (
                for _ in 0..$n {
                    result = x25519(k, u);
                    // OBVIOUS THING THAT I'M GOING TO NOTE ANYWAY BECAUSE I'VE
                    // SEEN PEOPLE DO THIS WITH GOLANG'S STDLIB AND YOU SURE AS
                    // HELL SHOULDN'T DO HORRIBLY STUPID THINGS LIKE THIS WITH
                    // MY LIBRARY:
                    //
                    // NEVER EVER TREAT SCALARS AS POINTS AND/OR VICE VERSA.
                    //
                    //                ↓↓ DON'T DO THIS ↓↓
                    u = k.clone();
                    k = result;
                }
            )
        }

        // After one iteration:
        //     422c8e7a6227d7bca1350b3e2bb7279f7897b87bb6854b783c60e80311ae3079
        // After 1,000 iterations:
        //     684cf59ba83309552800ef566f2f4d3c1c3887c49360e3875f2eb94d99532c51
        // After 1,000,000 iterations:
        //     7c3911e0ab2586fd864497297e575e6f3bc601c0883c30df5f4dd2d24f665424

        do_iterations!(1);
        assert_eq!(k, [ 0x42, 0x2c, 0x8e, 0x7a, 0x62, 0x27, 0xd7, 0xbc,
                        0xa1, 0x35, 0x0b, 0x3e, 0x2b, 0xb7, 0x27, 0x9f,
                        0x78, 0x97, 0xb8, 0x7b, 0xb6, 0x85, 0x4b, 0x78,
                        0x3c, 0x60, 0xe8, 0x03, 0x11, 0xae, 0x30, 0x79, ]);
        do_iterations!(999);
        assert_eq!(k, [ 0x68, 0x4c, 0xf5, 0x9b, 0xa8, 0x33, 0x09, 0x55,
                        0x28, 0x00, 0xef, 0x56, 0x6f, 0x2f, 0x4d, 0x3c,
                        0x1c, 0x38, 0x87, 0xc4, 0x93, 0x60, 0xe3, 0x87,
                        0x5f, 0x2e, 0xb9, 0x4d, 0x99, 0x53, 0x2c, 0x51, ]);
        do_iterations!(999_000);
        assert_eq!(k, [ 0x7c, 0x39, 0x11, 0xe0, 0xab, 0x25, 0x86, 0xfd,
                        0x86, 0x44, 0x97, 0x29, 0x7e, 0x57, 0x5e, 0x6f,
                        0x3b, 0xc6, 0x01, 0xc0, 0x88, 0x3c, 0x30, 0xdf,
                        0x5f, 0x4d, 0xd2, 0xd2, 0x4f, 0x66, 0x54, 0x24, ]);
    }
}