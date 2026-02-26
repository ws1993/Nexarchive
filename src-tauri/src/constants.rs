use crate::models::InitPreviewItem;

pub const CONTROLLED_VOCAB: &[&str] = &[
    "发票",
    "小票",
    "账单",
    "保单",
    "回执",
    "证件",
    "合同",
    "协议",
    "证书",
    "公文",
    "方案",
    "纪要",
    "报告",
    "文稿",
    "笔记",
    "教程",
    "研报",
    "书籍",
    "说明书",
    "素材",
    "门票",
    "行程单",
    "清单",
    "病历",
];

pub const TOP_DIR_CODES: &[&str] = &["10", "20", "30", "40", "50", "99"];

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "txt", "md", "pdf", "doc", "docx", "ppt", "pptx", "xlsx", "jpg", "jpeg", "png",
];

pub fn top_dir_name(code: &str) -> Option<&'static str> {
    match code {
        "10" => Some("10_身份基石"),
        "20" => Some("20_责任领域"),
        "30" => Some("30_行动项目"),
        "40" => Some("40_知识金库"),
        "50" => Some("50_数字资产"),
        "99" => Some("99_历史档案"),
        _ => None,
    }
}

pub fn init_preview() -> Vec<InitPreviewItem> {
    vec![
        item(
            "10",
            "身份基石",
            vec![
                item("11", "法律证件", vec![]),
                item("12", "教育背景", vec![]),
                item("13", "职业履历", vec![]),
                item("14", "健康档案", vec![]),
                item("15", "财务信用", vec![]),
                item("16", "社会关系", vec![]),
            ],
        ),
        item(
            "20",
            "责任领域",
            vec![
                item("21", "财务管理", vec![]),
                item("22", "健康管理", vec![]),
                item("23", "居住管理", vec![]),
                item("24", "职业发展", vec![]),
            ],
        ),
        item(
            "30",
            "行动项目",
            vec![item("31", "工作项目", vec![]), item("32", "个人项目", vec![])],
        ),
        item(
            "40",
            "知识金库",
            vec![
                item("41", "知识库", vec![]),
                item("42", "资料库", vec![]),
                item("43", "模板", vec![]),
            ],
        ),
        item(
            "50",
            "数字资产",
            vec![
                item("51", "媒体素材", vec![]),
                item("52", "创作产出", vec![]),
                item("53", "软件资源", vec![]),
            ],
        ),
        item("99", "历史档案", vec![]),
    ]
}

fn item(code: &str, folder: &str, children: Vec<InitPreviewItem>) -> InitPreviewItem {
    InitPreviewItem {
        code: code.to_string(),
        folder: folder.to_string(),
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
    }
}
