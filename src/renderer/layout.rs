use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::markdown_parser::StyledWord;

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutLine<'a> {
    pub elements: Vec<LayoutElement<'a>>,
}

impl<'a> LayoutLine<'a> {
    fn new(elements: Vec<LayoutElement<'a>>) -> Self {
        Self { elements }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LayoutElement<'a> {
    Word(StyledWord<'a>),
    Whitespace(usize),
}

fn styled_word_length(word: &StyledWord) -> usize {
    word.text.len()
}

fn split_styled_word<'a>(word: &StyledWord<'a>, index: usize) -> (StyledWord<'a>, StyledWord<'a>) {
    let (t1, t2) = word.text.split_at(index);
    (
        StyledWord {
            text: t1,
            style: word.style,
        },
        StyledWord {
            text: t2,
            style: word.style,
        },
    )
}

struct WordsInLine<'a> {
    words: Vec<StyledWord<'a>>,
    remaining_space: usize,
}

impl<'a> WordsInLine<'a> {
    fn spread_evenly(&self) -> LayoutLine<'a> {
        assert!(self.words.len() > 0);

        if self.words.len() == 1 {
            return self.align_left();
        }

        let mut layout_line = LayoutLine::new(Vec::new());
        let gaps_between_words = self.words.len() - 1;
        let spaces_per_gap = self.remaining_space / gaps_between_words;
        let num_extra_spaces = self.remaining_space - spaces_per_gap;
        assert!(num_extra_spaces <= gaps_between_words);

        let mut extra_spaces = get_unique_random_numbers(num_extra_spaces, 0, gaps_between_words);

        for (i, word) in self.words.iter().enumerate() {
            let mut add_word = || layout_line.elements.push(LayoutElement::Word(*word));

            add_word();

            let mut add_whitespace = || {
                if i != self.words.len() - 1 {
                    if let Some(&idx) = extra_spaces.last() {
                        if i == idx {
                            layout_line
                                .elements
                                .push(LayoutElement::Whitespace(spaces_per_gap + 2));
                            // Above it says +2 because there are some extra spaces left,
                            // so we're adding them to gaps one-by-one.
                            // In both arms of the if statement, we add +1 because there must be at
                            // least one space in each gap.
                            extra_spaces.pop();
                            return;
                        }
                    }

                    layout_line
                        .elements
                        .push(LayoutElement::Whitespace(spaces_per_gap + 1));
                }
            };

            add_whitespace();
        }
        layout_line
    }

    fn align_left(&self) -> LayoutLine<'a> {
        let mut layout_line = LayoutLine::new(Vec::new());
        for (i, word) in self.words.iter().enumerate() {
            layout_line.elements.push(LayoutElement::Word(*word));
            if i != self.words.len() - 1 {
                layout_line.elements.push(LayoutElement::Whitespace(1));
            }
        }
        if self.remaining_space > 0 {
            layout_line
                .elements
                .push(LayoutElement::Whitespace(self.remaining_space));
        }
        layout_line
    }
}

fn get_unique_random_numbers(count: usize, start: usize, end: usize) -> Vec<usize> {
    assert!(end >= start);
    assert!(count <= end - start);
    const SEED: [u8; 32] = [
        112, 111, 8, 251, 183, 240, 224, 102, 93, 80, 201, 131, 121, 56, 179, 229, 173, 121, 174,
        140, 110, 128, 175, 230, 32, 98, 16, 147, 254, 24, 1, 86,
    ];

    let mut numbers: Vec<_> = (start..end).collect();
    let mut rng = SmallRng::from_seed(SEED);
    for i in 0..count {
        let index_to_swap = rng.gen_range(0..numbers.len() - i);
        numbers.swap(i, index_to_swap);
    }
    numbers.resize(count, 0);
    numbers.sort();
    numbers.reverse();
    numbers
}

// This has to respect flow, so it can't happen first.
// It has to happen during layout calculation. Otherwise it's retarded.
fn split_words_longer_than_screen_width<'a>(
    screen_width: usize,
    words: &[StyledWord<'a>],
) -> Vec<StyledWord<'a>> {
    words
        .iter()
        .flat_map(|word| {
            let mut word = word.clone();
            let mut r = Vec::new();
            while styled_word_length(&word) > screen_width {
                let (w, remainder) = split_styled_word(&word, screen_width);
                r.push(w);
                word = remainder;
            }
            r.push(word);
            r
        })
        .collect()
}

fn get_words_in_lines<'a>(screen_width: usize, words: &[StyledWord<'a>]) -> Vec<WordsInLine<'a>> {
    let words = split_words_longer_than_screen_width(screen_width, words);
    let mut words = words.as_slice();
    let mut lines = Vec::new();
    while words.len() > 0 {
        let mut line = WordsInLine {
            words: Vec::new(),
            remaining_space: screen_width,
        };

        loop {
            line.words.push(words[0]);
            line.remaining_space -= styled_word_length(&words[0]);
            words = &words[1..];
            let is_last_word_in_line =
                words.len() == 0 || line.remaining_space <= styled_word_length(&words[0]);
            // For every word except the last one, we need to subtract 1 from the remaining space
            // to account for the whitespace. (There must be at least one space after each word.)
            if is_last_word_in_line {
                break;
            }
            line.remaining_space -= 1;
        }

        lines.push(line);
    }
    lines
}

