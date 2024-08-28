use crate::{FormatType, Token};

pub fn advance(text: &Vec<char>, current: &mut usize) -> char {
    *current += 1;
    if *current < text.len() {
        return text[*current - 1];
    }
    return '\0';
}

pub fn add_token(tokens: &mut Vec<Token>, start: usize, current: usize, format: FormatType) {
    tokens.push(Token {
        start,
        length: current - start - 1,
        format,
    });
}

pub fn add_new_line(tokens: &mut Vec<Token>, current: usize) {
    tokens.push(Token {
        start: current - 1,
        length: 2,
        format: FormatType::NewLine,
    })
}

pub fn peek_ahead(text: &Vec<char>, current: usize) -> char {
    if current + 1 < text.len() {
        return text[current + 1];
    }
    return '\0';
}

pub fn add_space(tokens: &mut Vec<Token>, current: usize) {
    tokens.push(Token {
        start: current,
        length: 1,
        format: FormatType::Space,
    })
}
