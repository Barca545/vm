use debugger::Debugger;
use vm::VM;
mod debugger;
mod errors;
mod helpers;
mod vm;

fn main() {
  let mut vm = VM::new();
  vm.load().unwrap();

  // vm.run().unwrap();
  vm.dbg_run().unwrap()
  // let mut dbgr = Debugger::new(&mut vm);
  // dbgr.dbg_run().unwrap();
}
