use std::collections::HashSet;

/// 表示结构体字段的信息
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub original_name: String, // 原始 JSON 字段名
    pub rust_type: String,
    pub is_optional: bool,
    pub needs_validation: bool,
    pub description: Option<String>,
    pub max_length: Option<u32>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
}

/// 表示一个结构体的信息
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub imports: HashSet<String>,
}

/// 表示一对 Request/Response 消息
#[derive(Debug, Clone)]
pub struct MessagePair {
    pub base_name: String,
    pub request: Option<StructInfo>,
    pub response: Option<StructInfo>,
    pub combined_imports: HashSet<String>,
}

impl MessagePair {
    pub fn new(base_name: String) -> Self {
        Self {
            base_name,
            request: None,
            response: None,
            combined_imports: HashSet::new(),
        }
    }

    pub fn add_request(&mut self, struct_info: StructInfo) {
        self.combined_imports.extend(struct_info.imports.clone());
        self.request = Some(struct_info);
    }

    pub fn add_response(&mut self, struct_info: StructInfo) {
        self.combined_imports.extend(struct_info.imports.clone());
        self.response = Some(struct_info);
    }

    pub fn is_complete(&self) -> bool {
        self.request.is_some() && self.response.is_some()
    }
}
