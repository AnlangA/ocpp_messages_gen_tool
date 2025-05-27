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
        let message_type = if pair.request.is_none() {
            // This is a standalone message, determine type from name
            if response.name.ends_with("Request") {
                "request"
            } else if response.name.ends_with("Response") {
                "response"
            } else {
                "message"
            }
        } else {
            "response"
        };
        code.push_str(&generate_struct_code(response, message_type)?);
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
    if struct_info.name.ends_with("Request") || struct_info.name.ends_with("Response") {
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
    } else {
        // For standalone messages
        code.push_str(&format!("/// {} message structure.\n", struct_info.name));
    }

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

    code.push_str("}\n\n");

    // Add implementation block
    code.push_str(&generate_impl_block(struct_info)?);

    Ok(code)
}

/// 添加验证属性
fn add_validation_attributes(code: &mut String, field: &crate::types::FieldInfo) {
    if field.needs_validation {
        if field.rust_type == "String" {
            // 处理字符串长度限制
            let mut length_constraints = Vec::new();

            if let Some(min_length) = &field.min_length {
                length_constraints.push(format!("min = {}", min_length));
            }

            if let Some(max_length) = &field.max_length {
                length_constraints.push(format!("max = {}", max_length));
            }

            if !length_constraints.is_empty() {
                code.push_str(&format!(
                    "    #[validate(length({}))]\n",
                    length_constraints.join(", ")
                ));
            } else {
                // 默认最大长度限制
                code.push_str("    #[validate(length(max = 255))]\n");
            }
        } else if field.rust_type.starts_with("Vec<") {
            // 处理数组类型
            let inner_type = field
                .rust_type
                .strip_prefix("Vec<")
                .unwrap()
                .strip_suffix(">")
                .unwrap();

            // 添加数组长度验证
            let mut length_constraints = Vec::new();

            if let Some(min_items) = &field.min_items {
                length_constraints.push(format!("min = {}", min_items));
            }

            if let Some(max_items) = &field.max_items {
                length_constraints.push(format!("max = {}", max_items));
            }

            if !length_constraints.is_empty() {
                code.push_str(&format!(
                    "    #[validate(length({}))]\n",
                    length_constraints.join(", ")
                ));
            }

            // 添加嵌套验证（如果需要）
            if inner_type.ends_with("Type") && !inner_type.ends_with("EnumType") {
                // 只对包含复杂数据类型的 Vec 添加 nested 验证
                // 注意：这需要内部类型也实现 Validate trait
                code.push_str("    #[validate(nested)]\n");
            }
        } else if field.rust_type.ends_with("Type") && !field.rust_type.ends_with("EnumType") {
            // 只对非枚举类型添加 nested 验证
            // 注意：这需要类型也实现 Validate trait
            code.push_str("    #[validate(nested)]\n");
        } else if field.rust_type == "i32"
            || field.rust_type == "i64"
            || field.rust_type == "u32"
            || field.rust_type == "u64"
        {
            // 处理整数类型的数值范围验证
            add_numeric_range_validation(code, field);
        } else if field.rust_type == "f32" || field.rust_type == "f64" {
            // 处理浮点数类型的数值范围验证
            add_numeric_range_validation(code, field);
        } else if field.rust_type == "Decimal" {
            // Decimal 类型需要特殊处理，因为 validator crate 不直接支持 Decimal
            // 我们可以添加自定义验证或者跳过验证
            // 这里我们跳过验证，因为 Decimal 类型本身就有精度保证
        }
    }
}

