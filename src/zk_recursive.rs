use anyhow::{anyhow, Result};
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::proof::{Proof, ProofWithPublicInputs};

type F = GoldilocksField; // Using GoldilocksField for simplicity

#[derive(Clone)]
struct State {
    proof: Option<Vec<u8>>, // Proof for ZK verification
    x: F,
    y: F,
}

impl State {
    pub fn new(proof: Option<Vec<u8>>, x: u32, y: u32) -> Self {
        Self {
            proof,
            x: F::from_canonical_u32(x),
            y: F::from_canonical_u32(y),
        }
    }

    pub fn move_by(&self, x_prime: u32, y_prime: u32) -> Result<Self> {
        let new_x = self.x + F::from_canonical_u32(x_prime);
        let new_y = self.y + F::from_canonical_u32(y_prime);

        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, 2>::new(config);

        // Create targets for old and new coordinates
        let x_target = builder.add_virtual_target();
        let y_target = builder.add_virtual_target();
        let x_prime_target = builder.add_virtual_target();
        let y_prime_target = builder.add_virtual_target();
        let new_x_target = builder.add_virtual_target();
        let new_y_target = builder.add_virtual_target();

        // Add constraints
        let computed_new_x = builder.add(x_target, x_prime_target);
        let computed_new_y = builder.add(y_target, y_prime_target);
        builder.connect(computed_new_x, new_x_target);
        builder.connect(computed_new_y, new_y_target);

        // Register all values as public inputs
        builder.register_public_input(x_target);
        builder.register_public_input(y_target);
        builder.register_public_input(x_prime_target);
        builder.register_public_input(y_prime_target);
        builder.register_public_input(new_x_target);
        builder.register_public_input(new_y_target);

        let data = builder.build::<PoseidonGoldilocksConfig>();

        // Create witness
        let mut pw = PartialWitness::new();
        pw.set_target(x_target, self.x)?;
        pw.set_target(y_target, self.y)?;
        pw.set_target(x_prime_target, F::from_canonical_u32(x_prime))?;
        pw.set_target(y_prime_target, F::from_canonical_u32(y_prime))?;
        pw.set_target(new_x_target, new_x)?;
        pw.set_target(new_y_target, new_y)?;

        let proof = data.prove(pw)?;

        Ok(Self {
            proof: Some(proof.to_bytes()),
            x: new_x,
            y: new_y,
        })
    }

    fn verify(&self) -> Result<bool> {
        match &self.proof {
            Some(proof_bytes) => {
                let config = CircuitConfig::standard_recursion_config();
                let mut builder = CircuitBuilder::<F, 2>::new(config);

                // Create the same circuit structure as in move_by
                let x_target = builder.add_virtual_target();
                let y_target = builder.add_virtual_target();
                let x_prime_target = builder.add_virtual_target();
                let y_prime_target = builder.add_virtual_target();
                let new_x_target = builder.add_virtual_target();
                let new_y_target = builder.add_virtual_target();

                // Add the same constraints
                let computed_new_x = builder.add(x_target, x_prime_target);
                let computed_new_y = builder.add(y_target, y_prime_target);
                builder.connect(computed_new_x, new_x_target);
                builder.connect(computed_new_y, new_y_target);

                // Register public inputs in the same order
                builder.register_public_input(x_target);
                builder.register_public_input(y_target);
                builder.register_public_input(x_prime_target);
                builder.register_public_input(y_prime_target);
                builder.register_public_input(new_x_target);
                builder.register_public_input(new_y_target);

                let data = builder.build::<PoseidonGoldilocksConfig>();

                // Deserialize and verify the proof
                let proof = ProofWithPublicInputs::<F, PoseidonGoldilocksConfig, 2>::from_bytes(
                    proof_bytes.clone(),
                    &data.common,
                )?;
                
                data.verify(proof)?;
                Ok(true)
            }
            None => Err(anyhow!("No proof to verify")),
        }
    }
}

#[test]
fn test_state_transitions() -> Result<()> {
    let mut states = vec![State::new(None, 0, 0)];
    
    // Create 3 state transitions
    for i in 0..30 {
        let next_state = states[i].move_by(1, 1)?;
        states.push(next_state);
        if (i > 0) {
            println!("s({}) -> s({}): ({}, {}) -> ({}, {})", i-1, i, states[i-1].x, states[i-1].y, states[i].x, states[i].y);
        }
    }

    // Verify final state
    assert!(states.last().unwrap().verify()?);


    Ok(())
}
