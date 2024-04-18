use std::collections::HashMap;

use helpers::teleport_check;
use vm::VM;
mod errors;
mod helpers;
mod vm;

fn main() {
  let mut vm = VM::new();
  vm.load().unwrap();

  // let mut cache = HashMap::new();
  // for r7 in 0..10 {
  //   let result = teleport_check(2, r7, r7, &mut cache);
  //   if result == 6 {
  //     vm.reg[7] = result;
  //     println!("Success with {r7}");
  //   }
  // }

  vm.reg[7] = 3;

  // vm.run().unwrap();
  vm.dbg_run().unwrap()
  // let mut dbgr = Debugger::new(&mut vm);
  // dbgr.dbg_run().unwrap();
}
fn act(r0:u16, r1:u16, r7:u16) -> u16 {
  if r0 != 0 {
    //Total loops: R1 * R0 - 1
    if r1 != 0 {
      act(act(r0 - 1, r1, r7), r1 - 1, r7)
    }
    //Total loops: R7 * R0
    else {
      act(r0 - 1, r7, r7)
    }
  }
  else {
    r1 + 1
  }
}

// fn main() {
//   let a = ack(2, 6, 6);
//   println!("{}", a); // 125
// }
