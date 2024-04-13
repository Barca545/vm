use crate::vm::{OpCode, VM};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  fs,
  io::{stdin, Write},
  thread::sleep,
  time::Duration
};

//Debug Bitflags
const PRINT:u8 = 1 << 7;

pub struct Debugger<'a> {
  vm:&'a mut VM,
  dbg_state:u8,
  log:Vec<u16>
}

impl<'a> Debugger<'a> {
  pub fn new(vm:&'a mut VM) -> Self {
    Debugger {
      vm,
      dbg_state:0,
      log:Vec::new()
    }
  }

  pub fn dbg_run(&mut self) -> Result<()> {
    self.vm.running = true;
    while self.vm.running {
      let op = OpCode::new(self.vm.mem[self.vm.pc])?;

      if self.dbg_state & PRINT != 0 {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        dbg!(input.clone());

        // match self.log.get_mut(&input) {
        //   Some(calls) => {
        //     //Append the opcode to the calls matching the current input
        //     calls.push(op);
        //   }
        //   None => {
        //     self.log.insert(input, vec![op]);
        //   }
        // }
      }
      if op == OpCode::In {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        dbg!(input.clone());
        self.vm.In();
      }
      // self.vm.execute(op).unwrap();
      // sleep(Duration::from_millis(1000));
    }
    let mut file = fs::File::create("debug_log.txt").unwrap();
    let output = serde_json::to_string(&self.log)?;
    file.write_all(output.as_bytes()).unwrap();
    Ok(())
  }
}
