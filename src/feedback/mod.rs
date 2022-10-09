//!
//! Macros for easier-coloured pretty printing.
//!

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
/// the red version of green! macro
#[macro_export]
macro_rules! red {
    ($msg:expr, $n:expr) => {{
        let coloured = Colour::Red.paint($msg.to_string());
        print!("- {} ", Colour::Yellow.paint($n.to_string()));
        print!("{} \n", coloured);
    }};
}

//TODO: put all the printers you want into a single macro.
//TODO: use macros in the Display for AdjustedLine and RawLine.