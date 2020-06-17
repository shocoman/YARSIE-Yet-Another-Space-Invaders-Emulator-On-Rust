use std::fs::File;
use std::io::{BufRead, BufReader};


pub fn load_mnemonics_file() -> std::io::Result<Vec<Vec<String>>> {
    let instr_mnemonics = File::open("./rom/mnemonics.csv")?;
    let mnemonics_reader = BufReader::new(instr_mnemonics);

    let mnemonics = mnemonics_reader
        .lines()
        .map(|line| line.expect("Line read problem"))
        .map(|line| {
            line.split("%")
                .map(|word| word.to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<String>>>();
    Ok(mnemonics)
}

pub fn i8080_disassembler(rom: &Vec<std::io::Result<u8>>, offset: usize) -> std::io::Result<()> {
    let mnemonics = load_mnemonics_file()?;

    let get_instr = |num: usize| *rom.get(num).unwrap_or(&Ok(0)).as_ref().unwrap_or(&0) as usize;

    let mut pc = 0;
    while pc < rom.len() {
        let instr = get_instr(pc);

        let ref current_instr = mnemonics[instr as usize];

        let instr_length: usize = current_instr[2].parse().unwrap_or(1);
        let full_instr = (0..instr_length)
            .map(|i| mnemonics[get_instr(pc + i)][0].to_string())
            .collect::<Vec<String>>()
            .join(" ");

        println!(
            "{:02X} ({:02X}) | {:15}: {}; {:15}; {:45}; {}",
            pc + offset, pc, full_instr, instr_length, current_instr[1], current_instr[4], current_instr[3]
        );

        pc += instr_length;
    }

    Ok(())
}


pub fn print_instr_description(mnemonics: &Vec<Vec<String>>, memory: &[u8; 0x10000], pc: usize) -> std::io::Result<()> {

    let ref current_instr = mnemonics[memory[pc] as usize];

    let instr_length = current_instr[2].parse().unwrap_or(1);
    let full_instr = (0..instr_length)
        .map(|i| mnemonics[memory[pc + i] as usize][0].to_string())
        .collect::<Vec<String>>()
        .join(" ");

    println!(
        "Instr: {:02X} | {:15}: {}; {:15}; {:45}; {}",
        pc, full_instr, instr_length, current_instr[1], current_instr[4], current_instr[3]
    );

    Ok(())
}
