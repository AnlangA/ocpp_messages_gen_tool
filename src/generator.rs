use convert_case::{Case, Casing};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::types::{MessagePair, StructInfo};

/// 生成配对的 Rust 文件
pub fn generate_paired_file(
    pair: &MessagePair,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let filename = format!("{}.rs", pair.base_name.to_case(Case::Snake));
    let output_path = PathBuf::from(output_dir).join(filename);

    let mut code = String::new();

    // Add optimized imports
    let optimized_imports = optimize_imports(&pair.combined_imports);
    for import in optimized_imports {
        code.push_str(&import);
        code.push('\n');
    }
    code.push('\n');

    // Generate Request struct if available
    if let Some(request) = &pair.request {
        code.push_str(&generate_struct_code(request, "request")?);
        code.push('\n');
    }

    // Generate Response struct if available
    if let Some(response) = &pair.response {
        code.push_str(&generate_struct_code(response, "response")?);
    }

    fs::write(output_path, code)?;
    Ok(())
}

/// 生成单个结构体的代码
pub fn generate_struct_code(
    struct_info: &StructInfo,
    message_type: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();

    // Add struct comment
    code.push_str(&format!(
        "/// {} body for the {} {}.\n",
        message_type
            .chars()
            .next()
            .unwrap()
            .to_uppercase()
            .collect::<String>()
            + &message_type[1..],
        struct_info
            .name
            .replace("Request", "")
            .replace("Response", ""),
        message_type
    ));

    // Add struct definition
    code.push_str("#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]\n");
    code.push_str("#[serde(rename_all = \"camelCase\")]\n");
    code.push_str(&format!("pub struct {} {{\n", struct_info.name));

    // Add fields
    for (index, field) in struct_info.fields.iter().enumerate() {
        // Add empty line before each field (except the first one)
        if index > 0 {
            code.push('\n');
        }

        // Add description as comment if available
        if let Some(description) = &field.description {
            if !description.is_empty() {
                code.push_str(&format!("    /// {}\n", description));
            }
        }

        // Add serde attributes using the existing project's multi-line format
        let mut serde_attrs = Vec::new();

        // Handle field renaming for Rust keywords or when camelCase conversion doesn't match
        if field.name != field.original_name {
            // 检查是否需要 rename：如果原始名称转换为 snake_case 后与字段名不同，则需要 rename
            let expected_snake_case = field.original_name.to_case(Case::Snake);
            if field.name != expected_snake_case {
                serde_attrs.push(format!("rename = \"{}\"", field.original_name));
            }
        }

        // Handle optional fields
        if field.is_optional {
            serde_attrs.push("skip_serializing_if = \"Option::is_none\"".to_string());
        }

        // Add serde attribute if needed
        if !serde_attrs.is_empty() {
            if serde_attrs.len() == 1 {
                code.push_str(&format!("    #[serde({})]\n", serde_attrs[0]));
            } else {
                // Multi-line format for multiple attributes
                code.push_str("    #[serde(\n");
                for (i, attr) in serde_attrs.iter().enumerate() {
                    if i == serde_attrs.len() - 1 {
                        code.push_str(&format!("        {}\n", attr));
                    } else {
                        code.push_str(&format!("        {},\n", attr));
                    }
                }
                code.push_str("    )]\n");
            }
        }

        // Add validation attributes
        add_validation_attributes(&mut code, field);

        // Add field definition
        let field_type = if field.is_optional {
            format!("Option<{}>", field.rust_type)
        } else {
            field.rust_type.clone()
        };

        code.push_str(&format!("    pub {}: {},\n", field.name, field_type));
    }

    code.push_str("}\n");
    Ok(code)
}

