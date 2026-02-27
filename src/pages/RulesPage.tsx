import { Card, Col, Row, Space, Table, Tag, Typography } from "antd";

interface FolderTreeRow {
  key: string;
  code: string;
  folder: string;
  desc: string;
  children?: FolderTreeRow[];
}

const folderTreeRows: FolderTreeRow[] = [
  {
    key: "10",
    code: "10",
    folder: "10_身份基石",
    desc: "身份与法律等长期关键文件",
    children: [
      { key: "11", code: "11", folder: "11_法律证件", desc: "证件与法律身份证明材料" },
      { key: "12", code: "12", folder: "12_教育背景", desc: "学历、证书与教育履历" },
      { key: "13", code: "13", folder: "13_职业履历", desc: "简历、合同与职业发展记录" },
      { key: "14", code: "14", folder: "14_健康档案", desc: "病历、体检和医疗相关资料" },
      { key: "15", code: "15", folder: "15_财务信用", desc: "信用、资产与金融证明信息" },
      { key: "16", code: "16", folder: "16_社会关系", desc: "家庭关系与社会关系资料" }
    ]
  },
  {
    key: "20",
    code: "20",
    folder: "20_责任领域",
    desc: "持续维护的生活/工作领域",
    children: [
      { key: "21", code: "21", folder: "21_财务管理", desc: "预算、账单与报税记录" },
      { key: "22", code: "22", folder: "22_健康管理", desc: "健康计划、诊疗与跟踪记录" },
      { key: "23", code: "23", folder: "23_居住管理", desc: "住房、物业与生活缴费资料" },
      { key: "24", code: "24", folder: "24_职业发展", desc: "工作成长与年度能力建设材料" }
    ]
  },
  {
    key: "30",
    code: "30",
    folder: "30_行动项目",
    desc: "有目标和截止时间的事项",
    children: [
      { key: "31", code: "31", folder: "31_工作项目", desc: "正在推进的工作类项目" },
      { key: "32", code: "32", folder: "32_个人项目", desc: "个人计划、兴趣或副业项目" }
    ]
  },
  {
    key: "40",
    code: "40",
    folder: "40_知识金库",
    desc: "学习资料、知识沉淀与模板",
    children: [
      { key: "41", code: "41", folder: "41_知识库", desc: "结构化知识与长期沉淀内容" },
      { key: "42", code: "42", folder: "42_资料库", desc: "参考资料、输入素材与研读材料" },
      { key: "43", code: "43", folder: "43_模板", desc: "可复用模板、规范与表单" }
    ]
  },
  {
    key: "50",
    code: "50",
    folder: "50_数字资产",
    desc: "媒体、创作与软件资源",
    children: [
      { key: "51", code: "51", folder: "51_媒体素材", desc: "图片、音视频等原始素材" },
      { key: "52", code: "52", folder: "52_创作产出", desc: "成品稿件与创作结果文件" },
      { key: "53", code: "53", folder: "53_软件资源", desc: "安装包、脚本与工具资源" }
    ]
  },
  { key: "99", code: "99", folder: "99_历史档案", desc: "完成或失效后的封存内容" }
];

const vocabRows = [
  ["凭证与财务", "发票、小票、账单、保单、回执"],
  ["法务与身份", "证件、合同、协议、证书、公文"],
  ["思考与产出", "方案、纪要、报告、文稿、笔记"],
  ["资料与输入", "教程、研报、书籍、说明书、素材"],
  ["生活与事务", "门票、行程单、清单、病历"]
];

export function RulesPage() {
  return (
    <Space direction="vertical" size="middle" style={{ width: "100%", paddingBottom: 24 }}>
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card className="section-card" title="命名规则" style={{ height: "100%" }}>
            <div style={{ background: '#fafafa', padding: '12px 16px', borderRadius: 8, marginBottom: 16, border: '1px solid #f0f0f0' }}>
              <Typography.Text code style={{ fontSize: 16 }}>
                YYYYMMDD_文档类型_核心标题_版本号#标签@人物&备注.扩展名
              </Typography.Text>
            </div>

            <Row gutter={[24, 16]}>
              <Col xs={24} sm={12}>
                <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>必填字段</Typography.Text>
                <div>
                  <Tag color="cyan" style={{ marginBottom: 8 }}>YYYYMMDD</Tag>
                  <Tag color="blue" style={{ marginBottom: 8 }}>文档类型</Tag>
                  <Tag color="geekblue" style={{ marginBottom: 8 }}>核心标题</Tag>
                </div>
              </Col>
              <Col xs={24} sm={12}>
                <Typography.Text strong style={{ display: 'block', marginBottom: 8 }}>可选字段</Typography.Text>
                <div>
                  <Tag style={{ marginBottom: 8 }}>_v版本号</Tag>
                  <Tag style={{ marginBottom: 8 }}>#标签</Tag>
                  <Tag style={{ marginBottom: 8 }}>@人物</Tag>
                  <Tag style={{ marginBottom: 8 }}>&备注</Tag>
                </div>
              </Col>
            </Row>
          </Card>
        </Col>

        <Col xs={24} lg={12}>
          <Card className="section-card" title="目录结构概览" style={{ height: "100%" }}>
            <Typography.Paragraph type="secondary" style={{ marginBottom: 16 }}>
              Inbox 为独立入口目录（由设置中的 Inbox 路径决定），不属于归档根目录层级。
            </Typography.Paragraph>
            <Table
              size="middle"
              pagination={false}
              rowKey="key"
              dataSource={folderTreeRows}
              defaultExpandAllRows
              expandable={{ childrenColumnName: "children", indentSize: 16, expandIconColumnIndex: 0 }}
              columns={[
                { title: "", key: "expand", width: 40 },
                {
                  title: "编号",
                  dataIndex: "code",
                  width: 70,
                  render: (text: string, row: FolderTreeRow) => (
                    <Tag color={row.children || row.code === "99" ? "blue" : "cyan"}>{text}</Tag>
                  )
                },
                {
                  title: "目录",
                  dataIndex: "folder",
                  width: 160,
                  render: (text: string, row: FolderTreeRow) => (
                    <Typography.Text strong style={row.children || row.code === "99" ? {} : { paddingLeft: 12 }}>
                      {text}
                    </Typography.Text>
                  )
                },
                { title: "说明", dataIndex: "desc" }
              ]}
              scroll={{ y: 300 }}
            />
          </Card>
        </Col>
      </Row>

      <Card className="section-card" title="控制词表">
        <Row gutter={[24, 24]}>
          {vocabRows.map((v, idx) => (
            <Col xs={24} sm={12} md={8} key={idx}>
              <Card type="inner" title={v[0]} size="small" style={{ height: '100%', background: '#fafafa' }}>
                <Typography.Text>{v[1]}</Typography.Text>
              </Card>
            </Col>
          ))}
        </Row>
      </Card>
    </Space>
  );
}
