use std::process::exit;

const LEFT: &str = ">> ";
const RIGHT: &str = " <<";

pub fn pretty_box(strings: &[&str]) -> String {
    let max_length = strings.iter().map(|s| s.len()).max().unwrap_or(0);

    let border = "*".repeat(LEFT.len() + max_length + RIGHT.len());

    let mut result = String::new();
    result.push_str(&format!("{}\n", border));

    for s in strings {
        result.push_str(&format!(
            "{LEFT}{: ^width$}{RIGHT}\n",
            s,
            width = max_length
        ));
    }

    result.push_str(&format!("{}\n", border));
    result
}

const DEFAULT: &[&str] = &[
    "Looks like an error occurred :-/",
    "",
    "If this is a bug in kystash, please contact me via Matrix or email.",
    "Those can be found on my website: https://kybe.xyz/.",
    "",
    "Exit code: 1",
];

pub fn fatal_error() -> ! {
    eprint!("{}", pretty_box(DEFAULT));
    exit(1);
}
