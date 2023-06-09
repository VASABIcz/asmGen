use std::arch::x86_64::_rdrand64_step;
use std::fs;
use std::io::Stderr;
use std::ops::Deref;
use std::ptr::write;
use crate::Register::{R11, R13, Rax, Rbx, Rdi, Rdx, Rsi};

enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rex,
    Rsi,
    Rdi,

    Rsp,
    Rbp,

    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15
}

impl Into<Location> for Register {
    fn into(self) -> Location {
        return Location::Register(self)
    }
}

impl Into<Value> for Register {
    fn into(self) -> Value {
        return Value::Concrete(Concrete::Register(self))
    }
}

impl Register {
    pub fn toStr(&self) -> &'static str {
        match self {
            Register::Rax => "rax",
            Register::Rbx => "rbx",
            Register::Rcx => "rcx",
            Register::Rdx => "rdx",
            Register::Rex => "rex",
            Register::Rsi => "rsi",
            Register::Rdi => "rdi",
            Register::Rsp => "rsp",
            Register::Rbp => "rbp",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
        }
    }
}

enum Location {
    Register(Register),
    Indexing(Box<Concrete>, isize),
    Identifier(String)
}

impl Location {
    pub fn toString(&self) -> String {
        match self {
            Location::Register(v) => v.toStr().to_string(),
            Location::Indexing(inner, offset) => format!("[{}+{}]", inner.toString(), offset),
            Location::Identifier(v) => v.to_string()
        }
    }
}

enum Concrete {
    Number(usize),
    Register(Register),
}

impl Concrete {
    pub fn toString(&self) -> String {
        match self {
            Concrete::Number(v) => v.to_string(),
            Concrete::Register(v) => v.toStr().to_string()
        }
    }
}

enum Value {
    Indexing(Box<Concrete>, isize),
    Concrete(Concrete),
    Identifier(String)
}

impl Value {
    pub fn toString(&self) -> String {
        match self {
            Value::Indexing(inner, offset) => format!("[{}+{}]", inner.toString(), offset),
            Value::Concrete(v) => v.toString(),
            Value::Identifier(v) => v.to_string()
        }
    }
}

impl Into<Concrete> for i32 {
    fn into(self) -> Concrete {
        return Concrete::Number(self as usize)
    }
}

impl Into<Value> for i32 {
    fn into(self) -> Value {
        return Value::Concrete(Concrete::Number(self as usize))
    }
}

impl Into<Value> for usize {
    fn into(self) -> Value {
        return Value::Concrete(Concrete::Number(self as usize))
    }
}

impl Into<Value> for Concrete {
    fn into(self) -> Value {
        return Value::Concrete(self)
    }
}

impl Into<Location> for &str {
    fn into(self) -> Location {
        return Location::Identifier(String::from(self))
    }
}

impl Into<Location> for String {
    fn into(self) -> Location {
        return Location::Identifier(self)
    }
}

impl Into<Value> for String {
    fn into(self) -> Value {
        return Value::Identifier(self)
    }
}

trait AsmGen {
    fn mov(&mut self, dest: Location, src: Value);
    fn lea(&mut self, dest: Location, src: Value);
    fn push(&mut self, reg: Register);
    fn pop(&mut self, reg: Register);
    fn add(&mut self, dest: Location, src: Value);
    fn sysCall(&mut self);
    fn makeString(&mut self, str: &str) -> String;
    fn ret(&mut self);

    fn jmpLabel(&mut self) -> String;
    fn compare(&mut self, a: Value, b: Value);
    fn jmp(&mut self, dest: Location);
    fn jmpIfEqual(&mut self, dest: Location);
    fn jmpIfNotEqual(&mut self, dest: Location);
    fn jmpIfGt(&mut self, dest: Location);
    fn jmpIfLess(&mut self, dest: Location);

    fn inc(&mut self, reg: Register);
    fn dec(&mut self, reg: Register);

    fn xor(&mut self, dest: Location, src: Value);
}

struct NasmGen {
    pub executable: String,
    pub readOnly: String,
    pub stringCounter: usize,
    pub jmpCounter: usize
}

impl NasmGen {
    pub fn new() -> Self {
        Self {
            executable: String::new(),
            readOnly: String::new(),
            stringCounter: 0,
            jmpCounter: 0,
        }
    }

    pub fn createEscapedString(s: &str) -> String {
        let mut buf = String::with_capacity(s.len());
        let mut isInString = false;

        for c in s.bytes() {
            let chr = c as char;
            if chr.is_ascii_alphanumeric() || chr.is_ascii_graphic() || chr.is_ascii_hexdigit() || chr == ' ' {
                if !isInString {
                    if !buf.is_empty() {
                        buf.push(',');
                    }
                    buf.push('\"');
                    isInString = true;
                }

                buf.push(c as char);
            }
            else {
                if isInString {
                    buf.push('\"');
                    isInString = false;
                }
                if !buf.is_empty() {
                    buf.push(',');
                }
                buf += &c.to_string();
            }
        }

        buf
    }

