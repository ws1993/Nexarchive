import { Card, Col, Divider, Row, Space, Table, Tag, Typography } from "antd";

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
              dataSource={folderRows.map((v) => ({ key: v[0], code: v[0], folder: v[1], desc: v[2] }))}
              columns={[
                { title: "编号", dataIndex: "code", width: 70, render: (text) => <Tag color="blue">{text}</Tag> },
                { title: "目录", dataIndex: "folder", width: 140, render: (text) => <Typography.Text strong>{text}</Typography.Text> },
                { title: "说明", dataIndex: "desc" }
              ]}
              scroll={{ y: 240 }}
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
