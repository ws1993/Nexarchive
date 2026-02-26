import { useEffect, useMemo, useState } from "react";
import {
  App as AntApp,
  Button,
  Card,
  Col,
  Form,
  Input,
  InputNumber,
  Row,
  Space,
  Switch
} from "antd";
import { api } from "../api";
import { useAppStore } from "../store";

export function SettingsPage() {
  const { message } = AntApp.useApp();
  const config = useAppStore((s) => s.config);
  const setConfig = useAppStore((s) => s.setConfig);
  const saveConfig = useAppStore((s) => s.saveConfig);
  const savingConfig = useAppStore((s) => s.savingConfig);

  const [apiKey, setApiKey] = useState("");
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    setApiKey(config.llm.api_key_encrypted);
  }, [config.llm.api_key_encrypted]);

  const canTest = useMemo(() => {
    return Boolean(config.llm.base_uri && config.llm.model && apiKey);
  }, [config.llm.base_uri, config.llm.model, apiKey]);

  const onSave = async () => {
    setConfig({
      ...config,
      llm: { ...config.llm, api_key_encrypted: apiKey }
    });
    const ok = await saveConfig();
    if (ok) message.success("设置已保存");
  };

  const onTest = async () => {
    setConfig({
      ...config,
      llm: { ...config.llm, api_key_encrypted: apiKey }
    });
    setTesting(true);
    try {
      await api.testLlmConnection();
      message.success("模型连通性测试成功");
    } finally {
      setTesting(false);
    }
  };

  return (
    <Row gutter={[16, 16]}>
      <Col xs={24} xl={12}>
        <Card className="section-card" title="系统设置">
          <Form layout="vertical">
            <Form.Item label="调度周期（小时）">
              <InputNumber
                min={1}
                max={168}
                value={config.schedule_hours}
                onChange={(v) =>
                  setConfig({ ...config, schedule_hours: Number(v ?? config.schedule_hours) })
                }
              />
            </Form.Item>
            <Form.Item label="后台运行">
              <Switch
                checked={config.run_in_background}
                onChange={(checked) => setConfig({ ...config, run_in_background: checked })}
              />
            </Form.Item>
            <Form.Item label="开机自启动">
              <Switch
                checked={config.autostart}
                onChange={(checked) => setConfig({ ...config, autostart: checked })}
              />
            </Form.Item>
          </Form>
        </Card>
      </Col>

      <Col xs={24} xl={12}>
        <Card className="section-card" title="大模型设置">
          <Form layout="vertical">
            <Form.Item label="Base URI">
              <Input
                value={config.llm.base_uri}
                onChange={(e) =>
                  setConfig({ ...config, llm: { ...config.llm, base_uri: e.target.value } })
                }
              />
            </Form.Item>
            <Form.Item label="Model">
              <Input
                value={config.llm.model}
                onChange={(e) =>
                  setConfig({ ...config, llm: { ...config.llm, model: e.target.value } })
                }
              />
            </Form.Item>
            <Form.Item label="API Key">
              <Input.Password value={apiKey} onChange={(e) => setApiKey(e.target.value)} />
            </Form.Item>
            <Form.Item label="超时（秒）">
              <InputNumber
                min={5}
                max={180}
                value={config.llm.timeout_sec}
                onChange={(v) =>
                  setConfig({ ...config, llm: { ...config.llm, timeout_sec: Number(v ?? 30) } })
                }
              />
            </Form.Item>
          </Form>
        </Card>
      </Col>

      <Col span={24}>
        <Card className="section-card" title="日志与保留策略">
          <Row gutter={[12, 12]}>
            <Col xs={24} md={8}>
              <Form layout="vertical">
                <Form.Item label="单个日志文件大小（MB）">
                  <InputNumber
                    min={1}
                    max={100}
                    value={config.retention.max_log_file_mb}
                    onChange={(v) =>
                      setConfig({
                        ...config,
                        retention: {
                          ...config.retention,
                          max_log_file_mb: Number(v ?? config.retention.max_log_file_mb)
                        }
                      })
                    }
                  />
                </Form.Item>
              </Form>
            </Col>
            <Col xs={24} md={8}>
              <Form layout="vertical">
                <Form.Item label="日志文件数量">
                  <InputNumber
                    min={1}
                    max={20}
                    value={config.retention.max_log_files}
                    onChange={(v) =>
                      setConfig({
                        ...config,
                        retention: {
                          ...config.retention,
                          max_log_files: Number(v ?? config.retention.max_log_files)
                        }
                      })
                    }
                  />
                </Form.Item>
              </Form>
            </Col>
            <Col xs={24} md={8}>
              <Form layout="vertical">
                <Form.Item label="DB 日志保留天数">
                  <InputNumber
                    min={1}
                    max={365}
                    value={config.retention.db_log_retention_days}
                    onChange={(v) =>
                      setConfig({
                        ...config,
                        retention: {
                          ...config.retention,
                          db_log_retention_days: Number(v ?? config.retention.db_log_retention_days)
                        }
                      })
                    }
                  />
                </Form.Item>
              </Form>
            </Col>
          </Row>

          <Space>
            <Button type="primary" loading={savingConfig} onClick={onSave}>
              保存设置
            </Button>
            <Button disabled={!canTest} loading={testing} onClick={onTest}>
              测试模型连通性
            </Button>
          </Space>
        </Card>
      </Col>
    </Row>
  );
}
