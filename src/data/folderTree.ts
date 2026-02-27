import type { InitPreviewItem } from "../types";

export const folderTreeData: InitPreviewItem[] = [
  {
    code: "10",
    folder: "10_身份基石",
    children: [
      { code: "11", folder: "11_法律证件" },
      { code: "12", folder: "12_教育背景" },
      { code: "13", folder: "13_职业履历" },
      { code: "14", folder: "14_健康档案" },
      { code: "15", folder: "15_财务信用" },
      { code: "16", folder: "16_社会关系" }
    ]
  },
  {
    code: "20",
    folder: "20_责任领域",
    children: [
      { code: "21", folder: "21_财务管理" },
      { code: "22", folder: "22_健康管理" },
      { code: "23", folder: "23_居住管理" },
      { code: "24", folder: "24_职业发展" }
    ]
  },
  {
    code: "30",
    folder: "30_行动项目",
    children: [
      { code: "31", folder: "31_工作项目" },
      { code: "32", folder: "32_个人项目" }
    ]
  },
  {
    code: "40",
    folder: "40_知识金库",
    children: [
      { code: "41", folder: "41_知识库" },
      { code: "42", folder: "42_资料库" },
      { code: "43", folder: "43_模板" }
    ]
  },
  {
    code: "50",
    folder: "50_数字资产",
    children: [
      { code: "51", folder: "51_媒体素材" },
      { code: "52", folder: "52_创作产出" },
      { code: "53", folder: "53_软件资源" }
    ]
  },
  { code: "99", folder: "99_历史档案" }
];
