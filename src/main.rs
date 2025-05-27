mod config;
mod generator;
mod parser;
mod processor;
mod types;

use config::Config;
use processor::SchemaProcessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_args();

    let processor = SchemaProcessor::new(config.clone());

    // Print statistics before processing if enabled
    if config.show_statistics {
        let stats = processor.get_stats()?;
        stats.print();
        println!();
    }

    // Process all schemas
    processor.process_all()?;

    Ok(())
}
