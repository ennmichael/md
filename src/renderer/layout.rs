use crate::markdown_parser::{Style, StyledWord};

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutLine<'a>(Vec<LayoutElement<'a>>);

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

        let mut layout_line = LayoutLine(Vec::new());
        let gaps_between_words = self.words.len() - 1;
        let spaces_per_gap = self.remaining_space / gaps_between_words;
        assert!(spaces_per_gap > 0);
        let mut extra_spaces = self.remaining_space - spaces_per_gap;
        assert!(extra_spaces <= spaces_per_gap);

        for (i, word) in self.words.iter().enumerate() {
            let mut add_word = || layout_line.0.push(LayoutElement::Word(*word));

            add_word();

            let mut add_whitespace = || {
                if i != self.words.len() - 1 {
                    if extra_spaces > 0 {
                        layout_line
                            .0
                            .push(LayoutElement::Whitespace(spaces_per_gap + 1));
                        // Above it says +1 because there are some extra spaces left,
                        // so we're adding them to gaps one-by-one.
                        extra_spaces -= 1;
                    } else {
                        layout_line
                            .0
                            .push(LayoutElement::Whitespace(spaces_per_gap));
                    }
                }
            };

            add_whitespace();
        }
        layout_line
    }

    fn align_left(&self) -> LayoutLine<'a> {
        let mut layout_line = LayoutLine(Vec::new());
        for (i, word) in self.words.iter().enumerate() {
            layout_line.0.push(LayoutElement::Word(*word));
            if i != self.words.len() - 1 {
                layout_line.0.push(LayoutElement::Whitespace(1));
            }
        }
        // The -1 is because we haven't inserted whitespace after the last word.
        let whitespace_at_the_end = self.remaining_space - (self.words.len() - 1);
        if whitespace_at_the_end > 0 {
            layout_line
                .0
                .push(LayoutElement::Whitespace(whitespace_at_the_end));
        }
        layout_line
    }
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

        // Adding line.words.len() at the end here because each word has a space after it (except
        // the last word).
        // Taking away the 1 because the last word doesn't have a space after it.
        while words.len() != 0
            && line.remaining_space >= styled_word_length(&words[0]) + line.words.len() - 1
        {
            line.words.push(words[0]);
            line.remaining_space -= styled_word_length(&words[0]);
            words = &words[1..];
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
                expected_layout: vec![LayoutLine(vec![
                    Word("Hello".into()),
                    Whitespace(1),
                    Word("world".into()),
                ])],
            },
            TestCase {
                screen_width: 12,
                words: vec!["Hello".into(), "world".into()],
                expected_layout: vec![LayoutLine(vec![
                    Word("Hello".into()),
                    Whitespace(1),
                    Word("world".into()),
                    Whitespace(1),
                ])],
            },
            TestCase {
                screen_width: 15,
                words: vec!["Hello".into(), "world".into()],
                expected_layout: vec![LayoutLine(vec![
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
                    LayoutLine(vec![
                        Word("Hello".into()),
                        Whitespace(1),
                        Word("dear".into()),
                    ]),
                    LayoutLine(vec![Word("world".into()), Whitespace(5)]),
                ],
            },
            TestCase {
                screen_width: 11,
                words: vec!["Hello".into(), "dear".into(), "world".into()],
                expected_layout: vec![
                    LayoutLine(vec![
                        Word("Hello".into()),
                        Whitespace(2),
                        Word("dear".into()),
                    ]),
                    LayoutLine(vec![Word("world".into()), Whitespace(6)]),
                ],
            },
            TestCase {
                screen_width: 12,
                words: vec!["Hello".into(), "dear".into(), "world".into()],
                expected_layout: vec![
                    LayoutLine(vec![
                        Word("Hello".into()),
                        Whitespace(3),
                        Word("dear".into()),
                    ]),
                    LayoutLine(vec![Word("world".into()), Whitespace(7)]),
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
                    LayoutLine(vec![
                        Word("Hello".into()),
                        Whitespace(1),
                        Word("world".into()),
                    ]),
                    LayoutLine(vec![
                        Word("Hello".into()),
                        Whitespace(1),
                        Word("world".into()),
                    ]),
                ],
            },
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
            let layout = calculate_layout(screen_width, &words);
            assert_eq!(layout, expected_layout)
        }
    }
}
