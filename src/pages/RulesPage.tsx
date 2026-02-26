import { Card, Col, Divider, Row, Table, Tag, Typography } from "antd";

const folderRows = [
  ["10", "10_身份基石", "身份与法律等长期关键文件"],
  ["20", "20_责任领域", "持续维护的生活/工作领域"],
  ["30", "30_行动项目", "有目标和截止时间的事项"],
  ["40", "40_知识金库", "学习资料、知识沉淀与模板"],
  ["50", "50_数字资产", "媒体、创作与软件资源"],
  ["99", "99_历史档案", "完成或失效后的封存内容"]
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
    <Row gutter={[16, 16]}>
      <Col xs={24} xl={12}>
        <Card className="section-card" title="命名规则">
          <Typography.Paragraph code>
            YYYYMMDD_文档类型_核心标题_版本号#标签@人物&备注.扩展名
          </Typography.Paragraph>
          <Divider />
          <Typography.Paragraph>
            必填字段: <Tag>YYYYMMDD</Tag>
            <Tag>文档类型</Tag>
            <Tag>核心标题</Tag>
          </Typography.Paragraph>
          <Typography.Paragraph>
            可选字段: <Tag>_v版本号</Tag>
            <Tag>#标签</Tag>
            <Tag>@人物</Tag>
            <Tag>&备注</Tag>
          </Typography.Paragraph>
        </Card>
      </Col>

      <Col xs={24} xl={12}>
        <Card className="section-card" title="目录结构概览">
          <Typography.Paragraph type="secondary">
            Inbox 为独立入口目录（由设置中的 Inbox 路径决定），不属于归档根目录层级。
          </Typography.Paragraph>
          <Table
            size="small"
            pagination={false}
            dataSource={folderRows.map((v) => ({ key: v[0], code: v[0], folder: v[1], desc: v[2] }))}
            columns={[
              { title: "编号", dataIndex: "code", width: 70 },
              { title: "目录", dataIndex: "folder", width: 120 },
              { title: "说明", dataIndex: "desc" }
            ]}
          />
        </Card>
      </Col>

      <Col span={24}>
        <Card className="section-card" title="控制词表">
          <Table
            size="small"
            pagination={false}
            dataSource={vocabRows.map((v, idx) => ({ key: idx, category: v[0], values: v[1] }))}
            columns={[
              { title: "类别", dataIndex: "category", width: 150 },
              { title: "词项", dataIndex: "values" }
            ]}
          />
        </Card>
      </Col>
    </Row>
  );
}
