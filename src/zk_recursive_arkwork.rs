use ark_crypto_primitives::snark::SNARK;
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_mnt4_753::{Fr as MNT4Fr, MNT4_753};
use ark_mnt6_753::{Fr as MNT6Fr, MNT6_753};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use rand::thread_rng;