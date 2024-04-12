use crate::vm::VM;
use itertools::Itertools;
use std::collections::VecDeque;

pub fn solver(vm:&mut VM) {
  let coins = Vec::from(["red coin", "blue coin", "shiny coin", "concave coin", "corroded coin"]);
  let pairs = coins.into_iter().permutations(5).collect::<Vec<Vec<&str>>>();
  place_coins(vm, pairs);
}

///Places coins into the slots.
/// Once 5 coins are placed, issues a command to pick them up.
fn place_coins(vm:&mut VM, pairs:Vec<Vec<&str>>) {
  for coins in pairs {
    //Place the coins in the pair
    for coin in 0..5 {
      let cmd = String::from("use ") + coins[coin] + "\n";
      vm.push_input(cmd);
      if coin == 4 {
        pick_up(vm)
      }
    }
  }
}

fn pick_up(vm:&mut VM) {
  let coins = VecDeque::from(["red coin", "blue coin", "shiny coin", "concave coin", "corroded coin"]);
  for coin in 0..5 {
    let cmd = String::from("take ") + coins[coin] + "\n";
    vm.push_input(cmd);
  }
}

#[cfg(test)]
mod test {
  use super::solver;
  use crate::vm::VM;

  #[test]
  fn coins() {
    let mut vm = VM::new();
    solver(&mut vm)
  }
}
