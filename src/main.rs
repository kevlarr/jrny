use clap::clap_app;
use jrny::{self, commands};

fn main() {
    let mut app = clap_app! {jrny =>
        (about: "Data's a journey, so manage yours with jrny - simple, isolated PostgreSQL schema management")
        (version: "0.1.0")

        (@subcommand begin =>
            (about: "Sets up relevant files and directories for a new revision timeline")
            (@arg dirpath: +required "The directory in which to set up new project files - will be created if does not exist")
        )

        (@subcommand revise =>
            (about: "Generates a timestamped SQL revision")
            (@arg name: +required "Name of the revision")
            (@arg config: -c --config [FILE] +takes_value "Sets a custom config file")
        )

        (@subcommand review =>
            (about: "Reviews which plans need to be applied, which have been applied and when, and whether or not plans already applied appear to differ from the plan file")
            (@arg config: -c --config [FILE] +takes_value "Sets a custom config file")
        )

        (@subcommand on =>
            (about: "Reviews and applies the available revisions")
            (@arg config: -c --config [FILE] +takes_value "Sets a custom config file")
            (@arg commit: --commit !takes_value "Commits the transaction, false by default")
        )

        (@subcommand anew =>
            (about: "Archives database revision records and removes associated files")
            (@arg config: -c --config [FILE] +takes_value "Sets a custom config file")
            (@arg commit: --commit !takes_value "Commits the transaction, false by default")
        )
    };

    let result = match app.clone().get_matches().subcommand() {
        ("begin", Some(cmd)) => commands::begin(
            cmd.value_of("dirpath").unwrap(),
        ),
        ("revise", Some(cmd)) => commands::revise(
            cmd.value_of("name").unwrap(),
            cmd.value_of("config"),
        ),
        ("review", Some(cmd)) => commands::review(
            cmd.value_of("config"),
        ),
        ("on", Some(cmd)) => commands::on(
            cmd.value_of("config"),
            cmd.is_present("commit"),
        ),

        // TODO How to print help in absence of subcommand without cloning?
        _ => {
            app.print_help().expect("Failed to print help");
            println!("");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
    }
}
