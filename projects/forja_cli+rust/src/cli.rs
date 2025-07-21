use clap::{ArgMatches, Command, arg, command};

pub(crate) fn get_matches() -> ArgMatches {
    let project_path_arg = arg!([path] "Path of the project (if not provided defaults to the current working directory)");
    let is_wasm = arg!(--wasm "Use the WASM variation of the project if avaliable");

    command!()
        .subcommand(Command::new("todo").about("Print the current tasks to do"))
        .subcommand(Command::new("check").about("Check the quality of the code"))
        .subcommand(Command::new("fix").about("Try to fix issues in the code"))
        .subcommand(
            Command::new("generate-ignores")
                .about("Generate ignore-like files from the '//ignores' directory"),
        )
        .subcommand(
            Command::new("dev")
                .about("Run a project in dev mode")
                .arg(&project_path_arg),
        )
        .subcommand(
            Command::new("test")
                .about("Run the tests of a project")
                .arg(&project_path_arg),
        )
        .subcommand(
            Command::new("build")
                .about("Build a project")
                .arg(&project_path_arg)
                .arg(&is_wasm),
        )
        .subcommand(
            Command::new("docs")
                .about("Build the documentation of a project")
                .arg(&project_path_arg)
                .arg(arg!(--open "Open the docs on the default application of the OS"))
                .arg(&is_wasm),
        )
        .get_matches()
}
