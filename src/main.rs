use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use log::{warn, Level, LevelFilter, Log, Metadata, Record};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use jrny::context::{Config, Environment};
use jrny::{Error as JrnyError, Result as JrnyResult, CONF, ENV};


#[derive(Parser, Debug)]
#[command(
    about = "PostgreSQL schema revisions made simple - just add SQL!",
    long_about = "\
PostgreSQL schema revisions made simple - just add SQL!

Journey aims to offer a clean, easy to use workflow for managing schema revisions \
for any project where plain SQL is appropriate, while also guaranteeing that:

  * Revision files have not been changed or removed after being applied

  * Revisions cannot be applied in different orders across environments",
    after_help = "\
For any given command, use the `-h` flag to view a concise description of the command
or `--help` for a more verbose description, eg. `jrny --help` or `jrny plan -h`.",
    // The autogenerated help command has a confusing description, and it
    // also clutters up the available subcommands list, so just disable it
    // until the situation improves and describe example usages of `--help`
    // in the `after_help` block.
    disable_help_subcommand = true,
    max_term_width = 100,
    next_line_help = false,
    version
)]
struct Jrny {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Begin(Begin),
    Plan(Plan),
    Review(Review),
    Embark(Embark),
}

#[derive(Parser, Debug)]
#[command(
    about = "Creates files and directories for a new journey",
    long_about = "\
Starts a new journey in the target directory, creating the configuration and environment \
files as well as the directory to hold revisions.",
)]
struct Begin {
    #[arg(
        help = "The directory in which to create project resources",
        long_help ="\
The directory in which to create project resources - will be created if it does not exist. \
The directory can be non-empty as long as it does not already contain Journey .toml files. \
Additionally, the revisions directory can exist already as long as it itself is empty.",
    )]
    dir_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(
    about = "Creates a new .sql revision file",
    long_about = "\
Creates a new .sql revision file with a unique sequential id, timestamp, and given title.",
)]
struct Plan {
    #[command(flatten)]
    cfg: CliConfig,

    #[arg(
        help = "Title of the new revision",
        long_help = "\
Title of the new revision. Surround with quotation marks to include whitespace in the title."
    )]
    name: String,
}

#[derive(Parser, Debug)]
#[command(
    about = "Lists all revisions, reporting on any errors observed",
    long_about = "\
Lists all revisions along with dates created and applied, along with \
any errors found such as a revision having been changed or removed \
after being applied."
)]
struct Review {
    #[clap(flatten)]
    cfg: CliConfig,

    #[clap(flatten)]
    env: CliEnvironment,
}

#[derive(Parser, Debug)]
#[command(
    about = "Reviews existing revisions for errors and applies pending revisions",
    long_about = "\
Reviews existing revisions for errors. Applies pending revisions only if \
no errors with existing revisions were found.",

)]
struct Embark {
    #[command(flatten)]
    cfg: CliConfig,

    #[command(flatten)]
    env: CliEnvironment,
}

#[derive(Parser, Debug)]
struct CliConfig {
    #[arg(
        help = "\
Path to required .toml configuration file, defaulting to `jrny.toml` in the \
current directory",
        short,
        long,
    )]
    conf_file: Option<PathBuf>,
}

impl TryFrom<CliConfig> for Config {
    type Error = JrnyError;

    fn try_from(cli_cfg: CliConfig) -> Result<Self, Self::Error> {
        let confpath = cli_cfg.conf_file.unwrap_or_else(|| PathBuf::from(CONF));

        Self::from_filepath(&confpath)
    }
}

#[derive(Parser, Debug)]
struct CliEnvironment {
    #[arg(
        help = "\
Path to optional .toml environment file, defaulting to `jrny-env.toml` in the \
same directory as the configuration file",
        short,
        long,
    )]
    env_file: Option<PathBuf>,

    #[arg(
        help = "\
Database connection string if overriding value from (or not using) an environment file",
        short,
        long,
    )]
    db_url: Option<String>,
}

// Can't implement from/into traits if `Config` is involved, since it's technically foreign
impl CliEnvironment {
    fn jrny_environment(self, cfg: &Config) -> JrnyResult<Environment> {
        let envpath = self
            .env_file
            .unwrap_or_else(|| cfg.revisions.directory.parent().unwrap().join(ENV));

        // This validates the env file, even if someone overrides it with the
        // database url flag. The file itself is optional as long as the
        // database url is supplied.
        let env_file = (match Environment::from_filepath(&envpath) {
            Ok(env) => Ok(Some(env)),
            Err(err) => match err {
                JrnyError::EnvNotFound => Ok(None),
                e => Err(e),
            },
        })?;

        match self.db_url {
            Some(url) => Ok(Environment::from_database_url(&url)),
            None => match env_file {
                Some(env) => Ok(env),
                None => Err(JrnyError::EnvNotFound),
            },
        }
    }
}

// Basic implementation of a Log, as none of the complexity of
// common crates is particularly necessary here and the CLI tool
// just wants to print info! and warn! as basic messages
//
// See: https://docs.rs/log/0.4.11/log/#implementing-a-logger
struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if record.metadata().level() == Level::Warn {
            let mut stderr = StandardStream::stderr(ColorChoice::Always);

            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                .unwrap();
            write!(&mut stderr, "{}", record.args()).unwrap();

            stderr.set_color(ColorSpec::new().set_fg(None)).unwrap();
            writeln!(&mut stderr).unwrap();

            return;
        }

        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        writeln!(&mut stdout, "{}", record.args()).unwrap();
    }

    fn flush(&self) {}
}

fn main() -> ExitCode {
    log::set_logger(&Logger)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| e.to_string())
        .unwrap();

    let opts: Jrny = Jrny::parse();

    let result = match opts.subcmd {
        SubCommand::Begin(cmd) => begin(cmd),
        SubCommand::Plan(cmd) => plan(cmd),
        SubCommand::Review(cmd) => review(cmd),
        SubCommand::Embark(cmd) => embark(cmd),
    };

    // Returning the result directly would debugs print the error and exit with an
    // appropriate code, but return an ExitCode instead so that there is
    // greater control over logging and formatting messages.
    //
    // See: https://doc.rust-lang.org/std/process/struct.ExitCode.html#impl-ExitCode
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            warn!("");
            warn!("{}", e);

            // TODO: More fine-grained error-dependent codes?
            // See: https://github.com/kevlarr/jrny/issues/33
            ExitCode::FAILURE
        }
    }
}

fn begin(cmd: Begin) -> JrnyResult<()> {
    jrny::begin(&cmd.dir_path)
}

fn plan(cmd: Plan) -> JrnyResult<()> {
    let cfg: Config = cmd.cfg.try_into()?;

    // TODO: Allow passing in file contents via command-line?
    jrny::plan(&cfg, &cmd.name, None)
}

fn review(cmd: Review) -> JrnyResult<()> {
    let cfg: Config = cmd.cfg.try_into()?;
    let env = cmd.env.jrny_environment(&cfg)?;

    jrny::review(&cfg, &env)
}

fn embark(cmd: Embark) -> JrnyResult<()> {
    let cfg: Config = cmd.cfg.try_into()?;
    let env = cmd.env.jrny_environment(&cfg)?;

    jrny::embark(&cfg, &env)
}
