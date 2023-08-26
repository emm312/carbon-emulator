use std::{
    fs,
    time::{Duration, Instant}, thread,
};

use cpu::CPU;

mod cpu;

fn load_bin_file(contents: String) -> Vec<u8> {
    let mut ret = Vec::new();
    let mut ctnts = String::new();
    for line in contents.lines() {
        if line.chars().nth(0).unwrap() != '/' {
            ctnts += &format!("{}\n", line);
        }
    }
    for elem in ctnts.split_whitespace() {
        if let Ok(v) = u8::from_str_radix(elem.trim(), 2) {
            ret.push(v);
        }
    }
    ret
}

fn main() {
    let mut cpu = CPU::new();
    cpu.load_rom(load_bin_file(fs::read_to_string("out.b").unwrap()));
    while !cpu.halted {
        cpu.exec_opcode();
        thread::sleep(Duration::from_millis(100));
        cpu.ports.flush();
    }

}
