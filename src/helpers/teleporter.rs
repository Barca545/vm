use crate::vm::WORDSIZE;

type Cache = [[Option<u16>; WORDSIZE as usize]; 5];

///Use with initial parameters R0=4, R1=1 to check whether a value of R7 is
/// correct. If R7 is correct, the output will be 6. Similar to an [Ackerman function](https://rosettacode.org/wiki/Ackermann_function#Rust).
pub fn teleport_check(r0:u16, r1:u16, r7:u16, cache:&mut Cache) -> u16 {
  if let Some(out) = cache[r0 as usize][r1 as usize] {
    return out;
  }
  stacker::maybe_grow(74 * 1024, 1024 * 1024, || match (r0, r1) {
    (0, r1) => {
      let out = (r1 + 1) % WORDSIZE;
      cache[0][r1 as usize] = Some(out);
      out
    }
    (r0, 0) => {
      let out = teleport_check(r0 - 1, r7, r7, cache);
      cache[r0 as usize][0] = Some(out);
      out
    }
    (r0, r1) => {
      let out = teleport_check(r0 - 1, teleport_check(r0, r1 - 1, r7, cache), r7, cache);
      cache[r0 as usize][r1 as usize] = Some(out);
      out
    }
  })
}

#[cfg(test)]
mod test {
  use super::teleport_check;
  use crate::vm::WORDSIZE;

  #[test]
  fn find_answer() {
    let mut cache = [[None; WORDSIZE as usize]; 5];
    for r7 in 0..=WORDSIZE {
      let result = teleport_check(4, 1, r7, &mut cache);
      if result == 6 {
        println!("Success with {r7}");
        break;
      }
      cache = [[None; WORDSIZE as usize]; 5];
    }
  }

  #[test]
  fn modulo() {
    dbg!((1 + WORDSIZE) % WORDSIZE);
  }
}
