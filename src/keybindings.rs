use crate::terminal::Key;
use std::{error::Error, fmt::Display};

pub struct Keybindings<Output> {
    keybindings: Vec<Keybinding<Output>>,
}

struct Keybinding<Output> {
    keys: Vec<Key>,
    output: Output,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeybindingResoluton<'a, Output> {
    Resolved(&'a Output),
    AwaitingNextKey,
    NoKeybinding,
}

impl<Output> Keybindings<Output> {
    pub fn resolve_keys(&self, keys: &[Key]) -> KeybindingResoluton<Output> {
        let matching_keybindings: Vec<_> = self
            .keybindings
            .iter()
            .filter(|keybinding| keybinding.keys.len() >= keys.len())
            .filter(|keybinding| keybinding.keys.iter().zip(keys.iter()).all(|(a, b)| a == b))
            .collect();
        match matching_keybindings
            .iter()
            .find(|keybinding| keybinding.keys.len() == keys.len())
        {
            Some(keybinding) => KeybindingResoluton::Resolved(&keybinding.output),
            None => {
                if matching_keybindings.is_empty() {
                    KeybindingResoluton::NoKeybinding
                } else {
                    KeybindingResoluton::AwaitingNextKey
                }
            }
        }
    }
}

pub struct KeybindingsBuilder<Output> {
    keybindings: Vec<Keybinding<Output>>,
}

#[derive(Debug)]
pub struct KeybindingsBuilderError(String);

impl Display for KeybindingsBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for KeybindingsBuilderError {}

impl<Output> KeybindingsBuilder<Output> {
    pub fn new() -> Self {
        Self {
            keybindings: Default::default(),
        }
    }

    pub fn add_keybinding(
        mut self,
        keys: &[Key],
        output: Output,
    ) -> Result<Self, KeybindingsBuilderError> {
        let keybinding = Keybinding {
            keys: keys.iter().cloned().collect(),
            output,
        };
        if self.keybindings.iter().any(|k| keybinding.keys == k.keys) {
            return Err(KeybindingsBuilderError(format!(
                "Duplicate keybindings for {}",
                keybinding
                    .keys
                    .iter()
                    .map(|key| key.character.into())
                    .collect::<Vec<String>>()
                    .join("")
            )));
        }
        self.keybindings.push(keybinding);
        Ok(self)
    }

    pub fn build(self) -> Keybindings<Output> {
        Keybindings {
            keybindings: self.keybindings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum TestKeybindingOutput {
        A,
        B,
    }

    #[test]
    fn empty_keybindings_always_return_no_keybinding() {
        for keys in &[
            vec!['h'.into()],
            vec!['j'.into()],
            vec!['g'.into(), 'g'.into()],
        ] {
            let keybindings = KeybindingsBuilder::<TestKeybindingOutput>::new().build();

            let output = keybindings.resolve_keys(&keys);

            assert_eq!(output, KeybindingResoluton::NoKeybinding);
        }
    }

    #[test]
    fn duplicate_keybindings_errors() {
        let builder = KeybindingsBuilder::new()
            .add_keybinding(&['j'.into()], TestKeybindingOutput::B)
            .unwrap();

        let result = builder.add_keybinding(&['j'.into()], TestKeybindingOutput::B);

        assert!(result.is_err());
    }

    #[test]
    fn duplicate_different_keybindings_errors() {
        let builder = KeybindingsBuilder::new()
            .add_keybinding(&['j'.into()], TestKeybindingOutput::B)
            .unwrap();

        let result = builder.add_keybinding(&['j'.into()], TestKeybindingOutput::A);

        assert!(result.is_err());
    }

    #[test]
    fn simple_resolution_works() {
        for (keys, output) in &[
            (vec!['j'.into()], TestKeybindingOutput::A),
            (vec!['j'.into(), 'k'.into()], TestKeybindingOutput::B),
        ] {
            let keybindings = KeybindingsBuilder::new()
                .add_keybinding(&keys, output.clone())
                .unwrap()
                .build();

            let res = keybindings.resolve_keys(&keys);

            assert_eq!(res, KeybindingResoluton::Resolved(&output));
        }
    }

    #[test]
    fn awaiting_next_key_resolution_works() {
        let keybindings = KeybindingsBuilder::new()
            .add_keybinding(&['j'.into(), 'g'.into()], TestKeybindingOutput::A)
            .unwrap()
            .add_keybinding(&['j'.into(), 'k'.into()], TestKeybindingOutput::B)
            .unwrap()
            .build();

        let res = keybindings.resolve_keys(&['j'.into()]);

        assert_eq!(res, KeybindingResoluton::AwaitingNextKey);
    }

    #[test]
    fn weird_bindings_key_resolution_works() {
        let keybindings = KeybindingsBuilder::new()
            .add_keybinding(&['j'.into()], TestKeybindingOutput::A)
            .unwrap()
            .add_keybinding(&['j'.into(), 'k'.into()], TestKeybindingOutput::B)
            .unwrap()
            .build();

        let res = keybindings.resolve_keys(&['j'.into()]);

        assert_eq!(res, KeybindingResoluton::Resolved(&TestKeybindingOutput::A));
    }

    #[test]
    fn multiple_bindings_key_resolution_works() {
        let keybindings = KeybindingsBuilder::new()
            .add_keybinding(&['j'.into(), 'g'.into()], TestKeybindingOutput::A)
            .unwrap()
            .add_keybinding(&['j'.into(), 'k'.into()], TestKeybindingOutput::B)
            .unwrap()
            .build();

        let res = keybindings.resolve_keys(&['j'.into(), 'g'.into()]);

        assert_eq!(res, KeybindingResoluton::Resolved(&TestKeybindingOutput::A));
    }

    #[test]
    fn key_resolution_with_modifiers_works() {
        let keybindings = KeybindingsBuilder::new()
            .add_keybinding(
                &[Key {
                    character: 'j',
                    shift: true,
                    control: false,
                }],
                TestKeybindingOutput::A,
            )
            .unwrap()
            .add_keybinding(
                &[Key {
                    character: 'j',
                    shift: false,
                    control: true,
                }],
                TestKeybindingOutput::B,
            )
            .unwrap()
            .build();

        let res = keybindings.resolve_keys(&[Key {
            character: 'j',
            shift: false,
            control: true,
        }]);

        assert_eq!(res, KeybindingResoluton::Resolved(&TestKeybindingOutput::B));
    }
}
