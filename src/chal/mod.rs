// Use Challenge API to check a permutation



mod chal {

    use crate::rlp::{rlc::{RlcChip, RlcConfig}, RlpConfig, RlpChip};
    use crate::halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        dev::MockProver,
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        plonk::*,
        poly::commitment::ParamsProver,
        poly::kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy,
        },
        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
    };

    use halo2_base::{
        AssignedValue,
        gates::{
            flex_gate::{FlexGateConfig, GateStrategy},
            GateInstructions,
            range::RangeConfig,
        },
        utils::{value_to_option, ScalarField},
        Context, ContextParams, SKIP_FIRST_PASS,
    };

    #[derive(Clone, Debug)]
    pub struct PermutationChip<'v, F: ScalarField> {
        pub rlc: RlcChip<'v, F>,
        pub range: RangeConfig<F>,
    }

    impl<'v, F: ScalarField> PermutationChip<'v, F> {
        pub fn new(config: RlpConfig<F>, gamma: Value<F>) -> PermutationChip<'v, F> {
            let rlc = RlcChip::new(config.rlc, gamma);
            Self { rlc, range: config.range }
        }

        fn gate(&self) -> &FlexGateConfig<F> {
            &self.range.gate
        }

        fn phase0(&self, ctx: &mut Context<F>, inputs: &PermutationCircuitInputs) -> PermutationCircuitWitness<F> {
            //let mut arr1: Vec<AssignedValue<F>> = Vec::new();
            //let mut arr2: Vec<AssignedValue<F>> = Vec::new();


            
            let arr1 = self.gate().assign_witnesses(
                ctx,
                inputs.arr1.iter().map(|item| Value::known(F::from(*item)))
            );
            let arr2 = self.gate().assign_witnesses(
                ctx,
                inputs.arr2.iter().map(|item| Value::known(F::from(*item)))
            );
            
            PermutationCircuitWitness{arr1, arr2}
        }

        fn phase1(&self, ctx: &mut Context<F>, witness: PermutationCircuitWitness<F>) {
            return;
        }
        //fn phase1(&self, ctx: &mut Context<F>) {
        //    return;
        //}
    }

    #[derive(Clone, Debug, Default)]
    pub struct PermutationCircuit {
        //pub rlc: RlcChip<F>,
        //pub range: RangeConfig<F>,
        pub inputs: PermutationCircuitInputs,
    }

    #[derive(Clone, Debug, Default)]
    pub struct PermutationCircuitInputs {
        arr1: Vec<u64>,
        arr2: Vec<u64>,
        len: usize,
    }

    #[derive(Clone, Debug, Default)]
    pub struct PermutationCircuitWitness<'a, F: ScalarField> {
        arr1: Vec<AssignedValue<'a, F>>,
        arr2: Vec<AssignedValue<'a, F>>,
    }

    const DEGREE: u32 = 10;

    impl PermutationCircuit {



    }

    impl<F: ScalarField> Circuit<F> for PermutationCircuit {
        type Config = RlpConfig<F>;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            RlpConfig::configure(meta, 1, &[1, 1], &[1], 1, 8, 0, DEGREE as usize)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {

            let gamma = config.rlc.gamma;
            let mut chip = PermutationChip::new(config, layouter.get_challenge(gamma));

            let mut first_pass = SKIP_FIRST_PASS;
            layouter.assign_region(
                || "RLP test",
                |region| {
                    if first_pass {
                        first_pass = false;
                        return Ok(());
                    }
                    let mut aux = Context::new(
                        region,
                        ContextParams {
                            max_rows: chip.gate().max_rows,
                            num_context_ids: 2,
                            fixed_columns: chip.gate().constants.clone(),
                        },
                    );
                    let ctx = &mut aux;

                    //let inputs_assigned = self.gate().assign_witnesses(
                    //    ctx,
                    //    self.inputs.iter().map(|x| Value::known(F::from(*x as u64))),
                    //);

                    // FirstPhase
                    let witness = chip.phase0(
                        ctx,
                        &self.inputs,
                    );

                    chip.range.finalize(ctx);
                    ctx.next_phase();

                    // SecondPhase
                    println!("=== SECOND PHASE ===");
                    chip.rlc.get_challenge(ctx);
                    chip.phase1(ctx, witness);

                    assert!(ctx.current_phase() <= 1);
                    #[cfg(feature = "display")]
                    {
                        let context_names = ["Range", "RLC"];
                        ctx.print_stats(&context_names);
                    }
                    Ok(())
                },
            )
        }

    }

    #[test]
    pub fn test_challenge() {
        let k = DEGREE;

        let arr1: Vec<u64> = vec![3, 7, 4, 2];
        let arr2: Vec<u64> = vec![7, 4, 3, 2];
        let len: usize = 4;

        let inputs = PermutationCircuitInputs {
            arr1, arr2, len,
        };

        let circuit = PermutationCircuit {
            inputs,
        };

        MockProver::<Fr>::run(k, &circuit, vec![]).unwrap().assert_satisfied();
    }
}