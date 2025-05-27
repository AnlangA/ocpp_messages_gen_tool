use convert_case::{Case, Casing};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::types::{FieldInfo, StructInfo};

/// 解析消息类型，返回基础名称和是否为请求
pub fn parse_message_type(filename: &str) -> (String, bool) {
    if filename.ends_with("Request") {
        let base_name = filename.strip_suffix("Request").unwrap();
        (base_name.to_string(), true)
    } else if filename.ends_with("Response") {
        let base_name = filename.strip_suffix("Response").unwrap();
        (base_name.to_string(), false)
    } else {
        // Handle special cases like "NotifyPeriodicEventStream" which doesn't have Request/Response suffix
        (filename.to_string(), false)
    }
}

/// 从文件中提取结构体信息
pub fn extract_struct_info_from_file(
    schema_path: &Path,
    struct_name: &str,
) -> Result<StructInfo, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(schema_path)?;
    let schema: Value = serde_json::from_str(&content)?;
    extract_struct_info_with_content(&schema, struct_name, &content)
}

/// 从 JSON schema 中提取结构体信息（带原始内容以保持字段顺序）
pub fn extract_struct_info_with_content(
    schema: &Value,
    struct_name: &str,
    content: &str,
) -> Result<StructInfo, Box<dyn std::error::Error>> {
    let mut imports = HashSet::new();
    let mut fields = Vec::new();

    // Add common imports
    imports.insert("use serde::{Deserialize, Serialize};".to_string());
    imports.insert("use validator::Validate;".to_string());

    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        let required_fields: HashSet<String> = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        // 保持字段顺序：从原始JSON字符串中解析字段顺序
        let field_order = extract_field_order_from_content(content)?;

        // 如果字段顺序提取失败，回退到原有方式
        if field_order.is_empty() {
            // 使用原有的无序遍历方式作为回退
            for (field_name, field_schema) in properties {
                let field_info = extract_field_info(
                    field_name,
                    field_schema,
                    &required_fields,
                    &mut imports,
                    schema,
                )?;
                fields.push(field_info);
            }
        } else {
            // 按照JSON中的顺序处理字段
            for field_name in field_order {
                if let Some(field_schema) = properties.get(&field_name) {
                    let field_info = extract_field_info(
                        &field_name,
                        field_schema,
                        &required_fields,
                        &mut imports,
                        schema,
                    )?;
                    fields.push(field_info);
                }
            }
        }
    }

    Ok(StructInfo {
        name: struct_name.to_string(),
        fields,
        imports,
    })
}

/// 提取字段信息
fn extract_field_info(
    field_name: &str,
    field_schema: &Value,
    required_fields: &HashSet<String>,
    imports: &mut HashSet<String>,
    root_schema: &Value,
) -> Result<FieldInfo, Box<dyn std::error::Error>> {
    let is_optional = !required_fields.contains(field_name);

    // 处理 Rust 关键字，特别是 "type"
    let rust_field_name = if field_name == "type" {
        "type_".to_string()
    } else {
        field_name.to_case(Case::Snake)
    };

    let (rust_type, needs_validation) = determine_rust_type(field_schema, imports, root_schema)?;

    let description = field_schema
        .get("description")
        .and_then(|d| d.as_str())
        .map(|s| s.replace('\r', "").replace('\n', " ").trim().to_string());

    // 提取长度限制
    let max_length = field_schema
        .get("maxLength")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let min_length = field_schema
        .get("minLength")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    // 提取数值范围（支持浮点数）
    let min_value = field_schema.get("minimum").and_then(|v| v.as_f64());

    let max_value = field_schema.get("maximum").and_then(|v| v.as_f64());

    // 提取数组项目数量限制
    let min_items = field_schema
        .get("minItems")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let max_items = field_schema
        .get("maxItems")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    Ok(FieldInfo {
        name: rust_field_name,
        original_name: field_name.to_string(),
        rust_type,
        is_optional,
        needs_validation,
        description,
        max_length,
        min_length,
        min_value,
        max_value,
        min_items,
        max_items,
    })
}

/// 确定 Rust 类型
fn determine_rust_type(
    field_schema: &Value,
    imports: &mut HashSet<String>,
    _root_schema: &Value,
) -> Result<(String, bool), Box<dyn std::error::Error>> {
    // Handle $ref references
    if let Some(ref_path) = field_schema.get("$ref").and_then(|r| r.as_str()) {
        return handle_ref_type(ref_path, imports);
    }

    // Handle arrays
    if let Some(field_type) = field_schema.get("type").and_then(|t| t.as_str()) {
        match field_type {
            "string" => {
                if field_schema.get("format").and_then(|f| f.as_str()) == Some("date-time") {
                    imports.insert("use chrono::{DateTime, Utc};".to_string());
                    Ok(("DateTime<Utc>".to_string(), false))
                } else {
                    Ok(("String".to_string(), true))
                }
            }
            "integer" => Ok(("i32".to_string(), true)),
            "number" => {
                imports.insert("use rust_decimal::Decimal;".to_string());
                Ok(("Decimal".to_string(), true))
            }
            "boolean" => Ok(("bool".to_string(), false)),
            "array" => {
                if let Some(items) = field_schema.get("items") {
                    let (item_type, _) = determine_rust_type(items, imports, _root_schema)?;
                    Ok((format!("Vec<{}>", item_type), true))
                } else {
                    imports.insert("use serde_json::Value;".to_string());
                    Ok(("Vec<Value>".to_string(), false))
                }
            }
            "object" => {
                imports.insert("use serde_json::Value;".to_string());
                Ok(("Value".to_string(), false))
            }
            _ => Ok(("String".to_string(), true)),
        }
    } else {
        Ok(("String".to_string(), true))
    }
}

