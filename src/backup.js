(function() {

    let nameEl, mnemonicEl, passwordEl, decryptEl;

    async function getMnemonic() {
        // clear any existing mnemonic
        mnemonicEl.innerHTML = "";
        // get the mnemonic from the rust backend
        let mnemonic = await invoke("get_mnemonic", {
            walletName: nameEl.textContent,
            decryptor: passwordEl.value,
        });
        // show the mnemonic
        mnemonicEl.textContent = mnemonic;
    }

    function clearMnemonic() {
        mnemonicEl.textContent = "";
    }

    window.addEventListener("DOMContentLoaded", () => {
      nameEl = document.querySelector(".backup .name");
      mnemonicEl = document.querySelector(".backup .mnemonic");
      passwordEl = document.querySelector(".backup .password");
      decryptEl = document.querySelector(".backup .decrypt");
      decryptEl.addEventListener("click", getMnemonic);
    });

    window.addEventListener("tab-changed", clearMnemonic);

    window.addEventListener("wallet-loaded", (e) => {
        let wallet = e.detail;
        nameEl.textContent = wallet.name;
    });

})()
