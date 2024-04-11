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

use crate::errors::VMErrors;
use eyre::Result;
use std::{env::current_dir, fs::read, io::stdin, thread::sleep, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

impl OpCode {
  fn from(value:u16) -> Result<OpCode> {
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
      // 20 => Ok(OpCode::In),
      21 => Ok(OpCode::Noop),
      _ => return Err(VMErrors::UnknownOpcode(value).into())
    }
  }
}

pub struct VM {
  reg:[u16; 8],
  ///Stores values in a growable storage as [`u16`]s.
  mem:Vec<u16>,
  ///Program counter. Contains the address of the next instruction.
  pc:usize,
  ///Tracks if the [`VM`] should continue to execute instructions or if it
  /// should terminate.
  running:bool,
  debug:bool
}

impl VM {
  pub fn new() -> Self {
    VM {
      reg:[0; 8],
      mem:Vec::default(),
      pc:0,
      running:false,
      debug:false
    }
  }

  pub fn load(&mut self) -> Result<()> {
    //Get the path of the binary.
    let path = current_dir()?.join("challenge.bin");

    //Load the binary as a Vec<u8> then convert it to a slice of u16s
    let bin = read(path)?;
    let bin = unsafe { bin.align_to::<u16>().1 };

    //Add the loaded binary to the stack
    self.mem.extend_from_slice(bin);

    Ok(())
  }

  //unsure this should be a method
  pub fn run(&mut self) -> Result<()> {
    self.running = true;

    while self.running {
      self.execute()?;
    }

    Ok(())
  }

  pub fn execute(&mut self) -> Result<()> {
    let op = OpCode::from(self.mem[self.pc])?;
    if op == OpCode::Wmem {
      if self.debug {
        self.Halt()
      }
      else {
        self.debug = true
      }
    }

    if self.debug {
      // sleep(Duration::from_millis(3000));
      dbg!(op);
    }

    match op {
      OpCode::Halt => self.Halt(),
      OpCode::Set => self.Set(),
      OpCode::Push => self.Push(),
      OpCode::Pop => self.Pop()?,
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
      OpCode::Ret => self.Ret()?,
      OpCode::Out => self.Out(),
      // OpCode::In => self.In(),
      OpCode::In => unreachable!(),
      OpCode::Noop => self.Noop()
    }

    Ok(())
  }

  ///Returns the requested number of arguments and increments the program
  /// counter by the requested number of arguments.
  pub fn get_args(&mut self, num:usize) -> &[u16] {
    //Increment the program counter to the first argument
    self.pc += 1;

    //Get the arguments
    let args = &self.mem[self.pc..self.pc + num];

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
    if test_arg >= Self::WORDSIZE {
      //Read the value from the register and return it as the argument
      let addr = (test_arg % Self::WORDSIZE) as usize;
      arg = self.reg[addr];
    }
    //Otherwise the argument is equal to the test_argument
    arg
  }
}

//Opcode implementations
impl VM {
  const WORDSIZE:u16 = 32768;

  #[allow(non_snake_case)]
  ///Takes 0 arguments. Stops execution, resets the program counter, and
  /// terminates the program.
  pub fn Halt(&mut self) {
    self.pc = 0;
    self.running = false;
  }

  #[allow(non_snake_case)]
  ///Takes 2 arguments. Set register indicated by the first argument equal to
  /// the second argument;
  pub fn Set(&mut self) {
    //Get the arguments
    let args = self.get_args(2);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    b = self.get_register_value(b);

    //Set register a equal to value b
    self.reg[a as usize] = b;
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Pushes the argument onto the stack.
  pub fn Push(&mut self) {
    //Get the argument
    let args = self.get_args(1);
    let mut a = args[0];
    a = self.get_register_value(a);

    //Push the argument onto the stack
    self.mem.push(a);
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Removes the last element from the stack and writes it
  /// into the register indicated by the argument.
  pub fn Pop(&mut self) -> Result<()> {
    //Get the argument
    let args = self.get_args(1);
    let a = args[0] % Self::WORDSIZE;

    //Get the last element of on the stack
    let val = self.mem.pop();

    match val {
      //Write the value removed from the stack into the register indicated by a
      Some(val) => self.reg[a as usize] = val,
      None => return Err(VMErrors::EmptyStack.into())
    }
    Ok(())
  }

  #[allow(non_snake_case)]
  ///Takes 3 arguments. Sets the register the first argument indicates
  /// equal to 1 if the second and third arguments are equal. Otherwise, sets
  /// the value of the register the first argument indicates to 0.
  pub fn Eq(&mut self) {
    let args = self.get_args(3);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];
    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Perform the comparison and set the register indicated by a to the result
    self.reg[a as usize] = (b == c) as u16;
  }

  #[allow(non_snake_case)]
  ///Takes 3 arguments. Sets the register indicated by the first argument equal
  /// to 1 if the second argument's value is greater than third argument's
  /// value. Sets the register indicated by the first argument equal to 0 if the
  /// second argument's value is not greater than third argument's value.
  pub fn Gt(&mut self) {
    let args = self.get_args(3);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];
    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Perform the comparison and set the register indicated by a to the result
    self.reg[a as usize] = (b > c) as u16;
  }

