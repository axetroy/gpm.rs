const path = require("path");
const fs = require("fs");
const crypto = require("crypto");

const inputRef = process.env.GIT_REF;
let formulaRepoDir = process.env.FORMULA_REPO_DIR;

if (!inputRef) {
  throw new Error("env 'GIT_REF' required");
}

if (!formulaRepoDir) {
  throw new Error("env 'FORMULA_REPO_DIR' required");
}

if (!path.isAbsolute(formulaRepoDir)) {
  formulaRepoDir = path.join(process.cwd(), formulaRepoDir);
}

const version = inputRef.replace(/^refs\/tags\/v?/, "");

const rbFile = path.join(formulaRepoDir, "Formula", "gpm.rs.rb");

const formulaContent = fs.readFileSync(rbFile, { encoding: "utf-8" });

let newContent = formulaContent.replace(
  /version\s"[\w\.]+"/g,
  `version "${version}"`
);

const fileBuffer = fs.readFileSync(
  path.join(
    __dirname,
    "..",
    "target/x86_64-apple-darwin/release/gpm_darwin_amd64.tar.gz"
  )
);
const hashSum = crypto.createHash("sha256");
hashSum.update(fileBuffer);

const sha256 = hashSum.digest("hex");

newContent = formulaContent.replace(/sha256\s"[\w]+"/g, `sha256 "${sha256}"`);

fs.writeFileSync(rbFile, newContent, { encoding: "utf-8" });
