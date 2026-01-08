mod cli;
mod config;
mod highlighter;
mod matcher;
mod processor;
mod search;

use clap::Parser;
use cli::Cli;
use processor::Processor;
use rayon::prelude::*;
use search::FileDiscovery;
use std::process;
use std::sync::Arc;

fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Convert to config
    let config = match cli.into_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Create processor
    let processor = match Processor::new(config.clone()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Discover files
    let discovery = FileDiscovery::new(&config);
    let files = match discovery.discover() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Process files or stdin
    if files.is_empty() {
        // Process stdin
        if let Err(e) = processor.process_stdin() {
            eprintln!("{}", e);
            process::exit(1);
        }
    } else if files.len() == 1 {
        // Single file - no need for parallelization
        if let Err(e) = processor.process_file(&files[0]) {
            eprintln!("{}", e);
        }
    } else {
        // Multiple files - use parallel processing
        let processor = Arc::new(processor);

        // Set number of threads if specified
        if let Some(jobs) = config.jobs {
            rayon::ThreadPoolBuilder::new()
                .num_threads(jobs)
                .build_global()
                .ok();
        }

        // Process files in parallel, but collect results to maintain order
        let results: Vec<_> = files
            .par_iter()
            .map(|file| {
                match processor.process_file(file) {
                    Ok(_) => None,
                    Err(e) => Some((file.clone(), e)),
                }
            })
            .collect();

        // Report any errors
        for (file, error) in results.into_iter().flatten() {
            eprintln!("Error processing {}: {}", file.display(), error);
        }
    }
}
