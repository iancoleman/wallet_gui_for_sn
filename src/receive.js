(function() {

    let addressEl, qrEl;

    async function getAddress(name) {
        let address = await invoke("get_address", {
            walletName: name,
        });
        addressEl.textContent = address;
        qrEl.innerHTML = "";
        new QRCode(qrEl, address);
    }

    window.addEventListener("DOMContentLoaded", () => {
        addressEl = document.querySelector(".receive .address");
        qrEl = document.querySelector(".receive .qrcode");
    });

    window.addEventListener("wallet-loaded", (e) => {
        let wallet = e.detail;
        // show the current receive address
        getAddress(wallet.name);
        // TODO
        // show the balance
        // show the tx history
    });

})()
