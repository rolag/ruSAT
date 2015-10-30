extern crate getopts; // https://doc.rust-lang.org/getopts/getopts/index.html
use getopts::Options;
use std::env;

// Show help and exit
fn show_help(program_name: &str, opts: Options) {
    let brief = format!("Usage: {} [options] [FILE|enter system on prompt]", program_name);
    print!("{}", opts.usage(&brief));
}

fn show_version() {
    println!("print license and version here");
}

fn set_options(opts: &mut Options) {
    opts.optflag("m", "model",      "Print a model of the system, or 'unsatisfiable'.");
    opts.optflag("M", "all-models", "Print all models of the system, or 'unsatisfiable'.");
    opts.optflag("T", "tautology",  "Print if the system is a tautology and list all its models.");
    opts.optopt("i",  "input-type", "Use the following format to input the system.", "dimacs");
    opts.optopt("o",  "output",     "Output to the specified file, or stdout if FILE is ``-''.",
                "FILE");
    opts.optflag("h", "help",       "Print help menu and exit, regardless of other arguments.");
    opts.optflag("v", "version",    "Print out version and exit, regardless of other arguments.");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();

    let mut opts = Options::new();
    set_options(&mut opts);
    let matches = match opts.parse(&args[1..]) {
            Ok(all_matches) => { all_matches }
            Err(failure)    => { panic!(failure.to_string()) }
    };
    if matches.opt_present("h") || args.len() == 1 {
        show_help(&program_name, opts);
        return;
    };
    if matches.opt_present("v") {
        show_version();
        return;
    }
}
