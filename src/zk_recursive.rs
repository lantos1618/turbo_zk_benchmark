use anyhow::Result;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, Witness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

#[derive(Clone, Copy)]
struct State<F: Field> {
    x: F,
    y: F,
}

impl<F: Field> State<F> {
    fn move_state(&self, x_prime: F, y_prime: F) -> Self {
        State {
            x: self.x + x_prime,
            y: self.y + y_prime,
        }
    }
}

#[test]
fn test_recursive_halo2() -> Result<()> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // Define initial state
    let initial_state = State {
        x: F::from_canonical_u32(1),
        y: F::from_canonical_u32(2),
    };

    // First state transition
    let x_prime1 = F::from_canonical_u32(3);
    let y_prime1 = F::from_canonical_u32(4);
    let state_after_first_move = initial_state.move_state(x_prime1, y_prime1);

    // Second state transition
    let x_prime2 = F::from_canonical_u32(5);
    let y_prime2 = F::from_canonical_u32(6);
    let final_state = state_after_first_move.move_state(x_prime2, y_prime2);

    // Create ZK proof for state transitions
    let x_target = builder.add_virtual_target();
    let y_target = builder.add_virtual_target();
    let x_new1 = builder.add_const(x_target, x_prime1);
    let y_new1 = builder.add_const(y_target, y_prime1);
    let x_new2 = builder.add_const(x_new1, x_prime2);
    let y_new2 = builder.add_const(y_new1, y_prime2);

    builder.register_public_input(x_target);
    builder.register_public_input(y_target);
    // first state transition
    builder.register_public_input(x_new1);
    builder.register_public_input(y_new1);
    // second state transition
    builder.register_public_input(x_new2);
    builder.register_public_input(y_new2);

    let mut pw = PartialWitness::new();
    // initial x state
    match pw.set_target(x_target, initial_state.x) {
        Ok(_) => (),
        Err(e) => println!("Error setting x_target: {:?}", e),
    }
    // initial y state
    match pw.set_target(y_target, initial_state.y) {
        Ok(_) => (),
        Err(e) => println!("Error setting y_target: {:?}", e),
    }

    let data = builder.build::<C>();
    let proof = data.prove(pw)?;

    println!(
        "State transitions: (x, y) = ({}, {}) -> ({}, {}) -> ({}, {})",
        proof.public_inputs[0],
        proof.public_inputs[1],
        proof.public_inputs[2],
        proof.public_inputs[3],
        proof.public_inputs[4],
        proof.public_inputs[5]
    );

    // Check if the final state matches the expected values
    assert_eq!(proof.public_inputs[4], final_state.x, "Final x state does not match");
    assert_eq!(proof.public_inputs[5], final_state.y, "Final y state does not match");

    data.verify(proof)
}
