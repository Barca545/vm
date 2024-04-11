use vm::VM;
mod errors;
mod vm;

fn main() {
  let mut vm = VM::new();
  vm.load().unwrap();

  vm.run().unwrap();
}
