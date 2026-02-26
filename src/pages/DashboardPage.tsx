import { Card, Col, Descriptions, Row, Space, Statistic, Tag, Typography } from "antd";
import { useMemo } from "react";
import { useAppStore } from "../store";

export function DashboardPage() {
  const config = useAppStore((s) => s.config);

  const scheduleText = useMemo(() => {
    if (!config.schedule_hours || config.schedule_hours <= 0) return "未启用";
    if (config.schedule_hours % 24 === 0) {
      return `${config.schedule_hours / 24} 天/次`;
    }
    return `${config.schedule_hours} 小时/次`;
  }, [config.schedule_hours]);

  return (
    <Space direction="vertical" size="middle" style={{ width: "100%", paddingBottom: 24 }}>
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card className="section-card" title="运行概览" style={{ height: "100%" }}>
            <div className="kpi-grid">
              <div className="kpi-item">
                <div className="kpi-label">调度周期</div>
                <div className="kpi-value">{scheduleText}</div>
              </div>
              <div className="kpi-item">
                <div className="kpi-label">后台运行</div>
                <div className="kpi-value">{config.run_in_background ? "开启" : "关闭"}</div>
              </div>
              <div className="kpi-item">
                <div className="kpi-label">开机自启</div>
                <div className="kpi-value">{config.autostart ? "开启" : "关闭"}</div>
              </div>
              <div className="kpi-item">
                <div className="kpi-label">模型</div>
                <div className="kpi-value" style={{ fontSize: 18 }}>
                  {config.llm.model || "未配置"}
                </div>
              </div>
            </div>
          </Card>
        </Col>

        <Col xs={24} lg={12}>
          <Card className="section-card" title="当前配置" style={{ height: "100%" }}>
            <Descriptions column={1} size="small" bordered>
              <Descriptions.Item label="Inbox">{config.inbox_path || "未配置"}</Descriptions.Item>
              <Descriptions.Item label="Archive Root">
                {config.archive_root_path || "未配置"}
              </Descriptions.Item>
              <Descriptions.Item label="Base URI">
                {config.llm.base_uri || "未配置"}
              </Descriptions.Item>
              <Descriptions.Item label="日志策略">
                <Tag color="blue">{config.retention.max_log_file_mb}MB</Tag>
                <Tag color="cyan">{config.retention.max_log_files} files</Tag>
                <Tag color="geekblue">db {config.retention.max_db_logs}</Tag>
              </Descriptions.Item>
            </Descriptions>
          </Card>
        </Col>
      </Row>

      <Card className="section-card" title="目录策略提示">
        <Row gutter={[24, 24]} align="middle">
          <Col xs={24} md={16}>
            <div style={{ display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: 8 }}>
              <Typography.Text strong style={{ marginRight: 8 }}>顶层目录前缀:</Typography.Text>
              <Tag color="purple">00</Tag>
              <Tag color="magenta">10</Tag>
              <Tag color="red">20</Tag>
              <Tag color="volcano">30</Tag>
              <Tag color="orange">40</Tag>
              <Tag color="gold">50</Tag>
              <Tag color="lime">99</Tag>
            </div>
          </Col>
          <Col xs={24} md={8}>
            <Statistic
              title="低置信度阈值"
              value={0.7}
              precision={2}
              suffix="confidence"
              valueStyle={{ color: '#cf1322' }}
            />
          </Col>
        </Row>
      </Card>
    </Space>
  );
}
