(function() {

    let nav = document.querySelectorAll(".nav")[0];
    let panes = document.querySelectorAll(".panes")[0];
    let tabChanged = new Event("tab-changed");

    function TabPane(tabEl) {

        let selector = tabEl.getAttribute("clickable-tab");
        let paneEl = document.querySelector("[tab-pane='" + selector + "']");

        let self = this;
        self.tab = tabEl;
        self.pane = paneEl;

        self.tab.addEventListener("click", function() {
            hideAllPanes();
            self.pane.classList.remove("hidden");
            self.tab.classList.add("active");
            window.dispatchEvent(tabChanged);
        });

        self.hide = function() {
            self.pane.classList.add("hidden");
            self.tab.classList.remove("active");
        }
    }

    let tabPaneEls = Array.from(document.querySelectorAll("[clickable-tab]"));

    let tabPanes = tabPaneEls.map((e) => {
        return new TabPane(e);
    });

    function hideAllPanes() {
        tabPanes.map((tp) => {
            tp.hide()
        });
    }

})()
