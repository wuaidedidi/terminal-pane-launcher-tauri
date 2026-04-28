import { spawnSync } from "node:child_process";
import process from "node:process";

const install = process.argv.includes("--install");

function run(command, args) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: false,
  });

  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }

  process.exit(result.status ?? 1);
}

if (process.platform === "win32") {
  run("powershell", [
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    "scripts\\check-env.ps1",
    ...(install ? ["-Install"] : []),
  ]);
}

if (process.platform === "darwin") {
  run("bash", [
    "scripts/check-env.sh",
    ...(install ? ["--install"] : []),
  ]);
}

console.error(`Unsupported platform for this launcher: ${process.platform}`);
process.exit(1);
