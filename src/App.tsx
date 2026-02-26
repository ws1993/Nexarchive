import { useEffect, useMemo, useState } from "react";
import {
  Alert,
  App as AntApp,
  Button,
  Card,
  Col,
  Layout,
  Menu,
  Row,
  Space,
  Typography
} from "antd";
import {
  DashboardOutlined,
  FileSearchOutlined,
  FolderOpenOutlined,
  PlayCircleOutlined,
  SettingOutlined
} from "@ant-design/icons";
import { useAppStore } from "./store";
import { DashboardPage } from "./pages/DashboardPage";
import { InitWizardPage } from "./pages/InitWizardPage";
import { JobsPage } from "./pages/JobsPage";
import { RulesPage } from "./pages/RulesPage";
import { SettingsPage } from "./pages/SettingsPage";

const { Header, Sider, Content } = Layout;

const menuItems = [
  { key: "dashboard", label: "总览", icon: <DashboardOutlined /> },
  { key: "init", label: "初始化", icon: <FolderOpenOutlined /> },
  { key: "jobs", label: "任务日志", icon: <FileSearchOutlined /> },
  { key: "rules", label: "规则预览", icon: <PlayCircleOutlined /> },
  { key: "settings", label: "设置", icon: <SettingOutlined /> }
];

function App() {
  const { message } = AntApp.useApp();
  const [activeKey, setActiveKey] = useState("dashboard");
  const [collapsed, setCollapsed] = useState(false);

  const loadingConfig = useAppStore((s) => s.loadingConfig);
  const refreshConfig = useAppStore((s) => s.refreshConfig);
  const runJobNow = useAppStore((s) => s.runJobNow);

  useEffect(() => {
    void refreshConfig();
  }, [refreshConfig]);

  const title = useMemo(() => {
    const item = menuItems.find((v) => v.key === activeKey);
    return item?.label ?? "NexArchive";
  }, [activeKey]);

  const renderPage = () => {
    if (activeKey === "init") return <InitWizardPage />;
    if (activeKey === "jobs") return <JobsPage />;
    if (activeKey === "rules") return <RulesPage />;
    if (activeKey === "settings") return <SettingsPage />;
    return <DashboardPage />;
  };

  const onRunNow = async () => {
    const jobId = await runJobNow();
    message.success(`任务已触发: ${jobId}`);
    setActiveKey("jobs");
  };

  return (
    <Layout style={{ height: "100vh", overflow: "hidden" }}>
      <Sider collapsible collapsed={collapsed} onCollapse={setCollapsed} width={210}>
        <div
          style={{
            height: 60,
            display: "grid",
            placeItems: "center",
            color: "#fff",
            fontWeight: 700,
            letterSpacing: 1
          }}
        >
          {collapsed ? "NA" : "NexArchive"}
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[activeKey]}
          items={menuItems}
          onClick={(e) => setActiveKey(e.key)}
        />
      </Sider>
      <Layout style={{ overflow: "auto" }}>
        <Header style={{ background: "transparent", padding: "12px 16px", height: "auto" }}>
          <Card>
            <Row justify="space-between" align="middle" gutter={[12, 12]}>
              <Col>
                <Typography.Title level={4} style={{ margin: 0 }}>
                  {title}
                </Typography.Title>
                <Typography.Text type="secondary">
                  Windows 本地文件自动归档系统
                </Typography.Text>
              </Col>
              <Col>
                <Space>
                  <Button type="primary" onClick={onRunNow} loading={loadingConfig}>
                    立即执行一次
                  </Button>
                </Space>
              </Col>
            </Row>
          </Card>
          <Alert
            style={{ marginTop: 10 }}
            type="info"
            showIcon
            message="默认策略: 低置信度文件进入 _Review，处理后文件进入应用回收区，可通过任务日志恢复。"
          />
        </Header>
        <Content className="page-wrap">{renderPage()}</Content>
      </Layout>
    </Layout>
  );
}

export default function AppWrapper() {
  return (
    <AntApp>
      <App />
    </AntApp>
  );
}
