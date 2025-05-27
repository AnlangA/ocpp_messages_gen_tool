/// 配置结构
#[derive(Debug, Clone)]
pub struct Config {
    pub schema_dir: String,
    pub output_dir: String,
    pub generate_mod_file: bool,
    pub show_statistics: bool,
}

impl Config {
    /// 创建默认配置
    pub fn default() -> Self {
        Self {
            schema_dir: "../tests/schema_validation/schemas/v2.1".to_string(),
            output_dir: "../v2_1/messages".to_string(),
            generate_mod_file: true, // 默认生成 mod.rs 文件
            show_statistics: true,
        }
    }

    /// 从命令行参数创建配置
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut config = Self::default();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--schema-dir" => {
                    if i + 1 < args.len() {
                        config.schema_dir = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("Error: --schema-dir requires a value");
                        std::process::exit(1);
                    }
                }
                "--output-dir" => {
                    if i + 1 < args.len() {
                        config.output_dir = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("Error: --output-dir requires a value");
                        std::process::exit(1);
                    }
                }
                "--no-mod-file" => {
                    config.generate_mod_file = false;
                    i += 1;
                }
                "--mod-file" => {
                    config.generate_mod_file = true;
                    i += 1;
                }
                "--no-stats" => {
                    config.show_statistics = false;
                    i += 1;
                }
                "--help" | "-h" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Error: Unknown argument '{}'", args[i]);
                    Self::print_help();
                    std::process::exit(1);
                }
            }
        }

        config
    }

    /// 打印帮助信息
    pub fn print_help() {
        println!("OCPP v2.1 Message Generator");
        println!();
        println!("USAGE:");
        println!("    gen_messages [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("    --schema-dir <DIR>    Schema files directory (default: ../tests/schema_validation/schemas/v2.1)");
        println!(
            "    --output-dir <DIR>    Output directory (default: ../generated/v2_1/messages)"
        );
        println!("    --mod-file            Generate mod.rs file (default)");
        println!("    --no-mod-file         Don't generate mod.rs file");
        println!("    --no-stats            Don't show statistics");
        println!("    -h, --help            Print help information");
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if !std::path::Path::new(&self.schema_dir).exists() {
            return Err(format!(
                "Schema directory does not exist: {}",
                self.schema_dir
            ));
        }

        Ok(())
    }
}