    pub fn generate(&self) -> String {
        let mut buf = String::with_capacity(self.readOnly.len()+self.executable.len());
        buf.push_str("BITS 64\n");
        buf.push_str("DEFAULT REL\n\n");
        buf.push_str("section .text\n");
        buf.push_str(&self.executable);
        buf.push_str("\nsection .rodata\n");
        buf.push_str(&self.readOnly);

        buf
    }

    pub fn writeLine(&mut self, data: &str) {
        self.executable.push_str(data);
        self.executable.push('\n');
    }

    pub fn writeComment(&mut self, data: &str) {
        self.executable.push_str(";;");
        self.executable.push_str(data);
        self.executable.push('\n');
    }

    pub fn writeOp2(&mut self, op: &str, arg1: &str, arg2: &str) {
        self.executable.push_str(op);
        self.executable.push(' ');
        self.executable.push_str(arg1);
        self.executable.push_str(", ");
        self.executable.push_str(arg2);
        self.executable.push('\n');
    }

    pub fn writeOp1(&mut self, op: &str, arg1: &str) {
        self.executable.push_str(op);
        self.executable.push(' ');
        self.executable.push_str(arg1);
        self.executable.push('\n');
    }
}

impl AsmGen for NasmGen {
    fn mov(&mut self, dest: Location, src: Value) {
        self.writeOp2("mov", &dest.toString(), &src.toString())
    }

    fn lea(&mut self, dest: Location, src: Value) {
        self.writeOp2("lea", &dest.toString(), &src.toString())
    }

    fn push(&mut self, reg: Register) {
        self.writeOp1("push", reg.toStr());
    }

    fn pop(&mut self, reg: Register) {
        self.writeOp1("pop", reg.toStr());
    }

    fn add(&mut self, dest: Location, src: Value) {
        self.writeOp2("add", &dest.toString(), &dest.toString())
    }

    fn sysCall(&mut self) {
        self.writeLine("syscall")
    }

    fn makeString(&mut self, str: &str) -> String {
        // FIXME sanitaze str
        let id = format!("str{}", self.stringCounter);
        self.stringCounter += 1;

        self.readOnly.push_str(&id.clone());
        self.readOnly.push_str(": db ");
        self.readOnly += &NasmGen::createEscapedString(str);
        self.readOnly.push('\n');

        id
    }

    fn ret(&mut self) {
        self.writeLine("ret");
    }

    fn jmpLabel(&mut self) -> String {
        let label = format!("jmp{}", self.jmpCounter);
        self.executable += &label;
        self.executable += ":\n";
        self.jmpCounter += 1;
        label
    }

    fn compare(&mut self, a: Value, b: Value) {
        self.writeOp2("cmp", &a.toString(), &b.toString());
    }

    fn jmp(&mut self, dest: Location) {
        self.writeOp1("jmp", &dest.toString())
    }

    fn jmpIfEqual(&mut self, dest: Location) {
        self.writeOp1("je", &dest.toString())
    }

    fn jmpIfNotEqual(&mut self, dest: Location) {
        self.writeOp1("jne", &dest.toString())
    }

    fn jmpIfGt(&mut self, dest: Location) {
        self.writeOp1("jg", &dest.toString())
    }

    fn jmpIfLess(&mut self, dest: Location) {
        self.writeOp1("jl", &dest.toString())
    }

    fn inc(&mut self, reg: Register) {
        self.writeOp1("inc", reg.toStr());
    }

    fn dec(&mut self, reg: Register) {
        self.writeOp1("dec", reg.toStr());
    }

    fn xor(&mut self, dest: Location, src: Value) {
        self.writeOp2("xor", dest.toString().deref(), dest.toString().deref())
    }
}

fn main() {
    let mut n = NasmGen::new();
    let msg = "Hello World!\nUwU\n";
    let str = n.makeString(msg);

    n.xor(R13.into(), R13.into());
    n.mov(R13.into(), 0.into());
    let loopLabel = n.jmpLabel();
    n.mov(Rax.into(), 1.into());
    n.mov(Rdi.into(), 1.into());
    n.lea(Rsi.into(), str.into());
    n.mov(Rdx.into(), msg.len().into());
    n.sysCall();
    n.inc(R13);
    n.compare(R13.into(), 10.into());
    n.jmpIfNotEqual(loopLabel.into());

    n.mov(Rax.into(), 60.into());
    n.mov(Rdi.into(), 420.into());
    n.sysCall();

    let g = n.generate();
    println!("{}", &g);
    writeToFile("sus.asm", &g);
}

pub fn writeToFile(file: &str, s: &str) {
    fs::write(file, s).unwrap();
}