  #[allow(non_snake_case)]
  ///Takes in 1 argument. Sets the program counter to the argument.
  pub fn Jmp(&mut self) {
    //Get the argument
    let args = self.get_args(1);
    let mut a = args[0];
    a = self.get_register_value(a);

    //Set the program counter to the memory address indicated by a
    self.pc = a as usize;
  }

  #[allow(non_snake_case)]
  ///Takes in 2 arguments. Sets the program counter to the value of the second
  /// argument if the first argument is nonzero.
  pub fn Jt(&mut self) {
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
  }

  #[allow(non_snake_case)]
  ///Takes in 2 arguments. Sets the program counter to the second argument if
  /// the first argument is zero.
  pub fn Jf(&mut self) {
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
  }

  #[allow(non_snake_case)]
  ///Takes in 3 arguments. Stores the result of summing the second and third
  /// arguments' values (modulo WORDSIZE) in the register indicated by the first
  /// argument.
  pub fn Add(&mut self) {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];
    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Add c to b
    let sum = (b + c) % Self::WORDSIZE;

    //Store sum in the register indicated by a
    self.reg[a as usize] = sum;
  }

  #[allow(non_snake_case)]
  ///Takes in 3 arguments. Stores the product of the second and
  /// third arguments' values (modulo WORDSIZE) in the register indicated by the
  /// first argument.
  pub fn Mult(&mut self) {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];
    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Multiply c and b
    let prod = (b as u32 * c as u32) as u16 % Self::WORDSIZE;

    //Store the product in the register indicated by a
    self.reg[a as usize] = prod;
  }

  #[allow(non_snake_case)]
  ///Takes 3 arguments. Stores the remainder of the second argument divided by
  /// the third argument in the register indicated by the first argument.
  pub fn Mod(&mut self) {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];
    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Calculate the quotient
    //No need to divide by WORDSIZE because b can never b > WORDSIZE
    let quot = b % c;

    //Store the quotient in the register indicated by a
    self.reg[a as usize] = quot;
  }

  #[allow(non_snake_case)]
  ///Takes 3 arguments. Stores the value of the bitwise `AND` of the second and
  /// third arguments' values in the register indicated by the first
  /// arguement.
  pub fn And(&mut self) {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];
    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Calculate the value
    let val = b & c;

    //Store value in the register indicated by a
    self.reg[a as usize] = val;
  }

  #[allow(non_snake_case)]
  ///Takes 3 values. Stores the value of the bitwise `OR` of the second and
  /// third arguments' values in the register indicated by the first
  /// arguement.
  pub fn Or(&mut self) {
    //Get the arguments
    let args = self.get_args(3);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    let mut c = args[2];
    b = self.get_register_value(b);
    c = self.get_register_value(c);

    //Calculate the value
    let val = b | c;

    //Store value in the register indicated by a
    self.reg[a as usize] = val;
  }

  #[allow(non_snake_case)]
  ///Takes 2 arguments. Stores the value of the 15-bit bitwise `INVERSE` of the
  /// second argument's value in the register indicated by the first
  /// arguement.
  pub fn Not(&mut self) {
    //Get the arguments
    let args = self.get_args(2);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    b = self.get_register_value(b);

    //Calculate the value
    //Modulo by WORDSIZE to get the 15-bit inverse
    let val = !b % Self::WORDSIZE;

    //Store value in the register indicated by a
    self.reg[a as usize] = val;
  }

  #[allow(non_snake_case)]
  ///Takes 2 arguments. Reads memory from the memory address indicated by the
  /// second argument into the register indicated by the first argument.
  pub fn Rmem(&mut self) {
    //Get the arguments
    let args = self.get_args(2);
    let a = args[0] % Self::WORDSIZE;
    let mut b = args[1];
    b = self.get_register_value(b);

    //Read from the address b
    let val = self.mem[b as usize];

    //Store the value in the register indicated by a
    self.reg[a as usize] = val;
  }

  #[allow(non_snake_case)]
  ///Takes 2 arguments. Writes the value of the
  /// second argument into the memory address indicated by the first argument.
  pub fn Wmem(&mut self) {
    //Get the arguments
    let args = self.get_args(2);
    let mut a = args[0];
    let mut b = args[1];
    a = self.get_register_value(a);
    b = self.get_register_value(b);

    //Store b in address a
    self.mem[a as usize] = b;
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Writes the address of the next instruction to the stack
  /// then set the program counter to the memory address indicated by the
  /// argument.
  pub fn Call(&mut self) {
    //Get the args
    let args = self.get_args(1);
    let mut a = args[0];
    a = self.get_register_value(a);

    //Push the instruction of the next address to the stack
    let next = self.pc;
    self.mem.push(next as u16);

    //Set the program counter to the address indicated by a
    self.pc = a as usize;
  }

  #[allow(non_snake_case)]
  ///Takes 0 arguments. Remove the top value from the stack and jump to it.
  /// Panics if the stack is empty.
  pub fn Ret(&mut self) -> Result<()> {
    //Get the last element from the stack
    let val = self.mem.pop();

    //Jump to the memory address indicated by the value
    //Halt if the stack is empty
    match val {
      Some(val) => self.pc = val as usize,
      None => return Err(VMErrors::EmptyStack.into())
    }
    Ok(())
  }

  #[allow(non_snake_case)]
  ///Takes 1 argument. Prints the next character represented by the argument's
  /// ascii representation to the terminal.
  pub fn Out(&mut self) {
    //Get the arguments
    let args = self.get_args(1);
    let mut a = args[0];
    a = self.get_register_value(a);

    let character = char::from_u32(a as u32).unwrap().to_string();
    print!("{character}")
  }

  #[allow(non_snake_case)]
  pub fn In(&mut self) {
    //  read a character from the terminal and write its ascii code to <a>; it
    // can be assumed that once input starts, it will continue until a newline
    // is encountered; this means that you can safely read whole lines from the
    // keyboard instead of having to figure out how to read individual
    // characters
    let args = self.get_args(1);
    let mut a = args[0];

    let mut s = String::new();
    println!("Input:");

    stdin().read_line(&mut s).unwrap();

    println!("{s}");
  }

  #[allow(non_snake_case)]
  ///No operation. Increments the program counter by 1.
  pub fn Noop(&mut self) {
    self.pc += 1;
  }
}

#[cfg(test)]
mod test {
  use std::io::stdin;

  use super::VM;

  #[test]
  fn wmem_is_working() {
    let mut vm = VM::new();
    //Should write the value 6 into the 4th address of memory
    let raw = &[0x0010, 0x0003, 0x0006, 0x0000];
    vm.mem.extend_from_slice(raw);

    vm.execute().unwrap();
    assert_eq!(vm.mem[3], 6);

    let mut vm = VM::new();
    //Should write the value stored in the 1st register (6) into the 4th address of
    // memory
    let raw = &[0x0010, 0x0003, 0x8000, 0x0000];
    vm.reg[0] = 0x0006;
    vm.mem.extend_from_slice(raw);

    vm.execute().unwrap();

    assert_eq!(vm.mem[3], 6);
  }

  #[test]
  fn test_hex_nums() {
    dbg!(0x8000);
    dbg!(3 % 10);
    dbg!(28912 | 19626);
    dbg!(!0_u16);
    dbg!(32770 % 0x7FFF);
  }

  #[test]
  fn take_input() {
    let mut s = String::new();
    println!("Input...");

    stdin().read_line(&mut s).unwrap();

    println!("{s}");
  }
}
