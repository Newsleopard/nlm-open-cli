#!/usr/bin/env node

"use strict";

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");
const http = require("http");

const VERSION = require("./package.json").version;
const REPO = "Newsleopard/nlm-open-cli";
const BIN_DIR = path.join(__dirname, "bin");

const PLATFORM_MAP = {
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "win32-x64": "x86_64-pc-windows-msvc",
};

function getTarget() {
  const key = `${process.platform}-${process.arch}`;
  const target = PLATFORM_MAP[key];
  if (!target) {
    console.error(
      `Error: Unsupported platform ${process.platform}-${process.arch}.\n` +
        `Supported: ${Object.keys(PLATFORM_MAP).join(", ")}\n` +
        `You can build from source: https://github.com/${REPO}#build-from-source`
    );
    process.exit(1);
  }
  return target;
}

function getArchiveInfo(target) {
  const isWindows = process.platform === "win32";
  const ext = isWindows ? "zip" : "tar.gz";
  const name = `nlm-v${VERSION}-${target}.${ext}`;
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${name}`;
  return { name, url, ext, isWindows };
}

function download(url) {
  return new Promise((resolve, reject) => {
    const get = url.startsWith("https:") ? https.get : http.get;
    get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return download(res.headers.location).then(resolve, reject);
      }
      if (res.statusCode !== 200) {
        reject(new Error(`Download failed: HTTP ${res.statusCode} for ${url}`));
        return;
      }
      const chunks = [];
      res.on("data", (chunk) => chunks.push(chunk));
      res.on("end", () => resolve(Buffer.concat(chunks)));
      res.on("error", reject);
    }).on("error", reject);
  });
}

function extractTarGz(buffer, destDir) {
  const tmpArchive = path.join(destDir, "archive.tar.gz");
  fs.writeFileSync(tmpArchive, buffer);
  execSync(`tar xzf "${tmpArchive}" -C "${destDir}"`, { stdio: "ignore" });
  fs.unlinkSync(tmpArchive);
}

function extractZip(buffer, destDir) {
  const tmpArchive = path.join(destDir, "archive.zip");
  fs.writeFileSync(tmpArchive, buffer);
  if (process.platform === "win32") {
    execSync(
      `powershell -Command "Expand-Archive -Path '${tmpArchive}' -DestinationPath '${destDir}' -Force"`,
      { stdio: "ignore" }
    );
  } else {
    execSync(`unzip -o "${tmpArchive}" -d "${destDir}"`, { stdio: "ignore" });
  }
  fs.unlinkSync(tmpArchive);
}

async function main() {
  const target = getTarget();
  const { url, ext, isWindows } = getArchiveInfo(target);
  // Download as "nlm-binary" so it doesn't conflict with the Node.js wrapper "nlm"
  const extractedName = isWindows ? "nlm.exe" : "nlm";
  const finalName = isWindows ? "nlm-binary.exe" : "nlm-binary";
  const finalPath = path.join(BIN_DIR, finalName);

  // Skip if binary already exists (e.g., CI caching)
  if (fs.existsSync(finalPath)) {
    console.log(`nlm binary already exists at ${finalPath}, skipping download.`);
    return;
  }

  console.log(`Downloading nlm v${VERSION} for ${target}...`);
  console.log(`  ${url}`);

  const buffer = await download(url);

  // Extract to a temp directory to avoid overwriting the Node.js wrapper scripts in bin/
  const tmpDir = path.join(__dirname, ".tmp-extract");
  fs.mkdirSync(tmpDir, { recursive: true });

  if (ext === "zip") {
    extractZip(buffer, tmpDir);
  } else {
    extractTarGz(buffer, tmpDir);
  }

  // The archive contains "nlm"; move it to bin/ as "nlm-binary" to avoid conflicting
  // with the Node.js wrapper script that npm links as the "nlm" command
  const extractedPath = path.join(tmpDir, extractedName);
  if (!fs.existsSync(extractedPath)) {
    console.error(`Error: Archive did not contain expected "${extractedName}" binary.`);
    // Clean up temp dir
    fs.rmSync(tmpDir, { recursive: true, force: true });
    process.exit(1);
  }

  fs.mkdirSync(BIN_DIR, { recursive: true });
  fs.renameSync(extractedPath, finalPath);

  // Clean up temp directory
  fs.rmSync(tmpDir, { recursive: true, force: true });

  // Set executable permissions on Unix
  if (!isWindows) {
    fs.chmodSync(finalPath, 0o755);
  }

  console.log(`Installed nlm v${VERSION} to ${finalPath}`);
}

main().catch((err) => {
  console.error(`Failed to install nlm: ${err.message}`);
  process.exit(1);
});
