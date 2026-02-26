import { useEffect, useState } from "react";
import {
  Alert,
  App as AntApp,
  Button,
  Card,
  Col,
  Divider,
  Form,
  Input,
  Row,
  Space,
  Tree,
  Typography
} from "antd";
import { api } from "../api";
import { useAppStore } from "../store";
import type { InitPreviewItem } from "../types";

function toTreeNode(item: InitPreviewItem, keyPrefix = ""): any {
  const key = `${keyPrefix}${item.code}_${item.folder}`;
  return {
    key,
    title: `${item.code}_${item.folder}`,
    children: item.children?.map((v, idx) => toTreeNode(v, `${key}_${idx}_`))
  };
}

export function InitWizardPage() {
  const { message } = AntApp.useApp();
  const config = useAppStore((s) => s.config);
  const setConfig = useAppStore((s) => s.setConfig);
  const refreshConfig = useAppStore((s) => s.refreshConfig);

  const [preview, setPreview] = useState<InitPreviewItem[]>([]);
  const [working, setWorking] = useState(false);

  useEffect(() => {
    void api.getInitPreview().then(setPreview);
  }, []);

  const onInit = async () => {
    if (!config.inbox_path || !config.archive_root_path) {
      message.error("请先填写 Inbox 和 Archive Root 路径");
      return;
    }
    setWorking(true);
    try {
      await api.initSystem(config.inbox_path, config.archive_root_path);
      await refreshConfig();
      message.success("初始化完成");
    } catch (e) {
      message.error(`初始化失败：${e}`);
    } finally {
      setWorking(false);
    }
  };

  return (
    <Space direction="vertical" size="middle" style={{ width: "100%", paddingBottom: 24 }}>
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={10}>
          <Card className="section-card" title="初始化向导" style={{ height: "100%" }}>
            <Alert
              showIcon
              type="info"
              message="首次使用请先配置路径并执行初始化。Inbox 使用你单独配置的路径，不会在归档根目录重复创建 00_收件箱。"
              style={{ marginBottom: 24 }}
            />
            <Form layout="vertical">
              <Form.Item label="Inbox 路径">
                <Input
                  value={config.inbox_path}
                  placeholder="例如: E:\\收件箱"
                  onChange={(e) => setConfig({ ...config, inbox_path: e.target.value })}
                />
              </Form.Item>
              <Form.Item label="Archive Root 路径">
                <Input
                  value={config.archive_root_path}
                  placeholder="例如: E:\\个人档案库"
                  onChange={(e) => setConfig({ ...config, archive_root_path: e.target.value })}
                />
              </Form.Item>
              <Row justify="end" style={{ marginTop: 16 }}>
                <Button type="primary" onClick={onInit} loading={working} size="large">
                  一键初始化目录结构
                </Button>
              </Row>
            </Form>
          </Card>
        </Col>

        <Col xs={24} lg={14}>
          <Card className="section-card" title="结构预览" style={{ height: "100%" }}>
            <Typography.Text type="secondary" style={{ display: 'block', marginBottom: 16 }}>
              结构来源于 reference.md 的附录规则。
            </Typography.Text>
            <div style={{ maxHeight: 400, overflowY: 'auto', padding: '0 8px' }}>
              <Tree
                defaultExpandAll
                treeData={preview.map((v, idx) => toTreeNode(v, `${idx}_`))}
              />
            </div>
          </Card>
        </Col>
      </Row>
    </Space>
  );
}
