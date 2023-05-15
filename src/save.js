(function() {

    let mnemonicEl;

    async function getMnemonic() {
        // clear any existing mnemonic
        mnemonicEl.innerHTML = "";
        // get the mnemonic from the rust backend
        let mnemonic = await invoke("get_mnemonic", {
            walletName: "default_wallet",
        });
        // show the mnemonic
        mnemonicEl.textContent = mnemonic;
    }

    window.addEventListener("DOMContentLoaded", () => {
      mnemonicEl = document.querySelector("#mnemonic");
      getMnemonic();
    });

})()
