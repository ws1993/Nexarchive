import { useEffect, useMemo, useRef, useState } from "react";
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
import {
  checkForUpdate,
  disposeUpdate,
  downloadAndInstallUpdate,
  formatUpdaterError,
  summarizeUpdate
} from "./updater";

const { Header, Sider, Content } = Layout;

const menuItems = [
  { key: "dashboard", label: "总览", icon: <DashboardOutlined /> },
  { key: "init", label: "初始化", icon: <FolderOpenOutlined /> },
  { key: "jobs", label: "任务日志", icon: <FileSearchOutlined /> },
  { key: "rules", label: "规则预览", icon: <PlayCircleOutlined /> },
  { key: "settings", label: "设置", icon: <SettingOutlined /> }
];

function App() {
  const { message, modal } = AntApp.useApp();
  const [activeKey, setActiveKey] = useState("dashboard");
  const [collapsed, setCollapsed] = useState(false);
  const [configReady, setConfigReady] = useState(false);
  const autoUpdateCheckedRef = useRef(false);

  const config = useAppStore((s) => s.config);
  const loadingConfig = useAppStore((s) => s.loadingConfig);
  const refreshConfig = useAppStore((s) => s.refreshConfig);
  const runJobNow = useAppStore((s) => s.runJobNow);

  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        await refreshConfig();
      } finally {
        if (active) setConfigReady(true);
      }
    })();

    return () => {
      active = false;
    };
  }, [refreshConfig]);

  useEffect(() => {
    if (!configReady || autoUpdateCheckedRef.current) return;
    autoUpdateCheckedRef.current = true;

    if (!config.updater.auto_check_on_startup) return;

    void (async () => {
      try {
        const update = await checkForUpdate({
          proxyEnabled: config.updater.proxy_enabled,
          proxyUrl: config.updater.proxy_url_encrypted
        });

        if (!update) return;

        const summary = summarizeUpdate(update);
        const notes = summary.body?.trim() || "本次版本未提供更新说明。";

        modal.confirm({
          title: `发现新版本 ${summary.version}`,
          width: 680,
          okText: "立即更新",
          cancelText: "稍后再说",
          content: (
            <Space direction="vertical" size="small" style={{ width: "100%" }}>
              <Typography.Text type="secondary">
                当前版本 {summary.currentVersion}，发现可用更新 {summary.version}
              </Typography.Text>
              <Typography.Text strong>更新说明</Typography.Text>
              <div
                style={{
                  maxHeight: 220,
                  overflow: "auto",
                  whiteSpace: "pre-wrap",
                  background: "#f6f8fb",
                  borderRadius: 8,
                  padding: 12
                }}
              >
                {notes}
              </div>
            </Space>
          ),
          onOk: async () => {
            try {
              message.open({
                key: "startup-update",
                type: "loading",
                content: "正在下载并安装更新，请稍候...",
                duration: 0
              });
              await downloadAndInstallUpdate(update);
            } catch (error) {
              message.error(formatUpdaterError(error));
            } finally {
              await disposeUpdate(update);
              message.destroy("startup-update");
            }
          },
          onCancel: () => {
            void disposeUpdate(update);
          }
        });
      } catch (error) {
        console.warn("startup update check skipped:", error);
      }
    })();
  }, [
    configReady,
    config.updater.auto_check_on_startup,
    config.updater.proxy_enabled,
    config.updater.proxy_url_encrypted,
    message,
    modal
  ]);

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
