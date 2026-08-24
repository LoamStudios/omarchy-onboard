use console::style;

pub fn heading(s: &str) {
    println!("\n{}", style(s).bold().underlined());
}

pub fn group(title: &str, count: &str) {
    println!("\n{} {}", style(title).bold().cyan(), style(format!("({count})")).dim());
}

pub fn item(s: &str) {
    println!("  {s}");
}

pub fn note(s: &str) {
    println!("    {}", style(s).dim());
}

pub fn human_bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 { format!("{n} B") } else { format!("{v:.1} {}", UNITS[i]) }
}
