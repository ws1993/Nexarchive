import { useEffect, useMemo, useState } from "react";
import { App as AntApp, Button, Card, Input, Space, Table, Tag, Typography } from "antd";
import { api } from "../api";
import type { FileTaskRecord, JobRecord, LogEvent } from "../types";

export function JobsPage() {
  const { message } = AntApp.useApp();
  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [tasks, setTasks] = useState<FileTaskRecord[]>([]);
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [selectedJob, setSelectedJob] = useState<string>();
  const [loadingJobs, setLoadingJobs] = useState(false);
  const [logQuery, setLogQuery] = useState("");

  const refreshJobs = async () => {
    setLoadingJobs(true);
    try {
      const data = await api.getJobs(1, 30);
      setJobs(data.items);
      if (!selectedJob && data.items.length > 0) {
        setSelectedJob(data.items[0].job_id);
      }
    } finally {
      setLoadingJobs(false);
    }
  };

  useEffect(() => {
    void refreshJobs();
  }, []);

  useEffect(() => {
    if (!selectedJob) return;
    void api.getFileTasks(selectedJob).then(setTasks);
    void api
      .getLogs({ page: 1, page_size: 100, job_id: selectedJob, query: logQuery })
      .then((v) => setLogs(v.items));
  }, [selectedJob, logQuery]);

  const taskFailures = useMemo(
    () => tasks.filter((v) => v.archive_status === "failed" || v.classify_status === "failed"),
    [tasks]
  );

  const onRestore = async (taskId: string) => {
    await api.restoreFromRecycleBin(taskId);
    message.success("已恢复到原路径");
  };

  return (
    <Space direction="vertical" style={{ width: "100%" }} size={16}>
      <Card
        className="section-card"
        title="任务列表"
        extra={
          <Button onClick={refreshJobs} loading={loadingJobs}>
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
            { title: "Job ID", dataIndex: "job_id", width: 260 },
            { title: "触发", dataIndex: "trigger_type", width: 100 },
            {
              title: "状态",
              dataIndex: "status",
              width: 110,
              render: (v: string) => <Tag color={v === "success" ? "green" : "orange"}>{v}</Tag>
            },
            { title: "开始时间", dataIndex: "start_at", width: 180 },
            { title: "摘要", dataIndex: "summary" }
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
            { title: "提取", dataIndex: "extract_status", width: 90 },
            { title: "分类", dataIndex: "classify_status", width: 90 },
            { title: "归档", dataIndex: "archive_status", width: 90 },
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
            { title: "时间", dataIndex: "timestamp", width: 180 },
            {
              title: "级别",
              dataIndex: "level",
              width: 90,
              render: (v: string) => <Tag color={v === "ERROR" ? "red" : "blue"}>{v}</Tag>
            },
            { title: "阶段", dataIndex: "stage", width: 110 },
            { title: "消息", dataIndex: "message" }
          ]}
        />
      </Card>
    </Space>
  );
}
