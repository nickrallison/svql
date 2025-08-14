use std::collections::BTreeMap;

use prjunnamed_netlist::{assert_isomorphic, Target};

use prjunnamed_siliconblue::SiliconBlueTarget;

// I hate rustfmt.
macro_rules! parse {
    ($source:expr) => {{
        let target = SiliconBlueTarget::new(BTreeMap::new());
        let design = prjunnamed_netlist::parse(Some(target.clone()), $source).unwrap();
        (target, design)
    }};
}

#[test]
fn test_export_lut4_carry() {
    let (target, mut design) = parse! {r#"
        %0:4 = input "i"
        %4:1 = input "ci"
        %5:_ = target "SB_LUT4_CARRY" {
            param "LUT_INIT" = 0001001000110100
            param "IS_I3_CI" = 0
            input "I" = %0:4
            input "CI" = %4:1
            %5:1 = output "O"
            %6:1 = output "CO"
        }
        %7:0 = output "o" %5
        %8:0 = output "co" %6
    "#};
    target.export(&mut design);
    let (_, mut gold) = parse! {r#"
        %0:4 = input "i"
        %4:1 = input "ci"
        %5:_ = "SB_LUT4" {
            param "LUT_INIT" = 0001001000110100
            input "I0" = %0+0
            input "I1" = %0+1
            input "I2" = %0+2
            input "I3" = %0+3
            %5:1 = output "O"
        }
        %6:_ = "SB_CARRY" {
            input "I0" = %0+1
            input "I1" = %0+2
            input "CI" = %4
            %6:1 = output "CO"
        }
        %7:0 = output "o" %5
        %8:0 = output "co" %6
    "#};
    assert_isomorphic!(design, gold);
}
