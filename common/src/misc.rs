use std::iter;

use crate::units::Miliseconds;

pub fn human_duration(duration: Miliseconds) -> String {
    let ms = duration.raw();
    if ms < 1000.0 {
        format!("{ms}ms")
    } else if ms < 60_000.0 {
        format!("{:.2}s", ms / 1000.0)
    } else if ms < 3_600_000.0 {
        let minutes = ms / 60_000.0;
        let seconds = (minutes - minutes.floor()) * 60.0;
        format!("{:.0}m {:.2}s", minutes.floor(), seconds)
    } else {
        let hours = ms / 3_600_000.0;
        let minutes = (hours - hours.floor()) * 60.0;
        let seconds = (minutes - minutes.floor()) * 60.0;
        format!(
            "{:.0}h {:.0}m {:.2}s",
            hours.floor(),
            minutes.floor(),
            seconds
        )
    }
}

pub fn separate_thousands(number: impl TryInto<u64>) -> String {
    let str = number.try_into().unwrap_or(u64::MAX).to_string();
    let separators = [None, None, Some(',')]
        .into_iter()
        .cycle()
        .skip(3 - str.len() % 3);

    (str.chars().map(Some))
        .zip(iter::once(None).chain(separators))
        .flat_map(|(a, b)| [b, a])
        .flatten()
        .collect()
}

pub fn subscript_number(num: impl Into<u64>) -> String {
    const SUBSCRIPT: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

    let mut num = num.into();
    let mut out = String::new();
    while num > 0 {
        out.push(SUBSCRIPT[(num % 10) as usize]);
        num /= 10;
    }

    out
}
