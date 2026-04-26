// musubi WebUI — wires the WASM module to the DOM.
//
// All cipher operations run locally in the browser via the
// `musubi-wasm` crate; no network requests are made.

import init, { keygen, encrypt, decrypt } from "./pkg/musubi_wasm.js";

await init();

// ---- Tab switching ----------------------------------------------------------

const tabs = document.querySelectorAll(".tabs button");
const panels = document.querySelectorAll(".panel");
tabs.forEach((btn) => {
  btn.addEventListener("click", () => {
    tabs.forEach((b) => b.classList.remove("active"));
    panels.forEach((p) => p.classList.remove("active"));
    btn.classList.add("active");
    document.getElementById("panel-" + btn.dataset.tab).classList.add("active");
  });
});

// ---- Status ribbon ----------------------------------------------------------

const status = document.getElementById("status");
let statusTimer = null;
function setStatus(message, kind = "info") {
  status.textContent = message;
  status.className = kind;
  status.hidden = false;
  if (statusTimer) clearTimeout(statusTimer);
  statusTimer = setTimeout(() => {
    status.hidden = true;
  }, 4000);
}

// ---- Helpers ----------------------------------------------------------------

function download(content, filename, mime) {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

async function copyValue(elementId) {
  const v = document.getElementById(elementId).value;
  if (!v) {
    setStatus("コピーする内容がありません", "error");
    return;
  }
  await navigator.clipboard.writeText(v);
  setStatus("クリップボードにコピーしました", "success");
}

// ---- Keygen ----------------------------------------------------------------

document.getElementById("btn-keygen").addEventListener("click", () => {
  try {
    const key = keygen();
    document.getElementById("key-output").value = key;
    setStatus("鍵を生成しました", "success");
  } catch (e) {
    setStatus("鍵生成に失敗: " + e.message, "error");
  }
});

document
  .getElementById("btn-copy-key")
  .addEventListener("click", () => copyValue("key-output"));

document.getElementById("btn-download-key").addEventListener("click", () => {
  const v = document.getElementById("key-output").value;
  if (!v) {
    setStatus("先に鍵を生成してください", "error");
    return;
  }
  download(v, "musubi-key.json", "application/json");
});

// ---- Encrypt ---------------------------------------------------------------

document.getElementById("btn-encrypt").addEventListener("click", () => {
  try {
    const key = document.getElementById("enc-key").value.trim();
    const plain = document.getElementById("enc-plain").value;
    if (!key) {
      setStatus("鍵が入力されていません", "error");
      return;
    }
    if (!plain) {
      setStatus("平文が入力されていません", "error");
      return;
    }
    const anchorStr = document.getElementById("enc-anchor").value;
    const anchor = anchorStr === "" ? undefined : Number(anchorStr);
    const cipher = encrypt(plain, key, anchor);
    document.getElementById("enc-output").value = cipher;
    setStatus("暗号化しました", "success");
  } catch (e) {
    setStatus("暗号化に失敗: " + e.message, "error");
  }
});

document
  .getElementById("btn-copy-cipher")
  .addEventListener("click", () => copyValue("enc-output"));

document.getElementById("btn-download-cipher").addEventListener("click", () => {
  const v = document.getElementById("enc-output").value;
  if (!v) {
    setStatus("先に暗号化してください", "error");
    return;
  }
  download(v, "musubi-cipher.json", "application/json");
});

// ---- Decrypt ---------------------------------------------------------------

document.getElementById("btn-decrypt").addEventListener("click", () => {
  try {
    const key = document.getElementById("dec-key").value.trim();
    const cipher = document.getElementById("dec-cipher").value.trim();
    if (!key) {
      setStatus("鍵が入力されていません", "error");
      return;
    }
    if (!cipher) {
      setStatus("暗号文が入力されていません", "error");
      return;
    }
    const plain = decrypt(cipher, key);
    document.getElementById("dec-output").value = plain;
    setStatus("復号しました", "success");
  } catch (e) {
    setStatus("復号に失敗: " + e.message, "error");
  }
});
