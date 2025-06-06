use std::io;

fn main() {
    let mut input = String::new();

    loop {
        let mut output = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                for c in input.chars() {
                    output.push(translate_to_runic(c));
                }
            }
            Err(error) => println!("error: {error}"),
        }

        print!("{output}");
        input.clear();
        output.clear();
    }
}

// Icelandic Runic created by Alexander R. (https://www.omniglot.com/conscripts/icelandicrunic.htm)
fn translate_to_runic(c: char) -> char {
    match c {
        ' ' => ' ',
        '\n' => '\n',
        '(' => '(',
        ')' => ')',
        'A' | 'a' => 'ᛆ',
        'Á' | 'á' => 'ᚨ',
        'B' | 'b' => 'ᛒ',
        'D' | 'd' => 'ᛑ',
        'Ð' | 'ð' => 'ᚧ',
        'E' | 'e' => 'ᛂ',
        'É' | 'é' => 'ᛖ',
        'F' | 'f' => 'ᚠ',
        'G' | 'g' => 'ᚵ',
        'H' | 'h' => 'ᚼ',
        'I' | 'i' => 'ᛁ',
        'Í' | 'í' => 'ᛇ',
        'J' | 'j' => 'ᛃ',
        'K' | 'k' => 'ᚴ',
        'L' | 'l' => 'ᛚ',
        'M' | 'm' => 'ᛘ',
        'N' | 'n' => 'ᚿ',
        'O' | 'o' => 'ᚮ',
        'Ó' | 'ó' => 'ᛟ',
        'P' | 'p' => 'ᛔ',
        'R' | 'r' => 'ᚱ',
        'S' | 's' => 'ᛋ',
        'T' | 't' => 'ᛐ',
        'U' | 'u' => 'ᚢ',
        'Ú' | 'ú' => 'ᚤ',
        'V' | 'v' => 'ᚡ',
        'X' | 'x' => 'ᛪ',
        'Y' | 'y' => 'ᛣ',
        'Ý' | 'ý' => 'ᛨ',
        'Þ' | 'þ' => 'ᚦ',
        'Æ' | 'æ' => 'ᛅ',
        'Ö' | 'ö' => 'ᚯ',
        _ => panic!("an invalid character was reached"),
    }
}
