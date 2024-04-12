use vm::VM;
mod errors;
mod helpers;
mod vm;

fn main() {
  let mut vm = VM::new();
  vm.load().unwrap();

  // vm.run().unwrap();
  vm.dbg_run().unwrap()
}
