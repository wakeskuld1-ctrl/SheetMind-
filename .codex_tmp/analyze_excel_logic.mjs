import fs from "node:fs/promises";
import path from "node:path";
import { FileBlob, SpreadsheetFile } from "@oai/artifact-tool";

const inputPath = process.argv[2];
const outputPath = process.argv[3];

if (!inputPath || !outputPath) {
  throw new Error("Usage: node analyze_excel_logic.mjs <inputPath> <outputPath>");
}

async function main() {
  const input = await FileBlob.load(inputPath);
  const workbook = await SpreadsheetFile.importXlsx(input);

  const summary = await workbook.inspect({
    kind: "workbook,sheet,table,definedName,drawing",
    maxChars: 20000,
    tableMaxRows: 20,
    tableMaxCols: 20,
    tableMaxCellChars: 120,
  });

  const sheetInfo = await workbook.inspect({
    kind: "sheet",
    include: "id,name",
    maxChars: 8000,
  });

  const result = {
    inputPath,
    summary: summary.ndjson,
    sheets: [],
    sheetInfo: sheetInfo.ndjson,
  };

  const sheetLines = sheetInfo.ndjson
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  for (const line of sheetLines) {
    let parsed;
    try {
      parsed = JSON.parse(line);
    } catch {
      continue;
    }

    if (!parsed?.name) {
      continue;
    }

    const region = await workbook.inspect({
      kind: "region,table",
      sheetId: parsed.name,
      range: "A1:Z80",
      maxChars: 15000,
      tableMaxRows: 80,
      tableMaxCols: 26,
      tableMaxCellChars: 120,
    });

    const formulas = await workbook.inspect({
      kind: "formula",
      sheetId: parsed.name,
      range: "A1:Z120",
      maxChars: 15000,
      options: { maxResults: 300 },
    });

    result.sheets.push({
      name: parsed.name,
      region: region.ndjson,
      formulas: formulas.ndjson,
    });
  }

  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(outputPath, JSON.stringify(result, null, 2), "utf8");
}

await main();
