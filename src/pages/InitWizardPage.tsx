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
      message.success("初始化完成");
    } finally {
      setWorking(false);
    }
  };

  return (
    <Row gutter={[16, 16]}>
      <Col xs={24} xl={10}>
        <Card className="section-card" title="初始化向导">
          <Alert
            showIcon
            type="info"
            message="首次使用请先配置路径并执行初始化。Inbox 使用你单独配置的路径，不会在归档根目录重复创建 00_收件箱。"
          />
          <Divider />
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
            <Button type="primary" onClick={onInit} loading={working}>
              一键初始化目录结构
            </Button>
          </Form>
        </Card>
      </Col>

      <Col xs={24} xl={14}>
        <Card className="section-card" title="结构预览">
          <Typography.Text type="secondary">
            结构来源于 reference.md 的附录规则。
          </Typography.Text>
          <Divider />
          <Tree
            defaultExpandAll
            treeData={preview.map((v, idx) => toTreeNode(v, `${idx}_`))}
          />
        </Card>
      </Col>
    </Row>
  );
}
