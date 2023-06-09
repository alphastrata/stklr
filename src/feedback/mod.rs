//!
//! Macros for easier-coloured pretty printing.
//!

/// Generates the printers so that RawLine(s) when being processed can get pretty-printed when
/// using Preview or Verbose modes.
#[macro_export]
macro_rules! green {
    ($msg:expr, $n:expr) => {{
        let coloured = ansi_term::Colour::Green.paint($msg.to_string());
        print!("+ {} ", ansi_term::Colour::Yellow.paint($n.to_string()));
        print!("{} \n", coloured);
    }};
}
/// the red version of green! macro
#[macro_export]
macro_rules! red {
    ($msg:expr, $n:expr) => {{
        let coloured = ansi_term::Colour::Red.paint($msg.to_string());
        print!("- {} ", ansi_term::Colour::Yellow.paint($n.to_string()));
        print!("{} \n", coloured);
    }};
}

// the blue! takes a single arg, unlike green! or red!
#[macro_export]
macro_rules! blue {
    ($msg:expr) => {{
        ansi_term::Colour::Blue.paint($msg.to_string());
        //print!("{} \n", coloured);
    }};
}

/// Take a true/false on a pass value for a test and glamour that sucker up`
#[macro_export]
macro_rules! show {
    ($msg:expr) => {
        match $msg {
            true => ansi_term::Colour::Green.paint($msg.to_string()),
            _ => ansi_term::Colour::Red.paint($msg.to_string()),
        }
    };
}

//TODO: put all the printers you want into a single macro.
//TODO: use macros in the Display for AdjustedLine and RawLine.
