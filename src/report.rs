//! Console reporting helpers (port of src/framework/report.py).

pub const WIDTH: usize = 100;

/// Centered horizontal rule with a title, like `rich.console.rule`.
pub fn section(title: &str) {
    println!("{}", rule(title));
}

pub fn rule(title: &str) -> String {
    let t = format!(" {title} ");
    let n = t.chars().count();
    if n >= WIDTH {
        return t.trim().to_string();
    }
    let left = (WIDTH - n) / 2;
    format!("{}{}{}", "─".repeat(left), t, "─".repeat(WIDTH - n - left))
}

/// Format a float with `dp` decimals, `nan` for NaN (matches Python's `format`).
pub fn fnum(v: f64, dp: usize) -> String {
    if v.is_nan() {
        "nan".into()
    } else if v.is_infinite() {
        if v > 0.0 {
            "inf".into()
        } else {
            "-inf".into()
        }
    } else {
        format!("{v:.dp$}")
    }
}

pub fn table_string(headers: &[&str], rows: &[Vec<String>]) -> String {
    let widths: Vec<usize> = (0..headers.len())
        .map(|c| {
            rows.iter()
                .map(|r| r[c].chars().count())
                .chain([headers[c].chars().count()])
                .max()
                .unwrap_or(0)
        })
        .collect();
    let line = |l: &str, m: &str, r: &str| {
        format!(
            "{l}{}{r}",
            widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join(m)
        )
    };
    let row = |cells: Vec<String>| {
        let body: Vec<String> = cells
            .iter()
            .zip(&widths)
            .map(|(c, w)| format!(" {c:<w$} "))
            .collect();
        format!("│{}│", body.join("│"))
    };
    let mut out = vec![
        line("┏", "┳", "┓"),
        row(headers.iter().map(|h| h.to_string()).collect()),
        line("┡", "╇", "┩"),
    ];
    out.extend(rows.iter().map(|r| row(r.clone())));
    out.push(line("└", "┴", "┘"));
    out.join("\n")
}

pub fn table(headers: &[&str], rows: &[Vec<String>]) {
    println!("{}", table_string(headers, rows));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_nan_and_rule_width() {
        assert_eq!(fnum(f64::NAN, 3), "nan");
        assert_eq!(fnum(1.23456, 3), "1.235");
        assert_eq!(rule("abc").chars().count(), WIDTH);
        let t = table_string(&["a", "bb"], &[vec!["1".into(), "22".into()]]);
        assert_eq!(t.lines().count(), 5);
    }
}
