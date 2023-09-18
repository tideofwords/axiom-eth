// Use Challenge API to check a permutation



mod chal {

    #[derive(Clone, Debug, Default)]
    pub struct PermutationCircuit<F> {
        arr1: Vec<i32>,
        arr2: Vec<i32>,
        len: usize,
    }

    impl<F: ScalarField> Circuit<F> for RlpTestCircuit<F> {
        type Config = RlcConfig<F>;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }


        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            RlcConfig::configure(meta, 1, &[1, 1], &[1], 1, 8, 0, DEGREE as usize)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {

            let gamma = config.rlc.gamma;
            let mut chip = RlcChip::new(config, layouter.get_challenge(gamma));

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

                    let inputs_assigned = chip.gate().assign_witnesses(
                        ctx,
                        self.inputs.iter().map(|x| Value::known(F::from(*x as u64))),
                    );

                    if self.is_array {
                        // FirstPhase
                        let witness = chip.decompose_rlp_array_phase0(
                            ctx,
                            inputs_assigned,
                            &self.max_field_lens,
                            self.is_variable_len,
                        );

                        chip.range.finalize(ctx);
                        ctx.next_phase();

                        // SecondPhase
                        println!("=== SECOND PHASE ===");
                        chip.get_challenge(ctx);
                        chip.decompose_rlp_array_phase1(ctx, witness, self.is_variable_len);
                    } else {
                        // FirstPhase
                        let witness =
                            chip.decompose_rlp_field_phase0(ctx, inputs_assigned, self.max_len);

                        chip.range.finalize(ctx);
                        ctx.next_phase();

                        // SecondPhase
                        println!("=== SECOND PHASE ===");
                        chip.get_challenge(ctx);
                        chip.decompose_rlp_field_phase1(ctx, witness);
                    }

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
        let arr1: Vec<i32> = vec!([3, 7, 4, 2]);
        let arr2: Vec<i32> = vec!([7, 4, 3, 2]);
        let len: usize = 4;

        let circuit = PermutationCircuit<Fr> {
            arr1, arr2, len,
        };
    }
}