pub fn calculate_layout<'a>(screen_width: usize, words: &[StyledWord<'a>]) -> Vec<LayoutLine<'a>> {
    let mut layout_lines = Vec::new();
    let words_in_lines = get_words_in_lines(screen_width, words);
    for (i, words_in_line) in words_in_lines.iter().enumerate() {
        if i != words_in_lines.len() - 1 {
            layout_lines.push(words_in_line.spread_evenly());
        } else {
            layout_lines.push(words_in_line.align_left());
        }
    }
    layout_lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_tests() {
        struct TestCase {
            screen_width: usize,
            words: Vec<StyledWord<'static>>,
            expected_layout: Vec<LayoutLine<'static>>,
        }

        use LayoutElement::*;

        let test_cases = vec![
            TestCase {
                screen_width: 11,
                words: vec!["Hello".into(), "world".into()],
                expected_layout: vec![LayoutLine::new(vec![
                    Word("Hello".into()),
                    Whitespace(1),
                    Word("world".into()),
                ])],
            },
            TestCase {
                screen_width: 12,
                words: vec!["Hello".into(), "world".into()],
                expected_layout: vec![LayoutLine::new(vec![
                    Word("Hello".into()),
                    Whitespace(1),
                    Word("world".into()),
                    Whitespace(1),
                ])],
            },
            TestCase {
                screen_width: 15,
                words: vec!["Hello".into(), "world".into()],
                expected_layout: vec![LayoutLine::new(vec![
                    Word("Hello".into()),
                    Whitespace(1),
                    Word("world".into()),
                    Whitespace(4),
                ])],
            },
            TestCase {
                screen_width: 10,
                words: vec!["Hello".into(), "dear".into(), "world".into()],
                expected_layout: vec![
                    LayoutLine::new(vec![
                        Word("Hello".into()),
                        Whitespace(1),
                        Word("dear".into()),
                    ]),
                    LayoutLine::new(vec![Word("world".into()), Whitespace(5)]),
                ],
            },
            TestCase {
                screen_width: 11,
                words: vec!["Hello".into(), "dear".into(), "world".into()],
                expected_layout: vec![
                    LayoutLine::new(vec![
                        Word("Hello".into()),
                        Whitespace(2),
                        Word("dear".into()),
                    ]),
                    LayoutLine::new(vec![Word("world".into()), Whitespace(6)]),
                ],
            },
            TestCase {
                screen_width: 12,
                words: vec!["Hello".into(), "dear".into(), "world".into()],
                expected_layout: vec![
                    LayoutLine::new(vec![
                        Word("Hello".into()),
                        Whitespace(3),
                        Word("dear".into()),
                    ]),
                    LayoutLine::new(vec![Word("world".into()), Whitespace(7)]),
                ],
            },
            TestCase {
                screen_width: 11,
                words: vec![
                    "Hello".into(),
                    "world".into(),
                    "Hello".into(),
                    "world".into(),
                ],
                expected_layout: vec![
                    LayoutLine::new(vec![
                        Word("Hello".into()),
                        Whitespace(1),
                        Word("world".into()),
                    ]),
                    LayoutLine::new(vec![
                        Word("Hello".into()),
                        Whitespace(1),
                        Word("world".into()),
                    ]),
                ],
            },
            TestCase {
                screen_width: 11,
                words: vec!["Hello".into(), "world".into(), "xyz".into()],
                expected_layout: vec![
                    LayoutLine::new(vec![
                        Word("Hello".into()),
                        Whitespace(1),
                        Word("world".into()),
                    ]),
                    LayoutLine::new(vec![Word("xyz".into()), Whitespace(8)]),
                ],
            },
            TestCase {
                screen_width: 10,
                words: vec!["Hello".into(), "world".into()],
                expected_layout: vec![
                    LayoutLine::new(vec![Word("Hello".into()), Whitespace(5)]),
                    LayoutLine::new(vec![Word("world".into()), Whitespace(5)]),
                ],
            },
            /*
            TestCase {
                screen_width: 5,
                words: vec!["verylongword".into()],
                expected_layout: vec![
                    LayoutLine(vec![Word("veryl".into())]),
                    LayoutLine(vec![Word("ongwo".into())]),
                    LayoutLine(vec![Word("rd".into()), Whitespace(3)]),
                ],
            },
            TestCase {
                screen_width: 5,
                words: vec!["verylongword and some more".into()],
                expected_layout: vec![
                    LayoutLine(vec![Word("veryl".into())]),
                    LayoutLine(vec![Word("ongwo".into())]),
                    LayoutLine(vec![Word("rd".into()), Whitespace(3)]),
                    LayoutLine(vec![Word("and".into()), Whitespace(2)]),
                    LayoutLine(vec![Word("some".into()), Whitespace(1)]),
                    LayoutLine(vec![Word("more".into()), Whitespace(1)]),
                ],
            },
            TestCase {
                screen_width: 5,
                words: vec!["verylongword and some more.".into()],
                expected_layout: vec![
                    LayoutLine(vec![Word("veryl".into())]),
                    LayoutLine(vec![Word("ongwo".into())]),
                    LayoutLine(vec![Word("rd".into()), Whitespace(3)]),
                    LayoutLine(vec![Word("and".into()), Whitespace(2)]),
                    LayoutLine(vec![Word("some".into()), Whitespace(1)]),
                    LayoutLine(vec![Word("more.".into())]),
                ],
            },
            TestCase {
                screen_width: 10,
                words: vec!["verylongword and so on".into()],
                expected_layout: vec![
                    LayoutLine(vec![Word("verylongwo".into())]),
                    LayoutLine(vec![
                        Word("rd".into()),
                        Whitespace(2),
                        Word("and".into()),
                        Whitespace(1),
                        Word("so".into()),
                    ]),
                    LayoutLine(vec![Word("on".into()), Whitespace(8)]),
                ],
            },
            */
        ];

        for (
            i,
            TestCase {
                screen_width,
                words,
                expected_layout,
            },
        ) in test_cases.into_iter().enumerate()
        {
            println!("{}", i);
            let layout = calculate_layout(screen_width, &words);
            assert_eq!(layout, expected_layout)
        }
    }
}
