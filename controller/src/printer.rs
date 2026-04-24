use crate::story::Story;
use chrono::{DateTime, Local};
use escpos::{
    driver::SerialPortDriver,
    printer::Printer,
    printer_options::PrinterOptions,
    utils::{DebugMode, JustifyMode, Protocol, UnderlineMode},
};
use log::warn;

pub(crate) fn init(port: &str, baud: u32) -> escpos::errors::Result<Printer<SerialPortDriver>> {
    let driver = SerialPortDriver::open(port, baud, None).unwrap();
    let mut printer = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));
    printer
        .debug_mode(Some(DebugMode::Dec))
        .init()?
        .feed()?
        .writeln("The Phantasm Press")?
        .writeln(&format!("Git rev: {}", git_version::git_version!()))?
        .writeln(&format!("{}", Local::now()))?
        .feed()?
        .print()?
        .reset_size()?
        .writeln(".........1.........2.........3.........4.........5")?
        .writeln("12345678901234567890123456789012345678901234567890")?
        .print()?
        .size(2, 2)?
        .writeln(".........1.........2.........3")?
        .writeln("123456789012345678901234567890")?
        .feed()?
        .print_cut()?;
    Ok(printer)
}

const LINE_CHAR_WIDTH_SMALL: usize = 42;
const LINE_CHAR_WIDTH_BIG: usize = 21;

pub(crate) trait PrinterExt {
    fn print_story(&mut self, story: &Story) -> escpos::errors::Result<()>;
}

impl PrinterExt for Printer<SerialPortDriver> {
    fn print_story(&mut self, story: &Story) -> escpos::errors::Result<()> {
        // Clean the title of non-ASCII characters and warn about any remaining
        let clean_title = replace_non_ascii_characters(story.title());
        warn_on_non_ascii_chars(&clean_title);

        // Clean the body text of non-ASCII characters and warn about any remaining
        let clean_body = replace_non_ascii_characters(story.text());
        warn_on_non_ascii_chars(&clean_body);

        // Print the title in big text, underlined
        self.size(2, 2)?
            .underline(UnderlineMode::Single)?
            .justify(JustifyMode::CENTER)?;
        for line in split_into_lines(&clean_title, LINE_CHAR_WIDTH_BIG) {
            self.writeln(&line)?.print()?;
        }
        self.feed()?.print()?;

        // Print the story text body
        self.reset_size()?
            .underline(UnderlineMode::None)?
            .justify(JustifyMode::LEFT)?;
        for line in split_into_lines(&clean_body, LINE_CHAR_WIDTH_SMALL) {
            self.writeln(&line)?.print()?;
        }
        self.feeds(2)?;

        // Get the timestamp in local time
        let timestamp: DateTime<Local> = (*story.timestamp()).into();

        // Get the word count
        let words = words_count::count(clean_body).words;

        // Print the footer
        self.justify(JustifyMode::CENTER)?
            .writeln("_____  stats for nerds  _____")?
            .justify(JustifyMode::LEFT)?
            .write("Revision: ")?
            .writeln(git_version::git_version!())?
            .print()?
            .writeln(&format!(
                "Timestamp: {}",
                timestamp.format("%Y-%m-%d %H:%M:%S")
            ))?
            .print()?
            .write("Word count: ")?
            .writeln(&format!("{words}"))?
            .print()?
            .writeln("Cards:")?
            .print()?;
        for card in story.origin().iter() {
            self.writeln(&format!(" - {}", card.id()))?.print()?;
        }
        self.write("Model: ")?
            .writeln(story.model())?
            .print()?
            .writeln(&format!(
                "Tokens (prompt/completion): {}/{}",
                story.usage().prompt_tokens(),
                story.usage().completion_tokens()
            ))?
            .print()?
            .writeln(&format!("Cost: ${:.5}", story.usage().cost()))?
            .justify(JustifyMode::CENTER)?
            .feeds(2)?
            .print()?
            .writeln("thelateshows.org.uk")?
            .writeln("makerspace.org.uk")?
            .writeln("github.com/DanNixon/the-phantasm-press")?
            .print()?
            .feed()?
            .print_cut()?;

        Ok(())
    }
}

fn warn_on_non_ascii_chars(s: &str) {
    for (byte_idx, ch) in s.char_indices() {
        if !ch.is_ascii() {
            warn!("Found non-ASCII char {} at index {byte_idx}", ch);
        }
    }
}

fn replace_non_ascii_characters(s: &str) -> String {
    s.replace("—", "-")
        .replace("—", "-")
        .replace("–", "-")
        .replace("’", "'")
        .replace("‘", "'")
        .replace("“", "\"")
        .replace("”", "\"")
        .replace("…", "...")
        .replace("é", "e")
}

fn split_into_lines(s: &str, max_line_length: usize) -> Vec<String> {
    textwrap::wrap(s, max_line_length)
        .into_iter()
        .map(|l| l.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_short_string() {
        let s = "Hello";
        let got = split_into_lines(s, 32);
        let expected = vec!["Hello".to_string()];
        assert_eq!(got, expected);
    }

    #[test]
    fn splits_exact_width() {
        let s = "a".repeat(32);
        let got = split_into_lines(&s, 32);
        let expected = vec![s];
        assert_eq!(got, expected);
    }

    #[test]
    fn wraps_on_word_boundaries() {
        let s = "The quick brown fox jumps over the lazy dog";
        let got = split_into_lines(s, 10);
        let expected = vec!["The quick", "brown fox", "jumps over", "the lazy", "dog"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        assert_eq!(got, expected);
    }

    #[test]
    fn empty_string_returns_empty_vec() {
        let s = "";
        let got = split_into_lines(s, 32);
        let expected = vec!["".to_string()];
        assert_eq!(got, expected);
    }

    #[test]
    fn format_uses_32_char_limit() {
        let s = "word ".repeat(20);
        for l in split_into_lines(&s, 32) {
            assert!(
                l.len() <= 32,
                "line '{}' exceeds 32 chars (len={})",
                l,
                l.len()
            );
        }
    }
}