/// 处理 $ref 类型引用
fn handle_ref_type(
    ref_path: &str,
    imports: &mut HashSet<String>,
) -> Result<(String, bool), Box<dyn std::error::Error>> {
    if ref_path.starts_with("#/definitions/") {
        let type_name = ref_path.replace("#/definitions/", "");

        // Map OCPP types to their Rust equivalents with special handling for known types
        let (rust_type, needs_validation) = match type_name.as_str() {
            // Special cases that need specific handling
            "DERControlStatusEnumType" => {
                imports.insert(
                    "use crate::v2_1::enumerations::der_control::DERControlStatusEnumType;"
                        .to_string(),
                );
                (type_name.clone(), false)
            }
            "EventDataType" => {
                // EventDataType 可能不存在，使用 Value 作为替代
                imports.insert("use serde_json::Value;".to_string());
                ("Value".to_string(), false)
            }
            "AuthorizationData" => {
                imports.insert("use crate::v2_1::datatypes::AuthorizationData;".to_string());
                (type_name.clone(), true)
            }
            // 常见的数据类型
            "CustomDataType"
            | "StatusInfoType"
            | "IdTokenType"
            | "IdTokenInfoType"
            | "EVSEType"
            | "TariffType"
            | "OCSPRequestDataType" => {
                imports.insert(format!("use crate::v2_1::datatypes::{};", type_name));
                (type_name.clone(), true)
            }
            // 常见的枚举类型
            "GenericStatusEnumType"
            | "AuthorizeCertificateStatusEnumType"
            | "EnergyTransferModeEnumType"
            | "ResetEnumType"
            | "ResetStatusEnumType"
            | "MessageTriggerEnumType"
            | "TriggerMessageStatusEnumType" => {
                imports.insert(format!("use crate::v2_1::enumerations::{};", type_name));
                (type_name.clone(), false)
            }
            _ => {
                // For other types, try to determine if it's an enum or datatype
                if type_name.ends_with("EnumType") {
                    imports.insert(format!("use crate::v2_1::enumerations::{};", type_name));
                    (type_name.clone(), false) // 枚举类型不需要 nested 验证
                } else if type_name.ends_with("Type") {
                    imports.insert(format!("use crate::v2_1::datatypes::{};", type_name));
                    (type_name.clone(), true) // 数据类型需要 nested 验证
                } else {
                    (type_name.clone(), true)
                }
            }
        };

        Ok((rust_type.to_string(), needs_validation))
    } else {
        Ok(("String".to_string(), true))
    }
}

/// 从原始JSON内容中提取properties字段的顺序
fn extract_field_order_from_content(
    content: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut field_order = Vec::new();

    // 使用正则表达式方法来查找最后一个properties对象中的字段
    // 首先找到所有"properties"的位置
    let properties_positions: Vec<_> = content.match_indices("\"properties\"").collect();

    if let Some((last_pos, _)) = properties_positions.last() {
        // 从最后一个"properties"开始查找
        let content_from_properties = &content[*last_pos..];

        // 查找 "properties": { 的开始
        if let Some(brace_start) = content_from_properties.find('{') {
            let mut current_pos = brace_start + 1; // 跳过开始的 {
            let chars: Vec<char> = content_from_properties.chars().collect();
            let mut brace_count = 1; // 已经进入了一个大括号

            while current_pos < chars.len() && brace_count > 0 {
                match chars[current_pos] {
                    '{' => brace_count += 1,
                    '}' => brace_count -= 1,
                    '"' if brace_count == 1 => {
                        // 在properties对象的第一层找到字符串
                        current_pos += 1; // 跳过开始引号
                        let mut field_name = String::new();

                        // 收集字段名
                        while current_pos < chars.len() && chars[current_pos] != '"' {
                            field_name.push(chars[current_pos]);
                            current_pos += 1;
                        }

                        if current_pos < chars.len() && chars[current_pos] == '"' {
                            current_pos += 1; // 跳过结束引号

                            // 跳过空白字符
                            while current_pos < chars.len() && chars[current_pos].is_whitespace() {
                                current_pos += 1;
                            }

                            // 检查是否是字段定义（有冒号）
                            if current_pos < chars.len() && chars[current_pos] == ':' {
                                field_order.push(field_name);
                            }
                        }
                    }
                    _ => {}
                }
                current_pos += 1;
            }
        }
    }

    Ok(field_order)
}
