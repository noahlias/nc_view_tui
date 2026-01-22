use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;

pub fn parse_key_spec(raw: &str) -> Result<super::KeySpec> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("empty key binding"));
    }
    if trimmed.len() == 1 {
        let ch = trimmed.chars().next().unwrap();
        return Ok(super::KeySpec {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::empty(),
        });
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower == "plus" {
        return Ok(super::KeySpec {
            code: KeyCode::Char('+'),
            modifiers: KeyModifiers::empty(),
        });
    }
    if lower == "minus" {
        return Ok(super::KeySpec {
            code: KeyCode::Char('-'),
            modifiers: KeyModifiers::empty(),
        });
    }
    if lower == "space" {
        return Ok(super::KeySpec {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::empty(),
        });
    }

    let mut modifiers = KeyModifiers::empty();
    let parts: Vec<&str> = lower.split('+').collect();
    if parts.is_empty() {
        return Err(anyhow!("invalid key binding: {}", raw));
    }

    let key_part = parts.last().unwrap().trim();
    for part in &parts[..parts.len().saturating_sub(1)] {
        match part.trim() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "" => {}
            other => return Err(anyhow!("unknown modifier: {}", other)),
        }
    }

    let code = parse_key_code(key_part)?;
    Ok(super::KeySpec { code, modifiers })
}

fn parse_key_code(raw: &str) -> Result<KeyCode> {
    if raw.len() == 1 {
        let ch = raw.chars().next().unwrap();
        return Ok(KeyCode::Char(ch));
    }

    match raw {
        "esc" | "escape" => Ok(KeyCode::Esc),
        "enter" | "return" => Ok(KeyCode::Enter),
        "tab" => Ok(KeyCode::Tab),
        "backspace" => Ok(KeyCode::Backspace),
        "left" => Ok(KeyCode::Left),
        "right" => Ok(KeyCode::Right),
        "up" => Ok(KeyCode::Up),
        "down" => Ok(KeyCode::Down),
        "home" => Ok(KeyCode::Home),
        "end" => Ok(KeyCode::End),
        "pageup" => Ok(KeyCode::PageUp),
        "pagedown" => Ok(KeyCode::PageDown),
        "plus" => Ok(KeyCode::Char('+')),
        "minus" => Ok(KeyCode::Char('-')),
        other => Err(anyhow!("unknown key: {}", other)),
    }
}

pub fn parse_color(raw: &str) -> Result<Color> {
    let lower = raw.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return Err(anyhow!("empty color"));
    }
    if let Some(color) = parse_named_color(&lower) {
        return Ok(color);
    }
    if let Some(color) = parse_hex_color(&lower) {
        return Ok(color);
    }
    Err(anyhow!("unknown color: {}", raw))
}

fn parse_named_color(name: &str) -> Option<Color> {
    match name {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => None,
    }
}

fn parse_hex_color(raw: &str) -> Option<Color> {
    let hex = raw.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

pub fn parse_marker(raw: &str) -> Result<ratatui::symbols::Marker> {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "braille" => Ok(ratatui::symbols::Marker::Braille),
        "halfblock" | "half_block" | "half-block" => Ok(ratatui::symbols::Marker::HalfBlock),
        "dot" => Ok(ratatui::symbols::Marker::Dot),
        "block" => Ok(ratatui::symbols::Marker::Block),
        "bar" => Ok(ratatui::symbols::Marker::Bar),
        _ => Err(anyhow!("unknown canvas_marker: {}", raw)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_marker_values() {
        assert!(matches!(parse_marker("braille").unwrap(), ratatui::symbols::Marker::Braille));
        assert!(matches!(
            parse_marker("halfblock").unwrap(),
            ratatui::symbols::Marker::HalfBlock
        ));
        assert!(matches!(parse_marker("dot").unwrap(), ratatui::symbols::Marker::Dot));
        assert!(matches!(parse_marker("block").unwrap(), ratatui::symbols::Marker::Block));
        assert!(matches!(parse_marker("bar").unwrap(), ratatui::symbols::Marker::Bar));
    }
}