/// 添加验证属性
fn add_validation_attributes(code: &mut String, field: &crate::types::FieldInfo) {
    if field.needs_validation {
        if field.rust_type == "String" {
            // 检查是否有长度限制
            if let Some(max_length) = &field.max_length {
                code.push_str(&format!("    #[validate(length(max = {}))]\n", max_length));
            } else {
                code.push_str("    #[validate(length(max = 255))]\n");
            }
        } else if field.rust_type.starts_with("Vec<") {
            // 检查 Vec 的内容类型
            let inner_type = field.rust_type.strip_prefix("Vec<").unwrap().strip_suffix(">").unwrap();
            if inner_type.ends_with("Type") && !inner_type.ends_with("EnumType") {
                // 只对包含复杂数据类型的 Vec 添加 nested 验证
                code.push_str("    #[validate(nested)]\n");
            }
            // Vec<String>, Vec<i32>, Vec<EnumType> 等不需要验证属性
        } else if field.rust_type.ends_with("Type") && !field.rust_type.ends_with("EnumType") {
            // 只对非枚举类型添加 nested 验证
            code.push_str("    #[validate(nested)]\n");
        } else if field.rust_type == "i32" {
            // 添加数值范围验证
            if let (Some(min), Some(max)) = (&field.min_value, &field.max_value) {
                code.push_str(&format!(
                    "    #[validate(range(min = {}, max = {}))]\n",
                    min, max
                ));
            } else if let Some(min) = &field.min_value {
                code.push_str(&format!("    #[validate(range(min = {}))]\n", min));
            } else if let Some(max) = &field.max_value {
                code.push_str(&format!("    #[validate(range(max = {}))]\n", max));
            } else if field.name.contains("id") {
                code.push_str("    #[validate(range(min = 0))]\n");
            }
        } else if field.rust_type == "Decimal" {
            // 添加 Decimal 类型的数值范围验证
            if let (Some(min), Some(max)) = (&field.min_value, &field.max_value) {
                code.push_str(&format!(
                    "    #[validate(range(min = {}, max = {}))]\n",
                    min, max
                ));
            } else if let Some(min) = &field.min_value {
                code.push_str(&format!("    #[validate(range(min = {}))]\n", min));
            } else if let Some(max) = &field.max_value {
                code.push_str(&format!("    #[validate(range(max = {}))]\n", max));
            }
        }
    }
}

/// 生成模块文件
pub fn generate_mod_file(
    message_pairs: &[String],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mod_path = PathBuf::from(output_dir).join("mod.rs");
    let mut code = String::new();

    code.push_str("// Generated message modules for OCPP v2.1\n");
    code.push_str("// This file is auto-generated. Do not edit manually.\n\n");

    // Add module declarations
    for base_name in message_pairs {
        let module_name = base_name.to_case(Case::Snake);
        code.push_str(&format!("pub mod {};\n", module_name));
    }

    code.push('\n');

    // Add re-exports
    for base_name in message_pairs {
        let module_name = base_name.to_case(Case::Snake);

        code.push_str(&format!(
            "pub use {}::{{{}Request, {}Response}};\n",
            module_name, base_name, base_name
        ));
    }

    fs::write(mod_path, code)?;
    Ok(())
}

/// 优化导入语句，将同一模块的导入合并，使用现有项目的多行格式
fn optimize_imports(imports: &HashSet<String>) -> Vec<String> {
    let mut grouped_imports: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut other_imports = Vec::new();

    for import in imports {
        if let Some(parsed) = parse_import(import) {
            match parsed {
                ImportType::Crate { module, types } => {
                    grouped_imports
                        .entry(module)
                        .or_insert_with(Vec::new)
                        .extend(types);
                }
                ImportType::Other(import_str) => {
                    other_imports.push(import_str);
                }
            }
        }
    }

    let mut result = Vec::new();

    // Add grouped crate imports first
    for (module, mut types) in grouped_imports {
        types.sort();
        types.dedup();

        if types.len() == 1 {
            result.push(format!("use crate::{}::{};", module, types[0]));
        } else if types.len() <= 3 {
            // 短列表使用单行格式
            result.push(format!("use crate::{}::{{{}}};", module, types.join(", ")));
        } else {
            // 长列表使用多行格式，匹配现有项目风格
            let mut multi_line_import = format!("use crate::{}::{{\n", module);
            for (i, type_name) in types.iter().enumerate() {
                if i == types.len() - 1 {
                    multi_line_import.push_str(&format!("    {},\n", type_name));
                } else {
                    multi_line_import.push_str(&format!("    {}, \n", type_name));
                }
            }
            multi_line_import.push_str("};");
            result.push(multi_line_import);
        }
    }

    // Add other imports after crate imports
    other_imports.sort();
    result.extend(other_imports);

    result
}

#[derive(Debug)]
enum ImportType {
    Crate { module: String, types: Vec<String> },
    Other(String),
}

/// 解析导入语句
fn parse_import(import: &str) -> Option<ImportType> {
    if import.starts_with("use crate::") {
        // 解析 crate 导入
        let import_part = import.strip_prefix("use crate::")?.strip_suffix(";")?;

        // 找到最后一个 :: 来分离模块和类型
        if let Some(last_colon_pos) = import_part.rfind("::") {
            let module = import_part[..last_colon_pos].to_string();
            let type_name = import_part[last_colon_pos + 2..].to_string();

            Some(ImportType::Crate {
                module,
                types: vec![type_name],
            })
        } else {
            // 如果没有找到 ::，说明是直接导入模块
            Some(ImportType::Crate {
                module: String::new(),
                types: vec![import_part.to_string()],
            })
        }
    } else {
        // 其他类型的导入
        Some(ImportType::Other(import.to_string()))
    }
}
