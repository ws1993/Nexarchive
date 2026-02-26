import { Card, Col, Descriptions, Row, Statistic, Tag, Typography } from "antd";
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
    <div className="split">
      <Card className="section-card" title="运行概览">
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

      <Card className="section-card" title="当前配置">
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

      <Card className="section-card" title="目录策略提示">
        <Row gutter={[12, 12]}>
          <Col span={24}>
            <Typography.Text>
              顶层目录以数字前缀组织: <Tag>00</Tag>
              <Tag>10</Tag>
              <Tag>20</Tag>
              <Tag>30</Tag>
              <Tag>40</Tag>
              <Tag>50</Tag>
              <Tag>99</Tag>
            </Typography.Text>
          </Col>
          <Col span={24}>
            <Statistic
              title="低置信度阈值"
              value={0.7}
              precision={2}
              suffix="confidence"
            />
          </Col>
        </Row>
      </Card>
    </div>
  );
}
