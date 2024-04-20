//VM needs
// == architecture ==
// - three storage regions
//   - memory with 15-bit address space storing 16-bit values
//   - eight reg
//   - an unbounded stack which holds individual 16-bit values
// - all numbers are unsigned integers 0..32767 (15-bit)
// - all math is modulo 32768; 32758 + 15 => 5

// 15-bit address space means each memory address is two bytes with the first
// byte equal to 0 each value starts on a memory adress divisible by 2

//I need some error that flags if the value taken in is an error

//Load file from a path not a preset one

use crate::{
  errors::VMErrors,
  helpers::{solver, Graph, Operation}
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
  collections::VecDeque,
  fs::{self, File},
  io::{stdin, Write}
};

pub const WORDSIZE:u16 = 32768;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OpCode {
  Halt = 0,
  Set = 1,
  Push = 2,
  Pop = 3,
  Eq = 4,
  Gt = 5,
  Jmp = 6,
  Jt = 7,
  Jf = 8,
  Add = 9,
  Mult = 10,
  Mod = 11,
  And = 12,
  Or = 13,
  Not = 14,
  Rmem = 15,
  Wmem = 16,
  Call = 17,
  Ret = 18,
  Out = 19,
  In = 20,
  Noop = 21
}
pub struct OpCall {
  code:OpCode,
  pc:usize,
  args:Vec<u16>,
  reg:[u16; 8],
  stack_size:usize,
  stack:Vec<u16>
}

impl OpCall {
  ///Create a new [`OpCall`].
  pub fn new(code:OpCode, pc:usize, args:Vec<u16>, reg:[u16; 8], stack_size:usize, stack:Vec<u16>) -> Self {
    OpCall {
      code,
      pc,
      args,
      reg,
      stack_size,
      stack
    }
  }

  ///Convert an [`OpCall`] into its [`String`] representation.
  pub fn to_string(&self) -> String {
    let code = serde_json::to_string(&self.code).unwrap();
    let args = format!("{:?}", self.args);
    let reg = format!(
      "R0:{:#?},R1:{:#?},R2:{:#?},R3:{:#?},R4:{:#?},R5:{:#?},R6:{:#?},R7:{:#?}",
      self.reg[0], self.reg[1], self.reg[2], self.reg[3], self.reg[4], self.reg[5], self.reg[6], self.reg[7]
    );
    let stack_size = format!("Len: {}", self.stack_size);

    let stack;
    if self.stack.len() == 0 {
      stack = format!("Stack Contents: {}", "Empty");
    }
    else if self.stack.len() > 15 {
      let mut items = String::new();
      for i in &self.stack[self.stack.len() - 16..] {
        items += &i.to_string();
        items += ", ";
      }
      stack = format!("Stack Contents(Abbreviated): {items}");
    }
    else {
      let mut items = String::new();
      for i in &self.stack {
        items += &i.to_string();
        items += ", ";
      }
      stack = format!("Stack Contents: {items}");
    }

    let call = format!(
      "Code: {}",
      code + "\n\t" + "Pc: " + &self.pc.to_string() + "\n\t" + "Args: " + &args + "\n\t" + "Regs: " + &reg + "\n\t" + &stack_size + "\n\t" + &stack + "\n"
    );
    String::from(call)
  }
}

