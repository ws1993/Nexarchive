import fs from "node:fs";
import path from "node:path";

const rootDir = process.cwd();
const packageJsonPath = path.join(rootDir, "package.json");
const packageLockPath = path.join(rootDir, "package-lock.json");
const cargoTomlPath = path.join(rootDir, "src-tauri", "Cargo.toml");
const tauriConfPath = path.join(rootDir, "src-tauri", "tauri.conf.json");

function readCargoPackageVersion(content) {
  const match = content.match(/(?:^|\n)\[package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/);
  return match?.[1];
}

const packageJsonVersion = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")).version;
const packageLockJson = JSON.parse(fs.readFileSync(packageLockPath, "utf8"));
const packageLockVersion = packageLockJson.version;
const packageLockRootVersion = packageLockJson.packages?.[""]?.version;
const cargoTomlVersion = readCargoPackageVersion(fs.readFileSync(cargoTomlPath, "utf8"));
const tauriConfVersion = JSON.parse(fs.readFileSync(tauriConfPath, "utf8")).version;

const versionMap = {
  "package.json": packageJsonVersion,
  "package-lock.json(version)": packageLockVersion,
  "package-lock.json(packages[''])": packageLockRootVersion,
  "src-tauri/Cargo.toml": cargoTomlVersion,
  "src-tauri/tauri.conf.json": tauriConfVersion
};

const values = Object.values(versionMap);
const expectedVersion = process.argv[2]?.replace(/^v/, "") || packageJsonVersion;
const allMatch = values.every((value) => value === expectedVersion);

if (!allMatch) {
  console.error("Version mismatch detected:");
  for (const [file, version] of Object.entries(versionMap)) {
    console.error(`- ${file}: ${version ?? "<missing>"}`);
  }
  console.error(`Expected version: ${expectedVersion}`);
  process.exit(1);
}

console.log(`Version check passed: ${expectedVersion}`);
