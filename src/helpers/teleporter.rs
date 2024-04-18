use std::collections::HashMap;

type Cache = HashMap<(u16, u16), u16>;

pub fn teleport_check(r0:u16, r1:u16, r7:u16, cache:&mut Cache) -> u16 {
  // if let Some(out) = cache.get(&(r0, r1)) {
  //   return *out;
  // }
  match (r0, r1) {
    (0, r1) => {
      let out = r1 + 1;
      cache.insert((r0, r1), out);
      out
    }
    (r0, 0) => teleport_check(r0 - 1, r7, r7, cache),
    (r0, r1) => teleport_check(r0 - 1, teleport_check(r0, r1 - 1, r7, cache), r7, cache)
  }
}

#[cfg(test)]
mod test {
  use super::teleport_check;
  use std::collections::HashMap;

  #[test]
  fn find_answer() {
    let mut cache = HashMap::new();
    for r7 in 0..10 {
      let result = teleport_check(2, r7, r7, &mut cache);
      dbg!(r7);
      dbg!(result);
      if result == 6 {
        println!("Success with {r7}");
        break;
      }
    }
  }

  #[test]
  fn math_test() {
    dbg!((2 - 1) % 32768);
  }
}
