use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value}, pasta::Fp, plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance}
};

struct AdderCircuit {
    x: Fp,
    y: Fp,
}

#[derive(Clone, Debug)]
struct Config {
    x: Column<Advice>,
    y: Column<Advice>,
    sum: Column<Advice>,
    sum_public: Column<Instance>,
}

impl Circuit<Fp> for AdderCircuit {
    type Config = Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            x: Fp::zero(),
            y: Fp::zero(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let sum = meta.advice_column();

        meta.enable_equality(x);
        meta.enable_equality(y);
        meta.enable_equality(sum);

        let sum_public = meta.instance_column();
        meta.enable_equality(sum_public);

        Config { x, y, sum, sum_public }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut cs: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let sum_cell = cs.assign_region(
            || "addition",
            |mut region| {
                // Assign the inputs
                let _x_cell = region.assign_advice(|| "x", config.x, 0, || Value::known(self.x))?;
                let _y_cell = region.assign_advice(|| "y", config.y, 0, || Value::known(self.y))?;

                // Calculate the sum
                let sum = self.x + self.y;

                // Assign the sum
                let sum_cell = region.assign_advice(|| "sum", config.sum, 0, || Value::known(sum))?;

                Ok(sum_cell)
            },
        )?;

        // Constrain the sum to be equal to the public input using the instance column
        cs.constrain_instance(sum_cell.cell(), config.sum_public, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    #[test]
    fn test_adder() {
        let circuit = AdderCircuit { x: Fp::from(1), y: Fp::from(2) };
        let public_inputs = vec![vec![Fp::from(3)]]; // Expected sum as public input
        let mock_prover = match MockProver::run(3, &circuit, public_inputs) {
            Ok(mock_prover) => mock_prover,
            Err(e) => panic!("Error generating mock prover: {:?}", e),
        };
        assert_eq!(mock_prover.verify(), Ok(()));
    }
}
