#[cfg(feature = "easy-smt")]
mod smt {
    use prjunnamed_netlist::{Design, EasySmtEngine};

    #[test]
    fn test_demorgan_correct() -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::try_init();

        let mut design = Design::new();
        let a = design.add_input("a", 1);
        let b = design.add_input("b", 1);
        let an = design.add_not(&a);
        let bn = design.add_not(&b);
        let y = design.add_and(an, bn);
        design.add_output("y", &y);
        design.apply();

        design.replace_value(y, design.add_not(design.add_or(a, b)));
        design.verify(EasySmtEngine::z3()?)?;
        Ok(())
    }

    #[test]
    fn test_demorgan_wrong() -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::try_init();

        let error = std::panic::catch_unwind(|| {
            let mut design = Design::new();
            let a = design.add_input("a", 1);
            let b = design.add_input("b", 1);
            let an = design.add_not(&a);
            let bn = design.add_not(&b);
            let y = design.add_and(an, bn);
            design.add_output("y", &y);
            design.apply();

            design.replace_value(y, design.add_or(a, b));
            design.verify(EasySmtEngine::z3()?)
        })
        .unwrap_err();
        assert!(error.downcast_ref::<String>().unwrap().starts_with("verification failed!"));
        Ok(())
    }
}