/// 添加数值范围验证
fn add_numeric_range_validation(code: &mut String, field: &crate::types::FieldInfo) {
    let mut range_constraints = Vec::new();

    if let Some(min) = &field.min_value {
        // 对于整数类型，如果是整数值则不显示小数点
        if field.rust_type == "i32"
            || field.rust_type == "i64"
            || field.rust_type == "u32"
            || field.rust_type == "u64"
        {
            if min.fract() == 0.0 {
                range_constraints.push(format!("min = {}", *min as i64));
            } else {
                range_constraints.push(format!("min = {}", min));
            }
        } else {
            range_constraints.push(format!("min = {}", min));
        }
    }

    if let Some(max) = &field.max_value {
        // 对于整数类型，如果是整数值则不显示小数点
        if field.rust_type == "i32"
            || field.rust_type == "i64"
            || field.rust_type == "u32"
            || field.rust_type == "u64"
        {
            if max.fract() == 0.0 {
                range_constraints.push(format!("max = {}", *max as i64));
            } else {
                range_constraints.push(format!("max = {}", max));
            }
        } else {
            range_constraints.push(format!("max = {}", max));
        }
    }

    if !range_constraints.is_empty() {
        code.push_str(&format!(
            "    #[validate(range({}))]\n",
            range_constraints.join(", ")
        ));
    } else if field.name.contains("id") && (field.rust_type == "i32" || field.rust_type == "u32") {
        // 为 ID 字段添加默认的非负验证
        code.push_str("    #[validate(range(min = 0))]\n");
    }
}

/// 生成模块文件
pub fn generate_mod_file(
    message_pairs: &[String],
    standalone_messages: &[String],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mod_path = PathBuf::from(output_dir).join("mod.rs");
    let mut code = String::new();

    // 收集所有模块名并排序
    let mut all_modules = Vec::new();

    // 添加配对消息的模块名
    for base_name in message_pairs {
        let module_name = base_name.to_case(Case::Snake);
        all_modules.push((module_name, base_name.clone(), true)); // true 表示是配对消息
    }

    // 添加独立消息的模块名
    for base_name in standalone_messages {
        let module_name = base_name.to_case(Case::Snake);
        all_modules.push((module_name, base_name.clone(), false)); // false 表示是独立消息
    }

    // 按模块名排序
    all_modules.sort_by(|a, b| a.0.cmp(&b.0));

    // 添加模块声明
    for (module_name, _, _) in &all_modules {
        code.push_str(&format!("pub mod {};\n", module_name));
    }

    code.push('\n');

    // 添加重新导出，按类型分组
    let mut paired_exports = Vec::new();
    let mut standalone_exports = Vec::new();

    for (module_name, base_name, is_paired) in &all_modules {
        if *is_paired {
            paired_exports.push((module_name.clone(), base_name.clone()));
        } else {
            standalone_exports.push((module_name.clone(), base_name.clone()));
        }
    }

    // 导出配对消息
    for (module_name, base_name) in &paired_exports {
        code.push_str(&format!(
            "pub use {}::{{{}Request, {}Response}};\n",
            module_name, base_name, base_name
        ));
    }

    // 导出独立消息
    for (module_name, base_name) in &standalone_exports {
        code.push_str(&format!("pub use {}::{};\n", module_name, base_name));
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

/// 生成结构体的实现块
fn generate_impl_block(struct_info: &StructInfo) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();

    code.push_str(&format!("impl {} {{\n", struct_info.name));

    // Generate new method
    code.push_str(&generate_new_method(struct_info)?);
    code.push('\n');

    // Generate setter methods
    for field in &struct_info.fields {
        code.push_str(&generate_setter_method(field)?);
        code.push('\n');
    }

    // Generate getter methods
    for field in &struct_info.fields {
        code.push_str(&generate_getter_method(field)?);
        code.push('\n');
    }

    // Generate with methods for optional fields
    for field in &struct_info.fields {
        if field.is_optional {
            code.push_str(&generate_with_method(field)?);
            code.push('\n');
        }
    }

    code.push_str("}\n");
    Ok(code)
}

/// 生成 new 方法
fn generate_new_method(struct_info: &StructInfo) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();

    // Collect required fields
    let required_fields: Vec<&crate::types::FieldInfo> = struct_info
        .fields
        .iter()
        .filter(|field| !field.is_optional)
        .collect();

    // Generate method signature
    code.push_str("    /// Creates a new instance of the struct.\n");
    code.push_str("    ///\n");

    // Add parameter documentation
    for field in &required_fields {
        let param_doc = if let Some(description) = &field.description {
            description.clone()
        } else {
            format!("The {} field", field.name)
        };
        code.push_str(&format!("    /// * `{}` - {}\n", field.name, param_doc));
    }

    code.push_str("    ///\n");
    code.push_str("    /// # Returns\n");
    code.push_str("    ///\n");
    code.push_str("    /// A new instance of the struct with required fields set and optional fields as None.\n");
    code.push_str("    pub fn new(");

    // Add parameters
    for (i, field) in required_fields.iter().enumerate() {
        if i > 0 {
            code.push_str(", ");
        }
        code.push_str(&format!("{}: {}", field.name, field.rust_type));
    }

    code.push_str(") -> Self {\n");
    code.push_str("        Self {\n");

    // Initialize fields
    for field in &struct_info.fields {
        if field.is_optional {
            code.push_str(&format!("            {}: None,\n", field.name));
        } else {
            code.push_str(&format!("            {},\n", field.name));
        }
    }

    code.push_str("        }\n");
    code.push_str("    }\n");

    Ok(code)
}

