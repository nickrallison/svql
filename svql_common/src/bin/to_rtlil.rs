use std::path::Path;
use svql_common::{ModuleConfig, YosysModule};

fn write_yosys_to_rtlil(
    yosys_module: &YosysModule,
    config: &ModuleConfig,
    rtlil_out: Option<&Path>,
) -> Result<(), Box<dyn core::error::Error>> {
    let yosys = which::which("yosys")?;
    match rtlil_out {
        Some(path) => return yosys_module.write_rtlil_to_path(config, path, &yosys),
        None => return yosys_module.write_rtlil_to_stdout(config, &yosys),
    }
}

// EXAMPLES:
// cargo run --bin to_rtlil examples/fixtures/basic/and/verilog/and_tree.v and_tree -f -p N=4
// cargo run --bin to_rtlil examples/fixtures/basic/and/verilog/and_seq.v  and_seq  -f -p N=3

fn main() -> Result<(), Box<dyn core::error::Error>> {
    use argparse::{ArgumentParser, Store, StoreOption, StoreTrue};

    let mut input_file = String::new();
    let mut module_name = String::new();
    let mut output_file: Option<String> = None;
    let mut flatten = false;
    let mut params: Vec<String> = Vec::new();
    let mut steps: Vec<String> = Vec::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Convert Verilog/RTLIL/JSON to RTLIL using Yosys");

        ap.refer(&mut input_file)
            .add_argument("input", Store, "Input design file (.v, .il, or .json)")
            .required();

        ap.refer(&mut module_name)
            .add_argument("module", Store, "Top module name")
            .required();

        ap.refer(&mut output_file).add_option(
            &["-o", "--output"],
            StoreOption,
            "Output RTLIL file (stdout if not specified)",
        );

        ap.refer(&mut flatten).add_option(
            &["-f", "--flatten"],
            StoreTrue,
            "Flatten the design hierarchy",
        );

        ap.refer(&mut params).add_option(
            &["-p", "--param"],
            argparse::Collect,
            "Set parameter (format: NAME=VALUE)",
        );

        ap.refer(&mut steps).add_option(
            &["-s", "--step"],
            argparse::Collect,
            "Add custom Yosys step",
        );

        ap.parse_args_or_exit()
    };

    // Create the YosysModule
    let yosys_module = YosysModule::new(&input_file, &module_name)?;

    // Build the configuration
    let mut config = ModuleConfig::new().with_flatten(flatten);

    // Parse and add parameters
    for param_str in params {
        let parts: Vec<&str> = param_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid parameter format: '{param_str}'. Expected NAME=VALUE"
            )
            .into());
        }
        config = config.with_param(parts[0], parts[1]);
    }

    // Add custom steps
    for step in steps {
        config = config.with_step(&step);
    }

    // Convert output file to Path if provided
    let output_path = output_file.as_ref().map(Path::new);

    // Write the RTLIL
    write_yosys_to_rtlil(&yosys_module, &config, output_path)?;

    if let Some(output_file) = output_file {
        eprintln!("RTLIL written to: {output_file}");
    }

    return Ok(())
}
