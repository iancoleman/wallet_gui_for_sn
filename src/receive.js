(function() {

    let addressEl;

    async function getAddress() {
      let address = await invoke("get_address", {
          walletName: "default_wallet",
      });
      addressEl.textContent = address;
      new QRCode(document.getElementById("qrcode"), address);
    }

    window.addEventListener("DOMContentLoaded", () => {
      addressEl = document.querySelector("#address");
      getAddress();
    });

})()
