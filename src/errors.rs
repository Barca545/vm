use thiserror::Error;

#[derive(Error, Debug)]
pub enum VMErrors<'a> {
  #[error("Unknown OPCODE `{0}`")]
  UnknownOpcode(u16),
  #[error("Tried to remove a value from the stack when the stack was empty.")]
  EmptyStack,
  #[error("Command '{0}' is not recognized")]
  UnknownCommand(&'a str)
}
