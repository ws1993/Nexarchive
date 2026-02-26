import { useCallback, useEffect, useMemo, useState } from "react";
import { App as AntApp, Button, Card, Input, Space, Table, Tag, Typography } from "antd";
import { api } from "../api";
import { useAppStore } from "../store";
import type { FileTaskRecord, JobRecord, LogEvent } from "../types";

const beijingTimeFormatter = new Intl.DateTimeFormat("zh-CN", {
  timeZone: "Asia/Shanghai",
  hour12: false,
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit"
});

const statusTextMap: Record<string, string> = {
  running: "执行中",
  success: "成功",
  partial: "部分失败",
  failed: "失败",
  pending: "待处理",
  skipped: "已跳过",
  review: "待复核",
  执行中: "执行中",
  成功: "成功",
  部分失败: "部分失败",
  失败: "失败",
  待处理: "待处理",
  已跳过: "已跳过",
  待复核: "待复核"
};

const triggerTextMap: Record<string, string> = {
  manual: "手动",
  schedule: "定时"
};

const levelTextMap: Record<string, string> = {
  INFO: "信息",
  WARN: "警告",
  ERROR: "错误"
};

const stageTextMap: Record<string, string> = {
  init: "初始化",
  settings: "设置",
  llm: "模型",
  mineru: "MinerU",
  recycle: "回收区",
  job: "任务",
  pipeline: "流程",
  dedupe: "去重",
  classify: "分类",
  archive: "归档",
  extract: "提取",
  初始化: "初始化",
  设置: "设置",
  模型: "模型",
  回收区: "回收区",
  任务: "任务",
  流程: "流程",
  去重: "去重",
  分类: "分类",
  归档: "归档",
  提取: "提取"
};

const logMessageMap: Record<string, string> = {
  "system structure initialized": "目录结构初始化完成",
  "settings saved": "设置已保存",
  "connection test success": "连通性测试成功",
  "file restored from recycle": "文件已从回收区恢复",
  "job started": "任务开始执行",
  "unexpected processing error": "处理流程出现未预期错误",
  "job finished": "任务执行完成",
  "duplicate file skipped": "检测到重复文件，已跳过",
  "low confidence moved to review": "置信度较低，已移入复核目录",
  "source move to recycle failed; kept original": "移动到回收区失败，已保留原文件",
  "file archived": "文件归档完成",
  "mineru extract success": "MinerU 提取成功",
  "mineru returned empty text; fallback to local extractor": "MinerU 返回空文本，已回退本地解析",
  "mineru extract failed; fallback to local extractor": "MinerU 提取失败，已回退本地解析"
};

