use helpers::teleport_check;
use vm::{VM, WORDSIZE};
mod errors;
mod helpers;
mod vm;

fn main() {
  let mut vm = VM::new();
  vm.load().unwrap();

  let mut cache = [[None; WORDSIZE as usize]; 5];
  let check = teleport_check(4, 1, 25734, &mut cache);

  if check == 6 {
    vm.reg[7] = 25734;
    vm.dbg_run().unwrap();
  }
}