/// 生成 setter 方法
fn generate_setter_method(
    field: &crate::types::FieldInfo,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();

    let param_doc = if let Some(description) = &field.description {
        description.clone()
    } else {
        format!("The {} field", field.name)
    };

    let field_type = if field.is_optional {
        format!("Option<{}>", field.rust_type)
    } else {
        field.rust_type.clone()
    };

    code.push_str(&format!("    /// Sets the {} field.\n", field.name));
    code.push_str("    ///\n");
    code.push_str(&format!("    /// * `{}` - {}\n", field.name, param_doc));
    code.push_str("    ///\n");
    code.push_str("    /// # Returns\n");
    code.push_str("    ///\n");
    code.push_str("    /// A mutable reference to self for method chaining.\n");
    code.push_str(&format!(
        "    pub fn set_{}(&mut self, {}: {}) -> &mut Self {{\n",
        field.name, field.name, field_type
    ));
    code.push_str(&format!("        self.{} = {};\n", field.name, field.name));
    code.push_str("        self\n");
    code.push_str("    }\n");

    Ok(code)
}

/// 生成 getter 方法
fn generate_getter_method(
    field: &crate::types::FieldInfo,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();

    let param_doc = if let Some(description) = &field.description {
        description.clone()
    } else {
        format!("The {} field", field.name)
    };

    let return_type = if field.is_optional {
        format!("Option<&{}>", field.rust_type)
    } else {
        format!("&{}", field.rust_type)
    };

    code.push_str(&format!(
        "    /// Gets a reference to the {} field.\n",
        field.name
    ));
    code.push_str("    ///\n");
    code.push_str("    /// # Returns\n");
    code.push_str("    ///\n");
    code.push_str(&format!("    /// {}\n", param_doc));
    code.push_str(&format!(
        "    pub fn get_{}(&self) -> {} {{\n",
        field.name, return_type
    ));

    if field.is_optional {
        code.push_str(&format!("        self.{}.as_ref()\n", field.name));
    } else {
        code.push_str(&format!("        &self.{}\n", field.name));
    }

    code.push_str("    }\n");

    Ok(code)
}

/// 生成 with 方法（仅用于可选字段）
fn generate_with_method(
    field: &crate::types::FieldInfo,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut code = String::new();

    let param_doc = if let Some(description) = &field.description {
        description.clone()
    } else {
        format!("The {} field", field.name)
    };

    code.push_str(&format!(
        "    /// Sets the {} field and returns self for builder pattern.\n",
        field.name
    ));
    code.push_str("    ///\n");
    code.push_str(&format!("    /// * `{}` - {}\n", field.name, param_doc));
    code.push_str("    ///\n");
    code.push_str("    /// # Returns\n");
    code.push_str("    ///\n");
    code.push_str("    /// Self with the field set.\n");
    code.push_str(&format!(
        "    pub fn with_{}(mut self, {}: {}) -> Self {{\n",
        field.name, field.name, field.rust_type
    ));
    code.push_str(&format!(
        "        self.{} = Some({});\n",
        field.name, field.name
    ));
    code.push_str("        self\n");
    code.push_str("    }\n");

    Ok(code)
}