const formatBeijingTime = (value: string) => {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return beijingTimeFormatter.format(parsed).replace(/\//g, "-");
};

const toChineseStatus = (value: string) => statusTextMap[value] ?? value;
const toChineseTrigger = (value: string) => triggerTextMap[value] ?? value;
const toChineseLevel = (value: string) => levelTextMap[value] ?? value;
const toChineseStage = (value: string) => stageTextMap[value] ?? value;
const toChineseMessage = (value: string) => logMessageMap[value] ?? value;

const toChineseSummary = (value: string) =>
  value
    .replace(/running/g, "执行中")
    .replace(/success=/g, "成功=")
    .replace(/review=/g, "待复核=")
    .replace(/skipped=/g, "已跳过=")
    .replace(/failed=/g, "失败=");

const isRunningStatus = (value?: string) => value === "running" || value === "执行中";
const isFailedStatus = (value: string) => value === "failed" || value === "失败";

const statusColor = (value: string) => {
  if (value === "success" || value === "成功") return "green";
  if (value === "partial" || value === "部分失败") return "orange";
  if (value === "failed" || value === "失败") return "red";
  if (value === "running" || value === "执行中") return "blue";
  if (value === "review" || value === "待复核") return "gold";
  if (value === "skipped" || value === "已跳过") return "default";
  return "default";
};

export function JobsPage() {
  const { message } = AntApp.useApp();
  const lastRunJobId = useAppStore((s) => s.lastRunJobId);

  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [tasks, setTasks] = useState<FileTaskRecord[]>([]);
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [selectedJob, setSelectedJob] = useState<string>();
  const [loadingJobs, setLoadingJobs] = useState(false);
  const [logQuery, setLogQuery] = useState("");

  const refreshJobDetails = useCallback(async (jobId: string, query: string) => {
    const [taskRows, logRows] = await Promise.all([
      api.getFileTasks(jobId),
      api.getLogs({ page: 1, page_size: 100, job_id: jobId, query })
    ]);
    setTasks(taskRows);
    setLogs(logRows.items);
  }, []);

  const refreshJobs = useCallback(
    async (showLoading = true) => {
      if (showLoading) {
        setLoadingJobs(true);
      }
      try {
        const data = await api.getJobs(1, 30);
        setJobs(data.items);
        setSelectedJob((current) => {
          if (current && data.items.some((item) => item.job_id === current)) {
            return current;
          }
          if (lastRunJobId && data.items.some((item) => item.job_id === lastRunJobId)) {
            return lastRunJobId;
          }
          return data.items[0]?.job_id;
        });
      } finally {
        if (showLoading) {
          setLoadingJobs(false);
        }
      }
    },
    [lastRunJobId]
  );

  useEffect(() => {
    void refreshJobs();
  }, [refreshJobs]);

  useEffect(() => {
    if (!selectedJob) {
      setTasks([]);
      setLogs([]);
      return;
    }
    void refreshJobDetails(selectedJob, logQuery);
  }, [selectedJob, logQuery, refreshJobDetails]);

  useEffect(() => {
    if (!lastRunJobId) {
      return;
    }
    setSelectedJob(lastRunJobId);
    void refreshJobs(false);
    void refreshJobDetails(lastRunJobId, logQuery);
  }, [lastRunJobId, logQuery, refreshJobs, refreshJobDetails]);

  const selectedJobStatus = useMemo(
    () => jobs.find((item) => item.job_id === selectedJob)?.status,
    [jobs, selectedJob]
  );

  useEffect(() => {
    const intervalMs = isRunningStatus(selectedJobStatus) ? 2000 : 8000;
    const timer = window.setInterval(() => {
      void refreshJobs(false);
      if (selectedJob) {
        void refreshJobDetails(selectedJob, logQuery);
      }
    }, intervalMs);
    return () => window.clearInterval(timer);
  }, [selectedJob, selectedJobStatus, logQuery, refreshJobs, refreshJobDetails]);

  const onRefresh = () => {
    void refreshJobs();
    if (selectedJob) {
      void refreshJobDetails(selectedJob, logQuery);
    }
  };

  const taskFailures = useMemo(
    () => tasks.filter((v) => isFailedStatus(v.archive_status) || isFailedStatus(v.classify_status)),
    [tasks]
  );

  const onRestore = async (taskId: string) => {
    await api.restoreFromRecycleBin(taskId);
    message.success("已恢复到原路径");
    if (selectedJob) {
      void refreshJobDetails(selectedJob, logQuery);
    }
  };

  return (
    <Space direction="vertical" style={{ width: "100%" }} size={16}>
      <Card
        className="section-card"
        title="任务列表"
        extra={
          <Button onClick={onRefresh} loading={loadingJobs}>
            刷新
          </Button>
        }
      >
        <Table
          rowKey="job_id"
          dataSource={jobs}
          pagination={false}
          onRow={(record) => ({ onClick: () => setSelectedJob(record.job_id) })}
          columns={[
            { title: "任务ID", dataIndex: "job_id", width: 210 },
            {
              title: "触发方式",
              dataIndex: "trigger_type",
              width: 100,
              render: (value: string) => toChineseTrigger(value)
            },
            {
              title: "状态",
              dataIndex: "status",
              width: 110,
              render: (value: string) => <Tag color={statusColor(value)}>{toChineseStatus(value)}</Tag>
            },
            {
              title: "开始时间(北京时间)",
              dataIndex: "start_at",
              width: 190,
              render: (value: string) => formatBeijingTime(value)
            },
            { title: "摘要", dataIndex: "summary", render: (value: string) => toChineseSummary(value) }
          ]}
        />
      </Card>

      <Card className="section-card" title="文件任务" extra={<Typography.Text>{selectedJob}</Typography.Text>}>
        <Table
          rowKey="task_id"
          dataSource={tasks}
          pagination={{ pageSize: 8 }}
          columns={[
            { title: "文件", dataIndex: "src_path", ellipsis: true },
            {
              title: "提取",
              dataIndex: "extract_status",
              width: 90,
              render: (value: string) => <Tag color={statusColor(value)}>{toChineseStatus(value)}</Tag>
            },
            {
              title: "分类",
              dataIndex: "classify_status",
              width: 90,
              render: (value: string) => <Tag color={statusColor(value)}>{toChineseStatus(value)}</Tag>
            },
            {
              title: "归档",
              dataIndex: "archive_status",
              width: 90,
              render: (value: string) => <Tag color={statusColor(value)}>{toChineseStatus(value)}</Tag>
            },
            { title: "目标路径", dataIndex: "final_path", ellipsis: true },
            {
              title: "恢复",
              width: 90,
              render: (_: unknown, row: FileTaskRecord) => (
                <Button
                  size="small"
                  disabled={!row.recycle_path}
                  onClick={() => void onRestore(row.task_id)}
                >
                  恢复
                </Button>
              )
            }
          ]}
        />
        {taskFailures.length > 0 && (
          <Typography.Text type="danger">失败条数: {taskFailures.length}</Typography.Text>
        )}
      </Card>

      <Card
        className="section-card"
        title="日志"
        extra={
          <Input.Search
            placeholder="搜索日志"
            allowClear
            style={{ width: 240 }}
            value={logQuery}
            onChange={(e) => setLogQuery(e.target.value)}
          />
        }
      >
        <Table
          rowKey={(r) => `${r.timestamp}_${r.stage}_${r.message}`}
          dataSource={logs}
          pagination={{ pageSize: 10 }}
          columns={[
            {
              title: "时间(北京时间)",
              dataIndex: "timestamp",
              width: 190,
              render: (value: string) => formatBeijingTime(value)
            },
            {
              title: "级别",
              dataIndex: "level",
              width: 90,
              render: (value: string) => (
                <Tag color={value === "ERROR" ? "red" : value === "WARN" ? "orange" : "blue"}>
                  {toChineseLevel(value)}
                </Tag>
              )
            },
            { title: "阶段", dataIndex: "stage", width: 110, render: (value: string) => toChineseStage(value) },
            { title: "消息", dataIndex: "message", render: (value: string) => toChineseMessage(value) }
          ]}
        />
      </Card>
    </Space>
  );
}
