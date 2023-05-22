let walletTemplate = document.querySelector("#existing-wallet-template").innerHTML;

Wallet = function(name) {

    let self = this;

    self.select = function() {
        radio.setAttribute("checked", true);
        self.onSelected();
    }

    self.onSelected = function() {
        // only do this if the change was to become selected,
        // ie don't do it if the change was to become unselected.
        if (!radio.checked) {
            return;
        }
        // TODO
        // load this wallet details from the backend
        let walletLoaded = new CustomEvent("wallet-loaded", { detail: self });
        window.dispatchEvent(walletLoaded);
    }

    self.name = name;
    self.el = document.createElement("div");
    self.el.innerHTML = walletTemplate;

    let nameEl = self.el.querySelector(".name");
    let radio = self.el.querySelector("[type='radio']");

    radio.addEventListener("change", self.onSelected);

    nameEl.textContent = name;

}