impl OpCode {
  ///Create a new [`OpCode`] from a u16.
  pub fn new(value:u16) -> Result<OpCode> {
    match value {
      0 => Ok(OpCode::Halt),
      1 => Ok(OpCode::Set),
      2 => Ok(OpCode::Push),
      3 => Ok(OpCode::Pop),
      4 => Ok(OpCode::Eq),
      5 => Ok(OpCode::Gt),
      6 => Ok(OpCode::Jmp),
      7 => Ok(OpCode::Jt),
      8 => Ok(OpCode::Jf),
      9 => Ok(OpCode::Add),
      10 => Ok(OpCode::Mult),
      11 => Ok(OpCode::Mod),
      12 => Ok(OpCode::And),
      13 => Ok(OpCode::Or),
      14 => Ok(OpCode::Not),
      15 => Ok(OpCode::Rmem),
      16 => Ok(OpCode::Wmem),
      17 => Ok(OpCode::Call),
      18 => Ok(OpCode::Ret),
      19 => Ok(OpCode::Out),
      20 => Ok(OpCode::In),
      21 => Ok(OpCode::Noop),
      _ => return Err(VMErrors::UnknownOpcode(value).into())
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VM {
  pub reg:[u16; 8],
  ///Memory with 15-bit address space storing 16-bit ([`u16`]) values.
  pub mem:Vec<u16>,
  ///Stores values in a growable storage as [`u16`]s.
  pub stack:Vec<u16>,
  ///Program counter. Contains the address of the next instruction.
  pub pc:usize,
  ///Tracks if the [`VM`] should continue to execute instructions or if it
  /// should terminate.
  pub running:bool,
  ///Stores text inputs
  inputs:VecDeque<u8>,
  debug:u8
}

//Debug Bitflags
const DEBUG:u8 = 1 << 7;
const STEP:u8 = 1 << 6;
const PRINT:u8 = 1 << 5;

impl VM {
  pub fn new() -> Self {
    VM {
      reg:[0; 8],
      mem:Vec::default(),
      stack:Vec::default(),
      pc:0,
      running:true,
      inputs:VecDeque::new(),
      debug:0
    }
  }

  pub fn run(&mut self) -> Result<()> {
    while self.running {
      let op = OpCode::new(self.mem[self.pc])?;
      self.execute(op)?;
    }
    Ok(())
  }

  pub fn dbg_run(&mut self) -> Result<()> {
    self.debug();

    let mut file = fs::File::create("debug_log.txt").unwrap();

    while self.running {
      let op = OpCode::new(self.mem[self.pc])?;

      let call = self.execute(op)?;

      if self.debug & PRINT > 0 {
        self.debug_print(&mut file, call)
      }
    }

    Ok(())
  }

  pub fn execute(&mut self, op:OpCode) -> Result<OpCall> {
    match op {
      OpCode::Halt => self.Halt(),
      OpCode::Set => self.Set(),
      OpCode::Push => self.Push(),
      OpCode::Pop => self.Pop(),
      OpCode::Eq => self.Eq(),
      OpCode::Gt => self.Gt(),
      OpCode::Jmp => self.Jmp(),
      OpCode::Jt => self.Jt(),
      OpCode::Jf => self.Jf(),
      OpCode::Add => self.Add(),
      OpCode::Mult => self.Mult(),
      OpCode::Mod => self.Mod(),
      OpCode::And => self.And(),
      OpCode::Or => self.Or(),
      OpCode::Not => self.Not(),
      OpCode::Rmem => self.Rmem(),
      OpCode::Wmem => self.Wmem(),
      OpCode::Call => self.Call(),
      OpCode::Ret => self.Ret(),
      OpCode::Out => self.Out(),
      OpCode::In => self.In(),
      OpCode::Noop => self.Noop()
    }
  }

  ///Returns the requested number of arguments and increments the program
  /// counter by the requested number of arguments.
  pub fn get_args(&mut self, num:usize) -> Vec<u16> {
    //Increment the program counter to the first argument
    self.pc += 1;

    //Get the arguments
    let args = Vec::from(&self.mem[self.pc..self.pc + num]);

    //Update the PC
    self.pc += num;

    //Return the aruments
    args
  }

  ///Tests whether an argument is a register address or a value. If the
  /// argument is a register, return the value of the register.
  /// Otherwise return the argument.
  pub fn get_register_value(&self, test_arg:u16) -> u16 {
    let mut arg = test_arg;

    //If the argument >= 32768, it is a register
    if test_arg >= WORDSIZE {
      //Read the value from the register and return it as the argument
      let addr = (test_arg % WORDSIZE) as usize;
      arg = self.reg[addr];
    }
    //Otherwise the argument is equal to the test_argument
    arg
  }

  fn read_input(&mut self) {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();

    if '*' == s.chars().nth(0).unwrap() {
      self.exe_system_commands(s);
      self.read_input();
      return;
    }

    s.retain(|c| c != '\r');
    self.inputs.extend(s.as_bytes());
  }

  pub fn push_input(&mut self, s:String) {
    self.inputs.extend(s.as_bytes());
  }

  ///Convert an [`OpCode`] into an [`OpCall`].
  fn new_opcall(&self, op:OpCode, args:&Vec<u16>) -> OpCall {
    //Get the PC of the OpCode called
    let pc = self.pc - (args.len() + 1);

    //Get the last item on the stack or mark it as -1 for Empty
    let stack = self.stack.clone();

    OpCall::new(op, pc, args.clone(), self.reg, self.stack.len(), stack)
  }
}

//Opcode implementations
impl VM {
  #[allow(non_snake_case)]
  ///Takes 0 arguments. Stops execution, resets the program counter, and
  /// terminates the program.
  pub fn Halt(&mut self) -> Result<OpCall> {
    self.pc = 0;
    self.running = false;

    //Return the OpCall
    let call = self.new_opcall(OpCode::Halt, &Vec::new());
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 2 arguments. Set register indicated by the first argument equal to
  /// the second argument;
  pub fn Set(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(2);
    let a = args[0] % WORDSIZE;
    let mut b = args[1];

    b = self.get_register_value(b);

    //Set register a equal to value b
    self.reg[a as usize] = b;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Set, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Pushes the argument onto the stack.
  pub fn Push(&mut self) -> Result<OpCall> {
    //Get the argument
    let args = self.get_args(1);
    let mut a = args[0];

    a = self.get_register_value(a);

    //Push the argument onto the stack
    self.stack.push(a);

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Push, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Removes the last element from the stack and writes it
  /// into the register indicated by the argument.
  pub fn Pop(&mut self) -> Result<OpCall> {
    //Get the argument
    let args = self.get_args(1);
    let a = args[0] % WORDSIZE;

    //Get the last element of on the stack
    let val = self.stack.pop();

    match val {
      //Write the value removed from the stack into the register indicated by a
      Some(val) => self.reg[a as usize] = val,
      None => return Err(VMErrors::EmptyStack.into())
    }

    //Create and return the OpCall
    let call = { self.new_opcall(OpCode::Pop, &args) };
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 3 arguments. Sets the register the first argument indicates
  /// equal to 1 if the second and third arguments are equal. Otherwise, sets
  /// the value of the register the first argument indicates to 0.
  pub fn Eq(&mut self) -> Result<OpCall> {
    let args = self.get_args(3);
    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];

    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Perform the comparison and set the register indicated by a to the result
    self.reg[a as usize] = (b == c) as u16;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Eq, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 3 arguments. Sets the register indicated by the first argument equal
  /// to 1 if the second argument's value is greater than third argument's
  /// value. Sets the register indicated by the first argument equal to 0 if the
  /// second argument's value is not greater than third argument's value.
  pub fn Gt(&mut self) -> Result<OpCall> {
    let args = self.get_args(3);
    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];

    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Perform the comparison and set the register indicated by a to the result
    self.reg[a as usize] = (b > c) as u16;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Gt, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes in 1 argument. Sets the program counter to the argument.
  pub fn Jmp(&mut self) -> Result<OpCall> {
    //Get the argument
    let args = self.get_args(1);

    let mut a = args[0];
    a = self.get_register_value(a);

    //Set the program counter to the memory address indicated by a
    self.pc = a as usize;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Jmp, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes in 2 arguments. Sets the program counter to the value of the second
  /// argument if the first argument is nonzero.
  pub fn Jt(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(2);

    let mut a = args[0];
    let mut b = args[1];
    a = self.get_register_value(a);

    //Set the PC to b's value if a is nonzero
    if a != 0 {
      //Set the program counter to the memory address indicated by b
      b = self.get_register_value(b);
      self.pc = b as usize;
    }

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Jt, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes in 2 arguments. Sets the program counter to the second argument if
  /// the first argument is zero.
  pub fn Jf(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(2);
    let mut a = args[0];
    let mut b = args[1];

    a = self.get_register_value(a);

    //Set the PC to b's value if a is zero
    if a == 0 {
      //Set the program counter to the memory address indicated by b
      b = self.get_register_value(b);

      self.pc = b as usize;
    }

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Jf, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes in 3 arguments. Stores the result of summing the second and third
  /// arguments' values (modulo WORDSIZE) in the register indicated by the first
  /// argument.
  pub fn Add(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];

    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Add c to b
    let sum = (b + c) % WORDSIZE;

    //Store sum in the register indicated by a
    self.reg[a as usize] = sum;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Add, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes in 3 arguments. Stores the product of the second and
  /// third arguments' values (modulo WORDSIZE) in the register indicated by the
  /// first argument.
  pub fn Mult(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];

    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Multiply c and b
    let prod = (b as u32 * c as u32) as u16 % WORDSIZE;

    //Store the product in the register indicated by a
    self.reg[a as usize] = prod;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Mult, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 3 arguments. Stores the remainder of the second argument divided by
  /// the third argument in the register indicated by the first argument.
  pub fn Mod(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];

    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Calculate the quotient
    //No need to divide by WORDSIZE because b can never b > WORDSIZE
    let quot = b % c;

    //Store the quotient in the register indicated by a
    self.reg[a as usize] = quot;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Mod, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 3 arguments. Stores the value of the bitwise `AND` of the second and
  /// third arguments' values in the register indicated by the first
  /// arguement.
  pub fn And(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];

    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Calculate the value
    let val = b & c;

    //Store value in the register indicated by a
    self.reg[a as usize] = val;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::And, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 3 values. Stores the value of the bitwise `OR` of the second and
  /// third arguments' values in the register indicated by the first
  /// arguement.
  pub fn Or(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];

    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Calculate the value
    let val = b | c;

    //Store value in the register indicated by a
    self.reg[a as usize] = val;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Or, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 2 arguments. Stores the value of the 15-bit bitwise `INVERSE` of the
  /// second argument's value in the register indicated by the first
  /// arguement.
  pub fn Not(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(2);

    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    b = self.get_register_value(b);

    //Calculate the value
    //Modulo by WORDSIZE to get the 15-bit inverse
    let val = !b % WORDSIZE;

    //Store value in the register indicated by a
    self.reg[a as usize] = val;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Not, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 2 arguments. Reads memory from the memory address indicated by the
  /// second argument into the register indicated by the first argument.
  pub fn Rmem(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(2);

    let a = args[0] % WORDSIZE;
    let mut b = args[1];
    b = self.get_register_value(b);

    //Read from the address b
    let val = self.mem[b as usize];

    //Store the value in the register indicated by a
    self.reg[a as usize] = val;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Rmem, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 2 arguments. Writes the value of the
  /// second argument into the memory address indicated by the first argument.
  pub fn Wmem(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(2);
    let mut a = args[0];
    let mut b = args[1];

    a = self.get_register_value(a);
    b = self.get_register_value(b);

    //Store b in address a
    self.mem[a as usize] = b;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Wmem, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Writes the address of the next instruction to the stack
  /// then set the program counter to the memory address indicated by the
  /// argument.
  pub fn Call(&mut self) -> Result<OpCall> {
    //Get the args
    let args = self.get_args(1);

    let mut a = args[0];
    a = self.get_register_value(a);

    //Push the instruction of the next address to the stack
    let next = self.pc;
    self.stack.push(next as u16);

    //Set the program counter to the address indicated by a
    self.pc = a as usize;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Call, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 0 arguments. Remove the top value from the stack and jump to it.
  /// Panics if the stack is empty.
  pub fn Ret(&mut self) -> Result<OpCall> {
    //Get the last element from the stack
    let val = self.stack.pop();

    //Jump to the memory address indicated by the value
    //Halt if the stack is empty
    match val {
      Some(val) => self.pc = val as usize,
      None => return Err(VMErrors::EmptyStack.into())
    }

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Ret, &Vec::new());
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Prints the next character represented by the argument's
  /// ascii representation to the terminal.
  pub fn Out(&mut self) -> Result<OpCall> {
    //Get the arguments
    let args = self.get_args(1).clone();

    let mut a = args[0];
    a = self.get_register_value(a);

    let character = char::from_u32(a as u32).unwrap();
    print!("{character}");

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Out, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Reads characters from the [`VM`]'s input field until a
  /// linebreak is encountered.
  pub fn In(&mut self) -> Result<OpCall> {
    let args = self.get_args(1);

    let a = args[0] % WORDSIZE;

    //Read the input from memory
    let s = self.inputs.pop_front();

    match s {
      Some(s) => self.reg[a as usize] = s as u16,
      None => {
        self.read_input();
        let s = self.inputs.pop_front().unwrap();
        self.reg[a as usize] = s as u16
      }
    }

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::In, &args);
    Ok(call)
  }

  #[allow(non_snake_case)]
  ///No operation. Increments the program counter by 1.
  pub fn Noop(&mut self) -> Result<OpCall> {
    self.pc += 1;

    //Create and return the OpCall
    let call = self.new_opcall(OpCode::Noop, &Vec::new());
    Ok(call)
  }
}

//System implementations
impl VM {
  pub fn exe_system_commands(&mut self, s:String) {
    let s = &s.as_str()[1..s.len() - 2];

    match s {
      "s" => self.save(),
      "q" => self.quit(),
      "rq" => self.rage_quit(),
      "fq" => self.force_quit(),
      "ls" => self.load_save(),
      "dbg" => self.debug(),
      "print" => self.prt(),
      "clear" => self.dbg_clear(),
      "solve" => self.solve(),
      "1115" => self.prt_mem_addr(1115),
      "path" => self.path(),
      _ => println!("{}", VMErrors::UnknownCommand(s))
    }
  }

  ///Generate a path for the room and appended it to inputs
  fn path(&mut self) {
    let mut graph = Graph::new();
    graph.add_node(0, 22, &[(Operation::Sub, 1), (Operation::Sub, 2), (Operation::Add, 2), (Operation::Add, 4)]);
    graph.add_node(1, 9, &[(Operation::Sub, 2), (Operation::Sub, 3), (Operation::Mul, 3)]);
    graph.add_node(2, 4, &[(Operation::Sub, 3), (Operation::Add, 4), (Operation::Mul, 4), (Operation::Mul, 5)]);
    graph.add_node(3, 18, &[(Operation::Sub, 5), (Operation::Mul, 5), (Operation::Mul, 7)]);
    graph.add_node(4, 4, &[(Operation::Mul, 5), (Operation::Mul, 6)]);
    graph.add_node(5, 11, &[(Operation::Mul, 6), (Operation::Mul, 7), (Operation::Sub, 6), (Operation::Sub, 7)]);
    graph.add_node(6, 8, &[(Operation::Sub, 7)]);
    graph.add_node(7, 1, &[]);
  }

  ///Toggle the debug mode. Required for implementing other debug operations.
  fn debug(&mut self) {
    self.debug ^= DEBUG;
  }

  ///Toggle print debug mode.
  /// Each [`OpCode`] called will print to the standard output.
  fn prt(&mut self) {
    self.debug ^= PRINT;
  }

  fn debug_print(&self, file:&mut File, call:OpCall) {
    //Create the new log
    let log = call.to_string();

    //Append new OpCall to the end of the debug file
    write!(file, "{log}").unwrap();
  }

  fn dbg_clear(&self) {
    let mut file = fs::File::create("debug_log.txt").unwrap();
    file.write_all(&[]).unwrap();
  }

  ///Quit the game without saving.
  fn force_quit(&mut self) {
    self.Halt().unwrap();
  }

  ///Run the coin solver.
  fn solve(&mut self) {
    solver(self);
  }

  ///Prints the [`OpCode`] of the provided memory address + the next 3 values.
  fn prt_mem_addr(&mut self, addr:u16) {
    let mut file = File::create("dbg_console.txt").unwrap();
    let op = OpCode::new(self.mem[addr as usize]).unwrap();
    let op = serde_json::to_string(&op).unwrap();
    write!(file, "{op}").unwrap();
  }

  ///Quit the current game and start a new one.
  fn rage_quit(&mut self) {
    self.Halt().unwrap();
    let new = VM::new();
    *self = new;
    self.load_new().unwrap();
    self.run().unwrap();
  }

  ///Save and quit the game.
  fn quit(&mut self) {
    self.save();
    self.Halt().unwrap();
  }

  ///Reload the current save.
  fn load_save(&mut self) {
    self.Halt().unwrap();
    let new = VM::new();
    *self = new;
    self.load().unwrap();
  }

  fn save(&self) {
    let mut file = File::create("sync_save.json").unwrap();
    let state = serde_json::to_string(&self).unwrap();
    file.write_all(state.as_bytes()).unwrap();
  }

  pub fn load(&mut self) -> Result<()> {
    //Try to load from the save
    let f = fs::read_to_string("sync_save.json");

    match f {
      Ok(s) => {
        let state = serde_json::from_str::<VM>(&s)?;
        let inputs = VecDeque::from([b'\n', b'l', b'o', b'o', b'k', b'\n']);

        self.mem = state.mem;
        self.stack = state.stack;
        self.pc = state.pc;
        self.reg = state.reg;
        self.inputs = inputs;
      }
      Err(_) => self.load_new()?
    }

    Ok(())
  }

  fn load_new(&mut self) -> Result<()> {
    //Load the binary as a Vec<u8> then convert it to a slice of u16s
    let bin = fs::read("challenge.bin")?;
    let bin = unsafe { bin.align_to::<u16>().1 };

    //Add the loaded binary to the stack
    self.mem.extend_from_slice(bin);
    Ok(())
  }
}
