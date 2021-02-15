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
        let num_extra_spaces = self.remaining_space - gaps_between_words * spaces_per_gap;
        assert!(num_extra_spaces <= gaps_between_words);

        let mut extra_spaces = get_unique_random_numbers(num_extra_spaces, 0, gaps_between_words);

        for (i, word) in self.words.iter().enumerate() {
            let mut add_word = || layout_line.elements.push(LayoutElement::Word(*word));

            add_word();

            let mut add_whitespace = || {
                if i == self.words.len() - 1 {
                    return;
                }

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
pub mod tests {
    use super::*;

    pub fn debug_print(layout: &[LayoutLine]) {
        for line in layout {
            for layout_element in line.elements.iter() {
                match layout_element {
                    LayoutElement::Word(word) => print!("{}", word.text),
                    LayoutElement::Whitespace(n) => {
                        for _ in 0..*n {
                            print!(" ");
                        }
                    }
                }
            }
            print!("\n");
        }
    }

    const TEXT: &str = "Old software and hardware manuals (when there were such things) go through a lifecycle for me:\n
references for a couple years, then \"trash books\" taking up space, then, when they are 20+ years old,\n
they are \"antiques\". I'm happy to still have my Borland TurboPascal/C/Asm manuals.";

    #[test]
    fn layout_tests() {
        for screen_width in (20..=120).rev() {
            let words: Vec<StyledWord> = TEXT
                .split_ascii_whitespace()
                .into_iter()
                .map(|w| w.into())
                .collect();
            let layout = calculate_layout(screen_width, &words);
            for line in layout.iter() {
                let mut sum = 0;

                for layout_element in line.elements.iter() {
                    match layout_element {
                        LayoutElement::Word(w) => sum += w.text.len(),
                        &LayoutElement::Whitespace(n) => {
                            sum += n;
                            if n == 0 {
                                debug_print(&layout);
                                panic!(format!(
                                    "Found a 0-length whitespace, see debug output (screen width {})",
                                    screen_width
                                ));
                            }
                        }
                    }
                }

                if sum != screen_width {
                    debug_print(&layout);
                    panic!(format!(
                        "Line width not equal to screen width ({}), see debug output",
                        screen_width
                    ));
                }
            }
        }
    }
}
