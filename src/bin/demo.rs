use bitcoin::consensus::Encodable;
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::opcodes::all::{OP_PUSHBYTES_36, OP_RETURN};
use bitcoin::{Address, Network, OutPoint, ScriptBuf, Txid, WScriptHash};
use clap::Parser;
use colored::Colorize;
use covenants_gadgets::test::SimulationInstruction;
use covenants_gadgets::{get_script_pub_key, get_tx, CovenantInput, CovenantProgram, DUST_AMOUNT};
use rust_bitcoin_m31_acc::dsl::verifier::covenant::{
    compute_all_information, FibonacciVerifierProgram, FibonacciVerifierState,
    FIBONACCI_ALL_INFORMATION,
};
use std::io::Write;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Funding Txid
    #[arg(short, long)]
    funding_txid: Option<String>,

    /// Txid
    #[arg(short, long)]
    initial_program_txid: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut fees = vec![
        58975, 90601, 58961, 88459, 37709, 67963, 77924, 75411, 38752, 48055,
    ];
    for _ in 0..8 {
        fees.extend_from_slice(&[11522, 45416, 62055, 58618, 50848, 56889, 62517, 25634]);
    }

    let fee_rate = 3000; // 1 for signet, ~3000 for fractal
    let network = Network::Bitcoin;

    let amount =
        (fees.iter().sum::<usize>() as u64 + 10000) / 7 * fee_rate + 330 * 74 + 400 * fee_rate;
    let amount_display = (((amount as f64) / 1000.0 / 1000.0 / 100.0) * 10000.0).ceil() / 10000.0;
    let actual_amount = (amount_display * 100.0 * 1000.0 * 1000.0) as u64;
    let rest = actual_amount - 330 - 400 * fee_rate;

    if args.funding_txid.is_none() || args.initial_program_txid.is_none() {
        let script_pub_key = get_script_pub_key::<FibonacciVerifierProgram>();

        let program_address = Address::from_script(script_pub_key.as_script(), network).unwrap();

        let init_state = FibonacciVerifierProgram::new();
        let hash = FibonacciVerifierProgram::get_hash(&init_state);

        let mut bytes = vec![OP_RETURN.to_u8(), OP_PUSHBYTES_36.to_u8()];
        bytes.extend_from_slice(&hash);
        bytes.extend_from_slice(&12u32.to_le_bytes());

        let caboose_address = Address::from_script(
            ScriptBuf::new_p2wsh(&WScriptHash::hash(&bytes)).as_script(),
            network,
        )
        .unwrap();

        let rest_display = (rest as f64) / 1000.0 / 1000.0 / 100.0;

        println!("================= INSTRUCTIONS =================");
        println!("To start with, prepare {} BTC into a UTXO transaction which would be used to fund the transaction fee for the entire demo.",
                 amount_display
        );
        println!(
            "> ./bitcoin-cli --datadir=signet sendtoaddress {} {}",
            "\"[an address in the local wallet]\""
                .on_bright_green()
                .black(),
            amount_display
        );
        println!();
        println!("According to that transaction, send BTC from that UTXO to the program and the state caboose with the initial state");
        println!("> ./bitcoin-cli --datadir=signet createrawtransaction \"[{{\\\"txid\\\":\\\"{}\\\", \\\"vout\\\": {}}}]\" \"[{{\\\"{}\\\":{:.8}}}, {{\\\"{}\\\":0.0000033}}]\"",
                 "[txid]".on_bright_green().black(),
                 "[vout]".on_bright_green().black(), program_address, rest_display,
                 caboose_address
        );
        println!();
        println!("Then, sign the transaction");
        println!(
            "> ./bitcoin-cli --datadir=signet signrawtransactionwithwallet {}",
            "[tx hex]".on_bright_green().black()
        );
        println!();
        println!("Send the signed transaction");
        println!(
            "> ./bitcoin-cli --datadir=signet sendrawtransaction {}",
            "[signed tx hex]".on_bright_green().black()
        );
        println!();
        println!("Call this tool again with the funding txid and initial program id");
        println!(
            "> cargo run -- -f {} -i {}",
            "[funding txid]".on_bright_green().black(),
            "[initial program txid]".on_bright_green().black()
        );
        println!("================================================");
    } else {
        let mut initial_program_txid = [0u8; 32];
        initial_program_txid
            .copy_from_slice(&hex::decode(args.initial_program_txid.unwrap()).unwrap());
        initial_program_txid.reverse();

        let mut funding_txid = [0u8; 32];
        funding_txid.copy_from_slice(&hex::decode(args.funding_txid.unwrap()).unwrap());
        funding_txid.reverse();

        let mut old_state = FibonacciVerifierProgram::new();
        let mut old_randomizer = 12u32;
        let mut old_balance = rest;
        let mut old_txid =
            Txid::from_raw_hash(*sha256d::Hash::from_bytes_ref(&initial_program_txid));

        let mut old_tx_outpoint1 = OutPoint {
            txid: Txid::from_raw_hash(*sha256d::Hash::from_bytes_ref(&funding_txid)),
            vout: 1, // change this number if the funding tx is not the second output
        };

        let mut txs = Vec::new();

        let get_instruction = |old_state: &FibonacciVerifierState| {
            let all_information = FIBONACCI_ALL_INFORMATION.get_or_init(compute_all_information);

            if old_state.pc < fees.len() {
                Some(SimulationInstruction::<FibonacciVerifierProgram> {
                    program_index: old_state.pc,
                    fee: (fees[old_state.pc] as f64 / 7.0 * (fee_rate as f64)).ceil() as usize,
                    program_input: all_information.get_input(old_state.pc),
                })
            } else {
                unimplemented!()
            }
        };

        for _ in 0..74 {
            let next = get_instruction(&old_state).unwrap();

            let mut new_balance = old_balance;
            new_balance -= next.fee as u64; // as for transaction fee
            new_balance -= DUST_AMOUNT;

            let info = CovenantInput {
                old_randomizer,
                old_balance,
                old_txid,
                input_outpoint1: old_tx_outpoint1,
                input_outpoint2: None,
                optional_deposit_input: None,
                new_balance,
            };

            let new_state =
                FibonacciVerifierProgram::run(next.program_index, &old_state, &next.program_input)
                    .unwrap();

            let (tx_template, randomizer) = get_tx::<FibonacciVerifierProgram>(
                &info,
                next.program_index,
                &old_state,
                &new_state,
                &next.program_input,
            );

            txs.push(tx_template.tx.clone());

            old_state = new_state;
            old_randomizer = randomizer;
            old_balance = new_balance;
            old_txid = tx_template.tx.compute_txid();

            old_tx_outpoint1 = tx_template.tx.input[0].previous_output;
        }

        for (i, tx) in txs.iter().enumerate() {
            let mut bytes = vec![];
            tx.consensus_encode(&mut bytes).unwrap();

            // this directory is to Fractal mainnet
            let mut fs = std::fs::File::create(format!("./demo-fractal/tx-{}.txt", i + 1)).unwrap();
            fs.write_all(hex::encode(bytes).as_bytes()).unwrap();
        }

        println!("================= INSTRUCTIONS =================");
        println!("All 74 transactions have been generated and stored in the current directory.");
    }
}
