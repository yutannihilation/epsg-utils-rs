import init, { epsg_to_wkt2, epsg_to_projjson, wkt2_to_projjson } from "./pkg/epsg_utils_web.js";

const epsgCode = document.getElementById("epsg-code");
const outputFormat = document.getElementById("output-format");
const lookupBtn = document.getElementById("lookup-btn");
const lookupOutput = document.getElementById("lookup-output");

const wkt2Input = document.getElementById("wkt2-input");
const convertBtn = document.getElementById("convert-btn");
const convertOutput = document.getElementById("convert-output");

const status = document.getElementById("status");

async function run() {
  await init();

  epsgCode.disabled = false;
  outputFormat.disabled = false;
  lookupBtn.disabled = false;
  wkt2Input.disabled = false;
  convertBtn.disabled = false;
  status.textContent = "Ready.";

  lookupBtn.addEventListener("click", () => {
    const code = parseInt(epsgCode.value, 10);
    if (isNaN(code)) {
      lookupOutput.textContent = "Please enter a valid EPSG code.";
      return;
    }
    try {
      const fmt = outputFormat.value;
      const result = fmt === "wkt2" ? epsg_to_wkt2(code) : epsg_to_projjson(code);
      lookupOutput.textContent = fmt === "projjson"
        ? JSON.stringify(JSON.parse(result), null, 2)
        : result;
    } catch (e) {
      lookupOutput.textContent = "Error: " + e;
    }
  });

  convertBtn.addEventListener("click", () => {
    const input = wkt2Input.value.trim();
    if (!input) {
      convertOutput.textContent = "Please paste a WKT2 string above.";
      return;
    }
    try {
      convertOutput.textContent = wkt2_to_projjson(input);
    } catch (e) {
      convertOutput.textContent = "Error: " + e;
    }
  });
}

run();
