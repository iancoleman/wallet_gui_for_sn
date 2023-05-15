const { invoke } = window.__TAURI__.tauri;

let addressEl;

async function getAddress() {
  //greetMsgEl.textContent = await invoke("get_address", { _wallet_name: greetInputEl.value });
  addressEl.textContent = await invoke("get_address");
}

window.addEventListener("DOMContentLoaded", () => {
  addressEl = document.querySelector("#address");
  getAddress();
});
