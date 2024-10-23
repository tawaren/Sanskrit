//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use std::env;
use std::time::Instant;
use sanskrit_validator::execute_with_args;
use sp1_sdk::{CostEstimator, ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const SANSKRIT_ZERO_VALIDATOR_ELF: &[u8] = include_bytes!("../elf/validator-sp1-elf");

fn process_validation(stdin: &mut SP1Stdin, mods:Vec<Vec<u8>>, txts:Vec<Vec<u8>>,deps:Vec<Vec<u8>>,sys_mode:bool) {
    let mut count = 0;
    count+=1;
    stdin.write(&sys_mode);
    count+=4;
    stdin.write(&(mods.len() as u32));
    for m in mods {
        count+=m.len()+4;
        stdin.write_vec(m);
    }
    count+=4;
    stdin.write(&(txts.len() as u32));
    for t in txts {
        count+=t.len()+4;
        stdin.write_vec(t);
    }
    count+=4;
    stdin.write(&(deps.len() as u32));
    for d in deps {
        count+=d.len()+4;
        stdin.write_vec(d);
    }

    println!("Size of Input = {}", count);
}

enum Mode {
    Execute, Proove, Compact
}

//Returns true if it should be prooven false if it schould be executed
fn setup_input(mut stdin: &mut SP1Stdin) -> Mode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Not enough arguments");
    }

    let cmd = &args[1];
    let mut mode = None;
    if cmd.eq("execute") { mode = Some(Mode::Execute) }
    if cmd.eq("prove") { mode = Some(Mode::Proove) }
    if cmd.eq("compact") { mode = Some(Mode::Compact) }

    match mode {
        None => {
            execute_with_args(&args[1..],|m,t,d,b|process_validation(&mut stdin,m,t,d,b));
            Mode::Execute
        }
        Some(p) => {
            execute_with_args(&args[2..],|m,t,d,b|process_validation(&mut stdin,m,t,d,b));
            p
        }
    }
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    let mode = setup_input(&mut stdin);

    // Setup the prover client.
    let client = ProverClient::new();
    // Setup the program for proving (in case we execute we wasted a bit but not much).
    let (pk, vk) = client.setup(SANSKRIT_ZERO_VALIDATOR_ELF);


    let action = match mode {
        Mode::Execute => {
            // Execute the program
            let (_output, report) = client.execute(SANSKRIT_ZERO_VALIDATOR_ELF, stdin).run().unwrap();
            println!("Program executed successfully.");
            // Record the number of cycles executed.

            println!("Number of cycles: {}", report.total_instruction_count());
            println!("Number of syscalls: {}", report.total_syscall_count());
            println!("Touched Memory: {}", report.touched_memory_addresses);
            println!("Estimated Area: {}", report.estimate_area());
            println!("Estimated Gas: {}", report.estimate_gas());
            println!("Cycle Tracker: {:#?}", report.cycle_tracker);
            None
        }
        Mode::Proove => Some(client.prove(&pk, stdin)),
        Mode::Compact => Some(client.prove(&pk, stdin).compressed()),
    };

    match action {
        None => {}
        Some(action) => {
            let t0 = Instant::now();
            // Generate the proof
            let proof = action.run().expect("failed to generate proof");
            println!("Successfully generated proof in {} seconds", t0.elapsed().as_secs());
            let t1 = Instant::now();
            // Verify the proof.
            client.verify(&proof, &vk).expect("failed to verify proof");
            println!("Successfully verified proof in {} milli seconds", t1.elapsed().as_millis());
            //Allow to name it
            proof.save("proofs/proof.bin").expect("saving proof failed");
        }
    }
}
