import { useEffect, useMemo, useState } from "react";
import {
  App as AntApp,
  Button,
  Card,
  Col,
  Divider,
  Form,
  Input,
  InputNumber,
  Progress,
  Row,
  Space,
  Switch,
  Typography
} from "antd";
import { getVersion } from "@tauri-apps/api/app";
import { api } from "../api";
import { useAppStore } from "../store";
import {
  checkForUpdate,
  disposeUpdate,
  downloadAndInstallUpdate,
  formatUpdaterError,
  resolveUpdaterProxy,
  summarizeUpdate,
  type UpdaterProgress
} from "../updater";

export function SettingsPage() {
  const { message, modal } = AntApp.useApp();
  const config = useAppStore((s) => s.config);
  const setConfig = useAppStore((s) => s.setConfig);
  const saveConfig = useAppStore((s) => s.saveConfig);
  const savingConfig = useAppStore((s) => s.savingConfig);

  const [llmApiKey, setLlmApiKey] = useState("");
  const [mineruToken, setMineruToken] = useState("");
  const [appVersion, setAppVersion] = useState("");
  const [testingLlm, setTestingLlm] = useState(false);
  const [testingMineru, setTestingMineru] = useState(false);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [installingUpdate, setInstallingUpdate] = useState(false);
  const [updateProgress, setUpdateProgress] = useState<UpdaterProgress | null>(null);

  useEffect(() => {
    setLlmApiKey(config.llm.api_key_encrypted);
    setMineruToken(config.mineru.api_token_encrypted);
  }, [config.llm.api_key_encrypted, config.mineru.api_token_encrypted]);

  useEffect(() => {
    void getVersion()
      .then((version) => setAppVersion(version))
      .catch(() => setAppVersion("unknown"));
  }, []);

  const canTestLlm = useMemo(() => {
    return Boolean(config.llm.base_uri && config.llm.model && llmApiKey);
  }, [config.llm.base_uri, config.llm.model, llmApiKey]);

  const canTestMineru = useMemo(() => {
    return Boolean(config.mineru.enabled && config.mineru.base_uri && mineruToken);
  }, [config.mineru.enabled, config.mineru.base_uri, mineruToken]);

  const hasValidProxy = useMemo(() => {
    try {
      resolveUpdaterProxy(
        config.updater.proxy_enabled,
        config.updater.proxy_url_encrypted
      );
      return true;
    } catch {
      return false;
    }
  }, [config.updater.proxy_enabled, config.updater.proxy_url_encrypted]);

  const onSave = async () => {
    try {
      resolveUpdaterProxy(
        config.updater.proxy_enabled,
        config.updater.proxy_url_encrypted
      );
    } catch (error) {
      message.error(formatUpdaterError(error));
      return;
    }

    setConfig({
      ...config,
      llm: { ...config.llm, api_key_encrypted: llmApiKey },
      mineru: { ...config.mineru, api_token_encrypted: mineruToken }
    });
    const ok = await saveConfig();
    if (ok) message.success("设置已保存");
  };

  const onTestLlm = async () => {
    const nextConfig = {
      ...config,
      llm: { ...config.llm, api_key_encrypted: llmApiKey },
      mineru: { ...config.mineru, api_token_encrypted: mineruToken }
    };
    setConfig(nextConfig);
    setTestingLlm(true);
    try {
      await api.saveSettings(nextConfig);
      await api.testLlmConnection();
      message.success("模型连通性测试成功");
    } catch (e) {
      message.error(`模型连通性测试失败：${e}`);
    } finally {
      setTestingLlm(false);
    }
  };

  const onTestMineru = async () => {
    const nextConfig = {
      ...config,
      mineru: { ...config.mineru, api_token_encrypted: mineruToken }
    };
    setConfig(nextConfig);
    setTestingMineru(true);
    try {
      await api.saveSettings(nextConfig);
      await api.testMineruConnection();
      message.success("MinerU 连通性测试成功");
    } catch (e) {
      message.error(`MinerU 连通性测试失败：${e}`);
    } finally {
      setTestingMineru(false);
    }
  };

  const onCheckUpdate = async () => {
    setCheckingUpdate(true);
    setUpdateProgress(null);

    let updateHandle: Awaited<ReturnType<typeof checkForUpdate>> = null;
    try {
      updateHandle = await checkForUpdate({
        proxyEnabled: config.updater.proxy_enabled,
        proxyUrl: config.updater.proxy_url_encrypted
      });

      if (!updateHandle) {
        message.success("当前已是最新版本");
        return;
      }

      const summary = summarizeUpdate(updateHandle);
      const notes = summary.body?.trim() || "本次版本未提供更新说明。";

      modal.confirm({
        title: `发现新版本 ${summary.version}`,
        width: 700,
        okText: "下载并安装",
        cancelText: "取消",
        content: (
          <Space direction="vertical" size="small" style={{ width: "100%" }}>
            <Typography.Text type="secondary">
              当前版本 {summary.currentVersion}，可更新到 {summary.version}
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
          setInstallingUpdate(true);
          try {
            await downloadAndInstallUpdate(updateHandle!, (progress) => {
              setUpdateProgress(progress);
            });
          } catch (error) {
            message.error(formatUpdaterError(error));
          } finally {
            setInstallingUpdate(false);
            await disposeUpdate(updateHandle);
          }
        },
        onCancel: () => {
          void disposeUpdate(updateHandle);
        }
      });
    } catch (error) {
      message.error(formatUpdaterError(error));
      await disposeUpdate(updateHandle);
    } finally {
      setCheckingUpdate(false);
    }
  };

  const progressPercent = useMemo(() => {
    if (!updateProgress || !updateProgress.totalBytes || updateProgress.totalBytes <= 0) {
      return undefined;
    }
    return Math.min(
      100,
      Math.round((updateProgress.downloadedBytes / updateProgress.totalBytes) * 100)
    );
  }, [updateProgress]);

  return (
    <Form layout="vertical" className="settings-form">
      <Space direction="vertical" size="middle" style={{ width: "100%", paddingBottom: 24 }}>
        
        {/* Base Settings */}
        <Card className="section-card" title="基础设置">
          <Row gutter={24}>
            <Col xs={24} md={8}>
              <Form.Item label="调度周期（小时）">
                <InputNumber
                  style={{ width: "100%" }}
                  min={1}
                  max={168}
                  value={config.schedule_hours}
                  onChange={(v) =>
                    setConfig({ ...config, schedule_hours: Number(v ?? config.schedule_hours) })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="后台运行">
                <Switch
                  checked={config.run_in_background}
                  onChange={(checked) => setConfig({ ...config, run_in_background: checked })}
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="开机自启动">
                <Switch
                  checked={config.autostart}
                  onChange={(checked) => setConfig({ ...config, autostart: checked })}
                />
              </Form.Item>
            </Col>
          </Row>
        </Card>

        {/* AI & Parsing Services */}
        <Card className="section-card" title="大模型与解析服务">
          <Divider orientation="left" style={{ marginTop: 0 }}>LLM 设置</Divider>
          <Row gutter={24}>
            <Col xs={24} md={12}>
              <Form.Item label="LLM Base URI">
                <Input
                  value={config.llm.base_uri}
                  onChange={(e) =>
                    setConfig({ ...config, llm: { ...config.llm, base_uri: e.target.value } })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={12}>
              <Form.Item label="LLM Model">
                <Input
                  value={config.llm.model}
                  onChange={(e) =>
                    setConfig({ ...config, llm: { ...config.llm, model: e.target.value } })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={12}>
              <Form.Item label="LLM API Key">
                <Input.Password value={llmApiKey} onChange={(e) => setLlmApiKey(e.target.value)} />
              </Form.Item>
            </Col>
            <Col xs={24} md={12}>
              <Form.Item label="LLM 超时（秒）">
                <InputNumber
                  style={{ width: "100%" }}
                  min={5}
                  max={180}
                  value={config.llm.timeout_sec}
                  onChange={(v) =>
                    setConfig({ ...config, llm: { ...config.llm, timeout_sec: Number(v ?? 30) } })
                  }
                />
              </Form.Item>
            </Col>
          </Row>

          <Divider orientation="left">MinerU 解析服务（可选，失败自动回退本地解析）</Divider>
          <Row gutter={24}>
            <Col xs={24} md={8}>
              <Form.Item label="启用 MinerU">
                <Switch
                  checked={config.mineru.enabled}
                  onChange={(checked) =>
                    setConfig({ ...config, mineru: { ...config.mineru, enabled: checked } })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="OCR 模式">
                <Switch
                  checked={config.mineru.is_ocr}
                  onChange={(checked) =>
                    setConfig({ ...config, mineru: { ...config.mineru, is_ocr: checked } })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="请求超时（秒）">
                <InputNumber
                  style={{ width: "100%" }}
                  min={5}
                  max={300}
                  value={config.mineru.timeout_sec}
                  onChange={(v) =>
                    setConfig({
                      ...config,
                      mineru: { ...config.mineru, timeout_sec: Number(v ?? 60) }
                    })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={12}>
              <Form.Item label="Base URI">
                <Input
                  value={config.mineru.base_uri}
                  onChange={(e) =>
                    setConfig({ ...config, mineru: { ...config.mineru, base_uri: e.target.value } })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={12}>
              <Form.Item label="Token">
                <Input.Password value={mineruToken} onChange={(e) => setMineruToken(e.target.value)} />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="Model Version">
                <Input
                  value={config.mineru.model_version}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      mineru: { ...config.mineru, model_version: e.target.value }
                    })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="Language">
                <Input
                  value={config.mineru.language}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      mineru: { ...config.mineru, language: e.target.value }
                    })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="任务最大等待（秒）">
                <InputNumber
                  style={{ width: "100%" }}
                  min={30}
                  max={1800}
                  value={config.mineru.max_wait_sec}
                  onChange={(v) =>
                    setConfig({
                      ...config,
                      mineru: { ...config.mineru, max_wait_sec: Number(v ?? 300) }
                    })
                  }
                />
              </Form.Item>
            </Col>
          </Row>
        </Card>

        {/* Logs Retention */}
        <Card className="section-card" title="日志与保留策略">
          <Row gutter={24}>
            <Col xs={24} md={8}>
              <Form.Item label="单个日志文件大小（MB）">
                <InputNumber
                  style={{ width: "100%" }}
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
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="日志文件数量">
                <InputNumber
                  style={{ width: "100%" }}
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
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="DB 日志保留天数">
                <InputNumber
                  style={{ width: "100%" }}
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
            </Col>
          </Row>
        </Card>

        <Card className="section-card" title="应用更新">
          <Row gutter={24}>
            <Col xs={24} md={8}>
              <Form.Item label="当前版本">
                <Input value={appVersion} disabled />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="启动时自动检查更新">
                <Switch
                  checked={config.updater.auto_check_on_startup}
                  onChange={(checked) =>
                    setConfig({
                      ...config,
                      updater: {
                        ...config.updater,
                        auto_check_on_startup: checked
                      }
                    })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label="更新代理（HTTP）">
                <Switch
                  checked={config.updater.proxy_enabled}
                  onChange={(checked) =>
                    setConfig({
                      ...config,
                      updater: {
                        ...config.updater,
                        proxy_enabled: checked
                      }
                    })
                  }
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={16}>
              <Form.Item
                label="代理地址"
                validateStatus={hasValidProxy ? undefined : "error"}
                help={hasValidProxy ? "示例：http://user:pass@127.0.0.1:7890" : "仅支持 http:// 开头的代理地址"}
              >
                <Input
                  placeholder="http://user:pass@127.0.0.1:7890"
                  value={config.updater.proxy_url_encrypted}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      updater: {
                        ...config.updater,
                        proxy_url_encrypted: e.target.value
                      }
                    })
                  }
                  disabled={!config.updater.proxy_enabled}
                />
              </Form.Item>
            </Col>
            <Col xs={24} md={8}>
              <Form.Item label=" " colon={false}>
                <Button
                  block
                  onClick={onCheckUpdate}
                  loading={checkingUpdate || installingUpdate}
                  disabled={config.updater.proxy_enabled && !hasValidProxy}
                >
                  检查更新
                </Button>
              </Form.Item>
            </Col>
          </Row>
          {updateProgress ? (
            <Space direction="vertical" size={6} style={{ width: "100%" }}>
              <Typography.Text type="secondary">
                {updateProgress.phase === "downloading"
                  ? "正在下载更新包..."
                  : "更新包下载完成，正在安装..."}
              </Typography.Text>
              <Progress
                percent={progressPercent}
                status={updateProgress.phase === "installing" ? "active" : "normal"}
                showInfo={progressPercent !== undefined}
              />
            </Space>
          ) : null}
        </Card>

        {/* Action Buttons */}
        <Row justify="end">
          <Space>
            <Button disabled={!canTestLlm} loading={testingLlm} onClick={onTestLlm}>
              测试模型连通性
            </Button>
            <Button disabled={!canTestMineru} loading={testingMineru} onClick={onTestMineru}>
              测试 MinerU 连通性
            </Button>
            <Button type="primary" loading={savingConfig} onClick={onSave} style={{ minWidth: 100 }}>
              保存设置
            </Button>
          </Space>
        </Row>
      </Space>
    </Form>
  );
}
