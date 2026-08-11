/*! Load Mermaid for fenced ```mermaid blocks in mdBook. */
(function () {
  function convertMermaidBlocks() {
    document.querySelectorAll("code.language-mermaid").forEach(function (code) {
      var pre = code.parentElement;
      if (!pre || pre.tagName !== "PRE") return;
      var div = document.createElement("div");
      div.className = "mermaid";
      div.textContent = code.textContent;
      pre.replaceWith(div);
    });
  }

  function boot() {
    convertMermaidBlocks();
    if (window.mermaid) {
      window.mermaid.initialize({
        startOnLoad: true,
        theme: document.documentElement.classList.contains("navy") ||
          document.documentElement.classList.contains("coal") ||
          document.documentElement.classList.contains("ayu")
          ? "dark"
          : "neutral",
        securityLevel: "strict",
        fontFamily: "ui-sans-serif, system-ui, sans-serif",
      });
      window.mermaid.init(undefined, document.querySelectorAll(".mermaid"));
    }
  }

  var script = document.createElement("script");
  script.src = "https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js";
  script.onload = boot;
  document.head.appendChild(script);
})();
