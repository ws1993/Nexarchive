import { Card, Col, Row, Space, Table, Tag, Typography } from "antd";
import { folderTreeData } from "../data/folderTree";
import type { InitPreviewItem } from "../types";

const descMap: Record<string, string> = {
  "10": "身份与法律等长期关键文件",
  "11": "证件与法律身份证明材料",
  "12": "学历、证书与教育履历",
  "13": "简历、合同与职业发展记录",
  "14": "病历、体检和医疗相关资料",
  "15": "信用、资产与金融证明信息",
  "16": "家庭关系与社会关系资料",
  "20": "持续维护的生活/工作领域",
  "21": "预算、账单与报税记录",
  "22": "健康计划、诊疗与跟踪记录",
  "23": "住房、物业与生活缴费资料",
  "24": "工作成长与年度能力建设材料",
  "30": "有目标和截止时间的事项",
  "31": "正在推进的工作类项目",
  "32": "个人计划、兴趣或副业项目",
  "40": "学习资料、知识沉淀与模板",
  "41": "结构化知识与长期沉淀内容",
  "42": "参考资料、输入素材与研读材料",
  "43": "可复用模板、规范与表单",
  "50": "媒体、创作与软件资源",
  "51": "图片、音视频等原始素材",
  "52": "成品稿件与创作结果文件",
  "53": "安装包、脚本与工具资源",
  "99": "完成或失效后的封存内容"
};

interface FolderTreeRow {
  key: string;
  code: string;
  folder: string;
  desc: string;
  children?: FolderTreeRow[];
}

function toTableRow(item: InitPreviewItem): FolderTreeRow {
  return {
    key: item.code,
    code: item.code,
    folder: item.folder,
    desc: descMap[item.code] ?? "",
    children: item.children?.map(toTableRow)
  };
}

const folderTreeRows = folderTreeData.map(toTableRow);

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
