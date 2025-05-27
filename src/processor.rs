use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

use crate::config::Config;
use crate::generator::{generate_mod_file, generate_paired_file};
use crate::parser::{extract_struct_info_from_file, parse_message_type};
use crate::types::MessagePair;

/// 主要的处理器结构
pub struct SchemaProcessor {
    config: Config,
}

impl SchemaProcessor {
    /// 创建新的处理器实例
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 处理所有 schema 文件
    pub fn process_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate configuration
        self.config.validate()?;

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)?;

        // Collect all JSON files and group them by base name
        let message_pairs = self.collect_message_pairs()?;

        // Generate paired files
        let mut generated_pairs = Vec::new();
        for (base_name, pair) in &message_pairs {
            if pair.is_complete() {
                generate_paired_file(pair, &self.config.output_dir)?;
                generated_pairs.push(base_name.clone());
                println!("Generated: {}", base_name);
            } else {
                println!("Warning: Incomplete pair for {}", base_name);
            }
        }

        // Generate mod.rs file if enabled
        if self.config.generate_mod_file {
            generate_mod_file(&generated_pairs, &self.config.output_dir)?;
            println!("Generated mod.rs file");
        }

        println!("Paired schema processing completed!");
        println!("Generated {} message pairs", generated_pairs.len());
        Ok(())
    }

    /// 收集所有消息对
    fn collect_message_pairs(
        &self,
    ) -> Result<HashMap<String, MessagePair>, Box<dyn std::error::Error>> {
        let mut message_pairs: HashMap<String, MessagePair> = HashMap::new();

        for entry in WalkDir::new(&self.config.schema_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "json" {
                        let filename = entry.path().file_stem().unwrap().to_str().unwrap();
                        let (base_name, is_request) = parse_message_type(filename);

                        let struct_info = extract_struct_info_from_file(entry.path(), filename)?;

                        let pair = message_pairs
                            .entry(base_name.clone())
                            .or_insert_with(|| MessagePair::new(base_name.clone()));

                        if is_request {
                            pair.add_request(struct_info);
                        } else {
                            pair.add_response(struct_info);
                        }
                    }
                }
            }
        }

        Ok(message_pairs)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> Result<ProcessorStats, Box<dyn std::error::Error>> {
        let message_pairs = self.collect_message_pairs()?;
        let complete_pairs = message_pairs.values().filter(|p| p.is_complete()).count();
        let incomplete_pairs = message_pairs.len() - complete_pairs;

        Ok(ProcessorStats {
            total_pairs: message_pairs.len(),
            complete_pairs,
            incomplete_pairs,
        })
    }
}

/// 处理器统计信息
#[derive(Debug)]
pub struct ProcessorStats {
    pub total_pairs: usize,
    pub complete_pairs: usize,
    pub incomplete_pairs: usize,
}

impl ProcessorStats {
    pub fn print(&self) {
        println!("Schema Processing Statistics:");
        println!("  Total message pairs: {}", self.total_pairs);
        println!("  Complete pairs: {}", self.complete_pairs);
        println!("  Incomplete pairs: {}", self.incomplete_pairs);
    }
}
