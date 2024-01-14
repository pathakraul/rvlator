// Read binary file.
// Decode the instructions

mod rvlator;

fn print_rvlator() {
    println!("
░       ░░░  ░░░░  ░░  ░░░░░░░░░      ░░░        ░░░      ░░░       ░░
▒  ▒▒▒▒  ▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒▒▒▒▒  ▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒  ▒
▓       ▓▓▓▓  ▓▓  ▓▓▓  ▓▓▓▓▓▓▓▓  ▓▓▓▓  ▓▓▓▓▓  ▓▓▓▓▓  ▓▓▓▓  ▓▓       ▓▓
█  ███  █████    ████  ████████        █████  █████  ████  ██  ███  ██
█  ████  █████  █████        ██  ████  █████  ██████      ███  ████  █\n");
}

fn main() {
    print_rvlator();
    rvlator::rvlator();
}
