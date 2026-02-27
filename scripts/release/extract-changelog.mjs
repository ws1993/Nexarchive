import fs from "node:fs";
import path from "node:path";

const rawVersion = process.argv[2] || process.env.GITHUB_REF_NAME || "";
const version = rawVersion.replace(/^v/, "").trim();

if (!version) {
  console.error(
    "Usage: node scripts/release/extract-changelog.mjs <tag-or-version>\n" +
      "Example: node scripts/release/extract-changelog.mjs v0.2.0"
  );
  process.exit(1);
}

const changelogPath = path.join(process.cwd(), "CHANGELOG.md");
if (!fs.existsSync(changelogPath)) {
  console.error("CHANGELOG.md not found");
  process.exit(1);
}

const content = fs.readFileSync(changelogPath, "utf8");
const lines = content.split(/\r?\n/);

const heading = `## [${version}] - `;
const startIndex = lines.findIndex((line) => line.startsWith(heading));
if (startIndex < 0) {
  console.error(`Cannot find CHANGELOG section for version ${version}`);
  process.exit(1);
}

let endIndex = lines.length;
for (let i = startIndex + 1; i < lines.length; i += 1) {
  if (lines[i].startsWith("## [")) {
    endIndex = i;
    break;
  }
}

const section = lines.slice(startIndex, endIndex).join("\n").trim();
process.stdout.write(section);
