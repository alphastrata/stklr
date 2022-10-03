pub mod cmd;
pub mod feedback;
pub mod search;

// /// `SearchBuf` is a buffer holding all lines with matches.
// #[derive(Default, Debug, Clone)]
// struct SearchBuf {
//     //m: HashMap<usize, LineMatch>,
//     file: PathBuf,
//     idents: Vec<usize>,
//     docstrings: Vec<usize>,
// }
// impl SearchBuf {
//     /// Initialise a new `SearchBbuf` from a given file.
//     // NOTE: mpsc pattern used here, modify with care.
//     fn init<P>(filename: P) -> Self
//     where
//         P: AsRef<Path>,
//     {
//         let mut sb: SearchBuf = Self::default();
//         let (tx, rx) = mpsc::channel();
//
//         // Search docstrings, and build collection of them.
//         let file_buf_read = read_lines(filename);
//
//         if let Ok(lines) = file_buf_read {
//             lines
//                 .collect::<Vec<_>>()
//                 .iter() //NOTE: indentionally not par_iter, as we'll save that for multiple files.
//                 .enumerate()
//                 .for_each(|(e, l)| {
//                     let tx_c = tx.clone();
//                     //TODO: A macro for all .contains(tag) pls
//                     if let Ok(l) = l {
//                         if l.contains("///") {
//                             let lm = LineMatch {
//                                 line_num: e,
//                                 contents: l.into(),
//                                 is_ident: false,
//                                 hits: 0, //NOTE: this is updated later
//                             };
//                             if let Err(e) = tx_c.send(lm) {
//                                 eprintln!("Error tx_c.send() -> {}", e)
//                             }
//                         }
//                     }
//                 });
//             // close the sender. (otherwise the rx listens indefinitely)
//         }
//
//         // // Search file for $idents
//         // if let Ok(lines) = file_buf_read{
//         //     lines.collect::<Vec<_>>().iter().enumerate().for_each((e,l){
//         //         let tx_c = tx.clone();
//         //
//         //         if let Ok(l) = l{
//         //             // do regex matching and send
//         //             let (caps, hits) = idents_from_regex(&l);
//         //             let lm LineMatch{
//         //                 line_num: e,
//         //                 is_ident: true,
//         //                 contents: caps.name("ident"),
//         //                 hits: 0,
//         //             }.
//         //
//         //         }
//         //     });
    /// Creates a new [`CodeBase`] from the glob searching the current working directory the app is run
//         // }
//
//         while let Ok(lm) = rx.recv() {
//             if lm.has_ident{
//                 sb.idents.push(lm.line_num)
//             } else {
//                 sb.docstrings.push(lm.line_num)
//             }
//             sb.m.insert(lm.line_num, lm);
//         }
//         sb
//     }
// }
    /// makes adjustments to RawLines from within [`RawSourceCode`]'s RawLines
    /// Process preview_changes [`RawSourceCode`] [`find_docs`]
