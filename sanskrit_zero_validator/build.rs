use sp1_build::{build_program_with_args,BuildArgs};

fn main() {
    let mut build_args:BuildArgs = Default::default();
    build_args.elf_name = "validator-sp1-elf".to_string();
    build_args.output_directory = "sanskrit_zero_validator/elf".to_string();
    build_program_with_args("../sanskrit_zero_validator_guest", build_args)
}
