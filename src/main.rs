use core::fmt;
use std::{collections::HashSet, fmt::Write, num::NonZeroU32, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Word to start from
    #[arg(short, long)]
    start_word: String,
    /// Word to end at
    #[arg(short, long)]
    end_word: String,

    /// File for a set of words, seperated by newlines/whitespace
    #[arg(long, value_name = "FILE")]
    word_set: Option<PathBuf>,
}

/// 0b0000_0000_0000_aaaa_abbb_bbcc_cccd_dddd
#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
struct Word(u32);

/*
impl Word {
    const OFFSET: u32 = 5;
    const LETTER: u32 = 0b11111;
    const WORD: u32 = 0xFFFFF;

    fn offset_for(i: u32) -> u32 {
        (3 - i) * Self::OFFSET
    }

    fn get_letter(&self, i: u32) -> u32 {
        (self.0 >> Self::offset_for(i)) & Self::LETTER
    }

    fn new(s: &str) -> Self {
        assert!(s.len() == 4, "The word length must be 4");
        let mut word = 0;

        for (i, c) in s.chars().enumerate() {
            assert!(
                ('a'..='z').contains(&c),
                "Only lower case english alphabet letters are allowed"
            );

            let c = (c as u8 - b'a') as u32;
            word |= c << Self::offset_for(i as u32);
        }
        Self(word)
    }
}
*/

impl Word {
    fn new(s: &str) -> Self {
        assert!(s.len() == 4, "The word length must be 4");
        let mut word = 0;

        for (i, c) in s.chars().enumerate() {
            assert!(
                ('a'..='z').contains(&c),
                "Only lower case english alphabet letters are allowed"
            );

            let c = (c as u8 - b'a') as u32;
            word += c * 26u32.pow(3 - i as u32);
        }
        Self(word)
    }

    #[inline(always)]
    fn get_letter(&self, i: u32) -> u32 {
        (self.0 % 26u32.pow(4 - i)) / 26u32.pow(3 - i)
    }

    fn distance(&self, other: Word) -> u16 {
        let mut d = 0;
        for i in 0..4 {
            d += (self.get_letter(i) != other.get_letter(i)) as u16;
        }
        d
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..4 {
            let c = self.get_letter(i);
            let c = (b'a' + c as u8) as char;
            f.write_char(c)?;
        }
        Ok(())
    }
}

struct WordNode {
    word: Word,
    connected: Vec<u32>,
}

fn connect_words(mut words: Vec<Word>) -> Vec<WordNode> {
    words.sort_unstable();
    words.dedup();

    let mut word_nodes: Vec<_> = words
        .iter()
        .map(|word| WordNode {
            word: *word,
            connected: vec![],
        })
        .collect();

    let mut grid: Vec<Option<NonZeroU32>> = vec![None; 26usize.pow(4)];

    for (idx, word) in words.iter().enumerate() {
        let inz = NonZeroU32::new(idx as u32 + 1).unwrap();
        grid[word.0 as usize] = Some(inz);
        for dim in 0..4 {
            let offset = 26u32.pow(3 - dim);
            let letter = word.get_letter(dim);

            for i in 1..letter + 1 {
                let test_word = word.0 - i * offset;
                if let Some(i) = grid[test_word as usize] {
                    let i = i.get() - 1;
                    word_nodes[i as usize].connected.push(idx as u32);
                    word_nodes[idx].connected.push(i);
                }
            }
        }
    }

    word_nodes
}

fn weave(start: Word, end: Word, words: &Vec<WordNode>) -> Option<Vec<Word>> {
    let start_idx = words.iter().position(|n| n.word == start).unwrap();

    let mut open = HashSet::new();
    let mut g_score = vec![u16::MAX; words.len()];
    let mut f_score = vec![u16::MAX; words.len()];
    let mut came_from = vec![u32::MAX; words.len()];
    open.insert(start_idx as u32);
    g_score[start_idx] = 0;
    f_score[start_idx] = 0;

    while let Some(&node) = open
        .iter()
        .min_by_key(|&&n| g_score[n as usize] + words[n as usize].word.distance(end))
    {
        let word = &words[node as usize];
        if word.word == end {
            let mut node = node;
            let mut path = Vec::new();

            while came_from[node as usize] != u32::MAX {
                path.insert(0, words[node as usize].word);
                node = came_from[node as usize];
            }

            path.insert(0, words[node as usize].word);

            return Some(path);
        }

        open.remove(&node);

        for &n in &word.connected {
            let n_word = &words[n as usize];
            let tentative_g_score = g_score[node as usize] + word.word.distance(n_word.word);
            if tentative_g_score < g_score[n as usize] {
                came_from[n as usize] = node;
                g_score[n as usize] = tentative_g_score;
                f_score[n as usize] = tentative_g_score + n_word.word.distance(end);
                open.insert(n);
            }
        }
    }

    None
}

fn main() {
    let cli = Cli::parse();

    let start = Word::new(&cli.start_word);
    let end = Word::new(&cli.end_word);

    let words = if let Some(word_set_file) = cli.word_set {
        match std::fs::read_to_string(word_set_file) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Failed reading word set file.");
                return;
            }
        }
    } else {
        include_str!("words").to_string()
    }
    .split_whitespace()
    .map(|s| Word::new(s))
    .chain([start, end])
    .collect::<Vec<_>>();

    let words = connect_words(words);

    let result = weave(start, end, &words);

    match result {
        Some(s) => {
            println!("Solution:");
            for word in s {
                println!("\t{word}");
            }
        }
        None => {
            println!("No valid solution found.");
        }
    }
}
