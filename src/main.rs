use docopt::Docopt;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::process::exit;

static USAGE: &'static str = "
Usage: city-pop [options] [<data-path>] <city>
       city-pop --help

Options:
    -h, --help      Show this usage message.
    -q, --quiet     Don't show noisy messages.
";

#[derive(Debug, serde::Deserialize)]
struct Args {
    arg_data_path: Option<String>,
    arg_city: String,
    flag_quiet: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Row {
    country: String,
    city: String,
    population: Option<u64>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|err| err.exit());

    match search(args.arg_data_path, &args.arg_city) {
        Ok(results) => {
            for pop_data in results {
                println!(
                    "country={}, city={}, population={:?}",
                    pop_data.country, pop_data.city, pop_data.population
                );
            }
        }
        Err(CliError::NotFound) if args.flag_quiet => {
            exit(1)
        },
        Err(e) => {
            eprintln!("Failed to execute program -- {}", e);
            exit(1)
        }
    }
}

struct PopulationData {
    country: String,
    city: String,
    population: u64,
}

fn search<P: AsRef<Path>>(
    file_path: Option<P>,
    city: &str,
) -> Result<Vec<PopulationData>, CliError> {
    let reader: Box<dyn Read> = match file_path {
        None => Box::new(io::stdin()),
        Some(path) => Box::new(File::open(path)?),
    };

    let mut ans = vec![];
    let mut reader = csv::Reader::from_reader(reader);
    let iter = reader.deserialize::<Row>();
    for row_result in iter {
        let row: Row = row_result?;
        if row.population.is_some() && row.city == city {
            ans.push(PopulationData {
                city: row.city,
                country: row.country,
                population: row.population.unwrap(),
            })
        }
    }
    if ans.is_empty() {
        Err(CliError::NotFound)
    } else {
        Ok(ans)
    }
}

#[derive(Debug)]
enum CliError {
    Io(io::Error),
    Csv(csv::Error),
    NotFound,
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Io(err) => write!(f, "{}", err.to_string()),
            CliError::Csv(err) => write!(f, "{}", err.to_string()),
            CliError::NotFound => write!(f, "No match found for given city"),
        }
    }
}

impl Error for CliError {}

impl From<io::Error> for CliError {
    fn from(io_err: io::Error) -> Self {
        CliError::Io(io_err)
    }
}

impl From<csv::Error> for CliError {
    fn from(csv_err: csv::Error) -> Self {
        CliError::Csv(csv_err)
    }
}
