import fs from "node:fs";
import path from "node:path";

const inputVersion = process.argv[2]?.trim();
if (!inputVersion) {
  console.error("Usage: node scripts/release/prepare-version.mjs <version>");
  process.exit(1);
}

const semverPattern =
  /^\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;
if (!semverPattern.test(inputVersion)) {
  console.error(`Invalid semver version: ${inputVersion}`);
  process.exit(1);
}

const rootDir = process.cwd();
const packageJsonPath = path.join(rootDir, "package.json");
const packageLockPath = path.join(rootDir, "package-lock.json");
const cargoTomlPath = path.join(rootDir, "src-tauri", "Cargo.toml");
const tauriConfPath = path.join(rootDir, "src-tauri", "tauri.conf.json");

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
packageJson.version = inputVersion;
fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`, "utf8");

const packageLock = JSON.parse(fs.readFileSync(packageLockPath, "utf8"));
packageLock.version = inputVersion;
if (packageLock.packages && packageLock.packages[""]) {
  packageLock.packages[""].version = inputVersion;
}
fs.writeFileSync(packageLockPath, `${JSON.stringify(packageLock, null, 2)}\n`, "utf8");

const cargoTomlRaw = fs.readFileSync(cargoTomlPath, "utf8");
const cargoVersionRegex = /(\[package\][\s\S]*?\nversion\s*=\s*")[^"]+(")/;
if (!cargoVersionRegex.test(cargoTomlRaw)) {
  console.error("Failed to find package version in src-tauri/Cargo.toml");
  process.exit(1);
}
const cargoTomlNext = cargoTomlRaw.replace(
  cargoVersionRegex,
  `$1${inputVersion}$2`
);
fs.writeFileSync(cargoTomlPath, cargoTomlNext, "utf8");

const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, "utf8"));
tauriConf.version = inputVersion;
fs.writeFileSync(tauriConfPath, `${JSON.stringify(tauriConf, null, 2)}\n`, "utf8");

console.log(`Version synchronized to ${inputVersion}`);
