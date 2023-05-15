const { invoke } = window.__TAURI__.tauri;

let addressEl;

async function getAddress() {
  //greetMsgEl.textContent = await invoke("get_address", { _wallet_name: greetInputEl.value });
  let address = await invoke("get_address");
  addressEl.textContent = address;
  new QRCode(document.getElementById("qrcode"), address);
}

window.addEventListener("DOMContentLoaded", () => {
  addressEl = document.querySelector("#address");
  getAddress();
});
