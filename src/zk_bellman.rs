use anyhow::Result;
use bellman::{
    gadgets::boolean::{AllocatedBit, Boolean},
    groth16, Circuit, ConstraintSystem, SynthesisError,
};
use bls12_381::Bls12;
use ff::PrimeField;
use rand::{thread_rng, Rng};
use std::time::Instant;

#[derive(Clone)]
struct MyCircuit {
    payload: Vec<Boolean>,
}

impl<Scalar: PrimeField> Circuit<Scalar> for MyCircuit {
    fn synthesize<CS: ConstraintSystem<Scalar>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let payload_bits = self.payload.into_iter().enumerate().map(|(i, b)| {
            Ok(AllocatedBit::alloc(cs.namespace(|| format!("payload bit {}", i)), Some(b.get_value().unwrap()))?)
        }).collect::<Result<Vec<_>, SynthesisError>>()?;

        // Perform some arbitrary constraints on the payload bits
        for (i, bit) in payload_bits.iter().enumerate() {
            cs.enforce(
                || format!("payload bit {} is boolean", i),
                |lc| lc + bit.get_variable(),
                |lc| lc + CS::one(),
                |lc| lc + bit.get_variable(),
            );
        }

        Ok(())
    }
}

pub fn zk_bellman_benchmark(payload_size: usize) -> Result<std::time::Duration> {
    let rng = &mut thread_rng();
    let payload: Vec<u8> = (0..payload_size).map(|_| rng.gen()).collect();
    let payload_bits = payload.iter().map(|&byte| Boolean::constant(byte > 127)).collect();

    let circuit = MyCircuit { payload: payload_bits };

    let start = Instant::now();

    let params = {
        let mut rng = thread_rng();
        groth16::generate_random_parameters::<Bls12, _, _>(circuit.clone(), &mut rng)?
    };

    let _proof = {
        let mut rng = thread_rng();
        groth16::create_random_proof(circuit, &params, &mut rng)?
    };

    let elapsed = start.elapsed();

    Ok(elapsed)
}
