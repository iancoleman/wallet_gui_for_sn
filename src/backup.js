(function() {

    let nameEl, mnemonicEl, passwordEl, decryptEl;

    async function getMnemonic() {
        // clear any existing mnemonic
        mnemonicEl.innerHTML = "";
        // get the mnemonic from the rust backend
        let mnemonic = await invoke("get_mnemonic", {
            walletName: nameEl.value,
            decryptor: passwordEl.value,
        });
        // show the mnemonic
        mnemonicEl.textContent = mnemonic;
    }

    function clearMnemonic() {
        mnemonicEl.textContent = "";
    }

    window.addEventListener("DOMContentLoaded", () => {
      nameEl = document.getElementById("wallet-name");
      mnemonicEl = document.getElementById("mnemonic");
      passwordEl = document.getElementById("save-password");
      decryptEl = document.getElementById("decrypt");
      decryptEl.addEventListener("click", getMnemonic);
    });

    window.addEventListener("tab-changed", clearMnemonic);

})()
