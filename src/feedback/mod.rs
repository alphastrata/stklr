#[cfg(test)]
mod tests {
    use ansi_term::Colour::{Blue, Yellow};

    #[test]
    fn colour_tests() {
        println!(
            "Demonstrating {} and {}!",
            Blue.bold().paint("blue bold"),
            Yellow.underline().paint("yellow underline")
        );

        println!("Yellow on blue: {}", Yellow.on(Blue).paint("wow!"))
    }
}

/// Generates the printers so that RawLine(s) when being processed can get pretty-printed when
/// using Preview or Verbose modes.
#[macro_export]
macro_rules! green {
    ($msg:expr, $n:expr) => {{
        let coloured = Colour::Green.paint($msg.to_string());
        print!("+ {} ", Colour::Yellow.paint($n.to_string()));
        print!("{} \n", coloured);
    }};
}
#[macro_export]
macro_rules! red {
    ($msg:expr, $n:expr) => {{
        let coloured = Colour::Red.paint($msg.to_string());
        print!("- {} ", Colour::Yellow.paint($n.to_string()));
        print!("{} \n", coloured);
    }};
}
